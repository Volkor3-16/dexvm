#!/usr/bin/env python3
"""Aggregate all test reports into a comprehensive summary."""

import json
import os
from pathlib import Path
from collections import defaultdict

REPORT_DIR = Path("/home/volkor/git/dexvm")

# Find all report files
report_files = list(REPORT_DIR.glob("report_*.json"))
print(f"Found {len(report_files)} report files")

# Aggregate data
total_extensions = 0
total_sources = 0
sources_by_status = defaultdict(int)
operations_by_status = defaultdict(int)
failed_sources = []
extensions_with_load_errors = 0
extensions_with_zero_sources = 0

for report_file in report_files:
    try:
        with open(report_file) as f:
            report = json.load(f)
    except Exception as e:
        print(f"Error reading {report_file}: {e}")
        continue
    
    for ext in report.get("extensions", []):
        total_extensions += 1
        
        if ext.get("load_error"):
            extensions_with_load_errors += 1
            continue
            
        sources = ext.get("sources", [])
        if not sources:
            extensions_with_zero_sources += 1
            
        for src in sources:
            total_sources += 1
            has_failure = src.get("error") or any(
                not op.get("success") and op.get("error_type") != "skipped" 
                for op in src.get("operations", {}).values()
            )
            
            if has_failure:
                sources_by_status["failed"] += 1
                # Find the first failing operation
                for op_name, op in src.get("operations", {}).items():
                    if not op.get("success") and op.get("error_type") != "skipped":
                        failed_sources.append({
                            "extension": ext.get("apk_name", "unknown"),
                            "source": src.get("name", "unknown"),
                            "operation": op_name,
                            "error_type": op.get("error_type", "unknown"),
                            "error_message": op.get("error", "unknown")[:200]
                        })
                        break
            else:
                sources_by_status["passed"] += 1
            
            for op_name, op in src.get("operations", {}).items():
                status = "passed" if op.get("success") else ("skipped" if op.get("error_type") == "skipped" else "failed")
                operations_by_status[f"{op_name}:{status}"] += 1

# Print summary
print("\n" + "="*60)
print("AGGREGATE TEST SUMMARY")
print("="*60)
print(f"Extensions tested: {total_extensions}")
print(f"  With load errors: {extensions_with_load_errors}")
print(f"  With zero sources: {extensions_with_zero_sources}")
print(f"  Successfully loaded: {total_extensions - extensions_with_load_errors - extensions_with_zero_sources}")
print(f"Total sources: {total_sources}")
print(f"\nSources by status:")
for status, count in sorted(sources_by_status.items()):
    print(f"  {status}: {count}")

print(f"\nOperations by status:")
for op_status, count in sorted(operations_by_status.items()):
    print(f"  {op_status}: {count}")

print(f"\nFailed sources: {len(failed_sources)}")
if failed_sources:
    print("\nTop failure types:")
    error_types = defaultdict(int)
    for fs in failed_sources:
        error_types[fs["error_type"]] += 1
    for etype, count in sorted(error_types.items(), key=lambda x: -x[1])[:20]:
        print(f"  {etype}: {count}")

    print("\nSample failures:")
    for fs in failed_sources[:20]:
        print(f"  [{fs['error_type']}] {fs['extension']} / {fs['source']} / {fs['operation']}: {fs['error_message'][:100]}")

# Save aggregated report
aggregated = {
    "total_extensions": total_extensions,
    "extensions_with_load_errors": extensions_with_load_errors,
    "extensions_with_zero_sources": extensions_with_zero_sources,
    "total_sources": total_sources,
    "sources_by_status": dict(sources_by_status),
    "operations_by_status": dict(operations_by_status),
    "failed_sources": failed_sources
}

with open(REPORT_DIR / "aggregated_report.json", "w") as f:
    json.dump(aggregated, f, indent=2)

print(f"\nAggregated report saved to {REPORT_DIR / 'aggregated_report.json'}")