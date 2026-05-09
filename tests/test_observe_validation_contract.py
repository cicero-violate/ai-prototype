#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import re
import sys
import tarfile
import tempfile
import unittest
from importlib.machinery import SourceFileLoader
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OBSERVE = ROOT / "scripts" / "observe_validation.sh"
CARGO_CONFIG = ROOT / ".cargo" / "config.toml"
GRAPH_FIXTURE_VALIDATOR = ROOT / "scripts" / "validate_graph_workflow_fixture.py"


class ObserveValidationContractTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.script = OBSERVE.read_text(encoding="utf-8")
        cls.graph_fixture_validator = GRAPH_FIXTURE_VALIDATOR.read_text(encoding="utf-8")
        cls.config = CARGO_CONFIG.read_text(encoding="utf-8")
        loader = SourceFileLoader("observe_validation_contract_module", str(OBSERVE))
        spec = importlib.util.spec_from_loader(loader.name, loader)
        assert spec is not None
        module = importlib.util.module_from_spec(spec)
        assert spec.loader is not None
        sys.path.insert(0, str(OBSERVE.parent))
        spec.loader.exec_module(module)
        cls.observe_module = module

    def test_root_cargo_validation_clears_wrappers(self) -> None:
        self.assertIn('root_rust_env = {"RUSTC_WRAPPER": "", "RUSTC_WORKSPACE_WRAPPER": ""}', self.script)
        for name in ("cargo_fmt_check", "cargo_test_all_targets", "cargo_clippy_all_targets"):
            pattern = rf'run\("{name}".*env=root_rust_env\)'
            self.assertRegex(self.script, pattern)

    def test_default_cargo_config_has_no_absolute_wrapper(self) -> None:
        self.assertIsNone(re.search(r'^\s*rustc-wrapper\s*=', self.config, re.MULTILINE))
        self.assertIn("CANON_RUSTC_WRAPPER=/path/to/canon-rustc-v3", self.config)

    def test_wrapper_graph_capture_is_explicit_and_optional(self) -> None:
        self.assertIn('os.environ.get("CANON_RUSTC_WRAPPER", "")', self.script)
        self.assertIn('CANON_RUSTC_V3_ARTIFACT_DIR', self.script)
        self.assertIn('wrapper_graph_validation_requested', self.script)
        self.assertIn('def wrapper_graph_configuration_status(', self.script)
        self.assertIn('def wrapper_graph_validation_classification(', self.script)
        self.assertIn('wrapper_graph_configuration_status', self.script)
        self.assertIn('wrapper_graph_configuration_reason', self.script)
        self.assertIn('wrapper_graph_configuration_status_options', self.script)
        self.assertIn('wrapper_graph_validation_classification_present', self.script)
        self.assertIn('wrapper_graph_validation_classification_options', self.script)
        self.assertIn('not_configured', self.script)
        self.assertIn('artifact_dir_configured_without_wrapper', self.script)
        self.assertIn('wrapper_configured_missing', self.script)
        self.assertIn('wrapper_configured_available', self.script)
        self.assertIn('wrapper_graph_optional_not_configured', self.script)
        self.assertIn('wrapper_graph_requested_without_wrapper', self.script)
        self.assertIn('wrapper_graph_requested_wrapper_missing', self.script)
        self.assertIn('wrapper_graph_requested_passed', self.script)
        self.assertIn('wrapper_graph_requested_not_passed', self.script)
        self.assertIn('"missing_wrapper_graph_validation": wrapper_graph_validation["wrapper_graph_validation_missing"]', self.script)
        self.assertIn('"missing_rustc_wrapper_telemetry": wrapper_graph_validation["wrapper_graph_telemetry_missing"]', self.script)
        self.assertIn('status": "skipped_env_missing"', self.script)
        self.assertIn('CANON_RUSTC_WRAPPER not found', self.script)

    def test_wrapper_graph_configuration_classifier_executes_all_status_branches(self) -> None:
        cases = (
            ({"wrapper": "", "artifact_dir": "", "wrapper_available": False}, "not_configured"),
            (
                {"wrapper": "", "artifact_dir": "target/observe/wrapper-artifacts", "wrapper_available": False},
                "artifact_dir_configured_without_wrapper",
            ),
            ({"wrapper": "/missing/canon-rustc-v3", "artifact_dir": "", "wrapper_available": False}, "wrapper_configured_missing"),
            (
                {"wrapper": "/missing/canon-rustc-v3", "artifact_dir": "target/observe/wrapper-artifacts", "wrapper_available": False},
                "wrapper_configured_missing",
            ),
            ({"wrapper": "/bin/echo", "artifact_dir": "", "wrapper_available": True}, "wrapper_configured_available"),
            (
                {"wrapper": "/bin/echo", "artifact_dir": "target/observe/wrapper-artifacts", "wrapper_available": True},
                "wrapper_configured_available",
            ),
        )

        expected_reasons = {
            "not_configured": "CANON_RUSTC_WRAPPER and CANON_RUSTC_V3_ARTIFACT_DIR are unset",
            "artifact_dir_configured_without_wrapper": "CANON_RUSTC_V3_ARTIFACT_DIR is set but CANON_RUSTC_WRAPPER is unset",
            "wrapper_configured_missing": "CANON_RUSTC_WRAPPER is set but the path does not exist",
            "wrapper_configured_available": "CANON_RUSTC_WRAPPER is set and exists",
        }

        for kwargs, expected_status in cases:
            with self.subTest(kwargs=kwargs):
                status = self.observe_module.wrapper_graph_configuration_status(**kwargs)
                self.assertEqual(status, expected_status)
                self.assertEqual(
                    self.observe_module.wrapper_graph_configuration_reason(status),
                    expected_reasons[expected_status],
                )

    def test_wrapper_graph_validation_classifier_executes_all_status_branches(self) -> None:
        cases = (
            (
                {
                    "configuration_status": "not_configured",
                    "validation_result": "skipped_not_requested",
                    "validation_available": False,
                },
                {
                    "wrapper_graph_validation_classification": "wrapper_graph_optional_not_configured",
                    "wrapper_graph_validation_classification_reason": "wrapper graph validation is optional and was not requested",
                    "wrapper_graph_validation_required": False,
                    "wrapper_graph_telemetry_required": False,
                    "wrapper_graph_validation_missing": False,
                    "wrapper_graph_telemetry_missing": False,
                },
            ),
            (
                {
                    "configuration_status": "artifact_dir_configured_without_wrapper",
                    "validation_result": "skipped_env_missing",
                    "validation_available": False,
                },
                {
                    "wrapper_graph_validation_classification": "wrapper_graph_requested_without_wrapper",
                    "wrapper_graph_validation_missing": True,
                    "wrapper_graph_telemetry_missing": True,
                },
            ),
            (
                {
                    "configuration_status": "wrapper_configured_missing",
                    "validation_result": "skipped_env_missing",
                    "validation_available": False,
                },
                {
                    "wrapper_graph_validation_classification": "wrapper_graph_requested_wrapper_missing",
                    "wrapper_graph_validation_missing": True,
                    "wrapper_graph_telemetry_missing": True,
                },
            ),
            (
                {
                    "configuration_status": "wrapper_configured_available",
                    "validation_result": "pass",
                    "validation_available": True,
                },
                {
                    "wrapper_graph_validation_classification": "wrapper_graph_requested_passed",
                    "wrapper_graph_validation_missing": False,
                    "wrapper_graph_telemetry_missing": False,
                },
            ),
            (
                {
                    "configuration_status": "wrapper_configured_available",
                    "validation_result": "fail",
                    "validation_available": True,
                },
                {
                    "wrapper_graph_validation_classification": "wrapper_graph_requested_not_passed",
                    "wrapper_graph_validation_missing": True,
                    "wrapper_graph_telemetry_missing": True,
                },
            ),
        )

        for kwargs, expected in cases:
            with self.subTest(kwargs=kwargs):
                result = self.observe_module.wrapper_graph_validation_classification(**kwargs)
                for key, value in expected.items():
                    self.assertEqual(result[key], value)

    def test_missing_signals_separate_root_wrapper_and_graph(self) -> None:
        for flag in (
            "missing_cargo_test",
            "missing_wrapper_graph_validation",
            "missing_generated_graph_json",
            "missing_rustc_wrapper_telemetry",
        ):
            self.assertIn(flag, self.script)
        self.assertIn("def generated_graph_json_classification(", self.script)
        self.assertIn("generated_graph_json_classification_present", self.script)
        self.assertIn("generated_graph_json_classification_options", self.script)
        self.assertIn("generated_graph_json_present", self.script)
        self.assertIn("generated_graph_json_substituted_by_fixture", self.script)
        self.assertIn("generated_graph_json_missing_when_requested", self.script)
        self.assertIn("generated_graph_json_missing_no_fixture", self.script)
        self.assertIn('"missing_generated_graph_json": generated_graph_json["generated_graph_json_missing"]', self.script)

    def test_generated_graph_json_classifier_executes_all_status_branches(self) -> None:
        cases = (
            (
                {
                    "state_graph_present": True,
                    "wrapper_requested": True,
                    "fixture_receipt_snapshot_present": False,
                },
                {
                    "generated_graph_json_classification": "generated_graph_json_present",
                    "generated_graph_json_missing": False,
                    "generated_graph_json_fixture_substitution": False,
                },
            ),
            (
                {
                    "state_graph_present": False,
                    "wrapper_requested": False,
                    "fixture_receipt_snapshot_present": True,
                },
                {
                    "generated_graph_json_classification": "generated_graph_json_substituted_by_fixture",
                    "generated_graph_json_missing": False,
                    "generated_graph_json_fixture_substitution": True,
                },
            ),
            (
                {
                    "state_graph_present": False,
                    "wrapper_requested": True,
                    "fixture_receipt_snapshot_present": True,
                },
                {
                    "generated_graph_json_classification": "generated_graph_json_missing_when_requested",
                    "generated_graph_json_missing": True,
                    "generated_graph_json_fixture_substitution": False,
                },
            ),
            (
                {
                    "state_graph_present": False,
                    "wrapper_requested": False,
                    "fixture_receipt_snapshot_present": False,
                },
                {
                    "generated_graph_json_classification": "generated_graph_json_missing_no_fixture",
                    "generated_graph_json_missing": True,
                    "generated_graph_json_fixture_substitution": False,
                },
            ),
        )

        for kwargs, expected in cases:
            with self.subTest(kwargs=kwargs):
                result = self.observe_module.generated_graph_json_classification(**kwargs)
                for key, value in expected.items():
                    self.assertEqual(result[key], value)

    def test_panic_surface_validation_is_required(self) -> None:
        self.assertIn("validate_rust_panic_surface.py", self.script)
        self.assertIn("panic_surface_validation", self.script)
        self.assertIn("missing_panic_surface_validation", self.script)
        self.assertIn("panic_surface_production_unwrap_count", self.script)

    def test_policy_learning_trace_validation_is_required(self) -> None:
        for token in (
            "validate_policy_learning_trace.py",
            "policy_learning_trace_validation",
            "missing_policy_learning_replay_trace",
            "policy_learning_trace_status",
            "required.add(\"policy_learning_trace_validation\")",
        ):
            self.assertIn(token, self.script)

    def test_external_surface_evidence_is_source_derived(self) -> None:
        for token in (
            "def source_evidence()",
            "def token_files(tokens:",
            "def unique_files(mapping:",
            "external_observation_stream_test_present",
            "external_observation_stream_evidence_files",
            "external_observation_stream_evidence_tokens",
            "external_api_action_test_present",
            "external_api_action_evidence_files",
            "external_api_action_evidence_tokens",
            "semantic_artifact_verification_test_present",
            "semantic_artifact_verification_evidence_files",
            "semantic_artifact_verification_evidence_tokens",
            '"missing_external_observation_stream_test": not evidence["external_observation_stream_test_present"]',
            '"missing_external_api_action_test": not evidence["external_api_action_test_present"]',
            '"missing_semantic_artifact_verification_test": not evidence["semantic_artifact_verification_test_present"]',
        ):
            self.assertIn(token, self.script)

    def test_runtime_performance_contract_is_emitted_and_budgeted(self) -> None:
        for token in (
            "def runtime_performance_summary(",
            "runtime_performance_metrics",
            "runtime_performance_signal_present",
            "runtime_performance_budget_status",
            "project_agent_elapsed_ms_median",
            "project_agent_elapsed_ms_p95",
            "download_initial_get_ms_median",
            "download_follow_get_ms_median",
            "download_write_ms_median",
            "validation_command_duration_ms",
            "missing_runtime_performance_signal",
            "not runtime_performance[\"runtime_performance_signal_present\"]",
        ):
            self.assertIn(token, self.script)

    def test_runtime_performance_summary_is_source_derived_from_command_durations(self) -> None:
        summary = self.observe_module.runtime_performance_summary(
            [
                {"name": "fast", "duration_ms": 10},
                {"name": "slow", "duration_ms": 30},
                {"name": "missing_duration"},
            ]
        )

        self.assertTrue(summary["runtime_performance_signal_present"])
        self.assertEqual(summary["runtime_performance_budget_status"], "pass")
        self.assertEqual(summary["runtime_performance_budget_failures"], [])
        self.assertEqual(summary["project_agent_elapsed_ms_median"], 10)
        self.assertEqual(summary["project_agent_elapsed_ms_p95"], 30)
        self.assertEqual(summary["download_initial_get_ms_median"], 0)
        self.assertEqual(summary["download_follow_get_ms_median"], 0)
        self.assertEqual(summary["download_write_ms_median"], 0)
        self.assertEqual(summary["runtime_performance_metrics"]["validation_command_duration_ms"], 40)

    def test_runtime_performance_summary_reports_missing_without_command_durations(self) -> None:
        summary = self.observe_module.runtime_performance_summary([{"name": "missing_duration"}])

        self.assertFalse(summary["runtime_performance_signal_present"])
        self.assertEqual(summary["runtime_performance_budget_status"], "missing_signal")
        self.assertEqual(summary["runtime_performance_metrics"], {})
        self.assertIsNone(summary["project_agent_elapsed_ms_median"])

    def test_runtime_performance_budgets_are_environment_configurable(self) -> None:
        for name in (
            "CANON_MAX_PROJECT_AGENT_ELAPSED_MS_P95",
            "CANON_MAX_DOWNLOAD_INITIAL_GET_MS_P95",
            "CANON_MAX_DOWNLOAD_FOLLOW_GET_MS_P95",
            "CANON_MAX_DOWNLOAD_WRITE_MS_P95",
        ):
            self.assertIn(name, self.script)

    def test_long_running_validation_uses_300_second_test_timeout(self) -> None:
        self.assertIn("DEFAULT_TEST_TIMEOUT_SECONDS = 300", self.script)
        self.assertIn("CANON_TEST_TIMEOUT_SECONDS", self.script)
        for name in (
            "cargo_test_all_targets",
            "cargo_clippy_all_targets",
            "wrapper_graph_validation",
            "ollama_judgment_example",
        ):
            pattern = rf'run\("{name}".*timeout=test_timeout_seconds\(\)'
            self.assertRegex(self.script, pattern)
        self.assertNotIn("timeout=600", self.script)

    def test_runtime_archive_base_match_contract_is_emitted(self) -> None:
        for token in (
            "runtime_manifest_base_expected",
            "runtime_manifest_base_matches_delta_base",
            "runtime_manifest_base_commit",
            "missing_runtime_manifest_base_match",
            'base and runtime.get("runtime_manifest_base_commit") == base',
        ):
            self.assertIn(token, self.script)

    def test_runtime_archive_inspection_contract_is_emitted(self) -> None:
        for token in (
            "runtime_archive_inspection_status",
            "runtime_archive_download_index_files",
            "runtime_archive_prior_state_files",
            "runtime_archive_conversation_ledger_files",
            "runtime_archive_process_log_files",
            "runtime_archive_current_loop_evidence_files",
            "runtime_archive_semantic_history_files",
            "runtime_archive_delta_receipt_files",
            "runtime_archive_audit_files",
            "runtime_archive_current_run_summary_present",
            "runtime_archive_runtime_manifest_present",
            "missing_runtime_download_index",
            "missing_runtime_prior_state",
            "missing_runtime_conversation_ledger",
            "missing_runtime_inspection_contract",
        ):
            self.assertIn(token, self.script)

    def test_runtime_archive_inspection_counts_clear_archive_missing_flags(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            files = {
                "runtime/download-index.json": "{}",
                "runtime/prior-state.json": "{}",
                "runtime/conversation-ledger.ndjson": "",
                "runtime/current-run-summary.json": "{}",
                "runtime/runtime-manifest.json": '{"base_commit":"base-123"}',
            }
            for rel, content in files.items():
                path = root / rel
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding="utf-8")
            archive = root / "runtime.tar"
            with tarfile.open(archive, "w") as tf:
                for rel in files:
                    tf.add(root / rel, arcname=rel)

            result = self.observe_module.inspect_runtime_archive(str(archive))

        self.assertEqual(result["runtime_archive_inspection_status"], "pass")
        self.assertGreater(result["runtime_archive_download_index_files"], 0)
        self.assertGreater(result["runtime_archive_prior_state_files"], 0)
        self.assertGreater(result["runtime_archive_conversation_ledger_files"], 0)
        self.assertTrue(result["runtime_archive_current_run_summary_present"])
        self.assertTrue(result["runtime_archive_runtime_manifest_present"])
        self.assertEqual(result["runtime_manifest_base_commit"], "base-123")
        self.assertIn('"missing_runtime_download_index": runtime.get("runtime_archive_download_index_files", 0) <= 0', self.script)
        self.assertIn('"missing_runtime_prior_state": runtime.get("runtime_archive_prior_state_files", 0) <= 0', self.script)
        self.assertIn('"missing_runtime_conversation_ledger": runtime.get("runtime_archive_conversation_ledger_files", 0) <= 0', self.script)


    def test_runtime_manifest_base_match_uses_archive_manifest_commit(self) -> None:
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            manifest = root / "runtime" / "runtime-manifest.json"
            manifest.parent.mkdir(parents=True, exist_ok=True)
            manifest.write_text('{"base_commit":"abc123"}', encoding="utf-8")
            archive = root / "runtime.tar"
            with tarfile.open(archive, "w") as tf:
                tf.add(manifest, arcname="runtime/runtime-manifest.json")

            runtime = self.observe_module.inspect_runtime_archive(str(archive))

        self.assertEqual(runtime["runtime_manifest_base_commit"], "abc123")
        self.assertTrue("abc123" and runtime.get("runtime_manifest_base_commit") == "abc123")
        self.assertFalse("other" and runtime.get("runtime_manifest_base_commit") == "other")

    def test_connector_failure_classification_is_emitted(self) -> None:
        for token in (
            "connector_failure_classification_present",
            "connector_failure_present",
            "connector_failure_status",
            "connector_failure_classes",
            "connector_transport_instability_present",
            "missing_connector_failure_classification",
            "connector_failure_class",
            "environment_failure",
            "command_timeout",
        ):
            self.assertIn(token, self.script)

    def test_receipt_replay_classification_report_is_emitted(self) -> None:
        for token in (
            "receipt_replay_classification_present",
            "receipt_replay_classifications",
            "receipt_replay_classification_evidence_files",
            "receipt_replay_classification_evidence_tokens",
            "missing_receipt_replay_classification_report",
            "receipt_replay_classification_test_present",
            "forged_receipt",
            "duplicated_receipt",
            "reordered_receipt",
            "stale_receipt",
            "missing_receipt",
        ):
            self.assertIn(token, self.script)


    def test_graph_evidence_classification_report_is_emitted(self) -> None:
        for token in (
            "def graph_evidence_classification(",
            "graph_evidence_classification_present",
            "graph_evidence_status",
            "graph_evidence_status_options",
            "graph_wrapper_absent_by_configuration",
            "graph_wrapper_configured_missing",
            "graph_wrapper_configured_no_telemetry",
            "graph_mutation_evidence_contract_missing",
            "graph_mutation_evidence_emitted_not_landed",
            "graph_mutation_landed_without_receipt_ledger",
            "graph_mutation_landed_with_receipt_snapshot",
            "graph_source_contract_present",
            "graph_source_contract_evidence_files",
            "graph_source_contract_evidence_tokens",
            "graph_workflow_contract_present",
            "graph_workflow_contract_evidence_files",
            "graph_workflow_contract_evidence_tokens",
            "missing_graph_source_contract_report",
            "missing_graph_workflow_contract_report",
            "GRAPH_MUTATION_SCHEMA_VERSION",
            "GraphMutationReceipt",
            "GraphPatchReceipt",
            "verify_graph_mutation_landing",
            "verify_graph_receipt_ledger_files_ndjson",
            "graph_mutation_cli_contract",
            "graph_mutation_cli_workflow",
            "def inspect_graph_workflow_fixture()",
            "from validate_graph_workflow_fixture import graph_fixture_report",
            "def graph_workflow_fixture_fields(",
            "graph_workflow_fixture_validation",
            "validate_graph_workflow_fixture.py",
            "graph_workflow_fixture_validation_result",
            "graph_workflow_fixture_report_path",
            "graph-workflow-fixture.json",
            "graph_workflow_fixture_present",
            "graph_workflow_fixture_status",
            "graph_workflow_fixture_evidence_files",
            "graph_workflow_fixture_integrity_valid",
            "graph_workflow_fixture_commands_present",
            "graph_workflow_fixture_landing_command_present",
            "graph_workflow_fixture_ledger_command_present",
            "graph_workflow_fixture_receipt_snapshot_present",
            "graph_workflow_fixture_command_sequence_valid",
            "graph_workflow_fixture_generated_outputs_present",
            "graph_workflow_fixture_receipt_ledger_flow_valid",
            "missing_graph_workflow_fixture_receipt_snapshot",
            "fixture_receipt_snapshot_present",
            "graph_mutation_cli_workflow",
        ):
            self.assertIn(token, self.script + self.graph_fixture_validator)

    def test_graph_fixture_report_mode_is_report_only(self) -> None:
        for token in (
            "--graph-fixture-report",
            "def emit_graph_fixture_report()",
            "graph_fixture_report",
            "graph_fixture_report_only",
            "graph_fixture_report_command",
            "graph_fixture_validator",
            "usage: observe_validation.sh [--graph-fixture-report]",
            "return emit_graph_fixture_report()",
        ):
            self.assertIn(token, self.script + self.graph_fixture_validator)

    def test_ignored_artifact_counts_are_emitted(self) -> None:
        for token in (
            "ignored_artifact_count",
            "ignored_target_artifact_count",
            "ignored_runtime_artifact_count",
            "ignored_validation_artifact_count",
            "git status --ignored",
        ):
            self.assertIn(token, self.script)

    def test_router_tests_are_not_required_when_unavailable(self) -> None:
        self.assertIn('missing_router_offline_tests', self.script)
        self.assertIn('if router_test.get("available"):', self.script)
        self.assertIn('required.add("router_offline_tests")', self.script)
        self.assertIn("def router_offline_classification(", self.script)
        self.assertIn("router_offline_test_classification_present", self.script)
        self.assertIn("router_offline_test_classification_options", self.script)
        self.assertIn("router_offline_unavailable", self.script)
        self.assertIn("router_offline_available_passed", self.script)
        self.assertIn("router_offline_available_not_passed", self.script)
        self.assertIn('missing["missing_router_offline_tests"] = router_offline["router_offline_test_missing"]', self.script)

    def test_router_offline_classifier_executes_all_status_branches(self) -> None:
        cases = (
            (
                {"available": False, "status": "skipped_env_missing"},
                {
                    "router_offline_test_available": False,
                    "router_offline_test_status": "skipped_env_missing",
                    "router_offline_test_classification": "router_offline_unavailable",
                    "router_offline_test_reason": "router offline test harness is not configured in this environment",
                    "router_offline_test_missing": False,
                },
            ),
            (
                {"available": True, "status": "pass", "evidence_files": ["tests/router_offline_contract.rs"]},
                {
                    "router_offline_test_available": True,
                    "router_offline_test_status": "pass",
                    "router_offline_test_classification": "router_offline_available_passed",
                    "router_offline_test_reason": "router offline test harness is available and passed",
                    "router_offline_test_missing": False,
                    "router_offline_test_evidence_files": ["tests/router_offline_contract.rs"],
                },
            ),
            (
                {"available": True, "status": "fail"},
                {
                    "router_offline_test_available": True,
                    "router_offline_test_status": "fail",
                    "router_offline_test_classification": "router_offline_available_not_passed",
                    "router_offline_test_reason": "router offline test harness is available but did not pass",
                    "router_offline_test_missing": True,
                },
            ),
        )

        for router_test, expected in cases:
            with self.subTest(router_test=router_test):
                result = self.observe_module.router_offline_classification(router_test)
                for key, value in expected.items():
                    self.assertEqual(result[key], value)


if __name__ == "__main__":
    unittest.main()