#!/usr/bin/env python3
"""
SQLite 3.46+ Confrontation Benchmark
Storage Engine: In-Process B-Tree (WAL Mode)
Architecture: ENG-SPEC-03.1 / ENG-SLS-01.1
Author: Lysander Vance & The Vance Protocol Group / Engine Lang Core Working Group
License: Apache 2.0
"""

import sqlite3
import time
import os
import json

def run_benchmark(db_path="/tmp/benchmark_sqlite.db", operations=5000):
    if os.path.exists(db_path):
        os.remove(db_path)

    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    # Enable WAL mode and synchronous NORMAL
    cursor.execute("PRAGMA journal_mode = WAL;")
    cursor.execute("PRAGMA synchronous = NORMAL;")
    cursor.execute("""
        CREATE TABLE ledger (
            id INTEGER PRIMARY KEY,
            balance INTEGER NOT NULL,
            tx_count INTEGER NOT NULL,
            state_hash TEXT NOT NULL
        );
    """)
    conn.commit()

    dummy_hash = "a" * 64

    # 1. Insertion phase
    t_start = time.perf_counter()
    cursor.execute("BEGIN TRANSACTION;")
    for i in range(1, operations + 1):
        cursor.execute("INSERT INTO ledger (id, balance, tx_count, state_hash) VALUES (?, ?, ?, ?)",
                       (i, 1000, 0, dummy_hash))
    conn.commit()
    t_insert = time.perf_counter()

    # 2. Transactional Mutation phase (discrete transactions)
    t_mutate_start = time.perf_counter()
    for i in range(1, operations + 1):
        cursor.execute("BEGIN TRANSACTION;")
        cursor.execute("UPDATE ledger SET balance = balance + 50, tx_count = tx_count + 1 WHERE id = ?", (i,))
        conn.commit()
    t_mutate_end = time.perf_counter()

    # 3. Read lookup phase
    t_read_start = time.perf_counter()
    for i in range(1, operations + 1):
        cursor.execute("SELECT balance FROM ledger WHERE id = ?", (i,))
        cursor.fetchone()
    t_read_end = time.perf_counter()

    file_size_bytes = os.path.getsize(db_path)
    wal_size_bytes = os.path.getsize(db_path + "-wal") if os.path.exists(db_path + "-wal") else 0

    conn.close()
    if os.path.exists(db_path):
        os.remove(db_path)
    if os.path.exists(db_path + "-wal"):
        os.remove(db_path + "-wal")

    insert_ms = (t_insert - t_start) * 1000.0
    mutate_ms = (t_mutate_end - t_mutate_start) * 1000.0
    read_ms = (t_read_end - t_read_start) * 1000.0

    return {
        "engine": "sqlite_wal",
        "operations": operations,
        "insert_time_ms": round(insert_ms, 3),
        "mutate_time_ms": round(mutate_ms, 3),
        "read_time_ms": round(read_ms, 3),
        "mutate_tps": round(operations / (mutate_ms / 1000.0), 1) if mutate_ms > 0 else 0,
        "read_qps": round(operations / (read_ms / 1000.0), 1) if read_ms > 0 else 0,
        "storage_size_bytes": file_size_bytes + wal_size_bytes
    }

if __name__ == "__main__":
    ops = int(os.environ.get("OPERATIONS", "5000"))
    result = run_benchmark(operations=ops)
    print(json.dumps(result, indent=2))

