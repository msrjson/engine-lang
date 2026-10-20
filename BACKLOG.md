# Engine Lang Engineering Backlog & Production Control

**Document ID**: `ENG-BACKLOG-01.1`
**Governance**: The Vance Protocol Group / Engine Lang Core Working Group
**Standard**: SemVer 2.0.0 & Production Quality Engineering Gates
**License**: Apache License 2.0

---

## 1. Production Readiness Gates (Release Readiness Model)

Before any release is promoted from internal development to public tagging on GitHub, it must satisfy all criteria in the four production gates:

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│    GATE 0    │ ──> │    GATE 1    │ ──> │    GATE 2    │ ──> │    GATE 3    │
│ Formal Proof │     │ Memory & SLS │     │ Wire & Tools │     │ Distribution │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
```

| Gate | Focus Area | Mandatory Exit Criteria | Status (v0.3.1) |
|---|---|---|---|
| **Gate 0: Formal Correctness** | Syntax & SMT Prover | 100% of canonical `.eng` examples pass `enginec check` with Z3 SMT mathematical proofs; zero solver timeouts or false invariants. | **PASSED** (13/13 verified) |
| **Gate 1: Memory Durability** | SLS & Crash Consistency | Superblock integrity verified with valid CRC64-NVME checksum; atomic cache-line flushing (`clwb` + `sfence`); zero torn writes. | **PASSED** (Valid CRC64) |
| **Gate 2: Interoperability** | Bridges & Tooling | PostgreSQL Wire protocol responds to standard SQL queries on port 5432; Web visualizer functions on port 8090 without panics. | **PASSED** (DBeaver/psql verified) |
| **Gate 3: Release Packaging** | Binary Builds & CI | Reproducible builds pass on `x86_64-unknown-linux-gnu`; Docker container `engine-genesis-node` healthy; release checklist signed. | **PASSED** (Container active) |

---

## 2. Release Milestones & Target Schedule

### Milestone: `v0.3.1-silicon` (CURRENT RELEASE)
- **Codename**: Sovereign Silicon
- **Scope**: Core LL(1) compiler, Z3 SMT prover, SLS `engine.db` binary engine, PostgreSQL bridge, embedded Web gateway, 12-chapter book, Silicon Codegen specification (`ENG-CG-01.1`), Bare-Metal Unikernel specification (`ENG-OS-01.1`), and 13-example verification suite.
- **Production Status**: **RELEASED (v0.3.1 Tagged)**.

---

### Milestone: `v0.3.0-genesis` (INITIAL RELEASE)
- **Codename**: Genesis Alpha
- **Scope**: Core LL(1) compiler, Z3 SMT prover, SLS `engine.db` binary engine, PostgreSQL bridge, embedded Web gateway, 11-chapter book, and formal specification suite.
- **Production Status**: **ARCHIVED (Superseded by v0.3.1)**.

---

### Milestone: `v0.4.0-polyglot-dx` (Target: Q1-Q2 2027)
*Focus: Elimination of Enterprise Database Barriers.*

- [ ] **EPIC-3.1: Declarative Datasources (`@datasource`)**:
  - Direct syntax support for `datasource <Name> { driver: postgres | mysql | sqlite | clickhouse, uri: env("...") }`.
  - Automatic connection pooling and exponential backoff retry logic.
- [ ] **EPIC-3.2: Dual-Layer SLS Cache & CDC Streaming**:
  - Asynchronous background synchronization from local SLS arenas back to external relational databases.
- [ ] **EPIC-3.3: Language Server Protocol (`engine-lsp`)**:
  - Implementation of standard LSP protocol for real-time diagnostics, hover types, and SMT violation indicators in IDEs.
- [ ] **EPIC-3.4: Package Management (`enginec add`, `enginec publish`)**:
  - Decentralized Git-based and content-addressed package resolution.

---

### Milestone: `v0.5.0-silicon-dax` (Target: Q3-Q4 2027)
*Focus: Direct Hardware Codegen & Hard Real-Time Systems.*

- [ ] **EPIC-4.1: Native LLVM AOT Binary Compilation**:
  - Emit native object files (`.o`) and executables with zero runtime overhead.
- [ ] **EPIC-4.2: Direct DAX Memory-Mapped Silicon Intrinsics**:
  - Bypass OS page cache via Direct Access (DAX) NVMe and CXL.mem physical addressing.
- [ ] **EPIC-4.3: Hard Real-Time MMIO Memory Safety**:
  - Mathematical compile-time proofs that no hardware register read/write exceeds cycle deadlines (`ENG-RT-01`).

---

### Milestone: `v1.0.0-baremetal-os` (Target: 2028)
*Focus: Sovereign Single-Level Store Operating System.*

- [ ] **EPIC-7.1: Multiboot2 UEFI x86_64 Bootloader**:
  - Direct boot from BIOS/UEFI into Engine Lang runtime without host Linux kernel.
- [ ] **EPIC-7.2: Single-Address-Space SLS OS**:
  - The entire physical NVRAM and DRAM are unified into one persistent address space.

---

## 3. Active Bugfix & Defect Registry

| Defect ID | Severity | Component | Description | Status | Target Fix |
|---|---|---|---|---|---|
| **DEF-001** | P2 (Medium) | `bridge.rs` | PostgreSQL client query `SHOW server_version` requires explicit case-insensitive alias. | **CLOSED** | v0.3.0 |
| **DEF-002** | P3 (Low) | `memory.rs` | Superblock CRC-64 verification fails if padding bytes contain non-zero uninitialized data. | **CLOSED** | v0.3.0 |
| **DEF-003** | P2 (Medium) | `main.rs` | Version flag `enginec -V` only printed raw version string without spec metadata. | **RESOLVED** | v0.3.0 |
| **DEF-004** | P1 (High) | `smt.rs` | SMT translation of complex multi-clause boolean expressions could cause Z3 quantifier loops. | **CLOSED** | v0.3.0 |
| **DEF-005** | P3 (Low) | `web.rs` | Embedded blog UI lacked mobile viewport CSS responsiveness. | **CLOSED** | v0.3.0 |

*Severity Scale*:
- **P0 (Critical / Blocker)**: Data loss in SLS, crash in core compiler, or false SMT proof (soundness bug). Immediate emergency patch required within 24 hours.
- **P1 (High)**: Subcommand failure, broken bridge client connection, or parser panic. Fixed in current release cycle.
- **P2 (Medium)**: Ergonomic defect, missing non-critical diagnostics, or suboptimal error format.
- **P3 (Low)**: Minor cosmetic issue, documentation typo, or non-blocking enhancement.

---

## 4. Definition of Done (DoD) & Definition of Ready (DoR)

### Definition of Ready (DoR) - Entering Sprint / Development:
1. Feature or bugfix has an explicit issue ID (`ENG-xxx` or `DEF-xxx`).
2. Precise acceptance criteria and architectural impact statement provided.
3. Relevant normative specification identified (`ENG-SPEC`, `ENG-SLS`, `ENG-SMT`, etc.).
4. If a bug, a minimal `.eng` reproducible snippet must be attached.

### Definition of Done (DoD) - Promotion to Production:
1. **Zero Compiler Warnings**: `cargo clippy -- -D warnings` passes cleanly.
2. **Deterministic Formatting**: `cargo fmt --check` succeeds.
3. **Formal Verification Validation**: All `.eng` examples in `examples/` parse and pass `enginec check` with Z3 SMT proofs.
4. **Mandatory Container Verification (Rule 0.1)**: All tests and storage inspections must execute green inside Docker container `engine-genesis-node`.
5. **No Host-Only Assumptions**: Binaries must compile in clean environments without host paths.
6. **Documentation & Changelog**: All changes documented in `CHANGELOG.md` under `[Unreleased]` or target release version; relevant `docs/*.md` updated.
7. **License & Brand Compliance**: 100% English, Apache 2.0 header, canonical branding (Engine Lang, `enginec`, `engine.db`), and zero personal names.

---

## 5. Security & Vulnerability Triage SLA

Engine Lang enforces a strict responsible disclosure and triage model:
- **Triage Acknowledgement**: Within 48 hours of initial report.
- **P0 Soundness / SLS Corruption Bug**: Patched and verified within 72 hours with a cryptographic advisory.
- **Security Contact**: `security@engine-lang.org` / PGP Fingerprint listed in `BRANDING_AND_IP.md`.

