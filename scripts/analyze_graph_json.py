#!/usr/bin/env python3
"""Print deterministic evidence from a Canon graph.json artifact.

This script is intentionally read-only. It summarizes the graph metadata and
counts compiled P5 domain nodes without mutating graph state.
"""

from __future__ import annotations

import json
import sys
from collections import Counter
from pathlib import Path
from typing import Any


REQUIRED_TOP_LEVEL_KEYS = {"nodes", "edges", "intents", "meta"}
REQUIRED_META_KEYS = {"schema_version", "graph_hash", "receipt_hash", "risk_hash"}


def fail(message: str) -> int:
    print(f"error: {message}", file=sys.stderr)
    return 1


def load_graph(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, dict):
        raise ValueError("graph root must be an object")
    return data


def node_iter(nodes: Any):
    if isinstance(nodes, dict):
        for name, node in nodes.items():
            if isinstance(node, dict):
                yield str(name), node
            else:
                yield str(name), {}
    elif isinstance(nodes, list):
        for index, node in enumerate(nodes):
            if isinstance(node, dict):
                name = node.get("name") or node.get("symbol") or node.get("id") or str(index)
                yield str(name), node
            else:
                yield str(index), {}
    else:
        raise ValueError("nodes must be an object or array")


def count_entries(value: Any, name: str) -> int:
    if isinstance(value, (dict, list)):
        return len(value)
    raise ValueError(f"{name} must be an object or array")


def node_kind(node: dict[str, Any]) -> str:
    for key in ("kind", "node_kind", "type"):
        value = node.get(key)
        if isinstance(value, str) and value:
            return value
    return "unknown"


def is_domain_node(name: str, node: dict[str, Any]) -> bool:
    haystack = " ".join(
        str(part)
        for part in (
            name,
            node.get("name", ""),
            node.get("symbol", ""),
            node.get("path", ""),
            node.get("module", ""),
            node.get("crate", ""),
        )
    )
    return "domain::" in haystack or "src/domain" in haystack


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        return fail("usage: analyze_graph_json.py <state/rustc/ai/graph.json>")

    path = Path(argv[1])
    try:
        graph = load_graph(path)
        missing = sorted(REQUIRED_TOP_LEVEL_KEYS - set(graph.keys()))
        if missing:
            return fail("missing top-level graph keys: " + ", ".join(missing))

        meta = graph["meta"]
        if not isinstance(meta, dict):
            return fail("meta must be an object")
        missing_meta = sorted(REQUIRED_META_KEYS - set(meta.keys()))
        if missing_meta:
            return fail("missing graph meta keys: " + ", ".join(missing_meta))

        nodes = list(node_iter(graph["nodes"]))
        kind_counts = Counter(node_kind(node) for _, node in nodes)
        domain_matches = sorted(name for name, node in nodes if is_domain_node(name, node))

        print(f"schema_version: {meta['schema_version']}")
        print(f"graph_hash: {meta['graph_hash']}")
        print(f"receipt_hash: {meta['receipt_hash']}")
        print(f"risk_hash: {meta['risk_hash']}")
        print(f"node_count: {len(nodes)}")
        print(f"edge_count: {count_entries(graph['edges'], 'edges')}")
        print(f"intent_count: {count_entries(graph['intents'], 'intents')}")
        print("node_kind_counts:")
        for kind, count in sorted(kind_counts.items()):
            print(f"  {kind}: {count}")
        print(f"p5_domain_node_match_count: {len(domain_matches)}")
        if domain_matches:
            print("p5_domain_node_matches:")
            for name in domain_matches[:50]:
                print(f"  {name}")
            if len(domain_matches) > 50:
                print(f"  ... {len(domain_matches) - 50} more")
        return 0
    except json.JSONDecodeError as exc:
        return fail(f"invalid JSON: {exc}")
    except OSError as exc:
        return fail(f"could not read graph: {exc}")
    except ValueError as exc:
        return fail(str(exc))


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
