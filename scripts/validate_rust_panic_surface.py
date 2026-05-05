#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


CALLS = {
    "unwrap": re.compile(r"\.unwrap\s*\("),
    "expect": re.compile(r"\.expect\s*\("),
    "panic": re.compile(r"\bpanic!\s*\("),
}


@dataclass(frozen=True)
class Finding:
    path: str
    line: int
    bucket: str
    kind: str
    text: str


def git_files(root: Path) -> list[Path]:
    try:
        out = subprocess.check_output(["git", "ls-files"], cwd=root, text=True)
    except Exception:
        return sorted(root.rglob("*.rs"))
    return [root / line for line in out.splitlines() if line.endswith(".rs")]


def brace_delta(line: str) -> int:
    return line.count("{") - line.count("}")


def classify_src_lines(lines: list[str]) -> list[str]:
    buckets = ["production"] * len(lines)
    cfg_test_pending = False
    test_fn_pending = False
    cfg_depth: int | None = None
    fn_depth: int | None = None
    depth = 0

    for i, line in enumerate(lines):
        stripped = line.strip()
        if re.match(r"#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]", stripped):
            cfg_test_pending = True
        if re.match(r"#\s*\[\s*test\s*\]", stripped):
            test_fn_pending = True

        is_test = cfg_depth is not None or fn_depth is not None
        buckets[i] = "test" if is_test else "production"

        if cfg_test_pending and "{" in line:
            cfg_depth = depth + line[: line.index("{") + 1].count("{")
            buckets[i] = "test"
            cfg_test_pending = False
        if test_fn_pending and "{" in line:
            fn_depth = depth + line[: line.index("{") + 1].count("{")
            buckets[i] = "test"
            test_fn_pending = False

        depth += brace_delta(line)
        if cfg_depth is not None and depth < cfg_depth:
            cfg_depth = None
        if fn_depth is not None and depth < fn_depth:
            fn_depth = None

    return buckets


def scan_file(root: Path, path: Path) -> list[Finding]:
    rel = path.relative_to(root).as_posix()
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
    if "/validation/fixtures/" in f"/{rel}":
        buckets = ["test"] * len(lines)
    elif rel.startswith("examples/"):
        buckets = ["example"] * len(lines)
    else:
        buckets = classify_src_lines(lines)

    findings: list[Finding] = []
    for i, line in enumerate(lines, start=1):
        for kind, pattern in CALLS.items():
            if pattern.search(line):
                findings.append(Finding(rel, i, buckets[i - 1], kind, line.strip()))
    return findings


def summarize(findings: list[Finding]) -> dict[str, object]:
    buckets = {bucket: {kind: 0 for kind in CALLS} for bucket in ("production", "test", "example")}
    for item in findings:
        buckets[item.bucket][item.kind] += 1
    return {
        "schema_version": 1,
        "buckets": buckets,
        "production_total": sum(buckets["production"].values()),
        "test_total": sum(buckets["test"].values()),
        "example_total": sum(buckets["example"].values()),
        "finding_count": len(findings),
        "findings": [item.__dict__ for item in findings],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    parser.add_argument("--report", required=True)
    parser.add_argument("--fail-production-unwrap", action="store_true")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    files = [p for p in git_files(root) if p.exists() and ("/src/" in f"/{p.relative_to(root).as_posix()}" or p.relative_to(root).as_posix().startswith("examples/"))]
    findings = [finding for path in files for finding in scan_file(root, path)]
    report = summarize(findings)
    report_path = Path(args.report)
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print(json.dumps({k: report[k] for k in ("production_total", "test_total", "example_total", "finding_count")}, sort_keys=True))
    if args.fail_production_unwrap and report["production_total"]:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())