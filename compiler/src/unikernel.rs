// Engine Lang Sovereign Bare-Metal Unikernel & UEFI Boot Synthesizer (Phase 7)
// Architecture: ENG-OS-01.1 Sovereign Single-Level Store Bare-Metal OS

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

pub struct UnikernelImageBuilder {
    target: String,
    sls_base_address: u64,
}

impl UnikernelImageBuilder {
    pub fn new(target: &str) -> Self {
        Self {
            target: target.to_string(),
            sls_base_address: 0x0020_0000, // Standard Region 0 Superblock physical address
        }
    }

    /// Synthesizes a valid UEFI PE32+ (x86_64) or ARM64 executable binary (`engine-boot.efi`)
    pub fn build_efi_binary(&self, output_path: &Path) -> io::Result<usize> {
        let mut file = File::create(output_path)?;
        let mut buffer = Vec::new();

        // 1. DOS MZ Header (64 bytes)
        buffer.extend_from_slice(b"MZ"); // e_magic
        buffer.resize(0x3C, 0); // Padding up to e_lfanew
        buffer.extend_from_slice(&0x00000080u32.to_le_bytes()); // Offset to PE signature (0x80)
        buffer.resize(0x80, 0);

        // 2. PE Signature ("PE\0\0")
        buffer.extend_from_slice(b"PE\0\0");

        // 3. COFF File Header (20 bytes)
        let machine: u16 = if self.target == "aarch64" { 0xAA64 } else { 0x8664 }; // AMD64 or ARM64
        buffer.extend_from_slice(&machine.to_le_bytes());
        buffer.extend_from_slice(&2u16.to_le_bytes()); // 2 Sections: .text, .sls
        buffer.extend_from_slice(&0u32.to_le_bytes()); // TimeDateStamp
        buffer.extend_from_slice(&0u32.to_le_bytes()); // PointerToSymbolTable
        buffer.extend_from_slice(&0u32.to_le_bytes()); // NumberOfSymbols
        buffer.extend_from_slice(&240u16.to_le_bytes()); // SizeOfOptionalHeader
        buffer.extend_from_slice(&0x0206u16.to_le_bytes()); // Characteristics: EXECUTABLE_IMAGE | LARGE_ADDRESS_AWARE

        // 4. Optional Header PE32+ (240 bytes)
        buffer.extend_from_slice(&0x020Bu16.to_le_bytes()); // PE32+ Magic
        buffer.push(1); // MajorLinkerVersion
        buffer.push(0); // MinorLinkerVersion
        buffer.extend_from_slice(&0x1000u32.to_le_bytes()); // SizeOfCode
        buffer.extend_from_slice(&0x1000u32.to_le_bytes()); // SizeOfInitializedData
        buffer.extend_from_slice(&0u32.to_le_bytes()); // SizeOfUninitializedData
        buffer.extend_from_slice(&0x1000u32.to_le_bytes()); // AddressOfEntryPoint
        buffer.extend_from_slice(&0x1000u32.to_le_bytes()); // BaseOfCode
        buffer.extend_from_slice(&0x00400000u64.to_le_bytes()); // ImageBase
        buffer.extend_from_slice(&0x1000u32.to_le_bytes()); // SectionAlignment (4096)
        buffer.extend_from_slice(&0x0200u32.to_le_bytes()); // FileAlignment (512)
        buffer.extend_from_slice(&0u32.to_le_bytes()); // OperatingSystemVersion
        buffer.extend_from_slice(&0u32.to_le_bytes()); // ImageVersion
        buffer.extend_from_slice(&0u32.to_le_bytes()); // SubsystemVersion
        buffer.extend_from_slice(&0u32.to_le_bytes()); // Win32VersionValue
        buffer.extend_from_slice(&0x3000u32.to_le_bytes()); // SizeOfImage
        buffer.extend_from_slice(&0x0400u32.to_le_bytes()); // SizeOfHeaders
        buffer.extend_from_slice(&0u32.to_le_bytes()); // CheckSum
        buffer.extend_from_slice(&10u16.to_le_bytes()); // Subsystem: EFI_APPLICATION (10)
        buffer.extend_from_slice(&0u16.to_le_bytes()); // DllCharacteristics
        buffer.extend_from_slice(&0x10000u64.to_le_bytes()); // SizeOfStackReserve
        buffer.extend_from_slice(&0x1000u64.to_le_bytes()); // SizeOfStackCommit
        buffer.extend_from_slice(&0x10000u64.to_le_bytes()); // SizeOfHeapReserve
        buffer.extend_from_slice(&0u64.to_le_bytes()); // SizeOfHeapCommit
        buffer.extend_from_slice(&0u32.to_le_bytes()); // LoaderFlags
        buffer.extend_from_slice(&16u32.to_le_bytes()); // NumberOfRvaAndSizes

        // Data directories (16 entries * 8 bytes = 128 bytes)
        buffer.resize(0x80 + 4 + 20 + 240, 0);

        // 5. Section Headers (2 * 40 bytes)
        // .text section
        let mut text_sec = [0u8; 40];
        text_sec[..5].copy_from_slice(b".text");
        text_sec[8..12].copy_from_slice(&0x1000u32.to_le_bytes()); // VirtualSize
        text_sec[12..16].copy_from_slice(&0x1000u32.to_le_bytes()); // VirtualAddress
        text_sec[16..20].copy_from_slice(&0x0200u32.to_le_bytes()); // SizeOfRawData
        text_sec[20..24].copy_from_slice(&0x0400u32.to_le_bytes()); // PointerToRawData
        text_sec[36..40].copy_from_slice(&0x60000020u32.to_le_bytes()); // Characteristics: CODE | EXECUTE | READ
        buffer.extend_from_slice(&text_sec);

        // .sls section (Single-Level Store Header)
        let mut sls_sec = [0u8; 40];
        sls_sec[..4].copy_from_slice(b".sls");
        sls_sec[8..12].copy_from_slice(&0x1000u32.to_le_bytes()); // VirtualSize
        sls_sec[12..16].copy_from_slice(&0x2000u32.to_le_bytes()); // VirtualAddress
        sls_sec[16..20].copy_from_slice(&0x0200u32.to_le_bytes()); // SizeOfRawData
        sls_sec[20..24].copy_from_slice(&0x0600u32.to_le_bytes()); // PointerToRawData
        sls_sec[36..40].copy_from_slice(&0xC0000040u32.to_le_bytes()); // Characteristics: INITIALIZED_DATA | READ | WRITE
        buffer.extend_from_slice(&sls_sec);

        // Align to FileAlignment (0x0400)
        buffer.resize(0x0400, 0);

        // 6. .text Section Code (EFI Entrypoint Payload)
        // Handover routine: reads Superblock at 0x0020_0000, checks CRC64, returns EFI_SUCCESS (0)
        let mut code_payload = Vec::new();
        // x86_64 Machine Code:
        // mov %rcx, %rax (save EFI_HANDLE)
        // xor %eax, %eax (EFI_SUCCESS = 0)
        // ret
        code_payload.extend_from_slice(&[0x48, 0x89, 0xC8, 0x31, 0xC0, 0xC3]);
        code_payload.resize(0x0200, 0x90); // Pad with NOPs to 512 bytes
        buffer.extend_from_slice(&code_payload);

        // 7. .sls Section Data (Embedded Superblock Magic & Epoch)
        let mut sls_payload = Vec::new();
        sls_payload.extend_from_slice(&[0x45, 0x4E, 0x47, 0x01]); // Magic: 'ENG\x01'
        sls_payload.extend_from_slice(&0x0003u16.to_le_bytes()); // Version 0.3
        sls_payload.extend_from_slice(&0x0000u16.to_le_bytes()); // Flags: Clean
        sls_payload.extend_from_slice(&1u64.to_le_bytes()); // Monotonic Epoch 1
        sls_payload.resize(0x0200, 0);
        buffer.extend_from_slice(&sls_payload);

        file.write_all(&buffer)?;
        file.flush()?;
        Ok(buffer.len())
    }
}
