#!/usr/bin/env python3
"""Deterministic scale probe for the semantic-delta gate."""

from __future__ import annotations

import argparse
import json
import pathlib
import tempfile
import time

import semantic_delta


RISK_TARGET = "core::ptr::write"
GRAPH_SCHEMA_VERSION = 16
RECEIPT_SCHEMA_VERSION = 1


def stable_hash(value: object) -> str:
    return semantic_delta.canonical_hash(value)


def node(index: int) -> dict[str, object]:
    path = f"scale::f{index:06d}"
    return {
        "def": {"col": 1, "file": f"src/mod_{index // 1000:03d}.rs", "hi": index + 1, "line": index + 1, "lo": index},
        "def_id": f"fn-{index:06d}",
        "kind": "fn",
        "path": path,
    }


def graph(size: int, fanout: int, risks: int) -> dict[str, object]:
    nodes = {f"scale::f{index:06d}": node(index) for index in range(size)}
    intents = {path: "pure" for path in nodes}
    edges = []

    for index in range(size):
        source = f"scale::f{index:06d}"
        for offset in range(1, fanout + 1):
            target = f"scale::f{(index + offset) % size:06d}"
            edges.append({"from": source, "relation": "call", "to": target})

    for index in range(risks):
        source = f"scale::f{index:06d}"
        intents[source] = "unsafe"
        edges.append({"from": source, "relation": "unsafe", "to": RISK_TARGET})

    meta = {
        "captured_at_ms": risks,
        "crate_name": "semantic_scale_probe",
        "edge_count": len(edges),
        "node_count": len(nodes),
        "schema_version": GRAPH_SCHEMA_VERSION,
    }
    graph = {"edges": sorted(edges, key=lambda row: (row["relation"], row["from"], row["to"])), "intents": intents, "meta": meta, "nodes": nodes}
    meta["graph_hash"] = stable_hash({"edges": graph["edges"], "nodes": nodes})
    meta["intent_hash"] = stable_hash(intents)
    meta["risk_hash"] = stable_hash([edge for edge in graph["edges"] if edge["relation"] in semantic_delta.RISK_WEIGHT])
    meta["receipt_hash"] = stable_hash(
        {
            "schema_version": RECEIPT_SCHEMA_VERSION,
            "graph_schema_version": GRAPH_SCHEMA_VERSION,
            "crate_name": meta["crate_name"],
            "node_count": meta["node_count"],
            "edge_count": meta["edge_count"],
            "graph_hash": meta["graph_hash"],
            "intent_hash": meta["intent_hash"],
            "risk_hash": meta["risk_hash"],
        }
    )
    return graph


def write_json(path: pathlib.Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def scale_report(args: argparse.Namespace) -> dict[str, object]:
    with tempfile.TemporaryDirectory(prefix="semantic-scale-") as tmp:
        tmp_path = pathlib.Path(tmp)
        old_path = tmp_path / "old" / "graph.json"
        new_path = tmp_path / "new" / "graph.json"
        old = graph(args.nodes, args.fanout, 0)
        new = graph(args.nodes, args.fanout, args.risk_additions)
        write_json(old_path, old)
        write_json(new_path, new)

        started = time.perf_counter_ns()
        delta = semantic_delta.build_report(old, new, False, args.top)
        elapsed_ms = round((time.perf_counter_ns() - started) / 1_000_000, 3)

    report = {
        "schema_version": 1,
        "status": "pass" if elapsed_ms <= args.threshold_ms else "fail",
        "node_count": args.nodes,
        "edge_count": len(new["edges"]),
        "fanout": args.fanout,
        "risk_additions": args.risk_additions,
        "risk_delta_score": delta["risk_delta_score"],
        "changed_file_count": len(delta["changed_files"]),
        "prompt_fact_count": len(delta["prompt_facts"]),
        "gate_status": delta["gate_decision"]["status"],
        "gate_reason": delta["gate_decision"]["reason"],
        "elapsed_ms": elapsed_ms,
        "threshold_ms": args.threshold_ms,
        "old_graph_hash": delta["old_graph_hash"],
        "new_graph_hash": delta["new_graph_hash"],
    }
    report["report_hash"] = stable_hash(report)
    return report


def positive_int(value: str) -> int:
    parsed = int(value)
    if parsed <= 0:
        raise argparse.ArgumentTypeError("must be positive")
    return parsed


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--nodes", type=positive_int, default=5000)
    parser.add_argument("--fanout", type=positive_int, default=2)
    parser.add_argument("--risk-additions", type=positive_int, default=100)
    parser.add_argument("--threshold-ms", type=float, default=2000)
    parser.add_argument("--top", type=positive_int, default=8)
    parser.add_argument("--report", type=pathlib.Path, default=pathlib.Path("validation/semantic_scale_report.eval.json"))
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.risk_additions > args.nodes:
        raise SystemExit("--risk-additions must be <= --nodes")
    report = scale_report(args)
    write_json(args.report, report)
    print(f"semantic scale probe: {report['status']} elapsed_ms={report['elapsed_ms']} threshold_ms={report['threshold_ms']}")
    return 0 if report["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())