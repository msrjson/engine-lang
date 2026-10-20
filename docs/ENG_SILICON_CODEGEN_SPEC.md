# Engine Lang Silicon Codegen & Machine ABI Specification
**Document ID**: `ENG-CG-01.1`
**Status**: Normative Systems Specification
**Authorship**: The Vance Protocol Group / Engine Lang Core Working Group
**Target Architectures**: x86_64 (Intel/AMD), ARM64 (AArch64), RISC-V (RV64GC)
**License**: Apache License 2.0

---

## 1. Scope and Architectural Philosophy

This specification governs the direct translation of verified Engine Lang Abstract Syntax Trees (AST) and Intermediate Representation (IR) into native machine instructions without relying on an intermediary OS runtime or volatile garbage-collected heap.

For five decades, systems programming has been trapped in a trilemma:
1. **Raw Assembly**: Delivers absolute silicon control and predictable register allocation, but provides zero semantic safety, no portable abstractions, and catastrophic maintainability decay.
2. **C (ISO C99/C11/C23)**: Delivers a clean mapping to memory and an uncomplicated ABI, but weaponizes **Undefined Behavior (UB)**, invites out-of-bounds corruption, and lacks formal compile-time contracts.
3. **C++ (ISO C++20/C++23)**: Delivers expressive zero-cost abstractions, but introduces vtable dispatch jitter, unpredictable template instantiation bloat, fragile ABI stability, and non-deterministic exception unwinding (`.eh_frame`).

**Engine Lang (`enginec`)** harvests the architectural virtues of all three layers while annihilating their vulnerabilities:
- From **Assembly**: Direct instruction synthesis (`CLWB`, `SFENCE`, `DC CVAC`), explicit register constraints, and zero-cost stack layout.
- From **C**: Transparent Plain-Old-Data (POD) struct layout, flat memory representation, and zero runtime dependencies (no libc requirement).
- From **C++**: Monomorphized generic specialization and compile-time evaluation, elevated to mathematical proofs via the **Z3 SMT solver**.

---

## 2. Machine Calling Conventions & ABI Conformance

To guarantee seamless FFI with native hardware firmware (UEFI), hypervisors, and legacy libraries, `enginec` strictly adheres to standardized hardware ABIs per target architecture:

### 2.1 x86_64 Architecture (System V AMD64 ABI & Microsoft x64)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       x86_64 REGISTER ALLOCATION ABI                        │
├───────────────┬───────────────────────────────┬─────────────────────────────┤
│ Register      │ System V AMD64 Purpose        │ Engine Lang Codegen Role    │
├───────────────┼───────────────────────────────┼─────────────────────────────┤
│ `RAX`         │ 1st Return Register           │ Primary Action Return Value │
│ `RDX`         │ 2nd Return Register           │ Result<T, E> Error Payload  │
│ `RDI`         │ 1st Function Argument         │ Context / &mut Self pointer │
│ `RSI`         │ 2nd Function Argument         │ 1st Positional Parameter    │
│ `RDX`         │ 3rd Function Argument         │ 2nd Positional Parameter    │
│ `RCX`         │ 4th Function Argument         │ 3rd Positional Parameter    │
│ `R8`          │ 5th Function Argument         │ 4th Positional Parameter    │
│ `R9`          │ 6th Function Argument         │ 5th Positional Parameter    │
│ `R10`-`R11`   │ Temporary / Scratch           │ Expression Scratch Registers│
│ `RBX`, `R12`  │ Callee-Saved Registers        │ Persistent Arena Base Regs  │
│ `R13`-`R15`   │ Callee-Saved Registers        │ Local Epoch / Tenant Guard  │
│ `RSP`         │ Stack Pointer (16-byte align) │ Hardware Call Frame         │
│ `RBP`         │ Frame Pointer (Optional)      │ Deterministic Stack Walk    │
└───────────────┴───────────────────────────────┴─────────────────────────────┘
```

- **Stack Alignment**: The stack pointer `RSP` must remain **16-byte aligned** immediately before any `CALL` instruction.
- **Red Zone**: The 128-byte region beyond `RSP` is utilized by leaf actions without modifying `RSP`, eliminating frame allocation overhead in micro-actions.

### 2.2 ARM64 Architecture (AAPCS64 ABI)

- **Argument Registers**: `X0` through `X7` convey function arguments and return values. `X0` carries `&mut Self` or the primary return payload.
- **Scratch Registers**: `X9` through `X15` are caller-saved expression temporaries.
- **Hardware Platform Register**: `X18` is reserved for the Single-Level Store Thread/Tenant Context.
- **Link Register**: `X30` holds the return address, paired with Frame Pointer `X29`.

### 2.3 RISC-V Architecture (RV64GC ABI)

- **Integer Registers**: `a0`-`a7` for argument passing and return values; `s0`-`s11` for callee-saved state; `t0`-`t6` for temporaries.
- **Hardware Extensions**: Requires standard extensions `M` (Integer Multiplication/Division), `A` (Atomic Instructions), `F`/`D` (Single/Double Float), and `C` (Compressed Instructions).

---

## 3. Direct Silicon Instructions for Single-Level Store (SLS)

Traditional software relies on kernel context switches and POSIX syscalls (`msync`, `fsync`) to persist dirty pages to disk, burning thousands of CPU cycles.

`enginec` emits bare silicon memory bus instructions directly inside compiled action binaries:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     SILICON DURABILITY INSTRUCTION MATRIX                   │
├───────────────┬───────────────────────────────┬─────────────────────────────┤
│ Architecture  │ Cache-Line Flush / Writeback  │ Memory Ordering Fence       │
├───────────────┼───────────────────────────────┼─────────────────────────────┤
│ **x86_64**    │ `CLWB [reg]`                  │ `SFENCE` (Store Fence)      │
│               │ *(Fallback: `CLFLUSHOPT`)     │ `LFENCE` (Load Fence)       │
├───────────────┼───────────────────────────────┼─────────────────────────────┤
│ **ARM64**     │ `DC CVAC, Xn`                 │ `DMB ISHST` (Inner Share)   │
│               │ *(Data Cache Clean to PoC)    │ `ISB` (Instruction Sync)    │
├───────────────┼───────────────────────────────┼─────────────────────────────┤
│ **RISC-V**    │ `CBO.FLUSH (Xn)`              │ `FENCE rw, rw`              │
│               │ *(Cache-Block Clean & Inval)  │ `FENCE.I`                   │
└───────────────┴───────────────────────────────┴─────────────────────────────┘
```

### 3.1 Synthesis of Atomic Persistence Blocks

When an `.eng` action exits an `atomic { ... }` block:
1. Every mutated 64-byte cache line recorded in the action's shadow write-set is targeted with a cache-line writeback instruction (`CLWB` / `DC CVAC` / `CBO.FLUSH`).
2. A serializing store fence (`SFENCE` / `DMB ISHST` / `FENCE`) is synthesized.
3. The journal commit epoch pointer in Region 2 is incremented using an atomic hardware primitive (`LOCK CMPXCHG` / `LDREX`/`STREX` / `LR.D`/`SC.D`).
4. A concluding fence guarantees that external memory controllers and CXL persistent domains observe the commit before subsequent instructions execute.

---

## 4. Confronting the Legacy Pain: Undefined Behavior Taxonomy

The following matrix formally specifies how Engine Lang mathematically eliminates the catastrophic Undefined Behaviors of C and C++:

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│               THE ANATOMY OF SYSTEM PAIN: C/C++ VS ENGINE LANG                        │
├───────────────────────┬─────────────────────────────┬─────────────────────────────────┤
│ Legacy Defect (C/C++) │ Historical Root Cause       │ Engine Lang Formal Elimination  │
├───────────────────────┼─────────────────────────────┼─────────────────────────────────┤
│ **Buffer Overflow /   │ Pointer decay, lack of      │ Fixed-size slices and arrays    │
│ Out-of-Bounds Index** │ bounds validation, unsafe   │ proven by Z3 SMT at compile     │
│                       │ indexing via `ptr[i]`       │ time: `i < N` proven or rejected│
├───────────────────────┼─────────────────────────────┼─────────────────────────────────┤
│ **Use-After-Free /    │ Manual dynamic allocator    │ Arena-backed SLS (Region 3);    │
│ Double Free**         │ (`malloc`/`free`) and raw   │ Entities possess monotonic epoch│
│                       │ heap ownership transfers    │ lifetimes; no manual free exists│
├───────────────────────┼─────────────────────────────┼─────────────────────────────────┤
│ **Null Pointer        │ `NULL` / `nullptr` treated  │ Null does not exist. Optionality│
│ Dereference**         │ as valid pointer value in   │ is enforced via algebraic       │
│                       │ standard types (`T*`)       │ `Option<T>` with pattern match  │
├───────────────────────┼─────────────────────────────┼─────────────────────────────────┤
│ **Signed Integer      │ Compilers assume signed     │ Refinement types with bounded   │
│ Overflow (UB)**       │ overflow cannot happen,     │ ranges (`where value <= MAX`);  │
│                       │ producing security exploits │ SMT proves no overflow possible │
├───────────────────────┼─────────────────────────────┼─────────────────────────────────┤
│ **Data Races & Torn   │ Concurrent raw writes to    │ Hardware-enforced `atomic {}`   │
│ Reads/Writes**        │ shared memory without fence │ blocks with cache-line alignment│
│                       │ or atomic synchronization   │ and 64-byte atomic commit gates │
├───────────────────────┼─────────────────────────────┼─────────────────────────────────┤
│ **vtable Branch       │ Dynamic polymorphism via    │ 100% Monomorphization of generic│
│ Mispredictions**      │ function pointer tables     │ actions; direct machine jumps   │
│                       │ located in the volatile heap│ with zero indirect call overhead│
├───────────────────────┼─────────────────────────────┼─────────────────────────────────┤
│ **Exception Unwind    │ DWARF `.eh_frame` tables,   │ Zero exceptions. Strict monadic │
│ Jitter in Real-Time** │ stack unwinding runtimes,   │ `Result<T, E>` return values    │
│                       │ unpredictable latency       │ mapped directly to `RAX`/`RDX`  │
└───────────────────────┴─────────────────────────────┴─────────────────────────────────┘
```

---

## 5. Hardware Memory-Mapped I/O (`@mmio`) Specification

In C and Assembly, hardware registers are accessed via volatile pointers:
```c
// Traditional Unsafe C
#define UART0_DR (*((volatile uint32_t*)0x10000000))
UART0_DR = 0x41; // Untyped, unverified bit-width, no invariant checking
```

In Engine Lang, hardware peripherals are declared as formal silicon contracts:

```engine
module Hardware::Serial::UART16550;

@mmio(address: 0x10000000, size: 8)
register UARTRegisters {
    rbr_thr_dll: Byte, // Transmit/Receive Buffer / Divisor Latch Low
    ier_dlm: Byte,     // Interrupt Enable / Divisor Latch High
    iir_fcr: Byte,     // Interrupt Ident / FIFO Control
    lcr: Byte,         // Line Control Register
    mcr: Byte,         // Modem Control Register
    lsr: Byte,         // Line Status Register
    msr: Byte,         // Modem Status Register
    scr: Byte,         // Scratch Register
}
```

### 5.1 Codegen Rules for `@mmio`:
1. **Volatile Bus Access**: Every load/store to an `@mmio` field synthesizes an explicit, non-cached volatile load/store instruction (`MOV` with memory operand, `LDR`/`STR`, or `LW`/`SW`), preventing the compiler optimizer from dead-code eliminating hardware polling loops.
2. **Bit-Width Invariant Proving**: The Z3 SMT solver validates at compile time that written values strictly fit within the target field width (`Byte` = 8 bits, `Word` = 16 bits, `DWord` = 32 bits, `QWord` = 64 bits).
3. **Alignment Verification**: The compiler enforces that `address % size == 0`. Unaligned MMIO access is rejected as a compile-time error.

---

## 6. Bare-Metal Unikernel Bootstrap Hand-off

For bare-metal targets (`engine-os`), the compiler synthesizes an EFI binary (`engine-boot.efi`) conforming to the UEFI 2.10 specification:

```
[ UEFI Firmware ]
       │
       ▼ (Passes EFI_SYSTEM_TABLE and EFI_HANDLE in RCX/RDX)
[ engine-boot.efi ]
       │
       ├─► 1. Locate NVRAM / CXL Base (0x0020_0000)
       ├─► 2. Compute CRC64-NVME of Superblock (Region 0)
       ├─► 3. If FLAG_DIRTY: Replay Write Journal (Region 2)
       ├─► 4. ExitBootServices (Terminates firmware drivers)
       └─► 5. Jump to _start (action main in .eng)
```

The bootloader requires **zero external operating system code**, achieving instant execution of verified application code in under 15 milliseconds.

