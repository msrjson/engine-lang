# Engine Lang Mission-Critical Benchmark Registry

**Specification Reference**: `ENG-SPEC-03.1` / `ENG-SLS-01.1` / `ENG-BENCH-01.0`  
**Author**: Lysander Vance & The Vance Protocol Group / Engine Lang Core Working Group  
**Status**: Official Continuous Benchmark Standard  
**License**: Apache License 2.0  

---

## 1. Overview & Policy

This directory houses the continuous benchmark suites, raw telemetry, published reports, and anomaly tracking for **Engine Lang**.

Engine Lang is designed for deterministic, mission-critical execution. In accordance with zero-compromise engineering principles, every performance evaluation—from routine development tests to multi-stack stress tests—must be cataloged and published.

Whenever a benchmark reveals an unexpected bottleneck, memory spike, or correctness failure, an official **[ENG-ANOM](advisories/)** advisory is registered following the CVE standard.

---

## 2. Benchmark Categories

Reports are organized under `reports/` by category and timestamp:

1. **`reports/runtimes/`**: Language and runtime confrontation (Engine Lang vs C, C++, Rust, Go, Python).
2. **`reports/databases/`**: Storage engine confrontation (Engine Lang Silicon Single-Level Store vs PostgreSQL, MySQL/InnoDB, Oracle, MongoDB, Supabase, SQLite).
3. **`reports/frameworks/`**: Full-stack web and application gateway confrontation (Engine Forge vs Laravel, FastAPI, Axum, Spring Boot).
4. **`reports/stress/`**: Extreme stress, chaos engineering (SIGKILL crash consistency, connection pool saturation, SMT refinement explosion).

---

## 3. Directory Layout

```
benchmarks/
├── README.md               # This document
├── INDEX.md                # Automatically maintained Master Catalog of all benchmark runs
├── advisories/             # Mission-critical CVE-style Anomaly Tracking (ENG-ANOM)
├── reports/                # Published results partitioned by category and date/time
├── suite/                  # Executable workload code across all languages and databases
└── tools/                  # Testbed runners, telemetry probes, and index generators
```

---

## 4. Report Publishing Format

Every published run directory must follow the naming convention `YYYY-MM-DD_HHMMSS_<slug>/` and contain three mandatory artifacts:

1. **`benchmark.json`**: Structured machine-readable telemetry including request volume, concurrency, latency percentiles (P50, P90, P99, P99.9, P99.99), throughput (req/s), error counts, and system resource metrics (RSS, peak memory, CPU time).
2. **`report.md`**: Executive markdown analysis detailing workload scenario, comparison tables, latency distributions, and architectural findings.
3. **`env.json`**: Hardware and environment context (CPU model, core count, RAM, kernel version, compiler build flags, container environment).

---

## 5. Container Verification Mandate (Rule 0.1)

All official benchmark results reported in this repository are executed and verified inside containerized environments (`engine-genesis-node` / `benchmarks/tools/docker-compose.bench.yml`). Results collected on host environments without container isolation are marked `UNVERIFIED`.

