#!/usr/bin/env python3
"""
MySQL 8.4+ / MariaDB Confrontation Benchmark
Storage Engine: InnoDB Buffer Pool & Redo Log
Architecture: ENG-SPEC-03.1 / ENG-SLS-01.1
Author: Lysander Vance & The Vance Protocol Group / Engine Lang Core Working Group
License: Apache 2.0
"""

import os
import sys
import time
import json
try:
    import pymysql
    HAS_PYMYSQL = True
except ImportError:
    HAS_PYMYSQL = False

def run_benchmark(host="localhost", port=3306, user="root", password="password", db="test", operations=1000):
    if not HAS_PYMYSQL:
        return {
            "error": "pymysql is not installed in the environment. Run pip install pymysql."
        }

    try:
        conn = pymysql.connect(host=host, port=port, user=user, password=password, database=db, connect_timeout=5)
    except Exception as e:
        return {"error": f"Failed to connect to MySQL at {host}:{port}: {str(e)}"}

    cursor = conn.cursor()
    cursor.execute("DROP TABLE IF EXISTS benchmark_ledger;")
    cursor.execute("""
        CREATE TABLE benchmark_ledger (
            id INT PRIMARY KEY,
            balance BIGINT NOT NULL,
            tx_count INT NOT NULL,
            state_hash CHAR(64) NOT NULL
        ) ENGINE=InnoDB;
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

    # 2. Transactional Mutation phase (discrete transactions)
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

    cursor.execute("DROP TABLE benchmark_ledger;")
    conn.commit()
    cursor.close()
    conn.close()

    insert_ms = (t_insert - t_start) * 1000.0
    mutate_ms = (t_mutate_end - t_mutate_start) * 1000.0
    read_ms = (t_read_end - t_read_start) * 1000.0

    return {
        "engine": "mysql_innodb",
        "operations": operations,
        "insert_time_ms": round(insert_ms, 3),
        "mutate_time_ms": round(mutate_ms, 3),
        "read_time_ms": round(read_ms, 3),
        "mutate_tps": round(operations / (mutate_ms / 1000.0), 1) if mutate_ms > 0 else 0,
        "read_qps": round(operations / (read_ms / 1000.0), 1) if read_ms > 0 else 0
    }

if __name__ == "__main__":
    ops = int(os.environ.get("OPERATIONS", "1000"))
    h = os.environ.get("MYSQL_HOST", "localhost")
    p = int(os.environ.get("MYSQL_PORT", "3306"))
    u = os.environ.get("MYSQL_USER", "root")
    pw = os.environ.get("MYSQL_PASSWORD", "password")
    d = os.environ.get("MYSQL_DB", "test")
    res = run_benchmark(host=h, port=p, user=u, password=pw, db=d, operations=ops)
    print(json.dumps(res, indent=2))

