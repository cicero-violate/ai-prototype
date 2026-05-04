#!/usr/bin/env bash
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
from math import ceil
from pathlib import Path
from typing import Any

ROOT = Path.cwd()
REPORT = Path(os.environ.get("CANON_OBSERVE_REPORT", "target/observe/validation-report.ndjson"))
OUT = REPORT.parent / "command-output"
PANIC_SURFACE_REPORT = REPORT.parent / "panic-surface.json"
POLICY_LEARNING_TRACE_REPORT = REPORT.parent / "policy-learning-trace.json"
REPORT.parent.mkdir(parents=True, exist_ok=True)
OUT.mkdir(parents=True, exist_ok=True)
REPORT.write_text("", encoding="utf-8")


def now_ms() -> int:
    return time.time_ns() // 1_000_000


def short(text: str, limit: int = 4000) -> str:
    text = (text or "").replace("\x00", "")
    return text if len(text) <= limit else text[:limit] + "...[truncated]"


def emit(**record: Any) -> None:
    row = {"ts_ms": now_ms(), **record}
    REPORT.open("a", encoding="utf-8").write(json.dumps(row, sort_keys=True, separators=(",", ":")) + "\n")
    print(json.dumps(row, sort_keys=True, separators=(",", ":")))


def scalar(cmd: list[str]) -> str | None:
    try:
        return subprocess.check_output(cmd, cwd=ROOT, text=True).strip()
    except Exception:
        return None


def save(cmd: list[str], out: str, err: str) -> dict[str, str]:
    stem = re.sub(r"[^A-Za-z0-9_.-]+", "_", "_".join(cmd))[:80] or "cmd"
    digest = hashlib.sha256(json.dumps(cmd, separators=(",", ":")).encode()).hexdigest()[:12]
    base = OUT / f"{stem}.{digest}"
    stdout, stderr = f"{base}.stdout.txt", f"{base}.stderr.txt"
    Path(stdout).write_text(out, encoding="utf-8")
    Path(stderr).write_text(err, encoding="utf-8")
    return {"stdout_path": stdout, "stderr_path": stderr}


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def numeric(value: Any) -> float | None:
    return float(value) if isinstance(value, (int, float)) and value >= 0 else None


def add_number(bucket: dict[str, list[float]], name: str, value: Any) -> None:
    number = numeric(value)
    if number is not None:
        bucket.setdefault(name, []).append(number)


def summary(values: list[float]) -> dict[str, Any]:
    if not values:
        return {"count": 0, "min": None, "median": None, "p95": None, "max": None}
    ordered = sorted(values)
    mid = len(ordered) // 2
    median = ordered[mid] if len(ordered) % 2 else (ordered[mid - 1] + ordered[mid]) / 2
    p95 = ordered[max(0, ceil(len(ordered) * 0.95) - 1)]
    return {"count": len(ordered), "min": round(ordered[0], 3), "median": round(median, 3),
            "p95": round(p95, 3), "max": round(ordered[-1], 3)}


def budget(name: str, default: float) -> float:
    try:
        return float(os.environ.get(name, default))
    except ValueError:
        return default


def budget_status(perf: dict[str, Any]) -> str:
    checks = {
        "project_agent_elapsed_ms": budget("CANON_MAX_PROJECT_AGENT_ELAPSED_MS_P95", 1_800_000),
        "download_initial_get_ms": budget("CANON_MAX_DOWNLOAD_INITIAL_GET_MS_P95", 60_000),
        "download_follow_get_ms": budget("CANON_MAX_DOWNLOAD_FOLLOW_GET_MS_P95", 60_000),
        "download_write_ms": budget("CANON_MAX_DOWNLOAD_WRITE_MS_P95", 1_000),
    }
    seen = False
    failures: dict[str, dict[str, float]] = {}
    for key, limit in checks.items():
        item = perf.get(key, {})
        if int(item.get("count") or 0) < 3 or item.get("p95") is None:
            continue
        seen = True
        if float(item["p95"]) > limit:
            failures[key] = {"p95": float(item["p95"]), "budget": limit}
    perf["runtime_performance_budget_failures"] = failures
    perf["runtime_performance_budgets"] = checks
    if failures:
        return "fail"
    return "pass" if seen else "missing"


def run(name: str, cmd: list[str], *, cwd: Path = ROOT, timeout: int = 120,
        env: dict[str, str] | None = None) -> dict[str, Any]:
    full_env = {**os.environ, **(env or {})}
    base = {"name": name, "cmd": cmd, "env_overrides": sorted((env or {}).keys())}
    if not cwd.exists():
        result = {**base, "status": "unavailable", "available": False, "returncode": None,
                  "duration_ms": 0, "stdout": "", "stderr": f"cwd not found: {cwd}"}
    elif shutil.which(cmd[0], path=full_env.get("PATH")) is None:
        result = {**base, "status": "unavailable", "available": False, "returncode": None,
                  "duration_ms": 0, "stdout": "", "stderr": f"{cmd[0]} not found in PATH"}
    else:
        start = time.monotonic()
        try:
            done = subprocess.run(cmd, cwd=cwd, env=full_env, text=True, stdout=subprocess.PIPE,
                                  stderr=subprocess.PIPE, timeout=timeout, check=False)
            result = {**base, "status": "pass" if done.returncode == 0 else "fail",
                      "available": True, "returncode": done.returncode,
                      "duration_ms": int((time.monotonic() - start) * 1000),
                      "stdout": short(done.stdout), "stderr": short(done.stderr)}
        except subprocess.TimeoutExpired as exc:
            result = {**base, "status": "timeout", "available": True, "returncode": None,
                      "duration_ms": int((time.monotonic() - start) * 1000),
                      "stdout": short(exc.stdout or ""), "stderr": short(exc.stderr or f"timeout after {timeout}s")}
    result["output_path"] = save(cmd, result["stdout"], result["stderr"])
    emit(event="validation_command", result=result)
    return result


def router_dir() -> Path:
    for rel in ("ai-chromium/router-server", "ai-chromium/router-server_bak"):
        path = ROOT / rel
        if (path / "run_tests.sh").exists():
            return path
    return ROOT / "ai-chromium" / "router-server"


def configure_toolchain() -> dict[str, Any]:
    before = {name: shutil.which(name) for name in ("cargo", "rustc")}
    sandbox = Path("/mnt/data/rust-sandbox/bin")
    added = False
    if (not before["cargo"] or not before["rustc"]) and (sandbox / "cargo").exists():
        os.environ["PATH"] = f"{sandbox}{os.pathsep}{os.environ.get('PATH', '')}"
        added = True
    after = {name: shutil.which(name) for name in ("cargo", "rustc")}
    source = "path" if after["cargo"] and not added else ("/mnt/data/rust-sandbox/bin" if after["cargo"] else "missing")
    return {"cargo_path_before": before["cargo"], "rustc_path_before": before["rustc"],
            "cargo_path": after["cargo"], "rustc_path": after["rustc"],
            "cargo_available": bool(after["cargo"]), "rustc_available": bool(after["rustc"]),
            "toolchain_path_added": added, "rust_toolchain_source": source}


def wrapper() -> dict[str, Any]:
    cfg = ROOT / ".cargo" / "config.toml"
    text = cfg.read_text(encoding="utf-8") if cfg.exists() else ""
    match = re.search(r'^\s*rustc-wrapper\s*=\s*"([^"]+)"', text, re.MULTILINE)
    cfg_path = match.group(1) if match else None
    env_path = os.environ.get("CANON_RUSTC_WRAPPER", "").strip() or None
    path = env_path or cfg_path
    exists = bool(path and Path(path).exists())
    return {"config_present": cfg.exists(), "rustc_wrapper_configured": bool(path),
            "rustc_wrapper_config_path": cfg_path, "rustc_wrapper_env_path": env_path,
            "rustc_wrapper_requested": bool(env_path), "rustc_wrapper_path": path,
            "rustc_wrapper_path_exists": exists,
            "wrapper_override_required": bool(cfg_path and not Path(cfg_path).exists()),
            "wrapper_graph_validation_requested": bool(env_path),
            "wrapper_graph_validation_available": bool(env_path and exists)}


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
                "graph_parse_status": "pass", "graph_node_count": len(nodes),
                "graph_edge_count": len(edges), "graph_function_node_count": len(fns),
                "graph_intent_node_count": len(intents),
                "graph_intent_coverage": len(intents) / len(fns) if fns else None}
    except Exception as exc:
        return {"state_graph_present": True, "graph_paths": [str(p) for p in paths],
                "graph_parse_status": "fail", "graph_parse_error": str(exc)}


def runtime() -> dict[str, Any]:
    candidates = [Path(p) for p in [os.environ.get("CANON_RUNTIME_ARCHIVE", ""),
                                    str(ROOT / "ai-runtime.tar.gz"), str(ROOT.parent / "ai-runtime.tar.gz"),
                                    "/mnt/data/ai-runtime.tar.gz"] if p]
    archive = next((p for p in candidates if p.exists()), None)
    if not archive:
        return {"runtime_archive_present": False, "runtime_archive_candidates": [str(p) for p in candidates]}
    metrics: dict[str, Any] = {"runtime_archive_present": True, "runtime_archive_path": str(archive),
                               "runtime_archive_sha256": sha256(archive),
                               "runtime_archive_member_count": 0, "runtime_archive_log_total": 0,
                               "runtime_archive_download_total": 0, "runtime_archive_log_files": 0,
                               "runtime_archive_download_files": 0, "runtime_archive_sample_files": [],
                               "runtime_archive_conversation_snapshots": 0, "runtime_archive_cache_files": 0,
                               "runtime_archive_conversation_ledger_files": 0,
                               "runtime_archive_download_index_files": 0,
                               "runtime_archive_prior_state_files": 0,
                               "runtime_archive_delta_receipt_files": 0,
                               "runtime_archive_audit_files": 0,
                               "runtime_archive_current_run_summary_present": False,
                               "runtime_archive_runtime_manifest_present": False,
                               "runtime_candidate_error_count": 0, "runtime_duplicate_artifact_aliases": 0}
    aliases: dict[str, int] = {}
    accepted_aliases: dict[str, str] = {}
    timings: dict[str, list[float]] = {}
    try:
        with tarfile.open(archive, "r:gz") as tf:
            for member in (m for m in tf.getmembers() if m.isfile()):
                name = member.name
                lower_name = name.lower()
                metrics["runtime_archive_member_count"] += 1
                if len(metrics["runtime_archive_sample_files"]) < 12:
                    metrics["runtime_archive_sample_files"].append(name)
                metrics["runtime_archive_cache_files"] += int("cache" in lower_name)
                metrics["runtime_archive_conversation_snapshots"] += int(name.endswith(".conversation.json"))
                metrics["runtime_archive_conversation_ledger_files"] += int(name.endswith(".messages.ndjson"))
                metrics["runtime_archive_download_index_files"] += int(
                    name.endswith(".downloads.ndjson")
                    or name.endswith(".candidate-ledger.ndjson")
                    or name.endswith(".resolved-urls.json")
                    or name.endswith("RUNTIME_MANIFEST.json")
                )
                metrics["runtime_archive_delta_receipt_files"] += int(
                    ".repo-agent-runtime/delta-apply-receipts/" in name and name.endswith(".json")
                )
                metrics["runtime_archive_audit_files"] += int(name.endswith("audit.ndjson"))
                if name.endswith("current-run-summary.json"):
                    metrics["runtime_archive_current_run_summary_present"] = True
                if name.endswith("RUNTIME_MANIFEST.json"):
                    metrics["runtime_archive_runtime_manifest_present"] = True
                if name.endswith("RUNTIME_MANIFEST.json"):
                    manifest = json.load(tf.extractfile(member) or open(os.devnull))
                    metrics["runtime_manifest_base_commit"] = manifest.get("baseCommit")
                    metrics["runtime_download_history_by_classification"] = manifest.get("downloadHistoryByClassification", {})
                    metrics["runtime_download_history_record_count"] = len(manifest.get("downloadHistory", []))
                    for item in manifest.get("downloadHistory", []):
                        if item.get("classification") == "stale_advisory":
                            continue
                        alias = item.get("fileName") or item.get("savedAs")
                        if alias:
                            accepted_aliases[str(alias)] = str(item.get("sha256") or item.get("candidateHash") or "")
                    metrics["runtime_stale_advisory_count"] = int(
                        metrics["runtime_download_history_by_classification"].get("stale_advisory", 0) or 0
                    )
                if name.endswith((".ndjson", ".jsonl")):
                    fh = tf.extractfile(member)
                    count = 0
                    for raw in fh or []:
                        if not raw.strip():
                            continue
                        count += 1
                        try:
                            row = json.loads(raw)
                        except Exception:
                            row = None
                        if isinstance(row, dict):
                            if "elapsedMs" in row:
                                add_number(timings, "project_agent_elapsed_ms", row.get("elapsedMs"))
                            row_timings = row.get("timingsMs")
                            if isinstance(row_timings, dict):
                                for key in ("turnTotalMs", "loopTotalMs", "uploadMs", "validationMs"):
                                    add_number(timings, f"project_agent_{key}", row_timings.get(key))
                            candidate = row.get("candidate")
                            if isinstance(candidate, dict):
                                candidate_timings = candidate.get("timingsMs")
                                if isinstance(candidate_timings, dict):
                                    for src, dst in {
                                        "initialGetMs": "download_initial_get_ms",
                                        "followGetMs": "download_follow_get_ms",
                                        "resolvedGetMs": "download_resolved_get_ms",
                                        "writeMs": "download_write_ms",
                                    }.items():
                                        add_number(timings, dst, candidate_timings.get(src))
                        if "candidate-ledger" in name:
                            try:
                                row = row if isinstance(row, dict) else json.loads(raw)
                                alias = str(row.get("artifactAlias") or "")
                                if alias:
                                    aliases[alias] = aliases.get(alias, 0) + 1
                                metrics["runtime_candidate_error_count"] += int(bool(row.get("error")))
                            except Exception:
                                metrics["runtime_candidate_error_count"] += 1
                    is_download = "download" in name.lower()
                    metrics["runtime_archive_download_files" if is_download else "runtime_archive_log_files"] += 1
                    metrics["runtime_archive_download_total" if is_download else "runtime_archive_log_total"] += count
        metrics["runtime_duplicate_artifact_aliases"] = sum(1 for value in aliases.values() if value > 1)
        metrics["runtime_archive_prior_state_files"] = (
            metrics["runtime_archive_delta_receipt_files"]
            + metrics["runtime_archive_audit_files"]
            + int(bool(metrics["runtime_archive_current_run_summary_present"]))
        )
        metrics["runtime_unique_download_alias_count"] = len(accepted_aliases)
        metrics["runtime_unique_download_aliases"] = sorted(accepted_aliases)
        metrics["runtime_performance_metrics"] = {key: summary(value) for key, value in sorted(timings.items())}
        metrics["runtime_performance_signal_present"] = bool(timings)
        metrics["runtime_performance_budget_status"] = budget_status(metrics["runtime_performance_metrics"])
        metrics["runtime_archive_parse_status"] = "pass"
        metrics["runtime_archive_inspection_status"] = "pass" if (
            metrics["runtime_archive_log_files"] > 0
            and metrics["runtime_archive_download_index_files"] > 0
            and metrics["runtime_archive_prior_state_files"] > 0
            and (
                metrics["runtime_archive_conversation_snapshots"] > 0
                or metrics["runtime_archive_conversation_ledger_files"] > 0
            )
        ) else "missing"
    except Exception as exc:
        metrics.update({"runtime_archive_parse_status": "fail", "runtime_archive_inspection_status": "fail",
                        "runtime_archive_parse_error": str(exc)})
    delta_base = os.environ.get("CANON_DELTA_BASE", "").strip()
    manifest_base = metrics.get("runtime_manifest_base_commit")
    metrics["runtime_manifest_base_expected"] = delta_base or None
    metrics["runtime_manifest_base_matches_delta_base"] = (
        bool(delta_base) and bool(manifest_base) and manifest_base == delta_base
    )
    return metrics


def repo_metrics() -> dict[str, Any]:
    tracked = scalar(["git", "ls-files"]) or ""
    files = [ROOT / line for line in tracked.splitlines() if line]
    rust_files = [p for p in files if p.suffix == ".rs" and ("src" in p.parts or "examples" in p.parts)]
    rust_text = "\n".join(p.read_text(encoding="utf-8", errors="replace") for p in rust_files if p.exists())
    return {
        "tracked_file_count": len(files),
        "rust_file_count_src_examples": len(rust_files),
        "rust_test_attr_count": len(re.findall(r"#\s*\[\s*test\s*\]", rust_text)),
        "rust_cfg_test_count": len(re.findall(r"#\s*\[\s*cfg\s*\(\s*test\s*\)", rust_text)),
        "unwrap_call_count_src_examples": len(re.findall(r"\.unwrap\s*\(", rust_text)),
        "expect_call_count_src_examples": len(re.findall(r"\.expect\s*\(", rust_text)),
        "panic_call_count_src_examples": len(re.findall(r"\bpanic!\s*\(", rust_text)),
        "unsafe_token_count_src_examples": len(re.findall(r"\bunsafe\b", rust_text)),
    }


def panic_surface_metrics() -> dict[str, Any]:
    if not PANIC_SURFACE_REPORT.exists():
        return {"panic_surface_report_present": False}
    try:
        data = json.loads(PANIC_SURFACE_REPORT.read_text(encoding="utf-8"))
    except Exception as exc:
        return {"panic_surface_report_present": True, "panic_surface_report_status": "fail", "panic_surface_report_error": str(exc)}
    buckets = data.get("buckets", {}) if isinstance(data, dict) else {}
    production = buckets.get("production", {}) if isinstance(buckets, dict) else {}
    test = buckets.get("test", {}) if isinstance(buckets, dict) else {}
    example = buckets.get("example", {}) if isinstance(buckets, dict) else {}
    return {
        "panic_surface_report_present": True,
        "panic_surface_report_status": "pass",
        "panic_surface_production_unwrap_count": int(production.get("unwrap", 0) or 0),
        "panic_surface_production_expect_count": int(production.get("expect", 0) or 0),
        "panic_surface_production_panic_count": int(production.get("panic", 0) or 0),
        "panic_surface_test_total": int(data.get("test_total", 0) or 0),
        "panic_surface_example_total": int(data.get("example_total", 0) or 0),
        "panic_surface_finding_count": int(data.get("finding_count", 0) or 0),
    }


def policy_learning_trace_metrics() -> dict[str, Any]:
    if not POLICY_LEARNING_TRACE_REPORT.exists():
        return {"policy_learning_trace_report_present": False}
    try:
        data = json.loads(POLICY_LEARNING_TRACE_REPORT.read_text(encoding="utf-8"))
    except Exception as exc:
        return {"policy_learning_trace_report_present": True,
                "policy_learning_trace_status": "fail", "policy_learning_trace_error": str(exc)}
    checks = data.get("checks", []) if isinstance(data, dict) else []
    return {"policy_learning_trace_report_present": True,
            "policy_learning_trace_status": data.get("status"),
            "policy_learning_trace_function": data.get("trace_function"),
            "policy_learning_trace_check_count": len(checks),
            "policy_learning_trace_missing_count": int(data.get("missing_count") or 0)}


def delta_metrics() -> dict[str, Any]:
    base = os.environ.get("CANON_DELTA_BASE", "").strip()
    if not base:
        return {"delta_base_commit": None, "delta_base_is_ancestor": None, "delta_changed_files": []}
    head = scalar(["git", "rev-parse", "HEAD"])
    base_exists = subprocess.run(["git", "cat-file", "-e", f"{base}^{{commit}}"], cwd=ROOT).returncode == 0
    is_ancestor = base_exists and subprocess.run(["git", "merge-base", "--is-ancestor", base, head or "HEAD"], cwd=ROOT).returncode == 0
    changed = scalar(["git", "diff", "--name-only", f"{base}..{head}"]) if is_ancestor and head else ""
    return {
        "delta_base_commit": base,
        "delta_head_commit": head,
        "delta_base_exists": base_exists,
        "delta_base_is_ancestor": is_ancestor,
        "delta_changed_files": changed.splitlines() if changed else [],
    }


def delta_diff_command() -> list[str]:
    base = os.environ.get("CANON_DELTA_BASE", "").strip()
    if base:
        return ["git", "diff", "--check", f"{base}..HEAD"]
    return ["git", "diff", "--check"]


def cargo_tests(result: dict[str, Any]) -> int | None:
    text = f"{result.get('stdout', '')}\n{result.get('stderr', '')}"
    counts = [int(x) for x in re.findall(r"running\s+(\d+)\s+tests?", text)]
    return sum(counts) if counts else None


def router_tests(result: dict[str, Any]) -> int:
    text = f"{result.get('stdout', '')}\n{result.get('stderr', '')}"
    counts = [int(x) for x in re.findall(r"tests\s+(\d+)", text)]
    return max(counts) if counts else text.count(".")


def command_duration_metrics(commands: list[dict[str, Any]]) -> dict[str, Any]:
    durations = [float(c.get("duration_ms") or 0) for c in commands if c.get("duration_ms") is not None]
    return {"validation_command_duration_ms": summary(durations)}


def unittest_tests(result: dict[str, Any]) -> int:
    text = f"{result.get('stdout', '')}\n{result.get('stderr', '')}"
    counts = [int(x) for x in re.findall(r"Ran\s+(\d+)\s+tests?", text)]
    return max(counts) if counts else 0


def synthetic(name: str, cmd: list[str], status: str, stderr: str,
              env: dict[str, str] | None = None) -> dict[str, Any]:
    result = {"name": name, "cmd": cmd, "env_overrides": sorted((env or {}).keys()),
              "status": status, "available": False, "returncode": None,
              "duration_ms": 0, "stdout": "", "stderr": stderr}
    result["output_path"] = save(cmd, "", stderr)
    emit(event="validation_command", result=result)
    return result


def wrapper_graph_command(w: dict[str, Any], toolchain: dict[str, Any]) -> dict[str, Any]:
    cmd = ["cargo", "test", "--all-targets"]
    if not w["wrapper_graph_validation_requested"]:
        return synthetic("wrapper_graph_validation", cmd, "skipped_env_missing",
                         "missing CANON_RUSTC_WRAPPER")
    if not w["rustc_wrapper_path_exists"]:
        return synthetic("wrapper_graph_validation", cmd, "unavailable",
                         f"CANON_RUSTC_WRAPPER not found: {w['rustc_wrapper_env_path']}")
    artifact_dir = os.environ.get("CANON_RUSTC_V3_ARTIFACT_DIR") or os.environ.get("CANON_RUSTC_V2_ARTIFACT_DIR", "state/rustc")
    env = {"RUSTC_WRAPPER": str(w["rustc_wrapper_path"]), "RUSTC_WORKSPACE_WRAPPER": "",
           "CANON_RUSTC_V3_ARTIFACT_DIR": artifact_dir}
    if not toolchain["cargo_available"]:
        return synthetic("wrapper_graph_validation", cmd, "unavailable", "cargo not found in PATH", env)
    return run("wrapper_graph_validation", cmd, timeout=600, env=env)


toolchain = configure_toolchain()
w, g0, r, repo, delta = wrapper(), graph(), runtime(), repo_metrics(), delta_metrics()
root_rust_env = {"RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": ""}
wrapper_override_used = bool(w["wrapper_override_required"] and toolchain["cargo_available"])

head = scalar(["git", "rev-parse", "HEAD"])
git_status = subprocess.run(["git", "status", "--short"], cwd=ROOT, text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)

emit(event="validation_start", schema_version=3, repo=str(ROOT), report_path=str(REPORT), git_head=head)
emit(event="git_state", git_head=head, git_status_clean=git_status.returncode == 0 and not git_status.stdout.strip(),
     git_status_short=git_status.stdout.splitlines())
emit(event="toolchain", python3_available=shutil.which("python3") is not None, **toolchain, **w,
     root_rust_env_overrides=sorted(root_rust_env.keys()), wrapper_override_used=wrapper_override_used,
     wrapper_override_env=sorted(root_rust_env.keys()) if wrapper_override_used else [])
emit(event="repo_metrics", **repo)
emit(event="graph_metrics_before_validation", **g0)
emit(event="runtime_archive_metrics", **r)
emit(event="delta_metrics", **delta)

commands = [
    run("git_diff_check", ["git", "diff", "--check"], timeout=30),
    run("git_delta_diff_check", delta_diff_command(), timeout=30),
    run("python_unit_tests", ["python3", "-m", "unittest", "discover", "-s", "tests", "-p", "test_*.py"], timeout=90),
    run("panic_surface_validation", ["python3", "scripts/validate_rust_panic_surface.py", "--root", ".", "--fail-production-unwrap", "--report", str(PANIC_SURFACE_REPORT)], timeout=90),
    run("policy_learning_trace_validation", ["python3", "scripts/validate_policy_learning_trace.py", "--root", ".", "--report", str(POLICY_LEARNING_TRACE_REPORT)], timeout=90),
    run("router_offline_tests", ["bash", "run_tests.sh"], cwd=router_dir(), timeout=180),
    run("cargo_fmt_check", ["cargo", "fmt", "--check"], timeout=180, env=root_rust_env),
    run("cargo_test_all_targets", ["cargo", "test", "--all-targets"], timeout=600, env=root_rust_env),
    run("cargo_clippy_all_targets", ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"], timeout=600, env=root_rust_env),
    wrapper_graph_command(w, toolchain),
]
g = graph()
p = panic_surface_metrics()
policy_trace = policy_learning_trace_metrics()
command_perf = command_duration_metrics(commands)
emit(event="graph_metrics", **g)
emit(event="panic_surface_metrics", **p)
emit(event="policy_learning_trace_metrics", **policy_trace)
runtime_perf = r.get("runtime_performance_metrics", {}) if isinstance(r.get("runtime_performance_metrics"), dict) else {}
emit(event="runtime_performance_metrics", runtime_performance_signal_present=r.get("runtime_performance_signal_present", False),
     runtime_performance_budget_status=r.get("runtime_performance_budget_status", "missing"),
     **runtime_perf, **command_perf)

ollama_env = os.environ.get("CANON_OLLAMA_BASE_URL") and os.environ.get("CANON_OLLAMA_MODEL")
if ollama_env:
    commands.append(run("ollama_judgment_example", ["cargo", "run", "--example", "ollama_judgment"], timeout=600, env=root_rust_env))
else:
    skipped = {"name": "ollama_judgment_example", "cmd": ["cargo", "run", "--example", "ollama_judgment"],
               "env_overrides": sorted(root_rust_env.keys()), "status": "skipped_env_missing",
               "available": toolchain["cargo_available"], "returncode": None, "duration_ms": 0,
               "stdout": "", "stderr": "missing CANON_OLLAMA_BASE_URL or CANON_OLLAMA_MODEL"}
    skipped["output_path"] = save(skipped["cmd"], "", skipped["stderr"])
    commands.append(skipped)
    emit(event="validation_command", result=skipped)

cargo_test = next(c for c in commands if c["name"] == "cargo_test_all_targets")
router_test = next(c for c in commands if c["name"] == "router_offline_tests")
download_total = int(r.get("runtime_archive_download_total", 0) or 0)
log_total = int(r.get("runtime_archive_log_total", 0) or 0)
missing = {
    "missing_root_rust_toolchain": not (toolchain["cargo_available"] and toolchain["rustc_available"]),
    "missing_cargo_fmt": next(c for c in commands if c["name"] == "cargo_fmt_check")["status"] != "pass",
    "missing_cargo_test": cargo_test["status"] != "pass",
    "missing_cargo_run_ollama_judgment": next(c for c in commands if c["name"] == "ollama_judgment_example")["status"] != "pass",
    "missing_clippy": next(c for c in commands if c["name"] == "cargo_clippy_all_targets")["status"] != "pass",
    "missing_wrapper_graph_validation": next(c for c in commands if c["name"] == "wrapper_graph_validation")["status"] != "pass",
    "missing_generated_graph_json": not bool(g.get("state_graph_present")),
    "missing_rustc_wrapper_telemetry": next(c for c in commands if c["name"] == "wrapper_graph_validation")["status"] != "pass" or not bool(g.get("state_graph_present")),
    "missing_runtime_download_history": download_total == 0,
    "missing_runtime_download_index": int(r.get("runtime_archive_download_index_files", 0) or 0) == 0,
    "missing_runtime_prior_state": int(r.get("runtime_archive_prior_state_files", 0) or 0) == 0,
    "missing_runtime_conversation_ledger": (
        int(r.get("runtime_archive_conversation_snapshots", 0) or 0) == 0
        and int(r.get("runtime_archive_conversation_ledger_files", 0) or 0) == 0
    ),
    "missing_runtime_inspection_contract": r.get("runtime_archive_inspection_status") != "pass",
    "missing_runtime_manifest_base_match": bool(r.get("runtime_archive_present"))
    and bool(r.get("runtime_manifest_base_expected"))
    and not bool(r.get("runtime_manifest_base_matches_delta_base")),
    "missing_runtime_performance_signal": not bool(r.get("runtime_performance_signal_present")),
    "missing_panic_surface_validation": next(c for c in commands if c["name"] == "panic_surface_validation")["status"] != "pass",
    "missing_router_offline_tests": router_test["status"] != "pass",
    "missing_conversation_snapshot": int(r.get("runtime_archive_conversation_snapshots", 0) or 0) == 0,
    "missing_artifact_apply_worktree": not (ROOT / ".repo-agent-runtime" / "apply-worktrees").exists(),
    "missing_external_observation_stream_test": True,
    "missing_external_api_action_test": True,
    "missing_semantic_artifact_verification_test": True,
    "missing_policy_learning_replay_trace": next(c for c in commands if c["name"] == "policy_learning_trace_validation")["status"] != "pass",
}
required = {"git_diff_check", "git_delta_diff_check", "python_unit_tests"}
required.add("panic_surface_validation")
required.add("policy_learning_trace_validation")
if router_test.get("available"):
    required.add("router_offline_tests")
failed_required = [c["name"] for c in commands if c["name"] in required and c["status"] != "pass"]
performance_budget_status = r.get("runtime_performance_budget_status", "missing")
status = "fail" if failed_required or performance_budget_status == "fail" else ("partial" if any(missing.values()) else "pass")
python_test = next(c for c in commands if c["name"] == "python_unit_tests")
project_elapsed = runtime_perf.get("project_agent_elapsed_ms", {})
download_initial = runtime_perf.get("download_initial_get_ms", {})
download_follow = runtime_perf.get("download_follow_get_ms", {})
download_resolved = runtime_perf.get("download_resolved_get_ms", {})
download_write = runtime_perf.get("download_write_ms", {})
emit(event="validation_summary", validation_status=status, git_head=head,
     git_status_clean=git_status.returncode == 0 and not git_status.stdout.strip(),
     validation_command_count=len(commands),
     validation_commands=[{"name": c["name"], "cmd": c["cmd"], "status": c["status"],
                           "env_overrides": c.get("env_overrides", [])} for c in commands],
     validation_test_count=unittest_tests(python_test) + (cargo_tests(cargo_test) or 0) + router_tests(router_test),
     python_unit_test_count=unittest_tests(python_test), router_test_count=router_tests(router_test),
     cargo_test_count_when_available=cargo_tests(cargo_test),
     failed_required_commands=failed_required, cargo_available=toolchain["cargo_available"],
     rustc_available=toolchain["rustc_available"], toolchain_path_added=toolchain["toolchain_path_added"],
     rust_toolchain_source=toolchain["rust_toolchain_source"], root_rust_env_overrides=sorted(root_rust_env.keys()),
     wrapper_graph_validation_result=next(c for c in commands if c["name"] == "wrapper_graph_validation")["status"],
     wrapper_graph_validation_requested=w["wrapper_graph_validation_requested"],
     wrapper_graph_validation_available=w["wrapper_graph_validation_available"],
     wrapper_override_required=w["wrapper_override_required"],
     wrapper_override_used=wrapper_override_used, wrapper_override_env=sorted(root_rust_env.keys()) if wrapper_override_used else [],
     rustc_wrapper_configured=w["rustc_wrapper_configured"], rustc_wrapper_path_exists=w["rustc_wrapper_path_exists"],
     git_delta_diff_check_result=next(c for c in commands if c["name"] == "git_delta_diff_check")["status"],
     cargo_fmt_check_result=next(c for c in commands if c["name"] == "cargo_fmt_check")["status"],
     cargo_test_result=cargo_test["status"], clippy_result_when_available=next(c for c in commands if c["name"] == "cargo_clippy_all_targets")["status"],
     ollama_example_result_when_available=next(c for c in commands if c["name"] == "ollama_judgment_example")["status"],
     policy_learning_trace_validation_result=next(c for c in commands if c["name"] == "policy_learning_trace_validation")["status"],
     policy_learning_trace_status=policy_trace.get("policy_learning_trace_status"),
     policy_learning_trace_function=policy_trace.get("policy_learning_trace_function"),
     policy_learning_trace_check_count=policy_trace.get("policy_learning_trace_check_count"),
     policy_learning_trace_missing_count=policy_trace.get("policy_learning_trace_missing_count"),
     state_graph_present=g.get("state_graph_present", False), graph_node_count_when_present=g.get("graph_node_count"),
     graph_edge_count_when_present=g.get("graph_edge_count"), graph_intent_coverage_when_present=g.get("graph_intent_coverage"),
     runtime_archive_present=r.get("runtime_archive_present", False), runtime_archive_log_total=log_total,
     runtime_archive_download_total=download_total,
     runtime_archive_inspection_status=r.get("runtime_archive_inspection_status"),
     runtime_archive_download_index_files=r.get("runtime_archive_download_index_files", 0),
     runtime_archive_prior_state_files=r.get("runtime_archive_prior_state_files", 0),
     runtime_archive_conversation_ledger_files=r.get("runtime_archive_conversation_ledger_files", 0),
     runtime_archive_delta_receipt_files=r.get("runtime_archive_delta_receipt_files", 0),
     runtime_archive_audit_files=r.get("runtime_archive_audit_files", 0),
     runtime_archive_current_run_summary_present=r.get("runtime_archive_current_run_summary_present", False),
     runtime_archive_runtime_manifest_present=r.get("runtime_archive_runtime_manifest_present", False),
     runtime_performance_signal_present=r.get("runtime_performance_signal_present", False),
     runtime_performance_budget_status=performance_budget_status,
     runtime_performance_budget_failures=runtime_perf.get("runtime_performance_budget_failures", {}),
     runtime_performance_budgets=runtime_perf.get("runtime_performance_budgets", {}),
     validation_command_duration_ms=command_perf["validation_command_duration_ms"],
     project_agent_elapsed_ms_count=project_elapsed.get("count", 0),
     project_agent_elapsed_ms_median=project_elapsed.get("median"),
     project_agent_elapsed_ms_p95=project_elapsed.get("p95"),
     project_agent_elapsed_ms_max=project_elapsed.get("max"),
     download_initial_get_ms_median=download_initial.get("median"),
     download_initial_get_ms_p95=download_initial.get("p95"),
     download_initial_get_ms_max=download_initial.get("max"),
     download_follow_get_ms_median=download_follow.get("median"),
     download_follow_get_ms_p95=download_follow.get("p95"),
     download_follow_get_ms_max=download_follow.get("max"),
     download_resolved_get_ms_median=download_resolved.get("median"),
     download_resolved_get_ms_p95=download_resolved.get("p95"),
     download_resolved_get_ms_max=download_resolved.get("max"),
     download_write_ms_median=download_write.get("median"),
     download_write_ms_p95=download_write.get("p95"),
     download_write_ms_max=download_write.get("max"),
     runtime_download_history_record_count=r.get("runtime_download_history_record_count", 0),
     runtime_unique_download_alias_count=r.get("runtime_unique_download_alias_count", 0),
     runtime_unique_download_aliases=r.get("runtime_unique_download_aliases", []),
     runtime_archive_conversation_snapshots=r.get("runtime_archive_conversation_snapshots", 0),
     runtime_archive_sha256=r.get("runtime_archive_sha256"),
     runtime_manifest_base_commit=r.get("runtime_manifest_base_commit"),
     runtime_manifest_base_expected=r.get("runtime_manifest_base_expected"),
     runtime_manifest_base_matches_delta_base=r.get("runtime_manifest_base_matches_delta_base"),
     runtime_stale_advisory_count=r.get("runtime_stale_advisory_count", 0),
     runtime_candidate_error_count=r.get("runtime_candidate_error_count", 0),
     runtime_duplicate_artifact_aliases=r.get("runtime_duplicate_artifact_aliases", 0),
     delta_base_commit=delta.get("delta_base_commit"), delta_base_is_ancestor=delta.get("delta_base_is_ancestor"),
     delta_changed_file_count=len(delta.get("delta_changed_files", [])),
     tracked_file_count=repo["tracked_file_count"], rust_file_count_src_examples=repo["rust_file_count_src_examples"],
     rust_test_attr_count=repo["rust_test_attr_count"], rust_cfg_test_count=repo["rust_cfg_test_count"],
     unwrap_call_count_src_examples=repo["unwrap_call_count_src_examples"],
     expect_call_count_src_examples=repo["expect_call_count_src_examples"],
     panic_surface_production_unwrap_count=p.get("panic_surface_production_unwrap_count"),
     panic_surface_production_expect_count=p.get("panic_surface_production_expect_count"),
     panic_surface_production_panic_count=p.get("panic_surface_production_panic_count"),
     panic_surface_test_total=p.get("panic_surface_test_total"),
     panic_surface_example_total=p.get("panic_surface_example_total"),
     missing_signal_flags=missing, missing_signal_count=sum(1 for v in missing.values() if v))
PY
