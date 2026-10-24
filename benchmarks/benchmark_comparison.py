#!/usr/bin/env python3
"""
Engine Lang vs. Laravel Production Benchmark Suite
Architecture: ENG-SPEC-03.1 / ENG-SLS-01.1
The Vance Protocol Group / Engine Lang Core Working Group

Comparative evaluation of Silicon Persistence (Single-Level Store)
against Traditional Multi-Tiered Web Architecture (PHP 8.3 + Laravel 13 + SQLite).
"""

import sys
import time
import json
import statistics
import concurrent.futures
import http.client
from urllib.parse import urlparse

TOTAL_REQUESTS = 1000
CONCURRENCY = 25
WARMUP_REQUESTS = 50

def run_benchmark_target(name, url, method="GET", body=None, headers=None, total_requests=TOTAL_REQUESTS, concurrency=CONCURRENCY):
    parsed = urlparse(url)
    host = parsed.hostname
    port = parsed.port or (443 if parsed.scheme == "https" else 80)
    path = parsed.path or "/"
    if parsed.query:
        path += "?" + parsed.query

    req_headers = {
        "User-Agent": "EngineBench/1.0",
        "Accept": "*/*",
        "Connection": "keep-alive"
    }
    if headers:
        req_headers.update(headers)

    latencies_ms = []
    status_codes = {}
    bytes_transferred = 0

    def worker(count):
        nonlocal bytes_transferred
        local_lats = []
        local_codes = {}
        local_bytes = 0

        conn = http.client.HTTPConnection(host, port, timeout=10)
        for _ in range(count):
            t0 = time.perf_counter()
            try:
                conn.request(method, path, body=body, headers=req_headers)
                resp = conn.getresponse()
                resp_data = resp.read()
                t1 = time.perf_counter()

                local_lats.append((t1 - t0) * 1000.0)
                code = resp.status
                local_codes[code] = local_codes.get(code, 0) + 1
                local_bytes += len(resp_data)
            except Exception as e:
                t1 = time.perf_counter()
                local_lats.append((t1 - t0) * 1000.0)
                local_codes["error"] = local_codes.get("error", 0) + 1
                try:
                    conn.close()
                    conn = http.client.HTTPConnection(host, port, timeout=10)
                except Exception:
                    pass
        conn.close()
        return local_lats, local_codes, local_bytes

    # Warmup
    warmup_per_worker = max(1, WARMUP_REQUESTS // concurrency)
    with concurrent.futures.ThreadPoolExecutor(max_workers=min(concurrency, 5)) as ex:
        list(ex.map(worker, [warmup_per_worker] * min(concurrency, 5)))

    # Main Benchmark
    reqs_per_worker = total_requests // concurrency
    remainder = total_requests % concurrency
    tasks = [reqs_per_worker + (1 if i < remainder else 0) for i in range(concurrency)]

    start_time = time.perf_counter()
    with concurrent.futures.ThreadPoolExecutor(max_workers=concurrency) as executor:
        futures = [executor.submit(worker, c) for c in tasks]
        for f in concurrent.futures.as_completed(futures):
            lats, codes, b = f.result()
            latencies_ms.extend(lats)
            bytes_transferred += b
            for k, v in codes.items():
                status_codes[k] = status_codes.get(k, 0) + v
    total_time = time.perf_counter() - start_time

    rps = len(latencies_ms) / total_time if total_time > 0 else 0
    latencies_sorted = sorted(latencies_ms)
    count = len(latencies_sorted)

    p50 = latencies_sorted[int(count * 0.50)] if count > 0 else 0
    p90 = latencies_sorted[int(count * 0.90)] if count > 0 else 0
    p95 = latencies_sorted[int(count * 0.95)] if count > 0 else 0
    p99 = latencies_sorted[int(count * 0.99)] if count > 0 else 0
    mean_lat = statistics.mean(latencies_ms) if count > 0 else 0
    min_lat = latencies_sorted[0] if count > 0 else 0
    max_lat = latencies_sorted[-1] if count > 0 else 0

    mb_transferred = bytes_transferred / (1024 * 1024)
    throughput_mb = mb_transferred / total_time if total_time > 0 else 0

    return {
        "target": name,
        "method": method,
        "url": url,
        "rps": rps,
        "total_time_s": total_time,
        "mean_ms": mean_lat,
        "min_ms": min_lat,
        "p50_ms": p50,
        "p90_ms": p90,
        "p95_ms": p95,
        "p99_ms": p99,
        "max_ms": max_lat,
        "throughput_mb_s": throughput_mb,
        "status_codes": status_codes,
        "total_requests": len(latencies_ms)
    }

def main():
    print("=========================================================================================")
    print("⚡ ENGINE LANG vs. LARAVEL 13 PRODUCTION BENCHMARK (REAL-TIME)")
    print("   Engine Forge: http://127.0.0.1:8091 (Single-Level Store & Silicon In-Memory)")
    print("   Laravel 13:   http://127.0.0.1:8000 (PHP 8.3 OPcache JIT + SQLite Database)")
    print(f"   Parameters:   {TOTAL_REQUESTS} requests per scenario | {CONCURRENCY} parallel workers")
    print("=========================================================================================")

    scenarios = [
        {
            "id": "ssr_html",
            "name": "Scenario 1: Server-Rendered HTML (SSR Welcome)",
            "method": "GET",
            "engine_url": "http://127.0.0.1:8091/",
            "laravel_url": "http://127.0.0.1:8000/",
            "headers": {"Accept": "text/html"},
            "body": None,
        },
        {
            "id": "read_categories",
            "name": "Scenario 2: Simple Read API (11 Categories JSON)",
            "method": "GET",
            "engine_url": "http://127.0.0.1:8091/api/v1/categories",
            "laravel_url": "http://127.0.0.1:8000/api/v1/categories",
            "headers": {"Accept": "application/json"},
            "body": None,
        },
        {
            "id": "read_posts",
            "name": "Scenario 3: Complex Graph Read API (11 Rich Posts JSON)",
            "method": "GET",
            "engine_url": "http://127.0.0.1:8091/api/v1/posts",
            "laravel_url": "http://127.0.0.1:8000/api/v1/posts",
            "headers": {"Accept": "application/json"},
            "body": None,
        },
        {
            "id": "auth_login",
            "name": "Scenario 4: Stateless Auth Session Token (POST JSON)",
            "method": "POST",
            "engine_url": "http://127.0.0.1:8091/api/v1/auth/login",
            "laravel_url": "http://127.0.0.1:8000/api/v1/auth/login",
            "headers": {"Content-Type": "application/json", "Accept": "application/json"},
            "body": json.dumps({"email": "architect@engine-lang.org"}).encode("utf-8"),
        },
        {
            "id": "post_mutation",
            "name": "Scenario 5: Transactional Mutation / Persistence (POST)",
            "method": "POST",
            "engine_url": "http://127.0.0.1:8091/api/v1/posts",
            "laravel_url": "http://127.0.0.1:8000/api/v1/posts",
            "headers": {"Content-Type": "application/x-www-form-urlencoded"},
            "body": b"title=Benchmark+Post&slug=bench-item&category=Operadores+%26+SIMD&content=Silicon+Durability",
        },
    ]

    benchmark_data = []

    for sc in scenarios:
        print(f"\n▶ Executing: {sc['name']}")
        print(f"  • Benchmarking Engine Lang...")
        res_engine = run_benchmark_target(
            f"Engine Lang - {sc['name']}",
            sc["engine_url"],
            method=sc["method"],
            body=sc["body"],
            headers=sc["headers"]
        )

        print(f"  • Benchmarking Laravel 13...")
        res_laravel = run_benchmark_target(
            f"Laravel 13 - {sc['name']}",
            sc["laravel_url"],
            method=sc["method"],
            body=sc["body"],
            headers=sc["headers"]
        )

        speedup = res_engine["rps"] / res_laravel["rps"] if res_laravel["rps"] > 0 else 0
        lat_reduction = (1.0 - (res_engine["p50_ms"] / res_laravel["p50_ms"])) * 100 if res_laravel["p50_ms"] > 0 else 0

        benchmark_data.append({
            "scenario": sc,
            "engine": res_engine,
            "laravel": res_laravel,
            "speedup": speedup,
            "lat_reduction_pct": lat_reduction
        })

        print(f"    ✔ Engine Lang: {res_engine['rps']:>8.1f} req/s | P50: {res_engine['p50_ms']:>6.2f} ms | P99: {res_engine['p99_ms']:>6.2f} ms")
        print(f"    ✔ Laravel 13:  {res_laravel['rps']:>8.1f} req/s | P50: {res_laravel['p50_ms']:>6.2f} ms | P99: {res_laravel['p99_ms']:>6.2f} ms")
        print(f"    ⚡ Speedup:     {speedup:>8.2f}x faster | Latency Reduction: {lat_reduction:>5.1f}%")

    # Output detailed comparative table
    print("\n" + "=" * 105)
    print("📊 COMPREHENSIVE PRODUCTION BENCHMARK MATRIX")
    print("=" * 105)
    print(f"{'Scenario / Request Type':<35} | {'Stack':<12} | {'Req/s':>10} | {'P50 (ms)':>9} | {'P95 (ms)':>9} | {'P99 (ms)':>9} | {'Factor':>8}")
    print("-" * 105)

    for item in benchmark_data:
        sc_name = item["scenario"]["name"].split(":")[1].strip()
        eng = item["engine"]
        lar = item["laravel"]
        print(f"{sc_name:<35} | {'Engine Lang':<12} | {eng['rps']:>10.1f} | {eng['p50_ms']:>9.2f} | {eng['p95_ms']:>9.2f} | {eng['p99_ms']:>9.2f} | {item['speedup']:>7.2f}x")
        print(f"{'':<35} | {'Laravel 13':<12} | {lar['rps']:>10.1f} | {lar['p50_ms']:>9.2f} | {lar['p95_ms']:>9.2f} | {lar['p99_ms']:>9.2f} | {'1.00x':>8}")
        print("-" * 105)

    # Save output to JSON
    cleaned_data = []
    for item in benchmark_data:
        sc_copy = dict(item["scenario"])
        if isinstance(sc_copy.get("body"), bytes):
            sc_copy["body"] = sc_copy["body"].decode("utf-8", errors="replace")
        cleaned_data.append({
            "scenario": sc_copy,
            "engine": item["engine"],
            "laravel": item["laravel"],
            "speedup": item["speedup"],
            "lat_reduction_pct": item["lat_reduction_pct"]
        })

    with open("/workspace/benchmarks/benchmark_results.json", "w") as f:
        json.dump(cleaned_data, f, indent=2)
    print("\n💾 Benchmark data saved to benchmarks/benchmark_results.json")

if __name__ == "__main__":
    main()

