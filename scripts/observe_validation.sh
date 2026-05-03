#!/usr/bin/env bash
# Emit deterministic observe-stage validation evidence as NDJSON.
#
# Usage:
#   bash scripts/observe_validation.sh
#   CANON_OBSERVE_REPORT=target/observe/custom.ndjson bash scripts/observe_validation.sh
#   CANON_RUNTIME_ARCHIVE=/path/to/ai-runtime.tar.gz bash scripts/observe_validation.sh
#
# The script is intentionally evidence-only. It does not mutate source files,
# does not require cargo or Ollama to be installed, and records unavailable
# validation surfaces explicitly instead of silently skipping them.
set -euo pipefail

python3 - "$@" <<'PY'
from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import time
from pathlib import Path
from typing import Any


REPO = Path.cwd()
REPORT = Path(os.environ.get("CANON_OBSERVE_REPORT", "target/observe/validation-report.ndjson"))
REPORT.parent.mkdir(parents=True, exist_ok=True)

records: list[dict[str, Any]] = []


def now_ms() -> int:
    return time.time_ns() // 1_000_000


def clean_text(value: str, limit: int = 4000) -> str:
    value = value.replace("\x00", "")
    if len(value) > limit:
        return value[:limit] + "...[truncated]"
    return value


def emit(record: dict[str, Any]) -> None:
    full_record = {"ts_ms": now_ms(), **record}
    records.append(full_record)
    line = json.dumps(full_record, sort_keys=True, separators=(",", ":"))
    with REPORT.open("a", encoding="utf-8") as fh:
        fh.write(line + "\n")
    print(line)


def run(cmd: list[str], timeout_s: int = 120) -> dict[str, Any]:
    started = time.monotonic()
    if shutil.which(cmd[0]) is None:
        return {
            "cmd": cmd,
            "available": False,
            "status": "unavailable",
            "returncode": None,
            "duration_ms": 0,
            "stdout": "",
            "stderr": f"{cmd[0]} not found in PATH",
        }
    try:
        completed = subprocess.run(
            cmd,
            cwd=REPO,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=timeout_s,
            check=False,
        )
        return {
            "cmd": cmd,
            "available": True,
            "status": "pass" if completed.returncode == 0 else "fail",
            "returncode": completed.returncode,
            "duration_ms": int((time.monotonic() - started) * 1000),
            "stdout": clean_text(completed.stdout),
            "stderr": clean_text(completed.stderr),
        }
    except subprocess.TimeoutExpired as exc:
        return {
            "cmd": cmd,
            "available": True,
            "status": "timeout",
            "returncode": None,
            "duration_ms": int((time.monotonic() - started) * 1000),
            "stdout": clean_text(exc.stdout or ""),
            "stderr": clean_text(exc.stderr or f"timeout after {timeout_s}s"),
        }


def git_scalar(args: list[str]) -> str | None:
    result = run(["git", *args], timeout_s=30)
    if result["status"] != "pass":
        return None
    return str(result["stdout"]).strip()


def cargo_result(kind: str, args: list[str], timeout_s: int) -> dict[str, Any]:
    result = run(args, timeout_s=timeout_s)
    emit({"event": kind, "result": result})
    return result


def count_cargo_tests(output: str) -> int | None:
    counts = [int(match.group(1)) for match in re.finditer(r"running\s+(\d+)\s+tests?", output)]
    if counts:
        return sum(counts)
    return None


def read_wrapper_config() -> dict[str, Any]:
    config = REPO / ".cargo" / "config.toml"
    text = config.read_text(encoding="utf-8") if config.exists() else ""
    match = re.search(r"^\s*rustc-wrapper\s*=\s*\"([^\"]+)\"", text, flags=re.MULTILINE)
    wrapper = match.group(1) if match else None
    path_exists = bool(wrapper and Path(wrapper).exists())
    return {
        "config_present": config.exists(),
        "rustc_wrapper_configured": wrapper is not None,
        "rustc_wrapper_path": wrapper,
        "rustc_wrapper_path_exists": path_exists,
    }


def graph_candidates() -> list[Path]:
    candidates: list[Path] = []
    for root in [REPO / "state" / "rustc", REPO.parent / "state" / "rustc"]:
        if root.exists():
            candidates.extend(sorted(root.glob("*/graph.json")))
    return candidates


def graph_metrics() -> dict[str, Any]:
    graphs = graph_candidates()
    if not graphs:
        return {"state_graph_present": False, "graph_paths": []}

    path = graphs[0]
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001 - report evidence, do not crash.
        return {
            "state_graph_present": True,
            "graph_paths": [str(p) for p in graphs],
            "graph_parse_status": "fail",
            "graph_parse_error": str(exc),
        }

    nodes = data.get("nodes", []) if isinstance(data, dict) else []
    edges = data.get("edges", []) if isinstance(data, dict) else []
    fn_nodes = [n for n in nodes if isinstance(n, dict) and str(n.get("kind", "")).lower() in {"fn", "function"}]
    intent_nodes = [n for n in fn_nodes if isinstance(n, dict) and n.get("intent_class")]
    coverage = (len(intent_nodes) / len(fn_nodes)) if fn_nodes else None
    return {
        "state_graph_present": True,
        "graph_paths": [str(p) for p in graphs],
        "graph_parse_status": "pass",
        "graph_node_count": len(nodes) if isinstance(nodes, list) else None,
        "graph_edge_count": len(edges) if isinstance(edges, list) else None,
        "graph_function_node_count": len(fn_nodes),
        "graph_intent_node_count": len(intent_nodes),
        "graph_intent_coverage": coverage,
    }


def runtime_archive_candidates() -> list[Path]:
    paths: list[Path] = []
    env_path = os.environ.get("CANON_RUNTIME_ARCHIVE")
    if env_path:
        paths.append(Path(env_path))
    paths.extend([
        REPO / "ai-runtime.tar.gz",
        REPO.parent / "ai-runtime.tar.gz",
        Path("/mnt/data/ai-runtime.tar.gz"),
    ])
    deduped: list[Path] = []
    seen: set[str] = set()
    for path in paths:
        key = str(path)
        if key not in seen:
            seen.add(key)
            deduped.append(path)
    return deduped


def ndjson_line_count_from_tar(tf: tarfile.TarFile, member: tarfile.TarInfo) -> int:
    fh = tf.extractfile(member)
    if fh is None:
        return 0
    with fh:
        return sum(1 for line in fh if line.strip())


def runtime_metrics() -> dict[str, Any]:
    archive = next((path for path in runtime_archive_candidates() if path.exists()), None)
    if archive is None:
        return {"runtime_archive_present": False, "runtime_archive_candidates": [str(p) for p in runtime_archive_candidates()]}

    metrics: dict[str, Any] = {
        "runtime_archive_present": True,
        "runtime_archive_path": str(archive),
        "runtime_archive_member_count": 0,
        "runtime_archive_log_counts": {},
        "runtime_archive_download_counts": {},
        "runtime_archive_conversation_snapshots": 0,
        "runtime_archive_cache_files": 0,
    }
    try:
        with tarfile.open(archive, "r:gz") as tf:
            members = [member for member in tf.getmembers() if member.isfile()]
            metrics["runtime_archive_member_count"] = len(members)
            for member in members:
                name = member.name
                if ".chatgpt-agent-cache" in name or "cache" in name.lower():
                    metrics["runtime_archive_cache_files"] += 1
                if name.endswith(".conversation.json"):
                    metrics["runtime_archive_conversation_snapshots"] += 1
                if name.endswith(".ndjson") or name.endswith(".jsonl"):
                    line_count = ndjson_line_count_from_tar(tf, member)
                    if "download" in name.lower():
                        metrics["runtime_archive_download_counts"][name] = line_count
                    else:
                        metrics["runtime_archive_log_counts"][name] = line_count
    except Exception as exc:  # noqa: BLE001 - report evidence, do not crash.
        metrics["runtime_archive_parse_status"] = "fail"
        metrics["runtime_archive_parse_error"] = str(exc)
        return metrics
    metrics["runtime_archive_parse_status"] = "pass"
    return metrics


def bool_missing(status: str) -> bool:
    return status != "pass"


REPORT.write_text("", encoding="utf-8")

git_head = git_scalar(["rev-parse", "HEAD"])
git_status = run(["git", "status", "--short"], timeout_s=30)
wrapper = read_wrapper_config()
graph = graph_metrics()
runtime = runtime_metrics()

emit({
    "event": "validation_start",
    "schema_version": 1,
    "repo": str(REPO),
    "report_path": str(REPORT),
    "git_head": git_head,
})
emit({
    "event": "git_state",
    "git_head": git_head,
    "git_status_clean": git_status["status"] == "pass" and str(git_status["stdout"]).strip() == "",
    "git_status_short": str(git_status["stdout"]).splitlines(),
    "git_status_result": git_status,
})
emit({
    "event": "toolchain",
    "cargo_available": shutil.which("cargo") is not None,
    "rustc_available": shutil.which("rustc") is not None,
    "python3_available": shutil.which("python3") is not None,
    **wrapper,
})
emit({"event": "graph_metrics", **graph})
emit({"event": "runtime_archive_metrics", **runtime})

cargo_fmt = cargo_result("cargo_fmt_check", ["cargo", "fmt", "--check"], timeout_s=180)
cargo_test = cargo_result("cargo_test_all_targets", ["cargo", "test", "--all-targets"], timeout_s=600)
cargo_clippy = cargo_result("cargo_clippy_all_targets", ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"], timeout_s=600)

ollama_env_present = bool(os.environ.get("CANON_OLLAMA_BASE_URL") and os.environ.get("CANON_OLLAMA_MODEL"))
if ollama_env_present:
    ollama_example = cargo_result("ollama_judgment_example", ["cargo", "run", "--example", "ollama_judgment"], timeout_s=600)
else:
    ollama_example = {
        "cmd": ["cargo", "run", "--example", "ollama_judgment"],
        "available": shutil.which("cargo") is not None,
        "status": "skipped_env_missing",
        "returncode": None,
        "duration_ms": 0,
        "stdout": "",
        "stderr": "CANON_OLLAMA_BASE_URL and CANON_OLLAMA_MODEL are required for this optional live path",
    }
    emit({"event": "ollama_judgment_example", "result": ollama_example})

test_output = f"{cargo_test.get('stdout', '')}\n{cargo_test.get('stderr', '')}"
test_count = count_cargo_tests(test_output)
download_total = sum(int(v) for v in runtime.get("runtime_archive_download_counts", {}).values()) if runtime.get("runtime_archive_present") else 0
log_total = sum(int(v) for v in runtime.get("runtime_archive_log_counts", {}).values()) if runtime.get("runtime_archive_present") else 0

missing = {
    "missing_cargo_fmt": bool_missing(cargo_fmt["status"]),
    "missing_cargo_test": bool_missing(cargo_test["status"]),
    "missing_cargo_run_ollama_judgment": ollama_example["status"] != "pass",
    "missing_clippy": bool_missing(cargo_clippy["status"]),
    "missing_generated_graph_json": not bool(graph.get("state_graph_present")),
    "missing_rustc_wrapper_telemetry": not (wrapper["rustc_wrapper_configured"] and wrapper["rustc_wrapper_path_exists"] and graph.get("state_graph_present")),
    "missing_runtime_download_history": download_total == 0,
    "missing_conversation_snapshot": int(runtime.get("runtime_archive_conversation_snapshots", 0) or 0) == 0,
    "missing_artifact_apply_worktree": not (REPO / ".repo-agent-runtime" / "apply-worktrees").exists(),
    "missing_external_observation_stream_test": True,
    "missing_external_api_action_test": True,
    "missing_semantic_artifact_verification_test": True,
    "missing_policy_learning_replay_trace": True,
}

summary = {
    "event": "validation_summary",
    "git_head": git_head,
    "git_status_clean": git_status["status"] == "pass" and str(git_status["stdout"]).strip() == "",
    "cargo_available": shutil.which("cargo") is not None,
    "cargo_fmt_check_result": cargo_fmt["status"],
    "cargo_test_result": cargo_test["status"],
    "cargo_test_count_when_available": test_count,
    "ollama_example_result_when_available": ollama_example["status"],
    "clippy_result_when_available": cargo_clippy["status"],
    "rustc_wrapper_configured": wrapper["rustc_wrapper_configured"],
    "rustc_wrapper_path_exists": wrapper["rustc_wrapper_path_exists"],
    "state_graph_present": graph.get("state_graph_present", False),
    "graph_node_count_when_present": graph.get("graph_node_count"),
    "graph_edge_count_when_present": graph.get("graph_edge_count"),
    "graph_intent_coverage_when_present": graph.get("graph_intent_coverage"),
    "runtime_archive_present": runtime.get("runtime_archive_present", False),
    "runtime_archive_log_total": log_total,
    "runtime_archive_download_total": download_total,
    "runtime_archive_conversation_snapshots": runtime.get("runtime_archive_conversation_snapshots", 0),
    "missing_signal_flags": missing,
    "missing_signal_count": sum(1 for value in missing.values() if value),
}
emit(summary)
PY