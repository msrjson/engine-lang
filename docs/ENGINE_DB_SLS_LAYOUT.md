# Technical Specification: Continuous State File Format (.db) & Single-Level Store Layout
**Document ID**: `ENG-SLS-01.1`
**Status**: Official Normative Standard (Core Specification)
**Target Architectures**: x86-64 (CXL.mem / NVRAM / DAX) & ARM64 (FEAT_DPB / FEAT_DPB2)
**Byte Ordering**: Little-Endian (Bus Standard)

---

## 1. Overview and Physical Memory Model

The `.db` (default: `engine.db`) file format constitutes the physical media representation of the Engine Lang Single-Level Store (SLS). Mapped directly into the 64-bit virtual address space via Direct Access (DAX) file systems (e.g., ext4/XFS mounted with `-o dax` or POSIX memory mapping), the runtime bypasses the OS page cache entirely, achieving single-digit microsecond access to persistent graphs.

The container is partitioned into four contiguous physical regions aligned to HugePage (2 MiB) and CPU cache-line (64 bytes) boundaries:

```
+──────────────────────────────────────────────────────────────────────────+
|                      PHYSICAL MAP OF THE .db CONTAINER                   |
+---------------------+-------------------+-------------------+------------+
|  Region 0 (0-2 MiB) | Region 1 (2-6 MiB)| Region 2 (6-16 MiB| Region 3   |
|  Superblock &       | Free-Space Alloc  | Causal Journal    | Continuous |
|  Root Pointers      | Bitmap (Lock-free)| Ring Buffer       | Entity Heap|
|  (Global Header)    | (4 KiB Pages)     | (Hardware WAL)    | (Graphs)   |
+---------------------+-------------------+-------------------+------------+
```

---

## 2. Region 0: Superblock & Global Roots (Offset: `0x0000_0000` - `0x001F_FFFF`)

Region 0 spans 2,097,152 bytes (2 MiB). The primordial header starts at offset zero and is structured in 64-byte aligned blocks.

### 2.1. Binary Layout of the Primary Superblock (Offset: `0x0000_0000`)

```
 0               15 16             31 32             47 48             63
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|       Magic: 'E' 'N' 'G' 0x01   | Spec: 0x0003    | Flags: 0x0000       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Page Size: 0x1000 (4 KiB)       | HugePage Size: 0x0020_0000 (2 MiB)    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                       Global Epoch Counter (64 bits)                    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                     Total Allocated Pages (64 bits)                     |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                      Cluster Node UUID (128 bits)                       |
|                                                                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|           Root Table Offset (64 bits)           | Root Entries Count(16)|
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|          Journal Start (64 bits)                | Journal Size (64 bits)|
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                     CRC-64/NVME of Superblock (64 bits)                 |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

#### Field Specifications:
- **Magic Bytes** (`0x00` - 4 bytes): `0x45 0x4E 0x47 0x01` (`ENG\x01`). Identifies the Engine Lang container.
- **Spec Version** (`0x04` - 2 bytes): `0x0003` (Engine Lang SLS Version 0.3).
- **System Flags** (`0x06` - 2 bytes):
  - `Bit 0 (DIRTY)`: Process did not shut down cleanly; triggers journal recovery on boot.
  - `Bit 1 (ENCRYPTED)`: Heap pages encrypted with AES-256-XTS via hardware key.
  - `Bit 2 (COMPACTING)`: Background Causal GC was active during power cutoff.
- **Global Epoch Counter** (`0x10` - 8 bytes): Monotonically increasing atomic causal epoch counter.
- **CRC-64/NVME** (`0x38` - 8 bytes): Cyclic redundancy checksum computed over the first 56 bytes.

#### Mirror Superblock:
At offset `0x0001_0000` (64 KiB), an identical copy of the Superblock is maintained. If the primary superblock CRC-64 verification fails following a power cutoff, the runtime restores from the mirror.

### 2.2. Root Pointers Table (Offset: `0x0000_1000`)
The Root Pointers Table functions as the fast-lookup dictionary for master graph entry points. Each entry is 64 bytes:

```
[ Offset: 0x0000_1000: Root Table Entry (64 bytes each) ]
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Entity Type Fingerprint / MurmurHash3 (64 bits)                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Physical Root Pointer (128-bit Causal SLS Address)                      |
|                                                                         |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Last Mutated Epoch (64 bits)            | Tenant Context ID (64 bits)   |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
| Canonical Name String (16 bytes inline ASCII prefix)                    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

---

## 3. Region 1: Lock-Free Free-Space Bitmap (Offset: `0x0020_0000` - `0x005F_FFFF`)

This 4 MiB region manages allocation for standard 4 KiB heap pages in Region 3.
- Each bit corresponds to one 4 KiB page.
- $4\text{ MiB} \times 8\text{ bits/byte} = 33,554,432\text{ pages}$.
- Total addressable physical memory with this bitmap: **128 GiB**.

### Lock-Free Hardware Allocation Routine
Allocation inspects 64-bit words (`uint64_t`) using direct silicon instructions:
1. `_tzcnt_u64` (Trailing Zero Count) to locate the first free bit (`0`) in 1 CPU cycle.
2. `lock bts` (Bit Test and Set) to atomically transition the bit from `0` to `1`. If the carry flag indicates another thread acquired the page concurrently, the allocator loops with an adaptive backoff.

---

## 4. Region 2: Causal Journal Ring Buffer (Offset: `0x0060_0000` - `0x00FF_FFFF`)

This 10 MiB region implements a hardware-assisted circular log for atomic transactions, recording shadow page remaps and commit flags.

```
       [ Head Pointer: Next Write ]
                      │
                      ▼
 ┌──────────────┬──────────────┬──────────────┬──────────────┐
 │  Entry #102  │  Entry #103  │  Entry #104  │  [Empty...]  │
 └──────────────┴──────────────┴──────────────┴──────────────┘
                      ▲
                      │
        [ Tail Pointer: Last Reaped Epoch ]
```

### Binary Layout of `JournalEntry` (64 bytes aligned)

| Offset | Size | Type | Field | Operational Purpose |
|---|---|---|---|---|
| `+0x00` | 8 bytes | `uint64_t` | `tx_id` | Strict monotonic atomic transaction ID. |
| `+0x08` | 8 bytes | `uint64_t` | `epoch` | Creation epoch for Causal GC reclamation. |
| `+0x10` | 8 bytes | `uint64_t` | `target_page_offset` | Physical offset of the original page to mutate. |
| `+0x18` | 8 bytes | `uint64_t` | `shadow_page_offset` | Physical offset of new shadow page with mutations. |
| `+0x20` | 1 byte | `uint8_t` | `commit_bit` | `0x00`: Preparing, `0x01`: Committed, `0xFF`: Aborted. |
| `+0x21` | 7 bytes | `uint8_t[7]` | `reserved` | Padding to maintain 64-bit word alignment. |
| `+0x28` | 16 bytes | `uint128_t` | `tenant_id` | Cryptographic tenant isolation context. |
| `+0x38` | 8 bytes | `uint64_t` | `crc64` | Checksum protecting against torn writes. |

### Strict Hardware Commit Sequence
When an `atomic { ... }` block closes:
1. Write modified payload into `shadow_page`.
2. Issue `clwb [shadow_page]` (flush page to memory controller).
3. Populate `JournalEntry` with `commit_bit = 0x01`.
4. Issue `clwb [journal_entry.commit_bit]`.
5. Issue `sfence` (serialize store operations to memory controller).

The transaction is physically durable the nanosecond `sfence` completes.

---

## 5. Region 3: Continuous Entity Heap (Offset: `0x0100_0000`+)

Starting at offset 16 MiB, the heap stores persistent entity instances and graph relationships.

### 5.1. Entity Header (`EntityHeader` - 32 bytes)
Every persistent entity is prefixed with this header:
```
Offset 0x00: [ Type Fingerprint (8 bytes) ] -> SHA-256 truncated hash of contract & layout
Offset 0x08: [ Epoch ID (8 bytes)         ] -> Causal allocation epoch
Offset 0x10: [ Tenant Ref ID (8 bytes)    ] -> Cryptographic tenant security context
Offset 0x18: [ Object Flags & Status (8b) ] -> Bitflags (Immutable, Mutating, Marked for GC)
```

### 5.2. Universal 128-bit Causal Pointer (`engine_ptr_t`)
References between entities (`Ref<T>`) are stored as 16-byte scalar addresses:

```
 127             96 95          64 63                            0
+------------------+--------------+-------------------------------+
|  Cluster Node ID |   Epoch ID   |   Physical File Offset (.db)  |
+------------------+--------------+-------------------------------+
```
- **Cluster Node ID** (32 bits): `0x0000_0000` indicates local host memory. Any non-zero value targets a remote node in the P2P mesh, triggering transparent RDMA/QUIC resolution.
- **Epoch ID** (32 bits): Causal epoch of pointer generation.
- **Physical File Offset** (64 bits): Absolute byte offset within the `.db` container file.

---

## 6. Deterministic Crash Recovery Protocol (<3 ms)

On process restart or post-power-failure initialization:

```
       [ Open .db file with O_RDWR | O_DIRECT ]
                            │
                            ▼
     [ Validate Primary Superblock CRC-64 (Offset 0x00) ]
       ├─ If corrupt: Load Mirror Superblock (Offset 0x10000)
       └─ If both fail: Fatal Panic (Hardware Media Failure)
                            │
                            ▼
     [ Inspect DIRTY Bit in System Flags ]
       ├─ If DIRTY == 0: Clean boot. Ready instantly (<500 µs).
       └─ If DIRTY == 1: Execute Causal Journal Scan:
                            │
                            ▼
     [ Scan Ring Buffer between Tail Pointer and Head Pointer ]
       ├─ If commit_bit == 0x01:
       │    └─ Atomically remap target_page_offset to shadow_page_offset
       ├─ If commit_bit == 0x00 or 0xFF:
       │    └─ Free shadow_page_offset in Allocation Bitmap (Rollback)
                            │
                            ▼
     [ Clear DIRTY bit, issue CLWB + SFENCE, release Runtime to Traffic ]
```

---

## 7. C / Rust Direct ABI Mapping

```rust
#[repr(C, align(64))]
pub struct EngineSuperblock {
    pub magic: [u8; 4],            // 0x45, 0x4E, 0x47, 0x01 ('ENG\x01')
    pub spec_version: u16,         // 0x0003
    pub flags: u16,                // Bit 0: DIRTY, Bit 1: ENCRYPTED
    pub page_size: u32,            // 4096 (0x1000)
    pub huge_page_size: u32,       // 2097152 (0x200000)
    pub global_epoch: u64,         // Monotonic counter
    pub total_pages: u64,          // Total allocated pages
    pub node_uuid: [u8; 16],       // Host Cluster Node UUID
    pub root_table_offset: u64,    // Root table byte offset
    pub root_entries_count: u16,   // Number of registered roots
    pub reserved_pad: [u8; 6],     // 64-bit alignment padding
    pub journal_start: u64,        // Ring buffer offset (0x0060_0000)
    pub journal_size: u64,         // Ring buffer size (10 MiB)
    pub crc64: u64,                // CRC-64/NVME of first 56 bytes
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EngineCausalPtr {
    pub offset: u64,               // Physical byte offset in .db
    pub epoch_id: u32,             // Emission epoch
    pub cluster_node_id: u32,      // Cluster node identifier
}
```

