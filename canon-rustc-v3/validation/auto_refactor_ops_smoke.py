#!/usr/bin/env python3
from __future__ import annotations

import json
import pathlib
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
FIXTURE = ROOT / "validation" / "fixtures" / "auto_refactor_surface" / "graph.json"


def main() -> int:
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = pathlib.Path(tmp)
        surface = tmp_path / "surface.json"
        ops = tmp_path / "ops.json"
        subprocess.run(
            [sys.executable, str(ROOT / "validation" / "auto_refactor_surface.py"), str(FIXTURE), "--out", str(surface)],
            check=True,
        )
        subprocess.run(
            [sys.executable, str(ROOT / "validation" / "auto_refactor_ops.py"), str(surface), "--out", str(ops)],
            check=True,
        )
        data = json.loads(ops.read_text(encoding="utf-8"))
    assert data["schema_version"] == 1
    assert data["graph_schema_version"] == 16
    assert data["verification"]["status"] == "pass"
    kinds = {op["op"] for op in data["operations"]}
    assert "SplitFn" in kinds
    assert "MergeFns" in kinds
    assert "ExtractTrait" in kinds
    assert data["operation_count"] == len(data["operations"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
