#!/usr/bin/env python3
"""Hash-bound gate for delta manifests."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
import subprocess
from typing import Any


HEX40 = re.compile(r"^[0-9a-f]{40}$")
ROOT = pathlib.Path(__file__).resolve().parents[1]


def canonical_hash(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def issue(kind: str, detail: str) -> dict[str, str]:
    return {"kind": kind, "detail": detail}


def read_json(path: pathlib.Path | None) -> dict[str, Any] | None:
    if not path:
        return None
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def receipt_valid(report: dict[str, Any] | None) -> bool:
    if not report:
        return False
    field = "receipt_hash" if isinstance(report.get("receipt_hash"), str) else "report_hash"
    digest = report.get(field)
    stable = dict(report)
    stable.pop(field, None)
    return isinstance(digest, str) and digest == canonical_hash(stable)


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def field(text: str, name: str) -> str | None:
    match = re.search(rf"^{re.escape(name)}:\s*(\S+)\s*$", text, re.MULTILINE)
    return match.group(1) if match else None


def section_lines(text: str, headings: set[str]) -> list[str]:
    lines: list[str] = []
    active = False
    for raw in text.splitlines():
        stripped = raw.strip()
        if stripped in headings:
            active = True
            continue
        if active and stripped.endswith(":") and not stripped.startswith("-"):
            break
        if active and stripped.startswith("-"):
            lines.append(stripped[1:].strip())
    return lines


def validation_lines(text: str) -> list[str]:
    return section_lines(text, {"validation:", "validation_results:"})


def changed_file_lines(text: str) -> list[str]:
    return section_lines(text, {"changed_files:", "changed files:"})


def first_line_contract(text: str, base_commit: str | None, head_commit: str | None) -> bool:
    lines = text.splitlines()
    return (
        len(lines) >= 2
        and lines[0].strip() == f"base_commit: {base_commit}"
        and lines[1].strip() == f"head_commit: {head_commit}"
    )


def git_changed_files(base_commit: str | None, head_commit: str | None) -> list[str]:
    if not (isinstance(base_commit, str) and HEX40.match(base_commit)):
        return []
    if not (isinstance(head_commit, str) and HEX40.match(head_commit)):
        return []
    result = subprocess.run(
        ["git", "diff", "--name-only", f"{base_commit}..{head_commit}"],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode != 0:
        return []
    return sorted(line for line in result.stdout.splitlines() if line)


def has_token(lines: list[str], token: str) -> bool:
    return any(token in line for line in lines)


def manifest_state(path: pathlib.Path) -> dict[str, Any]:
    text = path.read_text(encoding="utf-8")
    validations = validation_lines(text)
    changed_files = sorted(changed_file_lines(text))
    base_commit = field(text, "base_commit")
    head_commit = field(text, "head_commit")
    expected_changed_files = git_changed_files(base_commit, head_commit)
    return {
        "path": str(path),
        "sha256": sha256(path),
        "base_commit": base_commit,
        "head_commit": head_commit,
        "first_lines_present": first_line_contract(text, base_commit, head_commit),
        "validation_commands": len(validations),
        "bundle_verify_present": has_token(validations, "git bundle verify"),
        "duration_receipts_present": bool(validations) and all("elapsed_ms=" in line for line in validations),
        "test_count_present": "test_count=" in text or "native_test_count=" in text,
        "native_skip_explicit": "native_tools_absent" in text,
        "changed_files": changed_files,
        "expected_changed_files": expected_changed_files,
        "changed_files_match": bool(expected_changed_files) and changed_files == expected_changed_files,
        "receiver_apply_commands_present": all(
            token in text for token in ["git fetch ./repo-delta-", "git merge --ff-only FETCH_HEAD"]
        ),
    }


def receipt_state(report: dict[str, Any] | None) -> dict[str, Any]:
    return {
        "report_present": report is not None,
        "receipt_hash_valid": receipt_valid(report),
        "status": (report or {}).get("status"),
        "failures": len((report or {}).get("failures") or []),
        "missing_signals": len((report or {}).get("missing_signals") or []),
    }


def native_state(report: dict[str, Any] | None) -> dict[str, Any]:
    state = receipt_state(report)
    state["test_count"] = (report or {}).get("test_count")
    state["duration_ms"] = (report or {}).get("duration_ms")
    return state


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    manifest = manifest_state(args.manifest)
    runtime = receipt_state(read_json(args.runtime_receipt_report))
    native = native_state(read_json(args.native_witness_report))
    missing: list[dict[str, str]] = []
    failures: list[dict[str, str]] = []

    for name in ["base_commit", "head_commit"]:
        value = manifest.get(name)
        if not isinstance(value, str) or not HEX40.match(value):
            failures.append(issue(f"{name}_missing", f"manifest lacks valid {name}"))
    if not manifest["first_lines_present"]:
        failures.append(issue("manifest_header_invalid", "manifest must start with base_commit then head_commit"))
    if manifest["validation_commands"] == 0:
        failures.append(issue("validation_commands_absent", "manifest has no validation command receipts"))
    if not manifest["bundle_verify_present"]:
        failures.append(issue("bundle_verify_absent", "manifest lacks git bundle verify evidence"))
    if not manifest["duration_receipts_present"]:
        failures.append(issue("validation_duration_absent", "every validation receipt needs elapsed_ms"))
    if not manifest["test_count_present"]:
        failures.append(issue("test_count_absent", "manifest lacks explicit test_count or native_test_count"))
    if not manifest["changed_files_match"]:
        failures.append(issue("changed_files_mismatch", "manifest changed_files must match git diff base..head"))
    if not manifest["receiver_apply_commands_present"]:
        failures.append(issue("receiver_apply_commands_absent", "manifest lacks receiver fetch and ff-only merge commands"))

    if not runtime["report_present"]:
        missing.append(issue("runtime_receipt_absent", "runtime receipt report not supplied"))
    elif not runtime["receipt_hash_valid"]:
        failures.append(issue("runtime_receipt_hash_invalid", "runtime receipt report is unsigned or hash-invalid"))
    elif runtime["status"] == "fail":
        missing.append(issue("runtime_receipt_rejected", "runtime report is failure evidence, not correctness proof"))

    if not native["report_present"]:
        if manifest["native_skip_explicit"]:
            missing.append(issue("native_tools_absent", "native Rust validation is explicitly skipped"))
        else:
            failures.append(issue("native_witness_absent", "native witness missing without native_tools_absent skip"))
    elif not native["receipt_hash_valid"]:
        failures.append(issue("native_witness_hash_invalid", "native witness is unsigned or hash-invalid"))

    report: dict[str, Any] = {
        "schema_version": 1,
        "mode": "delta_contract_gate",
        "status": "fail" if failures else ("pass_with_skip" if missing else "pass"),
        "manifest": manifest,
        "runtime_receipt": runtime,
        "native_witness": native,
        "missing_signals": missing,
        "failures": failures,
    }
    report["receipt_hash"] = canonical_hash(report)
    return report


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=pathlib.Path, required=True)
    parser.add_argument("--runtime-receipt-report", type=pathlib.Path)
    parser.add_argument("--native-witness-report", type=pathlib.Path)
    parser.add_argument("--report", type=pathlib.Path, default=pathlib.Path("validation/delta_contract_report.eval.json"))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = build_report(args)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"delta contract gate: {report['status']} missing={len(report['missing_signals'])} failures={len(report['failures'])}")
    return 1 if report["status"] == "fail" else 0


if __name__ == "__main__":
    raise SystemExit(main())