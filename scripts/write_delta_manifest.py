#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


def fail(message: str) -> None:
    raise SystemExit(message)


def run_git(args: list[str]) -> subprocess.CompletedProcess[str]:
    done = subprocess.run(
        ["git", *args],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if done.returncode != 0:
        fail(f"git {' '.join(args)} failed: {done.stderr.strip()}")
    return done


def run_bundle(args: list[str]) -> subprocess.CompletedProcess[str]:
    done = subprocess.run(
        ["git", "bundle", *args],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if done.returncode != 0:
        fail(f"git bundle {' '.join(args)} failed: {done.stderr.strip()}")
    return done


def file_sha256(path: str | Path | None) -> str | None:
    if not path:
        return None
    candidate = Path(path)
    if not candidate.exists():
        return None
    return hashlib.sha256(candidate.read_bytes()).hexdigest()


def report_rows(path: str | Path) -> list[dict[str, Any]]:
    report = Path(path)
    if not report.exists():
        fail(f"validation report does not exist: {report}")
    rows = []
    for line in report.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))
    return rows


def last_event(rows: list[dict[str, Any]], event: str) -> dict[str, Any]:
    return next((row for row in reversed(rows) if row.get("event") == event), {})


def command_rows(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [row for row in rows if row.get("event") == "validation_command"]


def command_fingerprint(command: dict[str, Any]) -> dict[str, Any]:
    return {
        "name": command.get("name"),
        "cmd": command.get("cmd"),
        "status": command.get("status"),
        "exit_code": command.get("exit_code"),
        "duration_ms": command.get("duration_ms"),
        "timed_out": command.get("timed_out"),
        "connector_failure_class": command.get("connector_failure_class"),
    }


def reject_conflicting_duplicate_command_rows(commands: list[dict[str, Any]]) -> None:
    by_name: dict[str, dict[str, Any]] = {}
    for command in commands:
        name = command.get("name")
        if not name:
            continue
        fingerprint = command_fingerprint(command)
        previous = by_name.get(name)
        if previous is not None and previous != fingerprint:
            fail(f"conflicting validation_command rows for command {name}")
        by_name[name] = fingerprint


def validate_commands(summary: dict[str, Any], rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    summary_commands = list(summary.get("validation_commands") or [])
    row_commands = command_rows(rows)
    reject_conflicting_duplicate_command_rows(row_commands)
    if summary_commands and row_commands:
        summary_fingerprints = [command_fingerprint(command) for command in summary_commands]
        row_fingerprints = [command_fingerprint(command) for command in row_commands]
        if summary_fingerprints != row_fingerprints:
            fail("conflicting validation command evidence between summary and command rows")
    commands = summary_commands or row_commands
    if not commands:
        fail("no validation commands")
    for index, command in enumerate(commands):
        if not command.get("name"):
            fail(f"validation command {index} has no name")
        if "status" not in command:
            fail(f"validation command {command.get('name')} has no status")
        if "cmd" not in command:
            fail(f"validation command {command.get('name')} has no cmd field")
    return commands


def positive_int(value: Any, name: str) -> int:
    try:
        result = int(value)
    except (TypeError, ValueError):
        fail(f"{name} is not an integer: {value}")
    if result < 0:
        fail(f"{name} is negative: {value}")
    return result


def validation_closure(
    summary: dict[str, Any], rows: list[dict[str, Any]], head: str
) -> tuple[list[dict[str, Any]], str | None]:
    report_head = summary.get("git_head")
    if not report_head:
        fail("validation report has no git_head")
    if report_head != head:
        fail(f"stale validation report: report head {report_head} != manifest head {head}")
    commands = validate_commands(summary, rows)
    command_count = positive_int(summary.get("validation_command_count"), "validation_command_count")
    if command_count == 0:
        fail("validation_command_count must be greater than zero")
    if command_count != len(commands):
        fail(f"validation_command_count {command_count} != command rows {len(commands)}")
    test_count = positive_int(summary.get("validation_test_count"), "validation_test_count")
    reason = summary.get("zero_test_reason")
    if test_count == 0 and not reason:
        fail("validation_test_count is zero without zero_test_reason")
    return commands, reason


def changed_files(base: str, head: str) -> list[str]:
    run_git(["cat-file", "-e", f"{base}^{{commit}}"])
    run_git(["cat-file", "-e", f"{head}^{{commit}}"])
    ancestor = subprocess.run(
        ["git", "merge-base", "--is-ancestor", base, head],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if ancestor.returncode != 0:
        fail(f"base is not an ancestor of head: {base}..{head}")
    files = run_git(["diff", "--name-only", base, head]).stdout.splitlines()
    if not files:
        fail("empty delta")
    return files


def bundle_required_refs(verify_output: str) -> list[str]:
    refs: list[str] = []
    capture = False
    for line in verify_output.splitlines():
        if line.startswith("The bundle requires this ref:"):
            capture = True
            continue
        if line.startswith("The bundle "):
            capture = False
        if capture:
            tokens = line.split()
            if tokens:
                refs.append(tokens[0])
    return refs


def verify_bundle(path: str, base: str, head: str) -> tuple[str, list[str], list[str]]:
    bundle = Path(path)
    if not bundle.exists():
        fail(f"bundle does not exist: {bundle}")
    verify = run_bundle(["verify", str(bundle)])
    heads = run_bundle(["list-heads", str(bundle)]).stdout.splitlines()
    if not any(head in line for line in heads):
        fail(f"bundle does not expose expected head: {head}")
    required = bundle_required_refs(verify.stdout + verify.stderr)
    if base not in required:
        fail(f"bundle does not require base commit {base}")
    return "pass", heads, required


PRESERVED_SUMMARY_KEYS = [
    "command_execution_status",
    "missing_signal_status",
    "runtime_archive_evidence_source",
    "full_summary_report_only",
    "full_summary_report_command",
    "runtime_manifest_base_expected",
    "runtime_manifest_base_commit",
    "runtime_manifest_base_matches_delta_base",
    "runtime_archive_report_present",
    "runtime_archive_report_status",
    "runtime_archive_report_base_matches_current",
    "runtime_archive_present",
    "runtime_archive_inspection_status",
    "runtime_archive_download_index_files",
    "runtime_archive_prior_state_files",
    "runtime_archive_conversation_ledger_files",
    "runtime_archive_delta_receipt_files",
    "runtime_archive_audit_files",
    "runtime_archive_current_run_summary_present",
    "runtime_archive_runtime_manifest_present",
    "policy_learning_trace_validation_result",
    "policy_learning_trace_status",
    "policy_learning_trace_function",
    "policy_learning_trace_check_count",
    "policy_learning_trace_missing_count",
    "panic_surface_production_unwrap_count",
    "panic_surface_production_expect_count",
    "panic_surface_production_panic_count",
    "panic_surface_test_total",
    "panic_surface_example_total",
    "router_test_count",
    "external_observation_stream_test_present",
    "external_observation_stream_evidence_files",
    "external_observation_stream_evidence_tokens",
    "external_api_action_test_present",
    "external_api_action_evidence_files",
    "external_api_action_evidence_tokens",
    "semantic_artifact_verification_test_present",
    "semantic_artifact_verification_evidence_files",
    "semantic_artifact_verification_evidence_tokens",
    "connector_failure_classification_present",
    "connector_failure_present",
    "connector_failure_status",
    "connector_failure_classes",
    "connector_transport_instability_present",
    "connector_transport_artifact_classification",
    "connector_transport_artifact_classification_options",
    "connector_transport_artifact_classification_reason",
    "connector_transport_status",
    "connector_transport_interrupted",
    "connector_transport_report_path",
    "connector_transport_report_present",
    "connector_transport_report_complete",
    "connector_transport_exit_file",
    "connector_transport_exit_file_present",
    "ignored_artifact_count",
    "ignored_target_artifact_count",
    "ignored_runtime_artifact_count",
    "ignored_validation_artifact_count",
]


def receipt(args: argparse.Namespace) -> dict[str, Any]:
    rows = report_rows(args.report)
    summary = last_event(rows, "validation_summary")
    if not summary:
        fail("validation report has no validation_summary event")
    commands, zero_reason = validation_closure(summary, rows, args.head)
    files = changed_files(args.base, args.head)
    bundle_verify, bundle_heads, bundle_required = verify_bundle(args.bundle, args.base, args.head)

    data: dict[str, Any] = {
        "schema_version": 1,
        "base_commit": args.base,
        "head_commit": args.head,
        "changed_files": files,
        "changed_file_count": len(files),
        "validation_status": summary.get("validation_status", "unknown"),
        "validation_commands": commands,
        "validation_command_count": summary.get("validation_command_count"),
        "validation_test_count": summary.get("validation_test_count"),
        "zero_test_reason": zero_reason,
        "failed_required_commands": summary.get("failed_required_commands", []),
        "missing_signal_flags": summary.get("missing_signal_flags", {}),
        "missing_signal_count": summary.get("missing_signal_count", 0),
        "report_path": str(args.report),
        "report_sha256": file_sha256(args.report),
        "bundle_path": str(args.bundle),
        "bundle_sha256": file_sha256(args.bundle),
        "bundle_verify": bundle_verify,
        "bundle_heads": bundle_heads,
        "bundle_required_refs": bundle_required,
        "bundle_requires_base_commit": args.base in bundle_required,
        "receiver_apply_command": f"git fetch ./{Path(args.bundle).name} HEAD",
        "validation_report_git_head": summary.get("git_head"),
    }
    for key in PRESERVED_SUMMARY_KEYS:
        if key in summary:
            data[key] = summary[key]
    return data


def write_manifest(path: str | Path, receipt_data: dict[str, Any]) -> None:
    lines = [
        f"base_commit: {receipt_data['base_commit']}",
        f"head_commit: {receipt_data['head_commit']}",
        "",
        "# Delta Manifest",
        "",
        "## Changed Files",
    ]
    lines.extend(f"- {name}" for name in receipt_data["changed_files"])
    lines.extend(["", "## Validation Results"])
    metric_keys = [
        "validation_status",
        "validation_command_count",
        "validation_test_count",
        "zero_test_reason",
        "failed_required_commands",
        "missing_signal_count",
        "report_sha256",
        "bundle_sha256",
        "bundle_verify",
        "bundle_heads",
        "bundle_required_refs",
        "bundle_requires_base_commit",
        "validation_report_git_head",
        "changed_file_count",
        *PRESERVED_SUMMARY_KEYS,
    ]
    seen = set()
    for key in metric_keys:
        if key in seen or key not in receipt_data or receipt_data[key] is None:
            continue
        seen.add(key)
        value = receipt_data[key]
        if isinstance(value, (dict, list)):
            value = json.dumps(value, sort_keys=True)
        lines.append(f"- {key}: {value}")
    lines.extend(["", "## Validation Commands"])
    for command in receipt_data.get("validation_commands", []):
        lines.append(f"- {command.get('name')}: {command.get('status')} :: {command.get('cmd')}")
    flags = receipt_data.get("missing_signal_flags") or {}
    if flags:
        lines.extend(["", "## Missing Signal Flags"])
        for key, value in sorted(flags.items()):
            lines.append(f"- {key}: {value}")
    lines.extend(["", "## Receiver Apply Commands", "```bash", receipt_data["receiver_apply_command"], "```"])
    Path(path).write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base", required=True)
    parser.add_argument("--head", required=True)
    parser.add_argument("--report", required=True)
    parser.add_argument("--bundle", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--receipt-out", required=True)
    parser.add_argument("--bundle-verify", default="ignored_compat")
    parser.add_argument("--zero-test-reason", default=None)
    args = parser.parse_args()
    data = receipt(args)
    out_receipt = Path(args.receipt_out)
    out_receipt.parent.mkdir(parents=True, exist_ok=True)
    out_receipt.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    write_manifest(args.out, data)


if __name__ == "__main__":
    main()