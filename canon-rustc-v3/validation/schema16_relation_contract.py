#!/usr/bin/env python3
"""Contract check for schema-16 relation vocabulary and fixture coverage."""

from __future__ import annotations

import json
import pathlib
import re


ROOT = pathlib.Path(__file__).resolve().parents[1]
FACTS = ROOT / "src" / "facts.rs"
AUTO_REFACTOR_FIXTURE = ROOT / "validation" / "fixtures" / "auto_refactor_surface" / "graph.json"
EXPECTED_ALLOWED = [
    "call",
    "impl",
    "mut",
    "io",
    "unsafe",
    "panic",
    "alloc",
    "use",
    "similar",
    "phase",
    "provider",
]
EXPECTED_RISK = ["mut", "io", "unsafe", "panic", "alloc", "similar", "phase"]
EXPECTED_AUTO_REFACTOR_RELATIONS = ["call", "phase", "provider", "similar"]


def rust_array(name: str) -> list[str]:
    source = FACTS.read_text(encoding="utf-8")
    match = re.search(rf"pub const {name}: &\[&str\] = &\[(.*?)\];", source, re.S)
    if not match:
        raise AssertionError(f"missing Rust vocabulary array: {name}")
    return re.findall(r'"([^"]+)"', match.group(1))


def fixture_relations(path: pathlib.Path) -> list[str]:
    graph = json.loads(path.read_text(encoding="utf-8"))
    assert graph["meta"]["schema_version"] == 16
    relations = sorted({edge["relation"] for edge in graph.get("edges", [])})
    unexpected = sorted(set(relations) - set(EXPECTED_ALLOWED))
    assert not unexpected, f"fixture emits non-canonical relations: {unexpected}"
    return relations


def main() -> int:
    assert rust_array("EDGE_RELATIONS") == EXPECTED_ALLOWED
    assert rust_array("RISK_RELATIONS") == EXPECTED_RISK
    assert fixture_relations(AUTO_REFACTOR_FIXTURE) == EXPECTED_AUTO_REFACTOR_RELATIONS
    assert set(EXPECTED_RISK).issubset(EXPECTED_ALLOWED)
    assert "provider" not in EXPECTED_RISK
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
