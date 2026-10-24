#!/usr/bin/env python3
"""
PostgreSQL 18 Confrontation Benchmark
Storage Engine: Relational MVCC / WAL (Write-Ahead Logging)
Architecture: ENG-SPEC-03.1 / ENG-SLS-01.1
Author: Lysander Vance & The Vance Protocol Group / Engine Lang Core Working Group
License: Apache 2.0
"""

import os
import sys
import time
import json
try:
    import psycopg2
    HAS_PSYCOPG2 = True
except ImportError:
    HAS_PSYCOPG2 = False

def run_benchmark(host="localhost", port=5432, user="postgres", password="password", dbname="postgres", operations=1000):
    if not HAS_PSYCOPG2:
        return {
            "error": "psycopg2 is not installed in the host/container environment. Run pip install psycopg2-binary."
        }

    try:
        conn = psycopg2.connect(host=host, port=port, user=user, password=password, dbname=dbname, connect_timeout=5)
    except Exception as e:
        return {"error": f"Failed to connect to PostgreSQL at {host}:{port}: {str(e)}"}

    cursor = conn.cursor()
    cursor.execute("DROP TABLE IF EXISTS benchmark_ledger;")
    cursor.execute("""
        CREATE TABLE benchmark_ledger (
            id INT PRIMARY KEY,
            balance BIGINT NOT NULL,
            tx_count INT NOT NULL,
            state_hash CHAR(64) NOT NULL
        );
    """)
    conn.commit()

    dummy_hash = "a" * 64

    # 1. Insertion phase
    t_start = time.perf_counter()
    for i in range(1, operations + 1):
        cursor.execute("INSERT INTO benchmark_ledger (id, balance, tx_count, state_hash) VALUES (%s, %s, %s, %s);",
                       (i, 1000, 0, dummy_hash))
    conn.commit()
    t_insert = time.perf_counter()

    # 2. Transactional Mutation phase (discrete transactions with fsync)
    t_mutate_start = time.perf_counter()
    for i in range(1, operations + 1):
        cursor.execute("UPDATE benchmark_ledger SET balance = balance + 50, tx_count = tx_count + 1 WHERE id = %s;", (i,))
        conn.commit()
    t_mutate_end = time.perf_counter()

    # 3. Read lookup phase
    t_read_start = time.perf_counter()
    for i in range(1, operations + 1):
        cursor.execute("SELECT balance FROM benchmark_ledger WHERE id = %s;", (i,))
        cursor.fetchone()
    t_read_end = time.perf_counter()

    cursor.execute("SELECT pg_total_relation_size('benchmark_ledger');")
    table_size = cursor.fetchone()[0]

    cursor.execute("DROP TABLE benchmark_ledger;")
    conn.commit()
    cursor.close()
    conn.close()

    insert_ms = (t_insert - t_start) * 1000.0
    mutate_ms = (t_mutate_end - t_mutate_start) * 1000.0
    read_ms = (t_read_end - t_read_start) * 1000.0

    return {
        "engine": "postgresql_18",
        "operations": operations,
        "insert_time_ms": round(insert_ms, 3),
        "mutate_time_ms": round(mutate_ms, 3),
        "read_time_ms": round(read_ms, 3),
        "mutate_tps": round(operations / (mutate_ms / 1000.0), 1) if mutate_ms > 0 else 0,
        "read_qps": round(operations / (read_ms / 1000.0), 1) if read_ms > 0 else 0,
        "table_size_bytes": table_size
    }

if __name__ == "__main__":
    ops = int(os.environ.get("OPERATIONS", "1000"))
    h = os.environ.get("PG_HOST", "localhost")
    p = int(os.environ.get("PG_PORT", "5432"))
    u = os.environ.get("PG_USER", "postgres")
    pw = os.environ.get("PG_PASSWORD", "password")
    db = os.environ.get("PG_DB", "postgres")
    res = run_benchmark(host=h, port=p, user=u, password=pw, dbname=db, operations=ops)
    print(json.dumps(res, indent=2))

