# Continuum Programming Language (`ctm`)

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](#)
[![Spec Version](https://img.shields.io/badge/spec-CTM--SPEC--02.1-orange.svg)](docs/SPECIFICATION.md)
[![SLS Binary](https://img.shields.io/badge/nvram-CTM--SLS--01.1-purple.svg)](docs/NVRAM_SLS_LAYOUT.md)

> **Continuum** is an infrastructure-agnostic, statically typed systems programming language that unifies computation, persistent memory (Single-Level Store), and distributed causal replication into a single compilation unit.

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

### The Continuum Architectural Thesis

With the arrival of **Byte-Addressable Non-Volatile RAM (NVRAM)**, **Compute Express Link (CXL)** buses, multi-hundred-gigabit network fabrics, and advanced **SMT solvers (Z3)**, the separation between memory and database is an obsolete historical accident.

Continuum collapses the entire operational stack into the compiled binary itself:

```
THE CONTINUUM ARCHITECTURE: UNIFIED SILICON COMPUTATION

  [ Edge Client / Traffic ]
             │
             ▼
  ┌─────────────────────────────────────────────────────────────────┐
  │ CONTINUUM COMPILED BINARY (`ctmc`)                              │
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
| **2. Deterministic Operator Matrix** | Closed, strictly ordered 14-level precedence table (`CTM-SPEC-02.1`). Zero dynamic syntax, zero LL(1) backtracking. | SIMD vector math (`@*`, `<=>`), pipeline composition (`|>`), and atomic transaction collapse (`?!`). |
| **3. Formal SMT Refinements** | Compile-time semantic verification using integrated Z3 SMT solver. Type refinement bounds (`where`) are mathematical theorems. | If an invariant can be broken by any execution path, compilation halts with a formal counterexample. |
| **4. Zero-Downtime LSE-CRC** | Lazy Structural Evolution with Causal Redirection. Schemas evolve without stopping or re-indexing physical memory. | Old layout nodes coexist with new layouts. Fields synthesize defaults in CPU registers on read; writes reallocate. |
| **5. Distributed Causal Mesh** | Conflict-Free Replicated Data Types (CRDTs) and causal vector timestamps replicated peer-to-peer. | Native QUIC/RDMA transport with automatic partition recovery and zero central master nodes. |

---

## 3. The Continuum Language at a Glance

### Refined Types, Invariants, and Persistent State

```continuum
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

```continuum
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
continuum-lang/
├── .github/
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── rfc_proposal.md
│   └── workflows/
│       └── ci.yml
├── docs/
│   ├── SPECIFICATION.md       # EBNF grammar, directives, ADTs, boundary FFI
│   ├── OPERATORS.md           # 14-level operator matrix & SMT algebraic contracts
│   ├── NVRAM_SLS_LAYOUT.md    # Binary specification of .nvram SLS containers
│   ├── SCHEMA_EVOLUTION.md    # Lazy structural evolution protocol (LSE-CRC)
│   ├── MEMORY_MODEL.md        # Causal memory consistency model & hardware flushes
│   ├── DISTRIBUTED_MESH_WIRE.md # P2P wire protocol (ctm-wire) & Delta-CRDT sync
│   ├── MODULES_AND_PACKAGES.md# Package manager (ctm-pkg) & 4-tier visibility
│   ├── REFINEMENT_TYPES_SMT.md# SMT refinement theory, VC generation & timeouts
│   ├── FFI_AND_SYSTEM_BRIDGE.md # POSIX C FFI & embedded PostgreSQL wire bridge
│   └── ROADMAP.md             # Complete roadmap from Phase 1 to Phase 5
├── compiler/                  # Reference bootstrap compiler (ctmc) in Rust
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # CLI entry point (check, run, inspect, bridge)
│       ├── ast.rs             # Complete Abstract Syntax Tree definitions
│       ├── lexer.rs           # Deterministic LL(1) tokenization engine
│       ├── parser.rs          # Recursive-descent LL(1) parser
│       ├── smt.rs             # Symbolic translation & SMT-LIB2 verification layer
│       ├── memory.rs          # Raw NVRAM superblock, journal, and entity layouts
│       ├── interpreter.rs     # Runtime evaluation & causal simulation engine
│       └── bridge.rs          # Embedded PostgreSQL v3.0 wire protocol responder
├── examples/
│   ├── 01_hello_causal.ctm    # Basic atomic mutation and causal logging
│   ├── 02_persistent_vehicle_fleet.ctm  # Canonical fleet domain with IPVA history
│   └── 03_vector_ai_search.ctm# Native 1536-dim vector cosine search
├── Continuum.toml             # Workspace manifest & memory engine configuration
├── CONTRIBUTING.md            # RFC process, proof guidelines & coding standards
├── LICENSE                    # Apache License 2.0
└── README.md                  # This engineering manifesto
```

---

## 5. Quickstart Guide

### Prerequisites
- Rust 1.80+ (for building the `ctmc` reference bootstrap compiler)
- Linux Kernel 5.4+ with DAX support (or any POSIX environment for simulation)
- Optional: Z3 Theorem Prover (`z3`) for formal verification backend

### 1. Clone the Workspace
```bash
git clone https://github.com/continuum-lang/continuum.git
cd continuum
```

### 2. Build the Reference Bootstrap Compiler (`ctmc`)
```bash
cd compiler
cargo build --release
```

### 3. Formally Verify Language Examples
```bash
# Run formal type-checking, refinement invariant checks, and SMT proofs
cargo run -- check ../examples/01_hello_causal.ctm
cargo run -- check ../examples/02_persistent_vehicle_fleet.ctm
cargo run -- check ../examples/03_vector_ai_search.ctm
```

### 4. Execute Bootstrap Simulations
```bash
# Execute sample programs under the causal runtime
cargo run -- run ../examples/01_hello_causal.ctm
cargo run -- run ../examples/02_persistent_vehicle_fleet.ctm
cargo run -- run ../examples/03_vector_ai_search.ctm
```

### 5. Inspect Binary NVRAM Containers
```bash
# Inspect the raw physical binary layout of the Single-Level Store
cargo run -- inspect ../data/continuum.nvram
```

---

## 6. A Call to Systems Engineers, Compiler Architects, and Researchers

Continuum is not another framework; it is an open-specification movement to reclaim sovereign, hyper-efficient computing. We invite:

1. **Compiler Engineers**: Help expand LLVM code-generation backends targeting native x86-64 `clwb`/`sfence` and ARM64 `DC CVAP`.
2. **Distributed Systems Researchers**: Refine our P2P causal mesh, CRDT delta synchronizers, and RDMA zero-copy network transports.
3. **Formal Verification Mathematicians**: Broaden our SMT translation layer (`compiler/src/smt.rs`) to support automated termination analysis and inductive invariants.
4. **Systems Programmers**: Stress-test the Single-Level Store file format against abrupt power failures and kernel panic simulations.

Review our [CONTRIBUTING.md](CONTRIBUTING.md) and submit your first RFC today.

---

## 7. License

Licensed under the **Apache License, Version 2.0** ([LICENSE](LICENSE)). You may reproduce, distribute, and modify this work in compliance with the License.

