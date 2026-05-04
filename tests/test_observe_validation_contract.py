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
        self.assertNotRegex(self.config, r'^\s*rustc-wrapper\s*=', re.MULTILINE)
        self.assertIn("CANON_RUSTC_WRAPPER=/path/to/canon-rustc-v3", self.config)

    def test_wrapper_graph_capture_is_explicit_and_optional(self) -> None:
        self.assertIn('os.environ.get("CANON_RUSTC_WRAPPER", "")', self.script)
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


if __name__ == "__main__":
    unittest.main()