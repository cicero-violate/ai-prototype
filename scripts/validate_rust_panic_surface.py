#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

PANIC_PATTERNS = {
    "unwrap": re.compile(r"\.unwrap\s*\("),
    "expect": re.compile(r"\.expect\s*\("),
    "panic": re.compile(r"\bpanic!\s*\("),
}

TEST_MARKERS = (
    "/tests/",
    "tests/",
    "/examples/",
    "examples/",
    "canon-rustc-v3/validation/fixtures/",
    "/validation/fixtures/",
)


def classify(path: Path, root: Path) -> str:
    rel = str(path.relative_to(root))
    if any(marker in rel for marker in TEST_MARKERS) or rel.startswith("tests/"):
        return "test"
    return "production"


def scan(root: Path) -> dict[str, object]:
    counts = {
        "production_unwrap_count": 0,
        "production_expect_count": 0,
        "production_panic_count": 0,
        "test_unwrap_count": 0,
        "test_expect_count": 0,
        "test_panic_count": 0,
        "example_total": 0,
        "files_scanned": 0,
    }
    for path in sorted(root.rglob("*.rs")):
        rel = str(path.relative_to(root))
        if "/target/" in f"/{rel}" or rel.startswith("target/"):
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        counts["files_scanned"] += 1
        bucket = classify(path, root)
        if rel.startswith("examples/"):
            counts["example_total"] += 1
        counts[f"{bucket}_unwrap_count"] += len(PANIC_PATTERNS["unwrap"].findall(text))
        counts[f"{bucket}_expect_count"] += len(PANIC_PATTERNS["expect"].findall(text))
        counts[f"{bucket}_panic_count"] += len(PANIC_PATTERNS["panic"].findall(text))
    counts["production_total"] = (
        counts["production_unwrap_count"]
        + counts["production_expect_count"]
        + counts["production_panic_count"]
    )
    counts["test_total"] = counts["test_unwrap_count"] + counts["test_expect_count"] + counts["test_panic_count"]
    return counts


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    parser.add_argument("--report", required=True)
    parser.add_argument("--fail-production-unwrap", action="store_true")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    report = Path(args.report)
    data = scan(root)
    data["status"] = "pass"
    if args.fail_production_unwrap and data["production_unwrap_count"]:
        data["status"] = "fail"
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0 if data["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
