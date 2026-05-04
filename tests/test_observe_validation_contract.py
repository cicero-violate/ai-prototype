#!/usr/bin/env python3
from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OBSERVE = ROOT / "scripts" / "observe_validation.sh"
CARGO_CONFIG = ROOT / ".cargo" / "config.toml"


class ObserveValidationContractTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.script = OBSERVE.read_text(encoding="utf-8")
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
            "external_observation_stream_test_present",
            "external_api_action_test_present",
            "semantic_artifact_verification_test_present",
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

    def test_router_tests_are_not_required_when_unavailable(self) -> None:
        self.assertIn('missing_router_offline_tests', self.script)
        self.assertIn('if router_test.get("available"):', self.script)
        self.assertIn('required.add("router_offline_tests")', self.script)


if __name__ == "__main__":
    unittest.main()