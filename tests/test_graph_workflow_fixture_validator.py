#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "validate_graph_workflow_fixture.py"
VALIDATOR_TIMEOUT_SECONDS = 30


class GraphWorkflowFixtureValidatorTest(unittest.TestCase):
    def run_script(self, root: Path, report: Path) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["python3", "-S", str(SCRIPT), "--root", str(root), "--report", str(report)],
            cwd=ROOT,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=VALIDATOR_TIMEOUT_SECONDS,
        )

    def test_current_fixture_emits_positive_landed_receipt_snapshot_report(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            report = Path(tmp) / "graph-fixture.json"
            done = self.run_script(ROOT, report)
            self.assertEqual(done.returncode, 0, done.stderr)
            data = json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual(data["event"], "graph_fixture_report")
            self.assertEqual(data["validation_status"], "pass")
            self.assertEqual(data["graph_evidence_status"], "graph_mutation_landed_with_receipt_snapshot")
            self.assertEqual(data["graph_fixture_validator"], "../scripts/validate_graph_workflow_fixture.py")
            self.assertTrue(data["graph_workflow_fixture_receipt_snapshot_present"])
            self.assertTrue(data["graph_workflow_fixture_integrity_valid"])
            self.assertTrue(data["graph_workflow_fixture_commands_present"])
            self.assertTrue(data["graph_workflow_fixture_command_sequence_valid"])
            self.assertTrue(data["graph_workflow_fixture_generated_outputs_present"])
            self.assertTrue(data["graph_workflow_fixture_receipt_ledger_flow_valid"])
            self.assertEqual(data["graph_workflow_fixture_missing"], [])

    def test_missing_fixture_fails_with_explicit_missing_signal(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            report = root / "graph-fixture.json"
            done = self.run_script(root, report)
            self.assertNotEqual(done.returncode, 0)
            data = json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual(data["validation_status"], "fail")
            self.assertEqual(data["graph_workflow_fixture_status"], "missing")
            self.assertIn("MANIFEST.txt", data["graph_workflow_fixture_missing"])
            self.assertTrue(data["missing_signal_flags"]["missing_graph_workflow_fixture_receipt_snapshot"])


if __name__ == "__main__":
    unittest.main()
