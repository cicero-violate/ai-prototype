#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from collections import Counter
from types import SimpleNamespace
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "write_delta_manifest.py"
sys.path.insert(0, str(SCRIPT.parent))
import write_delta_manifest  # noqa: E402


class DeltaManifestTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.tmp = tempfile.TemporaryDirectory()
        cls.repo = Path(cls.tmp.name) / "repo"
        cls.repo.mkdir()
        cls.git_class("init")
        cls.git_class("config", "user.email", "test@example.invalid")
        cls.git_class("config", "user.name", "Test User")
        (cls.repo / "file.txt").write_text("base\n", encoding="utf-8")
        cls.git_class("add", "file.txt")
        cls.git_class("commit", "-m", "base")
        cls.base = cls.git_class("rev-parse", "HEAD").stdout.strip()
        (cls.repo / "file.txt").write_text("head\n", encoding="utf-8")
        cls.git_class("commit", "-am", "head")
        cls.head = cls.git_class("rev-parse", "HEAD").stdout.strip()
        cls.bundle = cls.repo / "delta.bundle"
        cls.git_class("bundle", "create", str(cls.bundle), "HEAD", f"^{cls.base}")
        cls.report = cls.repo / "report.ndjson"
        cls.out = cls.repo / "DELTA_MANIFEST.md"
        cls.receipt = cls.repo / "receipt.json"

    @classmethod
    def tearDownClass(cls) -> None:
        cls.tmp.cleanup()

    def setUp(self) -> None:
        for path in (self.report, self.out, self.receipt):
            path.unlink(missing_ok=True)

    @classmethod
    def git_class(cls, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(["git", *args], cwd=cls.repo, text=True, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, check=True)

    def git(self, *args: str) -> subprocess.CompletedProcess[str]:
        return self.git_class(*args)

    def write_report(self, *, head: str | None = None, commands: list[dict] | None = None,
                     command_count: int | None = None, test_count: int = 1,
                     zero_test_reason: str | None = None,
                     extra: dict | None = None) -> None:
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
        if extra:
            summary.update(extra)
        self.report.write_text(json.dumps(summary) + "\n", encoding="utf-8")

    def run_script(self, *extra: str) -> subprocess.CompletedProcess[str]:
        bundle = str(self.bundle)
        if extra:
            self.assertEqual(extra[0], "--bundle")
            bundle = extra[1]
        args = SimpleNamespace(base=self.base, head=self.head, report=str(self.report),
                               bundle=bundle, out=str(self.out), receipt_out=str(self.receipt),
                               zero_test_reason=None)
        cwd = Path.cwd()
        os.chdir(self.repo)
        try:
            receipt = write_delta_manifest.receipt(args)
            self.receipt.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n",
                                    encoding="utf-8")
            write_delta_manifest.write_manifest(self.out, receipt)
            return SimpleNamespace(returncode=0, stdout="", stderr="")
        except SystemExit as exc:
            return SimpleNamespace(returncode=1, stdout="", stderr=str(exc))
        finally:
            os.chdir(cwd)

    def test_pass_with_commands_and_tests(self) -> None:
        self.write_report(test_count=2)
        done = self.run_script()
        self.assertEqual(done.returncode, 0, done.stderr)
        receipt = json.loads(self.receipt.read_text(encoding="utf-8"))
        manifest_lines = self.out.read_text(encoding="utf-8").splitlines()
        self.assertEqual(manifest_lines[0], f"base_commit: {self.base}")
        self.assertEqual(manifest_lines[1], f"head_commit: {self.head}")
        self.assertEqual(receipt["validation_command_count"], 1)
        self.assertEqual(receipt["validation_test_count"], 2)
        self.assertEqual(receipt["bundle_verify"], "pass")
        self.assertTrue(receipt["bundle_requires_base_commit"])
        self.assertIn(self.base, receipt["bundle_required_refs"])
        self.assertIn("file.txt", receipt["changed_files"])

    def test_rejects_complete_history_bundle_without_base_requirement(self) -> None:
        self.write_report(test_count=1)
        full_bundle = self.repo / "full.bundle"
        self.git("bundle", "create", str(full_bundle), "HEAD")
        done = self.run_script("--bundle", str(full_bundle))
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("bundle does not require base commit", done.stderr)

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

    def test_preserves_runtime_archive_base_match_evidence(self) -> None:
        self.write_report(extra={
            "runtime_manifest_base_expected": self.base,
            "runtime_manifest_base_matches_delta_base": True,
        })
        done = self.run_script()
        self.assertEqual(done.returncode, 0, done.stderr)
        receipt = json.loads(self.receipt.read_text(encoding="utf-8"))
        manifest = self.out.read_text(encoding="utf-8")
        self.assertEqual(receipt["runtime_manifest_base_expected"], self.base)
        self.assertTrue(receipt["runtime_manifest_base_matches_delta_base"])
        self.assertIn("runtime_manifest_base_matches_delta_base: True", manifest)

    def test_preserves_runtime_archive_inspection_evidence_once(self) -> None:
        self.write_report(extra={
            "runtime_archive_inspection_status": "pass",
            "runtime_archive_download_index_files": 11,
            "runtime_archive_prior_state_files": 7,
            "runtime_archive_conversation_ledger_files": 5,
            "runtime_archive_delta_receipt_files": 5,
            "runtime_archive_audit_files": 1,
            "runtime_archive_current_run_summary_present": True,
            "runtime_archive_runtime_manifest_present": True,
        })
        done = self.run_script()
        self.assertEqual(done.returncode, 0, done.stderr)
        receipt = json.loads(self.receipt.read_text(encoding="utf-8"))
        manifest = self.out.read_text(encoding="utf-8")
        self.assertEqual(receipt["runtime_archive_inspection_status"], "pass")
        self.assertEqual(receipt["runtime_archive_download_index_files"], 11)
        self.assertEqual(receipt["runtime_archive_prior_state_files"], 7)
        for token in (
            "runtime_archive_inspection_status: pass",
            "runtime_archive_download_index_files: 11",
            "runtime_archive_prior_state_files: 7",
            "runtime_archive_conversation_ledger_files: 5",
            "runtime_archive_current_run_summary_present: True",
            "runtime_archive_runtime_manifest_present: True",
        ):
            self.assertIn(token, manifest)
        metric_names = [line[2:].split(":", 1)[0] for line in manifest.splitlines()
                        if line.startswith("- ") and ":" in line]
        self.assertEqual(Counter(metric_names)["runtime_archive_inspection_status"], 1)
        self.assertEqual(Counter(metric_names)["runtime_archive_prior_state_files"], 1)

    def test_preserves_policy_learning_and_panic_surface_evidence_once(self) -> None:
        self.write_report(extra={
            "policy_learning_trace_validation_result": "pass",
            "policy_learning_trace_status": "pass",
            "policy_learning_trace_function": "policy_learning_trace_promotes_and_uses_policy",
            "policy_learning_trace_check_count": 12,
            "policy_learning_trace_missing_count": 0,
            "panic_surface_production_unwrap_count": 0,
            "panic_surface_production_expect_count": 0,
            "panic_surface_production_panic_count": 0,
            "panic_surface_test_total": 319,
            "panic_surface_example_total": 0,
            "router_test_count": 0,
        })
        done = self.run_script()
        self.assertEqual(done.returncode, 0, done.stderr)
        receipt = json.loads(self.receipt.read_text(encoding="utf-8"))
        manifest = self.out.read_text(encoding="utf-8")
        self.assertEqual(receipt["policy_learning_trace_validation_result"], "pass")
        self.assertEqual(receipt["policy_learning_trace_missing_count"], 0)
        self.assertEqual(receipt["panic_surface_production_unwrap_count"], 0)
        for token in (
            "policy_learning_trace_validation_result: pass",
            "policy_learning_trace_status: pass",
            "policy_learning_trace_missing_count: 0",
            "panic_surface_production_unwrap_count: 0",
            "panic_surface_test_total: 319",
        ):
            self.assertIn(token, manifest)
        metric_names = [line[2:].split(":", 1)[0] for line in manifest.splitlines()
                        if line.startswith("- ") and ":" in line]
        self.assertEqual(Counter(metric_names)["router_test_count"], 1)
        self.assertEqual(Counter(metric_names)["policy_learning_trace_validation_result"], 1)

    def test_preserves_external_surface_evidence_files_and_tokens_once(self) -> None:
        self.write_report(extra={
            "external_observation_stream_test_present": True,
            "external_observation_stream_evidence_files": ["src/api/protocol.rs"],
            "external_observation_stream_evidence_tokens": {
                "ObservationCursor": ["src/api/protocol.rs"]
            },
            "external_api_action_test_present": True,
            "external_api_action_evidence_files": ["src/api/protocol.rs"],
            "external_api_action_evidence_tokens": {
                "CommandEnvelope::new": ["src/api/protocol.rs"]
            },
            "semantic_artifact_verification_test_present": True,
            "semantic_artifact_verification_evidence_files": [
                "src/capability/verification/semantic.rs"
            ],
            "semantic_artifact_verification_evidence_tokens": {
                "SemanticVerificationReceipt": ["src/capability/verification/semantic.rs"]
            },
        })
        done = self.run_script()
        self.assertEqual(done.returncode, 0, done.stderr)
        receipt = json.loads(self.receipt.read_text(encoding="utf-8"))
        manifest = self.out.read_text(encoding="utf-8")
        self.assertEqual(receipt["external_observation_stream_evidence_files"], ["src/api/protocol.rs"])
        self.assertTrue(receipt["external_api_action_test_present"])
        self.assertIn("external_observation_stream_evidence_files", manifest)
        self.assertIn("external_api_action_evidence_tokens", manifest)
        self.assertIn("SemanticVerificationReceipt", manifest)
        metric_names = [line[2:].split(":", 1)[0] for line in manifest.splitlines()
                        if line.startswith("- ") and ":" in line]
        self.assertEqual(Counter(metric_names)["external_observation_stream_evidence_files"], 1)
        self.assertEqual(Counter(metric_names)["external_api_action_evidence_tokens"], 1)
        self.assertEqual(Counter(metric_names)["semantic_artifact_verification_evidence_files"], 1)


if __name__ == "__main__":
    unittest.main()
