#!/usr/bin/env python3
"""Portable semantic-witness validation with replay and timing receipts."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import time


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_FIXTURE = ROOT / "validation/fixtures/witness_crate/Cargo.toml"
REPORT_ENV = ["RUSTC_WRAPPER", "CANON_RUSTC_V3_ARTIFACT_DIR", "CARGO_TARGET_DIR"]
RECEIPT_FIELD = "receipt_hash"


class CommandFailed(Exception):
    def __init__(self, code: int) -> None:
        self.code = code


def tail(text: str, limit: int = 1600) -> str:
    return text if len(text) <= limit else text[-limit:]


def version(path: str | None) -> str | None:
    if not path:
        return None
    try:
        result = subprocess.run([path, "--version"], text=True, capture_output=True, check=False)
    except OSError:
        return None
    return (result.stdout or result.stderr).strip() or None


def tool(name: str) -> dict[str, object]:
    path = shutil.which(name)
    return {"available": bool(path), "path": path, "version": version(path)}


def write_report(path: pathlib.Path | None, report: dict[str, object]) -> None:
    if not path:
        return
    path = path if path.is_absolute() else ROOT / path
    path.parent.mkdir(parents=True, exist_ok=True)
    report[RECEIPT_FIELD] = receipt_hash(report)
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def canonical_hash(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def receipt_hash(report: dict[str, object]) -> str:
    stable = dict(report)
    stable.pop(RECEIPT_FIELD, None)
    return canonical_hash(stable)


def run(command: list[str], report: dict[str, object], name: str, **kwargs: object) -> None:
    print("+", " ".join(command))
    started = time.perf_counter()
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False, **kwargs)
    elapsed_ms = round((time.perf_counter() - started) * 1000, 3)
    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, end="", file=sys.stderr)
    report["commands"].append(
        {
            "name": name,
            "argv": command,
            "elapsed_ms": elapsed_ms,
            "returncode": result.returncode,
            "stdout_tail": tail(result.stdout),
            "stderr_tail": tail(result.stderr),
        }
    )
    if result.returncode:
        raise CommandFailed(result.returncode)


def cargo_missing(report: dict[str, object], require: bool) -> bool:
    if shutil.which("cargo"):
        return False
    print("cargo: unavailable")
    if require:
        report["cargo_validation"] = "required_unavailable"
        report["skip_reason"] = "cargo_unavailable"
        raise CommandFailed(127)
    report["cargo_validation"] = "skipped"
    report["skip_reason"] = "cargo_unavailable"
    return True


def fixture_env(root: str, wrapper: bool) -> dict[str, str]:
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(ROOT / root / "target")
    if wrapper:
        env["CANON_RUSTC_V3_ARTIFACT_DIR"] = root
        env["RUSTC_WRAPPER"] = str(ROOT / "target/debug/canon-rustc-v3")
    else:
        env.pop("CANON_RUSTC_V3_ARTIFACT_DIR", None)
        env.pop("RUSTC_WRAPPER", None)
    return env


def graph_metrics(root: str) -> dict[str, object]:
    base = ROOT / root
    graphs = sorted(base.rglob("graph.json"))
    nodes = edges = bytes_total = 0
    for graph in graphs:
        data = json.loads(graph.read_text(encoding="utf-8"))
        nodes += len(data.get("nodes", {}))
        edges += len(data.get("edges", []))
        bytes_total += graph.stat().st_size
    fingerprint = graph_fingerprint(base, graphs)
    return {
        "root": root,
        "graph_count": len(graphs),
        "node_count": nodes,
        "edge_count": edges,
        "bytes": bytes_total,
        "fingerprint": fingerprint,
    }


def replay_graph(path: pathlib.Path) -> dict[str, object]:
    graph = json.loads(path.read_text(encoding="utf-8"))
    graph.get("meta", {}).pop("captured_at_ms", None)
    return graph


def graph_fingerprint(root: pathlib.Path, graphs: list[pathlib.Path]) -> str | None:
    if not graphs:
        return None
    stable = []
    for graph in graphs:
        stable.append({"path": str(graph.relative_to(root)), "graph": replay_graph(graph)})
    return canonical_hash(stable)


def command_elapsed(report: dict[str, object], name: str) -> float | None:
    for command in report["commands"]:
        if command.get("name") == name:
            return float(command["elapsed_ms"])
    return None


def record_performance(report: dict[str, object]) -> None:
    baseline = command_elapsed(report, "fixture_baseline")
    wrapped = command_elapsed(report, "fixture_wrapped_left")
    ratio = None if not baseline or not wrapped else round(wrapped / baseline, 3)
    report["performance"] = {
        "baseline_ms": baseline,
        "wrapped_ms": wrapped,
        "overhead_ratio": ratio,
    }


def new_report(args: argparse.Namespace) -> dict[str, object]:
    return {
        "schema_version": 1,
        "status": "running",
        "mode": {"require_cargo": args.require_cargo, "static_only": args.static_only},
        "fixture": str(args.fixture),
        "tool_availability": {name: tool(name) for name in ["cargo", "rustc", "rustup"]},
        "environment": {name: os.environ.get(name) for name in REPORT_ENV},
        "cargo_validation": "not_started",
        "commands": [],
        "graphs": [],
        "performance": {"baseline_ms": None, "wrapped_ms": None, "overhead_ratio": None},
        "replay_comparison": "not_run",
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--require-cargo", action="store_true")
    parser.add_argument("--static-only", action="store_true")
    parser.add_argument("--fixture", type=pathlib.Path, default=DEFAULT_FIXTURE)
    parser.add_argument("--report", type=pathlib.Path)
    args = parser.parse_args()
    report = new_report(args)

    try:
        run([sys.executable, "validation/semantic_spine.py"], report, "semantic_spine_static")
        if args.static_only:
            report["cargo_validation"] = "static_only"
            report["status"] = "pass"
            return 0
        if cargo_missing(report, args.require_cargo):
            report["status"] = "pass_with_skip"
            return 0

        report["cargo_validation"] = "running"
        run(["cargo", "check"], report, "cargo_check")
        run(["cargo", "test"], report, "cargo_test")
        run(["cargo", "build"], report, "cargo_build")

        fixture = str(args.fixture)
        baseline = "state/rustc-baseline"
        left = "state/rustc-test-1"
        right = "state/rustc-test-2"
        shutil.rmtree(ROOT / baseline, ignore_errors=True)
        shutil.rmtree(ROOT / left, ignore_errors=True)
        shutil.rmtree(ROOT / right, ignore_errors=True)
        run(["cargo", "check", "--manifest-path", fixture], report, "fixture_baseline", env=fixture_env(baseline, False))
        for name, root in [("fixture_left", left), ("fixture_right", right)]:
            run(["cargo", "check", "--manifest-path", fixture], report, f"fixture_wrapped_{name.split('_')[-1]}", env=fixture_env(root, True))
            report["graphs"].append(graph_metrics(root))
        run(
            [sys.executable, "validation/semantic_spine.py", "--graph", left, "--graph", right, "--compare", left, right],
            report,
            "semantic_spine_graph_replay",
        )
        record_performance(report)
        report["cargo_validation"] = "ran"
        report["replay_comparison"] = "passed"
        report["status"] = "pass"
        return 0
    except CommandFailed as error:
        report["status"] = "fail"
        return error.code or 1
    finally:
        write_report(args.report, report)


if __name__ == "__main__":
    raise SystemExit(main())
