#!/usr/bin/env python3
"""Create a verified delta receipt and manifest."""
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


APPLY_COMMAND = "git fetch ./{} HEAD && git merge --ff-only FETCH_HEAD"


def fail(message: str) -> None:
    raise SystemExit(message)


def run_git(*args: str, check: bool = True) -> str:
    done = subprocess.run(["git", *args], text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    if check and done.returncode:
        raise SystemExit(done.stderr.strip() or f"git {' '.join(args)} failed")
    return done.stdout.strip()


def git_bundle(*args: str) -> str:
    done = subprocess.run(["git", "bundle", *args], text=True, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, check=False)
    if done.returncode:
        detail = (done.stderr or done.stdout).strip()
        raise SystemExit(detail or f"git bundle {' '.join(args)} failed")
    return done.stdout.strip()


def sha256(path: str) -> str | None:
    p = Path(path)
    return hashlib.sha256(p.read_bytes()).hexdigest() if path and p.exists() else None


def report_rows(path: Path) -> list[dict[str, Any]]:
    return [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]


def last_event(rows: list[dict[str, Any]], event: str) -> dict[str, Any]:
    return next((row for row in reversed(rows) if row.get("event") == event), {})


def command_rows(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [row.get("result", {}) for row in rows if row.get("event") == "validation_command"]


def validation_commands(summary: dict[str, Any], rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    commands = summary.get("validation_commands") or [
        {"name": c.get("name"), "cmd": c.get("cmd"), "status": c.get("status")}
        for c in command_rows(rows)
    ]
    if not commands:
        fail("validation report has no validation commands")
    for index, command in enumerate(commands, 1):
        if not command.get("name"):
            fail(f"validation command {index} has no name")
        if not command.get("status"):
            fail(f"validation command {command.get('name')} has no status")
        if "cmd" not in command:
            fail(f"validation command {command.get('name')} has no cmd field")
    return commands


def positive_int(value: Any, name: str) -> int:
    try:
        result = int(value)
    except (TypeError, ValueError):
        fail(f"{name} is not an integer: {value!r}")
    if result < 0:
        fail(f"{name} is negative: {result}")
    return result


def validation_closure(summary: dict[str, Any], rows: list[dict[str, Any]], head: str,
                       zero_test_reason: str | None) -> tuple[list[dict[str, Any]], str | None]:
    report_head = summary.get("git_head")
    if not report_head:
        fail("validation report has no git_head")
    if report_head != head:
        fail(f"stale validation report: report head {report_head} != manifest head {head}")
    commands = validation_commands(summary, rows)
    command_count = positive_int(summary.get("validation_command_count", len(commands)),
                                 "validation_command_count")
    if command_count <= 0:
        fail("validation_command_count must be greater than zero")
    if command_count != len(commands):
        fail(f"validation_command_count {command_count} != command rows {len(commands)}")
    test_count = positive_int(summary.get("validation_test_count", 0), "validation_test_count")
    reason = zero_test_reason or summary.get("zero_test_reason")
    if test_count == 0 and not reason:
        fail("validation_test_count is zero without zero_test_reason")
    return commands, reason


def changed_files(base: str, head: str) -> list[str]:
    run_git("cat-file", "-e", f"{base}^{{commit}}")
    run_git("cat-file", "-e", f"{head}^{{commit}}")
    if subprocess.run(["git", "merge-base", "--is-ancestor", base, head]).returncode:
        raise SystemExit(f"base is not an ancestor of head: {base}..{head}")
    files = run_git("diff", "--name-only", f"{base}..{head}").splitlines()
    if not files:
        raise SystemExit(f"empty delta: {base}..{head}")
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
            token = line.strip().split(maxsplit=1)[0] if line.strip() else ""
            if token:
                refs.append(token)
    return refs


def verify_bundle(bundle: str, base: str, head: str) -> tuple[str, list[str], list[str]]:
    if not bundle:
        return "not_provided", [], []
    path = Path(bundle)
    if not path.exists():
        raise SystemExit(f"bundle does not exist: {bundle}")
    verify_output = git_bundle("verify", bundle)
    heads = git_bundle("list-heads", bundle).splitlines()
    if not any(line.startswith(head) for line in heads):
        raise SystemExit(f"bundle does not expose expected head: {head}")
    required = bundle_required_refs(verify_output)
    if base not in required:
        raise SystemExit(f"bundle does not require base commit {base}")
    return "pass", heads, required


def receipt(args: argparse.Namespace) -> dict[str, Any]:
    rows = report_rows(Path(args.report))
    summary = last_event(rows, "validation_summary")
    if not summary:
        fail("validation report has no validation_summary event")
    commands, zero_test_reason = validation_closure(summary, rows, args.head, args.zero_test_reason)
    report_head = summary.get("git_head")
    bundle_verify, bundle_heads, bundle_required = verify_bundle(args.bundle, args.base, args.head)
    r = {
        "schema_version": 2,
        "base_commit": args.base,
        "head_commit": args.head,
        "changed_files": changed_files(args.base, args.head),
        "validation_status": summary.get("validation_status", "unknown"),
        "validation_commands": commands,
        "validation_command_count": summary.get("validation_command_count", len(commands)),
        "validation_test_count": summary.get("validation_test_count", 0),
        "zero_test_reason": zero_test_reason,
        "python_unit_test_count": summary.get("python_unit_test_count", 0),
        "router_test_count": summary.get("router_test_count", 0),
        "cargo_test_count_when_available": summary.get("cargo_test_count_when_available"),
        "failed_required_commands": summary.get("failed_required_commands", []),
        "missing_signal_flags": summary.get("missing_signal_flags", {}),
        "missing_signal_count": summary.get("missing_signal_count"),
        "report_path": args.report,
        "report_sha256": sha256(args.report),
        "bundle_path": args.bundle or None,
        "bundle_sha256": sha256(args.bundle),
        "bundle_verify": bundle_verify,
        "bundle_heads": bundle_heads,
        "bundle_required_refs": bundle_required,
        "bundle_requires_base_commit": args.base in bundle_required,
        "receiver_apply_command": APPLY_COMMAND.format(Path(args.bundle or "repo-delta.bundle").name),
        "validation_report_git_head": report_head,
    }
    r["changed_file_count"] = len(r["changed_files"])
    for key in [
        "cargo_available",
        "rustc_available",
        "toolchain_path_added",
        "rust_toolchain_source",
        "root_rust_env_overrides",
        "wrapper_override_required",
        "wrapper_override_used",
        "wrapper_override_env",
        "wrapper_graph_validation_result",
        "wrapper_graph_validation_requested",
        "wrapper_graph_validation_available",
        "rustc_wrapper_configured",
        "rustc_wrapper_path_exists",
        "git_delta_diff_check_result",
        "state_graph_present",
        "runtime_archive_present",
        "runtime_archive_sha256",
        "runtime_manifest_base_commit",
        "runtime_archive_log_total",
        "runtime_archive_download_total",
        "runtime_performance_signal_present",
        "runtime_performance_budget_status",
        "runtime_performance_budget_failures",
        "runtime_performance_budgets",
        "validation_command_duration_ms",
        "project_agent_elapsed_ms_count",
        "project_agent_elapsed_ms_median",
        "project_agent_elapsed_ms_p95",
        "project_agent_elapsed_ms_max",
        "download_initial_get_ms_median",
        "download_initial_get_ms_p95",
        "download_initial_get_ms_max",
        "download_follow_get_ms_median",
        "download_follow_get_ms_p95",
        "download_follow_get_ms_max",
        "download_resolved_get_ms_median",
        "download_resolved_get_ms_p95",
        "download_resolved_get_ms_max",
        "download_write_ms_median",
        "download_write_ms_p95",
        "download_write_ms_max",
        "runtime_download_history_record_count",
        "runtime_unique_download_alias_count",
        "runtime_unique_download_aliases",
        "runtime_archive_conversation_snapshots",
        "runtime_stale_advisory_count",
        "runtime_manifest_base_expected",
        "runtime_manifest_base_matches_delta_base",
        "runtime_candidate_error_count",
        "runtime_duplicate_artifact_aliases",
        "delta_base_is_ancestor",
        "delta_changed_file_count",
        "tracked_file_count",
        "rust_file_count_src_examples",
        "rust_test_attr_count",
        "rust_cfg_test_count",
        "unwrap_call_count_src_examples",
        "expect_call_count_src_examples",
    ]:
        r[key] = summary.get(key)
    return r


def write_manifest(path: Path, r: dict[str, Any]) -> None:
    lines = [
        f"base_commit: {r['base_commit']}",
        f"head_commit: {r['head_commit']}",
        "",
        "# Delta Manifest",
        "",
        "## Changed Files",
        *[f"- {name}" for name in r["changed_files"]],
        "",
        "## Validation Results",
        f"- validation_status: {r['validation_status']}",
        f"- validation_command_count: {r['validation_command_count']}",
        f"- validation_test_count: {r['validation_test_count']}",
        f"- zero_test_reason: {r['zero_test_reason']}",
        f"- python_unit_test_count: {r['python_unit_test_count']}",
        f"- router_test_count: {r['router_test_count']}",
        f"- cargo_test_count_when_available: {r['cargo_test_count_when_available']}",
        f"- failed_required_commands: {json.dumps(r['failed_required_commands'], sort_keys=True)}",
        f"- missing_signal_count: {r['missing_signal_count']}",
        f"- report_sha256: {r['report_sha256']}",
        f"- bundle_sha256: {r['bundle_sha256']}",
        f"- bundle_verify: {r['bundle_verify']}",
        f"- bundle_heads: {json.dumps(r['bundle_heads'], sort_keys=True)}",
        f"- bundle_required_refs: {json.dumps(r['bundle_required_refs'], sort_keys=True)}",
        f"- bundle_requires_base_commit: {r['bundle_requires_base_commit']}",
        f"- validation_report_git_head: {r['validation_report_git_head']}",
        f"- changed_file_count: {r['changed_file_count']}",
        f"- cargo_available: {r['cargo_available']}",
        f"- rustc_available: {r['rustc_available']}",
        f"- toolchain_path_added: {r['toolchain_path_added']}",
        f"- rust_toolchain_source: {r['rust_toolchain_source']}",
        f"- root_rust_env_overrides: {json.dumps(r['root_rust_env_overrides'], sort_keys=True)}",
        f"- wrapper_override_required: {r['wrapper_override_required']}",
        f"- wrapper_override_used: {r['wrapper_override_used']}",
        f"- wrapper_override_env: {json.dumps(r['wrapper_override_env'], sort_keys=True)}",
        f"- wrapper_graph_validation_result: {r['wrapper_graph_validation_result']}",
        f"- wrapper_graph_validation_requested: {r['wrapper_graph_validation_requested']}",
        f"- wrapper_graph_validation_available: {r['wrapper_graph_validation_available']}",
        f"- rustc_wrapper_path_exists: {r['rustc_wrapper_path_exists']}",
        f"- git_delta_diff_check_result: {r['git_delta_diff_check_result']}",
        f"- state_graph_present: {r['state_graph_present']}",
        f"- runtime_archive_sha256: {r['runtime_archive_sha256']}",
        f"- runtime_manifest_base_commit: {r['runtime_manifest_base_commit']}",
        f"- runtime_archive_log_total: {r['runtime_archive_log_total']}",
        f"- runtime_archive_download_total: {r['runtime_archive_download_total']}",
        f"- runtime_performance_signal_present: {r['runtime_performance_signal_present']}",
        f"- runtime_performance_budget_status: {r['runtime_performance_budget_status']}",
        f"- runtime_performance_budget_failures: {json.dumps(r['runtime_performance_budget_failures'], sort_keys=True)}",
        f"- runtime_performance_budgets: {json.dumps(r['runtime_performance_budgets'], sort_keys=True)}",
        f"- validation_command_duration_ms: {json.dumps(r['validation_command_duration_ms'], sort_keys=True)}",
        f"- project_agent_elapsed_ms_count: {r['project_agent_elapsed_ms_count']}",
        f"- project_agent_elapsed_ms_median: {r['project_agent_elapsed_ms_median']}",
        f"- project_agent_elapsed_ms_p95: {r['project_agent_elapsed_ms_p95']}",
        f"- project_agent_elapsed_ms_max: {r['project_agent_elapsed_ms_max']}",
        f"- download_initial_get_ms_median: {r['download_initial_get_ms_median']}",
        f"- download_initial_get_ms_p95: {r['download_initial_get_ms_p95']}",
        f"- download_initial_get_ms_max: {r['download_initial_get_ms_max']}",
        f"- download_follow_get_ms_median: {r['download_follow_get_ms_median']}",
        f"- download_follow_get_ms_p95: {r['download_follow_get_ms_p95']}",
        f"- download_follow_get_ms_max: {r['download_follow_get_ms_max']}",
        f"- download_resolved_get_ms_median: {r['download_resolved_get_ms_median']}",
        f"- download_resolved_get_ms_p95: {r['download_resolved_get_ms_p95']}",
        f"- download_resolved_get_ms_max: {r['download_resolved_get_ms_max']}",
        f"- download_write_ms_median: {r['download_write_ms_median']}",
        f"- download_write_ms_p95: {r['download_write_ms_p95']}",
        f"- download_write_ms_max: {r['download_write_ms_max']}",
        f"- runtime_download_history_record_count: {r['runtime_download_history_record_count']}",
        f"- runtime_unique_download_alias_count: {r['runtime_unique_download_alias_count']}",
        f"- runtime_unique_download_aliases: {json.dumps(r['runtime_unique_download_aliases'], sort_keys=True)}",
        f"- runtime_archive_conversation_snapshots: {r['runtime_archive_conversation_snapshots']}",
        f"- runtime_stale_advisory_count: {r['runtime_stale_advisory_count']}",
        f"- runtime_manifest_base_expected: {r['runtime_manifest_base_expected']}",
        f"- runtime_manifest_base_matches_delta_base: {r['runtime_manifest_base_matches_delta_base']}",
        f"- runtime_candidate_error_count: {r['runtime_candidate_error_count']}",
        f"- runtime_duplicate_artifact_aliases: {r['runtime_duplicate_artifact_aliases']}",
        f"- delta_base_is_ancestor: {r['delta_base_is_ancestor']}",
        f"- delta_changed_file_count: {r['delta_changed_file_count']}",
        f"- tracked_file_count: {r['tracked_file_count']}",
        f"- rust_file_count_src_examples: {r['rust_file_count_src_examples']}",
        f"- rust_test_attr_count: {r['rust_test_attr_count']}",
        f"- rust_cfg_test_count: {r['rust_cfg_test_count']}",
        f"- unwrap_call_count_src_examples: {r['unwrap_call_count_src_examples']}",
        f"- expect_call_count_src_examples: {r['expect_call_count_src_examples']}",
        "",
        "## Validation Commands",
    ]
    lines += [f"- {c.get('name')}: {c.get('status')} :: {' '.join(map(str, c.get('cmd') or []))}" for c in r["validation_commands"]]
    lines += ["", "## Missing Signal Flags"]
    lines += [f"- {key}: {value}" for key, value in sorted(r["missing_signal_flags"].items())]
    lines += ["", "## Receiver Apply Commands", "```bash", r["receiver_apply_command"], "```", ""]
    path.write_text("\n".join(lines), encoding="utf-8")


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--base", required=True)
    p.add_argument("--head", required=True)
    p.add_argument("--report", required=True)
    p.add_argument("--out", required=True)
    p.add_argument("--receipt-out", required=True)
    p.add_argument("--bundle", default="")
    p.add_argument("--bundle-verify", default="ignored_compat")
    p.add_argument("--zero-test-reason", default=None)
    args = p.parse_args()

    r = receipt(args)
    Path(args.receipt_out).parent.mkdir(parents=True, exist_ok=True)
    Path(args.receipt_out).write_text(json.dumps(r, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    write_manifest(Path(args.out), r)


if __name__ == "__main__":
    main()
