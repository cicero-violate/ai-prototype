#!/usr/bin/env python3
"""Emit deterministic auto-refactor operation specs from a surface report.

The generated records are a planning artifact. They do not rewrite source.
A later graph/source editor can lower these specs into concrete patches and
reject any operation whose spans, signatures, or post-checks do not match.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
from typing import Any

OPS_SCHEMA_VERSION = 1
SUPPORTED_OPS = {"SplitFn", "MergeFns", "ExtractTrait"}


def read_json(path: pathlib.Path) -> dict[str, Any]:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def write_json(path: pathlib.Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def stable_id(kind: str, payload: dict[str, Any]) -> str:
    encoded = json.dumps({"kind": kind, "payload": payload}, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()[:16]


def fn_name(path: str) -> str:
    return path.rsplit("::", 1)[-1]


def canonical_module(path: str) -> str:
    return path.rsplit("::", 1)[0] if "::" in path else path


def split_ops(surface: dict[str, Any]) -> list[dict[str, Any]]:
    ops = []
    for row in surface.get("split_surface", []):
        fn_path = row.get("fn")
        boundaries = row.get("recommended_split_boundaries", [])
        if not isinstance(fn_path, str) or not isinstance(boundaries, list) or len(boundaries) < 2:
            continue
        payload = {
            "fn": fn_path,
            "anchor": {"kind": "fn_path", "value": fn_path},
            "fan_out": row.get("fan_out"),
            "split_boundaries": boundaries,
            "generated_names": [f"{fn_name(fn_path)}__{str(boundary).split('::')[-1]}" for boundary in boundaries],
            "delegate_strategy": "preserve_original_signature",
            "preconditions": [
                "graph contains matching phase edges",
                "graph node for fn has source span or editor-provided def.lo/def.hi",
                "cargo check must pass after edit",
            ],
        }
        ops.append({"id": stable_id("SplitFn", payload), "op": "SplitFn", "payload": payload})
    return ops


def merge_ops(surface: dict[str, Any]) -> list[dict[str, Any]]:
    ops = []
    for row in surface.get("merge_surface", []):
        members = row.get("members", [])
        if not isinstance(members, list) or len(members) < 2 or not all(isinstance(item, str) for item in members):
            continue
        canonical = min(members)
        payload = {
            "members": sorted(members),
            "canonical_fn": canonical,
            "canonical_name": row.get("recommended_canonical_name") or fn_name(canonical),
            "module": row.get("module") or canonical_module(canonical),
            "similarity": row.get("similarity"),
            "replacement_strategy": "thin_wrappers_first",
            "preconditions": [
                "all members have compatible signatures or editor rejects operation",
                "similarity edge score is at least 0.75",
                "cargo check must pass after edit",
            ],
        }
        ops.append({"id": stable_id("MergeFns", payload), "op": "MergeFns", "payload": payload})
    return ops


def extract_trait_ops(surface: dict[str, Any]) -> list[dict[str, Any]]:
    ops = []
    for row in surface.get("canonicalize_surface", []):
        pair = row.get("pair", [])
        if not isinstance(pair, list) or len(pair) != 2 or not all(isinstance(item, str) for item in pair):
            continue
        payload = {
            "functions": sorted(pair),
            "trait": row.get("recommended_trait_boundary") or "trait::ProviderCanonicalBoundary",
            "providers": row.get("providers", {}),
            "similarity": row.get("similarity"),
            "extraction_strategy": "trait_with_provider_specific_impls",
            "preconditions": [
                "function signatures match after receiver normalization",
                "provider edges identify concrete provider-specific clauses",
                "cargo check must pass after edit",
            ],
        }
        ops.append({"id": stable_id("ExtractTrait", payload), "op": "ExtractTrait", "payload": payload})
    return ops


def validate_ops(ops: list[dict[str, Any]]) -> list[str]:
    errors: list[str] = []
    ids: set[str] = set()
    for index, op in enumerate(ops):
        op_id = op.get("id")
        kind = op.get("op")
        payload = op.get("payload")
        if kind not in SUPPORTED_OPS:
            errors.append(f"op[{index}] unsupported kind: {kind!r}")
        if not isinstance(op_id, str) or not op_id:
            errors.append(f"op[{index}] missing id")
        elif op_id in ids:
            errors.append(f"op[{index}] duplicate id: {op_id}")
        ids.add(str(op_id))
        if not isinstance(payload, dict):
            errors.append(f"op[{index}] missing payload")
    return errors


def build_ops(surface: dict[str, Any]) -> dict[str, Any]:
    operations = split_ops(surface) + merge_ops(surface) + extract_trait_ops(surface)
    errors = validate_ops(operations)
    return {
        "schema_version": OPS_SCHEMA_VERSION,
        "surface_schema_version": surface.get("schema_version"),
        "graph_schema_version": surface.get("graph_schema_version"),
        "crate_name": surface.get("crate_name", "unknown"),
        "operation_count": len(operations),
        "supported_ops": sorted(SUPPORTED_OPS),
        "operations": operations,
        "verification": {
            "status": "pass" if not errors else "fail",
            "errors": errors,
            "required_after_apply_checks": [
                "cargo check through wrapper",
                "new graph has rho_mut not greater than baseline",
                "new graph has lower or equal average mutation fan-out",
                "new graph has no additional panic surfaces",
            ],
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("surface_report", type=pathlib.Path)
    parser.add_argument("--out", type=pathlib.Path)
    args = parser.parse_args()
    result = build_ops(read_json(args.surface_report))
    if args.out:
        write_json(args.out, result)
    else:
        print(json.dumps(result, indent=2, sort_keys=True))
    return 0 if result["verification"]["status"] == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
