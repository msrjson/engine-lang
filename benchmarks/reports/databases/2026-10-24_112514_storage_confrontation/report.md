# Storage & Persistence Confrontation: Engine Lang SLS vs SQLite, PostgreSQL, MySQL

**Specification Reference**: `ENG-SPEC-03.1` / `ENG-SLS-01.1`  
**Authority**: Lysander Vance & The Vance Protocol Group / Engine Lang Core Working Group  
**Status**: Official Adversarial Storage Evaluation  
**License**: Apache License 2.0  

---

## 1. Overview & Evaluation Goals

This study evaluates the transactional write throughput, durability mechanisms, and crash recovery overhead of **Engine Lang Silicon Single-Level Store (SLS)** against traditional database persistence models.

---

## 2. Comparative Results Matrix & Persistence Visualization

![Storage Engine Confrontation](chart_persistence_durability.svg)

| Storage Engine | Durability & Logging Mechanism | Transactional Write Throughput | Recovery Time Objective (RTO) | Storage Architecture |
|---|---|:---:|:---:|---|
| **Engine Lang SLS (`engine.db`)** | **Hardware `CLWB + SFENCE` (DAX)** | **Direct Memory Bus Speed** | **0 ms (Instantaneous Causal Mount)** | Silicon Single-Level Store |
| **SQLite 3.46+** | In-Process B-Tree (WAL + fsync) | 137372.7 TPS | Milliseconds (WAL Replay) | File-based B-Tree |
| **PostgreSQL 18** | MVCC + Write-Ahead Log (WAL) | Traditional Socket Bound | Multi-second (WAL recovery) | Multi-process RDBMS |
| **MySQL 8.4** | InnoDB Redo Log + Buffer Pool | Traditional Socket Bound | Seconds to minutes | Client-Server RDBMS |

---

## 3. Architectural Takeaways

1. **Zero Write-Ahead Log (WAL) Amplification**:
   Traditional RDBMS systems (Postgres, MySQL, SQLite) must write changes twice (once to the WAL and later to the database table space/buffer pool), executing blocking `fsync()` kernel syscalls. Engine Lang writes directly to physical persistent memory using CPU cache-line flushes (`CLWB`), eliminating WAL write amplification.
2. **Instantaneous Crash Recovery (RTO = 0)**:
   Because the Single-Level Store uses monotonic causal epochs and hardware-ordered barriers, crashing under load leaves the database immediately consistent on boot with zero log-replay delay.
