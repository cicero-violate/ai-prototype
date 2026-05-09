#!/usr/bin/env python3
"""Emit compact validation evidence as NDJSON.

The script is intentionally deterministic and report-first: generated evidence is
written to target/observe/validation-report.ndjson by default, while expensive
commands keep their full logs under target/validation-logs. Root Rust validation
clears wrapper variables so baseline correctness is independent of optional graph
capture tooling.
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
            names = tf.getnames()
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


def main() -> int:
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
    commands.append(run("panic_surface_validation", ["python3", "scripts/validate_rust_panic_surface.py", "--root", ".", "--report", str(panic_report.relative_to(ROOT))], timeout=test_timeout_seconds()))
    commands.append(run("policy_learning_trace_validation", ["python3", "scripts/validate_policy_learning_trace.py", "--root", ".", "--report", str(policy_report.relative_to(ROOT))], timeout=test_timeout_seconds()))
    panic_surface = read_json(panic_report)
    policy_learning_trace = read_json(policy_report)

    wrapper = os.environ.get("CANON_RUSTC_WRAPPER", "")
    artifact_dir = os.environ.get("CANON_RUSTC_V3_ARTIFACT_DIR", "")
    wrapper_graph_validation_requested = bool(wrapper or artifact_dir)
    wrapper_graph_validation_available = bool(wrapper and Path(wrapper).exists())
    if wrapper_graph_validation_requested and wrapper_graph_validation_available:
        commands.append(run("wrapper_graph_validation", ["cargo", "test", "--all-targets"], timeout=test_timeout_seconds()))
        wrapper_graph_validation_result = commands[-1]["status"]
    elif wrapper_graph_validation_requested:
        wrapper_graph_validation_result = "skipped_env_missing"
        emit({"event": "validation_command", "name": "wrapper_graph_validation", "status": "skipped_env_missing", "reason": "CANON_RUSTC_WRAPPER not found", "connector_failure_class": "environment_failure"})
    else:
        wrapper_graph_validation_result = "skipped_not_requested"

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
    }

    missing = {
        "missing_cargo_test": "cargo_test_all_targets" not in command_statuses,
        "missing_wrapper_graph_validation": wrapper_graph_validation_result == "skipped_not_requested",
        "missing_generated_graph_json": not any((ROOT / "state").glob("**/graph.json")),
        "missing_rustc_wrapper_telemetry": not wrapper_graph_validation_available,
        "missing_panic_surface_validation": "panic_surface_validation" not in command_statuses,
        "missing_policy_learning_replay_trace": "policy_learning_trace_validation" not in command_statuses,
        "missing_runtime_performance_signal": True,
        "missing_runtime_manifest_base_match": not bool(base),
        "missing_runtime_download_index": True,
        "missing_runtime_prior_state": True,
        "missing_runtime_conversation_ledger": True,
        "missing_runtime_inspection_contract": False,
        "missing_external_observation_stream_test": not evidence["external_observation_stream_test_present"],
        "missing_external_api_action_test": not evidence["external_api_action_test_present"],
        "missing_semantic_artifact_verification_test": not evidence["semantic_artifact_verification_test_present"],
        "missing_receipt_replay_classification_report": not evidence[
            "receipt_replay_classification_test_present"
        ],
        "missing_router_offline_tests": True,
        "missing_connector_failure_classification": False,
    }
    router_test = {"available": False, "status": "skipped_env_missing"}
    required = {"cargo_test_all_targets", "panic_surface_validation"}
    required.add("policy_learning_trace_validation")
    if router_test.get("available"):
        required.add("router_offline_tests")
    validation_status = "pass" if not failed_required else "fail"

    runtime = inspect_runtime_archive(os.environ.get("CANON_RUNTIME_ARCHIVE"))
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
        "receipt_replay_classification_present": evidence[
            "receipt_replay_classification_test_present"
        ],
        "receipt_replay_classifications": sorted(receipt_replay_classification.keys()),
        "receipt_replay_classification_evidence_files": unique_files(
            receipt_replay_classification
        ),
        "receipt_replay_classification_evidence_tokens": receipt_replay_classification,
        "missing_signal_flags": missing,
        "missing_signal_count": sum(1 for value in missing.values() if value),
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
        "rustc_wrapper_configured": bool(wrapper),
        "rustc_wrapper_path_exists": bool(wrapper and Path(wrapper).exists()),
        "state_graph_present": any((ROOT / "state").glob("**/graph.json")),
        "runtime_performance_metrics": {},
        "runtime_performance_signal_present": False,
        "runtime_performance_budget_status": "missing_signal",
        "runtime_performance_budget_failures": [],
        "runtime_performance_budgets": performance_budgets,
        "project_agent_elapsed_ms_median": None,
        "project_agent_elapsed_ms_p95": None,
        "download_initial_get_ms_median": None,
        "download_follow_get_ms_median": None,
        "download_write_ms_median": None,
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
