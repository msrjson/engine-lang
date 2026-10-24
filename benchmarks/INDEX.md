# Engine Lang Benchmark Catalog & Anomaly Registry

**Specification Reference**: `ENG-SPEC-03.1` / `ENG-BENCH-01.0`  
**Authority**: Lysander Vance & The Vance Protocol Group / Engine Lang Core Working Group  
**Last Synchronized**: 2026-10-24 11:29:05 UTC  
**License**: Apache License 2.0  

---

## 1. Master Benchmark Execution Index

All benchmarks recorded across all categories, environments, and stress levels.

| Date & Time (UTC) | Category | Benchmark Title | Confrontation Targets | Verification | Artifacts |
|---|---|---|---|:---:|---|
| 2026-10-24T11:25:14.669570+00:00 | **`runtimes`** | Runtime Confrontation: Engine Lang vs C, C++, Rust, Go, Python | Standard Suite | 🟢 `CONTAINER` | [Report](reports/runtimes/2026-10-24_112514_languages_confrontation/report.md) / [JSON](reports/runtimes/2026-10-24_112514_languages_confrontation/benchmark.json) |
| 2026-10-24T11:25:14.836727+00:00 | **`databases`** | Storage Confrontation: Engine Lang SLS vs SQLite, PostgreSQL, MySQL | Standard Suite | 🟢 `CONTAINER` | [Report](reports/databases/2026-10-24_112514_storage_confrontation/report.md) / [JSON](reports/databases/2026-10-24_112514_storage_confrontation/benchmark.json) |
| 2026-10-24T12:08:00Z | **`frameworks`** | Engine Forge vs Laravel 13 Production Benchmark | Engine Forge / Engine Lang vs. Laravel Framework | 🟢 `CONTAINER` | [Report](reports/frameworks/2026-10-24_120800_engine_vs_laravel/report.md) / [JSON](reports/frameworks/2026-10-24_120800_engine_vs_laravel/benchmark.json) |

---

## 2. Mission-Critical Defect & Anomaly Log (`ENG-ANOM`)

System defects, tail-latency regressions, memory growth incidents, and invariant breaches discovered during benchmarking.

| Advisory ID | Severity | Status | Title | Document |
|---|---|:---:|---|---|
| — | — | — | *Zero open anomalies recorded. System invariants hold.* | [Advisory Policy](advisories/README.md) |

---

## 3. Methodology & Governance

1. **Adversarial Confrontation**: We benchmark against the fastest production systems (C, C++, Rust, Go, Python, PostgreSQL, MySQL, SQLite, MongoDB, Supabase).
2. **Reproducibility Mandate**: Every run can be reproduced via `benchmarks/tools/docker-compose.bench.yml`.
3. **Mission-Critical Integrity**: We publish failures as aggressively as victories. Every defect discovered is assigned an `ENG-ANOM` identifier and patched with formal verification.

