#!/usr/bin/env python3
"""Validate graph-editor auto-refactor plan generation for live graph artifacts."""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess


ROOT = pathlib.Path(__file__).resolve().parents[1]


def graph_paths(root: pathlib.Path) -> list[pathlib.Path]:
    if root.name == "graph.json" and root.exists():
        return [root]
    if not root.exists():
        return []
    return sorted(root.rglob("graph.json"))


def run(argv: list[str], cwd: pathlib.Path = ROOT) -> None:
    subprocess.run(argv, cwd=cwd, check=True)


def relative_path(path: pathlib.Path, base: pathlib.Path = ROOT) -> str:
    return str(path.relative_to(base) if path.is_relative_to(base) else path)


def query_supported_schema_version(graph_editor_root: pathlib.Path) -> int:
    result = subprocess.run(
        ["cargo", "run", "--quiet", "--bin", "auto_refactor_plan", "--", "--schema-version"],
        cwd=graph_editor_root,
        capture_output=True,
        text=True,
        check=True,
    )
    return int(result.stdout.strip())


def graph_schema_version(graph: pathlib.Path) -> int | None:
    try:
        data = json.loads(graph.read_text(encoding="utf-8"))
        return data.get("meta", {}).get("schema_version")
    except Exception:
        return None


def validate_graph(graph: pathlib.Path, report_dir: pathlib.Path, graph_editor_root: pathlib.Path) -> dict[str, object]:
    relative = graph.relative_to(ROOT) if graph.is_relative_to(ROOT) else graph
    safe_name = "__".join(relative.with_suffix("").parts)
    plan = report_dir / f"{safe_name}.graph-editor-plan.json"
    report_dir.mkdir(parents=True, exist_ok=True)

    run(
        [
            "cargo",
            "run",
            "--quiet",
            "--bin",
            "auto_refactor_plan",
            "--",
            "--graph",
            str(graph),
            "--out",
            str(plan),
        ],
        cwd=graph_editor_root,
    )

    data = json.loads(plan.read_text(encoding="utf-8"))
    status = data.get("verification", {}).get("status")
    if status != "planned":
        raise AssertionError(f"{graph}: graph-editor auto-refactor plan failed")
    return {
        "graph": str(relative),
        "crate_name": data.get("crate_name"),
        "operation_count": data.get("operation_count"),
        "split_surface_count": len(data.get("split_surface", [])),
        "merge_surface_count": len(data.get("merge_surface", [])),
        "canonicalize_surface_count": len(data.get("canonicalize_surface", [])),
        "plan_report": relative_path(plan),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifact-root", type=pathlib.Path, default=pathlib.Path("state/rustc"))
    parser.add_argument("--report-dir", type=pathlib.Path)
    parser.add_argument("--graph-editor-root", type=pathlib.Path, default=ROOT.parent / "graph-editor")
    args = parser.parse_args()

    artifact_root = args.artifact_root if args.artifact_root.is_absolute() else ROOT / args.artifact_root
    report_dir = args.report_dir or artifact_root / "auto-refactor"
    report_dir = report_dir if report_dir.is_absolute() else ROOT / report_dir
    graph_editor_root = args.graph_editor_root if args.graph_editor_root.is_absolute() else ROOT / args.graph_editor_root
    graphs = graph_paths(artifact_root)
    if not graphs:
        raise AssertionError(f"no graph.json files under {args.artifact_root}")

    supported_version = query_supported_schema_version(graph_editor_root)
    compatible = []
    for graph in graphs:
        version = graph_schema_version(graph)
        if version != supported_version:
            print(f"  skip {relative_path(graph)}: schema_version {version} (editor supports {supported_version})")
        else:
            compatible.append(graph)

    summaries = [validate_graph(graph, report_dir, graph_editor_root) for graph in compatible]

    print(f"graph-editor auto-refactor live validation: pass graphs={len(summaries)}")
    for summary in summaries:
        print(f"  {summary['graph']}: operations={summary['operation_count']}")
        print(
            "    surfaces: "
            f"split={summary['split_surface_count']} "
            f"merge={summary['merge_surface_count']} "
            f"canonicalize={summary['canonicalize_surface_count']}"
        )
        print(f"    plan: {summary['plan_report']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
