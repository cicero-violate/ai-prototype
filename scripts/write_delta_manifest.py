#!/usr/bin/env python3
"""Write a delta receipt and manifest from one observe-validation report."""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


def git(args: list[str]) -> str:
    return subprocess.check_output(["git", *args], text=True).strip()


def sha256(path: Path) -> str | None:
    return hashlib.sha256(path.read_bytes()).hexdigest() if path.exists() else None


def load_report(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))
    return rows


def latest(rows: list[dict[str, Any]], event: str) -> dict[str, Any]:
    for row in reversed(rows):
        if row.get("event") == event:
            return row
    return {}


def command_rows(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [row.get("result", {}) for row in rows if row.get("event") == "validation_command"]


def changed_files(base: str, head: str) -> list[str]:
    out = git(["diff", "--name-only", f"{base}..{head}"])
    return [line for line in out.splitlines() if line]


def build_receipt(args: argparse.Namespace) -> dict[str, Any]:
    rows = load_report(Path(args.report))
    summary = latest(rows, "validation_summary")
    commands = command_rows(rows)
    base, head = args.base, args.head
    bundle_name = Path(args.bundle).name if args.bundle else "repo-delta.bundle"
    return {
        "schema_version": 1,
        "base_commit": base,
        "head_commit": head,
        "changed_files": changed_files(base, head),
        "validation_status": summary.get("validation_status", "unknown"),
        "validation_commands": summary.get("validation_commands") or [
            {"name": c.get("name"), "cmd": c.get("cmd"), "status": c.get("status")} for c in commands
        ],
        "validation_command_count": summary.get("validation_command_count", len(commands)),
        "validation_test_count": summary.get("validation_test_count", 0),
        "router_test_count": summary.get("router_test_count", 0),
        "cargo_test_count_when_available": summary.get("cargo_test_count_when_available"),
        "missing_signal_flags": summary.get("missing_signal_flags", {}),
        "missing_signal_count": summary.get("missing_signal_count"),
        "failed_required_commands": summary.get("failed_required_commands", []),
        "report_path": str(Path(args.report)),
        "report_sha256": sha256(Path(args.report)),
        "bundle_path": str(Path(args.bundle)) if args.bundle else None,
        "bundle_sha256": sha256(Path(args.bundle)) if args.bundle else None,
        "bundle_verify": args.bundle_verify,
        "receiver_apply_commands": [
            f"git fetch ./{bundle_name} HEAD",
            "git merge --ff-only FETCH_HEAD",
        ],
    }


def write_manifest(path: Path, receipt: dict[str, Any]) -> None:
    lines = [
        f"base_commit: {receipt['base_commit']}",
        f"head_commit: {receipt['head_commit']}",
        "",
        "# Delta Manifest",
        "",
        "## Changed Files",
        *[f"- {name}" for name in receipt["changed_files"]],
        "",
        "## Validation Results",
        f"- validation_status: {receipt['validation_status']}",
        f"- validation_command_count: {receipt['validation_command_count']}",
        f"- validation_test_count: {receipt['validation_test_count']}",
        f"- router_test_count: {receipt.get('router_test_count', 0)}",
        f"- cargo_test_count_when_available: {receipt.get('cargo_test_count_when_available')}",
        f"- failed_required_commands: {json.dumps(receipt.get('failed_required_commands', []), sort_keys=True)}",
        f"- missing_signal_count: {receipt.get('missing_signal_count')}",
        f"- report_path: {receipt['report_path']}",
        f"- report_sha256: {receipt['report_sha256']}",
        f"- bundle_sha256: {receipt['bundle_sha256']}",
        f"- bundle_verify: {receipt['bundle_verify']}",
        "",
        "## Validation Commands",
    ]
    for command in receipt["validation_commands"]:
        lines.append(f"- {command.get('name')}: {command.get('status')} :: {' '.join(map(str, command.get('cmd') or []))}")
    lines += [
        "",
        "## Missing Signal Flags",
    ]
    for key, value in sorted(receipt.get("missing_signal_flags", {}).items()):
        lines.append(f"- {key}: {value}")
    lines += [
        "",
        "## Receiver Apply Commands",
        "```bash",
        " && ".join(receipt["receiver_apply_commands"]),
        "```",
        "",
    ]
    path.write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", required=True)
    parser.add_argument("--report", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--receipt-out", required=True)
    parser.add_argument("--bundle", default="")
    parser.add_argument("--bundle-verify", default="not_run")
    args = parser.parse_args()

    receipt = build_receipt(args)
    Path(args.receipt_out).parent.mkdir(parents=True, exist_ok=True)
    Path(args.receipt_out).write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    write_manifest(Path(args.out), receipt)


if __name__ == "__main__":
    main()