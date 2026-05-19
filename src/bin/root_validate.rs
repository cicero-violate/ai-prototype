#![forbid(unsafe_code)]

use ai::validation_harness;
use std::hash::{Hash, Hasher};

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
        Self::pass_json(text, passed)
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
        arg: "--policy-reuse-evidence-batch-execution-plan-smoke",
        marker: "policy_reuse_evidence_batch_execution_plan_smoke",
        run: policy_reuse_evidence_batch_execution_plan_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-batch-execution-plan-regression-smoke",
        marker: "policy_reuse_evidence_batch_execution_plan_regression_smoke",
        run: policy_reuse_evidence_batch_execution_plan_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-batch-evaluation-admission-smoke",
        marker: "policy_reuse_evidence_batch_evaluation_admission_smoke",
        run: policy_reuse_evidence_batch_evaluation_admission_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-batch-evaluation-admission-regression-smoke",
        marker: "policy_reuse_evidence_batch_evaluation_admission_regression_smoke",
        run: policy_reuse_evidence_batch_evaluation_admission_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-batch-run-request-smoke",
        marker: "policy_reuse_evidence_batch_run_request_smoke",
        run: policy_reuse_evidence_batch_run_request_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-batch-run-request-regression-smoke",
        marker: "policy_reuse_evidence_batch_run_request_regression_smoke",
        run: policy_reuse_evidence_batch_run_request_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-external-evaluator-result-smoke",
        marker: "policy_reuse_evidence_external_evaluator_result_smoke",
        run: policy_reuse_evidence_external_evaluator_result_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-external-evaluator-result-regression-smoke",
        marker: "policy_reuse_evidence_external_evaluator_result_regression_smoke",
        run: policy_reuse_evidence_external_evaluator_result_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-learning-candidate-smoke",
        marker: "policy_reuse_evidence_learning_candidate_smoke",
        run: policy_reuse_evidence_learning_candidate_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-learning-candidate-regression-smoke",
        marker: "policy_reuse_evidence_learning_candidate_regression_smoke",
        run: policy_reuse_evidence_learning_candidate_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-learning-data-admission-smoke",
        marker: "policy_reuse_evidence_learning_data_admission_smoke",
        run: policy_reuse_evidence_learning_data_admission_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-learning-data-admission-regression-smoke",
        marker: "policy_reuse_evidence_learning_data_admission_regression_smoke",
        run: policy_reuse_evidence_learning_data_admission_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-example-admission-smoke",
        marker: "policy_reuse_evidence_retrieval_example_admission_smoke",
        run: policy_reuse_evidence_retrieval_example_admission_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-example-admission-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_example_admission_regression_smoke",
        run: policy_reuse_evidence_retrieval_example_admission_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-example-index-smoke",
        marker: "policy_reuse_evidence_retrieval_example_index_smoke",
        run: policy_reuse_evidence_retrieval_example_index_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-example-index-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_example_index_regression_smoke",
        run: policy_reuse_evidence_retrieval_example_index_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-corpus-readiness-smoke",
        marker: "policy_reuse_evidence_retrieval_corpus_readiness_smoke",
        run: policy_reuse_evidence_retrieval_corpus_readiness_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-corpus-readiness-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_corpus_readiness_regression_smoke",
        run: policy_reuse_evidence_retrieval_corpus_readiness_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-corpus-admission-smoke",
        marker: "policy_reuse_evidence_retrieval_corpus_admission_smoke",
        run: policy_reuse_evidence_retrieval_corpus_admission_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-corpus-admission-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_corpus_admission_regression_smoke",
        run: policy_reuse_evidence_retrieval_corpus_admission_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-use-approval-smoke",
        marker: "policy_reuse_evidence_retrieval_use_approval_smoke",
        run: policy_reuse_evidence_retrieval_use_approval_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-use-approval-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_use_approval_regression_smoke",
        run: policy_reuse_evidence_retrieval_use_approval_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-use-manifest-smoke",
        marker: "policy_reuse_evidence_retrieval_use_manifest_smoke",
        run: policy_reuse_evidence_retrieval_use_manifest_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-use-manifest-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_use_manifest_regression_smoke",
        run: policy_reuse_evidence_retrieval_use_manifest_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-query-plan-smoke",
        marker: "policy_reuse_evidence_retrieval_query_plan_smoke",
        run: policy_reuse_evidence_retrieval_query_plan_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-query-plan-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_query_plan_regression_smoke",
        run: policy_reuse_evidence_retrieval_query_plan_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-query-approval-smoke",
        marker: "policy_reuse_evidence_retrieval_query_approval_smoke",
        run: policy_reuse_evidence_retrieval_query_approval_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-query-approval-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_query_approval_regression_smoke",
        run: policy_reuse_evidence_retrieval_query_approval_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-admission-smoke",
        marker: "policy_reuse_evidence_retrieval_result_admission_smoke",
        run: policy_reuse_evidence_retrieval_result_admission_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-admission-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_admission_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_admission_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-manifest-smoke",
        marker: "policy_reuse_evidence_retrieval_result_manifest_smoke",
        run: policy_reuse_evidence_retrieval_result_manifest_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-manifest-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_manifest_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_manifest_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-admission-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_admission_smoke",
        run: policy_reuse_evidence_retrieval_result_use_admission_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-admission-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_admission_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_admission_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-manifest-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_manifest_smoke",
        run: policy_reuse_evidence_retrieval_result_use_manifest_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-manifest-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_manifest_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_manifest_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-readiness-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_readiness_smoke",
        run: policy_reuse_evidence_retrieval_result_use_readiness_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-readiness-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_readiness_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_readiness_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-approval-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_approval_smoke",
        run: policy_reuse_evidence_retrieval_result_use_approval_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-approval-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_approval_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_approval_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-manifest-admission-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_manifest_admission_smoke",
        run: policy_reuse_evidence_retrieval_result_use_manifest_admission_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-manifest-admission-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_manifest_admission_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_manifest_admission_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-admission-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_admission_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_admission_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-admission-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_admission_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_admission_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-readiness-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_readiness_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_readiness_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-readiness-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_readiness_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_readiness_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-eligibility-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_eligibility_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_eligibility_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-eligibility-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_eligibility_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_eligibility_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-admission-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_admission_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_admission_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-admission-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_admission_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_admission_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-materialization-plan-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_materialization_plan_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_materialization_plan_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-materialization-plan-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_materialization_plan_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_materialization_plan_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-admission-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_admission_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_admission_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-admission-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_admission_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_admission_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-commit-intent-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_commit_intent_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_commit_intent_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-commit-intent-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_commit_intent_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_commit_intent_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-write-preflight-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_preflight_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_preflight_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-write-preflight-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_preflight_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_preflight_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-write-approval-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_approval_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_approval_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-write-approval-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_approval_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_approval_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-write-admission-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_admission_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_admission_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-write-admission-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_admission_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_admission_regression_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-write-commit-intent-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_commit_intent_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_commit_intent_smoke_mode,
    },
    CompactMode {
        arg: "--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-write-commit-intent-regression-smoke",
        marker: "policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_commit_intent_regression_smoke",
        run: policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_commit_intent_regression_smoke_mode,
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

fn all_false(values: &[bool]) -> bool {
    values.iter().all(|value| !value)
}

fn storage_write_commit_intent_regression_gate_passed(
    receipt: &validation_harness::PolicyReuseEvidenceRetrievalResultUseSummaryManifestApprovalAdmissionConsumptionLearningStorageWriteCommitIntentReceipt,
) -> bool {
    receipt.external_result_evidence_present
        && receipt.retrieval_example_storage_write_commit_intent_status
            == "retrieval_example_storage_write_commit_intent_not_ready"
        && receipt.not_ready_reason == "retrieval_example_storage_write_not_admitted"
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

fn policy_reuse_evidence_retrieval_query_approval_smoke_mode() -> Result<CompactModeOutcome, String>
{
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_query_approval_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_query_approval_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_query_approval_regression_smoke_receipt(
        );
    let passed = receipt.is_valid()
        && !receipt.retrieval_query_plan_ready
        && !receipt.retrieval_use_manifest_ready
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.policy_promotion_performed
        && !receipt.student_training_performed
        && !receipt.retrieval_query_approved
        && receipt.query_approval_status == "query_not_approved"
        && receipt.not_approved_reason == "query_plan_not_ready"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_admission_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_result_admission_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_admission_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_admission_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_query_approved
        && !receipt.retrieval_query_plan_ready
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.runtime_result_approval_performed
        && !receipt.policy_promotion_performed
        && !receipt.student_training_performed
        && receipt.external_result_evidence_present
        && !receipt.retrieval_result_admitted
        && receipt.result_admission_status == "result_not_admitted"
        && receipt.not_admitted_reason == "query_not_approved"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_manifest_smoke_mode() -> Result<CompactModeOutcome, String>
{
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_result_manifest_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_common_regression_guards(
    receipt_is_valid: bool,
    retrieval_read_performed: bool,
    retrieval_write_performed: bool,
    retrieval_query_executed: bool,
    runtime_result_approval_performed: bool,
    policy_promotion_performed: bool,
    student_training_performed: bool,
    external_result_evidence_present: bool,
    receipt_passed: bool,
) -> bool {
    receipt_is_valid
        && !retrieval_read_performed
        && !retrieval_write_performed
        && !retrieval_query_executed
        && !runtime_result_approval_performed
        && !policy_promotion_performed
        && !student_training_performed
        && external_result_evidence_present
        && !receipt_passed
}

fn policy_reuse_evidence_retrieval_result_manifest_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_manifest_regression_smoke_receipt();
    let passed = policy_reuse_common_regression_guards(
        receipt.is_valid(),
        receipt.retrieval_read_performed,
        receipt.retrieval_write_performed,
        receipt.retrieval_query_executed,
        receipt.runtime_result_approval_performed,
        receipt.policy_promotion_performed,
        receipt.student_training_performed,
        receipt.external_result_evidence_present,
        receipt.passed(),
    ) && !receipt.retrieval_result_admitted
        && !receipt.retrieval_query_approved
        && !receipt.retrieval_result_manifest_ready
        && receipt.result_manifest_status == "result_manifest_not_ready"
        && receipt.not_ready_reason == "result_not_admitted";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_admission_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_result_use_admission_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_admission_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_admission_regression_smoke_receipt();
    let passed = policy_reuse_common_regression_guards(
        receipt.is_valid(),
        receipt.retrieval_read_performed,
        receipt.retrieval_write_performed,
        receipt.retrieval_query_executed,
        receipt.runtime_result_approval_performed,
        receipt.policy_promotion_performed,
        receipt.student_training_performed,
        receipt.external_result_evidence_present,
        receipt.passed(),
    ) && !receipt.retrieval_result_manifest_ready
        && !receipt.retrieval_result_admitted
        && !receipt.retrieval_result_use_admitted
        && receipt.result_use_admission_status == "result_use_not_admitted"
        && receipt.not_admitted_reason == "manifest_not_ready";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_manifest_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_result_use_manifest_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_manifest_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_manifest_regression_smoke_receipt();
    let passed = policy_reuse_common_regression_guards(
        receipt.is_valid(),
        receipt.retrieval_read_performed,
        receipt.retrieval_write_performed,
        receipt.retrieval_query_executed,
        receipt.runtime_result_approval_performed,
        receipt.policy_promotion_performed,
        receipt.student_training_performed,
        receipt.external_result_evidence_present,
        receipt.passed(),
    ) && !receipt.retrieval_result_use_admitted
        && !receipt.retrieval_result_manifest_ready
        && !receipt.retrieval_result_use_manifest_ready
        && receipt.result_use_manifest_status == "result_use_manifest_not_ready"
        && receipt.not_ready_reason == "use_not_admitted";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_readiness_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_result_use_readiness_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_readiness_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_readiness_regression_smoke_receipt();
    let passed = policy_reuse_common_regression_guards(
        receipt.is_valid(),
        receipt.retrieval_read_performed,
        receipt.retrieval_write_performed,
        receipt.retrieval_query_executed,
        receipt.runtime_result_approval_performed,
        receipt.policy_promotion_performed,
        receipt.student_training_performed,
        receipt.external_result_evidence_present,
        receipt.passed(),
    ) && !receipt.retrieval_result_use_manifest_ready
        && !receipt.retrieval_result_use_admitted
        && !receipt.retrieval_result_use_ready
        && receipt.result_use_readiness_status == "result_use_not_ready"
        && receipt.not_ready_reason == "manifest_not_ready";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_approval_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_result_use_approval_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_approval_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_approval_regression_smoke_receipt();
    let passed = policy_reuse_common_regression_guards(
        receipt.is_valid(),
        receipt.retrieval_read_performed,
        receipt.retrieval_write_performed,
        receipt.retrieval_query_executed,
        receipt.runtime_result_approval_performed,
        receipt.policy_promotion_performed,
        receipt.student_training_performed,
        receipt.external_result_evidence_present,
        receipt.passed(),
    ) && !receipt.retrieval_result_use_ready
        && !receipt.retrieval_result_use_manifest_ready
        && !receipt.retrieval_result_use_approved
        && receipt.result_use_approval_status == "result_use_not_approved"
        && receipt.not_approved_reason == "readiness_not_ready";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_manifest_admission_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_manifest_admission_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_manifest_admission_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_manifest_admission_regression_smoke_receipt();
    let passed = policy_reuse_common_regression_guards(
        receipt.is_valid(),
        receipt.retrieval_read_performed,
        receipt.retrieval_write_performed,
        receipt.retrieval_query_executed,
        receipt.runtime_result_approval_performed,
        receipt.policy_promotion_performed,
        receipt.student_training_performed,
        receipt.external_result_evidence_present,
        receipt.passed(),
    ) && !receipt.retrieval_result_use_approved
        && !receipt.retrieval_result_use_ready
        && !receipt.retrieval_result_use_manifest_admitted
        && receipt.result_use_manifest_admission_status == "result_use_manifest_not_admitted"
        && receipt.not_admitted_reason == "approval_not_granted";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_result_use_summary_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_regression_smoke_receipt();
    let passed = policy_reuse_common_regression_guards(
        receipt.is_valid(),
        receipt.retrieval_read_performed,
        receipt.retrieval_write_performed,
        receipt.retrieval_query_executed,
        receipt.runtime_result_approval_performed,
        receipt.policy_promotion_performed,
        receipt.student_training_performed,
        receipt.external_result_evidence_present,
        receipt.passed(),
    ) && !receipt.retrieval_result_use_manifest_admitted
        && !receipt.retrieval_result_use_approved
        && !receipt.retrieval_result_use_summary_ready
        && receipt.result_use_summary_status == "result_use_summary_not_ready"
        && receipt.not_ready_reason == "manifest_not_admitted";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_regression_smoke_receipt();
    let passed = policy_reuse_common_regression_guards(
        receipt.is_valid(),
        receipt.retrieval_read_performed,
        receipt.retrieval_write_performed,
        receipt.retrieval_query_executed,
        receipt.runtime_result_approval_performed,
        receipt.policy_promotion_performed,
        receipt.student_training_performed,
        receipt.external_result_evidence_present,
        receipt.passed(),
    ) && !receipt.retrieval_result_use_summary_ready
        && !receipt.retrieval_result_use_manifest_admitted
        && !receipt.retrieval_result_use_summary_manifest_ready
        && receipt.result_use_summary_manifest_status == "result_use_summary_manifest_not_ready"
        && receipt.not_ready_reason == "summary_not_ready";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_admission_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_admission_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_admission_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_admission_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_result_use_summary_manifest_ready
        && !receipt.retrieval_result_use_summary_ready
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.runtime_result_approval_performed
        && !receipt.policy_promotion_performed
        && !receipt.student_training_performed
        && receipt.external_result_evidence_present
        && !receipt.retrieval_result_use_summary_manifest_admitted
        && receipt.result_use_summary_manifest_admission_status
            == "result_use_summary_manifest_not_admitted"
        && receipt.not_admitted_reason == "summary_manifest_not_ready"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_readiness_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_readiness_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_readiness_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_readiness_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_result_use_summary_manifest_admitted
        && !receipt.retrieval_result_use_summary_manifest_ready
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.runtime_result_approval_performed
        && !receipt.policy_promotion_performed
        && !receipt.student_training_performed
        && receipt.external_result_evidence_present
        && !receipt.retrieval_result_use_summary_manifest_ready_for_use
        && receipt.result_use_summary_manifest_readiness_status
            == "result_use_summary_manifest_not_ready_for_use"
        && receipt.not_ready_reason == "summary_manifest_not_admitted"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_result_use_summary_manifest_ready_for_use
        && !receipt.retrieval_result_use_summary_manifest_admitted
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.runtime_result_approval_performed
        && !receipt.policy_promotion_performed
        && !receipt.student_training_performed
        && receipt.external_result_evidence_present
        && !receipt.retrieval_result_use_summary_manifest_approved
        && receipt.result_use_summary_manifest_approval_status
            == "result_use_summary_manifest_not_approved"
        && receipt.not_approved_reason == "summary_manifest_not_ready_for_use"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_result_use_summary_manifest_approved
        && !receipt.retrieval_result_use_summary_manifest_ready_for_use
        && !receipt.retrieval_result_use_summary_manifest_admitted
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.runtime_result_approval_performed
        && !receipt.policy_promotion_performed
        && !receipt.student_training_performed
        && receipt.external_result_evidence_present
        && !receipt.retrieval_result_use_summary_manifest_approval_admitted
        && receipt.result_use_summary_manifest_approval_admission_status
            == "result_use_summary_manifest_approval_not_admitted"
        && receipt.not_admitted_reason == "summary_manifest_not_approved"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_result_use_summary_manifest_approval_admitted
        && !receipt.retrieval_result_use_summary_manifest_approved
        && !receipt.retrieval_result_use_summary_manifest_ready_for_use
        && !receipt.retrieval_result_use_summary_manifest_admitted
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.runtime_result_approval_performed
        && !receipt.policy_promotion_performed
        && !receipt.batch_execution_performed
        && !receipt.student_training_performed
        && receipt.external_result_evidence_present
        && !receipt.retrieval_result_use_summary_manifest_approval_admission_consumed
        && receipt.result_use_summary_manifest_approval_admission_consumption_status
            == "result_use_summary_manifest_approval_admission_not_consumed"
        && receipt.not_consumed_reason == "summary_manifest_approval_admission_not_admitted"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_eligibility_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_eligibility_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_eligibility_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_eligibility_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_result_use_summary_manifest_approval_admission_consumed
        && !receipt.retrieval_result_use_summary_manifest_approval_admitted
        && !receipt.retrieval_result_use_summary_manifest_approved
        && !receipt.retrieval_result_use_summary_manifest_ready_for_use
        && !receipt.retrieval_result_use_summary_manifest_admitted
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.runtime_result_approval_performed
        && !receipt.policy_promotion_performed
        && !receipt.batch_execution_performed
        && !receipt.student_training_performed
        && receipt.external_result_evidence_present
        && !receipt.retrieval_example_learning_eligible
        && receipt.retrieval_example_learning_eligibility_status
            == "retrieval_example_learning_not_eligible"
        && receipt.not_eligible_reason == "approval_admission_consumption_not_consumed"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_admission_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_admission_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_admission_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_admission_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_example_learning_eligible
        && !receipt.retrieval_result_use_summary_manifest_approval_admission_consumed
        && !receipt.retrieval_result_use_summary_manifest_approval_admitted
        && !receipt.retrieval_result_use_summary_manifest_approved
        && !receipt.retrieval_result_use_summary_manifest_ready_for_use
        && !receipt.retrieval_result_use_summary_manifest_admitted
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.runtime_result_approval_performed
        && !receipt.policy_promotion_performed
        && !receipt.batch_execution_performed
        && !receipt.student_training_performed
        && receipt.external_result_evidence_present
        && !receipt.retrieval_example_learning_admitted
        && receipt.retrieval_example_learning_admission_status
            == "retrieval_example_learning_not_admitted"
        && receipt.not_admitted_reason == "retrieval_example_learning_not_eligible"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_materialization_plan_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_materialization_plan_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_materialization_plan_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_materialization_plan_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_example_learning_admitted
        && !receipt.retrieval_example_learning_eligible
        && !receipt.retrieval_result_use_summary_manifest_approval_admission_consumed
        && !receipt.retrieval_result_use_summary_manifest_approval_admitted
        && !receipt.retrieval_result_use_summary_manifest_approved
        && !receipt.retrieval_result_use_summary_manifest_ready_for_use
        && !receipt.retrieval_result_use_summary_manifest_admitted
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.runtime_result_approval_performed
        && !receipt.policy_promotion_performed
        && !receipt.batch_execution_performed
        && !receipt.student_training_performed
        && receipt.external_result_evidence_present
        && !receipt.retrieval_example_materialization_plan_ready
        && receipt.retrieval_example_materialization_plan_status
            == "retrieval_example_materialization_plan_not_ready"
        && receipt.not_ready_reason == "retrieval_example_learning_not_admitted"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_admission_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_admission_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_admission_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_admission_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_example_materialization_plan_ready
        && !receipt.retrieval_example_learning_admitted
        && !receipt.retrieval_example_learning_eligible
        && !receipt.retrieval_result_use_summary_manifest_approval_admission_consumed
        && !receipt.retrieval_result_use_summary_manifest_approval_admitted
        && !receipt.retrieval_result_use_summary_manifest_approved
        && !receipt.retrieval_result_use_summary_manifest_ready_for_use
        && !receipt.retrieval_result_use_summary_manifest_admitted
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.runtime_result_approval_performed
        && !receipt.policy_promotion_performed
        && !receipt.batch_execution_performed
        && !receipt.student_training_performed
        && receipt.external_result_evidence_present
        && !receipt.retrieval_example_storage_admitted
        && receipt.retrieval_example_storage_admission_status
            == "retrieval_example_storage_not_admitted"
        && receipt.not_admitted_reason == "retrieval_example_materialization_plan_not_ready"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_commit_intent_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_commit_intent_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_commit_intent_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_commit_intent_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_example_storage_admitted
        && !receipt.retrieval_example_materialization_plan_ready
        && !receipt.retrieval_example_learning_admitted
        && !receipt.retrieval_example_learning_eligible
        && !receipt.retrieval_result_use_summary_manifest_approval_admission_consumed
        && !receipt.retrieval_result_use_summary_manifest_approval_admitted
        && !receipt.retrieval_result_use_summary_manifest_approved
        && !receipt.retrieval_result_use_summary_manifest_ready_for_use
        && !receipt.retrieval_result_use_summary_manifest_admitted
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.runtime_result_approval_performed
        && !receipt.policy_promotion_performed
        && !receipt.batch_execution_performed
        && !receipt.student_training_performed
        && receipt.external_result_evidence_present
        && !receipt.retrieval_example_storage_commit_intent_ready
        && receipt.retrieval_example_storage_commit_intent_status
            == "retrieval_example_storage_commit_intent_not_ready"
        && receipt.not_ready_reason == "retrieval_example_storage_not_admitted"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_preflight_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_preflight_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_preflight_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_preflight_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_example_storage_commit_intent_ready
        && !receipt.retrieval_example_storage_admitted
        && !receipt.retrieval_example_materialization_plan_ready
        && !receipt.retrieval_example_learning_admitted
        && !receipt.retrieval_example_learning_eligible
        && !receipt.retrieval_result_use_summary_manifest_approval_admission_consumed
        && !receipt.retrieval_result_use_summary_manifest_approval_admitted
        && !receipt.retrieval_result_use_summary_manifest_approved
        && !receipt.retrieval_result_use_summary_manifest_ready_for_use
        && !receipt.retrieval_result_use_summary_manifest_admitted
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.runtime_result_approval_performed
        && !receipt.policy_promotion_performed
        && !receipt.batch_execution_performed
        && !receipt.student_training_performed
        && receipt.external_result_evidence_present
        && !receipt.retrieval_example_storage_write_preflight_ready
        && receipt.retrieval_example_storage_write_preflight_status
            == "retrieval_example_storage_write_preflight_not_ready"
        && receipt.not_ready_reason == "retrieval_example_storage_commit_intent_not_ready"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_approval_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_approval_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_approval_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_approval_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_example_storage_write_preflight_ready
        && !receipt.retrieval_example_storage_commit_intent_ready
        && !receipt.retrieval_example_storage_admitted
        && !receipt.retrieval_example_materialization_plan_ready
        && !receipt.retrieval_example_learning_admitted
        && !receipt.retrieval_example_learning_eligible
        && !receipt.retrieval_result_use_summary_manifest_approval_admission_consumed
        && !receipt.retrieval_result_use_summary_manifest_approval_admitted
        && !receipt.retrieval_result_use_summary_manifest_approved
        && !receipt.retrieval_result_use_summary_manifest_ready_for_use
        && !receipt.retrieval_result_use_summary_manifest_admitted
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.runtime_result_approval_performed
        && !receipt.policy_promotion_performed
        && !receipt.batch_execution_performed
        && !receipt.student_training_performed
        && receipt.external_result_evidence_present
        && !receipt.retrieval_example_storage_write_approved
        && receipt.retrieval_example_storage_write_approval_status
            == "retrieval_example_storage_write_not_approved"
        && receipt.not_approved_reason == "retrieval_example_storage_write_preflight_not_ready"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_admission_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_admission_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_admission_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_admission_regression_smoke_receipt();
    let blocked_steps = [
        receipt.retrieval_example_storage_write_approved,
        receipt.retrieval_example_storage_write_preflight_ready,
        receipt.retrieval_example_storage_commit_intent_ready,
        receipt.retrieval_example_storage_admitted,
        receipt.retrieval_example_materialization_plan_ready,
        receipt.retrieval_example_learning_admitted,
        receipt.retrieval_example_learning_eligible,
        receipt.retrieval_result_use_summary_manifest_approval_admission_consumed,
        receipt.retrieval_result_use_summary_manifest_approval_admitted,
        receipt.retrieval_result_use_summary_manifest_approved,
        receipt.retrieval_result_use_summary_manifest_ready_for_use,
        receipt.retrieval_result_use_summary_manifest_admitted,
        receipt.retrieval_read_performed,
        receipt.retrieval_write_performed,
        receipt.retrieval_query_executed,
        receipt.runtime_result_approval_performed,
        receipt.policy_promotion_performed,
        receipt.batch_execution_performed,
        receipt.student_training_performed,
        receipt.retrieval_example_storage_write_admitted,
        receipt.passed(),
    ];
    let passed = receipt.is_valid()
        && all_false(&blocked_steps)
        && receipt.external_result_evidence_present
        && receipt.retrieval_example_storage_write_admission_status
            == "retrieval_example_storage_write_not_admitted"
        && receipt.not_admitted_reason == "retrieval_example_storage_write_not_approved";
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_commit_intent_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_commit_intent_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_commit_intent_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_commit_intent_regression_smoke_receipt();
    let blocked_steps = [
        receipt.retrieval_example_storage_write_admitted,
        receipt.retrieval_example_storage_write_approved,
        receipt.retrieval_example_storage_write_preflight_ready,
        receipt.retrieval_example_storage_commit_intent_ready,
        receipt.retrieval_example_storage_admitted,
        receipt.retrieval_example_materialization_plan_ready,
        receipt.retrieval_example_learning_admitted,
        receipt.retrieval_example_learning_eligible,
        receipt.retrieval_result_use_summary_manifest_approval_admission_consumed,
        receipt.retrieval_result_use_summary_manifest_approval_admitted,
        receipt.retrieval_result_use_summary_manifest_approved,
        receipt.retrieval_result_use_summary_manifest_ready_for_use,
        receipt.retrieval_result_use_summary_manifest_admitted,
        receipt.retrieval_read_performed,
        receipt.retrieval_write_performed,
        receipt.retrieval_query_executed,
        receipt.runtime_result_approval_performed,
        receipt.policy_promotion_performed,
        receipt.batch_execution_performed,
        receipt.student_training_performed,
        receipt.retrieval_example_storage_write_commit_intent_ready,
        receipt.passed(),
    ];
    let passed = receipt.is_valid()
        && all_false(&blocked_steps)
        && storage_write_commit_intent_regression_gate_passed(&receipt);
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_validate_dispatch_catalog_lists_every_compact_mode_once() {
        let catalog_payload = root_validate_dispatch_catalog_payload();
        let mut catalog_entries = std::collections::BTreeSet::new();

        for line in catalog_payload.lines() {
            assert!(
                catalog_entries.insert(line.to_string()),
                "duplicate dispatch catalog entry: {line}"
            );
        }

        assert_eq!(catalog_entries.len(), COMPACT_MODES.len());
        for mode in COMPACT_MODES {
            assert!(
                catalog_entries.contains(&format!("{}=>{}", mode.arg, mode.marker)),
                "missing dispatch catalog entry for {}",
                mode.arg
            );
        }
        assert!(catalog_entries
            .contains("--root-validate-dispatch-catalog=>canon_root_validate_dispatch_catalog_v1"));

        let outcome = try_run_compact_mode("--root-validate-dispatch-catalog")
            .expect("dispatch catalog compact mode is registered")
            .expect("dispatch catalog compact mode succeeds");

        assert_eq!(outcome.exit_code, 0);
        assert!(outcome
            .stdout
            .contains("\"schema\":\"canon_root_validate_dispatch_catalog_v1\""));
        assert!(outcome
            .stdout
            .contains("\"record_type\":\"root_validate_dispatch_catalog\""));
        assert!(outcome
            .stdout
            .contains(&format!("\"compact_mode_count\":{}", COMPACT_MODES.len())));
        assert!(outcome.stdout.contains("\"dispatch_catalog_hash\":"));
    }

    #[test]
    fn policy_reuse_common_regression_guards_preserve_initial_result_modes() {
        let cases = [
            (
                "--policy-reuse-evidence-retrieval-result-manifest-regression-smoke",
                "result_manifest_not_ready",
            ),
            (
                "--policy-reuse-evidence-retrieval-result-use-admission-regression-smoke",
                "result_use_not_admitted",
            ),
            (
                "--policy-reuse-evidence-retrieval-result-use-manifest-regression-smoke",
                "result_use_manifest_not_ready",
            ),
            (
                "--policy-reuse-evidence-retrieval-result-use-readiness-regression-smoke",
                "result_use_not_ready",
            ),
        ];

        for (arg, expected_status) in cases {
            let outcome = try_run_compact_mode(arg)
                .unwrap_or_else(|| panic!("compact mode is registered: {arg}"))
                .unwrap_or_else(|err| panic!("compact mode succeeds for {arg}: {err}"));

            assert_eq!(outcome.exit_code, 0, "unexpected exit code for {arg}");
            assert!(
                outcome.stdout.contains(expected_status),
                "missing expected status {expected_status} in {arg} output: {}",
                outcome.stdout
            );
        }
    }

    #[test]
    fn policy_reuse_common_regression_guards_preserve_follow_on_result_modes() {
        let cases = [
            (
                "--policy-reuse-evidence-retrieval-result-use-approval-regression-smoke",
                "result_use_not_approved",
            ),
            (
                "--policy-reuse-evidence-retrieval-result-use-manifest-admission-regression-smoke",
                "result_use_manifest_not_admitted",
            ),
            (
                "--policy-reuse-evidence-retrieval-result-use-summary-regression-smoke",
                "result_use_summary_not_ready",
            ),
            (
                "--policy-reuse-evidence-retrieval-result-use-summary-manifest-regression-smoke",
                "result_use_summary_manifest_not_ready",
            ),
        ];

        for (arg, expected_status) in cases {
            let outcome = try_run_compact_mode(arg)
                .unwrap_or_else(|| panic!("compact mode is registered: {arg}"))
                .unwrap_or_else(|err| panic!("compact mode succeeds for {arg}: {err}"));

            assert_eq!(outcome.exit_code, 0, "unexpected exit code for {arg}");
            assert!(
                outcome.stdout.contains(expected_status),
                "missing expected status {expected_status} in {arg} output: {}",
                outcome.stdout
            );
        }
    }
}

fn try_run_compact_mode(arg: &str) -> Option<Result<CompactModeOutcome, String>> {
    COMPACT_MODES
        .iter()
        .find(|mode| mode.arg == arg)
        .map(|mode| (mode.run)())
}

pub fn compact_mode_stdout_for_contract(
    arg: &str,
    expected_marker: &str,
) -> Result<String, String> {
    let outcome = try_run_compact_mode(arg)
        .ok_or_else(|| format!("root_validate compact mode not found: {arg}"))??;
    if outcome.exit_code != 0 {
        return Err(format!("root_validate {arg} failed"));
    }
    let mut stdout = outcome.stdout;
    if !stdout.ends_with('\n') {
        stdout.push('\n');
    }
    if !stdout.contains(expected_marker) {
        return Err(format!("root_validate {arg} missing {expected_marker}"));
    }
    if stdout.contains("canon_root_validation_v1") {
        return Err(format!(
            "root_validate {arg} emitted root validation output"
        ));
    }
    Ok(stdout)
}

#[allow(dead_code)]
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
                    mirror_validation_outcome(arg, &outcome);
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
            let stdout = receipt.to_json_line();
            println!("{stdout}");
            mirror_validation_json("validate_root", &stdout, receipt.passed());
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

fn mirror_validation_outcome(arg: &str, outcome: &CompactModeOutcome) {
    mirror_validation_json(
        arg.trim_start_matches('-'),
        &outcome.stdout,
        outcome.exit_code == 0,
    );
}

fn mirror_validation_json(record_type: &str, stdout: &str, passed: bool) {
    let Ok(path) = std::env::var("AI_CANONICAL_TLOG") else {
        return;
    };
    let receipt_hash = stable_text_hash(stdout);
    let _ = ai::append_validation_result_ndjson(path, record_type, passed, receipt_hash);
}

fn stable_text_hash(text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish().max(1)
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

fn policy_reuse_evidence_batch_execution_plan_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_batch_execution_plan_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_batch_execution_plan_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_batch_execution_plan_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.batch_ready
        && !receipt.compact_validation_passed
        && receipt.no_execute_plan
        && !receipt.execution_performed
        && !receipt.plan_ready
        && receipt.plan_status == "not_plannable"
        && receipt.not_plannable_reason == "batch_not_ready"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_batch_evaluation_admission_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_batch_evaluation_admission_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_batch_evaluation_admission_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_batch_evaluation_admission_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.plan_ready
        && !receipt.batch_ready
        && receipt.no_execute_plan
        && !receipt.execution_performed
        && !receipt.batch_evaluation_admitted
        && receipt.admission_status == "not_admitted"
        && receipt.not_admitted_reason == "plan_not_ready"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_batch_run_request_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_batch_run_request_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_batch_run_request_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_batch_run_request_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.batch_evaluation_admitted
        && !receipt.plan_ready
        && receipt.no_execute_request
        && !receipt.execution_performed
        && !receipt.batch_request_ready
        && receipt.request_status == "not_requestable"
        && receipt.not_requestable_reason == "admission_not_granted"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_external_evaluator_result_smoke_mode() -> Result<CompactModeOutcome, String>
{
    let receipt =
        validation_harness::policy_reuse_evidence_external_evaluator_result_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_external_evaluator_result_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_external_evaluator_result_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.batch_request_ready
        && !receipt.batch_evaluation_admitted
        && receipt.external_evaluator_independent
        && !receipt.llm_self_approved
        && !receipt.evaluator_result_passed
        && receipt.evaluator_status == "failed"
        && receipt.evaluator_failure_reason == "external_evaluator_failed"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_learning_candidate_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_learning_candidate_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_learning_candidate_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_learning_candidate_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.evaluator_result_passed
        && !receipt.batch_request_ready
        && !receipt.policy_promotion_performed
        && !receipt.retrieval_write_performed
        && !receipt.learning_candidate_ready
        && receipt.candidate_status == "not_candidate"
        && receipt.not_candidate_reason == "evaluator_not_passed"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_learning_data_admission_smoke_mode() -> Result<CompactModeOutcome, String>
{
    let receipt = validation_harness::policy_reuse_evidence_learning_data_admission_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_learning_data_admission_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_learning_data_admission_regression_smoke_receipt(
        );
    let passed = receipt.is_valid()
        && !receipt.learning_candidate_ready
        && !receipt.evaluator_result_passed
        && !receipt.policy_promotion_performed
        && !receipt.retrieval_write_performed
        && !receipt.student_training_performed
        && !receipt.learning_data_admitted
        && receipt.admission_status == "not_admitted"
        && receipt.not_admitted_reason == "candidate_not_ready"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_example_admission_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_example_admission_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_example_admission_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_example_admission_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.learning_data_admitted
        && !receipt.learning_candidate_ready
        && !receipt.retrieval_write_performed
        && !receipt.policy_promotion_performed
        && !receipt.student_training_performed
        && !receipt.retrieval_example_admitted
        && receipt.admission_status == "not_admitted"
        && receipt.not_admitted_reason == "data_not_admitted"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_example_index_smoke_mode() -> Result<CompactModeOutcome, String>
{
    let receipt = validation_harness::policy_reuse_evidence_retrieval_example_index_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_example_index_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_example_index_regression_smoke_receipt(
        );
    let passed = receipt.is_valid()
        && !receipt.retrieval_example_admitted
        && !receipt.learning_data_admitted
        && !receipt.retrieval_write_performed
        && !receipt.policy_promotion_performed
        && !receipt.student_training_performed
        && !receipt.retrieval_example_indexed
        && receipt.index_status == "not_indexed"
        && receipt.not_indexed_reason == "example_not_admitted"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_corpus_readiness_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_corpus_readiness_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_corpus_readiness_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_corpus_readiness_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_example_indexed
        && !receipt.retrieval_example_admitted
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.policy_promotion_performed
        && !receipt.student_training_performed
        && !receipt.retrieval_corpus_ready
        && receipt.readiness_status == "not_ready"
        && receipt.not_ready_reason == "index_not_ready"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_corpus_admission_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_corpus_admission_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_corpus_admission_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::
        policy_reuse_evidence_retrieval_corpus_admission_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_corpus_ready
        && !receipt.retrieval_example_indexed
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.policy_promotion_performed
        && !receipt.student_training_performed
        && !receipt.retrieval_corpus_admitted
        && receipt.admission_status == "not_admitted"
        && receipt.not_admitted_reason == "corpus_not_ready"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_use_approval_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_retrieval_use_approval_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_use_approval_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_use_approval_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_corpus_admitted
        && !receipt.retrieval_corpus_ready
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.policy_promotion_performed
        && !receipt.student_training_performed
        && !receipt.retrieval_use_approved
        && receipt.approval_status == "not_approved"
        && receipt.not_approved_reason == "corpus_not_admitted"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_use_manifest_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_retrieval_use_manifest_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_use_manifest_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_use_manifest_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_use_approved
        && !receipt.retrieval_corpus_admitted
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.policy_promotion_performed
        && !receipt.student_training_performed
        && !receipt.retrieval_use_manifest_ready
        && receipt.manifest_status == "manifest_not_ready"
        && receipt.not_ready_reason == "use_not_approved"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}

fn policy_reuse_evidence_retrieval_query_plan_smoke_mode() -> Result<CompactModeOutcome, String> {
    let receipt = validation_harness::policy_reuse_evidence_retrieval_query_plan_smoke_receipt();
    Ok(CompactModeOutcome::pass_json(
        receipt.to_json(),
        receipt.passed(),
    ))
}

fn policy_reuse_evidence_retrieval_query_plan_regression_smoke_mode(
) -> Result<CompactModeOutcome, String> {
    let receipt =
        validation_harness::policy_reuse_evidence_retrieval_query_plan_regression_smoke_receipt();
    let passed = receipt.is_valid()
        && !receipt.retrieval_use_manifest_ready
        && !receipt.retrieval_use_approved
        && !receipt.retrieval_read_performed
        && !receipt.retrieval_write_performed
        && !receipt.retrieval_query_executed
        && !receipt.policy_promotion_performed
        && !receipt.student_training_performed
        && !receipt.retrieval_query_plan_ready
        && receipt.query_plan_status == "query_plan_not_ready"
        && receipt.not_ready_reason == "manifest_not_ready"
        && !receipt.passed();
    Ok(CompactModeOutcome::controlled_json(
        receipt.to_json(),
        passed,
    ))
}
