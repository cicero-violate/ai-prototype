#!/usr/bin/env python3
"""Smoke test for deterministic auto_refactor_surface output."""

from __future__ import annotations

import json
import pathlib
import sys

import auto_refactor_surface


ROOT = pathlib.Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "validation" / "fixtures" / "auto_refactor_surface" / "graph.json"
EXPECTED = ROOT / "validation" / "fixtures" / "auto_refactor_surface" / "expected-report.json"


def main() -> int:
    graph = auto_refactor_surface.read_json(FIXTURE)
    actual = auto_refactor_surface.report(graph)
    expected = auto_refactor_surface.read_json(EXPECTED)
    if actual != expected:
        print(json.dumps({"actual": actual, "expected": expected}, indent=2, sort_keys=True))
        return 1

    assert actual["graph_schema_version"] == 16
    assert actual["crate_name"] == "demo"
    assert actual["split_surface"][0]["fn"] == "demo::wide"
    assert actual["split_surface"][0]["fan_out"] == 31
    assert actual["merge_surface"][0]["similarity"] == 0.9
    assert actual["canonicalize_surface"][0]["recommended_trait_boundary"] == "trait::chatProviderBoundary"
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
