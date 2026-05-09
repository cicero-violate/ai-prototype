#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import tempfile
import textwrap
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "validate_rust_panic_surface.py"


class PanicSurfaceContractTest(unittest.TestCase):
    def test_validation_fixtures_are_not_counted_as_production_panic_surface(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            fixture = root / "canon-rustc-v3" / "validation" / "fixtures" / "sample" / "src"
            fixture.mkdir(parents=True)
            (fixture / "lib.rs").write_text(
                textwrap.dedent(
                    """\
                    pub fn fixture(value: Option<u8>) -> u8 {
                        value.expect("fixture panic fact")
                    }
                    """
                ),
                encoding="utf-8",
            )
            report = root / "panic-surface.json"

            done = subprocess.run(
                [
                    "python3",
                    str(SCRIPT),
                    "--root",
                    str(root),
                    "--fail-production-unwrap",
                    "--report",
                    str(report),
                ],
                text=True,
                capture_output=True,
                timeout=30,
                check=False,
            )

            self.assertEqual(done.returncode, 0, done.stderr)
            data = json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual(data["production_total"], 0)
            self.assertEqual(data["test_total"], 1)


if __name__ == "__main__":
    unittest.main()