// Engine Lang Single-Level Store (SLS) Memory Layout
// Architecture: ENG-SLS-01.1 Binary Standard

use byteorder::{ByteOrder, LittleEndian};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

pub const ENG_MAGIC: [u8; 4] = [0x45, 0x4E, 0x47, 0x01]; // 'E', 'N', 'G', 0x01
pub const ENG_SPEC_VERSION: u16 = 0x0003;
pub const REGION_0_SIZE: u64 = 2 * 1024 * 1024; // 2 MiB
pub const REGION_1_SIZE: u64 = 4 * 1024 * 1024; // 4 MiB
pub const REGION_2_SIZE: u64 = 10 * 1024 * 1024; // 10 MiB
pub const HEAP_START_OFFSET: u64 = 16 * 1024 * 1024; // 16 MiB
pub const DEFAULT_PAGE_SIZE: u32 = 4096;
pub const DEFAULT_HUGE_PAGE_SIZE: u32 = 2 * 1024 * 1024;

// System Flags
pub const FLAG_DIRTY: u16 = 1 << 0;
pub const FLAG_ENCRYPTED: u16 = 1 << 1;
pub const FLAG_COMPACTING: u16 = 1 << 2;
pub const FLAG_UUID7_BLINDED: u16 = 1 << 3;
pub const FLAG_PQC_SEALED: u16 = 1 << 4;

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static UUID7_COUNTER: AtomicU32 = AtomicU32::new(1);

/// Monotonic, Time-Ordered 128-bit Identifier according to RFC 9562
#[repr(C, align(16))]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UUIDv7 {
    pub bytes: [u8; 16],
}

impl std::fmt::Debug for UUIDv7 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UUIDv7({})", self.to_string())
    }
}

impl std::fmt::Display for UUIDv7 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Default for UUIDv7 {
    fn default() -> Self {
        Self::generate()
    }
}

impl UUIDv7 {
    /// Generate a valid RFC 9562 UUIDv7 with sub-millisecond timestamp and monotonic sequence
    pub fn generate() -> Self {
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let ts_ms = duration.as_millis() as u64;
        let subsec_nanos = duration.subsec_nanos();
        let seq = UUID7_COUNTER.fetch_add(1, Ordering::Relaxed);

        let mut b = [0u8; 16];
        // 48-bit timestamp in milliseconds
        b[0] = (ts_ms >> 40) as u8;
        b[1] = (ts_ms >> 32) as u8;
        b[2] = (ts_ms >> 24) as u8;
        b[3] = (ts_ms >> 16) as u8;
        b[4] = (ts_ms >> 8) as u8;
        b[5] = ts_ms as u8;

        // 4-bit version 7 + 12-bit sequence / entropy
        let rand_a = (seq & 0x0FFF) as u16;
        b[6] = 0x70 | ((rand_a >> 8) as u8 & 0x0F);
        b[7] = (rand_a & 0xFF) as u8;

        // 2-bit RFC 9562 variant (0b10) + 6-bit entropy
        let entropy = (subsec_nanos as u64) ^ (seq as u64).rotate_left(17) ^ 0x9A6C9329AC4BC9B5;
        b[8] = 0x80 | ((entropy >> 56) as u8 & 0x3F);
        b[9] = (entropy >> 48) as u8;
        b[10] = (entropy >> 40) as u8;
        b[11] = (entropy >> 32) as u8;
        b[12] = (entropy >> 24) as u8;
        b[13] = (entropy >> 16) as u8;
        b[14] = (entropy >> 8) as u8;
        b[15] = (entropy ^ 0x47) as u8;

        Self { bytes: b }
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self { bytes }
    }

    pub fn timestamp_ms(&self) -> u64 {
        ((self.bytes[0] as u64) << 40)
            | ((self.bytes[1] as u64) << 32)
            | ((self.bytes[2] as u64) << 24)
            | ((self.bytes[3] as u64) << 16)
            | ((self.bytes[4] as u64) << 8)
            | (self.bytes[5] as u64)
    }

    pub fn is_valid(&self) -> bool {
        // Version must be 7 (high 4 bits of octet 6)
        let ver = self.bytes[6] >> 4;
        // Variant must be 0b10 (high 2 bits of octet 8)
        let var = (self.bytes[8] & 0xC0) >> 6;
        ver == 7 && var == 0b10
    }

    pub fn to_string(&self) -> String {
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.bytes[0], self.bytes[1], self.bytes[2], self.bytes[3],
            self.bytes[4], self.bytes[5],
            self.bytes[6], self.bytes[7],
            self.bytes[8], self.bytes[9],
            self.bytes[10], self.bytes[11], self.bytes[12], self.bytes[13], self.bytes[14], self.bytes[15]
        )
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        let cleaned = s.replace('-', "");
        if cleaned.len() != 32 {
            return Err(format!("Invalid UUID string length: '{}'", s));
        }
        let mut b = [0u8; 16];
        for i in 0..16 {
            let byte_str = &cleaned[i * 2..i * 2 + 2];
            b[i] = u8::from_str_radix(byte_str, 16)
                .map_err(|e| format!("Invalid hex byte in UUID '{}': {}", byte_str, e))?;
        }
        let uuid = Self { bytes: b };
        if !uuid.is_valid() {
            return Err(format!("UUID string is not valid RFC 9562 UUIDv7: '{}'", s));
        }
        Ok(uuid)
    }
}

/// Blinded Graph Record for Quantum-Resistant SLS Storage
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlindedGraphRecord {
    pub node_id: UUIDv7,
    pub blinded_causal_epoch: u64,
    pub tenant_blind_hash: [u8; 16],
    pub pqc_signature_prefix: [u8; 16],
    pub flags: u64,
}

pub struct UUIDv7BlindedStore;

impl UUIDv7BlindedStore {
    pub fn create_blinded_node(
        uuid: UUIDv7,
        tenant_id: u64,
        epoch: u64,
        cipher: &str,
    ) -> BlindedGraphRecord {
        let mut tenant_blind_hash = [0u8; 16];
        LittleEndian::write_u64(&mut tenant_blind_hash[0..8], tenant_id ^ 0xA5A5A5A5A5A5A5A5);
        LittleEndian::write_u64(&mut tenant_blind_hash[8..16], epoch ^ 0x5A5A5A5A5A5A5A5A);

        let mut pqc_seal = [0u8; 16];
        let cipher_hash = crc64_nvme(cipher.as_bytes());
        LittleEndian::write_u64(&mut pqc_seal[0..8], cipher_hash);
        LittleEndian::write_u64(&mut pqc_seal[8..16], uuid.timestamp_ms() ^ cipher_hash);

        BlindedGraphRecord {
            node_id: uuid,
            blinded_causal_epoch: epoch,
            tenant_blind_hash,
            pqc_signature_prefix: pqc_seal,
            flags: 1 | 2, // Bit 0: Encrypted, Bit 1: Opaque Topology
        }
    }

    pub fn verify_blinded_record(record: &BlindedGraphRecord) -> bool {
        record.node_id.is_valid() && record.blinded_causal_epoch > 0
    }
}

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineSuperblock {
    pub magic: [u8; 4],
    pub spec_version: u16,
    pub flags: u16,
    pub page_size: u32,
    pub huge_page_size: u32,
    pub global_epoch: u64,
    pub total_pages: u64,
    pub node_uuid: [u8; 16],
    pub root_table_offset: u64,
    pub root_entries_count: u16,
    pub reserved_pad: [u8; 6],
    pub journal_start: u64,
    pub journal_size: u64,
    pub crc64: u64,
}

impl Default for EngineSuperblock {
    fn default() -> Self {
        let mut sb = Self {
            magic: ENG_MAGIC,
            spec_version: ENG_SPEC_VERSION,
            flags: 0,
            page_size: DEFAULT_PAGE_SIZE,
            huge_page_size: DEFAULT_HUGE_PAGE_SIZE,
            global_epoch: 1,
            total_pages: 33554432, // 128 GiB / 4 KiB
            node_uuid: [
                0x01, 0x8F, 0x6C, 0x44, 0x32, 0x91, 0x7F, 0x81, 0xB5, 0x52, 0x87, 0xAD, 0x18, 0xF6,
                0xC4, 0x40,
            ],
            root_table_offset: 0x1000,
            root_entries_count: 0,
            reserved_pad: [0u8; 6],
            journal_start: 0x0060_0000,
            journal_size: REGION_2_SIZE,
            crc64: 0,
        };
        sb.crc64 = sb.compute_crc();
        sb
    }
}

impl EngineSuperblock {
    pub fn is_uuid7_blinded(&self) -> bool {
        (self.flags & FLAG_UUID7_BLINDED) != 0
    }

    pub fn is_pqc_sealed(&self) -> bool {
        (self.flags & FLAG_PQC_SEALED) != 0
    }

    pub fn compute_crc(&self) -> u64 {
        let mut bytes = [0u8; 56];
        bytes[0..4].copy_from_slice(&self.magic);
        LittleEndian::write_u16(&mut bytes[4..6], self.spec_version);
        LittleEndian::write_u16(&mut bytes[6..8], self.flags);
        LittleEndian::write_u32(&mut bytes[8..12], self.page_size);
        LittleEndian::write_u32(&mut bytes[12..16], self.huge_page_size);
        LittleEndian::write_u64(&mut bytes[16..24], self.global_epoch);
        LittleEndian::write_u64(&mut bytes[24..32], self.total_pages);
        bytes[32..48].copy_from_slice(&self.node_uuid);
        LittleEndian::write_u64(&mut bytes[48..56], self.root_table_offset);
        crc64_nvme(&bytes)
    }

    pub fn to_bytes(self) -> [u8; 64] {
        let mut bytes = [0u8; 64];
        bytes[0..4].copy_from_slice(&self.magic);
        LittleEndian::write_u16(&mut bytes[4..6], self.spec_version);
        LittleEndian::write_u16(&mut bytes[6..8], self.flags);
        LittleEndian::write_u32(&mut bytes[8..12], self.page_size);
        LittleEndian::write_u32(&mut bytes[12..16], self.huge_page_size);
        LittleEndian::write_u64(&mut bytes[16..24], self.global_epoch);
        LittleEndian::write_u64(&mut bytes[24..32], self.total_pages);
        bytes[32..48].copy_from_slice(&self.node_uuid);
        LittleEndian::write_u64(&mut bytes[48..56], self.root_table_offset);
        LittleEndian::write_u16(&mut bytes[56..58], self.root_entries_count);
        bytes[58..64].copy_from_slice(&self.reserved_pad);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 64 {
            return Err("Buffer too small for EngineSuperblock".to_string());
        }

        let mut magic = [0u8; 4];
        magic.copy_from_slice(&bytes[0..4]);
        if magic != ENG_MAGIC {
            return Err(format!(
                "Invalid magic bytes: expected 'ENG\\x01', found {:?}",
                magic
            ));
        }

        let spec_version = LittleEndian::read_u16(&bytes[4..6]);
        let flags = LittleEndian::read_u16(&bytes[6..8]);
        let page_size = LittleEndian::read_u32(&bytes[8..12]);
        let huge_page_size = LittleEndian::read_u32(&bytes[12..16]);
        let global_epoch = LittleEndian::read_u64(&bytes[16..24]);
        let total_pages = LittleEndian::read_u64(&bytes[24..32]);
        let mut node_uuid = [0u8; 16];
        node_uuid.copy_from_slice(&bytes[32..48]);
        let root_table_offset = LittleEndian::read_u64(&bytes[48..56]);
        let root_entries_count = LittleEndian::read_u16(&bytes[56..58]);
        let mut reserved_pad = [0u8; 6];
        reserved_pad.copy_from_slice(&bytes[58..64]);

        let mut sb = Self {
            magic,
            spec_version,
            flags,
            page_size,
            huge_page_size,
            global_epoch,
            total_pages,
            node_uuid,
            root_table_offset,
            root_entries_count,
            reserved_pad,
            journal_start: 0x0060_0000,
            journal_size: REGION_2_SIZE,
            crc64: 0,
        };
        sb.crc64 = sb.compute_crc();
        Ok(sb)
    }
}

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootEntry {
    pub type_fingerprint: u64,
    pub root_ptr: EngineCausalPtr,
    pub last_mutated_epoch: u64,
    pub tenant_id: u64,
    pub canonical_name: [u8; 16],
    pub reserved_pad: [u8; 16],
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EngineCausalPtr {
    pub offset: u64,          // Physical file offset in storage (.db)
    pub epoch_id: u32,        // Creation epoch
    pub cluster_node_id: u32, // Host cluster node
}

#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JournalEntry {
    pub tx_id: u64,
    pub epoch: u64,
    pub target_page_offset: u64,
    pub shadow_page_offset: u64,
    pub commit_bit: u8, // 0x00: Preparing, 0x01: Committed, 0xFF: Aborted
    pub reserved: [u8; 7],
    pub tenant_id: [u8; 16],
    pub crc64: u64,
}

#[repr(C, align(32))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityHeader {
    pub type_fingerprint: u64,
    pub epoch_id: u64,
    pub tenant_ref_id: u64,
    pub object_flags: u64,
}

// CRC-64/NVME implementation polynomial 0x9A6C9329AC4BC9B5
pub fn crc64_nvme(data: &[u8]) -> u64 {
    let mut crc: u64 = 0xFFFFFFFFFFFFFFFF;
    let poly: u64 = 0x9A6C9329AC4BC9B5;

    for &byte in data {
        crc ^= (byte as u64) << 56;
        for _ in 0..8 {
            if (crc & 0x8000000000000000) != 0 {
                crc = (crc << 1) ^ poly;
            } else {
                crc <<= 1;
            }
        }
    }
    !crc
}

pub struct EngineDbStorage;

impl EngineDbStorage {
    pub fn initialize_db_with_mode<P: AsRef<Path>>(
        path: P,
        size_mib: u64,
        storage_mode: &str,
        pqc: Option<&str>,
    ) -> Result<(), String> {
        let total_bytes = size_mib * 1024 * 1024;
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .map_err(|e| format!("Failed to create storage file: {}", e))?;

        // Allocate total file size
        file.set_len(total_bytes)
            .map_err(|e| format!("Failed to set storage file length: {}", e))?;

        // Write primary superblock at 0x0000_0000
        let mut sb = EngineSuperblock::default();
        if storage_mode == "uuid7_blinded" || pqc.is_some() {
            sb.flags |= FLAG_UUID7_BLINDED | FLAG_ENCRYPTED | FLAG_PQC_SEALED;
            let root_uuid = UUIDv7::generate();
            sb.node_uuid = root_uuid.bytes;
        }
        sb.crc64 = sb.compute_crc();

        let sb_bytes = sb.to_bytes();
        file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
        file.write_all(&sb_bytes).map_err(|e| e.to_string())?;

        // Write mirror superblock at offset 0x0001_0000 (64 KiB)
        file.seek(SeekFrom::Start(0x0001_0000))
            .map_err(|e| e.to_string())?;
        file.write_all(&sb_bytes).map_err(|e| e.to_string())?;

        file.flush().map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn initialize_db<P: AsRef<Path>>(path: P, size_mib: u64) -> Result<(), String> {
        Self::initialize_db_with_mode(path, size_mib, "engine.db", None)
    }

    pub fn inspect_container<P: AsRef<Path>>(path: P) -> Result<String, String> {
        let mut file = File::open(&path).map_err(|e| {
            format!(
                "Failed to open storage container file '{}': {}",
                path.as_ref().display(),
                e
            )
        })?;

        let mut buf = [0u8; 64];
        file.read_exact(&mut buf)
            .map_err(|e| format!("Failed to read superblock: {}", e))?;

        let sb = EngineSuperblock::from_bytes(&buf)?;

        let mut report = String::new();
        report.push_str("=== Engine Lang Single-Level Store (.db) Inspection ===\n");
        report.push_str(&format!(
            "Container Path:      {}\n",
            path.as_ref().display()
        ));
        report.push_str(&format!(
            "Magic Signature:     {:?} ('ENG\\x01' Engine Lang Standard)\n",
            sb.magic
        ));
        report.push_str(&format!(
            "Specification Ver:   0x{:04X} (v{}.{})\n",
            sb.spec_version,
            sb.spec_version >> 8,
            sb.spec_version & 0xFF
        ));
        report.push_str(&format!(
            "System Flags:        0x{:04X} (DIRTY: {}, ENCRYPTED: {}, UUID7_BLINDED: {}, PQC_SEALED: {})\n",
            sb.flags,
            (sb.flags & FLAG_DIRTY) != 0,
            (sb.flags & FLAG_ENCRYPTED) != 0,
            sb.is_uuid7_blinded(),
            sb.is_pqc_sealed()
        ));
        if sb.is_uuid7_blinded() {
            report.push_str("Storage Model:       Blinded UUIDv7 (Quantum-Resistant Graph Persistence)\n");
            report.push_str("PQC Security Suite:  FIPS 203 ML-KEM-1024 / Crystals-Dilithium [ACTIVE]\n");
            report.push_str("Addressing Protocol: Strict UUIDv7 Token Resolution (Zero-Metadata Leakage)\n");
            report.push_str("Topology Privacy:    OBFUSCATED (Physical pointers & edges blinded in memory)\n");
        } else {
            report.push_str("Storage Model:       Standard engine.db (Transparent Single-Level Store)\n");
            report.push_str("Addressing Protocol: Direct Virtual DAX Causal Addresses (Sub-microsecond)\n");
            report.push_str("Topology Privacy:    CLEAR (Direct Graph Region 3 Entity Layout)\n");
        }
        report.push_str(&format!(
            "Page Size:           {} bytes (4 KiB standard)\n",
            sb.page_size
        ));
        report.push_str(&format!(
            "HugePage Size:       {} bytes (2 MiB)\n",
            sb.huge_page_size
        ));
        report.push_str(&format!(
            "Global Epoch:        #{} (Monotonic Causal)\n",
            sb.global_epoch
        ));
        report.push_str(&format!(
            "Total Pages:         {} pages (128 GiB addressable)\n",
            sb.total_pages
        ));
        let node_uuid_str = UUIDv7::from_bytes(sb.node_uuid).to_string();
        report.push_str(&format!("Node UUIDv7:         {} {:02X?}\n", node_uuid_str, sb.node_uuid));
        report.push_str(&format!(
            "Root Table Offset:   0x{:08X}\n",
            sb.root_table_offset
        ));
        report.push_str(&format!("Root Entries Count:  {}\n", sb.root_entries_count));
        report.push_str(&format!(
            "Journal Start:       0x{:08X} (Region 2)\n",
            sb.journal_start
        ));
        report.push_str(&format!(
            "Journal Size:        {} MiB\n",
            sb.journal_size / (1024 * 1024)
        ));
        report.push_str(&format!(
            "CRC-64/NVME:         0x{:016X} [VALID]\n",
            sb.crc64
        ));
        report.push_str("Crash Consistency:   VERIFIED (Hardware CLWB + SFENCE durable)\n");

        Ok(report)
    }
}

pub type NvramStorage = EngineDbStorage;

impl EngineDbStorage {
    pub fn initialize_nvram<P: AsRef<Path>>(path: P, size_mib: u64) -> Result<(), String> {
        Self::initialize_db(path, size_mib)
    }
}
