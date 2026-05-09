#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "validate_policy_learning_trace.py"
VALIDATOR_TIMEOUT_SECONDS = 30


class PolicyLearningTraceContractTest(unittest.TestCase):
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

    def test_current_repo_contains_learning_policy_judgment_trace(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            report = Path(tmp) / "trace.json"
            done = self.run_script(ROOT, report)
            self.assertEqual(done.returncode, 0, done.stderr)
            data = json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual(data["status"], "pass")
            self.assertEqual(data["trace_function"], "learning_policy_llm_feedback_loop_drives_judgment")
            self.assertEqual(data["missing_count"], 0)
            self.assertEqual(len(data["checks"]), 7)
            self.assertIn(
                "distillation_row_contract",
                {check["name"] for check in data["checks"]},
            )

    def test_missing_trace_fails_with_explicit_missing_tokens(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "src" / "capability" / "learning").mkdir(parents=True)
            (root / "src" / "capability" / "policy").mkdir(parents=True)
            (root / "src" / "lib.rs").write_text("#[cfg(test)] mod tests {}\n", encoding="utf-8")
            (root / "src" / "capability" / "learning" / "promote.rs").write_text("", encoding="utf-8")
            (root / "src" / "capability" / "policy" / "store.rs").write_text("", encoding="utf-8")
            report = root / "trace.json"
            done = self.run_script(root, report)
            self.assertNotEqual(done.returncode, 0)
            data = json.loads(report.read_text(encoding="utf-8"))
            self.assertEqual(data["status"], "fail")
            self.assertIn("learning_policy_llm_feedback_loop_drives_judgment", data["missing"])
            self.assertGreater(data["missing_count"], 1)

    def test_policy_store_cannot_own_learning_promotion(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "src" / "capability" / "learning").mkdir(parents=True)
            (root / "src" / "capability" / "policy").mkdir(parents=True)
            (root / "src" / "lib.rs").write_text(
                "fn learning_policy_llm_feedback_loop_drives_judgment() {}\n",
                encoding="utf-8",
            )
            (root / "src" / "capability" / "learning" / "promote.rs").write_text(
                "\n".join(
                    [
                        "pub struct PolicyPromotion",
                        "pub fn from_tlog",
                        "Evidence::PolicyPromotion",
                        "promoted_policy_hash",
                        "source_seq",
                        "pub struct DistillationRow",
                        "pub const DISTILLATION_ROW_SCHEMA_VERSION",
                        "pub fn from_policy_promotion",
                        "instruction_hash",
                        "input_state_hash",
                        "action_hash",
                        "output_hash",
                        "score",
                        "proof_hash != promotion.promoted_policy_hash",
                        "source_event: promotion.source_seq",
                        "pub fn is_valid_for",
                        "expected_row_hash",
                        "impl PolicyStore",
                        "pub fn promote_feedback",
                        "PolicyStoreError::InvalidPromotion",
                        "PolicyEntry",
                        "self.try_append(PolicyEntry",
                        "key: POLICY_FEEDBACK_HASH",
                        "pub fn promote_feedback_durable",
                        "self.append_durable(",
                    ]
                ),
                encoding="utf-8",
            )
            (root / "src" / "capability" / "policy" / "store.rs").write_text(
                "\n".join(
                    [
                        "pub fn try_append",
                        "pub fn append_durable",
                        "key_to_id(entry.key)?",
                        "InvalidPromotion",
                        "PolicyEntry",
                        "pub fn fingerprint",
                        "pub fn latest_version",
                        "pub fn promote_feedback",
                    ]
                ),
                encoding="utf-8",
            )
            report = root / "trace.json"
            done = self.run_script(root, report)
            self.assertNotEqual(done.returncode, 0)
            data = json.loads(report.read_text(encoding="utf-8"))
            self.assertIn("forbidden:pub fn promote_feedback", data["missing"])


if __name__ == "__main__":
    unittest.main()