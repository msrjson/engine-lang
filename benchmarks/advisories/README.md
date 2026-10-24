# Engine Lang Anomaly & Vulnerability Advisory System (`ENG-ANOM`)

**Specification Reference**: `ENG-SPEC-03.1` / `ENG-ANOM-01.0`
**Standard**: Mission-Critical Anomaly Tracking (CVE / DO-178C / ISO 26262 Aligned)
**Authority**: Lysander Vance & The Vance Protocol Group / Engine Lang Core Working Group
**License**: Apache License 2.0

---

## 1. Purpose & Guiding Principles

In mission-critical software engineering—such as aerospace flight control, autonomous industrial systems, high-frequency settlement, and silicon memory persistence—flaws, memory leaks, SMT proof escapes, and tail-latency anomalies cannot be swept into generic bug trackers or unrecorded git commits.

Every unexpected behavior, performance regression, data corruption, or runtime panic discovered during stress testing and comparative benchmarks must be:
1. Assigned a permanent, deterministic identifier: `ENG-ANOM-YYYY-NNNN`.
2. Scored using CVSS v3.1 / mission-critical criticality criteria (P1 to P4).
3. Documented with a deterministic, minimal reproduction script.
4. Analyzed through a microarchitectural Root Cause Analysis (RCA).
5. Resolved with an audited patch and an accompanying **Z3 SMT formal proof or regression test**.

---

## 2. Severity Classification Matrix

| Level | Severity | CVSS v3.1 | Criteria | Release Policy |
|---|---|---|---|---|
| **P1** | **CRITICAL** | 9.0 – 10.0 | Single-Level Store (`engine.db`) corruption, dirty cache-line flush omission (`CLWB`), memory safety violation, SMT refinement type bypass, or unhandled panic in production mode. | **Blocker**: All production releases halted immediately until patched and verified. |
| **P2** | **HIGH** | 7.0 – 8.9 | Unbounded memory growth (> 100 KiB per 1M requests), lock-free atomic queue deadlock, connection starvation under high concurrency (> 5,000 workers). | **High Priority**: Must be resolved before minor version release. |
| **P3** | **MEDIUM** | 4.0 – 6.9 | Latency tail collapse (P99.9 > 100x P50), non-deterministic SMT proof delay (> 2.0s per function), or non-optimal memory page alignment. | **Standard Cycle**: Scheduled for next regular sprint. |
| **P4** | **LOW** | 0.1 – 3.9 | Minor telemetry variance, non-optimal assembly instruction emission without functional impact, or CLI display formatting glitch. | **Backlog**: Resolved during code hygiene passes. |

---

## 3. Advisory Lifecycle

```
[DISCOVERED IN BENCHMARK]
           │
           ▼
[ENG-ANOM FILED] ──> Status: OPEN (Reproduction script verified in container)
           │
           ▼
[ROOT CAUSE ANALYSIS] ──> Hardware / Memory / SMT failure mechanism identified
           │
           ▼
[PATCH IMPLEMENTED] ──> Commit SHA recorded, Z3 formal invariant added
           │
           ▼
[CONTAINER VERIFIED] ──> Re-tested on `engine-genesis-node` under identical load
           │
           ▼
[STATUS: RESOLVED] ──> Advisory updated, published, and added to regression test suite
```

---

## 4. Submitting an Anomaly Report

1. Copy `TEMPLATE_ENG_ANOM.md` to `ENG-ANOM-<YEAR>-<NNNN>.md`.
2. Ensure reproduction commands run completely inside the standard Docker container:
   ```bash
   docker exec engine-genesis-node sh -lc '...'
   ```
3. Link the report to the benchmark run located in `benchmarks/reports/`.

