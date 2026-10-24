# Engine Forge vs. Laravel 13: Production Latency & Performance Benchmark

**Specification Reference**: `ENG-SPEC-03.1` / `ENG-SLS-01.1`  
**Author**: Lysander Vance & The Vance Protocol Group / Engine Lang Core Working Group  
**Status**: Official Technical Evaluation  
**License**: Apache License 2.0  

---

## 1. Overview & Evaluation Goals

This study evaluates the performance, throughput, latency profiles, and hardware resource consumption of **Engine Forge** (running natively on **Engine Lang** with silicon Single-Level Store persistence) against **Laravel 13** (PHP 8.3 CLI Server with OPcache JIT, compiled route/config/view caches, and SQLite persistence).

The goal of this benchmark is to discover the production ceiling of the unified Silicon Persistence architecture in comparison with the traditional multi-tiered software architecture.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                   ARCHITECTURAL COMPARISON: DATA PIPELINE                   │
├─────────────────────────────────────────────────────────────────────────────┤
│ TRADITIONAL STACK (Laravel):                                                │
│ HTTP Client ──> Web Gateway ──> Middleware Stack ──> Eloquent ORM          │
│             ──> SQL Generator ──> PDO Driver ──> Database Engine ──> fsync  │
├─────────────────────────────────────────────────────────────────────────────┤
│ SILICON PERSISTENCE (Engine Forge):                                         │
│ HTTP Client ──> Zero-Copy @boundary ──> SMT Invariant Proof                 │
│             ──> Direct SLS Memory Map (DAX) ──> CLWB + SFENCE Hardware      │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Benchmark Environment & Methodology

- **Host Environment**: x86_64 Linux 6.12 kernel, AMD Ryzen 7 / Intel Xeon architecture.
- **Container Isolation**:
  - `engine-framework-test` (`engine-lang-engine-genesis`): HTTP Web Gateway running on port `8090` (mapped to `8091`), Single-Level Store database `engine_test.db` mounted via volume.
  - `engine-laravel-benchmark` (`php:8.3-cli-alpine`): Running on port `8000`, OPcache JIT enabled (`opcache.enable=1`, `opcache.jit=tracing`), SQLite database seeded with identical records.
- **Test Harness**: Independent containerized benchmark client (`python:3.14-slim`) connected via host networking executing concurrent keep-alive HTTP workers.
- **Workload**: 1,000 requests per scenario across 25 concurrent worker threads, preceded by a 50-request warm-up run.

---

## 3. Results Matrix

| Scenario | Request Pattern | Engine Lang (Req/s) | Laravel 13 (Req/s) | Latency P50 (Engine vs Laravel) | Latency P99 (Engine vs Laravel) | Speedup Factor |
|---|---|:---:|:---:|:---:|:---:|:---:|
| **1. SSR HTML** | `GET /` (Welcome Page) | **3,820.7** | 25.9 | **5.79 ms** vs 731.81 ms | **11.02 ms** vs 6,340.44 ms | **147.3x** |
| **2. Read API (Simple)** | `GET /api/v1/categories` (11 Items) | **3,947.9** | 32.9 | **5.57 ms** vs 613.04 ms | **11.61 ms** vs 2,270.43 ms | **120.0x** |
| **3. Read API (Graph)** | `GET /api/v1/posts` (11 Rich Entities) | **3,465.2** | 27.8 | **6.33 ms** vs 533.78 ms | **18.66 ms** vs 7,402.99 ms | **124.8x** |
| **4. Auth Session Gate** | `POST /api/v1/auth/login` (JSON payload) | **4,275.3** | 36.2 | **5.26 ms** vs 618.87 ms | **10.11 ms** vs 1,442.13 ms | **118.0x** |
| **5. Transactional Mutation** | `POST /api/v1/posts` (Write + Invariant) | **5,918.1** | 27.2 | **3.68 ms** vs 720.40 ms | **9.13 ms** vs 2,087.22 ms | **217.2x** |

---

## 4. Hardware Resource Footprint

Measurements captured live during execution via `docker stats`:

| Parameter | Engine Lang Container | Laravel 13 Container | Operational Advantage |
|---|:---:|:---:|---|
| **Resident Memory (RAM)** | **840 KiB** (~0.82 MiB) | **31.9 MiB** | **38.9x lower RAM footprint** |
| **Peak Memory Under Load** | **1.2 MiB** | **32.6 MiB** | **27.1x lower memory peak** |
| **Container Image Size** | **56.4 MB** | **210+ MB** | **3.7x smaller image** |
| **Cold Start Boot Time** | **< 4 ms** | **~280 ms** | **70x faster bootstrap** |

---

## 5. Architectural Findings & Key Takeaways

1. **Elimination of the Object-Relational Impedance Mismatch**:
   Traditional frameworks waste significant CPU cycles hydrating SQL tuples into in-memory objects and re-serializing them into JSON. Engine Lang entities reside in contiguous Single-Level Store memory buffers, streaming directly to network sockets without memory copying.

2. **Silicon Durability vs. File-System fsync**:
   In Scenario 5 (Transactional Mutation), traditional databases must serialize statements to write-ahead logs and execute synchronous `fsync()` system calls to prevent corruption. Engine Lang executes mutations inside atomic blocks compiled into hardware cache-line flushes (`CLWB + SFENCE`), achieving **5,900+ req/s** directly on physical storage.

3. **SMT Formal Proofs Eliminate Dynamic Validation Overhead**:
   Dynamic frameworks execute regexes, reflection, and database existence queries on every request. Engine Lang verifies all domain contracts and refinement constraints statically with Z3 SMT at compile-time, reducing runtime input validation to pure pointer bounds checking.

4. **Extreme Infrastructure Density**:
   Because each Engine Lang node consumes under 1 MiB of RAM, cloud hosts can accommodate hundreds of independent multi-tenant environments on standard commodity hardware, drastically reducing cloud hosting expenditure.
