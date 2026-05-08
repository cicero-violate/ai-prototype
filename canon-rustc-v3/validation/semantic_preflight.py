#!/usr/bin/env python3
"""Portable preflight for semantic witness validation."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import pathlib
import re
import shutil
import subprocess
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
RISK_RELATIONS = {"mut", "io", "unsafe", "panic", "alloc", "similar", "phase"}
HASH_FIELDS = {"graph_hash", "intent_hash", "risk_hash", "receipt_hash"}
VOLATILE_FIELDS = ["meta.captured_at_ms"]
GRAPH_SCHEMA_VERSION = 16
RECEIPT_SCHEMA_VERSION = 1


def canonical_hash(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def read_json(path: pathlib.Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def write_json(path: pathlib.Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def run(argv: list[str], cwd: pathlib.Path = ROOT) -> subprocess.CompletedProcess[str]:
    return subprocess.run(argv, cwd=cwd, text=True, capture_output=True, check=False)


def first_line(text: str) -> str | None:
    line = text.strip().splitlines()
    return line[0] if line else None


def tool_receipt(name: str) -> dict[str, Any]:
    path = shutil.which(name)
    receipt: dict[str, Any] = {"available": bool(path), "path": path, "version": None}
    if not path:
        return receipt
    result = run([path, "--version"])
    receipt.update({"returncode": result.returncode, "version": first_line(result.stdout) or first_line(result.stderr)})
    return receipt


def rust_toolchain_channel() -> str | None:
    path = ROOT / "rust-toolchain.toml"
    if not path.exists():
        return None
    text = path.read_text(encoding="utf-8")
    match = re.search(r'^\s*channel\s*=\s*"([^"]+)"', text, re.MULTILINE)
    return match.group(1) if match else None


def submodule_initialized(path: pathlib.Path) -> bool:
    if not path.exists():
        return False
    if not (path / ".git").exists() and not any(path.iterdir()):
        return False
    status = run(["git", "submodule", "status", "--", str(path.relative_to(ROOT))])
    if status.returncode != 0:
        return False
    line = status.stdout.strip()
    return bool(line) and not line.startswith("-")


def git_head() -> str | None:
    result = run(["git", "rev-parse", "HEAD"])
    if result.returncode != 0:
        return None
    return result.stdout.strip()


def graph_paths(root: pathlib.Path | None) -> list[pathlib.Path]:
    if not root:
        return []
    if root.name == "graph.json" and root.exists():
        return [root]
    if not root.exists():
        return []
    return sorted(root.rglob("graph.json"))


def normalized_graph(graph: dict[str, Any]) -> dict[str, Any]:
    view = copy.deepcopy(graph)
    view.get("meta", {}).pop("captured_at_ms", None)
    return view


def risk_edges(graph: dict[str, Any]) -> list[dict[str, Any]]:
    return [edge for edge in graph.get("edges", []) if edge.get("relation") in RISK_RELATIONS]


def receipt_envelope(
    crate_name: str,
    node_count: int,
    edge_count: int,
    graph_hash: str,
    intent_hash: str,
    risk_hash: str,
) -> dict[str, Any]:
    return {
        "schema_version": RECEIPT_SCHEMA_VERSION,
        "graph_schema_version": GRAPH_SCHEMA_VERSION,
        "crate_name": crate_name,
        "node_count": node_count,
        "edge_count": edge_count,
        "graph_hash": graph_hash,
        "intent_hash": intent_hash,
        "risk_hash": risk_hash,
    }


def graph_receipt(path: pathlib.Path) -> dict[str, Any]:
    graph = read_json(path)
    meta = graph.get("meta", {})
    recomputed = {
        "graph_hash": canonical_hash({"edges": graph.get("edges", []), "nodes": graph.get("nodes", {})}),
        "intent_hash": canonical_hash(graph.get("intents", {})),
        "risk_hash": canonical_hash(risk_edges(graph)),
    }
    crate_name = meta.get("crate_name", "unknown")
    nodes = graph.get("nodes", {})
    edges = graph.get("edges", [])
    recomputed["receipt_hash"] = canonical_hash(
        receipt_envelope(
            crate_name,
            len(nodes) if isinstance(nodes, dict) else int(meta.get("node_count") or 0),
            len(edges) if isinstance(edges, list) else int(meta.get("edge_count") or 0),
            recomputed["graph_hash"],
            recomputed["intent_hash"],
            recomputed["risk_hash"],
        )
    )
    return {
        "path": str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path),
        "crate_name": crate_name,
        "normalized_hash": canonical_hash(normalized_graph(graph)),
        "declared_hashes_present": sorted(HASH_FIELDS & set(meta)),
        "declared_hashes_complete": HASH_FIELDS <= set(meta) and all(meta.get(field) for field in HASH_FIELDS),
        "recomputed_hashes": recomputed,
        "declared_hashes": {field: meta.get(field) for field in sorted(HASH_FIELDS)},
        "declared_match_canonical": {field: meta.get(field) == recomputed[field] for field in sorted(HASH_FIELDS)},
    }


def replay_receipts(left: pathlib.Path | None, right: pathlib.Path | None) -> dict[str, Any]:
    left_paths = graph_paths(left)
    right_paths = graph_paths(right)
    left_receipts = [graph_receipt(path) for path in left_paths]
    right_receipts = [graph_receipt(path) for path in right_paths]
    if not left_paths and not right_paths:
        return {"comparison": "not_run", "left_graphs": [], "right_graphs": [], "graph_roots_comparable": False}
    if left_paths and not right_paths:
        return {"comparison": "not_run", "left_graphs": left_receipts, "right_graphs": [], "graph_roots_comparable": False}
    left_by_crate = {receipt["crate_name"]: receipt for receipt in left_receipts}
    right_by_crate = {receipt["crate_name"]: receipt for receipt in right_receipts}
    comparable = set(left_by_crate) == set(right_by_crate)
    pairs = []
    passed = comparable
    for crate in sorted(set(left_by_crate) | set(right_by_crate)):
        left_hash = (left_by_crate.get(crate) or {}).get("normalized_hash")
        right_hash = (right_by_crate.get(crate) or {}).get("normalized_hash")
        equal = bool(left_hash) and left_hash == right_hash
        passed = passed and equal
        pairs.append({"crate_name": crate, "left_hash": left_hash, "right_hash": right_hash, "equal": equal})
    return {
        "comparison": "passed" if passed else "failed",
        "left_graphs": left_receipts,
        "right_graphs": right_receipts,
        "graph_roots_comparable": comparable,
        "pairs": pairs,
    }


def missing_signal(kind: str, detail: str) -> dict[str, str]:
    return {"kind": kind, "detail": detail}


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    tools = {name: tool_receipt(name) for name in ["cargo", "rustc", "rustup"]}
    channel = rust_toolchain_channel()
    replay = replay_receipts(args.artifact_root, args.compare_artifact_root)
    missing: list[dict[str, str]] = []
    failures: list[dict[str, str]] = []

    for name, receipt in tools.items():
        if not receipt["available"]:
            missing.append(missing_signal(f"{name}_unavailable", f"{name} is not available on PATH"))
    if channel == "nightly":
        missing.append(missing_signal("unpinned_nightly", "rust-toolchain.toml uses bare nightly"))
    if not (ROOT / "Cargo.lock").exists():
        missing.append(missing_signal("cargo_lock_absent", "Cargo.lock is absent"))
    if not submodule_initialized(ROOT / "vendor" / "rust-source"):
        missing.append(missing_signal("rust_source_uninitialized", "vendor/rust-source is not initialized"))
    if replay["comparison"] == "not_run":
        missing.append(missing_signal("live_replay_not_run", "no comparable graph artifact roots were supplied"))
    if args.require_live_replay and replay["comparison"] != "passed":
        failures.append(missing_signal("live_replay_required", "required live replay comparison did not pass"))

    live_validation = replay["comparison"] == "passed" and bool(replay.get("left_graphs")) and replay.get("graph_roots_comparable")
    status = "fail" if failures else ("pass_with_skip" if missing else "pass")
    report: dict[str, Any] = {
        "schema_version": 1,
        "status": status,
        "mode": "semantic_preflight",
        "tool_availability": tools,
        "repo_state": {
            "head_commit": git_head(),
            "cargo_lock_present": (ROOT / "Cargo.lock").exists(),
            "rust_toolchain_channel": channel,
            "rust_toolchain_drift_risk": channel == "nightly",
            "vendor_rust_source_initialized": submodule_initialized(ROOT / "vendor" / "rust-source"),
        },
        "determinism_inputs": {
            "volatile_fields_removed": VOLATILE_FIELDS,
            "artifact_root": str(args.artifact_root) if args.artifact_root else None,
            "compare_artifact_root": str(args.compare_artifact_root) if args.compare_artifact_root else None,
            "require_live_replay": args.require_live_replay,
            "replay_comparison": replay["comparison"],
            "graph_roots_comparable": replay["graph_roots_comparable"],
            "normalized_replay": replay,
        },
        "live_validation": live_validation,
        "missing_signals": missing,
        "failures": failures,
    }
    report["receipt_hash"] = canonical_hash({key: value for key, value in report.items() if key != "receipt_hash"})
    return report


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--artifact-root", type=pathlib.Path)
    parser.add_argument("--compare-artifact-root", type=pathlib.Path)
    parser.add_argument("--require-live-replay", action="store_true")
    parser.add_argument("--report", type=pathlib.Path, default=pathlib.Path("validation/semantic_preflight_report.eval.json"))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = build_report(args)
    write_json(args.report, report)
    print(f"semantic preflight: {report['status']} missing={len(report['missing_signals'])} failures={len(report['failures'])}")
    return 1 if report["status"] == "fail" else 0


if __name__ == "__main__":
    raise SystemExit(main())