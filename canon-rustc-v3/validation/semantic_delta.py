#!/usr/bin/env python3
"""Deterministic semantic-delta gate for emitted witness graphs."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib


RISK_WEIGHT = {"unsafe": 5, "panic": 4, "io": 3, "mut": 2, "alloc": 1}
INTENT_WEIGHT = {"unsafe": 5, "boundary": 4, "io": 3, "mutation": 2, "validation": 2, "orchestration": 1, "pure": 0}
RECEIPT_FIELD = "receipt_hash"


def load_json(path: pathlib.Path) -> dict:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def canonical_hash(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def valid_receipt_hash(report: dict) -> bool:
    receipt = report.get(RECEIPT_FIELD)
    stable = dict(report)
    stable.pop(RECEIPT_FIELD, None)
    return isinstance(receipt, str) and receipt == canonical_hash(stable)


def graph_hash(graph: dict, field: str) -> str:
    value = graph.get("meta", {}).get(field)
    return value if isinstance(value, str) and value else canonical_hash(graph.get(field.removesuffix("_hash"), {}))


def nodes(graph: dict) -> dict[str, dict]:
    out = {}
    for key, node in graph.get("nodes", {}).items():
        path = node.get("path") or key
        out[path] = {"kind": node.get("kind"), "path": path, "file": node.get("def", {}).get("file")}
    return out


def edges(graph: dict) -> set[tuple[str, str, str]]:
    return {
        (edge.get("relation"), edge.get("from"), edge.get("to"))
        for edge in graph.get("edges", [])
        if edge.get("relation") and edge.get("from") and edge.get("to")
    }


def as_edge(items: set[tuple[str, str, str]]) -> list[dict]:
    return [{"relation": relation, "from": source, "to": target} for relation, source, target in sorted(items)]


def as_node(items: dict[str, dict], keys: set[str]) -> list[dict]:
    return [items[key] for key in sorted(keys)]


def intent_changes(old: dict, new: dict) -> list[dict]:
    old_intents = old.get("intents", {})
    new_intents = new.get("intents", {})
    changes = []
    for path in sorted(set(old_intents) | set(new_intents)):
        before = old_intents.get(path)
        after = new_intents.get(path)
        if before != after:
            changes.append({"path": path, "old": before, "new": after})
    return changes


def edge_file(edge: tuple[str, str, str], old_nodes: dict[str, dict], new_nodes: dict[str, dict]) -> str:
    _, source, target = edge
    for path in [source, target]:
        node = new_nodes.get(path) or old_nodes.get(path)
        if node and node.get("file"):
            return node["file"]
    return "unknown"


def changed_files(old_nodes: dict[str, dict], new_nodes: dict[str, dict], added: set, removed: set, node_keys: set[str]) -> list[str]:
    files = {edge_file(edge, old_nodes, new_nodes) for edge in added | removed}
    for key in node_keys:
        node = new_nodes.get(key) or old_nodes.get(key)
        if node and node.get("file"):
            files.add(node["file"])
    return sorted(files)


def leverage_rank(files: list[str], added: set, removed: set, changes: list[dict], old_nodes: dict, new_nodes: dict) -> list[dict]:
    rows = {path: {"file": path, "risk_weight": 0, "intent_weight": 0, "edge_count": 0, "node_count": 0} for path in files}
    for edge in added | removed:
        relation = edge[0]
        file = edge_file(edge, old_nodes, new_nodes)
        row = rows.setdefault(file, {"file": file, "risk_weight": 0, "intent_weight": 0, "edge_count": 0, "node_count": 0})
        row["risk_weight"] += RISK_WEIGHT.get(relation, 0)
        row["edge_count"] += 1
    for change in changes:
        file = (new_nodes.get(change["path"]) or old_nodes.get(change["path"]) or {}).get("file", "unknown")
        row = rows.setdefault(file, {"file": file, "risk_weight": 0, "intent_weight": 0, "edge_count": 0, "node_count": 0})
        row["intent_weight"] += max(INTENT_WEIGHT.get(change.get("old"), 0), INTENT_WEIGHT.get(change.get("new"), 0))
    for node in new_nodes.values():
        if node.get("file") in rows:
            rows[node["file"]]["node_count"] += 1
    for row in rows.values():
        row["leverage"] = round(row["risk_weight"] + row["intent_weight"] + row["edge_count"] + row["node_count"] * 0.25, 3)
    return sorted(rows.values(), key=lambda row: (-row["leverage"], row["file"]))


def prompt_facts(added: set, changes: list[dict], old_nodes: dict, new_nodes: dict, limit: int) -> list[dict]:
    facts = []
    for relation, source, target in sorted(added, key=lambda edge: (-RISK_WEIGHT.get(edge[0], 0), edge)):
        facts.append({
            "kind": "edge_added",
            "risk": RISK_WEIGHT.get(relation, 0),
            "file": edge_file((relation, source, target), old_nodes, new_nodes),
            "summary": f"{relation}: {source} -> {target}",
        })
    for change in changes:
        file = (new_nodes.get(change["path"]) or old_nodes.get(change["path"]) or {}).get("file", "unknown")
        facts.append({
            "kind": "intent_changed",
            "risk": max(INTENT_WEIGHT.get(change.get("old"), 0), INTENT_WEIGHT.get(change.get("new"), 0)),
            "file": file,
            "summary": f"intent: {change['path']} {change.get('old')} -> {change.get('new')}",
        })
    return sorted(facts, key=lambda fact: (-fact["risk"], fact["file"], fact["summary"]))[:limit]


def validation_ok(path: pathlib.Path | None) -> bool:
    if not path:
        return False
    report = load_json(path)
    cargo_proof = report.get("cargo_validation") == "ran" and report.get("replay_comparison") == "passed"
    return report.get("schema_version") == 1 and report.get("status") == "pass" and cargo_proof and valid_receipt_hash(report)


def gate(risk_added: list[dict], risk_delta_score: int, proof: bool) -> dict:
    if risk_added and not proof:
        return {"status": "deny", "reason": "risk_added_without_validation", "required_validation": ["test_or_proof_receipt"]}
    if risk_delta_score > 0 and not proof:
        return {"status": "deny", "reason": "risk_delta_without_validation", "required_validation": ["test_or_proof_receipt"]}
    if risk_delta_score > 0:
        return {"status": "allow", "reason": "risk_delta_has_validation", "required_validation": []}
    return {"status": "allow", "reason": "no_risk_increase", "required_validation": []}


def build_report(old: dict, new: dict, proof: bool, limit: int) -> dict:
    old_nodes = nodes(old)
    new_nodes = nodes(new)
    old_edges = edges(old)
    new_edges = edges(new)
    added = new_edges - old_edges
    removed = old_edges - new_edges
    changes = intent_changes(old, new)
    added_risk = {edge for edge in added if edge[0] in RISK_WEIGHT}
    removed_risk = {edge for edge in removed if edge[0] in RISK_WEIGHT}
    risk_delta_score = sum(RISK_WEIGHT[edge[0]] for edge in added_risk) - sum(RISK_WEIGHT[edge[0]] for edge in removed_risk)
    changed_node_keys = set(old_nodes) ^ set(new_nodes)
    files = changed_files(old_nodes, new_nodes, added, removed, changed_node_keys)
    decision = gate(as_edge(added_risk), risk_delta_score, proof)

    return {
        "schema_version": 1,
        "old_graph_hash": graph_hash(old, "graph_hash"),
        "new_graph_hash": graph_hash(new, "graph_hash"),
        "old_intent_hash": graph_hash(old, "intent_hash"),
        "new_intent_hash": graph_hash(new, "intent_hash"),
        "old_risk_hash": graph_hash(old, "risk_hash"),
        "new_risk_hash": graph_hash(new, "risk_hash"),
        "added_nodes": as_node(new_nodes, set(new_nodes) - set(old_nodes)),
        "removed_nodes": as_node(old_nodes, set(old_nodes) - set(new_nodes)),
        "added_edges": as_edge(added),
        "removed_edges": as_edge(removed),
        "intent_changes": changes,
        "risk_added": as_edge(added_risk),
        "risk_removed": as_edge(removed_risk),
        "risk_delta_score": risk_delta_score,
        "changed_files": files,
        "leverage_rank": leverage_rank(files, added, removed, changes, old_nodes, new_nodes),
        "prompt_facts": prompt_facts(added, changes, old_nodes, new_nodes, limit),
        "gate_decision": decision,
        "required_validation": decision["required_validation"],
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--old", required=True, type=pathlib.Path)
    parser.add_argument("--new", required=True, type=pathlib.Path)
    parser.add_argument("--report", type=pathlib.Path)
    parser.add_argument("--validation-report", "--proof", dest="validation_report", type=pathlib.Path)
    parser.add_argument("--top", type=int, default=8)
    parser.add_argument("--enforce-gate", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    report = build_report(load_json(args.old), load_json(args.new), validation_ok(args.validation_report), args.top)
    output = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(output, encoding="utf-8")
    else:
        print(output, end="")
    return 1 if args.enforce_gate and report["gate_decision"]["status"] == "deny" else 0


if __name__ == "__main__":
    raise SystemExit(main())