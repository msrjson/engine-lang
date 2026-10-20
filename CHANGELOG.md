# Changelog

All notable changes to the **Engine Lang** ecosystem (compiler `enginec`, Single-Level Store specification, runtime, and tools) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Planned (Phase 3 & Phase 4)
- **Polyglot Database Connectors (`@datasource`)**: Declarative schema mapping for PostgreSQL, MySQL, SQLite, and ClickHouse.
- **Language Server Protocol (`engine-lsp`)**: Real-time Z3 SMT diagnostics and counterexample margin decorations for VS Code and Neovim.
- **Native LLVM JIT & AOT Object Emitter**: Direct generation of `.o` and `.so` binaries with direct DAX silicon intrinsics (`clwb`, `pcommit`, `clflushopt`).
- **Bare-Metal Bootloader Binary (`engine-boot.efi`)**: Pre-compiled UEFI x86_64 single-address-space binary running without Linux (`ENG-OS-01.1`).

---

## [0.3.1] - 2026-10-20 - "Sovereign Silicon"

### Added
- **Silicon Codegen & Machine ABI Specification (`docs/ENG_SILICON_CODEGEN_SPEC.md`)**:
  - Normative specification `ENG-CG-01.1` establishing register allocation rules for System V AMD64, AAPCS64, and RV64GC.
  - Direct hardware instruction synthesis: `CLWB`, `SFENCE`, `LFENCE`, `PAUSE` on x86_64; `DC CVAC`, `DMB ISHST` on ARM64; `CBO.FLUSH`, `FENCE` on RISC-V.
  - Formal Zero-UB taxonomy mathematically eliminating signed overflow, use-after-free, buffer overflows, and null dereferences via Z3 SMT contracts.
  - Formal specification of `@mmio register` declarations with volatile memory bus semantics and compile-time bit-width checks.
- **Bare-Metal Unikernel & Sovereign OS Specification (`docs/BARE_METAL_UNIKERNEL_SPEC.md`)**:
  - Normative specification `ENG-OS-01.1` detailing instant warm boot (< 15ms) via `engine-boot.efi` and flat 64-bit Single-Level Store physical address mapping.
- **The Book of Engine Lang - Chapter 12 (`book/src/12_confronting_c_cpp_assembly.md`)**:
  - *"Confronting C, C++, and Assembly: From Legacy Pain to Sovereign Silicon"*.
  - Comprehensive historical analysis of the legacy triad's pain points and how Engine Lang harvests their machine-level virtues.
- **Canonical Silicon Confrontation Example (`examples/12_c_cpp_assembly_confrontation.eng`)**:
  - Full-stack verified example demonstrating bounded refinement types, UART 16550 `@mmio` hardware registers, and atomic persistence with hardware cache-line flushing.

### Changed
- **CLI Version Diagnostics Alignment**:
  - Upgraded `enginec version` (and `--json`) to report `v0.3.1-silicon` with full codegen (`ENG-CG-01.1`) and unikernel (`ENG-OS-01.1`) metadata.
- **Component Version Synchronization**:
  - Synchronized `compiler/Cargo.toml`, `Engine.toml`, and `Dockerfile` to `0.3.1`.

---

## [0.3.0] - 2026-10-20 - "Genesis Alpha"

### Added
- **Official Brand Unification**: Consolidated project identity to **Engine Lang** (`enginec`, `.eng`, `Engine.toml`, `engine.db`).
- **PostgreSQL Wire Virtual Bridge (`compiler/src/bridge.rs`)**:
  - Emulated PostgreSQL v3 protocol responder listening on port `5432`.
  - Native integration with standard SQL clients: DBeaver, `psql`, pgAdmin, and business intelligence dashboards.
  - Interactive SQL parser supporting `SELECT * FROM <entity>`, `pg_catalog`, `information_schema`, and virtual entity introspection.
- **Embedded Web Visualizer & Runtime Gateway (`compiler/src/web.rs`)**:
  - HTTP web server on port `8090` providing real-time dashboard visualization for the Single-Level Store.
  - Interactive Causal Blog demo UI with user registration, categories, and atomic post publishing.
  - Real-time SLS page table viewer with live CRC64 validation status and epoch telemetry.
- **Target-Native LLVM IR Emitter (`compiler/src/codegen.rs`)**:
  - Subcommand `enginec emit-ir <file.eng> --target <x86_64|aarch64>`.
  - Emits LLVM Intermediate Representation with inline assembly for hardware cache-line write-back (`clwb`) and store fence (`sfence`).
- **Standard Library of Canonical Examples (`examples/`)**:
  - `01_hello_causal.eng`: Causal epoch sequencing and immutable log entities.
  - `02_persistent_vehicle_fleet.eng`: Enterprise IoT fleet management with SMT refinement invariants.
  - `03_vector_ai_search.eng`: High-dimensional vector cosine distance search with cosine similarity constraints.
  - `04_causal_blog.eng`: Full-stack multi-user transactional CMS with refinement types.
  - `05_polyglot_industrial_iot.eng`: Industrial telemetry with dual-layer SLS write-through caching.
  - `06_industrial_plc_mmio.eng`: Direct hardware register access (`@mmio`) and hard real-time safety bounds.
  - `05_book_chapter1_ledger.eng` through `10_book_chapter6_causal_mesh_scaling.eng`: Accompanying code for the official book.
  - `11_rust_vs_engine_paradigm.eng`: Architectural confrontation matrix between Rust and Engine Lang.
- **Bare-Metal Unikernel Specification (`docs/BARE_METAL_UNIKERNEL_SPEC.md`)**:
  - Normative specification `ENG-UNIKERNEL-01.1` for running Engine Lang directly on bare-metal x86_64/AArch64 hardware without an underlying OS.
- **Authoritative Book (`book/src/`)**:
  - 11 complete engineering chapters covering foundational theory, compiler architecture, memory models, SMT proofs, and bare-metal OS deployment.
- **Comprehensive Whitepaper**:
  - Full normative document available in Markdown (`docs/whitepaper/`) and compiled PDF formats.
- **Rich CLI Version Diagnostics**:
  - `enginec version` subcommand outputting full compiler, prover, specification, and storage architecture metadata.

### Changed
- **Storage Subsystem Standardization (`ENG-SLS-01.1`)**:
  - Renamed all persistence terminology to Single-Level Store (`engine.db`).
  - Standardized Superblock Magic to `0x454E474E` (`ENG\x01`) and specification version `0x0003`.
  - Standardized hardware persistence barriers to `clwb` + `sfence` for non-volatile RAM and DAX storage.
- **File Extension Standardization**:
  - Migrated source file extension from `.ctm` to `.eng`.
  - Migrated workspace manifest from `Continuum.toml` to `Engine.toml`.
- **Compiler Subcommand Alignment**:
  - `enginec inspect` reads `.db` containers with updated CRC-64/NVME checksum validation.
  - `enginec run` automatically provisions and mounts `data/engine.db` with default zero-config.

### Deprecated
- Legacy file formats (`.ctm`) and legacy workspace manifests (`Continuum.toml`).
- All manual ORM and database connection code in favor of declarative `@datasource` and native SLS.

### Removed
- Legacy references to older development naming conventions.
- Unused monolithic storage mocks.

### Fixed
- Fixed parser operator precedence in binary arithmetic and logical comparison chains (`ENG-SPEC-03.1`).
- Fixed PostgreSQL protocol parameter response for client compatibility with DBeaver 24+.
- Fixed CRC-64/NVME polynomial calculation on zero-padded superblock blocks.

### Security
- Invariant verification enforced at compile time via Z3 SMT prover: prevented arithmetic overflows, negative account balances, and unconstrained memory writes before runtime.
- Hardened SLS container header with CRC64-NVME checksum to guarantee crash consistency and detect data corruption.

---

## [0.2.0] - 2026-10-19 - "Persistent Memory Alpha"

### Added
- Single-Level Store (SLS) physical binary layout prototype with 4 KiB base pages and 2 MiB huge pages.
- Automated Z3 SMT prover integration (`enginec check`) for arithmetic refinement constraints.
- Atomic mutation journal and crash recovery mechanisms.
- Initial Docker container test harness (`engine-genesis-node`).

### Fixed
- Fixed recursive descent parser backtracking in complex nested type definitions.
- Resolved memory leak in AST allocation arenas.

---

## [0.1.0] - 2026-10-18 - "Formal Foundations PoC"

### Added
- Core EBNF grammar specification (`ENG-SPEC-01`).
- Deterministic LL(1) Lexer and Recursive-Descent Parser in Rust.
- AST definitions for modules, entities, actions, and refinement types.
- Apache License 2.0 governance and decentralized authorship attribution.

