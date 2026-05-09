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
OBSERVE = ROOT / "scripts" / "observe_validation.sh"
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

    def test_preserves_connector_transport_artifact_classification_once(self) -> None:
        self.write_report(extra={
            "connector_failure_classification_present": True,
            "connector_failure_present": False,
            "connector_failure_status": "none",
            "connector_failure_classes": [],
            "connector_transport_instability_present": False,
            "connector_transport_artifact_classification": "transport_interrupted_artifacts_complete",
            "connector_transport_artifact_classification_options": [
                "transport_ok_no_artifacts",
                "transport_ok_artifacts_present",
                "transport_interrupted_artifacts_complete",
                "transport_interrupted_artifacts_incomplete",
            ],
            "connector_transport_artifact_classification_reason": "connector transport was interrupted but report and exit artifacts are present",
            "connector_transport_status": "502",
            "connector_transport_interrupted": True,
            "connector_transport_report_path": "target/observe/full-observe.ndjson",
            "connector_transport_report_present": True,
            "connector_transport_report_complete": True,
            "connector_transport_exit_file": "target/validation-logs/full-observe.exit",
            "connector_transport_exit_file_present": True,
        })
        done = self.run_script()
        self.assertEqual(done.returncode, 0, done.stderr)
        receipt = json.loads(self.receipt.read_text(encoding="utf-8"))
        manifest = self.out.read_text(encoding="utf-8")
        self.assertEqual(
            receipt["connector_transport_artifact_classification"],
            "transport_interrupted_artifacts_complete",
        )
        self.assertEqual(receipt["connector_transport_status"], "502")
        self.assertTrue(receipt["connector_transport_report_complete"])
        self.assertTrue(receipt["connector_transport_exit_file_present"])
        for token in (
            "connector_transport_artifact_classification: transport_interrupted_artifacts_complete",
            "connector_transport_status: 502",
            "connector_transport_report_complete: True",
            "connector_transport_exit_file_present: True",
            "connector_transport_artifact_classification_options",
        ):
            self.assertIn(token, manifest)
        metric_names = [line[2:].split(":", 1)[0] for line in manifest.splitlines()
                        if line.startswith("- ") and ":" in line]
        self.assertEqual(Counter(metric_names)["connector_transport_artifact_classification"], 1)
        self.assertEqual(Counter(metric_names)["connector_transport_status"], 1)

    def test_accepts_compact_full_summary_artifact_replay(self) -> None:
        commands = [
            {"name": "cargo_test_all_targets", "cmd": ["cargo", "test", "--all-targets"], "status": "pass"},
            {"name": "panic_surface_validation", "cmd": ["python3", "scripts/validate_rust_panic_surface.py"], "status": "pass"},
            {"name": "policy_learning_trace_validation", "cmd": ["python3", "scripts/validate_policy_learning_trace.py"], "status": "pass"},
        ]
        self.write_report(commands=commands, command_count=3, test_count=3, extra={
            "full_summary_report_only": True,
            "full_summary_report_command": "--full-summary-report",
            "command_execution_status": "pass",
            "missing_signal_status": "pass",
            "missing_signal_count": 0,
            "missing_signal_flags": {
                "missing_cargo_test": False,
                "missing_runtime_manifest_base_match": False,
            },
            "runtime_archive_evidence_source": "compact_report",
            "runtime_archive_report_status": "pass",
            "runtime_manifest_base_expected": self.base,
            "runtime_manifest_base_commit": self.base,
            "runtime_manifest_base_matches_delta_base": True,
            "connector_transport_artifact_classification": "transport_interrupted_artifacts_complete",
            "connector_transport_status": "502",
            "connector_transport_report_complete": True,
            "connector_transport_exit_file_present": True,
        })
        done = self.run_script()
        self.assertEqual(done.returncode, 0, done.stderr)
        receipt = json.loads(self.receipt.read_text(encoding="utf-8"))
        manifest = self.out.read_text(encoding="utf-8")
        self.assertEqual(receipt["validation_command_count"], 3)
        self.assertEqual(receipt["validation_test_count"], 3)
        self.assertEqual(receipt["connector_transport_artifact_classification"], "transport_interrupted_artifacts_complete")
        self.assertTrue(receipt["runtime_manifest_base_matches_delta_base"])
        self.assertIn("cargo_test_all_targets: pass", manifest)
        self.assertIn("connector_transport_artifact_classification: transport_interrupted_artifacts_complete", manifest)
        self.assertIn("runtime_manifest_base_matches_delta_base: True", manifest)

    def test_generates_manifest_from_actual_full_summary_report_artifact(self) -> None:
        scripts_dir = self.repo / "scripts"
        scripts_dir.mkdir(exist_ok=True)
        observe_copy = scripts_dir / "observe_validation.sh"
        observe_copy.write_text(OBSERVE.read_text(encoding="utf-8"), encoding="utf-8")
        (scripts_dir / "validate_graph_workflow_fixture.py").write_text(
            "def graph_fixture_report(*args, **kwargs):\n"
            "    return {'event': 'graph_fixture_report', 'graph_evidence_status': 'not_used'}\n",
            encoding="utf-8",
        )

        report = self.repo / "target" / "observe" / "actual-full-summary.ndjson"
        exit_file = self.repo / "target" / "validation-logs" / "actual-full-summary.exit"
        exit_file.parent.mkdir(parents=True, exist_ok=True)
        exit_file.write_text("0\n", encoding="utf-8")
        env = os.environ.copy()
        env.update({
            "CANON_OBSERVE_REPORT": str(report),
            "CANON_DELTA_BASE": self.base,
            "CANON_CONNECTOR_TRANSPORT_STATUS": "502",
            "CANON_CONNECTOR_TRANSPORT_REPORT": str(report),
            "CANON_CONNECTOR_TRANSPORT_EXIT_FILE": str(exit_file),
        })
        done = subprocess.run(
            [sys.executable, str(observe_copy), "--full-summary-report"],
            cwd=self.repo,
            env=env,
            text=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertTrue(report.exists())

        self.report.write_text(report.read_text(encoding="utf-8"), encoding="utf-8")
        manifest_done = self.run_script()
        self.assertEqual(manifest_done.returncode, 0, manifest_done.stderr)

        rows = [json.loads(line) for line in report.read_text(encoding="utf-8").splitlines()]
        summary = rows[-1]
        receipt = json.loads(self.receipt.read_text(encoding="utf-8"))
        manifest = self.out.read_text(encoding="utf-8")
        self.assertEqual(summary["git_head"], self.head)
        self.assertTrue(receipt["full_summary_report_only"])
        self.assertEqual(receipt["full_summary_report_command"], "--full-summary-report")
        self.assertEqual(receipt["validation_status"], "pass")
        self.assertEqual(receipt["command_execution_status"], "pass")
        self.assertEqual(receipt["missing_signal_status"], "pass")
        self.assertEqual(receipt["runtime_archive_evidence_source"], "compact_report")
        self.assertEqual(
            receipt["connector_transport_artifact_classification"],
            "transport_interrupted_artifacts_complete",
        )
        self.assertEqual(receipt["runtime_manifest_base_expected"], self.base)
        self.assertEqual(receipt["runtime_manifest_base_commit"], self.base)
        self.assertTrue(receipt["runtime_manifest_base_matches_delta_base"])
        self.assertEqual(receipt["validation_command_count"], 3)
        self.assertEqual(receipt["validation_test_count"], 3)
        for command in receipt["validation_commands"]:
            self.assertIn("cmd", command)
            self.assertTrue(command["cmd"])
        for token in (
            "validation_status: pass",
            "command_execution_status: pass",
            "missing_signal_status: pass",
            "runtime_archive_evidence_source: compact_report",
            "full_summary_report_only: True",
            "full_summary_report_command: --full-summary-report",
            "connector_transport_artifact_classification: transport_interrupted_artifacts_complete",
            "runtime_manifest_base_matches_delta_base: True",
            "cargo_test_all_targets: pass",
        ):
            self.assertIn(token, manifest)

    def test_compact_preserved_fields_render_once_with_command_rows_fallback(self) -> None:
        commands = [
            {"event": "validation_command", "name": "unit", "cmd": ["python3", "-m", "unittest"], "status": "pass"},
            {"event": "validation_command", "name": "compile", "cmd": ["python3", "-m", "py_compile"], "status": "pass"},
        ]
        summary = {
            "event": "validation_summary",
            "git_head": self.head,
            "validation_status": "pass",
            "validation_command_count": len(commands),
            "validation_test_count": len(commands),
            "command_execution_status": "pass",
            "missing_signal_status": "pass",
            "missing_signal_count": 0,
            "missing_signal_flags": {"missing_cargo_test": False},
            "runtime_archive_evidence_source": "compact_report",
            "runtime_archive_report_present": True,
            "runtime_archive_report_status": "pass",
            "runtime_archive_report_base_matches_current": True,
            "runtime_archive_present": True,
            "runtime_manifest_base_expected": self.base,
            "runtime_manifest_base_commit": self.base,
            "runtime_manifest_base_matches_delta_base": True,
            "full_summary_report_only": True,
            "full_summary_report_command": "--full-summary-report",
        }
        self.report.write_text(
            "\n".join(json.dumps(row) for row in [*commands, summary]) + "\n",
            encoding="utf-8",
        )

        done = self.run_script()
        self.assertEqual(done.returncode, 0, done.stderr)
        receipt = json.loads(self.receipt.read_text(encoding="utf-8"))
        manifest = self.out.read_text(encoding="utf-8")
        metric_names = [line[2:].split(":", 1)[0] for line in manifest.splitlines()
                        if line.startswith("- ") and ":" in line]

        self.assertEqual(receipt["validation_command_count"], 2)
        self.assertEqual([command["name"] for command in receipt["validation_commands"]], ["unit", "compile"])
        self.assertEqual(receipt["command_execution_status"], "pass")
        self.assertEqual(receipt["missing_signal_status"], "pass")
        self.assertEqual(receipt["runtime_archive_evidence_source"], "compact_report")
        self.assertTrue(receipt["runtime_archive_report_present"])
        self.assertTrue(receipt["runtime_archive_present"])
        for key in (
            "validation_status",
            "command_execution_status",
            "missing_signal_status",
            "missing_signal_count",
            "runtime_archive_evidence_source",
            "runtime_archive_report_present",
            "runtime_archive_report_status",
            "runtime_archive_report_base_matches_current",
            "runtime_archive_present",
            "runtime_manifest_base_expected",
            "runtime_manifest_base_commit",
            "runtime_manifest_base_matches_delta_base",
            "full_summary_report_only",
            "full_summary_report_command",
        ):
            self.assertEqual(Counter(metric_names)[key], 1, key)
        self.assertIn("unit: pass :: ['python3', '-m', 'unittest']", manifest)
        self.assertIn("compile: pass :: ['python3', '-m', 'py_compile']", manifest)

    def test_accepts_matching_summary_and_row_command_evidence(self) -> None:
        commands = [
            {
                "name": "unit",
                "cmd": ["python3", "-m", "unittest"],
                "status": "pass",
                "exit_code": 0,
                "duration_ms": 7,
                "timed_out": False,
                "connector_failure_class": "none",
            },
            {
                "name": "compile",
                "cmd": ["python3", "-m", "py_compile"],
                "status": "pass",
                "exit_code": 0,
                "duration_ms": 3,
                "timed_out": False,
                "connector_failure_class": "none",
            },
        ]
        rows = [
            {"event": "validation_command", **command}
            for command in commands
        ]
        summary = {
            "event": "validation_summary",
            "git_head": self.head,
            "validation_status": "pass",
            "validation_commands": commands,
            "validation_command_count": len(commands),
            "validation_test_count": len(commands),
        }
        self.report.write_text(
            "\n".join(json.dumps(row) for row in [*rows, summary]) + "\n",
            encoding="utf-8",
        )

        done = self.run_script()
        self.assertEqual(done.returncode, 0, done.stderr)
        receipt = json.loads(self.receipt.read_text(encoding="utf-8"))
        self.assertEqual(receipt["validation_command_count"], 2)
        self.assertEqual(receipt["validation_command_distinct_count"], 2)
        self.assertEqual(receipt["validation_command_source"], "summary_validation_commands")
        self.assertEqual(receipt["validation_command_summary_input_count"], 2)
        self.assertEqual(receipt["validation_command_summary_distinct_count"], 2)
        self.assertEqual(receipt["validation_command_summary_duplicate_count"], 0)
        self.assertEqual(receipt["validation_command_row_input_count"], 2)
        self.assertEqual(receipt["validation_command_row_distinct_count"], 2)
        self.assertEqual(receipt["validation_command_row_duplicate_count"], 0)
        self.assertEqual([command["name"] for command in receipt["validation_commands"]], ["unit", "compile"])
        self.assertEqual([command["duration_ms"] for command in receipt["validation_commands"]], [7, 3])
        manifest = self.out.read_text(encoding="utf-8")
        self.assertIn("validation_command_distinct_count: 2", manifest)
        self.assertIn("validation_command_source: summary_validation_commands", manifest)
        self.assertIn("validation_command_summary_input_count: 2", manifest)
        self.assertIn("validation_command_row_input_count: 2", manifest)

    def test_rejects_conflicting_summary_and_row_command_evidence(self) -> None:
        row_commands = [
            {"event": "validation_command", "name": "unit", "cmd": ["python3", "-m", "unittest"], "status": "pass"},
            {"event": "validation_command", "name": "compile", "cmd": ["python3", "-m", "py_compile"], "status": "pass"},
        ]
        summary_commands = [
            {"name": "unit", "cmd": ["python3", "-m", "unittest"], "status": "pass"},
            {"name": "compile", "cmd": ["python3", "-m", "py_compile"], "status": "fail"},
        ]
        summary = {
            "event": "validation_summary",
            "git_head": self.head,
            "validation_status": "pass",
            "validation_commands": summary_commands,
            "validation_command_count": len(summary_commands),
            "validation_test_count": len(summary_commands),
        }
        self.report.write_text(
            "\n".join(json.dumps(row) for row in [*row_commands, summary]) + "\n",
            encoding="utf-8",
        )

        done = self.run_script()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("conflicting validation command evidence", done.stderr)

    def test_rejects_conflicting_summary_and_row_command_execution_metadata(self) -> None:
        row_commands = [
            {
                "event": "validation_command",
                "name": "unit",
                "cmd": ["python3", "-m", "unittest"],
                "status": "pass",
                "exit_code": 0,
                "duration_ms": 7,
                "timed_out": False,
                "connector_failure_class": "none",
            },
        ]
        summary_commands = [
            {
                "name": "unit",
                "cmd": ["python3", "-m", "unittest"],
                "status": "pass",
                "exit_code": 0,
                "duration_ms": 9,
                "timed_out": False,
                "connector_failure_class": "none",
            },
        ]
        summary = {
            "event": "validation_summary",
            "git_head": self.head,
            "validation_status": "pass",
            "validation_commands": summary_commands,
            "validation_command_count": len(summary_commands),
            "validation_test_count": len(summary_commands),
        }
        self.report.write_text(
            "\n".join(json.dumps(row) for row in [*row_commands, summary]) + "\n",
            encoding="utf-8",
        )

        done = self.run_script()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("conflicting validation command evidence", done.stderr)

    def test_accepts_duplicate_summary_commands_with_distinct_count(self) -> None:
        command = {
            "name": "unit",
            "cmd": ["python3", "-m", "unittest"],
            "status": "pass",
            "exit_code": 0,
            "duration_ms": 7,
            "timed_out": False,
            "connector_failure_class": "none",
        }
        self.write_report(
            commands=[command, command],
            command_count=1,
            test_count=2,
        )

        done = self.run_script()
        self.assertEqual(done.returncode, 0, done.stderr)
        receipt = json.loads(self.receipt.read_text(encoding="utf-8"))
        self.assertEqual(receipt["validation_command_count"], 1)
        self.assertEqual(receipt["validation_command_distinct_count"], 1)
        self.assertEqual(receipt["validation_command_source"], "summary_validation_commands")
        self.assertEqual(receipt["validation_command_summary_input_count"], 2)
        self.assertEqual(receipt["validation_command_summary_distinct_count"], 1)
        self.assertEqual(receipt["validation_command_summary_duplicate_count"], 1)
        self.assertEqual(receipt["validation_command_row_input_count"], 0)
        self.assertEqual(receipt["validation_command_row_distinct_count"], 0)
        self.assertEqual(receipt["validation_command_row_duplicate_count"], 0)
        self.assertEqual([command["name"] for command in receipt["validation_commands"]], ["unit"])
        manifest = self.out.read_text(encoding="utf-8")
        self.assertIn("validation_command_summary_duplicate_count: 1", manifest)
        self.assertIn("validation_command_row_input_count: 0", manifest)

    def test_rejects_duplicate_summary_commands_with_inflated_count(self) -> None:
        command = {
            "name": "unit",
            "cmd": ["python3", "-m", "unittest"],
            "status": "pass",
            "exit_code": 0,
            "duration_ms": 7,
            "timed_out": False,
            "connector_failure_class": "none",
        }
        self.write_report(
            commands=[command, command],
            command_count=2,
            test_count=2,
        )

        done = self.run_script()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("validation_command_count 2 != command rows 1", done.stderr)

    def test_rejects_conflicting_duplicate_summary_commands(self) -> None:
        first = {
            "name": "unit",
            "cmd": ["python3", "-m", "unittest"],
            "status": "pass",
            "exit_code": 0,
            "duration_ms": 7,
            "timed_out": False,
            "connector_failure_class": "none",
        }
        second = dict(first)
        second["duration_ms"] = 8
        self.write_report(
            commands=[first, second],
            command_count=2,
            test_count=2,
        )

        done = self.run_script()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("conflicting summary validation_commands command evidence for command unit", done.stderr)

    def test_accepts_duplicate_validation_command_rows_with_distinct_count(self) -> None:
        command = {
            "event": "validation_command",
            "name": "unit",
            "cmd": ["python3", "-m", "unittest"],
            "status": "pass",
            "exit_code": 0,
            "duration_ms": 7,
            "timed_out": False,
            "connector_failure_class": "none",
        }
        summary = {
            "event": "validation_summary",
            "git_head": self.head,
            "validation_status": "pass",
            "validation_command_count": 1,
            "validation_test_count": 2,
        }
        self.report.write_text(
            "\n".join(json.dumps(row) for row in [command, command, summary]) + "\n",
            encoding="utf-8",
        )

        done = self.run_script()
        self.assertEqual(done.returncode, 0, done.stderr)
        receipt = json.loads(self.receipt.read_text(encoding="utf-8"))
        self.assertEqual(receipt["validation_command_count"], 1)
        self.assertEqual(receipt["validation_command_distinct_count"], 1)
        self.assertEqual(receipt["validation_command_source"], "validation_command_rows")
        self.assertEqual(receipt["validation_command_summary_input_count"], 0)
        self.assertEqual(receipt["validation_command_summary_distinct_count"], 0)
        self.assertEqual(receipt["validation_command_summary_duplicate_count"], 0)
        self.assertEqual(receipt["validation_command_row_input_count"], 2)
        self.assertEqual(receipt["validation_command_row_distinct_count"], 1)
        self.assertEqual(receipt["validation_command_row_duplicate_count"], 1)
        self.assertEqual([command["name"] for command in receipt["validation_commands"]], ["unit"])
        manifest = self.out.read_text(encoding="utf-8")
        self.assertIn("validation_command_source: validation_command_rows", manifest)
        self.assertIn("validation_command_row_duplicate_count: 1", manifest)

    def test_rejects_duplicate_validation_command_rows_with_inflated_count(self) -> None:
        command = {
            "event": "validation_command",
            "name": "unit",
            "cmd": ["python3", "-m", "unittest"],
            "status": "pass",
            "exit_code": 0,
            "duration_ms": 7,
            "timed_out": False,
            "connector_failure_class": "none",
        }
        summary = {
            "event": "validation_summary",
            "git_head": self.head,
            "validation_status": "pass",
            "validation_command_count": 2,
            "validation_test_count": 2,
        }
        self.report.write_text(
            "\n".join(json.dumps(row) for row in [command, command, summary]) + "\n",
            encoding="utf-8",
        )

        done = self.run_script()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("validation_command_count 2 != command rows 1", done.stderr)

    def test_rejects_conflicting_duplicate_validation_command_rows(self) -> None:
        first = {
            "event": "validation_command",
            "name": "unit",
            "cmd": ["python3", "-m", "unittest"],
            "status": "pass",
            "exit_code": 0,
            "duration_ms": 7,
            "timed_out": False,
            "connector_failure_class": "none",
        }
        second = dict(first)
        second["duration_ms"] = 8
        summary = {
            "event": "validation_summary",
            "git_head": self.head,
            "validation_status": "pass",
            "validation_command_count": 2,
            "validation_test_count": 2,
        }
        self.report.write_text(
            "\n".join(json.dumps(row) for row in [first, second, summary]) + "\n",
            encoding="utf-8",
        )

        done = self.run_script()
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("conflicting validation_command rows command evidence for command unit", done.stderr)


if __name__ == "__main__":
    unittest.main()
