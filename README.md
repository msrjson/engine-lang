# Engine Lang Programming Language (`engine`)

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](#)
[![Version](https://img.shields.io/badge/version-v0.3.1--silicon-blue.svg)](CHANGELOG.md)
[![Website & Book](https://img.shields.io/badge/website-online-success.svg)](https://enginelang.org/)
[![MSR JSON Protocol](https://img.shields.io/badge/MSR--JSON-2.1.0-blue.svg)](msr.json)
[![Spec Version](https://img.shields.io/badge/spec-ENG--SPEC--03.1-orange.svg)](docs/SPECIFICATION.md)
[![SLS Binary](https://img.shields.io/badge/db-ENG--SLS--01.1-purple.svg)](docs/ENGINE_DB_SLS_LAYOUT.md)
[![Codegen Spec](https://img.shields.io/badge/codegen-ENG--CG--01.1-teal.svg)](docs/ENG_SILICON_CODEGEN_SPEC.md)
[![Unikernel OS](https://img.shields.io/badge/unikernel-ENG--OS--01.1-red.svg)](docs/BARE_METAL_UNIKERNEL_SPEC.md)
[![Backlog](https://img.shields.io/badge/backlog-active-informational.svg)](BACKLOG.md)
[![Official Book](https://img.shields.io/badge/book-Lysander_Vance-success.svg)](book/SUMMARY.md)

> **Engine Lang** is an open, unowned, statically typed systems programming language that unifies computation, persistent memory (Single-Level Store), and distributed causal replication into a single compilation unit. Released under the **Apache 2.0 License** as a sovereign digital public good for human developers and AI agents alike.

📖 **Official Book**: Read [*From the Silicon Island to the Causal Cosmos*](book/SUMMARY.md) by **Lysander Vance** — exploring software from a single-node memory ledger to global causal mesh networks.

---

## 1. The Manifesto: Abolishing the Cloud Fragmentation Tax

For five decades, enterprise software architecture has been trapped in the **Von Neumann memory-storage duality**. In the 1970s, RAM was microscopic, volatile, and expensive, while magnetic disk drives were slow, mechanical, and block-oriented. This historical limitation forced computing to bifurcate into two decoupled worlds: **volatile runtime code** and **external persistent storage**.

To bridge this synthetic divide in modern distributed systems, the software industry invented what we designate as the **Cloud Fragmentation Tax**:

```
THE LEGACY FRAGMENTED ENTERPRISE STACK
┌─────────────────────────────────────────────────────────────┐
│ 1. Application Process (FastAPI / Node / Go / Spring / Rust)│
├─────────────────────────────────────────────────────────────┤
│ 2. Reverse Proxy / Ingress Gateway (Nginx / Envoy / Traefik)│
├─────────────────────────────────────────────────────────────┤
│ 3. Object-Relational Mapper (Hibernate / Prisma / SQLAlchemy│
├─────────────────────────────────────────────────────────────┤
│ 4. Relational Database (PostgreSQL / MySQL / Oracle RAC)    │
├─────────────────────────────────────────────────────────────┤
│ 5. Ephemeral In-Memory Cache (Redis / KeyDB / Memcached)    │
├─────────────────────────────────────────────────────────────┤
│ 6. Message Broker / Streaming Queue (Kafka / RabbitMQ / SQS)│
├─────────────────────────────────────────────────────────────┤
│ 7. Specialized Vector Database (Pinecone / Milvus / Qdrant) │
├─────────────────────────────────────────────────────────────┤
│ 8. Consensus & Service Discovery (Consul / Zookeeper / Raft)│
├─────────────────────────────────────────────────────────────┤
│ 9. Distributed Orchestration (Kubernetes / Helm / Istio)    │
├─────────────────────────────────────────────────────────────┤
│ 10. Proprietary Hyperscaler Cloud Billing (AWS / GCP / Azure)│
└─────────────────────────────────────────────────────────────┘
```

Every single boundary between these 10 layers incurs:
- **Massive Latency Overhead**: Up to 30% of total CPU cycles in microservices are burned serializing and deserializing JSON, Protobuf, and SQL strings.
- **Cross-AZ Transit Fees**: Corporations pay hundreds of thousands of dollars each month simply transmitting identical payloads back and forth between virtual machines within the same physical data center.
- **Cascading Failure Modes**: If any one service suffers a connection pool exhaustion, deadlock, or network partition, the entire application collapses.
- **Anti-Competitive Lock-In**: Proprietary managed database triggers, Lambdas, and serverless queues artificially lock enterprises into hyperscaler clouds.

### The Engine Lang Architectural Thesis

With the arrival of **Byte-Addressable Non-Volatile RAM (NVRAM)**, **Compute Express Link (CXL)** buses, multi-hundred-gigabit network fabrics, and advanced **SMT solvers (Z3)**, the separation between memory and database is an obsolete historical accident.

Engine Lang collapses the entire operational stack into the compiled binary itself:

```
THE ENGINE LANG ARCHITECTURE: UNIFIED SILICON COMPUTATION

  [ Edge Client / Traffic ]
             │
             ▼
  ┌─────────────────────────────────────────────────────────────────┐
  │ ENGINE LANG COMPILED BINARY (`enginec`)                              │
  │                                                                 │
  │  ┌───────────────────────┐         ┌─────────────────────────┐  │
  │  │ HTTP/gRPC `@boundary` │         │ P2P Causal Mesh (QUIC)  │  │
  │  └──────────┬────────────┘         └────────────┬────────────┘  │
  │             │                                   │               │
  │             ▼                                   ▼               │
  │  ┌───────────────────────────────────────────────────────────┐  │
  │  │ Deterministic LL(1) Operator Matrix & Pipeline (`|>`, `?!`)│  │
  │  └──────────────────────────┬────────────────────────────────┘  │
  │                             │                                   │
  │                             ▼                                   │
  │  ┌───────────────────────────────────────────────────────────┐  │
  │  │ Compile-Time SMT Invariants & Refinement Proofs (`where`) │  │
  │  └──────────────────────────┬────────────────────────────────┘  │
  │                             │                                   │
  │                             ▼                                   │
  │  ┌───────────────────────────────────────────────────────────┐  │
  │  │ Single-Level Store (SLS) on NVRAM / CXL.mem (`@persistent`)│  │
  │  │ (Direct Access DAX, Cache-Line Flush `clwb` + `sfence`)   │  │
  │  └───────────────────────────────────────────────────────────┘  │
  └─────────────────────────────────────────────────────────────────┘
                                │
               (Raw Linux VPS / Bare Metal / Edge)
```

No external database. No Redis cache. No Kafka broker. No ORM. Memory **is** storage.

---

## 2. Core Architectural Pillars

| Pillar | Mechanism | Silicon & Runtime Mapping |
|---|---|---|
| **1. Single-Level Store (SLS)** | `@persistent entity` declarations live directly in a unified 64-bit virtual address space backed by NVRAM/CXL. | Direct Access (DAX) without kernel page cache copy. Atomic commits compile to hardware `clwb` + `sfence` instructions. |
| **2. Deterministic Operator Matrix** | Closed, strictly ordered 14-level precedence table (`ENG-SPEC-03.1`). Zero dynamic syntax, zero LL(1) backtracking. | SIMD vector math (`@*`, `<=>`), pipeline composition (`|>`), and atomic transaction collapse (`?!`). |
| **3. Formal SMT Refinements** | Compile-time semantic verification using integrated Z3 SMT solver. Type refinement bounds (`where`) are mathematical theorems. | If an invariant can be broken by any execution path, compilation halts with a formal counterexample. |
| **4. Zero-Downtime LSE-CRC** | Lazy Structural Evolution with Causal Redirection. Schemas evolve without stopping or re-indexing physical memory. | Old layout nodes coexist with new layouts. Fields synthesize defaults in CPU registers on read; writes reallocate. |
| **5. Distributed Causal Mesh** | Conflict-Free Replicated Data Types (CRDTs) and causal vector timestamps replicated peer-to-peer. | Native QUIC/RDMA transport with automatic partition recovery and zero central master nodes. |

---

## 3. Engine Lang at a Glance

### Refined Types, Invariants, and Persistent State

```engine
module Logistics::Fleet;

// Type Refinement verified at compile time via Z3 SMT
type PlateNumber = Text where len(value) == 7 && is_alphanumeric(value);
type Money = Decimal(18, 4) where value >= 0.0000;

// Persistent entity mapped directly to NVRAM
@persistent
@TenantGuard(strict: true)
entity Vehicle {
    id: UUIDv7,
    plate: PlateNumber,
    model: Text,
    year: Int where value >= 1900 && value <= 2100,
    market_value: Money,
    ipva_records: List<TaxAssessment>,
    embedding: Vector(1536),

    // Domain invariant proven by SMT solver
    invariant {
        self.market_value >= 0.0000 &&
        len(self.plate) == 7
    }
}

// Atomic transaction with hardware persistence
action register_tax(vehicle: &mut Vehicle, assessment: TaxAssessment) -> Result<(), Error> {
    atomic {
        vehicle.ipva_records.push(assessment);
        vehicle.market_value = assessment.depreciated_value?!;

        // Emits CLWB and SFENCE automatically at block boundary
    }
    return Ok(());
}
```

### High-Dimensional Vector Search via Native SIMD Operator

```engine
action find_similar_vehicles(query_vector: Vector(1536)) -> List<Vehicle> {
    return Vehicle.query()
        .filter(v => (v.embedding <=> query_vector) >= 0.8500)
        |> order_by_descending(v => v.embedding <=> query_vector)
        |> limit(10);
}
```

---

## 4. Repository Structure

```
engine-lang/
├── .github/
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── rfc_proposal.md
│   └── workflows/
│       ├── ci.yml             # Automated compiler build and container verification
│       └── release.yml        # Cross-compilation & artifact release pipeline
├── book/                      # The Book of Engine Lang by Lysander Vance (14 Chapters & Appendices)
│   ├── SUMMARY.md             # Complete Table of Contents
│   └── src/                   # 14 authoritative engineering chapters & appendices
├── docs/
│   ├── SPECIFICATION.md       # EBNF grammar, directives, ADTs, boundary FFI
│   ├── OPERATORS.md           # 14-level operator matrix & SMT algebraic contracts
│   ├── ENGINE_DB_SLS_LAYOUT.md# Binary specification of .db SLS containers (CRC64-NVME)
│   ├── ENG_SILICON_CODEGEN_SPEC.md # Direct silicon codegen & Zero-UB ABI specification (ENG-CG-01.1)
│   ├── BARE_METAL_UNIKERNEL_SPEC.md# Sovereign bare-metal OS & UEFI unikernel spec (ENG-OS-01.1)
│   ├── CONFRONTING_THE_LISP_TRAP.md# Rebuttal of the Lisp Trap: empirical codebase confrontation (ENG-TRAP-01.1)
│   ├── SCHEMA_EVOLUTION.md    # Lazy structural evolution protocol (LSE-CRC)
│   ├── MEMORY_MODEL.md        # Causal memory consistency model & hardware flushes
│   ├── DISTRIBUTED_MESH_WIRE.md# P2P wire protocol (engine-wire) & Delta-CRDT sync
│   ├── MODULES_AND_PACKAGES.md# Package manager (engine-pkg) & 4-tier visibility
│   ├── REFINEMENT_TYPES_SMT.md# SMT refinement theory, VC generation & timeouts
│   ├── FFI_AND_SYSTEM_BRIDGE.md# POSIX C FFI & embedded PostgreSQL wire bridge
│   ├── ENGINE_VS_RUST_PARADIGM.md# Comprehensive architectural confrontation with Rust
│   ├── USE_CASES.md           # Killer use cases, business domains & cost analysis
│   ├── BRANDING_AND_IP.md     # Official branding, nomenclature and patent guidelines
│   └── ROADMAP.md             # Full lifecycle engineering roadmap (Phases 1 through 7)
├── compiler/                  # Reference bootstrap compiler (enginec) in Rust
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # CLI entry point (check, run, inspect, build, fmt, mesh, etc.)
│       ├── ast.rs             # Complete Abstract Syntax Tree definitions
│       ├── lexer.rs           # Deterministic LL(1) tokenization engine
│       ├── parser.rs          # Recursive-descent LL(1) parser
│       ├── smt.rs             # Symbolic translation & Z3 SMT-LIB2 verification layer
│       ├── memory.rs          # Raw Engine DB superblock, journal, and entity layouts
│       ├── interpreter.rs     # Runtime evaluation & causal simulation engine
│       ├── bridge.rs          # Embedded PostgreSQL v3.0 wire protocol responder
│       ├── web.rs             # Embedded HTTP boundary runtime & SLS visualizer
│       ├── codegen.rs         # Target-native LLVM IR & silicon assembly emitter with SIMD
│       ├── tooling.rs         # Developer tooling: formatter, static linter & LSP daemon
│       ├── mesh.rs            # Planetary causal mesh & Delta-CRDT vector clocks
│       └── unikernel.rs       # UEFI PE32+ bare-metal bootloader image builder
├── examples/                  # Standard library of 13 canonical .eng examples
│   ├── 01_hello_causal.eng    # Basic atomic mutation and causal logging
│   ├── 02_persistent_vehicle_fleet.eng # Fleet management with SMT refinement invariants
│   ├── 03_vector_ai_search.eng# Native 1536-dim vector cosine search (<=>)
│   ├── 04_causal_blog.eng     # Complete transactional blog engine with auth
│   ├── 05_polyglot_industrial_iot.eng # Dual-layer SLS write-through caching
│   ├── 06_industrial_plc_mmio.eng # Hardware MMIO (@mmio) with hard real-time bounds
│   └── 12_c_cpp_assembly_confrontation.eng # Zero-UB confrontation example
├── Engine.toml                # Workspace manifest & memory engine configuration
├── msr.json                   # MSR JSON 2.1 open protocol metadata manifest
├── CHANGELOG.md               # Keep a Changelog 1.1.0 specification history
├── BACKLOG.md                 # Production readiness gates & defect registry
├── RELEASING.md               # Systems-grade release engineering handbook
├── CONTRIBUTING.md            # RFC process, proof guidelines & coding standards
├── CODE_OF_CONDUCT.md         # Contributor Covenant v2.1 code of conduct
├── SECURITY.md                # Responsible security disclosure policy
├── LICENSE                    # Apache License 2.0
├── docker-compose.yml         # Genesis multi-service cluster definition
├── Dockerfile                 # Multi-stage release compiler & runtime container
└── README.md                  # This engineering manifesto
```

### The 5 Architectural Layers of Engine Lang

For developers exploring or contributing to the codebase, Engine Lang is systematically partitioned into five distinct, decoupled layers:

| Layer | System Domain | Key Source Components | Primary Specifications |
|:---:|---|---|---|
| **Layer 1** | **Deterministic Syntax & Formal SMT Invariants** | `compiler/src/lexer.rs`<br>`compiler/src/parser.rs`<br>`compiler/src/smt.rs`<br>`examples/*.eng` | [`docs/SPECIFICATION.md`](docs/SPECIFICATION.md)<br>[`docs/OPERATORS.md`](docs/OPERATORS.md)<br>[`docs/REFINEMENT_TYPES_SMT.md`](docs/REFINEMENT_TYPES_SMT.md) |
| **Layer 2** | **Single-Level Store (SLS) Memory Engine** | `compiler/src/memory.rs`<br>`data/engine.db` | [`docs/ENGINE_DB_SLS_LAYOUT.md`](docs/ENGINE_DB_SLS_LAYOUT.md)<br>[`docs/MEMORY_MODEL.md`](docs/MEMORY_MODEL.md)<br>[`docs/SCHEMA_EVOLUTION.md`](docs/SCHEMA_EVOLUTION.md) |
| **Layer 3** | **Causal Wire Mesh & Polyglot Protocols** | `compiler/src/mesh.rs`<br>`compiler/src/bridge.rs`<br>`compiler/src/web.rs` | [`docs/DISTRIBUTED_MESH_WIRE.md`](docs/DISTRIBUTED_MESH_WIRE.md)<br>[`docs/FFI_AND_SYSTEM_BRIDGE.md`](docs/FFI_AND_SYSTEM_BRIDGE.md) |
| **Layer 4** | **Direct Silicon Codegen & Sovereign Unikernel** | `compiler/src/codegen.rs`<br>`compiler/src/unikernel.rs` | [`docs/ENG_SILICON_CODEGEN_SPEC.md`](docs/ENG_SILICON_CODEGEN_SPEC.md)<br>[`docs/BARE_METAL_UNIKERNEL_SPEC.md`](docs/BARE_METAL_UNIKERNEL_SPEC.md) |
| **Layer 5** | **Genesis Toolchain & Developer Experience** | `compiler/src/main.rs`<br>`compiler/src/tooling.rs`<br>`Dockerfile`<br>`docker-compose.yml` | [`CONTRIBUTING.md`](CONTRIBUTING.md)<br>[`RELEASING.md`](RELEASING.md)<br>[`msr.json`](msr.json) |

---

## 5. Genesis Setup & Compilation Guide

Developers can build and run Engine Lang immediately using either the **Host Rust Toolchain** or the **Docker Genesis Container**.

### Option A: Genesis Docker Container (Zero-Setup, Recommended)

The Genesis Node container encapsulates the reference compiler, Z3 theorem prover headers, persistent storage, and exposed network bridges.

```bash
# 1. Clone the repository
git clone https://github.com/msrjson/engine-lang.git
cd engine-lang

# 2. Launch the genesis node service
docker compose up -d

# 3. Verify toolchain diagnostics & formal proof of canonical blog
docker exec engine-genesis-node sh -lc 'enginec version && enginec check /app/examples/04_causal_blog.eng'

# 4. Inspect the running Single-Level Store (.db)
docker exec engine-genesis-node sh -lc 'enginec inspect /app/data/engine.db'

# 5. Access the embedded services
# - Web Gateway & SLS Visualizer: http://localhost:8090
# - PostgreSQL Wire Bridge:       localhost:5442 (user: engine, db: engine_sls)
# - Causal Mesh Wire Protocol:    localhost:9443
```

### Option B: Local Native Toolchain (`cargo`)

Requires **Rust 1.80+** and **Z3 Theorem Prover 4.12+** (`z3`, `libz3-dev`).

```bash
# Debian / Ubuntu prerequisite installation:
sudo apt-get update && sudo apt-get install -y z3 libz3-dev pkg-config build-essential

# Build the release compiler binary:
cd compiler
cargo build --release

# Symlink or install binary onto your PATH:
sudo ln -sf $(pwd)/target/release/enginec /usr/local/bin/enginec

# Verify compilation:
enginec version
enginec check ../examples/01_hello_causal.eng
```

### Option C: VS Code Remote Containers / GitHub Codespaces

This repository includes a fully configured `.devcontainer/devcontainer.json`. Simply open this folder in **VS Code** with the *Remote - Containers* extension or launch a **GitHub Codespace** to receive a pre-configured environment with `rust-analyzer`, `z3`, and automated formatting on save.

---

## 6. Complete CLI Reference (`enginec`)

| Command | Purpose | Lifecycle Phase |
|---|---|---|
| `enginec version [--json]` | Displays comprehensive version, SMT prover, and specification diagnostics | Toolchain Standard |
| `enginec check <file.eng>` | Deterministic LL(1) parse and compile-time **Z3 SMT formal verification** | Phase 2 (SMT) |
| `enginec run <file.eng> [--db <path>]` | Executes application logic bound to persistent Single-Level Store | Phase 2 (Runtime) |
| `enginec inspect <path.db>` | Validates physical binary layout, Superblock, and **CRC-64/NVME** checksum | Phase 1 (SLS) |
| `enginec init-db [--path <path>]` | Formats a crash-consistent 64-byte aligned SLS container (`.db`) | Phase 1 (SLS) |
| `enginec bridge [--port 5432]` | Starts embedded PostgreSQL v3.0 protocol responder for DBeaver/psql | Phase 2 (Bridge) |
| `enginec serve [--http-port 8090]` | Concurrently launches Web visualizer gateway and PostgreSQL bridge | Phase 2 (Serve) |
| `enginec emit-ir <file.eng>` | Emits target-independent LLVM IR with `clwb` and `sfence` intrinsics | Phase 4 (Codegen) |
| `enginec emit-asm <file.eng>` | Synthesizes pure native assembly (`.s`) with AVX2 SIMD `<=>` acceleration | Phase 4 (Codegen) |
| `enginec build <file.eng>` | Ahead-of-time (AOT) builds a native standalone silicon binary | Phase 4 (AOT) |
| `enginec fmt <file.eng> [--write]` | Deterministically formats `.eng` source code into canonical styling | Phase 3 (DX) |
| `enginec lint <file.eng>` | Static linter checking refinement bounds, durability, and style rules | Phase 3 (DX) |
| `enginec lsp` | Runs Language Server Protocol (LSP 3.17) daemon over stdio | Phase 3 (DX) |
| `enginec mesh [--node-id <id>]` | Connects to Planetary Causal Mesh node with Delta-CRDT vector clocks | Phase 6 (Mesh) |
| `enginec boot-image [--output <efi>]` | Generates freestanding UEFI 2.10 PE32+ image (`engine-boot.efi`) | Phase 7 (Unikernel) |

---

## 7. A Call to Systems Engineers, Compiler Architects, and Researchers

Engine Lang is not another framework; it is an open-specification movement to reclaim sovereign, hyper-efficient computing. We invite:

1. **Compiler Engineers**: Help expand LLVM code-generation backends targeting native x86-64 `clwb`/`sfence` and ARM64 `DC CVAP`.
2. **Distributed Systems Researchers**: Refine our P2P causal mesh, CRDT delta synchronizers, and RDMA zero-copy network transports.
3. **Formal Verification Mathematicians**: Broaden our SMT translation layer (`compiler/src/smt.rs`) to support automated termination analysis and inductive invariants.
4. **Systems Programmers**: Stress-test the Single-Level Store file format against abrupt power failures and kernel panic simulations.

Review our [CONTRIBUTING.md](CONTRIBUTING.md) and submit your first RFC today.

---

## 8. License

Licensed under the **Apache License, Version 2.0** ([LICENSE](LICENSE)). You may reproduce, distribute, and modify this work in compliance with the License.

