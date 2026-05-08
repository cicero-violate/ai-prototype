#!/usr/bin/env python3
"""Report deterministic auto-refactor surfaces from a canon graph.json.

The report is evidence-only. It names split, merge, and canonicalize surfaces
from graph relations emitted by canon-rustc-v3. It does not edit source.
"""

from __future__ import annotations

import argparse
import json
import pathlib
from collections import defaultdict
from typing import Any


REPORT_SCHEMA_VERSION = 1
SIMILAR_PREFIX = "similar::"
PHASE_PREFIX = "phase::"
PROVIDER_PREFIX = "provider::"


def read_json(path: pathlib.Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def write_json(path: pathlib.Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def parse_similarity(to_value: str) -> tuple[float, str] | None:
    if not to_value.startswith(SIMILAR_PREFIX):
        return None
    rest = to_value[len(SIMILAR_PREFIX) :]
    score, sep, target = rest.partition("::")
    if not sep or not target:
        return None
    try:
        return float(score), target
    except ValueError:
        return None


def grouped_edges(graph: dict[str, Any]) -> dict[str, list[dict[str, Any]]]:
    groups: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for edge in graph.get("edges", []):
        relation = edge.get("relation")
        if isinstance(relation, str):
            groups[relation].append(edge)
    return groups


def call_fanout(groups: dict[str, list[dict[str, Any]]]) -> dict[str, int]:
    targets: dict[str, set[str]] = defaultdict(set)
    for edge in groups.get("call", []):
        source = edge.get("from")
        target = edge.get("to")
        if isinstance(source, str) and isinstance(target, str):
            targets[source].add(target)
    return {source: len(targets) for source, targets in targets.items()}


def phases_by_fn(groups: dict[str, list[dict[str, Any]]]) -> dict[str, list[str]]:
    phases: dict[str, set[str]] = defaultdict(set)
    for edge in groups.get("phase", []):
        source = edge.get("from")
        target = edge.get("to")
        if isinstance(source, str) and isinstance(target, str) and target.startswith(PHASE_PREFIX):
            phases[source].add(target[len(PHASE_PREFIX) :])
    return {source: sorted(values) for source, values in phases.items()}


def providers_by_fn(groups: dict[str, list[dict[str, Any]]]) -> dict[str, list[str]]:
    providers: dict[str, set[str]] = defaultdict(set)
    for edge in groups.get("provider", []):
        source = edge.get("from")
        target = edge.get("to")
        if isinstance(source, str) and isinstance(target, str) and target.startswith(PROVIDER_PREFIX):
            providers[source].add(target[len(PROVIDER_PREFIX) :])
    return {source: sorted(values) for source, values in providers.items()}


def split_surface(groups: dict[str, list[dict[str, Any]]]) -> list[dict[str, Any]]:
    fanout = call_fanout(groups)
    phases = phases_by_fn(groups)
    rows = []
    for fn_path, count in fanout.items():
        fn_phases = phases.get(fn_path, [])
        if count > 30 and len(fn_phases) >= 2:
            rows.append(
                {
                    "fn": fn_path,
                    "fan_out": count,
                    "phases": fn_phases,
                    "rank": count * len(fn_phases),
                    "recommended_split_boundaries": [f"phase::{phase}" for phase in fn_phases],
                }
            )
    return sorted(rows, key=lambda row: (-row["rank"], row["fn"]))


def merge_surface(groups: dict[str, list[dict[str, Any]]]) -> list[dict[str, Any]]:
    pairs = []
    seen = set()
    for edge in groups.get("similar", []):
        source = edge.get("from")
        target_raw = edge.get("to")
        if not isinstance(source, str) or not isinstance(target_raw, str):
            continue
        parsed = parse_similarity(target_raw)
        if not parsed:
            continue
        score, target = parsed
        key = tuple(sorted([source, target]))
        if key in seen:
            continue
        seen.add(key)
        module = source.rsplit("::", 1)[0] if "::" in source else source
        pairs.append(
            {
                "module": module,
                "members": [source, target],
                "similarity": score,
                "recommended_canonical_name": key[0].rsplit("::", 1)[-1],
            }
        )
    return sorted(pairs, key=lambda row: (-row["similarity"], row["module"], row["members"]))


def canonicalize_surface(groups: dict[str, list[dict[str, Any]]]) -> list[dict[str, Any]]:
    providers = providers_by_fn(groups)
    rows = []
    seen = set()
    for edge in groups.get("similar", []):
        source = edge.get("from")
        target_raw = edge.get("to")
        if not isinstance(source, str) or not isinstance(target_raw, str):
            continue
        parsed = parse_similarity(target_raw)
        if not parsed:
            continue
        score, target = parsed
        source_providers = providers.get(source, [])
        target_providers = providers.get(target, [])
        if source_providers == target_providers:
            continue
        key = tuple(sorted([source, target]))
        if key in seen:
            continue
        seen.add(key)
        rows.append(
            {
                "pair": [source, target],
                "similarity": score,
                "providers": {source: source_providers, target: target_providers},
                "recommended_trait_boundary": common_suffix_boundary(source, target),
            }
        )
    return sorted(rows, key=lambda row: (-row["similarity"], row["pair"]))


def common_suffix_boundary(left: str, right: str) -> str:
    left_name = left.rsplit("::", 1)[-1]
    right_name = right.rsplit("::", 1)[-1]
    if left_name == right_name:
        return f"trait::{left_name}ProviderBoundary"
    return "trait::ProviderCanonicalBoundary"


def report(graph: dict[str, Any]) -> dict[str, Any]:
    groups = grouped_edges(graph)
    return {
        "schema_version": REPORT_SCHEMA_VERSION,
        "graph_schema_version": graph.get("meta", {}).get("schema_version"),
        "crate_name": graph.get("meta", {}).get("crate_name", "unknown"),
        "split_surface": split_surface(groups),
        "merge_surface": merge_surface(groups),
        "canonicalize_surface": canonicalize_surface(groups),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("graph", type=pathlib.Path, help="Path to graph.json")
    parser.add_argument("--out", type=pathlib.Path, help="Optional output report path")
    args = parser.parse_args()

    result = report(read_json(args.graph))
    if args.out:
        write_json(args.out, result)
    else:
        print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())