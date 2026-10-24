#!/usr/bin/env python3
"""
Engine Forge Performance & Latency Benchmark Suite
Architecture: ENG-SPEC-03.1 / ENG-SLS-01.1
The Vance Protocol Group / Engine Lang Core Working Group
"""

import sys
import time
import urllib.request
import urllib.error
import http.client
import json
import statistics
import concurrent.futures
from urllib.parse import urlparse

def run_http_benchmark(name, url, method="GET", body=None, total_requests=1000, concurrency=25):
    print(f"\n==================================================================")
    print(f"🔥 Benchmark: {name}")
    print(f"   Target:      {method} {url}")
    print(f"   Requests:    {total_requests} requests")
    print(f"   Concurrency: {concurrency} parallel workers")
    print(f"==================================================================")

    parsed = urlparse(url)
    host = parsed.hostname
    port = parsed.port or (443 if parsed.scheme == "https" else 80)
    path = parsed.path or "/"
    if parsed.query:
        path += "?" + parsed.query

    latencies_ms = []
    status_codes = {}
    bytes_transferred = 0

    headers = {
        "User-Agent": "EngineForge-Bench/1.0",
        "Accept": "*/*",
        "Connection": "keep-alive"
    }
    if body and method == "POST":
        headers["Content-Type"] = "application/json"

    def worker(req_count):
        nonlocal bytes_transferred
        local_latencies = []
        local_codes = {}
        local_bytes = 0

        conn = http.client.HTTPConnection(host, port, timeout=10)
        for _ in range(req_count):
            t0 = time.perf_counter()
            try:
                conn.request(method, path, body=body, headers=headers)
                resp = conn.getresponse()
                resp_data = resp.read()
                t1 = time.perf_counter()
                
                local_latencies.append((t1 - t0) * 1000.0)
                code = resp.status
                local_codes[code] = local_codes.get(code, 0) + 1
                local_bytes += len(resp_data)
            except Exception as e:
                t1 = time.perf_counter()
                local_latencies.append((t1 - t0) * 1000.0)
                local_codes["error"] = local_codes.get("error", 0) + 1
                try:
                    conn.close()
                    conn = http.client.HTTPConnection(host, port, timeout=10)
                except Exception:
                    pass
        conn.close()
        return local_latencies, local_codes, local_bytes

    # Distribute requests among workers
    reqs_per_worker = total_requests // concurrency
    remainder = total_requests % concurrency
    tasks = [reqs_per_worker + (1 if i < remainder else 0) for i in range(concurrency)]

    start_time = time.perf_counter()
    with concurrent.futures.ThreadPoolExecutor(max_workers=concurrency) as executor:
        futures = [executor.submit(worker, count) for count in tasks]
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

    print(f"📊 Results:")
    print(f"   Total Time:         {total_time:.3f} s")
    print(f"   Throughput (RPS):   {rps:.2f} req/sec")
    print(f"   Transfer Rate:      {throughput_mb:.2f} MB/sec ({mb_transferred:.2f} MB total)")
    print(f"   Latency Mean:       {mean_lat:.2f} ms")
    print(f"   Latency Min:        {min_lat:.2f} ms")
    print(f"   Latency P50 (Med):  {p50:.2f} ms")
    print(f"   Latency P90:        {p90:.2f} ms")
    print(f"   Latency P95:        {p95:.2f} ms")
    print(f"   Latency P99:        {p99:.2f} ms")
    print(f"   Latency Max:        {max_lat:.2f} ms")
    print(f"   Status Codes:       {status_codes}")
    return {
        "name": name,
        "rps": rps,
        "mean_ms": mean_lat,
        "p50_ms": p50,
        "p95_ms": p95,
        "p99_ms": p99,
        "codes": status_codes
    }

def main():
    print("==================================================================")
    print("🚀 Engine Forge Full-Stack Benchmark Suite")
    print("   Testing Host Port: http://localhost:8091")
    print("   Comparing Silicon Single-Level Store against Traditional Stacks")
    print("==================================================================")

    # Health check
    try:
        urllib.request.urlopen("http://localhost:8091/", timeout=2)
    except Exception as e:
        print(f"❌ Error: Cannot connect to http://localhost:8091. Is the container running? ({e})")
        sys.exit(1)

    results = []

    # Benchmark 1: Clean Welcome Dashboard (HTML)
    r1 = run_http_benchmark(
        name="Engine Forge Welcome Screen (HTML)",
        url="http://localhost:8091/",
        method="GET",
        total_requests=1000,
        concurrency=25
    )
    results.append(r1)

    # Benchmark 2: Persistent Categories (JSON)
    r2 = run_http_benchmark(
        name="Single-Level Store Categories API (JSON)",
        url="http://localhost:8091/api/v1/categories",
        method="GET",
        total_requests=1000,
        concurrency=25
    )
    results.append(r2)

    # Benchmark 3: Persistent Articles Graph (JSON)
    r3 = run_http_benchmark(
        name="Single-Level Store Articles Graph API (JSON)",
        url="http://localhost:8091/api/v1/posts",
        method="GET",
        total_requests=1000,
        concurrency=25
    )
    results.append(r3)

    # Benchmark 4: Authentication Gate (POST JSON)
    r4 = run_http_benchmark(
        name="Sanctum Session Authentication Gate (POST)",
        url="http://localhost:8091/api/v1/auth/login",
        method="POST",
        body=json.dumps({"email": "architect@engine-lang.org"}).encode("utf-8"),
        total_requests=1000,
        concurrency=25
    )
    results.append(r4)

    # Summary Table
    print("\n==================================================================")
    print("📋 BENCHMARK EXECUTIVE SUMMARY")
    print("==================================================================")
    print(f"{'Benchmark Target':<42} | {'Req/s':>10} | {'P50 (ms)':>9} | {'P95 (ms)':>9} | {'P99 (ms)':>9}")
    print("-" * 88)
    for res in results:
        print(f"{res['name']:<42} | {res['rps']:>10.1f} | {res['p50_ms']:>9.2f} | {res['p95_ms']:>9.2f} | {res['p99_ms']:>9.2f}")
    print("==================================================================")

if __name__ == "__main__":
    main()

