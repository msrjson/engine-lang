# Engine Lang Engineering Roadmap: From Micro-Logic to Bare-Metal OS
**Document ID**: `ENG-ROAD-01.2`  
**Status**: Canonical Engineering Roadmap  
**Authorship**: The Vance Protocol Group / Engine Lang Core Working Group  
**License**: Apache License 2.0  

---

## Strategic Vision

Engine Lang is engineered to eliminate the fundamental systemic frictions of modern software engineering:
1. **The In-Process Silo & Serialization Tax**: Eliminated via Single-Level Store (SLS) and declarative polyglot data connectors.
2. **Runtime Failures & Defensive Boilerplate**: Eliminated via compile-time Z3 SMT formal verification.
3. **Infrastructure Fragmentation**: Eliminated by embedding persistence, causality, and legacy bridges directly into the language syntax.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       ENGINE LANG EVOLUTIONARY PHASES                       │
├─────────┬───────────────────────────────────────────────┬───────────────────┤
│ Phase   │ Architectural Objective                       │ Status / Target   │
├─────────┼───────────────────────────────────────────────┼───────────────────┤
│ Phase 1 │ Formal Specification & Grammar (ENG-SPEC-03)  │ COMPLETE          │
│ Phase 2 │ Bootstrap Compiler & Z3 SMT Engine (enginec)  │ ACTIVE / VERIFIED │
│ Phase 3 │ Polyglot DB Connectors & Frictionless DX      │ ACTIVE / VERIFIED │
│ Phase 4 │ Silicon DAX & Native LLVM Codegen (emit-ir)   │ ACTIVE / VERIFIED │
│ Phase 5 │ Industrial Hard Real-Time & Hardware MMIO     │ ACTIVE / VERIFIED │
│ Phase 6 │ Planetary Causal Mesh & Autonomous Swarms     │ Q3-Q4 2027        │
│ Phase 7 │ Sovereign Single-Level Store Bare-Metal OS    │ SPEC COMPLETED    │
└─────────┴───────────────────────────────────────────────┴───────────────────┘
```

---

## Phase 1: Architectural Foundations & Formal Specifications (Status: COMPLETE)
*Objective: Define the normative mathematical, syntactic, and binary layout foundations.*

- [x] **Core Language EBNF Grammar**: Completed in [SPECIFICATION.md](SPECIFICATION.md).
- [x] **14-Level Operator Precedence Matrix (`ENG-SPEC-03.1`)**: Completed in [OPERATORS.md](OPERATORS.md).
- [x] **Single-Level Store Binary Specification (`ENG-SLS-01.1`)**: Superblock layout, 64-byte alignment, CRC64-NVME in [ENGINE_DB_SLS_LAYOUT.md](ENGINE_DB_SLS_LAYOUT.md).
- [x] **Lazy Schema Evolution Protocol (`ENG-MIG-01`)**: Field addition/deprecation rules in [SCHEMA_EVOLUTION.md](SCHEMA_EVOLUTION.md).
- [x] **Memory Consistency Model (`ENG-MEM-01.1`)**: Monotonic epoch arenas and causal happened-before in [MEMORY_MODEL.md](MEMORY_MODEL.md).
- [x] **P2P Wire & Delta-CRDT Replication Protocol (`ENG-WIRE-01.1`)**: Gossip synchronization in [DISTRIBUTED_MESH_WIRE.md](DISTRIBUTED_MESH_WIRE.md).
- [x] **Module System & Package Standard (`ENG-PKG-01.1`)**: Manifest semantics in [MODULES_AND_PACKAGES.md](MODULES_AND_PACKAGES.md).
- [x] **SMT Refinement & Domain Invariant Theory (`ENG-SMT-01.1`)**: Predicate proving in [REFINEMENT_TYPES_SMT.md](REFINEMENT_TYPES_SMT.md).
- [x] **Foreign Interfaces & PostgreSQL Wire Bridge (`ENG-FFI-01.1`)**: Port 5432 catalog emulation in [FFI_AND_SYSTEM_BRIDGE.md](FFI_AND_SYSTEM_BRIDGE.md).

---

## Phase 2: Bootstrap Compiler & Verification Engine (`enginec`) (Status: ACTIVE & VERIFIED)
*Objective: Provide a fully verifiable reference compiler and execution harness.*

- [x] **Deterministic LL(1) Compiler**: Modular Rust implementation with zero-backtracking lexer and recursive-descent parser.
- [x] **Automated Z3 SMT Verification (`enginec check`)**: Translates refinement constraints and invariants into first-order SMT-LIB2 logic to mathematically prove code correctness at compile time.
- [x] **SLS Binary Engine (`engine.db`)**: Direct binary page management with atomicity protection (`clwb` + `sfence` simulation) and journal tracking.
- [x] **Embedded PostgreSQL Wire Protocol Server (`bridge.rs`)**: Emulated port 5432 responder allowing DBeaver, psql, and BI tools to inspect `engine.db`.
- [x] **Embedded Web Playground & Visualizer (`web.rs`)**: In-memory inspection dashboard and HTTP runtime.
- [x] **Docker Containerized Validation (`engine-genesis-node`)**: Full environment test harness enforcing rule `0.1`.

---

## Phase 3: Polyglot Database Connectors & Frictionless DX (Target: Q1-Q2 2027)
*Objective: Eliminate enterprise adoption barriers by granting developers complete database freedom while focusing 100% on domain logic rather than runtime mechanics.*

### 3.1 Zero-Friction Storage Paradigm
- [ ] **Default Zero-Config SLS (`engine.db`)**:
  - The default out-of-the-box storage remains the embedded Single-Level Store. No database setup, no connection strings, zero external daemon requirements.
- [ ] **Declarative Pluggable Datasource Specification (`@datasource`)**:
  - Enable entities and modules to seamlessly bind to external database systems:
    ```engine
    datasource EnterpriseWarehouse {
        driver: postgres,  // postgres | mysql | sqlite | clickhouse | mongodb
        uri: env("DATABASE_URL"),
        mode: read_write   // read_write | read_only | write_through | mirror
    }

    @datasource(EnterpriseWarehouse, table: "accounts")
    entity Account {
        id: UUIDv7,
        balance: Decimal(18, 4),
        invariant { self.balance >= 0.0000 }
    }
    ```
- [ ] **Transparent Infrastructure Orchestration**:
  - The compiler and runtime handle connection pooling, reconnect backoff, schema mapping, and dialect translation automatically.
  - The developer interacts with entities purely as native structs without manual SQL, query builders, or ORM annotations.
- [ ] **Dual-Layer SLS Caching & CDC Write-Through**:
  - High-throughput local operations execute with nanosecond SLS performance, backed by asynchronous change data capture (CDC) streaming back to enterprise legacy databases.

### 3.2 Developer Experience (DX) & Tooling
- [ ] **Language Server Protocol (`engine-lsp`)**:
  - Real-time compile-time diagnostics in VS Code, Neovim, and JetBrains.
  - Live SMT counterexample visualization directly on the editor margin when an invariant fails.
- [ ] **Automated Formatter & Linter (`enginec fmt`, `enginec lint`)**:
  - Enforce canonical formatting and detect unverified code branches.
- [ ] **Package Registry Client (`enginec add`, `enginec publish`)**:
  - Decentralized package resolution via Git remotes and content-addressed hashes.

---

## Phase 4: Silicon DAX & Native Codegen (Target: Q3-Q4 2027)
*Objective: Transform Engine Lang into a high-performance native ahead-of-time (AOT) compiled language.*

- [ ] **Native LLVM / Cranelift Backend**:
  - Replace the AST interpreter with ahead-of-time machine code generation for `x86_64`, `aarch64`, and `riscv64`.
- [ ] **Direct Silicon Instruction Synthesis**:
  - Emit hardware `clwb` (Cache Line Write Back) and `sfence` instructions on x86_64 for atomic durable memory transactions.
  - Emit `DC CVAP` and `DSB ISH` on ARM64 platforms.
  - Vectorized SIMD compilation for AI cosine similarity (`<=>`) using AVX-512 and ARM NEON.
- [ ] **Linux Direct Access (DAX) Kernel Driver**:
  - Direct zero-copy memory mapping (`MAP_SHARED_VALIDATE | MAP_SYNC`) over `/dev/pmem0` and CXL 2.0/3.0 memory interconnects.

---

## Phase 5: Industrial Automation & Hard Real-Time Systems (Target: 2028)
*Objective: Enable deployment in mission-critical manufacturing, SCADA, automotive, and aerospace environments.*

- [ ] **Industrial Protocol Adapters**:
  - Native, verified drivers for standard industrial fieldbuses: Modbus TCP/RTU, OPC-UA, CANopen, and Siemens S7.
- [ ] **Bounded-Jitter Real-Time Scheduling**:
  - Deterministic epoch progression with zero GC latency, guaranteeing microsecond response deadlines for safety loops.
- [ ] **Memory-Mapped I/O Primitives (`@mmio`)**:
  - Direct interaction with hardware registers and FPGA peripherals:
    ```engine
    @mmio(address: 0x4000_1000, size: 32)
    register ValveActuatorControl {
        enable: Bit,
        pressure_cutoff: Bits(7)
    }
    ```
- [ ] **Formal Safety Certification Dossier**:
  - Tool qualification artifacts and formal proofs adhering to IEC 61508 / ISO 26262 safety standards.

---

## Phase 6: Planetary Causal Mesh & Autonomous Swarms (Target: 2028-2029)
*Objective: Enable geo-distributed resilience without central coordinators, cloud locks, or single points of failure.*

- [ ] **Native Wire QUIC & RDMA Transport**:
  - High-throughput, zero-copy packet exchange over peer-to-peer transport layers.
- [ ] **Autonomous Delta-CRDT Gossip Sync**:
  - Self-healing state synchronization across transatlantic peer clusters with vector-clock conflict resolution.
- [ ] **Autonomous AI Agent Swarm Primitives**:
  - Native linguistic primitives for spawning autonomous agents whose operational bounds and actions are strictly constrained by compile-time SMT invariants.

---

## Phase 7: Sovereign Single-Level Store Bare-Metal OS (Target: 2029+)
*Objective: Boot Engine Lang directly on bare silicon as an autonomous, persistent Unikernel Operating System.*

- [ ] **Freestanding Bootloader (`engine-boot`)**:
  - Direct UEFI initialization for x86_64 and ARM64 without Linux or host OS dependencies.
- [ ] **Single-Level Store Flat Address Space**:
  - Elimination of the kernel/user-space dichotomy. Memory *is* the filesystem, and execution is persistent across power cycles.
- [ ] **Engine Lang Hardware Abstraction Layer (HAL)**:
  - Storage controllers (NVMe, CXL), timer interrupts, and network interfaces implemented directly in verified `.eng` code.
