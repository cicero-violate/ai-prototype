#!/usr/bin/env python3
"""Hash-bound gate for ChatGPT runtime receipt evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import tarfile
from typing import Any, Iterable


def canonical_hash(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def issue(kind: str, detail: str) -> dict[str, str]:
    return {"kind": kind, "detail": detail}


def read_json_bytes(name: str, data: bytes) -> Iterable[tuple[str, Any]]:
    if name.endswith(".ndjson"):
        for line_no, line in enumerate(data.decode("utf-8").splitlines(), 1):
            if line.strip():
                yield f"{name}:{line_no}", json.loads(line)
    elif name.endswith(".json"):
        yield name, json.loads(data.decode("utf-8"))


def archive_records(path: pathlib.Path) -> Iterable[tuple[str, Any]]:
    with tarfile.open(path, "r:gz") as archive:
        for member in archive.getmembers():
            if not member.isfile():
                continue
            if not (member.name.endswith(".json") or member.name.endswith(".ndjson")):
                yield member.name, None
                continue
            handle = archive.extractfile(member)
            if handle is not None:
                yield from read_json_bytes(member.name, handle.read())


def dir_records(root: pathlib.Path) -> Iterable[tuple[str, Any]]:
    for path in sorted(root.rglob("*")):
        if path.is_file() and not (path.name.endswith(".json") or path.name.endswith(".ndjson")):
            yield str(path.relative_to(root)), None
    for path in sorted(root.rglob("*.json")) + sorted(root.rglob("*.ndjson")):
        yield from read_json_bytes(str(path.relative_to(root)), path.read_bytes())


def records(args: argparse.Namespace) -> Iterable[tuple[str, Any]]:
    if args.runtime_archive:
        return archive_records(args.runtime_archive)
    return dir_records(args.runtime_dir)


def normalized_record_path(name: str) -> str:
    """Return a slash-normalized archive/log path without NDJSON line suffixes."""
    path = name.split(":", 1)[0].replace("\\", "/")
    return path.lstrip("./")


def path_has_component(path: str, component: str) -> bool:
    return f"/{component}/" in f"/{path}/"


def count_path(counts: dict[str, int], name: str) -> None:
    path = normalized_record_path(name)
    if path_has_component(path, "delta-apply-receipts"):
        counts["delta_apply_receipts"] += 1
    elif path_has_component(path, "loop-stop-receipts"):
        counts["loop_stop_receipts"] += 1
    elif path.endswith(".downloads.ndjson"):
        counts["download_ledger_rows"] += 1
    elif path.endswith(".candidate-ledger.ndjson"):
        counts["candidate_ledger_rows"] += 1
    elif path.endswith(".messages.ndjson"):
        counts["message_ledger_rows"] += 1
    elif "network-requests-turn-" in path:
        counts["network_log_rows"] += 1
    if pathlib.PurePosixPath(path).name == "DELTA_MANIFEST.md":
        counts["delta_manifests"] += 1


def validation_claim(receipt: dict[str, Any]) -> tuple[bool, dict[str, Any]]:
    validation = receipt.get("validation")
    if not isinstance(validation, dict):
        return False, {}
    passed = validation.get("passed") is True or validation.get("status") == "pass" or validation.get("classification") == "validation_success"
    return passed, validation


def empty_validation(validation: dict[str, Any]) -> bool:
    commands = validation.get("commands")
    command = validation.get("command")
    no_commands = isinstance(commands, list) and not commands and (not command)
    no_tests = validation.get("testCount") == 0 or validation.get("unitTests") == "not_applicable"
    return no_commands or no_tests


def digest_file(path: pathlib.Path | None) -> str | None:
    if not path:
        return None
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    failures: list[dict[str, str]] = []
    missing: list[dict[str, str]] = []
    observations: list[dict[str, Any]] = []
    counts = {
        "delta_apply_receipts": 0,
        "loop_stop_receipts": 0,
        "download_ledger_rows": 0,
        "candidate_ledger_rows": 0,
        "message_ledger_rows": 0,
        "network_log_rows": 0,
        "delta_manifests": 0,
        "stale_advisories": 0,
    }

    for name, obj in records(args):
        count_path(counts, name)
        if isinstance(obj, dict) and obj.get("fileName") == "DELTA_MANIFEST.md":
            counts["delta_manifests"] += 1
        if isinstance(obj, dict) and obj.get("classification") == "stale_advisory":
            counts["stale_advisories"] += 1
        if name.endswith("RUNTIME_MANIFEST.json") and isinstance(obj, dict):
            for event in obj.get("downloadHistory") or []:
                if not isinstance(event, dict):
                    continue
                if event.get("fileName") == "DELTA_MANIFEST.md":
                    counts["delta_manifests"] += 1
                if event.get("classification") == "stale_advisory":
                    counts["stale_advisories"] += 1

        if not isinstance(obj, dict):
            continue

        path = normalized_record_path(name)

        if path_has_component(path, "delta-apply-receipts"):
            passed, validation = validation_claim(obj)
            if passed and empty_validation(validation):
                failures.append(issue("empty_validation_false_pass", f"{name} claims validation pass without commands/tests"))
            if obj.get("decision") == "accepted" and not obj.get("receiptSignature"):
                missing.append(issue("unsigned_delta_receipt", f"{name} has no receiptSignature"))
            observations.append(
                {
                    "path": name,
                    "decision": obj.get("decision"),
                    "after_head": obj.get("afterHead"),
                    "validation_status": validation.get("status"),
                    "validation_commands": len(validation.get("commands") or []),
                    "test_count": validation.get("testCount"),
                }
            )
        elif path_has_component(path, "loop-stop-receipts"):
            if obj.get("failureClass") == "turn_timeout":
                failures.append(issue("turn_timeout", f"{name} records a turn timeout"))
            network = obj.get("networkFailures") if isinstance(obj.get("networkFailures"), dict) else {}
            blocking = int(network.get("blocking_conversation_api") or 0) + int(network.get("blocking_download") or 0)
            if blocking:
                failures.append(issue("blocking_network_failure", f"{name} records {blocking} blocking network failure(s)"))
        elif name.endswith("current-run-summary.json"):
            gate = ((obj.get("gate") or {}).get("observed") or {}) if isinstance(obj.get("gate"), dict) else {}
            if gate.get("validationPassed") is True and gate.get("receiptSigned") is False:
                missing.append(issue("unsigned_runtime_gate", "current runtime gate passed with unsigned receipt evidence"))
            delta = obj.get("deltaApply") if isinstance(obj.get("deltaApply"), dict) else {}
            passed, validation = validation_claim(delta)
            if passed and empty_validation(validation):
                failures.append(issue("empty_current_validation_false_pass", "current delta apply summary claims validation pass without commands/tests"))

    if counts["delta_apply_receipts"] == 0:
        missing.append(issue("delta_apply_receipts_absent", "no delta apply receipts were found"))
    if counts["delta_manifests"] == 0:
        missing.append(issue("delta_manifest_absent", "no downloaded delta manifest evidence was found"))
    if counts["stale_advisories"]:
        missing.append(issue("stale_advisory_present", f"{counts['stale_advisories']} stale advisory record(s) found"))

    report: dict[str, Any] = {
        "schema_version": 1,
        "mode": "runtime_receipt_gate",
        "status": "fail" if failures else ("pass_with_skip" if missing else "pass"),
        "archive_sha256": digest_file(args.runtime_archive),
        "evidence_counts": counts,
        "missing_signals": missing,
        "failures": failures,
        "observations": observations[:24],
    }
    report["receipt_hash"] = canonical_hash(report)
    return report


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--runtime-archive", type=pathlib.Path)
    source.add_argument("--runtime-dir", type=pathlib.Path)
    parser.add_argument("--report", type=pathlib.Path, default=pathlib.Path("validation/runtime_receipt_report.eval.json"))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = build_report(args)
    args.report.parent.mkdir(parents=True, exist_ok=True)
    args.report.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"runtime receipt gate: {report['status']} missing={len(report['missing_signals'])} failures={len(report['failures'])}")
    return 1 if report["status"] == "fail" else 0


if __name__ == "__main__":
    raise SystemExit(main())