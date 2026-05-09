#!/usr/bin/env python3
from __future__ import annotations

import re
import unittest
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
        self.assertIn('status": "skipped_env_missing"', self.script)
        self.assertIn('CANON_RUSTC_WRAPPER not found', self.script)

    def test_missing_signals_separate_root_wrapper_and_graph(self) -> None:
        for flag in (
            "missing_cargo_test",
            "missing_wrapper_graph_validation",
            "missing_generated_graph_json",
            "missing_rustc_wrapper_telemetry",
        ):
            self.assertIn(flag, self.script)

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
        ):
            self.assertIn(token, self.script)

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
            "missing_runtime_manifest_base_match",
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
            "graph_workflow_fixture_present",
            "graph_workflow_fixture_status",
            "graph_workflow_fixture_evidence_files",
            "graph_workflow_fixture_integrity_valid",
            "graph_workflow_fixture_commands_present",
            "graph_workflow_fixture_landing_command_present",
            "graph_workflow_fixture_ledger_command_present",
            "graph_workflow_fixture_receipt_snapshot_present",
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


if __name__ == "__main__":
    unittest.main()