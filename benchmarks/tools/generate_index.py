#!/usr/bin/env python3
"""
Engine Lang Benchmark Index & Catalog Generator
Specification: ENG-SPEC-03.1 / ENG-BENCH-01.0
Author: Lysander Vance & The Vance Protocol Group / Engine Lang Core Working Group

Scans benchmarks/reports/ and benchmarks/advisories/ to automatically generate
the authoritative benchmarks/INDEX.md catalog.
"""

import os
import json
import glob
from pathlib import Path
from datetime import datetime, timezone

BENCHMARKS_DIR = Path(__file__).resolve().parent.parent
REPORTS_DIR = BENCHMARKS_DIR / "reports"
ADVISORIES_DIR = BENCHMARKS_DIR / "advisories"
INDEX_FILE = BENCHMARKS_DIR / "INDEX.md"

def load_json(path):
    try:
        with open(path, "r", encoding="utf-8") as f:
            return json.load(f)
    except Exception:
        return None

def scan_advisories():
    advisories = []
    for md_file in sorted(ADVISORIES_DIR.glob("ENG-ANOM-*.md")):
        advisory_id = md_file.stem
        title = "Unknown Anomaly"
        severity = "Unknown"
        status = "OPEN"
        with open(md_file, "r", encoding="utf-8") as f:
            for line in f:
                if line.startswith("# ENG-ANOM-"):
                    title = line.replace("# ", "").strip()
                elif "- **Severity**:" in line:
                    severity = line.split(":", 1)[1].strip()
                elif "- **Status**:" in line:
                    status = line.split(":", 1)[1].strip()
        advisories.append({
            "id": advisory_id,
            "title": title,
            "severity": severity,
            "status": status,
            "file": md_file.relative_to(BENCHMARKS_DIR).as_posix()
        })
    return advisories

def scan_reports():
    reports = []
    categories = ["runtimes", "databases", "frameworks", "stress"]
    
    for category in categories:
        cat_dir = REPORTS_DIR / category
        if not cat_dir.exists():
            continue
        for run_dir in sorted(cat_dir.iterdir(), reverse=True):
            if not run_dir.is_dir():
                continue
            
            env_file = run_dir / "env.json"
            bench_file = run_dir / "benchmark.json"
            report_file = run_dir / "report.md"
            
            env_data = load_json(env_file) or {}
            bench_data = load_json(bench_file) or []
            
            timestamp = env_data.get("timestamp", run_dir.name[:17])
            title = env_data.get("title", run_dir.name)
            verification = env_data.get("verification", {}).get("status", "UNVERIFIED")
            
            targets = []
            if "targets" in env_data:
                for t_key, t_val in env_data["targets"].items():
                    targets.append(t_val.get("name", t_key))
            target_str = " vs. ".join(targets) if targets else "Standard Suite"
            
            reports.append({
                "category": category,
                "dir_name": run_dir.name,
                "timestamp": timestamp,
                "title": title,
                "targets": target_str,
                "verification": verification,
                "report_link": f"reports/{category}/{run_dir.name}/report.md",
                "json_link": f"reports/{category}/{run_dir.name}/benchmark.json"
            })
    return reports

def generate_index_markdown(reports, advisories):
    lines = []
    lines.append("# Engine Lang Benchmark Catalog & Anomaly Registry")
    lines.append("")
    lines.append("**Specification Reference**: `ENG-SPEC-03.1` / `ENG-BENCH-01.0`  ")
    lines.append("**Authority**: Lysander Vance & The Vance Protocol Group / Engine Lang Core Working Group  ")
    lines.append(f"**Last Synchronized**: {datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M:%S UTC')}  ")
    lines.append("**License**: Apache License 2.0  ")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 1. Master Benchmark Execution Index")
    lines.append("")
    lines.append("All benchmarks recorded across all categories, environments, and stress levels.")
    lines.append("")
    lines.append("| Date & Time (UTC) | Category | Benchmark Title | Confrontation Targets | Verification | Artifacts |")
    lines.append("|---|---|---|---|:---:|---|")
    
    if not reports:
        lines.append("| — | — | *No runs recorded yet* | — | — | — |")
    else:
        for r in reports:
            badge = "🟢 `CONTAINER`" if "CONTAINER" in r["verification"] else "🟡 `UNVERIFIED`"
            lines.append(f"| {r['timestamp']} | **`{r['category']}`** | {r['title']} | {r['targets']} | {badge} | [Report]({r['report_link']}) / [JSON]({r['json_link']}) |")
            
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 2. Mission-Critical Defect & Anomaly Log (`ENG-ANOM`)")
    lines.append("")
    lines.append("System defects, tail-latency regressions, memory growth incidents, and invariant breaches discovered during benchmarking.")
    lines.append("")
    lines.append("| Advisory ID | Severity | Status | Title | Document |")
    lines.append("|---|---|:---:|---|---|")
    
    if not advisories:
        lines.append("| — | — | — | *Zero open anomalies recorded. System invariants hold.* | [Advisory Policy](advisories/README.md) |")
    else:
        for a in advisories:
            lines.append(f"| `{a['id']}` | **{a['severity']}** | `{a['status']}` | {a['title']} | [View Report]({a['file']}) |")
            
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("## 3. Methodology & Governance")
    lines.append("")
    lines.append("1. **Adversarial Confrontation**: We benchmark against the fastest production systems (C, C++, Rust, Go, Python, PostgreSQL, MySQL, SQLite, MongoDB, Supabase).")
    lines.append("2. **Reproducibility Mandate**: Every run can be reproduced via `benchmarks/tools/docker-compose.bench.yml`.")
    lines.append("3. **Mission-Critical Integrity**: We publish failures as aggressively as victories. Every defect discovered is assigned an `ENG-ANOM` identifier and patched with formal verification.")
    lines.append("")
    return "\n".join(lines) + "\n"

def main():
    advisories = scan_advisories()
    reports = scan_reports()
    content = generate_index_markdown(reports, advisories)
    with open(INDEX_FILE, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"Successfully generated {INDEX_FILE} with {len(reports)} reports and {len(advisories)} advisories.")

if __name__ == "__main__":
    main()
