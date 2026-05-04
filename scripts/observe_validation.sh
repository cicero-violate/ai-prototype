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
from pathlib import Path
from typing import Any

ROOT = Path.cwd()
REPORT = Path(os.environ.get("CANON_OBSERVE_REPORT", "target/observe/validation-report.ndjson"))
OUT = REPORT.parent / "command-output"
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
    path = match.group(1) if match else None
    exists = bool(path and Path(path).exists())
    return {"config_present": cfg.exists(), "rustc_wrapper_configured": bool(path),
            "rustc_wrapper_path": path, "rustc_wrapper_path_exists": exists,
            "wrapper_override_required": bool(path and not exists)}


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
                               "runtime_archive_member_count": 0, "runtime_archive_log_counts": {},
                               "runtime_archive_download_counts": {}, "runtime_archive_conversation_snapshots": 0,
                               "runtime_archive_cache_files": 0, "runtime_candidate_error_count": 0,
                               "runtime_duplicate_artifact_aliases": 0}
    aliases: dict[str, int] = {}
    try:
        with tarfile.open(archive, "r:gz") as tf:
            for member in (m for m in tf.getmembers() if m.isfile()):
                name = member.name
                metrics["runtime_archive_member_count"] += 1
                metrics["runtime_archive_cache_files"] += int("cache" in name.lower())
                metrics["runtime_archive_conversation_snapshots"] += int(name.endswith(".conversation.json"))
                if name.endswith("RUNTIME_MANIFEST.json"):
                    manifest = json.load(tf.extractfile(member) or open(os.devnull))
                    metrics["runtime_manifest_base_commit"] = manifest.get("baseCommit")
                    metrics["runtime_download_history_by_classification"] = manifest.get("downloadHistoryByClassification", {})
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
                        if "candidate-ledger" in name:
                            try:
                                row = json.loads(raw)
                                alias = str(row.get("artifactAlias") or "")
                                if alias:
                                    aliases[alias] = aliases.get(alias, 0) + 1
                                metrics["runtime_candidate_error_count"] += int(bool(row.get("error")))
                            except Exception:
                                metrics["runtime_candidate_error_count"] += 1
                    key = "runtime_archive_download_counts" if "download" in name.lower() else "runtime_archive_log_counts"
                    metrics[key][name] = count
        metrics["runtime_duplicate_artifact_aliases"] = sum(1 for value in aliases.values() if value > 1)
        metrics["runtime_archive_parse_status"] = "pass"
    except Exception as exc:
        metrics.update({"runtime_archive_parse_status": "fail", "runtime_archive_parse_error": str(exc)})
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


toolchain = configure_toolchain()
w, g, r, repo, delta = wrapper(), graph(), runtime(), repo_metrics(), delta_metrics()
cargo_env = {"RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": ""} if w["wrapper_override_required"] else {}
wrapper_override_used = bool(cargo_env and toolchain["cargo_available"])

head = scalar(["git", "rev-parse", "HEAD"])
git_status = subprocess.run(["git", "status", "--short"], cwd=ROOT, text=True,
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)

emit(event="validation_start", schema_version=3, repo=str(ROOT), report_path=str(REPORT), git_head=head)
emit(event="git_state", git_head=head, git_status_clean=git_status.returncode == 0 and not git_status.stdout.strip(),
     git_status_short=git_status.stdout.splitlines())
emit(event="toolchain", python3_available=shutil.which("python3") is not None, **toolchain, **w,
     wrapper_override_used=wrapper_override_used, wrapper_override_env=sorted(cargo_env.keys()))
emit(event="repo_metrics", **repo)
emit(event="graph_metrics", **g)
emit(event="runtime_archive_metrics", **r)
emit(event="delta_metrics", **delta)

commands = [
    run("git_diff_check", ["git", "diff", "--check"], timeout=30),
    run("git_delta_diff_check", delta_diff_command(), timeout=30),
    run("router_offline_tests", ["bash", "run_tests.sh"], cwd=router_dir(), timeout=180),
    run("cargo_fmt_check", ["cargo", "fmt", "--check"], timeout=180, env=cargo_env),
    run("cargo_test_all_targets", ["cargo", "test", "--all-targets"], timeout=600, env=cargo_env),
    run("cargo_clippy_all_targets", ["cargo", "clippy", "--all-targets", "--", "-D", "warnings"], timeout=600, env=cargo_env),
]

ollama_env = os.environ.get("CANON_OLLAMA_BASE_URL") and os.environ.get("CANON_OLLAMA_MODEL")
if ollama_env:
    commands.append(run("ollama_judgment_example", ["cargo", "run", "--example", "ollama_judgment"], timeout=600, env=cargo_env))
else:
    skipped = {"name": "ollama_judgment_example", "cmd": ["cargo", "run", "--example", "ollama_judgment"],
               "env_overrides": sorted(cargo_env.keys()), "status": "skipped_env_missing",
               "available": toolchain["cargo_available"], "returncode": None, "duration_ms": 0,
               "stdout": "", "stderr": "missing CANON_OLLAMA_BASE_URL or CANON_OLLAMA_MODEL"}
    skipped["output_path"] = save(skipped["cmd"], "", skipped["stderr"])
    commands.append(skipped)
    emit(event="validation_command", result=skipped)

cargo_test = next(c for c in commands if c["name"] == "cargo_test_all_targets")
router_test = next(c for c in commands if c["name"] == "router_offline_tests")
download_total = sum(map(int, r.get("runtime_archive_download_counts", {}).values())) if r.get("runtime_archive_present") else 0
log_total = sum(map(int, r.get("runtime_archive_log_counts", {}).values())) if r.get("runtime_archive_present") else 0
missing = {
    "missing_root_rust_toolchain": not (toolchain["cargo_available"] and toolchain["rustc_available"]),
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
required = {"git_diff_check", "git_delta_diff_check", "router_offline_tests"}
failed_required = [c["name"] for c in commands if c["name"] in required and c["status"] != "pass"]
status = "fail" if failed_required else ("partial" if any(missing.values()) else "pass")
emit(event="validation_summary", validation_status=status, git_head=head,
     git_status_clean=git_status.returncode == 0 and not git_status.stdout.strip(),
     validation_command_count=len(commands),
     validation_commands=[{"name": c["name"], "cmd": c["cmd"], "status": c["status"],
                           "env_overrides": c.get("env_overrides", [])} for c in commands],
     validation_test_count=(cargo_tests(cargo_test) or 0) + router_tests(router_test),
     router_test_count=router_tests(router_test), cargo_test_count_when_available=cargo_tests(cargo_test),
     failed_required_commands=failed_required, cargo_available=toolchain["cargo_available"],
     rustc_available=toolchain["rustc_available"], toolchain_path_added=toolchain["toolchain_path_added"],
     rust_toolchain_source=toolchain["rust_toolchain_source"], wrapper_override_required=w["wrapper_override_required"],
     wrapper_override_used=wrapper_override_used, wrapper_override_env=sorted(cargo_env.keys()),
     rustc_wrapper_configured=w["rustc_wrapper_configured"], rustc_wrapper_path_exists=w["rustc_wrapper_path_exists"],
     git_delta_diff_check_result=next(c for c in commands if c["name"] == "git_delta_diff_check")["status"],
     cargo_fmt_check_result=next(c for c in commands if c["name"] == "cargo_fmt_check")["status"],
     cargo_test_result=cargo_test["status"], clippy_result_when_available=next(c for c in commands if c["name"] == "cargo_clippy_all_targets")["status"],
     ollama_example_result_when_available=next(c for c in commands if c["name"] == "ollama_judgment_example")["status"],
     state_graph_present=g.get("state_graph_present", False), graph_node_count_when_present=g.get("graph_node_count"),
     graph_edge_count_when_present=g.get("graph_edge_count"), graph_intent_coverage_when_present=g.get("graph_intent_coverage"),
     runtime_archive_present=r.get("runtime_archive_present", False), runtime_archive_log_total=log_total,
     runtime_archive_download_total=download_total,
     runtime_archive_conversation_snapshots=r.get("runtime_archive_conversation_snapshots", 0),
     runtime_archive_sha256=r.get("runtime_archive_sha256"),
     runtime_manifest_base_commit=r.get("runtime_manifest_base_commit"),
     runtime_stale_advisory_count=r.get("runtime_stale_advisory_count", 0),
     runtime_candidate_error_count=r.get("runtime_candidate_error_count", 0),
     runtime_duplicate_artifact_aliases=r.get("runtime_duplicate_artifact_aliases", 0),
     delta_base_commit=delta.get("delta_base_commit"), delta_base_is_ancestor=delta.get("delta_base_is_ancestor"),
     delta_changed_file_count=len(delta.get("delta_changed_files", [])),
     tracked_file_count=repo["tracked_file_count"], rust_file_count_src_examples=repo["rust_file_count_src_examples"],
     rust_test_attr_count=repo["rust_test_attr_count"], rust_cfg_test_count=repo["rust_cfg_test_count"],
     unwrap_call_count_src_examples=repo["unwrap_call_count_src_examples"],
     expect_call_count_src_examples=repo["expect_call_count_src_examples"],
     missing_signal_flags=missing, missing_signal_count=sum(1 for v in missing.values() if v))
PY
