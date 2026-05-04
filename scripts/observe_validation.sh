#!/usr/bin/env bash
# Emit one current-head validation report without requiring unavailable tools.
set -euo pipefail

python3 - <<'PY'
from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import tarfile
import time
from pathlib import Path
from typing import Any

ROOT = Path.cwd()
REPORT = Path(os.environ.get("CANON_OBSERVE_REPORT", "target/observe/validation-report.ndjson"))
OUT = REPORT.parent / "command-output"
REPORT.parent.mkdir(parents=True, exist_ok=True)
OUT.mkdir(parents=True, exist_ok=True)
REPORT.write_text("", encoding="utf-8")

records: list[dict[str, Any]] = []


def ms() -> int:
    return time.time_ns() // 1_000_000


def short(text: str, limit: int = 4000) -> str:
    text = text.replace("\x00", "")
    return text if len(text) <= limit else text[:limit] + "...[truncated]"


def emit(record: dict[str, Any]) -> None:
    row = {"ts_ms": ms(), **record}
    records.append(row)
    line = json.dumps(row, sort_keys=True, separators=(",", ":"))
    REPORT.open("a", encoding="utf-8").write(line + "\n")
    print(line)


def save(cmd: list[str], stdout: str, stderr: str) -> dict[str, str]:
    safe = re.sub(r"[^A-Za-z0-9_.-]+", "_", "_".join(cmd))[:80] or "cmd"
    digest = hashlib.sha256(json.dumps(cmd, separators=(",", ":")).encode()).hexdigest()[:12]
    base = OUT / f"{safe}.{digest}"
    stdout_path, stderr_path = f"{base}.stdout.txt", f"{base}.stderr.txt"
    Path(stdout_path).write_text(stdout, encoding="utf-8")
    Path(stderr_path).write_text(stderr, encoding="utf-8")
    return {"stdout_path": stdout_path, "stderr_path": stderr_path}


def run(name: str, cmd: list[str], cwd: Path = ROOT, timeout: int = 120) -> dict[str, Any]:
    if shutil.which(cmd[0]) is None:
        result = {"name": name, "cmd": cmd, "status": "unavailable", "available": False,
                  "returncode": None, "duration_ms": 0, "stdout": "", "stderr": f"{cmd[0]} not found in PATH"}
    else:
        start = time.monotonic()
        try:
            done = subprocess.run(cmd, cwd=cwd, text=True, stdout=subprocess.PIPE,
                                  stderr=subprocess.PIPE, timeout=timeout, check=False)
            result = {"name": name, "cmd": cmd, "status": "pass" if done.returncode == 0 else "fail",
                      "available": True, "returncode": done.returncode,
                      "duration_ms": int((time.monotonic() - start) * 1000),
                      "stdout": short(done.stdout), "stderr": short(done.stderr)}
        except subprocess.TimeoutExpired as exc:
            result = {"name": name, "cmd": cmd, "status": "timeout", "available": True,
                      "returncode": None, "duration_ms": int((time.monotonic() - start) * 1000),
                      "stdout": short(exc.stdout or ""), "stderr": short(exc.stderr or f"timeout after {timeout}s")}
    result["output_path"] = save(cmd, result["stdout"], result["stderr"])
    emit({"event": "validation_command", "result": result})
    return result


def scalar(cmd: list[str]) -> str | None:
    try:
        return subprocess.check_output(cmd, cwd=ROOT, text=True).strip()
    except Exception:
        return None


def wrapper() -> dict[str, Any]:
    cfg = ROOT / ".cargo" / "config.toml"
    text = cfg.read_text(encoding="utf-8") if cfg.exists() else ""
    match = re.search(r'^\s*rustc-wrapper\s*=\s*"([^"]+)"', text, re.MULTILINE)
    path = match.group(1) if match else None
    return {"config_present": cfg.exists(), "rustc_wrapper_configured": path is not None,
            "rustc_wrapper_path": path, "rustc_wrapper_path_exists": bool(path and Path(path).exists())}


def graph() -> dict[str, Any]:
    paths = sorted((ROOT / "state" / "rustc").glob("*/graph.json"))
    if not paths:
        return {"state_graph_present": False, "graph_paths": []}
    try:
        data = json.loads(paths[0].read_text(encoding="utf-8"))
        nodes, edges = data.get("nodes", []), data.get("edges", [])
        fns = [n for n in nodes if isinstance(n, dict) and str(n.get("kind", "")).lower() in {"fn", "function"}]
        intents = [n for n in fns if n.get("intent_class")]
        return {"state_graph_present": True, "graph_paths": [str(p) for p in paths],
                "graph_parse_status": "pass", "graph_node_count": len(nodes), "graph_edge_count": len(edges),
                "graph_function_node_count": len(fns), "graph_intent_node_count": len(intents),
                "graph_intent_coverage": len(intents) / len(fns) if fns else None}
    except Exception as exc:
        return {"state_graph_present": True, "graph_paths": [str(p) for p in paths],
                "graph_parse_status": "fail", "graph_parse_error": str(exc)}


def runtime() -> dict[str, Any]:
    candidates = [Path(p) for p in [os.environ.get("CANON_RUNTIME_ARCHIVE", ""), str(ROOT / "ai-runtime.tar.gz"),
                                    str(ROOT.parent / "ai-runtime.tar.gz"), "/mnt/data/ai-runtime.tar.gz"] if p]
    archive = next((p for p in candidates if p.exists()), None)
    if not archive:
        return {"runtime_archive_present": False, "runtime_archive_candidates": [str(p) for p in candidates]}
    metrics: dict[str, Any] = {"runtime_archive_present": True, "runtime_archive_path": str(archive),
                               "runtime_archive_member_count": 0, "runtime_archive_log_counts": {},
                               "runtime_archive_download_counts": {}, "runtime_archive_conversation_snapshots": 0,
                               "runtime_archive_cache_files": 0}
    try:
        with tarfile.open(archive, "r:gz") as tf:
            for member in [m for m in tf.getmembers() if m.isfile()]:
                name = member.name
                metrics["runtime_archive_member_count"] += 1
                metrics["runtime_archive_cache_files"] += int("cache" in name.lower())
                metrics["runtime_archive_conversation_snapshots"] += int(name.endswith(".conversation.json"))
                if name.endswith((".ndjson", ".jsonl")):
                    fh = tf.extractfile(member)
                    count = sum(1 for line in fh or [] if line.strip())
                    key = "runtime_archive_download_counts" if "download" in name.lower() else "runtime_archive_log_counts"
                    metrics[key][name] = count
        metrics["runtime_archive_parse_status"] = "pass"
    except Exception as exc:
        metrics.update({"runtime_archive_parse_status": "fail", "runtime_archive_parse_error": str(exc)})
    return metrics


def cargo_tests(result: dict[str, Any]) -> int | None:
    text = f"{result.get('stdout', '')}\n{result.get('stderr', '')}"
    counts = [int(x) for x in re.findall(r"running\s+(\d+)\s+tests?", text)]
    return sum(counts) if counts else None


def router_tests(result: dict[str, Any]) -> int:
    text = f"{result.get('stdout', '')}\n{result.get('stderr', '')}"
    node_counts = [int(x) for x in re.findall(r"tests\s+(\d+)", text)]
    return max(node_counts) if node_counts else text.count(".")


head = scalar(["git", "rev-parse", "HEAD"])
git_status = subprocess.run(["git", "status", "--short"], cwd=ROOT, text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
w, g, r = wrapper(), graph(), runtime()

emit({"event": "validation_start", "schema_version": 2, "repo": str(ROOT), "report_path": str(REPORT), "git_head": head})
emit({"event": "git_state", "git_head": head, "git_status_clean": git_status.returncode == 0 and not git_status.stdout.strip(),
      "git_status_short": git_status.stdout.splitlines()})
emit({"event": "toolchain", "cargo_available": shutil.which("cargo") is not None,
      "rustc_available": shutil.which("rustc") is not None, "python3_available": shutil.which("python3") is not None, **w})
emit({"event": "graph_metrics", **g})
emit({"event": "runtime_archive_metrics", **r})

commands = [
    run("git_diff_check", ["git", "diff", "--check"], timeout=30),
    run("router_offline_tests", ["bash", "run_tests.sh"], cwd=ROOT / "ai-chromium" / "router-server", timeout=180),
    run("cargo_fmt_check", ["cargo", "fmt", "--check"], timeout=180),
    run("cargo_test_all_targets", ["cargo", "test", "--all-targets"], timeout=600),
    run("cargo_clippy_all_targets", ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"], timeout=600),
]
if os.environ.get("CANON_OLLAMA_BASE_URL") and os.environ.get("CANON_OLLAMA_MODEL"):
    commands.append(run("ollama_judgment_example", ["cargo", "run", "--example", "ollama_judgment"], timeout=600))
else:
    skipped = {"name": "ollama_judgment_example", "cmd": ["cargo", "run", "--example", "ollama_judgment"],
               "status": "skipped_env_missing", "available": shutil.which("cargo") is not None,
               "returncode": None, "duration_ms": 0, "stdout": "", "stderr": "missing CANON_OLLAMA_BASE_URL or CANON_OLLAMA_MODEL"}
    skipped["output_path"] = save(skipped["cmd"], "", skipped["stderr"])
    commands.append(skipped)
    emit({"event": "validation_command", "result": skipped})

cargo_test = next(c for c in commands if c["name"] == "cargo_test_all_targets")
router_test = next(c for c in commands if c["name"] == "router_offline_tests")
download_total = sum(map(int, r.get("runtime_archive_download_counts", {}).values())) if r.get("runtime_archive_present") else 0
log_total = sum(map(int, r.get("runtime_archive_log_counts", {}).values())) if r.get("runtime_archive_present") else 0
missing = {
    "missing_cargo_fmt": next(c for c in commands if c["name"] == "cargo_fmt_check")["status"] != "pass",
    "missing_cargo_test": cargo_test["status"] != "pass",
    "missing_cargo_run_ollama_judgment": next(c for c in commands if c["name"] == "ollama_judgment_example")["status"] != "pass",
    "missing_clippy": next(c for c in commands if c["name"] == "cargo_clippy_all_targets")["status"] != "pass",
    "missing_generated_graph_json": not bool(g.get("state_graph_present")),
    "missing_rustc_wrapper_telemetry": not (w["rustc_wrapper_configured"] and w["rustc_wrapper_path_exists"] and g.get("state_graph_present")),
    "missing_runtime_download_history": download_total == 0,
    "missing_conversation_snapshot": int(r.get("runtime_archive_conversation_snapshots", 0) or 0) == 0,
    "missing_artifact_apply_worktree": not (ROOT / ".repo-agent-runtime" / "apply-worktrees").exists(),
    "missing_external_observation_stream_test": True,
    "missing_external_api_action_test": True,
    "missing_semantic_artifact_verification_test": True,
    "missing_policy_learning_replay_trace": True,
}
required = ["git_diff_check", "router_offline_tests"]
failed_required = [c["name"] for c in commands if c["name"] in required and c["status"] != "pass"]
status = "fail" if failed_required else ("partial" if any(missing.values()) else "pass")
emit({
    "event": "validation_summary", "validation_status": status, "git_head": head,
    "git_status_clean": git_status.returncode == 0 and not git_status.stdout.strip(),
    "validation_command_count": len(commands), "validation_commands": [{"name": c["name"], "cmd": c["cmd"], "status": c["status"]} for c in commands],
    "validation_test_count": (cargo_tests(cargo_test) or 0) + router_tests(router_test),
    "router_test_count": router_tests(router_test), "cargo_test_count_when_available": cargo_tests(cargo_test),
    "failed_required_commands": failed_required, "cargo_available": shutil.which("cargo") is not None,
    "cargo_fmt_check_result": next(c for c in commands if c["name"] == "cargo_fmt_check")["status"],
    "cargo_test_result": cargo_test["status"], "ollama_example_result_when_available": next(c for c in commands if c["name"] == "ollama_judgment_example")["status"],
    "clippy_result_when_available": next(c for c in commands if c["name"] == "cargo_clippy_all_targets")["status"],
    "rustc_wrapper_configured": w["rustc_wrapper_configured"], "rustc_wrapper_path_exists": w["rustc_wrapper_path_exists"],
    "state_graph_present": g.get("state_graph_present", False), "graph_node_count_when_present": g.get("graph_node_count"),
    "graph_edge_count_when_present": g.get("graph_edge_count"), "graph_intent_coverage_when_present": g.get("graph_intent_coverage"),
    "runtime_archive_present": r.get("runtime_archive_present", False), "runtime_archive_log_total": log_total,
    "runtime_archive_download_total": download_total, "runtime_archive_conversation_snapshots": r.get("runtime_archive_conversation_snapshots", 0),
    "missing_signal_flags": missing, "missing_signal_count": sum(1 for v in missing.values() if v),
})
PY