#!/usr/bin/env python3
"""
MongoDB 7+ / 8+ Confrontation Benchmark
Storage Engine: WiredTiger (Document Store / BSON)
Architecture: ENG-SPEC-03.1 / ENG-SLS-01.1
Author: Lysander Vance & The Vance Protocol Group / Engine Lang Core Working Group
License: Apache 2.0
"""

import os
import sys
import time
import json
try:
    from pymongo import MongoClient
    HAS_PYMONGO = True
except ImportError:
    HAS_PYMONGO = False

def run_benchmark(uri="mongodb://localhost:27017", db_name="bench_db", operations=1000):
    if not HAS_PYMONGO:
        return {
            "error": "pymongo is not installed in the environment. Run pip install pymongo."
        }

    try:
        client = MongoClient(uri, serverSelectionTimeoutMS=5000)
        client.admin.command('ping')
    except Exception as e:
        return {"error": f"Failed to connect to MongoDB at {uri}: {str(e)}"}

    db = client[db_name]
    collection = db["benchmark_ledger"]
    collection.drop()

    dummy_hash = "a" * 64

    # 1. Insertion phase
    docs = [{"_id": i, "balance": 1000, "tx_count": 0, "state_hash": dummy_hash} for i in range(1, operations + 1)]
    t_start = time.perf_counter()
    collection.insert_many(docs)
    t_insert = time.perf_counter()

    # 2. Transactional Mutation phase (discrete updates with journal write concern)
    t_mutate_start = time.perf_counter()
    for i in range(1, operations + 1):
        collection.update_one({"_id": i}, {"$inc": {"balance": 50, "tx_count": 1}})
    t_mutate_end = time.perf_counter()

    # 3. Read lookup phase
    t_read_start = time.perf_counter()
    for i in range(1, operations + 1):
        collection.find_one({"_id": i})
    t_read_end = time.perf_counter()

    collection.drop()
    client.close()

    insert_ms = (t_insert - t_start) * 1000.0
    mutate_ms = (t_mutate_end - t_mutate_start) * 1000.0
    read_ms = (t_read_end - t_read_start) * 1000.0

    return {
        "engine": "mongodb_wiredtiger",
        "operations": operations,
        "insert_time_ms": round(insert_ms, 3),
        "mutate_time_ms": round(mutate_ms, 3),
        "read_time_ms": round(read_ms, 3),
        "mutate_tps": round(operations / (mutate_ms / 1000.0), 1) if mutate_ms > 0 else 0,
        "read_qps": round(operations / (read_ms / 1000.0), 1) if read_ms > 0 else 0
    }

if __name__ == "__main__":
    ops = int(os.environ.get("OPERATIONS", "1000"))
    u = os.environ.get("MONGO_URI", "mongodb://localhost:27017")
    res = run_benchmark(uri=u, operations=ops)
    print(json.dumps(res, indent=2))

