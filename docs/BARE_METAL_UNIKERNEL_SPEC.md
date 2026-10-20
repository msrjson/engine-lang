# Engine Lang Bare-Metal Unikernel & Sovereign OS Specification
**Document ID**: `ENG-OS-01.1`
**Status**: Normative Systems Specification
**Authorship**: The Vance Protocol Group / Engine Lang Core Working Group
**Target Platform**: Bare Silicon (x86_64 UEFI, ARM64 SystemReady, RISC-V Privileged)
**License**: Apache License 2.0

---

## 1. Executive Vision: The Language as the Operating System

Traditional software systems suffer from an artificial tripartite division:
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       TRADITIONAL COMPUTING STACK                           │
│  [ Application Layer ] ──► Runtime / GC / In-Memory Heap                   │
│          │                                                                  │
│  [ OS Kernel Layer ]   ──► Syscalls, Context Switches, Virtual Memory VMA   │
│          │                                                                  │
│  [ Storage / DB ]      ──► Page Cache, Block Drivers, File Systems, WAL     │
└─────────────────────────────────────────────────────────────────────────────┘
```

Every transition across these layers introduces latency, serialization overhead, cache invalidation, and potential failure modes.

**Engine Lang Bare-Metal Unikernel (`engine-os`)** collapses this entire stack into a singular, inviolable silicon runtime:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ENGINE LANG SOVEREIGN UNIKERNEL                          │
│                                                                             │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │ Verified Domain Logic (.eng Actions, Entities & Refinements)           │  │
│  ├───────────────────────────────────────────────────────────────────────┤  │
│  │ Single-Level Store (SLS): Physical Memory IS Persistent Storage       │  │
│  ├───────────────────────────────────────────────────────────────────────┤  │
│  │ Hardware MMIO & Silicon Drivers (@mmio, clwb, sfence, Interrupts)     │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                     │                                       │
│                         BARE SILICON (CXL / NVRAM)                          │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Silicon Memory Map & Single-Level Store Mapping

When booting on UEFI bare silicon, `engine-os` maps the entire physical NVRAM / CXL memory pool directly into a flat 64-bit address space. There is no concept of a separate swap space or volatile swap file:

```
0x0000_0000_0000_0000 ┌──────────────────────────────────────────────┐
                      │ UEFI Firmware Reserved / Real-Mode Vector    │
0x0000_0000_0010_0000 ├──────────────────────────────────────────────┤
                      │ engine-boot Bootloader Core (Stage 1 & 2)    │
0x0000_0000_0020_0000 ├──────────────────────────────────────────────┤
                      │ Single-Level Store Superblock (Region 0)     │
                      │ 64-byte aligned, CRC64-NVME checksummed      │
0x0000_0000_0040_0000 ├──────────────────────────────────────────────┤
                      │ Page Allocation Bitmap (Region 1 - 4 MiB)    │
0x0000_0000_0080_0000 ├──────────────────────────────────────────────┤
                      │ Atomic Write Journal (Region 2 - 10 MiB)     │
0x0000_0000_0100_0000 ├──────────────────────────────────────────────┤
                      │ Persistent Entity Heap (Region 3 - 128 GiB+) │
                      │ Direct CXL / NVRAM Cache-Line Mapped         │
0xFFFF_8000_0000_0000 ├──────────────────────────────────────────────┤
                      │ Memory-Mapped I/O Space (@mmio Base)         │
                      │ Direct Peripheral & NIC Register Access      │
                      └──────────────────────────────────────────────┘
```

---

## 3. Boot Sequence: Instant Warm Boot (`engine-boot`)

1. **UEFI Firmware Hand-off**:
   - Firmware locates the EFI boot partition containing `engine-boot.efi`.
   - Firmware loads the bootstrap binary and enters 64-bit Long Mode.
2. **NVRAM Verification**:
   - `engine-boot` reads physical address `0x0020_0000` to locate the `EngineSuperblock`.
   - The CRC64-NVME checksum is verified against the header fields.
3. **Crash Consistency Check**:
   - If `FLAG_DIRTY` is clean (`0x0000`), the previous shutdown was clean or all atomic shadow pages committed.
   - If `FLAG_DIRTY` is set (`0x0001`), the hardware write journal (Region 2) scans uncommitted transactions, replays valid epoch commits, and rolls back incomplete shadow writes.
4. **Instant State Resumption**:
   - The global causal epoch is loaded.
   - All `@persistent` entities remain in their exact physical addresses. Zero deserialization or warm-up cache phase is required.
   - The root action (`action main()`) commences execution within **< 15 milliseconds** from power-on.

---

## 4. Hardware Interaction: Memory-Mapped I/O (`@mmio`)

In `engine-os`, hardware peripherals (industrial serial ports, timer chips, network controllers) are not manipulated via opaque kernel driver files (`/dev/*`). They are declared as strongly-typed, SMT-constrained registers in `.eng`:

```engine
module Drivers::Network::VirtIO;

@mmio(address: 0xFFFF_8000_1000, size: 32)
register VirtioNetRegisters {
    device_features: DWord,
    driver_features: DWord,
    queue_address: QWord,
    queue_size: Word,
    queue_select: Word,
    queue_notify: Word,
    device_status: Byte,
    isr_status: Byte,
}

action initialize_nic(status: &mut VirtioNetRegisters) -> Result<(), Text> {
    atomic {
        status.device_status = 1; // ACKNOWLEDGE
        status.device_status = 3; // DRIVER
    }
    return Ok(());
}
```

The Z3 SMT prover guarantees in compile time that register bit operations do not attempt invalid configurations or write to out-of-bounds MMIO regions.

---

## 5. Certification and Industrial Suitability

- **Zero Dynamic Allocation Jitter**: Unikernel execution relies on statically sized arena pages, eliminating garbage-collector pauses and fragmented heap latency.
- **Formally Verified Control**: Every device driver written in `.eng` can prove safety invariants before firmware flash.
- **Fail-Safe Durability**: Sudden power loss leaves the system in a consistent epoch state, ready for millisecond recovery.

