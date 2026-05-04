#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "write_delta_manifest.py"


class DeltaManifestTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Path(self.tmp.name) / "repo"
        self.repo.mkdir()
        self.git("init")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "Test User")
        (self.repo / "file.txt").write_text("base\n", encoding="utf-8")
        self.git("add", "file.txt")
        self.git("commit", "-m", "base")
        self.base = self.git("rev-parse", "HEAD").stdout.strip()
        (self.repo / "file.txt").write_text("head\n", encoding="utf-8")
        self.git("commit", "-am", "head")
        self.head = self.git("rev-parse", "HEAD").stdout.strip()
        self.bundle = self.repo / "delta.bundle"
        self.git("bundle", "create", str(self.bundle), "HEAD", f"^{self.base}")
        self.report = self.repo / "report.ndjson"
        self.out = self.repo / "DELTA_MANIFEST.md"
        self.receipt = self.repo / "receipt.json"

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def git(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(["git", *args], cwd=self.repo, text=True, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, check=True)

    def write_report(self, *, head: str | None = None, commands: list[dict] | None = None,
                     command_count: int | None = None, test_count: int = 1,
                     zero_test_reason: str | None = None) -> None:
        commands = commands if commands is not None else [
            {"name": "unit", "cmd": ["python3", "-m", "unittest"], "status": "pass"}
        ]
        summary = {
            "event": "validation_summary",
            "git_head": head or self.head,
            "validation_status": "pass",
            "validation_commands": commands,
            "validation_command_count": len(commands) if command_count is None else command_count,
            "validation_test_count": test_count,
        }
        if zero_test_reason:
            summary["zero_test_reason"] = zero_test_reason
        self.report.write_text(json.dumps(summary) + "\n", encoding="utf-8")

    def run_script(self, *extra: str) -> subprocess.CompletedProcess[str]:
        cmd = [
            sys.executable, str(SCRIPT),
            "--base", self.base,
            "--head", self.head,
            "--report", str(self.report),
            "--bundle", str(self.bundle),
            "--out", str(self.out),
            "--receipt-out", str(self.receipt),
            *extra,
        ]
        env = os.environ.copy()
        return subprocess.run(cmd, cwd=self.repo, text=True, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, env=env, check=False)

    def test_pass_with_commands_and_tests(self) -> None:
        self.write_report(test_count=2)
        done = self.run_script()
        self.assertEqual(done.returncode, 0, done.stderr)
        receipt = json.loads(self.receipt.read_text(encoding="utf-8"))
        self.assertEqual(receipt["validation_command_count"], 1)
        self.assertEqual(receipt["validation_test_count"], 2)
        self.assertEqual(receipt["bundle_verify"], "pass")
        self.assertIn("file.txt", receipt["changed_files"])

    def test_rejects_empty_commands(self) -> None:
        self.write_report(commands=[], command_count=0, test_count=1)
        done = self.run_script()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("no validation commands", done.stderr)

    def test_rejects_stale_report_head(self) -> None:
        self.write_report(head=self.base, test_count=1)
        done = self.run_script()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("stale validation report", done.stderr)

    def test_rejects_zero_tests_without_reason(self) -> None:
        self.write_report(test_count=0)
        done = self.run_script()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("zero without zero_test_reason", done.stderr)

    def test_allows_zero_tests_with_reason(self) -> None:
        self.write_report(test_count=0, zero_test_reason="environment has no test runner")
        done = self.run_script()
        self.assertEqual(done.returncode, 0, done.stderr)
        receipt = json.loads(self.receipt.read_text(encoding="utf-8"))
        self.assertEqual(receipt["validation_test_count"], 0)
        self.assertEqual(receipt["zero_test_reason"], "environment has no test runner")


if __name__ == "__main__":
    unittest.main()
