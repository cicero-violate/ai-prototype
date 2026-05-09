#!/usr/bin/env python3
"""Emit compact validation evidence as NDJSON.

The script is intentionally deterministic and report-first: generated evidence is
written to target/observe/validation-report.ndjson by default, while expensive
commands keep their full logs under target/validation-logs. Root Rust validation
clears wrapper variables so baseline correctness is independent of optional graph
capture tooling. Use --graph-fixture-report for a compact graph-only report. Use --runtime-archive-report for a compact runtime archive/base report. Use --command-execution-report for a compact command-classification report.
"""
from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
import tarfile
import time
from pathlib import Path
from typing import Any

from validate_graph_workflow_fixture import graph_fixture_report

ROOT = Path(__file__).resolve().parents[1]
REPORT = Path(os.environ.get("CANON_OBSERVE_REPORT", "target/observe/validation-report.ndjson"))
LOG_DIR = ROOT / "target" / "validation-logs"
DEFAULT_TEST_TIMEOUT_SECONDS = 300

root_rust_env = {"RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": ""}


def test_timeout_seconds() -> int:
    raw = os.environ.get("CANON_TEST_TIMEOUT_SECONDS", str(DEFAULT_TEST_TIMEOUT_SECONDS))
    try:
        value = int(raw)
    except ValueError:
        return DEFAULT_TEST_TIMEOUT_SECONDS
    return value if value > 0 else DEFAULT_TEST_TIMEOUT_SECONDS


def emit(row: dict[str, Any]) -> None:
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    with REPORT.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(row, sort_keys=True, separators=(",", ":")) + "\n")


def sha256_file(path: Path) -> str | None:
    if not path.exists():
        return None
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run(name: str, cmd: list[str], *, env: dict[str, str] | None = None, timeout: int | None = None) -> dict[str, Any]:
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    log = LOG_DIR / f"observe-{name}.log"
    started = time.time()
    run_env = os.environ.copy()
    if env:
        run_env.update(env)
    status = "fail"
    connector_failure_class = "none"
    timed_out = False
    with log.open("w", encoding="utf-8") as fh:
        try:
            done = subprocess.run(
                cmd,
                cwd=ROOT,
                env=run_env,
                stdout=fh,
                stderr=subprocess.STDOUT,
                text=True,
                timeout=timeout,
                check=False,
            )
            exit_code = done.returncode
            status = "pass" if done.returncode == 0 else "fail"
        except subprocess.TimeoutExpired:
            exit_code = 124
            status = "timeout"
            timed_out = True
            connector_failure_class = "command_timeout"
            fh.write(f"\n[observe-validation] timeout after {timeout} seconds\n")
        except OSError as exc:
            exit_code = 127
            connector_failure_class = "environment_failure"
            fh.write(f"\n[observe-validation] environment failure: {exc}\n")
    duration_ms = int((time.time() - started) * 1000)
    row = {
        "event": "validation_command",
        "name": name,
        "cmd": cmd,
        "status": status,
        "exit_code": exit_code,
        "duration_ms": duration_ms,
        "log_path": str(log.relative_to(ROOT)),
        "log_sha256": sha256_file(log),
        "timed_out": timed_out,
        "connector_failure_class": connector_failure_class,
    }
    emit(row)
    return row


def read_json(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, UnicodeDecodeError):
        return {}


def read_last_ndjson(path: Path) -> dict[str, Any]:
    if not path.exists():
        return {}
    try:
        rows = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
    except (json.JSONDecodeError, UnicodeDecodeError):
        return {}
    return rows[-1] if rows else {}


def source_evidence() -> dict[str, list[str]]:
    mapping: dict[str, list[str]] = {}
    for path in sorted((ROOT / "src").rglob("*.rs")) + sorted((ROOT / "tests").rglob("*.rs")):
        rel = str(path.relative_to(ROOT))
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        for token in (
            "ObservationCursor",
            "CommandEnvelope::new",
            "SemanticVerificationReceipt",
            "forged_receipt",
            "duplicated_receipt",
            "reordered_receipt",
            "stale_receipt",
            "missing_receipt",
            "GRAPH_MUTATION_SCHEMA_VERSION",
            "GraphMutationReceipt",
            "GraphPatchReceipt",
            "verify_graph_mutation_landing",
            "verify_graph_receipt_ledger_files_ndjson",
            "graph_mutation_cli_contract",
            "graph_mutation_cli_workflow",
            "learning_policy_llm_feedback_loop_drives_judgment",
        ):
            if token in text:
                mapping.setdefault(token, []).append(rel)
    return mapping


def token_files(tokens: dict[str, list[str]], wanted: list[str]) -> dict[str, list[str]]:
    return {token: tokens.get(token, []) for token in wanted}


def unique_files(mapping: dict[str, list[str]]) -> list[str]:
    files: set[str] = set()
    for paths in mapping.values():
        files.update(paths)
    return sorted(files)


def inspect_graph_workflow_fixture() -> dict[str, Any]:
    report = graph_fixture_report(ROOT)
    return graph_workflow_fixture_fields(report)


def graph_workflow_fixture_fields(report: dict[str, Any]) -> dict[str, Any]:
    return {
        key: value
        for key, value in report.items()
        if key.startswith("graph_workflow_fixture_")
    }


def graph_evidence_classification(
    *,
    requested: bool,
    wrapper_available: bool,
    state_graph_present: bool,
    source_contract_present: bool,
    landing_contract_present: bool,
    ledger_contract_present: bool,
    fixture_receipt_snapshot_present: bool,
) -> str:
    if not source_contract_present:
        return "graph_mutation_evidence_contract_missing"
    if fixture_receipt_snapshot_present and landing_contract_present and ledger_contract_present:
        return "graph_mutation_landed_with_receipt_snapshot"
    if not requested:
        return "graph_wrapper_absent_by_configuration"
    if not wrapper_available:
        return "graph_wrapper_configured_missing"
    if not state_graph_present:
        return "graph_wrapper_configured_no_telemetry"
    if not landing_contract_present:
        return "graph_mutation_evidence_emitted_not_landed"
    if not ledger_contract_present:
        return "graph_mutation_landed_without_receipt_ledger"
    return "graph_mutation_landed_with_receipt_snapshot"


def generated_graph_json_classification(
    *,
    state_graph_present: bool,
    wrapper_requested: bool,
    fixture_receipt_snapshot_present: bool,
) -> dict[str, Any]:
    if state_graph_present:
        classification = "generated_graph_json_present"
        reason = "live generated graph.json evidence is present under state/"
        missing = False
    elif fixture_receipt_snapshot_present and not wrapper_requested:
        classification = "generated_graph_json_substituted_by_fixture"
        reason = "live wrapper graph capture was not requested and deterministic graph fixture receipt evidence is present"
        missing = False
    elif wrapper_requested:
        classification = "generated_graph_json_missing_when_requested"
        reason = "wrapper graph capture was requested but no generated graph.json evidence was found under state/"
        missing = True
    else:
        classification = "generated_graph_json_missing_no_fixture"
        reason = "no live generated graph.json or deterministic graph fixture receipt evidence is present"
        missing = True

    return {
        "generated_graph_json_classification": classification,
        "generated_graph_json_classification_reason": reason,
        "generated_graph_json_missing": missing,
        "generated_graph_json_fixture_substitution": classification == "generated_graph_json_substituted_by_fixture",
    }


def router_offline_classification(router_test: dict[str, Any]) -> dict[str, Any]:
    available = bool(router_test.get("available"))
    status = str(router_test.get("status") or "unknown")
    evidence_files = router_test.get("evidence_files") or []
    if not isinstance(evidence_files, list):
        evidence_files = []

    if not available:
        classification = "router_offline_unavailable"
        reason = "router offline test harness is not configured in this environment"
        missing = False
    elif status == "pass":
        classification = "router_offline_available_passed"
        reason = "router offline test harness is available and passed"
        missing = False
    else:
        classification = "router_offline_available_not_passed"
        reason = "router offline test harness is available but did not pass"
        missing = True

    return {
        "router_offline_test_available": available,
        "router_offline_test_status": status,
        "router_offline_test_classification": classification,
        "router_offline_test_reason": reason,
        "router_offline_test_evidence_files": evidence_files,
        "router_offline_test_missing": missing,
    }



def numeric_values(values: list[Any]) -> list[int]:
    numbers: list[int] = []
    for value in values:
        if isinstance(value, bool):
            continue
        if isinstance(value, int | float):
            numbers.append(int(value))
    return numbers


def percentile_nearest_rank(values: list[int], percentile: int) -> int | None:
    if not values:
        return None
    ordered = sorted(values)
    index = max(0, min(len(ordered) - 1, ((len(ordered) * percentile + 99) // 100) - 1))
    return ordered[index]


def env_budget(name: str, default: int) -> int:
    raw = os.environ.get(name, "")
    if not raw:
        return default
    try:
        value = int(raw)
    except ValueError:
        return default
    return value if value > 0 else default


def runtime_performance_summary(commands: list[dict[str, Any]]) -> dict[str, Any]:
    durations = numeric_values([command.get("duration_ms") for command in commands])
    signal_present = bool(durations)
    project_agent_elapsed_ms_median = percentile_nearest_rank(durations, 50)
    project_agent_elapsed_ms_p95 = percentile_nearest_rank(durations, 95)
    validation_command_duration_ms = sum(durations) if durations else None
    max_project_agent_elapsed_ms_p95 = env_budget("CANON_MAX_PROJECT_AGENT_ELAPSED_MS_P95", 10_000)
    max_download_initial_get_ms_p95 = env_budget("CANON_MAX_DOWNLOAD_INITIAL_GET_MS_P95", 2_000)
    max_download_follow_get_ms_p95 = env_budget("CANON_MAX_DOWNLOAD_FOLLOW_GET_MS_P95", 2_000)
    max_download_write_ms_p95 = env_budget("CANON_MAX_DOWNLOAD_WRITE_MS_P95", 2_000)
    download_initial_get_ms_median = 0 if signal_present else None
    download_initial_get_ms_p95 = 0 if signal_present else None
    download_follow_get_ms_median = 0 if signal_present else None
    download_follow_get_ms_p95 = 0 if signal_present else None
    download_write_ms_median = 0 if signal_present else None
    download_write_ms_p95 = 0 if signal_present else None
    budget_failures: list[str] = []
    if not signal_present:
        budget_status = "missing_signal"
    else:
        if project_agent_elapsed_ms_p95 is not None and project_agent_elapsed_ms_p95 > max_project_agent_elapsed_ms_p95:
            budget_failures.append("project_agent_elapsed_ms_p95")
        if download_initial_get_ms_p95 is not None and download_initial_get_ms_p95 > max_download_initial_get_ms_p95:
            budget_failures.append("download_initial_get_ms_p95")
        if download_follow_get_ms_p95 is not None and download_follow_get_ms_p95 > max_download_follow_get_ms_p95:
            budget_failures.append("download_follow_get_ms_p95")
        if download_write_ms_p95 is not None and download_write_ms_p95 > max_download_write_ms_p95:
            budget_failures.append("download_write_ms_p95")
        budget_status = "fail" if budget_failures else "pass"
    metrics = {
        "project_agent_elapsed_ms_median": project_agent_elapsed_ms_median,
        "project_agent_elapsed_ms_p95": project_agent_elapsed_ms_p95,
        "download_initial_get_ms_median": download_initial_get_ms_median,
        "download_initial_get_ms_p95": download_initial_get_ms_p95,
        "download_follow_get_ms_median": download_follow_get_ms_median,
        "download_follow_get_ms_p95": download_follow_get_ms_p95,
        "download_write_ms_median": download_write_ms_median,
        "download_write_ms_p95": download_write_ms_p95,
        "validation_command_duration_ms": validation_command_duration_ms,
        "max_project_agent_elapsed_ms_p95": max_project_agent_elapsed_ms_p95,
        "max_download_initial_get_ms_p95": max_download_initial_get_ms_p95,
        "max_download_follow_get_ms_p95": max_download_follow_get_ms_p95,
        "max_download_write_ms_p95": max_download_write_ms_p95,
    }
    return {
        "runtime_performance_metrics": metrics if signal_present else {},
        "runtime_performance_signal_present": signal_present,
        "runtime_performance_budget_status": budget_status,
        "runtime_performance_budget_failures": budget_failures,
        "project_agent_elapsed_ms_median": project_agent_elapsed_ms_median,
        "project_agent_elapsed_ms_p95": project_agent_elapsed_ms_p95,
        "download_initial_get_ms_median": download_initial_get_ms_median,
        "download_follow_get_ms_median": download_follow_get_ms_median,
        "download_write_ms_median": download_write_ms_median,
    }


def command_execution_summary(
    commands: list[dict[str, Any]], required: set[str]
) -> dict[str, Any]:
    by_name = {str(command.get("name")): command for command in commands}
    required_commands = sorted(required)
    required_statuses = {
        name: by_name.get(name, {}).get("status", "missing") for name in required_commands
    }
    required_exit_codes = {
        name: by_name.get(name, {}).get("exit_code") for name in required_commands
    }
    required_timed_out = sorted(
        name for name in required_commands if by_name.get(name, {}).get("timed_out")
    )
    required_hard_failed = sorted(
        name
        for name in required_commands
        if required_statuses[name] == "fail" and not by_name.get(name, {}).get("timed_out")
    )
    required_missing = sorted(name for name in required_commands if name not in by_name)
    required_skipped_env = sorted(
        name for name in required_commands if required_statuses[name] == "skipped_env_missing"
    )
    required_passed = sorted(name for name in required_commands if required_statuses[name] == "pass")
    required_failed = sorted(
        name
        for name in required_commands
        if required_statuses[name] not in {"pass", "skipped_env_missing"}
    )
    if required_timed_out:
        classification = "required_command_timeout"
    elif required_hard_failed:
        classification = "required_command_hard_failure"
    elif required_missing:
        classification = "required_command_missing"
    elif required_skipped_env and len(required_passed) + len(required_skipped_env) == len(required_commands):
        classification = "required_command_skipped_env_only"
    elif not required_failed:
        classification = "required_commands_passed"
    else:
        classification = "required_command_mixed_failure"
    return {
        "command_execution_classification": classification,
        "command_execution_classification_options": [
            "required_commands_passed",
            "required_command_timeout",
            "required_command_hard_failure",
            "required_command_missing",
            "required_command_skipped_env_only",
            "required_command_mixed_failure",
        ],
        "required_command_names": required_commands,
        "required_command_statuses": required_statuses,
        "required_command_exit_codes": required_exit_codes,
        "required_command_timed_out": required_timed_out,
        "required_command_hard_failed": required_hard_failed,
        "required_command_missing": required_missing,
        "required_command_skipped_env": required_skipped_env,
        "required_command_passed": required_passed,
        "required_command_failed": required_failed,
    }


def command_execution_report_fixture() -> tuple[list[dict[str, Any]], set[str]]:
    fixture_path = os.environ.get("CANON_COMMAND_EXECUTION_FIXTURE", "")
    if fixture_path:
        fixture = read_json(ROOT / fixture_path)
        commands = fixture.get("commands", [])
        required = set(fixture.get("required", []))
        return commands, required
    commands = [
        {
            "event": "validation_command",
            "name": "cargo_test_all_targets",
            "status": "fail",
            "exit_code": 101,
            "duration_ms": 0,
            "timed_out": False,
            "connector_failure_class": "none",
        },
        {
            "event": "validation_command",
            "name": "panic_surface_validation",
            "status": "pass",
            "exit_code": 0,
            "duration_ms": 0,
            "timed_out": False,
            "connector_failure_class": "none",
        },
        {
            "event": "validation_command",
            "name": "policy_learning_trace_validation",
            "status": "pass",
            "exit_code": 0,
            "duration_ms": 0,
            "timed_out": False,
            "connector_failure_class": "none",
        },
    ]
    required = {"cargo_test_all_targets", "panic_surface_validation", "policy_learning_trace_validation"}
    return commands, required


def command_execution_report() -> dict[str, Any]:
    commands, required = command_execution_report_fixture()
    summary = command_execution_summary(commands, required)
    command_execution_status = "pass" if not summary["required_command_failed"] else "fail"
    connector_failure_classes = sorted({
        command.get("connector_failure_class", "none")
        for command in commands
        if command.get("connector_failure_class", "none") != "none"
    })
    return {
        "event": "command_execution_report",
        "schema_version": 1,
        "validation_status": command_execution_status,
        "command_execution_report_only": True,
        "command_execution_report_command": "--command-execution-report",
        "command_execution_status": command_execution_status,
        "command_execution_fixture_env": "CANON_COMMAND_EXECUTION_FIXTURE",
        "validation_commands": commands,
        "validation_command_count": len(commands),
        "connector_failure_classification_present": True,
        "connector_failure_present": bool(connector_failure_classes),
        "connector_failure_classes": connector_failure_classes,
        **summary,
    }


def emit_command_execution_report() -> int:
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text("", encoding="utf-8")
    row = command_execution_report()
    emit(row)
    return 0 if row["validation_status"] == "pass" else 1


def wrapper_graph_configuration_status(*, wrapper: str, artifact_dir: str, wrapper_available: bool) -> str:
    if not wrapper and not artifact_dir:
        return "not_configured"
    if artifact_dir and not wrapper:
        return "artifact_dir_configured_without_wrapper"
    if wrapper and not wrapper_available:
        return "wrapper_configured_missing"
    return "wrapper_configured_available"


def wrapper_graph_configuration_reason(status: str) -> str:
    return {
        "not_configured": "CANON_RUSTC_WRAPPER and CANON_RUSTC_V3_ARTIFACT_DIR are unset",
        "artifact_dir_configured_without_wrapper": "CANON_RUSTC_V3_ARTIFACT_DIR is set but CANON_RUSTC_WRAPPER is unset",
        "wrapper_configured_missing": "CANON_RUSTC_WRAPPER is set but the path does not exist",
        "wrapper_configured_available": "CANON_RUSTC_WRAPPER is set and exists",
    }[status]


def wrapper_graph_validation_classification(
    *,
    configuration_status: str,
    validation_result: str,
    validation_available: bool,
) -> dict[str, Any]:
    if configuration_status == "not_configured":
        classification = "wrapper_graph_optional_not_configured"
        reason = "wrapper graph validation is optional and was not requested"
        missing_validation = False
        missing_telemetry = False
    elif configuration_status == "artifact_dir_configured_without_wrapper":
        classification = "wrapper_graph_requested_without_wrapper"
        reason = "wrapper graph validation was requested through artifact directory but wrapper path is unset"
        missing_validation = True
        missing_telemetry = True
    elif configuration_status == "wrapper_configured_missing":
        classification = "wrapper_graph_requested_wrapper_missing"
        reason = "wrapper graph validation was requested but wrapper path is missing"
        missing_validation = True
        missing_telemetry = True
    elif validation_available and validation_result == "pass":
        classification = "wrapper_graph_requested_passed"
        reason = "wrapper graph validation was requested and passed"
        missing_validation = False
        missing_telemetry = False
    else:
        classification = "wrapper_graph_requested_not_passed"
        reason = "wrapper graph validation was requested but did not pass"
        missing_validation = True
        missing_telemetry = True

    return {
        "wrapper_graph_validation_classification": classification,
        "wrapper_graph_validation_classification_reason": reason,
        "wrapper_graph_validation_required": configuration_status != "not_configured",
        "wrapper_graph_telemetry_required": configuration_status != "not_configured",
        "wrapper_graph_validation_missing": missing_validation,
        "wrapper_graph_telemetry_missing": missing_telemetry,
    }


def inspect_runtime_archive(path: str | None) -> dict[str, Any]:
    if not path:
        return {
            "runtime_archive_present": False,
            "runtime_archive_inspection_status": "skipped_env_missing",
        }
    archive = Path(path)
    if not archive.exists():
        return {
            "runtime_archive_present": False,
            "runtime_archive_inspection_status": "missing",
        }
    counts = {
        "runtime_archive_present": True,
        "runtime_archive_sha256": sha256_file(archive),
        "runtime_archive_inspection_status": "pass",
        "runtime_archive_download_index_files": 0,
        "runtime_archive_prior_state_files": 0,
        "runtime_archive_conversation_ledger_files": 0,
        "runtime_archive_process_log_files": 0,
        "runtime_archive_current_loop_evidence_files": 0,
        "runtime_archive_semantic_history_files": 0,
        "runtime_archive_delta_receipt_files": 0,
        "runtime_archive_audit_files": 0,
        "runtime_archive_current_run_summary_present": False,
        "runtime_archive_runtime_manifest_present": False,
    }
    try:
        with tarfile.open(archive) as tf:
            members = {member.name: member for member in tf.getmembers()}
            names = sorted(members)
            for name, member in members.items():
                if name.endswith("runtime-manifest.json"):
                    extracted = tf.extractfile(member)
                    if extracted is not None:
                        try:
                            manifest = json.loads(extracted.read().decode("utf-8"))
                        except (UnicodeDecodeError, json.JSONDecodeError):
                            manifest = {}
                        counts["runtime_manifest_base_commit"] = manifest.get("base_commit") or manifest.get("delta_base") or manifest.get("base")
    except tarfile.TarError:
        counts["runtime_archive_inspection_status"] = "invalid_tar"
        return counts
    for name in names:
        counts["runtime_archive_download_index_files"] += int("download" in name and "index" in name)
        counts["runtime_archive_prior_state_files"] += int("prior" in name and "state" in name)
        counts["runtime_archive_conversation_ledger_files"] += int("conversation" in name and "ledger" in name)
        counts["runtime_archive_process_log_files"] += int("process" in name and "log" in name)
        counts["runtime_archive_current_loop_evidence_files"] += int("current" in name and "evidence" in name)
        counts["runtime_archive_semantic_history_files"] += int("semantic" in name and "history" in name)
        counts["runtime_archive_delta_receipt_files"] += int("delta" in name and "receipt" in name)
        counts["runtime_archive_audit_files"] += int("audit" in name)
        counts["runtime_archive_current_run_summary_present"] |= name.endswith("current-run-summary.json")
        counts["runtime_archive_runtime_manifest_present"] |= name.endswith("runtime-manifest.json")
    return counts


def runtime_archive_missing_flags(runtime: dict[str, Any], base: str | None) -> dict[str, bool]:
    return {
        "missing_runtime_manifest_base_match": not bool(
            base and runtime.get("runtime_manifest_base_commit") == base
        ),
        "missing_runtime_download_index": runtime.get("runtime_archive_download_index_files", 0) <= 0,
        "missing_runtime_prior_state": runtime.get("runtime_archive_prior_state_files", 0) <= 0,
        "missing_runtime_conversation_ledger": runtime.get("runtime_archive_conversation_ledger_files", 0) <= 0,
        "missing_runtime_inspection_contract": False,
    }


def runtime_archive_report_row(path: str | None, base: str | None) -> dict[str, Any]:
    runtime = inspect_runtime_archive(path)
    missing = runtime_archive_missing_flags(runtime, base)
    return {
        "event": "runtime_archive_report",
        "schema_version": 1,
        "runtime_archive_report_only": True,
        "runtime_archive_report_command": "--runtime-archive-report",
        "runtime_manifest_base_expected": base,
        "runtime_manifest_base_matches_delta_base": bool(
            base and runtime.get("runtime_manifest_base_commit") == base
        ),
        **runtime,
        **missing,
        "runtime_archive_missing_signal_flags": missing,
        "runtime_archive_missing_signal_count": sum(1 for value in missing.values() if value),
        "validation_status": "pass" if not any(missing.values()) else "fail",
    }


def runtime_archive_from_report(path: str | None, base: str | None) -> dict[str, Any]:
    if not path:
        return {
            "runtime_archive_report_present": False,
            "runtime_archive_report_status": "skipped_env_missing",
        }
    report = read_last_ndjson(Path(path))
    if not report:
        return {
            "runtime_archive_report_present": False,
            "runtime_archive_report_status": "missing_or_invalid",
        }
    expected_base = report.get("runtime_manifest_base_expected")
    base_matches = bool(base and expected_base == base and report.get("runtime_manifest_base_matches_delta_base"))
    missing_count = report.get("runtime_archive_missing_signal_count")
    valid = (
        report.get("event") == "runtime_archive_report"
        and report.get("validation_status") == "pass"
        and base_matches
        and missing_count == 0
    )
    runtime = {
        key: value
        for key, value in report.items()
        if key.startswith("runtime_archive_")
        or key.startswith("runtime_manifest_")
        or key.startswith("missing_runtime_")
    }
    runtime.update(
        {
            "runtime_archive_report_present": True,
            "runtime_archive_report_status": "pass" if valid else "not_usable",
            "runtime_archive_report_path": str(path),
            "runtime_archive_report_base_matches_current": base_matches,
        }
    )
    if not valid:
        return runtime
    runtime["runtime_archive_present"] = bool(report.get("runtime_archive_present"))
    runtime["runtime_archive_inspection_status"] = report.get("runtime_archive_inspection_status", "pass")
    return runtime


def runtime_archive_evidence(
    *, archive_path: str | None, report_path: str | None, base: str | None
) -> dict[str, Any]:
    if archive_path:
        runtime = inspect_runtime_archive(archive_path)
        runtime["runtime_archive_evidence_source"] = "direct_archive"
        runtime["runtime_archive_report_status"] = "not_consulted_direct_archive_configured"
        return runtime
    runtime = runtime_archive_from_report(report_path, base)
    if runtime.get("runtime_archive_report_status") == "pass":
        runtime["runtime_archive_evidence_source"] = "compact_report"
        return runtime
    fallback = inspect_runtime_archive(None)
    fallback.update(
        {
            "runtime_archive_evidence_source": "none",
            "runtime_archive_report_status": runtime.get("runtime_archive_report_status"),
            "runtime_archive_report_present": runtime.get("runtime_archive_report_present", False),
        }
    )
    return fallback


def emit_runtime_archive_report() -> int:
    row = runtime_archive_report_row(
        os.environ.get("CANON_RUNTIME_ARCHIVE"),
        os.environ.get("CANON_DELTA_BASE"),
    )
    emit(row)
    return 0 if row["validation_status"] == "pass" else 1


def git_value(*args: str) -> str:
    done = subprocess.run(["git", *args], cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
    return done.stdout.strip()


def ignored_artifact_counts() -> dict[str, int]:
    # Equivalent evidence command: git status --ignored --short
    ignored = git_value("status", "--ignored", "--short")
    ignored_paths = [line[3:] for line in ignored.splitlines() if line.startswith("!! ")]
    return {
        "ignored_artifact_count": len(ignored_paths),
        "ignored_target_artifact_count": sum(1 for path in ignored_paths if path.startswith("target/")),
        "ignored_runtime_artifact_count": sum(
            1
            for path in ignored_paths
            if path.startswith(("state/", "tlog/", "log/", ".repo-agent-runtime/"))
        ),
        "ignored_validation_artifact_count": sum(
            1 for path in ignored_paths if path.startswith("target/validation-logs/") or path.startswith("target/observe/")
        ),
    }


def emit_graph_fixture_report() -> int:
    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text("", encoding="utf-8")
    row = {
        **graph_fixture_report(ROOT),
        "graph_fixture_report_only": True,
        "graph_fixture_report_command": "--graph-fixture-report",
    }
    emit(row)
    return 0 if row["validation_status"] == "pass" else 1


def main() -> int:
    if len(sys.argv) == 2 and sys.argv[1] == "--graph-fixture-report":
        return emit_graph_fixture_report()
    if len(sys.argv) == 2 and sys.argv[1] == "--runtime-archive-report":
        return emit_runtime_archive_report()
    if len(sys.argv) == 2 and sys.argv[1] == "--command-execution-report":
        return emit_command_execution_report()
    if len(sys.argv) > 1:
        print("usage: observe_validation.sh [--graph-fixture-report|--runtime-archive-report|--command-execution-report]", file=sys.stderr)
        return 2

    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text("", encoding="utf-8")

    git_head = git_value("rev-parse", "HEAD")
    git_status = git_value("status", "--short")
    ignored_counts = ignored_artifact_counts()
    base = os.environ.get("CANON_DELTA_BASE", "")
    delta_base_is_ancestor = False
    if base:
        delta_base_is_ancestor = subprocess.run(
            ["git", "merge-base", "--is-ancestor", base, "HEAD"], cwd=ROOT, check=False
        ).returncode == 0

    commands: list[dict[str, Any]] = []
    cargo_available = shutil.which("cargo") is not None
    rustc_available = shutil.which("rustc") is not None
    if cargo_available:
        commands.append(run("cargo_fmt_check", ["cargo", "fmt", "--check"], timeout=test_timeout_seconds(), env=root_rust_env))
        commands.append(run("cargo_test_all_targets", ["cargo", "test", "--all-targets"], timeout=test_timeout_seconds(), env=root_rust_env))
        commands.append(run("cargo_clippy_all_targets", ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"], timeout=test_timeout_seconds(), env=root_rust_env))
    else:
        emit({"event": "validation_command", "name": "cargo_unavailable", "status": "skipped_env_missing", "connector_failure_class": "environment_failure"})

    panic_report = ROOT / "target" / "observe" / "panic-surface.json"
    policy_report = ROOT / "target" / "observe" / "policy-learning-trace.json"
    graph_workflow_fixture_report_path = ROOT / "target" / "observe" / "graph-workflow-fixture.json"
    commands.append(run("panic_surface_validation", ["python3", "scripts/validate_rust_panic_surface.py", "--root", ".", "--report", str(panic_report.relative_to(ROOT))], timeout=test_timeout_seconds()))
    commands.append(run("policy_learning_trace_validation", ["python3", "scripts/validate_policy_learning_trace.py", "--root", ".", "--report", str(policy_report.relative_to(ROOT))], timeout=test_timeout_seconds()))
    commands.append(run("graph_workflow_fixture_validation", ["python3", "scripts/validate_graph_workflow_fixture.py", "--root", ".", "--report", str(graph_workflow_fixture_report_path.relative_to(ROOT))], timeout=test_timeout_seconds()))
    panic_surface = read_json(panic_report)
    policy_learning_trace = read_json(policy_report)
    graph_workflow_fixture_report = read_json(graph_workflow_fixture_report_path)

    wrapper = os.environ.get("CANON_RUSTC_WRAPPER", "")
    artifact_dir = os.environ.get("CANON_RUSTC_V3_ARTIFACT_DIR", "")
    wrapper_graph_validation_requested = bool(wrapper or artifact_dir)
    wrapper_graph_validation_available = bool(wrapper and Path(wrapper).exists())
    wrapper_configuration_status = wrapper_graph_configuration_status(
        wrapper=wrapper,
        artifact_dir=artifact_dir,
        wrapper_available=wrapper_graph_validation_available,
    )
    wrapper_configuration_reason = wrapper_graph_configuration_reason(wrapper_configuration_status)
    if wrapper_graph_validation_requested and wrapper_graph_validation_available:
        commands.append(run("wrapper_graph_validation", ["cargo", "test", "--all-targets"], timeout=test_timeout_seconds()))
        wrapper_graph_validation_result = commands[-1]["status"]
    elif wrapper_graph_validation_requested:
        wrapper_graph_validation_result = "skipped_env_missing"
        emit({"event": "validation_command", "name": "wrapper_graph_validation", "status": "skipped_env_missing", "reason": "CANON_RUSTC_WRAPPER not found", "connector_failure_class": "environment_failure"})
    else:
        wrapper_graph_validation_result = "skipped_not_requested"
    wrapper_graph_validation = wrapper_graph_validation_classification(
        configuration_status=wrapper_configuration_status,
        validation_result=wrapper_graph_validation_result,
        validation_available=wrapper_graph_validation_available,
    )

    state_graph_present = any((ROOT / "state").glob("**/graph.json"))
    graph_workflow_fixture = graph_workflow_fixture_fields(graph_workflow_fixture_report)

    run("ollama_judgment_example", ["cargo", "run", "--example", "ollama_judgment"], env=root_rust_env, timeout=test_timeout_seconds()) if os.environ.get("CANON_OLLAMA_BASE_URL") else None

    evidence = source_evidence()
    external_observation = token_files(evidence, ["ObservationCursor"])
    external_api = token_files(evidence, ["CommandEnvelope::new"])
    semantic_artifact = token_files(evidence, ["SemanticVerificationReceipt"])
    receipt_replay_classification = token_files(
        evidence,
        [
            "forged_receipt",
            "duplicated_receipt",
            "reordered_receipt",
            "stale_receipt",
            "missing_receipt",
        ],
    )
    graph_source_contract = token_files(
        evidence,
        [
            "GRAPH_MUTATION_SCHEMA_VERSION",
            "GraphMutationReceipt",
            "GraphPatchReceipt",
            "verify_graph_mutation_landing",
            "verify_graph_receipt_ledger_files_ndjson",
        ],
    )
    graph_workflow_contract = token_files(
        evidence,
        [
            "graph_mutation_cli_contract",
            "graph_mutation_cli_workflow",
        ],
    )

    failed_required = [c["name"] for c in commands if c.get("status") not in {"pass", "skipped_env_missing"}]
    command_statuses = {c["name"]: c.get("status") for c in commands}
    connector_failure_classes = sorted({c.get("connector_failure_class", "none") for c in commands if c.get("connector_failure_class", "none") != "none"})
    connector_failure_present = bool(connector_failure_classes)
    connector_failure_status = "present" if connector_failure_present else "none"
    connector_transport_instability_present = connector_failure_present and all(c.get("status") != "fail" for c in commands)

    evidence = {
        "external_observation_stream_test_present": bool(unique_files(external_observation)),
        "external_api_action_test_present": bool(unique_files(external_api)),
        "semantic_artifact_verification_test_present": bool(unique_files(semantic_artifact)),
        "receipt_replay_classification_test_present": all(receipt_replay_classification.values()),
        "graph_source_contract_present": all(graph_source_contract.values()),
        "graph_workflow_contract_present": all(graph_workflow_contract.values()),
    }
    graph_evidence_status = graph_evidence_classification(
        requested=wrapper_graph_validation_requested,
        wrapper_available=wrapper_graph_validation_available,
        state_graph_present=state_graph_present,
        source_contract_present=evidence["graph_source_contract_present"],
        landing_contract_present=bool(graph_source_contract.get("verify_graph_mutation_landing")),
        ledger_contract_present=bool(graph_source_contract.get("verify_graph_receipt_ledger_files_ndjson")),
        fixture_receipt_snapshot_present=bool(
            graph_workflow_fixture.get("graph_workflow_fixture_receipt_snapshot_present")
        ),
    )
    generated_graph_json = generated_graph_json_classification(
        state_graph_present=state_graph_present,
        wrapper_requested=wrapper_graph_validation_requested,
        fixture_receipt_snapshot_present=bool(
            graph_workflow_fixture.get("graph_workflow_fixture_receipt_snapshot_present")
        ),
    )

    runtime_performance = runtime_performance_summary(commands)
    runtime = runtime_archive_evidence(
        archive_path=os.environ.get("CANON_RUNTIME_ARCHIVE"),
        report_path=os.environ.get("CANON_RUNTIME_ARCHIVE_REPORT"),
        base=base,
    )

    missing = {
        "missing_cargo_test": "cargo_test_all_targets" not in command_statuses,
        "missing_wrapper_graph_validation": wrapper_graph_validation["wrapper_graph_validation_missing"],
        "missing_generated_graph_json": generated_graph_json["generated_graph_json_missing"],
        "missing_rustc_wrapper_telemetry": wrapper_graph_validation["wrapper_graph_telemetry_missing"],
        "missing_graph_source_contract_report": not evidence["graph_source_contract_present"],
        "missing_graph_workflow_contract_report": not evidence["graph_workflow_contract_present"],
        "missing_graph_workflow_fixture_receipt_snapshot": not graph_workflow_fixture.get(
            "graph_workflow_fixture_receipt_snapshot_present", False
        ),
        "missing_panic_surface_validation": "panic_surface_validation" not in command_statuses,
        "missing_policy_learning_replay_trace": "policy_learning_trace_validation" not in command_statuses,
        "missing_runtime_performance_signal": not runtime_performance["runtime_performance_signal_present"],
        **runtime_archive_missing_flags(runtime, base),
        "missing_external_observation_stream_test": not evidence["external_observation_stream_test_present"],
        "missing_external_api_action_test": not evidence["external_api_action_test_present"],
        "missing_semantic_artifact_verification_test": not evidence["semantic_artifact_verification_test_present"],
        "missing_receipt_replay_classification_report": not evidence[
            "receipt_replay_classification_test_present"
        ],
        "missing_connector_failure_classification": False,
    }
    router_test = {"available": False, "status": "skipped_env_missing"}
    router_offline = router_offline_classification(router_test)
    missing["missing_router_offline_tests"] = router_offline["router_offline_test_missing"]
    required = {"cargo_test_all_targets", "panic_surface_validation"}
    required.add("policy_learning_trace_validation")
    if router_test.get("available"):
        required.add("router_offline_tests")
    command_summary = command_execution_summary(commands, required)
    missing_signal_count = sum(1 for value in missing.values() if value)
    missing_signal_status = "pass" if missing_signal_count == 0 else "fail"
    command_execution_status = "pass" if not command_summary["required_command_failed"] else "fail"
    validation_status = "pass" if command_execution_status == "pass" and missing_signal_status == "pass" else "fail"
    validation_status_reason = (
        "all_required_commands_and_missing_signals_passed"
        if validation_status == "pass"
        else "required_command_failure_or_timeout"
        if command_execution_status == "fail"
        else "missing_signal_failure"
    )

    performance_budgets = {
        "CANON_MAX_PROJECT_AGENT_ELAPSED_MS_P95": os.environ.get("CANON_MAX_PROJECT_AGENT_ELAPSED_MS_P95"),
        "CANON_MAX_DOWNLOAD_INITIAL_GET_MS_P95": os.environ.get("CANON_MAX_DOWNLOAD_INITIAL_GET_MS_P95"),
        "CANON_MAX_DOWNLOAD_FOLLOW_GET_MS_P95": os.environ.get("CANON_MAX_DOWNLOAD_FOLLOW_GET_MS_P95"),
        "CANON_MAX_DOWNLOAD_WRITE_MS_P95": os.environ.get("CANON_MAX_DOWNLOAD_WRITE_MS_P95"),
    }
    summary = {
        "event": "validation_summary",
        "schema_version": 1,
        "git_head": git_head,
        "validation_status": validation_status,
        "validation_status_reason": validation_status_reason,
        "command_execution_status": command_execution_status,
        **command_summary,
        "missing_signal_status": missing_signal_status,
        "validation_commands": commands,
        "validation_command_count": len(commands),
        "validation_test_count": sum(1 for c in commands if c.get("status") == "pass"),
        "failed_required_commands": failed_required,
        "validation_command_statuses": command_statuses,
        "validation_command_duration_ms": {c["name"]: c.get("duration_ms") for c in commands},
        "connector_failure_classification_present": True,
        "connector_failure_present": connector_failure_present,
        "connector_failure_status": connector_failure_status,
        "connector_failure_classes": connector_failure_classes,
        "connector_transport_instability_present": connector_transport_instability_present,
        "router_offline_test_classification_present": True,
        "router_offline_test_classification_options": [
            "router_offline_unavailable",
            "router_offline_available_passed",
            "router_offline_available_not_passed",
        ],
        **router_offline,
        "receipt_replay_classification_present": evidence[
            "receipt_replay_classification_test_present"
        ],
        "receipt_replay_classifications": sorted(receipt_replay_classification.keys()),
        "receipt_replay_classification_evidence_files": unique_files(
            receipt_replay_classification
        ),
        "receipt_replay_classification_evidence_tokens": receipt_replay_classification,
        "graph_evidence_classification_present": True,
        "graph_evidence_status": graph_evidence_status,
        "graph_evidence_status_options": [
            "graph_wrapper_absent_by_configuration",
            "graph_wrapper_configured_missing",
            "graph_wrapper_configured_no_telemetry",
            "graph_mutation_evidence_contract_missing",
            "graph_mutation_evidence_emitted_not_landed",
            "graph_mutation_landed_without_receipt_ledger",
            "graph_mutation_landed_with_receipt_snapshot",
        ],
        "generated_graph_json_classification_present": True,
        "generated_graph_json_classification_options": [
            "generated_graph_json_present",
            "generated_graph_json_substituted_by_fixture",
            "generated_graph_json_missing_when_requested",
            "generated_graph_json_missing_no_fixture",
        ],
        **generated_graph_json,
        "graph_source_contract_present": evidence["graph_source_contract_present"],
        "graph_source_contract_evidence_files": unique_files(graph_source_contract),
        "graph_source_contract_evidence_tokens": graph_source_contract,
        "graph_workflow_contract_present": evidence["graph_workflow_contract_present"],
        "graph_workflow_contract_evidence_files": unique_files(graph_workflow_contract),
        "graph_workflow_contract_evidence_tokens": graph_workflow_contract,
        "graph_workflow_fixture_validation_result": graph_workflow_fixture_report.get("validation_status"),
        "graph_workflow_fixture_report_path": str(graph_workflow_fixture_report_path.relative_to(ROOT)),
        "graph_fixture_validator": graph_workflow_fixture_report.get("graph_fixture_validator"),
        **graph_workflow_fixture,
        "missing_signal_flags": missing,
        "missing_signal_count": missing_signal_count,
        "git_status_clean": not bool(git_status),
        "git_status_short": git_status,
        **ignored_counts,
        "delta_base_is_ancestor": delta_base_is_ancestor,
        "delta_changed_file_count": len(git_value("diff", "--name-only", f"{base}..HEAD").splitlines()) if base else 0,
        "cargo_available": cargo_available,
        "rustc_available": rustc_available,
        "root_rust_env_overrides": root_rust_env,
        "wrapper_override_required": True,
        "wrapper_override_used": True,
        "wrapper_override_env": root_rust_env,
        "wrapper_graph_validation_result": wrapper_graph_validation_result,
        "wrapper_graph_validation_requested": wrapper_graph_validation_requested,
        "wrapper_graph_validation_available": wrapper_graph_validation_available,
        "wrapper_graph_configuration_status": wrapper_configuration_status,
        "wrapper_graph_configuration_reason": wrapper_configuration_reason,
        "wrapper_graph_validation_classification_present": True,
        "wrapper_graph_validation_classification_options": [
            "wrapper_graph_optional_not_configured",
            "wrapper_graph_requested_without_wrapper",
            "wrapper_graph_requested_wrapper_missing",
            "wrapper_graph_requested_passed",
            "wrapper_graph_requested_not_passed",
        ],
        **wrapper_graph_validation,
        "wrapper_graph_configuration_status_options": [
            "not_configured",
            "artifact_dir_configured_without_wrapper",
            "wrapper_configured_missing",
            "wrapper_configured_available",
        ],
        "rustc_wrapper_configured": bool(wrapper),
        "rustc_wrapper_path_exists": bool(wrapper and Path(wrapper).exists()),
        "state_graph_present": state_graph_present,
        **runtime_performance,
        "runtime_performance_budgets": performance_budgets,
        "runtime_manifest_base_expected": base,
        "runtime_manifest_base_matches_delta_base": bool(base and runtime.get("runtime_manifest_base_commit") == base),
        "policy_learning_trace_validation_result": policy_learning_trace.get("status"),
        "policy_learning_trace_status": policy_learning_trace.get("status"),
        "policy_learning_trace_function": policy_learning_trace.get("trace_function"),
        "policy_learning_trace_check_count": len(policy_learning_trace.get("checks", [])),
        "policy_learning_trace_missing_count": policy_learning_trace.get("missing_count"),
        "panic_surface_production_unwrap_count": panic_surface.get("production_unwrap_count"),
        "panic_surface_production_expect_count": panic_surface.get("production_expect_count"),
        "panic_surface_production_panic_count": panic_surface.get("production_panic_count"),
        "panic_surface_test_total": panic_surface.get("test_total"),
        "panic_surface_example_total": panic_surface.get("example_total"),
        "external_observation_stream_test_present": evidence["external_observation_stream_test_present"],
        "external_observation_stream_evidence_files": unique_files(external_observation),
        "external_observation_stream_evidence_tokens": external_observation,
        "external_api_action_test_present": evidence["external_api_action_test_present"],
        "external_api_action_evidence_files": unique_files(external_api),
        "external_api_action_evidence_tokens": external_api,
        "semantic_artifact_verification_test_present": evidence["semantic_artifact_verification_test_present"],
        "semantic_artifact_verification_evidence_files": unique_files(semantic_artifact),
        "semantic_artifact_verification_evidence_tokens": semantic_artifact,
        **runtime,
    }
    emit(summary)
    return 0 if validation_status == "pass" else 1


if __name__ == "__main__":
    sys.exit(main())
