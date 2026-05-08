#![forbid(unsafe_code)]

use ai::validation_harness;

struct CompactMode {
    arg: &'static str,
    marker: &'static str,
    run: fn() -> Result<CompactModeOutcome, String>,
}

struct CompactModeOutcome {
    stdout: String,
    exit_code: i32,
}

impl CompactModeOutcome {
    fn pass_json(json: String, passed: bool) -> Self {
        Self {
            stdout: json,
            exit_code: if passed { 0 } else { 1 },
        }
    }

    fn controlled_json(json: String, passed: bool) -> Self {
        Self::pass_json(json, passed)
    }

    fn text(text: String, passed: bool) -> Self {
        Self {
            stdout: text,
            exit_code: if passed { 0 } else { 1 },
        }
    }
}

const COMPACT_MODES: &[CompactMode] = &[
    CompactMode {
        arg: "--validation-footprint",
        marker: "canon_validation_footprint_v1",
        run: validation_footprint_mode,
    },
    CompactMode {
        arg: "--validation-command-footprint-planning",
        marker: "canon_validation_command_footprint_planning_v1",
        run: validation_command_footprint_planning_mode,
    },
    CompactMode {
        arg: "--validation-command-footprint-target-met-smoke",
        marker: "validation_command_footprint_target_met_smoke",
        run: validation_command_footprint_target_met_smoke_mode,
    },
    CompactMode {
        arg: "--validation-command-footprint-unsafe-target-smoke",
        marker: "validation_command_footprint_unsafe_target_smoke",
        run: validation_command_footprint_unsafe_target_smoke_mode,
    },
    CompactMode {
        arg: "--validation-command-footprint-trend-smoke",
        marker: "validation_command_footprint_trend_smoke",
        run: validation_command_footprint_trend_smoke_mode,
    },
    CompactMode {
        arg: "--validation-command-footprint-regression-smoke",
        marker: "validation_command_footprint_regression_smoke",
        run: validation_command_footprint_regression_smoke_mode,
    },
    CompactMode {
        arg: "--validation-command-footprint-fixture",
        marker: "canon_validation_command_footprint_receipts_v1",
        run: validation_command_footprint_fixture_mode,
    },
    CompactMode {
        arg: "--validation-fixture-catalog",
        marker: "canon_validation_fixture_catalog_v1",
        run: validation_fixture_catalog_mode,
    },
    CompactMode {
        arg: "--validation-fixture-catalog-detail",
        marker: "canon_validation_fixture_catalog_detail_v1",
        run: validation_fixture_catalog_detail_mode,
    },
    CompactMode {
        arg: "--validation-duration-planning",
        marker: "canon_validation_duration_planning_v1",
        run: validation_duration_planning_mode,
    },
    CompactMode {
        arg: "--validation-duration-planning-budget-exhaustion-smoke",
        marker: "validation_duration_planning_budget_exhaustion_smoke",
        run: validation_duration_planning_budget_exhaustion_smoke_mode,
    },
    CompactMode {
        arg: "--validation-duration-planning-trend-smoke",
        marker: "validation_duration_planning_trend_smoke",
        run: validation_duration_planning_trend_smoke_mode,
    },
    CompactMode {
        arg: "--validation-duration-planning-regression-smoke",
        marker: "validation_duration_planning_regression_smoke",
        run: validation_duration_planning_regression_smoke_mode,
    },
    CompactMode {
        arg: "--validation-duration-planning-fixture",
        marker: "canon_validation_duration_planning_receipts_v1",
        run: validation_duration_planning_fixture_mode,
    },
    CompactMode {
        arg: "--runtime-budget-smoke",
        marker: "runtime_performance_budget_smoke",
        run: runtime_budget_smoke_mode,
    },
    CompactMode {
        arg: "--runtime-trend-smoke",
        marker: "runtime_performance_trend_smoke",
        run: runtime_trend_smoke_mode,
    },
    CompactMode {
        arg: "--runtime-trend-regression-smoke",
        marker: "runtime_performance_trend_regression_smoke",
        run: runtime_trend_regression_smoke_mode,
    },
    CompactMode {
        arg: "--validation-cost-smoke",
        marker: "validation_cost_footprint_smoke",
        run: validation_cost_smoke_mode,
    },
    CompactMode {
        arg: "--validation-cost-growth-smoke",
        marker: "validation_cost_footprint_growth_smoke",
        run: validation_cost_growth_smoke_mode,
    },
    CompactMode {
        arg: "--policy-orchestration-capacity-trend-smoke",
        marker: "policy_orchestration_capacity_trend_smoke",
        run: policy_orchestration_capacity_trend_smoke_mode,
    },
    CompactMode {
        arg: "--policy-orchestration-capacity-regression-smoke",
        marker: "policy_orchestration_capacity_regression_smoke",
        run: policy_orchestration_capacity_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-capacity-cost-summary-smoke",
        marker: "policy_capacity_cost_summary_smoke",
        run: policy_capacity_cost_summary_smoke_mode,
    },
    CompactMode {
        arg: "--policy-capacity-cost-summary-growth-smoke",
        marker: "policy_capacity_cost_summary_growth_smoke",
        run: policy_capacity_cost_summary_growth_smoke_mode,
    },
    CompactMode {
        arg: "--policy-capacity-cost-summary-trend-smoke",
        marker: "policy_capacity_cost_summary_trend_smoke",
        run: policy_capacity_cost_summary_trend_smoke_mode,
    },
    CompactMode {
        arg: "--policy-capacity-cost-summary-regression-smoke",
        marker: "policy_capacity_cost_summary_regression_smoke",
        run: policy_capacity_cost_summary_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-capacity-cost-summary-fixture",
        marker: "canon_policy_capacity_cost_summary_receipts_v1",
        run: policy_capacity_cost_summary_fixture_mode,
    },
    CompactMode {
        arg: "--policy-orchestration-capacity-smoke",
        marker: "policy_orchestration_capacity_smoke",
        run: policy_orchestration_capacity_smoke_mode,
    },
    CompactMode {
        arg: "--policy-orchestration-capacity-fixture",
        marker: "canon_policy_orchestration_capacity_receipts_v1",
        run: policy_orchestration_capacity_fixture_mode,
    },
    CompactMode {
        arg: "--policy-validation-health-smoke",
        marker: "policy_validation_health_smoke",
        run: policy_validation_health_smoke_mode,
    },
    CompactMode {
        arg: "--policy-validation-health-trend-smoke",
        marker: "policy_validation_health_trend_smoke",
        run: policy_validation_health_trend_smoke_mode,
    },
    CompactMode {
        arg: "--policy-validation-health-fixture",
        marker: "canon_policy_validation_health_receipts_v1",
        run: policy_validation_health_fixture_mode,
    },
    CompactMode {
        arg: "--policy-validation-health-trend-fixture",
        marker: "canon_policy_validation_health_trend_receipts_v1",
        run: policy_validation_health_trend_fixture_mode,
    },
    CompactMode {
        arg: "--policy-reuse-smoke",
        marker: "policy_reuse_smoke",
        run: policy_reuse_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-trend-smoke",
        marker: "policy_reuse_trend_smoke",
        run: policy_reuse_trend_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-regression-smoke",
        marker: "policy_reuse_regression_smoke",
        run: policy_reuse_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-ledger-summary-smoke",
        marker: "policy_reuse_ledger_summary_smoke",
        run: policy_reuse_ledger_summary_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-ledger-summary-regression-smoke",
        marker: "policy_reuse_ledger_summary_regression_smoke",
        run: policy_reuse_ledger_summary_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-scale-trace-smoke",
        marker: "policy_reuse_scale_trace_smoke",
        run: policy_reuse_scale_trace_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-scale-trace-regression-smoke",
        marker: "policy_reuse_scale_trace_regression_smoke",
        run: policy_reuse_scale_trace_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-performance-cost-trend-smoke",
        marker: "policy_reuse_performance_cost_trend_smoke",
        run: policy_reuse_performance_cost_trend_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-performance-cost-trend-regression-smoke",
        marker: "policy_reuse_performance_cost_trend_regression_smoke",
        run: policy_reuse_performance_cost_trend_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-cost-catalog-smoke",
        marker: "policy_reuse_cost_catalog_smoke",
        run: policy_reuse_cost_catalog_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-cost-catalog-incomplete-smoke",
        marker: "policy_reuse_cost_catalog_incomplete_smoke",
        run: policy_reuse_cost_catalog_incomplete_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evaluator-savings-smoke",
        marker: "policy_reuse_evaluator_savings_smoke",
        run: policy_reuse_evaluator_savings_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evaluator-savings-regression-smoke",
        marker: "policy_reuse_evaluator_savings_regression_smoke",
        run: policy_reuse_evaluator_savings_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-scaling-projection-smoke",
        marker: "policy_reuse_scaling_projection_smoke",
        run: policy_reuse_scaling_projection_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-scaling-projection-regression-smoke",
        marker: "policy_reuse_scaling_projection_regression_smoke",
        run: policy_reuse_scaling_projection_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-distillation-readiness-smoke",
        marker: "policy_reuse_distillation_readiness_smoke",
        run: policy_reuse_distillation_readiness_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-distillation-readiness-regression-smoke",
        marker: "policy_reuse_distillation_readiness_regression_smoke",
        run: policy_reuse_distillation_readiness_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-surface-index-smoke",
        marker: "policy_reuse_evidence_surface_index_smoke",
        run: policy_reuse_evidence_surface_index_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-surface-index-regression-smoke",
        marker: "policy_reuse_evidence_surface_index_regression_smoke",
        run: policy_reuse_evidence_surface_index_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-bundle-smoke",
        marker: "policy_reuse_evidence_bundle_smoke",
        run: policy_reuse_evidence_bundle_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-bundle-regression-smoke",
        marker: "policy_reuse_evidence_bundle_regression_smoke",
        run: policy_reuse_evidence_bundle_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-quickcheck-smoke",
        marker: "policy_reuse_evidence_quickcheck_smoke",
        run: policy_reuse_evidence_quickcheck_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-quickcheck-regression-smoke",
        marker: "policy_reuse_evidence_quickcheck_regression_smoke",
        run: policy_reuse_evidence_quickcheck_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-maturity-smoke",
        marker: "policy_reuse_evidence_maturity_smoke",
        run: policy_reuse_evidence_maturity_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-maturity-regression-smoke",
        marker: "policy_reuse_evidence_maturity_regression_smoke",
        run: policy_reuse_evidence_maturity_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-summary-smoke",
        marker: "policy_reuse_evidence_summary_smoke",
        run: policy_reuse_evidence_summary_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-summary-regression-smoke",
        marker: "policy_reuse_evidence_summary_regression_smoke",
        run: policy_reuse_evidence_summary_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-manifest-smoke",
        marker: "policy_reuse_evidence_manifest_smoke",
        run: policy_reuse_evidence_manifest_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-manifest-regression-smoke",
        marker: "policy_reuse_evidence_manifest_regression_smoke",
        run: policy_reuse_evidence_manifest_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-validation-budget-smoke",
        marker: "policy_reuse_evidence_validation_budget_smoke",
        run: policy_reuse_evidence_validation_budget_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-validation-budget-regression-smoke",
        marker: "policy_reuse_evidence_validation_budget_regression_smoke",
        run: policy_reuse_evidence_validation_budget_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-rollout-readiness-smoke",
        marker: "policy_reuse_evidence_rollout_readiness_smoke",
        run: policy_reuse_evidence_rollout_readiness_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-rollout-readiness-regression-smoke",
        marker: "policy_reuse_evidence_rollout_readiness_regression_smoke",
        run: policy_reuse_evidence_rollout_readiness_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-learning-admission-smoke",
        marker: "policy_reuse_evidence_learning_admission_smoke",
        run: policy_reuse_evidence_learning_admission_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-learning-admission-regression-smoke",
        marker: "policy_reuse_evidence_learning_admission_regression_smoke",
        run: policy_reuse_evidence_learning_admission_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-readiness-smoke",
        marker: "policy_reuse_evidence_retrieval_readiness_smoke",
        run: policy_reuse_evidence_retrieval_readiness_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-readiness-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_readiness_regression_smoke",
        run: policy_reuse_evidence_retrieval_readiness_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-compact-validation-smoke",
        marker: "policy_reuse_evidence_compact_validation_smoke",
        run: policy_reuse_evidence_compact_validation_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-compact-validation-regression-smoke",
        marker: "policy_reuse_evidence_compact_validation_regression_smoke",
        run: policy_reuse_evidence_compact_validation_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-batch-readiness-smoke",
        marker: "policy_reuse_evidence_batch_readiness_smoke",
        run: policy_reuse_evidence_batch_readiness_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-batch-readiness-regression-smoke",
        marker: "policy_reuse_evidence_batch_readiness_regression_smoke",
        run: policy_reuse_evidence_batch_readiness_regression_smoke_mode,
    },
    CompactMode {
        arg: "--root-validate-dispatch-catalog",
        marker: "canon_root_validate_dispatch_catalog_v1",
        run: root_validate_dispatch_catalog_mode,
    },
];

fn validation_footprint_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::validation_footprint_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn validation_command_footprint_planning_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::validation_command_footprint_planning_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn validation_command_footprint_target_met_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::validation_command_footprint_target_met_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn validation_command_footprint_unsafe_target_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::validation_command_footprint_unsafe_target_smoke_receipt();
    let passed = receipt.safety_status == "fail"
        && receipt.planning_status == "unsafe_target"
        && receipt.verdict == "fail";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn validation_command_footprint_trend_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::validation_command_footprint_trend_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn validation_command_footprint_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::validation_command_footprint_regression_smoke_receipt();
    let passed = receipt.trend_status == "regressed" && receipt.verdict == "fail";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn validation_command_footprint_fixture_mode() -> Result<CompactModeOutcome, String> {
    retained_fixture_text_mode(
        validation_harness::VALIDATION_COMMAND_FOOTPRINT_RECEIPTS_FIXTURE,
        "canon_validation_command_footprint_receipts_v1",
        5,
    )
}

fn validation_fixture_catalog_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::validation_fixture_catalog_receipt()?;
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn validation_fixture_catalog_detail_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::validation_fixture_catalog_detail_receipt()?;
    Ok(CompactModeOutcome::text(
        receipt.to_text(),
        receipt.passed(),
    ))
}

fn validation_duration_planning_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::validation_duration_planning_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn validation_duration_planning_budget_exhaustion_smoke_mode() -> Result<CompactModeOutcome, String>
{
    let receipt =
        validation_harness::validation_duration_planning_budget_exhaustion_smoke_receipt();
    let passed = receipt.runtime_budget_status == "fail"
        && receipt.planning_status == "fail"
        && receipt.verdict == "fail"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn validation_duration_planning_trend_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::validation_duration_planning_trend_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn validation_duration_planning_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::validation_duration_planning_regression_smoke_receipt();
    let passed = receipt.trend_status == "regressed" && receipt.verdict == "fail";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn retained_fixture_text_mode(
    fixture_path: &str,
    schema: &str,
    receipt_count: usize,
) -> Result<CompactModeOutcome, String> {
    let fixture = std::fs::read_to_string(fixture_path).map_err(|err| err.to_string())?;
    let passed = validation_harness::retained_fixture_header_valid(&fixture, schema, receipt_count);
    Ok(CompactModeOutcome::text(fixture, passed))
}

fn validation_duration_planning_fixture_mode() -> Result<CompactModeOutcome, String> {
    retained_fixture_text_mode(
        validation_harness::VALIDATION_DURATION_PLANNING_RECEIPTS_FIXTURE,
        "canon_validation_duration_planning_receipts_v1",
        4,
    )
}

fn runtime_budget_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::runtime_performance_budget_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn runtime_trend_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::runtime_performance_trend_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn runtime_trend_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::runtime_performance_trend_regression_smoke_receipt();
    let passed = receipt.budget_status == "pass" && receipt.trend_status == "fail";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn validation_cost_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::validation_cost_footprint_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn validation_cost_growth_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::validation_cost_footprint_growth_smoke_receipt();
    let passed = receipt.budget_status == "pass"
        && receipt.trend_status == "pass"
        && receipt.footprint_status == "fail"
        && receipt.verdict == "fail";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_orchestration_capacity_trend_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_orchestration_capacity_trend_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_orchestration_capacity_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_orchestration_capacity_regression_smoke_receipt();
    let passed = receipt.trend_status == "regressed"
        && receipt.verdict == "fail"
        && receipt.baseline_capacity_status == "pass"
        && receipt.current_capacity_status == "pass";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_orchestration_capacity_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_orchestration_capacity_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_capacity_cost_summary_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_capacity_cost_summary_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_capacity_cost_summary_growth_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_capacity_cost_summary_growth_smoke_receipt();
    let passed = receipt.policy_capacity_status == "pass"
        && receipt.validation_cost_verdict == "fail"
        && receipt.summary_status == "fail"
        && receipt.verdict == "fail";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_capacity_cost_summary_trend_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_capacity_cost_summary_trend_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_capacity_cost_summary_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_capacity_cost_summary_regression_smoke_receipt();
    let passed = receipt.trend_status == "regressed" && receipt.verdict == "fail";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_capacity_cost_summary_fixture_mode() -> Result<CompactModeOutcome, String> {
    retained_fixture_text_mode(
        validation_harness::POLICY_CAPACITY_COST_SUMMARY_RECEIPTS_FIXTURE,
        "canon_policy_capacity_cost_summary_receipts_v1",
        4,
    )
}

fn policy_orchestration_capacity_fixture_mode() -> Result<CompactModeOutcome, String> {
    retained_fixture_text_mode(
        validation_harness::POLICY_ORCHESTRATION_CAPACITY_RECEIPTS_FIXTURE,
        "canon_policy_orchestration_capacity_receipts_v1",
        3,
    )
}

fn policy_validation_health_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_validation_health_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_validation_health_fixture_mode() -> Result<CompactModeOutcome, String> {
    retained_fixture_text_mode(
        validation_harness::POLICY_VALIDATION_HEALTH_RECEIPTS_FIXTURE,
        "canon_policy_validation_health_receipts_v1",
        1,
    )
}

fn policy_validation_health_trend_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_validation_health_trend_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_validation_health_trend_fixture_mode() -> Result<CompactModeOutcome, String> {
    retained_fixture_text_mode(
        validation_harness::POLICY_VALIDATION_HEALTH_TREND_RECEIPTS_FIXTURE,
        "canon_policy_validation_health_trend_receipts_v1",
        1,
    )
}

fn policy_reuse_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_trend_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_trend_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_regression_smoke_receipt();
    let passed = receipt.trend_status == "regressed" && receipt.verdict == "fail";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_ledger_summary_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_ledger_summary_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_ledger_summary_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_ledger_summary_regression_smoke_receipt();
    let passed = receipt.is_valid() && receipt.regression_flag && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_scale_trace_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_scale_trace_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_scale_trace_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_scale_trace_regression_smoke_receipt();
    let passed = receipt.is_valid() && receipt.regression_flag && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_performance_cost_trend_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_performance_cost_trend_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_performance_cost_trend_regression_smoke_mode() -> Result<CompactModeOutcome, String>
{
    let receipt =
        validation_harness::policy_reuse_performance_cost_trend_regression_smoke_receipt();
    let passed = receipt.is_valid() && receipt.cost_regression_flag && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_cost_catalog_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_cost_catalog_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_cost_catalog_incomplete_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_cost_catalog_incomplete_smoke_receipt();
    let passed = receipt.is_valid() && !receipt.summary_complete && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evaluator_savings_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evaluator_savings_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evaluator_savings_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evaluator_savings_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.validation_passed
        && receipt.regression_reason == "catalog_incomplete"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_scaling_projection_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_scaling_projection_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_scaling_projection_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_scaling_projection_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.projection_passed
        && receipt.regression_reason == "evaluator_savings_failed"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_distillation_readiness_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_distillation_readiness_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_distillation_readiness_regression_smoke_mode() -> Result<CompactModeOutcome, String>
{
    let receipt =
        validation_harness::policy_reuse_distillation_readiness_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.distillation_ready
        && receipt.regression_reason == "catalog_incomplete"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_surface_index_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_surface_index_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_surface_index_regression_smoke_mode() -> Result<CompactModeOutcome, String>
{
    let receipt =
        validation_harness::policy_reuse_evidence_surface_index_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.index_complete
        && receipt.missing_surface == "required_regression_modes"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_bundle_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_bundle_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_bundle_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_bundle_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.bundle_complete
        && receipt.regression_reason == "surface_index_incomplete"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_quickcheck_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_quickcheck_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_quickcheck_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_quickcheck_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.quickcheck_passed
        && receipt.missing_command == "validation_harness_contract"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_maturity_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_maturity_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_maturity_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_maturity_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.promotion_eligible
        && receipt.maturity_stage == "immature"
        && receipt.regression_reason == "quickcheck_failed"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_summary_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_summary_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_summary_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_summary_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && receipt.summary_status == "fail"
        && receipt.evaluator_action == "inspect_maturity"
        && receipt.regression_reason == "maturity_immature"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_manifest_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_manifest_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_manifest_regression_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_manifest_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.manifest_complete
        && receipt.missing_surface == "required_summary_modes"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_validation_budget_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_validation_budget_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_validation_budget_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_validation_budget_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.budget_within_limit
        && receipt.budget_status == "fail"
        && receipt.regression_reason == "budget_exceeded"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_rollout_readiness_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_rollout_readiness_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_rollout_readiness_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_rollout_readiness_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.validation_budget_passed
        && !receipt.rollout_ready
        && receipt.readiness_status == "not_ready"
        && receipt.not_ready_reason == "validation_budget_failed"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_learning_admission_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_learning_admission_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_learning_admission_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_learning_admission_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.rollout_ready
        && !receipt.validation_budget_passed
        && !receipt.learning_data_admissible
        && receipt.admission_status == "not_admissible"
        && receipt.not_admissible_reason == "rollout_not_ready"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_readiness_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_retrieval_readiness_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_readiness_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_readiness_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.learning_data_admissible
        && !receipt.rollout_ready
        && !receipt.retrieval_storage_write_performed
        && !receipt.retrieval_example_ready
        && receipt.retrieval_status == "not_ready"
        && receipt.not_ready_reason == "learning_not_admissible"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_compact_validation_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_compact_validation_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_compact_validation_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_compact_validation_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_ready
        && !receipt.learning_data_admissible
        && !receipt.validation_budget_passed
        && receipt.targeted_test_count > receipt.max_targeted_test_count
        && !receipt.compact_validation_passed
        && receipt.compact_validation_status == "fail"
        && receipt.failure_reason == "retrieval_not_ready"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn stable_hash64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn root_validate_dispatch_catalog_payload() -> String {
    COMPACT_MODES
        .iter()
        .map(|mode| format!("{}=>{}", mode.arg, mode.marker))
        .collect::<Vec<_>>()
        .join("\n")
}

fn root_validate_dispatch_catalog_mode() -> Result<CompactModeOutcome, String> {
    let modes = COMPACT_MODES
        .iter()
        .map(|mode| {
            format!(
                "{{\"arg\":\"{}\",\"marker\":\"{}\"}}",
                mode.arg, mode.marker
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let catalog_payload = root_validate_dispatch_catalog_payload();
    let dispatch_catalog_hash = stable_hash64(catalog_payload.as_bytes());
    let json = format!(
        "{{\"schema\":\"canon_root_validate_dispatch_catalog_v1\",\"record_type\":\"root_validate_dispatch_catalog\",\"compact_mode_count\":{},\"dispatch_catalog_hash\":\"{}\",\"compact_modes\":[{}],\"verdict\":\"pass\"}}",
        COMPACT_MODES.len(),
        dispatch_catalog_hash,
        modes
    );
    Ok(CompactModeOutcome::pass_json(json, true))
}

fn try_run_compact_mode(arg: &str) -> Option<Result<CompactModeOutcome, String>> {
    COMPACT_MODES
        .iter()
        .find(|mode| mode.arg == arg)
        .map(|mode| (mode.run)())
}

fn main() {
    let arg = std::env::args().nth(1);
    if let Some(arg) = arg.as_deref() {
        if let Some(outcome) = try_run_compact_mode(arg) {
            match outcome {
                Ok(outcome) => {
                    print!("{}", outcome.stdout);
                    if !outcome.stdout.ends_with('\n') {
                        println!();
                    }
                    if outcome.exit_code != 0 {
                        std::process::exit(outcome.exit_code);
                    }
                }
                Err(err) => {
                    eprintln!("{err}");
                    std::process::exit(2);
                }
            }
            return;
        }
    }

    match validation_harness::validate_root() {
        Ok(receipt) => {
            println!("{}", receipt.to_json_line());
            if !receipt.passed() {
                std::process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}

fn policy_reuse_evidence_batch_readiness_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_batch_readiness_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_batch_readiness_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_batch_readiness_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.compact_validation_passed
        && !receipt.retrieval_ready
        && !receipt.scaling_projection_passed
        && !receipt.batch_ready
        && receipt.batch_readiness_status == "not_ready"
        && receipt.not_ready_reason == "compact_validation_failed"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}
