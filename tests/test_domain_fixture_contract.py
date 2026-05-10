import json
import unittest
from pathlib import Path

FIXTURE_DIR = Path(__file__).parent / "fixtures" / "domain"
REQUIRED_FIXTURES = {
    "global_signal_macro.json": ("GlobalIntelligence", "ObservationRecord", "Watch"),
    "business_workflow_opportunity.json": ("Business", "PlanRecord", "ActBusiness"),
    "finance_hypothesis_research.json": ("Finance", "PlanRecord", "ActFinanceResearch"),
    "trading_simulation_sandbox.json": ("TradingSandbox", "PlanRecord", "SimulateTrading"),
    "trading_live_blocked.json": ("TradingSandbox", "Blocked", "Block"),
}
SCORE_FIELDS = {
    "opportunity",
    "confidence",
    "policy_fit",
    "verification_readiness",
    "risk",
    "uncertainty",
    "staleness_penalty",
    "source_quality",
    "context_quality",
}


def bounded_product(*values: int) -> int:
    product = 1000
    for value in values:
        product = (product * value) // 1000
    return max(0, min(1000, product))


def domain_value_score(inputs: dict) -> int:
    positive = bounded_product(
        inputs["opportunity"],
        inputs["confidence"],
        inputs["policy_fit"],
        inputs["verification_readiness"],
    )
    penalty = bounded_product(inputs["risk"], inputs["uncertainty"], inputs["staleness_penalty"])
    return max(0, min(1000, positive - penalty))


def actionability_score(domain_value: int, inputs: dict) -> int:
    return bounded_product(domain_value, inputs["source_quality"], inputs["context_quality"])


class DomainFixtureContract(unittest.TestCase):
    def load_fixture(self, name: str) -> dict:
        path = FIXTURE_DIR / name
        self.assertTrue(path.exists(), f"missing domain fixture {name}")
        with path.open("r", encoding="utf-8") as handle:
            return json.load(handle)

    def test_required_fixtures_exist_with_expected_domain_target_and_verdict(self):
        for name, (domain_id, bridge_target, verdict) in REQUIRED_FIXTURES.items():
            with self.subTest(name=name):
                fixture = self.load_fixture(name)
                self.assertEqual(fixture["schema_version"], "canon_domain_fixture_v1")
                self.assertEqual(fixture["domain_id"], domain_id)
                self.assertEqual(fixture["expected_bridge_target"], bridge_target)
                self.assertEqual(fixture["expected_verdict"], verdict)

    def test_fixtures_include_provenance_uncertainty_and_eval_behavior(self):
        for name in REQUIRED_FIXTURES:
            with self.subTest(name=name):
                fixture = self.load_fixture(name)
                self.assertTrue(fixture["fixture_id"].strip())
                self.assertTrue(fixture["source_id"].strip())
                self.assertTrue(fixture["provenance_hash"].startswith("sha256:"))
                self.assertTrue(fixture["payload_hash"].startswith("sha256:"))
                uncertainty = fixture["uncertainty"]
                self.assertIsInstance(uncertainty["drivers"], list)
                self.assertGreater(len(uncertainty["drivers"]), 0)
                self.assertEqual(uncertainty["score"], fixture["score_inputs"]["uncertainty"])
                self.assertTrue(fixture["expected_eval_behavior"].strip())

    def test_fixture_scores_are_bounded_and_match_integer_equations(self):
        for name in REQUIRED_FIXTURES:
            with self.subTest(name=name):
                fixture = self.load_fixture(name)
                inputs = fixture["score_inputs"]
                self.assertEqual(set(inputs), SCORE_FIELDS)
                for field, value in inputs.items():
                    self.assertIsInstance(value, int, field)
                    self.assertGreaterEqual(value, 0, field)
                    self.assertLessEqual(value, 1000, field)
                expected_domain_value = domain_value_score(inputs)
                expected_actionability = actionability_score(expected_domain_value, inputs)
                self.assertEqual(fixture["expected_domain_value_score"], expected_domain_value)
                self.assertEqual(fixture["expected_actionability_score"], expected_actionability)

    def test_fixtures_map_to_existing_capability_targets_without_runtime_effects(self):
        allowed_targets = {"ObservationRecord", "ContextRecord", "JudgmentRecord", "PlanRecord", "VerificationRecord", "EvalRecord", "PolicyPromotion", "Blocked"}
        for name in REQUIRED_FIXTURES:
            with self.subTest(name=name):
                fixture = self.load_fixture(name)
                self.assertIn(fixture["expected_bridge_target"], allowed_targets)
                self.assertNotIn("CommandEnvelope", fixture)
                self.assertNotIn("TLog", fixture)
                self.assertNotIn("runtime_mutation", fixture)

    def test_trading_fixtures_preserve_sandbox_only_boundary(self):
        sandbox = self.load_fixture("trading_simulation_sandbox.json")
        blocked = self.load_fixture("trading_live_blocked.json")

        self.assertTrue(sandbox["sandbox_only"])
        self.assertFalse(sandbox["live_execution_allowed"])
        self.assertEqual(sandbox["expected_risk_envelope"]["expected_result"], "pass")

        self.assertFalse(blocked["sandbox_only"])
        self.assertTrue(blocked["live_execution_allowed"])
        self.assertEqual(blocked["expected_risk_envelope"]["expected_result"], "block")
        self.assertEqual(blocked["expected_bridge_target"], "Blocked")
        self.assertEqual(blocked["expected_verdict"], "Block")


if __name__ == "__main__":
    unittest.main()
