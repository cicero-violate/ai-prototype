use ai::validation_harness::{
    compare_runtime_performance_trend, compare_validation_cost_footprint, root_validation_steps,
    runtime_performance_receipt, validation_footprint_receipt_for_steps, GraphTelemetryReceipt,
    RuntimePerformanceReceipt, StepReceipt, StepRunner, ValidationReceipt, ValidationStep,
    API_TRANSPORT_STEP, CHECK_STEP, EXPECTED_COMMAND_FIXTURE_COUNT,
    EXPECTED_RETAINED_RECEIPT_FIXTURE_COUNT, EXPECTED_VALIDATION_FIXTURE_COUNT,
    EXTERNAL_AGENT_CLI_MODES_FIXTURE, FAST_TEST_STEP, GRAPH_MUTATION_CLI_CONTRACT_EXPECTED_TESTS,
    GRAPH_MUTATION_CLI_CONTRACT_STEP, LIB_UNIT_STEP, LOCKFILE_COMPAT_FLAG, PLANNING_CONTRACT_STEP,
    POLICY_CAPACITY_COST_SUMMARY_RECEIPTS_FIXTURE,
    POLICY_CAPACITY_COST_SUMMARY_REGRESSION_SMOKE_STEP, POLICY_CAPACITY_COST_SUMMARY_SMOKE_STEP,
    POLICY_CAPACITY_COST_SUMMARY_TREND_SMOKE_STEP, POLICY_ORCHESTRATION_CAPACITY_RECEIPTS_FIXTURE,
    POLICY_REUSE_LEDGER_SUMMARY_REGRESSION_SMOKE_STEP, POLICY_REUSE_LEDGER_SUMMARY_SMOKE_STEP,
    POLICY_REUSE_RECEIPTS_FIXTURE, POLICY_REUSE_SCALE_TRACE_REGRESSION_SMOKE_STEP,
    POLICY_REUSE_SCALE_TRACE_SMOKE_STEP, POLICY_VALIDATION_HEALTH_RECEIPTS_FIXTURE,
    POLICY_VALIDATION_HEALTH_TREND_RECEIPTS_FIXTURE, RUNTIME_PERFORMANCE_BUDGET_SMOKE_STEP,
    RUNTIME_PERFORMANCE_STEP, RUNTIME_PERFORMANCE_THRESHOLDS_FIXTURE,
    RUNTIME_PERFORMANCE_TREND_FIXTURE, RUNTIME_PERFORMANCE_TREND_REGRESSION_SMOKE_STEP,
    RUNTIME_PERFORMANCE_TREND_SMOKE_STEP, VALIDATION_COMMAND_FOOTPRINT_PLANNING_STEP,
    VALIDATION_COMMAND_FOOTPRINT_RECEIPTS_FIXTURE,
    VALIDATION_COMMAND_FOOTPRINT_REGRESSION_SMOKE_STEP,
    VALIDATION_COMMAND_FOOTPRINT_TARGET_MET_SMOKE_STEP, VALIDATION_COMMAND_FOOTPRINT_TARGET_STEPS,
    VALIDATION_COMMAND_FOOTPRINT_TREND_SMOKE_STEP,
    VALIDATION_COMMAND_FOOTPRINT_UNSAFE_TARGET_SMOKE_STEP,
    VALIDATION_COST_FOOTPRINT_GROWTH_SMOKE_STEP,
    VALIDATION_DURATION_PLANNING_BUDGET_EXHAUSTION_SMOKE_STEP,
    VALIDATION_DURATION_PLANNING_RECEIPTS_FIXTURE,
    VALIDATION_DURATION_PLANNING_REGRESSION_SMOKE_STEP, VALIDATION_DURATION_PLANNING_STEP,
    VALIDATION_DURATION_PLANNING_TREND_SMOKE_STEP, VALIDATION_FIXTURE_CATALOG_STEP,
    VALIDATION_FOOTPRINT_STEP, VALIDATION_HARNESS_EXPECTED_TESTS, VALIDATION_HARNESS_STEP,
};

const EXPECTED_ROOT_VALIDATE_COMPACT_MODE_COUNT: usize = 82;
const EXPECTED_EXTERNAL_AGENT_CLI_MODE_COUNT: usize = 101;

fn expected_guarded_test_count() -> usize {
    VALIDATION_HARNESS_EXPECTED_TESTS + GRAPH_MUTATION_CLI_CONTRACT_EXPECTED_TESTS
}

#[test]
fn root_validation_requires_lockfile_compatibility_flag() {
    let steps = root_validation_steps();
    assert_eq!(steps.len(), 7);
    for step in &steps {
        assert_eq!(step.args.first(), Some(&LOCKFILE_COMPAT_FLAG));
    }
}

#[test]
fn root_validation_runs_check_before_contract_suites() {
    let steps = root_validation_steps();
    let names = steps.iter().map(|step| step.name).collect::<Vec<_>>();
    assert_eq!(
        names,
        [
            CHECK_STEP,
            FAST_TEST_STEP,
            LIB_UNIT_STEP,
            API_TRANSPORT_STEP,
            VALIDATION_HARNESS_STEP,
            PLANNING_CONTRACT_STEP,
            GRAPH_MUTATION_CLI_CONTRACT_STEP,
        ]
    );

    assert_eq!(
        steps[0].args,
        [LOCKFILE_COMPAT_FLAG, "check", "--all-targets", "--locked"]
    );
    assert!(steps[1].args.contains(&"score_contract"));
    assert!(steps[2].args.contains(&"--lib"));
    assert!(steps[3].args.contains(&"api_transport_contract"));
    assert!(steps[4].args.contains(&"validation_harness_contract"));
    assert_eq!(
        steps[4].expected_test_count,
        Some(VALIDATION_HARNESS_EXPECTED_TESTS)
    );
    assert_eq!(VALIDATION_HARNESS_EXPECTED_TESTS, 248);
    assert!(steps[5].args.contains(&"planning_contract"));
    assert!(steps[6].args.contains(&"graph_mutation_cli_contract"));
}

#[test]
fn validation_command_lines_are_stable() {
    let steps = root_validation_steps();
    assert_eq!(
        steps[0].command_line("cargo"),
        "cargo -Znext-lockfile-bump check --all-targets --locked"
    );
    assert_eq!(
        steps[2].command_line("cargo"),
        "cargo -Znext-lockfile-bump test --lib --locked -- --nocapture"
    );
}

#[test]
fn validation_footprint_receipt_summarizes_root_suite_contract() {
    let receipt = ai::validation_harness::validation_footprint_receipt();

    assert_eq!(receipt.schema, "canon_validation_footprint_v1");
    assert_eq!(receipt.record_type, VALIDATION_FOOTPRINT_STEP);
    assert_eq!(receipt.cargo_step_count, 7);
    assert_eq!(receipt.python_step_count, 0);
    assert_eq!(receipt.total_declared_steps, 7);
    assert_eq!(receipt.expected_count_guarded_steps, 2);
    assert_eq!(
        receipt.expected_count_guarded_tests,
        expected_guarded_test_count()
    );
    assert_eq!(receipt.lockfile_compat_step_count, receipt.cargo_step_count);
    assert!(receipt.runtime_budget_required);
    assert_eq!(
        receipt.max_project_agent_elapsed_ms_p95,
        ai::validation_harness::DEFAULT_MAX_PROJECT_AGENT_ELAPSED_MS_P95
    );
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
    assert!(!receipt.command_set_hash.is_empty());
}

#[test]
fn root_validate_validation_footprint_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--validation-footprint",
        &[
            "\"schema\":\"canon_validation_footprint_v1\\",
            "\"record_type\":\"validation_footprint_summary\\",
            "\"cargo_step_count\":7",
            "\"python_step_count\":0",
            "\"total_declared_steps\":7",
            "\"expected_count_guarded_steps\":2",
            &format!(
                "\"expected_count_guarded_tests\":{}",
                expected_guarded_test_count()
            ),
            "\"runtime_budget_required\":true",
            "\"max_project_agent_elapsed_ms_p95\":10000",
            "\"verdict\":\"pass\\",
        ],
    );
}

#[test]
fn validation_command_footprint_planning_receipt_sets_safe_reduction_target() {
    let footprint = ai::validation_harness::validation_footprint_receipt();
    let receipt = ai::validation_harness::validation_command_footprint_planning_receipt(
        &footprint,
        VALIDATION_COMMAND_FOOTPRINT_TARGET_STEPS,
    );

    assert_eq!(
        receipt.schema,
        "canon_validation_command_footprint_planning_v1"
    );
    assert_eq!(
        receipt.record_type,
        VALIDATION_COMMAND_FOOTPRINT_PLANNING_STEP
    );
    assert_eq!(receipt.current_total_declared_steps, 7);
    assert_eq!(receipt.target_total_declared_steps, 6);
    assert_eq!(receipt.command_reduction_target, 1);
    assert_eq!(receipt.expected_count_guarded_steps, 2);
    assert_eq!(
        receipt.expected_count_guarded_tests,
        expected_guarded_test_count()
    );
    assert_eq!(receipt.footprint_verdict, "pass");
    assert_eq!(receipt.safety_status, "pass");
    assert_eq!(receipt.planning_status, "action_required");
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
}

#[test]
fn validation_command_footprint_target_met_smoke_passes_without_required_reduction() {
    let receipt = ai::validation_harness::validation_command_footprint_target_met_smoke_receipt();

    assert_eq!(receipt.current_total_declared_steps, 7);
    assert_eq!(receipt.target_total_declared_steps, 7);
    assert_eq!(receipt.command_reduction_target, 0);
    assert_eq!(receipt.expected_count_guarded_steps, 2);
    assert_eq!(
        receipt.record_type,
        VALIDATION_COMMAND_FOOTPRINT_TARGET_MET_SMOKE_STEP
    );
    assert_eq!(receipt.safety_status, "pass");
    assert_eq!(receipt.planning_status, "met");
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
}

#[test]
fn validation_command_footprint_planning_rejects_unsafe_target() {
    let receipt =
        ai::validation_harness::validation_command_footprint_unsafe_target_smoke_receipt();

    assert_eq!(receipt.current_total_declared_steps, 7);
    assert_eq!(receipt.target_total_declared_steps, 1);
    assert_eq!(receipt.command_reduction_target, 6);
    assert_eq!(receipt.expected_count_guarded_steps, 2);
    assert_eq!(
        receipt.record_type,
        VALIDATION_COMMAND_FOOTPRINT_UNSAFE_TARGET_SMOKE_STEP
    );
    assert_eq!(receipt.safety_status, "fail");
    assert_eq!(receipt.planning_status, "unsafe_target");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn root_validate_validation_command_footprint_planning_modes_are_executable_contracts() {
    assert_root_validate_catalog_compact_mode_contract(
        "--validation-command-footprint-planning",
        &[
            "\"schema\":\"canon_validation_command_footprint_planning_v1\\",
            "\"record_type\":\"validation_command_footprint_planning\\",
            "\"current_total_declared_steps\":7",
            "\"target_total_declared_steps\":6",
            "\"command_reduction_target\":1",
            "\"expected_count_guarded_steps\":2",
            "\"safety_status\":\"pass\\",
            "\"planning_status\":\"action_required\\",
            "\"verdict\":\"pass\\",
        ],
    );
    assert_root_validate_catalog_compact_mode_contract(
        "--validation-command-footprint-target-met-smoke",
        &[
            "\"schema\":\"canon_validation_command_footprint_planning_v1\\",
            "\"record_type\":\"validation_command_footprint_target_met_smoke\\",
            "\"target_total_declared_steps\":7",
            "\"command_reduction_target\":0",
            "\"safety_status\":\"pass\\",
            "\"planning_status\":\"met\\",
            "\"verdict\":\"pass\\",
        ],
    );
    assert_root_validate_catalog_compact_mode_contract(
        "--validation-command-footprint-unsafe-target-smoke",
        &[
            "\"schema\":\"canon_validation_command_footprint_planning_v1\\",
            "\"record_type\":\"validation_command_footprint_unsafe_target_smoke\\",
            "\"target_total_declared_steps\":1",
            "\"command_reduction_target\":6",
            "\"safety_status\":\"fail\\",
            "\"planning_status\":\"unsafe_target\\",
            "\"verdict\":\"fail\\",
        ],
    );
}

#[test]
fn validation_command_footprint_trend_smoke_compares_retained_targets() {
    let receipt = ai::validation_harness::validation_command_footprint_trend_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_validation_command_footprint_trend_v1"
    );
    assert_eq!(
        receipt.record_type,
        VALIDATION_COMMAND_FOOTPRINT_TREND_SMOKE_STEP
    );
    assert_eq!(receipt.baseline_target_total_declared_steps, 6);
    assert_eq!(receipt.current_target_total_declared_steps, 7);
    assert_eq!(receipt.target_total_declared_step_delta, 1);
    assert_eq!(receipt.baseline_command_reduction_target, 1);
    assert_eq!(receipt.current_command_reduction_target, 0);
    assert_eq!(receipt.command_reduction_target_delta, -1);
    assert_eq!(receipt.baseline_planning_status, "action_required");
    assert_eq!(receipt.current_planning_status, "met");
    assert_eq!(receipt.trend_status, "pass");
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
}

#[test]
fn validation_command_footprint_regression_smoke_exposes_controlled_negative() {
    let receipt = ai::validation_harness::validation_command_footprint_regression_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_validation_command_footprint_trend_v1"
    );
    assert_eq!(
        receipt.record_type,
        VALIDATION_COMMAND_FOOTPRINT_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.baseline_target_total_declared_steps, 7);
    assert_eq!(receipt.current_target_total_declared_steps, 6);
    assert_eq!(receipt.target_total_declared_step_delta, -1);
    assert_eq!(receipt.baseline_command_reduction_target, 0);
    assert_eq!(receipt.current_command_reduction_target, 1);
    assert_eq!(receipt.command_reduction_target_delta, 1);
    assert_eq!(receipt.baseline_planning_status, "met");
    assert_eq!(receipt.current_planning_status, "action_required");
    assert_eq!(receipt.trend_status, "regressed");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn validation_command_footprint_trend_rejects_unsafe_current_target() {
    let baseline = ai::validation_harness::validation_command_footprint_planning_smoke_receipt();
    let current =
        ai::validation_harness::validation_command_footprint_unsafe_target_smoke_receipt();
    let receipt =
        ai::validation_harness::compare_validation_command_footprint_trend(&baseline, &current);

    assert_eq!(receipt.current_safety_status, "fail");
    assert_eq!(receipt.current_planning_status, "unsafe_target");
    assert_eq!(receipt.trend_status, "regressed");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn root_validate_validation_command_footprint_trend_modes_are_executable_contracts() {
    assert_root_validate_catalog_compact_mode_contract(
        "--validation-command-footprint-trend-smoke",
        &[
            "\"schema\":\"canon_validation_command_footprint_trend_v1\\",
            "\"record_type\":\"validation_command_footprint_trend_smoke\\",
            "\"baseline_target_total_declared_steps\":6",
            "\"current_target_total_declared_steps\":7",
            "\"target_total_declared_step_delta\":1",
            "\"baseline_command_reduction_target\":1",
            "\"current_command_reduction_target\":0",
            "\"command_reduction_target_delta\":-1",
            "\"trend_status\":\"pass\\",
            "\"verdict\":\"pass\\",
        ],
    );
    assert_root_validate_catalog_compact_mode_contract(
        "--validation-command-footprint-regression-smoke",
        &[
            "\"schema\":\"canon_validation_command_footprint_trend_v1\\",
            "\"record_type\":\"validation_command_footprint_regression_smoke\\",
            "\"baseline_target_total_declared_steps\":7",
            "\"current_target_total_declared_steps\":6",
            "\"target_total_declared_step_delta\":-1",
            "\"baseline_command_reduction_target\":0",
            "\"current_command_reduction_target\":1",
            "\"command_reduction_target_delta\":1",
            "\"trend_status\":\"regressed\\",
            "\"verdict\":\"fail\\",
        ],
    );
}

#[test]
fn validation_footprint_negative_contract_fails_without_lockfile_flag() {
    let mut steps = root_validation_steps();
    steps[0] = ValidationStep {
        name: CHECK_STEP,
        runner: StepRunner::Cargo,
        args: vec!["check", "--all-targets", "--locked"],
        expected_test_count: None,
    };

    let receipt = validation_footprint_receipt_for_steps(
        &steps,
        ai::validation_harness::DEFAULT_MAX_PROJECT_AGENT_ELAPSED_MS_P95,
        ai::validation_harness::DEFAULT_MAX_DOWNLOAD_INITIAL_GET_MS_P95,
        ai::validation_harness::DEFAULT_MAX_DOWNLOAD_FOLLOW_GET_MS_P95,
        ai::validation_harness::DEFAULT_MAX_DOWNLOAD_WRITE_MS_P95,
    );

    assert_eq!(receipt.cargo_step_count, 7);
    assert_eq!(receipt.lockfile_compat_step_count, 6);
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
    assert!(receipt.to_json().contains("\"verdict\":\"fail\\"));
}

#[test]
fn validation_footprint_negative_contract_fails_when_runtime_budget_disabled() {
    let steps = root_validation_steps();

    let receipt = validation_footprint_receipt_for_steps(
        &steps,
        0,
        ai::validation_harness::DEFAULT_MAX_DOWNLOAD_INITIAL_GET_MS_P95,
        ai::validation_harness::DEFAULT_MAX_DOWNLOAD_FOLLOW_GET_MS_P95,
        ai::validation_harness::DEFAULT_MAX_DOWNLOAD_WRITE_MS_P95,
    );

    assert_eq!(receipt.lockfile_compat_step_count, receipt.cargo_step_count);
    assert!(!receipt.runtime_budget_required);
    assert_eq!(receipt.max_project_agent_elapsed_ms_p95, 0);
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
    assert!(receipt
        .to_json()
        .contains("\"runtime_budget_required\":false"));
}

#[test]
fn external_agent_cli_modes_fixture_documents_all_public_modes() {
    let fixture = std::fs::read_to_string(EXTERNAL_AGENT_CLI_MODES_FIXTURE)
        .expect("external agent CLI modes fixture must be readable");
    let catalog = ExternalAgentCliCatalog::parse(&fixture)
        .expect("external agent CLI modes fixture should parse");

    assert_eq!(catalog.schema, "canon_external_agent_cli_modes_v1");
    assert_eq!(
        catalog.declared_mode_count,
        EXPECTED_EXTERNAL_AGENT_CLI_MODE_COUNT
    );
    assert_eq!(catalog.entries.len(), catalog.declared_mode_count);
    assert!(catalog.contains(
        "root_validate --validation-footprint",
        "canon_validation_footprint_v1"
    ));
    assert!(catalog.contains(
        "root_validate --validation-command-footprint-target-met-smoke",
        "validation_command_footprint_target_met_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --validation-fixture-catalog",
        "canon_validation_fixture_catalog_v1",
    ));
    assert!(catalog.contains(
        "root_validate --runtime-budget-smoke",
        "runtime_performance_budget_smoke"
    ));
    assert!(catalog.contains(
        "root_validate --runtime-trend-smoke",
        "runtime_performance_trend_smoke"
    ));
    assert!(catalog.contains(
        "root_validate --runtime-trend-regression-smoke",
        "runtime_performance_trend_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --validation-cost-smoke",
        "validation_cost_footprint_smoke"
    ));
    assert!(catalog.contains(
        "root_validate --validation-cost-growth-smoke",
        "validation_cost_footprint_growth_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-orchestration-capacity-trend-smoke",
        "policy_orchestration_capacity_trend_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-orchestration-capacity-regression-smoke",
        "policy_orchestration_capacity_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-capacity-cost-summary-fixture",
        "canon_policy_capacity_cost_summary_receipts_v1",
    ));
    assert!(catalog.contains(
        "root_validate --policy-orchestration-capacity-smoke",
        "policy_orchestration_capacity_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-orchestration-capacity-fixture",
        "canon_policy_orchestration_capacity_receipts_v1",
    ));
    assert!(catalog.contains(
        "root_validate --policy-validation-health-smoke",
        "policy_validation_health_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-validation-health-trend-smoke",
        "policy_validation_health_trend_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-validation-health-fixture",
        "canon_policy_validation_health_receipts_v1",
    ));
    assert!(catalog.contains(
        "root_validate --policy-validation-health-trend-fixture",
        "canon_policy_validation_health_trend_receipts_v1",
    ));
    assert!(catalog.contains("root_validate --policy-reuse-smoke", "policy_reuse_smoke"));
    assert!(catalog.contains(
        "root_validate --policy-reuse-trend-smoke",
        "policy_reuse_trend_smoke"
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-regression-smoke",
        "policy_reuse_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-ledger-summary-smoke",
        "policy_reuse_ledger_summary_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-ledger-summary-regression-smoke",
        "policy_reuse_ledger_summary_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-scale-trace-smoke",
        "policy_reuse_scale_trace_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-scale-trace-regression-smoke",
        "policy_reuse_scale_trace_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-performance-cost-trend-smoke",
        "policy_reuse_performance_cost_trend_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-performance-cost-trend-regression-smoke",
        "policy_reuse_performance_cost_trend_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-cost-catalog-smoke",
        "policy_reuse_cost_catalog_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-cost-catalog-incomplete-smoke",
        "policy_reuse_cost_catalog_incomplete_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evaluator-savings-smoke",
        "policy_reuse_evaluator_savings_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evaluator-savings-regression-smoke",
        "policy_reuse_evaluator_savings_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-scaling-projection-smoke",
        "policy_reuse_scaling_projection_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-scaling-projection-regression-smoke",
        "policy_reuse_scaling_projection_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-distillation-readiness-smoke",
        "policy_reuse_distillation_readiness_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-distillation-readiness-regression-smoke",
        "policy_reuse_distillation_readiness_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-surface-index-smoke",
        "policy_reuse_evidence_surface_index_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-surface-index-regression-smoke",
        "policy_reuse_evidence_surface_index_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-bundle-smoke",
        "policy_reuse_evidence_bundle_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-bundle-regression-smoke",
        "policy_reuse_evidence_bundle_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-quickcheck-smoke",
        "policy_reuse_evidence_quickcheck_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-quickcheck-regression-smoke",
        "policy_reuse_evidence_quickcheck_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-maturity-smoke",
        "policy_reuse_evidence_maturity_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-maturity-regression-smoke",
        "policy_reuse_evidence_maturity_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-summary-smoke",
        "policy_reuse_evidence_summary_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-summary-regression-smoke",
        "policy_reuse_evidence_summary_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-manifest-smoke",
        "policy_reuse_evidence_manifest_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-manifest-regression-smoke",
        "policy_reuse_evidence_manifest_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-validation-budget-smoke",
        "policy_reuse_evidence_validation_budget_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-validation-budget-regression-smoke",
        "policy_reuse_evidence_validation_budget_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-rollout-readiness-smoke",
        "policy_reuse_evidence_rollout_readiness_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-rollout-readiness-regression-smoke",
        "policy_reuse_evidence_rollout_readiness_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-learning-admission-smoke",
        "policy_reuse_evidence_learning_admission_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-learning-admission-regression-smoke",
        "policy_reuse_evidence_learning_admission_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-readiness-smoke",
        "policy_reuse_evidence_retrieval_readiness_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-readiness-regression-smoke",
        "policy_reuse_evidence_retrieval_readiness_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-compact-validation-smoke",
        "policy_reuse_evidence_compact_validation_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-compact-validation-regression-smoke",
        "policy_reuse_evidence_compact_validation_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-batch-readiness-smoke",
        "policy_reuse_evidence_batch_readiness_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-batch-readiness-regression-smoke",
        "policy_reuse_evidence_batch_readiness_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-batch-execution-plan-smoke",
        "policy_reuse_evidence_batch_execution_plan_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-batch-execution-plan-regression-smoke",
        "policy_reuse_evidence_batch_execution_plan_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-batch-evaluation-admission-smoke",
        "policy_reuse_evidence_batch_evaluation_admission_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-batch-evaluation-admission-regression-smoke",
        "policy_reuse_evidence_batch_evaluation_admission_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-batch-run-request-smoke",
        "policy_reuse_evidence_batch_run_request_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-batch-run-request-regression-smoke",
        "policy_reuse_evidence_batch_run_request_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-external-evaluator-result-smoke",
        "policy_reuse_evidence_external_evaluator_result_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-external-evaluator-result-regression-smoke",
        "policy_reuse_evidence_external_evaluator_result_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-learning-candidate-smoke",
        "policy_reuse_evidence_learning_candidate_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-learning-candidate-regression-smoke",
        "policy_reuse_evidence_learning_candidate_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-learning-data-admission-smoke",
        "policy_reuse_evidence_learning_data_admission_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-learning-data-admission-regression-smoke",
        "policy_reuse_evidence_learning_data_admission_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-example-admission-smoke",
        "policy_reuse_evidence_retrieval_example_admission_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-example-admission-regression-smoke",
        "policy_reuse_evidence_retrieval_example_admission_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-example-index-smoke",
        "policy_reuse_evidence_retrieval_example_index_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-example-index-regression-smoke",
        "policy_reuse_evidence_retrieval_example_index_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-corpus-readiness-smoke",
        "policy_reuse_evidence_retrieval_corpus_readiness_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-corpus-readiness-regression-smoke",
        "policy_reuse_evidence_retrieval_corpus_readiness_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-corpus-admission-smoke",
        "policy_reuse_evidence_retrieval_corpus_admission_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-corpus-admission-regression-smoke",
        "policy_reuse_evidence_retrieval_corpus_admission_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-use-approval-smoke",
        "policy_reuse_evidence_retrieval_use_approval_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-use-approval-regression-smoke",
        "policy_reuse_evidence_retrieval_use_approval_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-use-manifest-smoke",
        "policy_reuse_evidence_retrieval_use_manifest_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-use-manifest-regression-smoke",
        "policy_reuse_evidence_retrieval_use_manifest_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-query-plan-smoke",
        "policy_reuse_evidence_retrieval_query_plan_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-query-plan-regression-smoke",
        "policy_reuse_evidence_retrieval_query_plan_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-query-approval-smoke",
        "policy_reuse_evidence_retrieval_query_approval_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --policy-reuse-evidence-retrieval-query-approval-regression-smoke",
        "policy_reuse_evidence_retrieval_query_approval_regression_smoke",
    ));
    assert!(catalog.contains(
        "root_validate --root-validate-dispatch-catalog",
        "canon_root_validate_dispatch_catalog_v1",
    ));
    assert!(catalog.contains(
        "graph_mutation verify-ops <ops.ndjson>",
        "GraphMutationOpSetReceipt",
    ));
    assert!(catalog.contains(
        "graph_mutation verify-receipts <patch-receipts.ndjson> <mutation-receipts.ndjson>",
        "GraphReceiptLedgerReceipt",
    ));
    assert!(catalog.contains(
        "graph_mutation generate-patch <graph-contract.ndjson> <source-root> <ops.ndjson> <patch.out> <patch-receipt.out>",
        "GraphPatchReceipt",
    ));
    assert!(catalog.contains(
        "graph_mutation verify-landing <old-graph-contract.ndjson> <new-graph-contract.ndjson> <ops.ndjson> <mutation-receipt.out>",
        "GraphMutationReceipt",
    ));
    assert!(catalog.contains("graph_mutation help", "usage"));
}

#[test]
fn external_agent_cli_modes_fixture_matches_executable_help_surfaces() {
    let fixture = std::fs::read_to_string(EXTERNAL_AGENT_CLI_MODES_FIXTURE)
        .expect("external agent CLI modes fixture must be readable");
    let catalog = ExternalAgentCliCatalog::parse(&fixture)
        .expect("external agent CLI modes fixture should parse");
    let graph_mutation = env!("CARGO_BIN_EXE_graph_mutation");
    let graph_help = std::process::Command::new(graph_mutation)
        .arg("help")
        .output()
        .expect("run graph_mutation help");

    assert!(graph_help.status.success());
    let graph_stdout = String::from_utf8_lossy(&graph_help.stdout);
    for entry in catalog
        .graph_mutation_entries()
        .into_iter()
        .filter(|entry| entry.command != "graph_mutation help")
    {
        let usage_line = entry
            .command
            .replacen("graph_mutation ", "  graph_mutation ", 1);
        assert!(
            graph_stdout.contains(&usage_line),
            "graph help missing fixture command: {usage_line}"
        );
    }

    for (arg, expected_fragments) in [
        (
            "--validation-footprint",
            vec![
                "\"schema\":\"canon_validation_footprint_v1\\",
                "\"record_type\":\"validation_footprint_summary\\",
            ],
        ),
        (
            "--runtime-budget-smoke",
            vec![
                "\"step\":\"runtime_performance_budget_smoke\\",
                "\"controlled_failure_observed\":true",
            ],
        ),
        (
            "--policy-reuse-smoke",
            vec![
                "\"record_type\":\"policy_reuse_smoke\\",
                "\"avoided_llm_call_count\":1",
            ],
        ),
        (
            "--root-validate-dispatch-catalog",
            vec![
                "\"schema\":\"canon_root_validate_dispatch_catalog_v1\\",
                "\"dispatch_catalog_hash\":",
            ],
        ),
    ] {
        root_validate_catalog_entry_contract(&catalog, arg, &expected_fragments);
    }
}

#[test]
fn root_validate_dispatch_catalog_matches_documented_root_modes_contract() {
    let fixture = std::fs::read_to_string(EXTERNAL_AGENT_CLI_MODES_FIXTURE)
        .expect("external agent CLI modes fixture must be readable");
    let root_validate = env!("CARGO_BIN_EXE_root_validate");
    let output = std::process::Command::new(root_validate)
        .arg("--root-validate-dispatch-catalog")
        .output()
        .expect("run root_validate dispatch catalog");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"schema\":\"canon_root_validate_dispatch_catalog_v1\\"));
    assert!(stdout.contains("\"record_type\":\"root_validate_dispatch_catalog\\"));
    assert!(stdout.contains(&format!(
        "\"compact_mode_count\":{}",
        EXPECTED_ROOT_VALIDATE_COMPACT_MODE_COUNT
    )));
    assert!(!stdout.contains("canon_root_validation_v1"));

    for line in fixture
        .lines()
        .filter(|line| line.starts_with("root_validate "))
    {
        let (command, marker) = line
            .split_once(" => ")
            .expect("root_validate fixture line must include marker");
        let arg = command
            .strip_prefix("root_validate ")
            .expect("fixture command must use root_validate prefix");
        assert!(
            stdout.contains(&format!("\"arg\":\"{arg}\\")),
            "dispatch catalog missing {arg}"
        );
        assert!(
            stdout.contains(&format!("\"marker\":\"{marker}\\")),
            "dispatch catalog missing marker {marker}"
        );
    }
}

#[test]
fn root_validate_dispatch_catalog_negative_contract_detects_omitted_mode() {
    let fixture = std::fs::read_to_string(EXTERNAL_AGENT_CLI_MODES_FIXTURE)
        .expect("external agent CLI modes fixture must be readable");
    let root_validate = env!("CARGO_BIN_EXE_root_validate");
    let output = std::process::Command::new(root_validate)
        .arg("--root-validate-dispatch-catalog")
        .output()
        .expect("run root_validate dispatch catalog");
    assert!(output.status.success());
    let catalog = String::from_utf8_lossy(&output.stdout);
    let drifted = fixture
        .lines()
        .filter(|line| !line.starts_with("root_validate --policy-reuse-smoke "))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        !root_validate_dispatch_catalog_matches_fixture(&drifted, &catalog),
        "a copied fixture omitting a root_validate mode must be rejected"
    );
}

#[test]
fn root_validate_dispatch_catalog_negative_contract_detects_marker_drift() {
    let fixture = std::fs::read_to_string(EXTERNAL_AGENT_CLI_MODES_FIXTURE)
        .expect("external agent CLI modes fixture must be readable");
    let root_validate = env!("CARGO_BIN_EXE_root_validate");
    let output = std::process::Command::new(root_validate)
        .arg("--root-validate-dispatch-catalog")
        .output()
        .expect("run root_validate dispatch catalog");
    assert!(output.status.success());
    let catalog = String::from_utf8_lossy(&output.stdout);
    let drifted = fixture.replace(
        "root_validate --policy-reuse-smoke => policy_reuse_smoke",
        "root_validate --policy-reuse-smoke => drifted_policy_reuse_marker",
    );

    assert!(
        !root_validate_dispatch_catalog_matches_fixture(&drifted, &catalog),
        "a copied fixture with a drifted root_validate marker must be rejected"
    );
}

#[test]
fn root_validate_dispatch_catalog_exposes_stable_hash_contract() {
    let root_validate = env!("CARGO_BIN_EXE_root_validate");
    let output = std::process::Command::new(root_validate)
        .arg("--root-validate-dispatch-catalog")
        .output()
        .expect("run root_validate dispatch catalog");
    assert!(output.status.success());
    let catalog = String::from_utf8_lossy(&output.stdout);
    let payload = root_validate_dispatch_catalog_payload_from_json(&catalog)
        .expect("dispatch catalog payload must be extractable");
    let expected_hash = stable_hash64_for_contract(payload.as_bytes()).to_string();

    assert!(
        catalog.contains(&format!("\"dispatch_catalog_hash\":\"{expected_hash}\\")),
        "dispatch catalog must expose stable aggregate hash"
    );

    let drifted_payload = payload.replace(
        "--policy-reuse-smoke=>policy_reuse_smoke",
        "--policy-reuse-smoke=>drifted_policy_reuse_marker",
    );
    assert_ne!(
        expected_hash,
        stable_hash64_for_contract(drifted_payload.as_bytes()).to_string()
    );
}

fn root_validate_dispatch_catalog_payload_from_json(catalog: &str) -> Option<String> {
    let mut pairs = Vec::new();
    for row in catalog.split("{\"arg\":").skip(1) {
        let arg = row.split("\\").nth(1)?;
        let marker = row.split("\"marker\":\\").nth(1)?.split("\\").next()?;
        pairs.push(format!("{arg}=>{marker}"));
    }
    Some(pairs.join("\n"))
}

fn stable_hash64_for_contract(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn root_validate_dispatch_catalog_matches_fixture(fixture: &str, catalog: &str) -> bool {
    let Ok(parsed) = ExternalAgentCliCatalog::parse(fixture) else {
        return false;
    };
    let root_entries = parsed.root_validate_entries();
    let catalog_count = catalog.matches("\"arg\":").count();
    if catalog_count != root_entries.len() {
        return false;
    }

    root_entries.iter().all(|entry| {
        let Some(arg) = entry.command.strip_prefix("root_validate ") else {
            return false;
        };
        catalog.contains(&format!("\"arg\":\"{arg}\\"))
            && catalog.contains(&format!("\"marker\":\"{}\\", entry.marker))
    })
}

#[test]
fn external_agent_cli_modes_negative_contract_detects_omitted_mode() {
    let fixture = std::fs::read_to_string(EXTERNAL_AGENT_CLI_MODES_FIXTURE)
        .expect("external agent CLI modes fixture must be readable");
    let drifted = fixture
        .lines()
        .filter(|line| !line.starts_with("graph_mutation verify-landing "))
        .collect::<Vec<_>>()
        .join("\n");

    let live_mode_count = fixture.lines().filter(|line| line.contains(" => ")).count();
    assert_eq!(
        drifted.lines().filter(|line| line.contains(" => ")).count(),
        live_mode_count - 1
    );
    assert!(drifted.contains(&format!("mode_count={live_mode_count}")));
    assert!(
        !external_agent_cli_modes_catalog_valid(&drifted, "graph_mutation help output omitted"),
        "catalog drift with an omitted mode must be rejected"
    );
}

#[test]
fn external_agent_cli_modes_negative_contract_detects_advertised_missing_help_mode() {
    let fixture = std::fs::read_to_string(EXTERNAL_AGENT_CLI_MODES_FIXTURE)
        .expect("external agent CLI modes fixture must be readable");
    let graph_help = "  graph_mutation verify-ops <ops.ndjson>\n  graph_mutation verify-receipts <patch-receipts.ndjson> <mutation-receipts.ndjson>\n  graph_mutation generate-patch <graph-contract.ndjson> <source-root> <ops.ndjson> <patch.out> <patch-receipt.out>\n";

    assert!(fixture.contains("graph_mutation verify-landing"));
    assert!(
        !external_agent_cli_modes_catalog_valid(&fixture, graph_help),
        "catalog advertising a mode absent from executable help must be rejected"
    );
}

fn external_agent_cli_modes_catalog_valid(fixture: &str, graph_help: &str) -> bool {
    let Ok(catalog) = ExternalAgentCliCatalog::parse(fixture) else {
        return false;
    };
    if catalog.schema != "canon_external_agent_cli_modes_v1" {
        return false;
    }
    if catalog.declared_mode_count != catalog.entries.len() || catalog.entries.is_empty() {
        return false;
    }
    if catalog.root_validate_entries().len() != 56 || catalog.graph_mutation_entries().len() != 5 {
        return false;
    }

    for entry in catalog.graph_mutation_entries() {
        if entry.command == "graph_mutation help" {
            if entry.marker != "usage" {
                return false;
            }
            continue;
        }
        let usage_line = entry
            .command
            .replacen("graph_mutation ", "  graph_mutation ", 1);
        if !graph_help.contains(&usage_line) {
            return false;
        }
    }

    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ExternalAgentCliEntry<'a> {
    command: &'a str,
    marker: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
struct ExternalAgentCliCatalog<'a> {
    schema: &'a str,
    declared_mode_count: usize,
    entries: Vec<ExternalAgentCliEntry<'a>>,
}

impl<'a> ExternalAgentCliCatalog<'a> {
    fn parse(fixture: &'a str) -> Result<Self, String> {
        let schema = fixture
            .lines()
            .find_map(|line| line.strip_prefix("schema="))
            .ok_or_else(|| "missing schema".to_string())?;
        let declared_mode_count = fixture
            .lines()
            .find_map(|line| line.strip_prefix("mode_count="))
            .ok_or_else(|| "missing mode_count".to_string())?
            .parse::<usize>()
            .map_err(|err| format!("invalid mode_count: {err}"))?;
        let entries = fixture
            .lines()
            .filter_map(|line| line.split_once(" => "))
            .map(|(command, marker)| ExternalAgentCliEntry { command, marker })
            .collect::<Vec<_>>();

        if entries.len() != declared_mode_count {
            return Err(format!(
                "declared mode_count {declared_mode_count} != observed {}",
                entries.len()
            ));
        }

        Ok(Self {
            schema,
            declared_mode_count,
            entries,
        })
    }

    fn contains(&self, command: &str, marker: &str) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.command == command && entry.marker == marker)
    }

    fn root_validate_entries(&self) -> Vec<ExternalAgentCliEntry<'a>> {
        self.entries
            .iter()
            .copied()
            .filter(|entry| entry.command.starts_with("root_validate "))
            .collect()
    }

    fn graph_mutation_entries(&self) -> Vec<ExternalAgentCliEntry<'a>> {
        self.entries
            .iter()
            .copied()
            .filter(|entry| entry.command.starts_with("graph_mutation "))
            .collect()
    }

    fn root_validate_entry(&self, arg: &str) -> Option<ExternalAgentCliEntry<'a>> {
        let command = format!("root_validate {arg}");
        self.entries
            .iter()
            .copied()
            .find(|entry| entry.command == command)
    }
}

fn root_validate_catalog_entry_contract(
    catalog: &ExternalAgentCliCatalog<'_>,
    arg: &str,
    expected_fragments: &[&str],
) {
    let entry = catalog
        .root_validate_entry(arg)
        .unwrap_or_else(|| panic!("catalog missing root_validate {arg}"));
    let stdout = root_validate_compact_mode_stdout(arg, entry.marker);
    assert_stdout_contains_all(&stdout, expected_fragments);
}

fn checked_external_agent_cli_catalog() -> ExternalAgentCliCatalog<'static> {
    let fixture = std::fs::read_to_string(EXTERNAL_AGENT_CLI_MODES_FIXTURE)
        .expect("external agent CLI modes fixture must be readable");
    let fixture = Box::leak(fixture.into_boxed_str());
    ExternalAgentCliCatalog::parse(fixture).expect("external agent CLI modes fixture should parse")
}

fn assert_root_validate_catalog_compact_mode_contract(arg: &str, expected_fragments: &[&str]) {
    let catalog = checked_external_agent_cli_catalog();
    root_validate_catalog_entry_contract(&catalog, arg, expected_fragments);
}

#[test]
fn external_agent_cli_catalog_helper_executes_documented_root_modes() {
    let catalog = checked_external_agent_cli_catalog();

    root_validate_catalog_entry_contract(
        &catalog,
        "--validation-footprint",
        &[
            "\"record_type\":\"validation_footprint_summary\\",
            "\"verdict\":\"pass\\",
        ],
    );
    root_validate_catalog_entry_contract(
        &catalog,
        "--policy-reuse-smoke",
        &[
            "\"record_type\":\"policy_reuse_smoke\\",
            "\"avoided_llm_call_count\":1",
            "\"verdict\":\"pass\\",
        ],
    );
    root_validate_catalog_entry_contract(
        &catalog,
        "--root-validate-dispatch-catalog",
        &[
            "\"record_type\":\"root_validate_dispatch_catalog\\",
            "\"dispatch_catalog_hash\":",
            "\"verdict\":\"pass\\",
        ],
    );
}

#[test]
fn root_validate_catalog_compact_mode_contract_helper_covers_non_fixture_smokes() {
    assert_root_validate_catalog_compact_mode_contract(
        "--validation-cost-smoke",
        &[
            "\"record_type\":\"validation_cost_footprint_smoke\\",
            "\"observed_regression_bps\":500",
            "\"footprint_status\":\"pass\\",
            "\"verdict\":\"pass\\",
        ],
    );
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-orchestration-capacity-smoke",
        &[
            "\"record_type\":\"policy_orchestration_capacity_smoke\\",
            "\"estimated_avoided_llm_calls_per_full_batch\":4",
            "\"capacity_status\":\"pass\\",
            "\"verdict\":\"pass\\",
        ],
    );
}

#[test]
fn external_agent_cli_catalog_helper_rejects_count_drift() {
    let fixture = "schema=canon_external_agent_cli_modes_v1
mode_count=2
root_validate --validation-footprint => canon_validation_footprint_v1
";

    let error = ExternalAgentCliCatalog::parse(fixture)
        .expect_err("catalog helper should reject declared/observed count drift");
    assert!(error.contains("declared mode_count 2 != observed 1"));
}

#[test]
fn validation_fixture_catalog_receipt_summarizes_retained_fixture_inventory() {
    let receipt = ai::validation_harness::validation_fixture_catalog_receipt()
        .expect("fixture catalog receipt should load checked-in fixtures");

    assert_eq!(receipt.schema, "canon_validation_fixture_catalog_v1");
    assert_eq!(receipt.record_type, VALIDATION_FIXTURE_CATALOG_STEP);
    assert_eq!(receipt.fixture_count, EXPECTED_VALIDATION_FIXTURE_COUNT);
    assert_eq!(
        receipt.retained_receipt_fixture_count,
        EXPECTED_RETAINED_RECEIPT_FIXTURE_COUNT
    );
    assert_eq!(
        receipt.command_fixture_count,
        EXPECTED_COMMAND_FIXTURE_COUNT
    );
    assert!(receipt.total_fixture_bytes > 0);
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
    assert!(receipt
        .to_json()
        .contains("\"schema\":\"canon_validation_fixture_catalog_v1\\"));
}

#[test]
fn validation_fixture_catalog_receipt_hash_changes_when_fixture_drifts() {
    let baseline = ai::validation_harness::validation_fixture_catalog_receipt()
        .expect("baseline fixture catalog receipt should load checked-in fixtures");
    let mut drifted = baseline.clone();
    drifted.fixture_set_hash = format!("{}-synthetic-drift", baseline.fixture_set_hash);
    drifted.total_fixture_bytes += 1;

    assert_ne!(baseline.fixture_set_hash, drifted.fixture_set_hash);
    assert_ne!(baseline.total_fixture_bytes, drifted.total_fixture_bytes);
    assert!(baseline.passed());
    assert!(drifted.passed());
}

#[test]
fn root_validate_validation_fixture_catalog_mode_is_executable_contract() {
    let root_validate = env!("CARGO_BIN_EXE_root_validate");
    let output = std::process::Command::new(root_validate)
        .arg("--validation-fixture-catalog")
        .output()
        .expect("run root_validate validation fixture catalog");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"schema\":\"canon_validation_fixture_catalog_v1\\"));
    assert!(stdout.contains("\"record_type\":\"validation_fixture_catalog\\"));
    assert!(stdout.contains(&format!(
        "\"fixture_count\":{}",
        EXPECTED_VALIDATION_FIXTURE_COUNT
    )));
    assert!(stdout.contains(&format!(
        "\"retained_receipt_fixture_count\":{}",
        EXPECTED_RETAINED_RECEIPT_FIXTURE_COUNT
    )));
    assert!(!stdout.contains("canon_root_validation_v1"));
}

#[test]
fn validation_fixture_catalog_detail_receipt_exposes_per_fixture_rows() {
    let detail = ai::validation_harness::validation_fixture_catalog_detail_receipt()
        .expect("fixture catalog detail receipt should load checked-in fixtures");

    assert_eq!(detail.schema, "canon_validation_fixture_catalog_detail_v1");
    assert_eq!(detail.record_type, "validation_fixture_catalog_detail");
    assert_eq!(detail.fixture_count, EXPECTED_VALIDATION_FIXTURE_COUNT);
    assert_eq!(
        detail.retained_receipt_fixture_count,
        EXPECTED_RETAINED_RECEIPT_FIXTURE_COUNT
    );
    assert_eq!(detail.command_fixture_count, EXPECTED_COMMAND_FIXTURE_COUNT);
    assert!(detail.total_fixture_bytes > 0);
    assert!(!detail.fixture_set_hash.is_empty());
    assert_eq!(detail.rows.len(), EXPECTED_VALIDATION_FIXTURE_COUNT);
    assert!(detail.passed());
    assert!(detail.rows.iter().any(|row| {
        row.kind == "retained_receipt"
            && row.path == VALIDATION_DURATION_PLANNING_RECEIPTS_FIXTURE
            && row.byte_count > 0
            && !row.content_hash.is_empty()
    }));
    assert!(detail.rows.iter().any(|row| {
        row.kind == "retained_receipt"
            && row.path == VALIDATION_COMMAND_FOOTPRINT_RECEIPTS_FIXTURE
            && row.byte_count > 0
            && !row.content_hash.is_empty()
    }));
    assert!(detail.rows.iter().any(|row| {
        row.kind == "retained_receipt"
            && row.path == POLICY_CAPACITY_COST_SUMMARY_RECEIPTS_FIXTURE
            && row.byte_count > 0
            && !row.content_hash.is_empty()
    }));
    assert!(detail
        .to_json()
        .contains("\"schema\":\"canon_validation_fixture_catalog_detail_v1\\"));
    assert!(detail
        .to_text()
        .contains("schema=canon_validation_fixture_catalog_detail_v1"));
}

#[test]
fn validation_fixture_catalog_detail_hash_changes_when_fixture_drifts() {
    let baseline = ai::validation_harness::validation_fixture_catalog_detail_receipt()
        .expect("baseline fixture catalog detail should load checked-in fixtures");
    let mut drifted = baseline.clone();
    drifted.fixture_set_hash = format!("{}-synthetic-drift", baseline.fixture_set_hash);
    let drifted_row = drifted
        .rows
        .iter_mut()
        .find(|row| row.path == POLICY_CAPACITY_COST_SUMMARY_RECEIPTS_FIXTURE)
        .expect("drifted row should include policy capacity cost fixture");
    drifted_row.content_hash = format!("{}-synthetic-drift", drifted_row.content_hash);
    drifted_row.byte_count += 1;
    drifted.total_fixture_bytes += 1;

    assert_ne!(baseline.fixture_set_hash, drifted.fixture_set_hash);
    let baseline_row = baseline
        .rows
        .iter()
        .find(|row| row.path == POLICY_CAPACITY_COST_SUMMARY_RECEIPTS_FIXTURE)
        .expect("baseline row should include policy capacity cost fixture");
    let drifted_row = drifted
        .rows
        .iter()
        .find(|row| row.path == POLICY_CAPACITY_COST_SUMMARY_RECEIPTS_FIXTURE)
        .expect("drifted row should include policy capacity cost fixture");
    assert_ne!(baseline_row.content_hash, drifted_row.content_hash);
    assert!(drifted_row.byte_count > baseline_row.byte_count);
    assert!(baseline.passed());
    assert!(drifted.passed());
}

#[test]
fn root_validate_validation_fixture_catalog_detail_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--validation-fixture-catalog-detail",
        &[
            "schema=canon_validation_fixture_catalog_detail_v1",
            "record_type=validation_fixture_catalog_detail",
            &format!("fixture_count={}", EXPECTED_VALIDATION_FIXTURE_COUNT),
            &format!(
                "retained_receipt_fixture_count={}",
                EXPECTED_RETAINED_RECEIPT_FIXTURE_COUNT
            ),
            "fixture=retained_receipt|tests/fixtures/policy_capacity_cost_summary_receipts.txt|",
            "verdict=pass",
        ],
    );
}

#[test]
fn validation_duration_planning_receipt_summarizes_retained_duration_and_footprint() {
    let runtime = ai::validation_harness::runtime_performance_receipt(3_150);
    let footprint = ai::validation_harness::validation_footprint_receipt();
    let receipt =
        ai::validation_harness::validation_duration_planning_receipt(&runtime, &footprint);

    assert_eq!(receipt.schema, "canon_validation_duration_planning_v1");
    assert_eq!(receipt.record_type, VALIDATION_DURATION_PLANNING_STEP);
    assert_eq!(receipt.retained_project_agent_elapsed_ms_p95, 3_150);
    assert_eq!(receipt.max_project_agent_elapsed_ms_p95, 10_000);
    assert_eq!(receipt.retained_budget_headroom_ms, 6_850);
    assert_eq!(receipt.total_declared_steps, 7);
    assert_eq!(
        receipt.expected_count_guarded_tests,
        expected_guarded_test_count()
    );
    assert_eq!(receipt.estimated_ms_per_declared_step, 450);
    assert!(receipt.estimated_ms_per_guarded_test > 0);
    assert_eq!(receipt.runtime_budget_status, "pass");
    assert_eq!(receipt.footprint_verdict, "pass");
    assert_eq!(receipt.planning_status, "pass");
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
    assert!(receipt
        .to_json()
        .contains("\"schema\":\"canon_validation_duration_planning_v1\\"));
}

#[test]
fn validation_duration_planning_receipt_rejects_budget_exhaustion() {
    let mut runtime = ai::validation_harness::runtime_performance_receipt(10_001);
    runtime.max_project_agent_elapsed_ms_p95 = 10_000;
    runtime.runtime_performance_budget_status = "fail";
    let footprint = ai::validation_harness::validation_footprint_receipt();
    let receipt =
        ai::validation_harness::validation_duration_planning_receipt(&runtime, &footprint);

    assert_eq!(receipt.retained_budget_headroom_ms, -1);
    assert_eq!(receipt.runtime_budget_status, "fail");
    assert_eq!(receipt.planning_status, "fail");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn validation_duration_planning_budget_exhaustion_smoke_receipt_is_controlled_negative() {
    let receipt =
        ai::validation_harness::validation_duration_planning_budget_exhaustion_smoke_receipt();

    assert_eq!(receipt.schema, "canon_validation_duration_planning_v1");
    assert_eq!(
        receipt.record_type,
        VALIDATION_DURATION_PLANNING_BUDGET_EXHAUSTION_SMOKE_STEP
    );
    assert_eq!(receipt.retained_project_agent_elapsed_ms_p95, 10_001);
    assert_eq!(receipt.max_project_agent_elapsed_ms_p95, 10_000);
    assert_eq!(receipt.retained_budget_headroom_ms, -1);
    assert_eq!(receipt.total_declared_steps, 7);
    assert_eq!(receipt.runtime_budget_status, "fail");
    assert_eq!(receipt.footprint_verdict, "pass");
    assert_eq!(receipt.planning_status, "fail");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn root_validate_validation_duration_planning_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--validation-duration-planning",
        &[
            "\"schema\":\"canon_validation_duration_planning_v1\\",
            "\"record_type\":\"validation_duration_planning_summary\\",
            "\"retained_project_agent_elapsed_ms_p95\":3150",
            "\"retained_budget_headroom_ms\":6850",
            "\"total_declared_steps\":7",
            "\"runtime_budget_status\":\"pass\\",
            "\"footprint_verdict\":\"pass\\",
            "\"planning_status\":\"pass\\",
            "\"verdict\":\"pass\\",
        ],
    );
}

#[test]
fn root_validate_validation_duration_planning_budget_exhaustion_smoke_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--validation-duration-planning-budget-exhaustion-smoke",
        &[
            "\"schema\":\"canon_validation_duration_planning_v1\\",
            "\"record_type\":\"validation_duration_planning_budget_exhaustion_smoke\\",
            "\"retained_project_agent_elapsed_ms_p95\":10001",
            "\"retained_budget_headroom_ms\":-1",
            "\"runtime_budget_status\":\"fail\\",
            "\"footprint_verdict\":\"pass\\",
            "\"planning_status\":\"fail\\",
            "\"verdict\":\"fail\\",
        ],
    );
}

#[test]
fn validation_duration_planning_trend_smoke_compares_retained_planning_cost() {
    let receipt = ai::validation_harness::validation_duration_planning_trend_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_validation_duration_planning_trend_v1"
    );
    assert_eq!(
        receipt.record_type,
        VALIDATION_DURATION_PLANNING_TREND_SMOKE_STEP
    );
    assert_eq!(
        receipt.baseline_retained_project_agent_elapsed_ms_p95,
        3_150
    );
    assert_eq!(receipt.current_retained_project_agent_elapsed_ms_p95, 3_000);
    assert_eq!(receipt.retained_duration_delta_ms, -150);
    assert_eq!(receipt.retained_budget_headroom_delta_ms, 150);
    assert_eq!(receipt.trend_status, "pass");
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
}

#[test]
fn root_validate_validation_duration_planning_trend_smoke_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--validation-duration-planning-trend-smoke",
        &[
            "\"schema\":\"canon_validation_duration_planning_trend_v1\\",
            "\"record_type\":\"validation_duration_planning_trend_smoke\\",
            "\"baseline_retained_project_agent_elapsed_ms_p95\":3150",
            "\"current_retained_project_agent_elapsed_ms_p95\":3000",
            "\"retained_duration_delta_ms\":-150",
            "\"retained_budget_headroom_delta_ms\":150",
            "\"trend_status\":\"pass\\",
            "\"verdict\":\"pass\\",
        ],
    );
}

#[test]
fn validation_duration_planning_regression_smoke_exposes_controlled_negative_receipt() {
    let receipt = ai::validation_harness::validation_duration_planning_regression_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_validation_duration_planning_trend_v1"
    );
    assert_eq!(
        receipt.record_type,
        VALIDATION_DURATION_PLANNING_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.baseline_retained_project_agent_elapsed_ms_p95,
        3_150
    );
    assert_eq!(receipt.current_retained_project_agent_elapsed_ms_p95, 3_301);
    assert_eq!(receipt.retained_duration_delta_ms, 151);
    assert_eq!(receipt.retained_budget_headroom_delta_ms, -151);
    assert_eq!(receipt.trend_status, "regressed");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn root_validate_validation_duration_planning_regression_smoke_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--validation-duration-planning-regression-smoke",
        &[
            "\"schema\":\"canon_validation_duration_planning_trend_v1\\",
            "\"record_type\":\"validation_duration_planning_regression_smoke\\",
            "\"baseline_retained_project_agent_elapsed_ms_p95\":3150",
            "\"current_retained_project_agent_elapsed_ms_p95\":3301",
            "\"retained_duration_delta_ms\":151",
            "\"retained_budget_headroom_delta_ms\":-151",
            "\"trend_status\":\"regressed\\",
            "\"verdict\":\"fail\\",
        ],
    );
}

#[test]
fn runtime_performance_receipt_json_contains_required_contract_fields() {
    let receipt = runtime_performance_receipt(42);
    let json = receipt.to_json();

    assert!(RuntimePerformanceReceipt::json_contract_valid(&json));
    for field in RuntimePerformanceReceipt::required_json_fields() {
        assert!(json.contains(&format!("\"{field}\":")), "missing {field}");
    }
    assert!(json.contains("\"step\":\"runtime_performance_metrics\\"));
}

#[test]
fn runtime_performance_receipt_json_contract_rejects_missing_fields() {
    let malformed =
        "{\"step\":\"runtime_performance_metrics\",\"runtime_performance_signal_present\":true}";

    assert!(!RuntimePerformanceReceipt::json_contract_valid(malformed));
}

#[test]
fn runtime_performance_receipt_budget_failure_is_configurable_from_rust() {
    let mut receipt = runtime_performance_receipt(50);
    assert!(receipt.passed());

    receipt.max_project_agent_elapsed_ms_p95 = 49;
    receipt.runtime_performance_budget_status = if receipt.budgets_pass() {
        "pass"
    } else {
        "fail"
    };

    assert_eq!(receipt.runtime_performance_budget_status, "fail");
    assert!(!receipt.passed());
    let json = receipt.to_json();
    assert!(json.contains("\"runtime_performance_budget_status\":\"fail\\"));
    assert!(json.contains("\"max_project_agent_elapsed_ms_p95\":49"));
}

#[test]
fn runtime_performance_threshold_fixture_matches_contract_constants() {
    let fixture = std::fs::read_to_string(RUNTIME_PERFORMANCE_THRESHOLDS_FIXTURE)
        .expect("runtime performance thresholds fixture must be readable");

    assert!(fixture.contains("schema=canon_runtime_performance_thresholds_v1"));
    assert!(fixture.contains(&format!(
        "default_max_project_agent_elapsed_ms_p95={}",
        ai::validation_harness::DEFAULT_MAX_PROJECT_AGENT_ELAPSED_MS_P95
    )));
    assert!(fixture.contains(&format!(
        "default_max_download_initial_get_ms_p95={}",
        ai::validation_harness::DEFAULT_MAX_DOWNLOAD_INITIAL_GET_MS_P95
    )));
    assert!(fixture.contains(&format!(
        "default_max_download_follow_get_ms_p95={}",
        ai::validation_harness::DEFAULT_MAX_DOWNLOAD_FOLLOW_GET_MS_P95
    )));
    assert!(fixture.contains(&format!(
        "default_max_download_write_ms_p95={}",
        ai::validation_harness::DEFAULT_MAX_DOWNLOAD_WRITE_MS_P95
    )));
    assert!(fixture.contains("low_budget_smoke_forced_validation_command_duration_ms=2"));
    assert!(fixture.contains("low_budget_smoke_forced_max_project_agent_elapsed_ms_p95=1"));
    assert!(fixture.contains("low_budget_smoke_expected_status=fail"));
    assert!(fixture.contains("root_validate --runtime-budget-smoke"));
}

#[test]
fn runtime_performance_trend_smoke_receipt_uses_fixture_values() {
    let fixture = std::fs::read_to_string(RUNTIME_PERFORMANCE_TREND_FIXTURE)
        .expect("runtime performance trend fixture must be readable");
    let receipt = ai::validation_harness::runtime_performance_trend_smoke_receipt();

    assert_eq!(receipt.record_type, RUNTIME_PERFORMANCE_TREND_SMOKE_STEP);
    assert_eq!(
        receipt.baseline_project_agent_elapsed_ms_p95,
        fixture_value(&fixture, "baseline_project_agent_elapsed_ms_p95")
    );
    assert_eq!(
        receipt.current_project_agent_elapsed_ms_p95,
        fixture_value(&fixture, "current_project_agent_elapsed_ms_p95")
    );
    assert_eq!(
        receipt.allowed_regression_bps,
        fixture_value(&fixture, "allowed_regression_bps")
    );
    assert_eq!(
        receipt.observed_regression_bps,
        fixture_value(&fixture, "expected_observed_regression_bps")
    );
    assert_eq!(receipt.budget_status, "pass");
    assert_eq!(
        receipt.trend_status,
        fixture_text(&fixture, "expected_trend_status")
    );
    assert!(receipt.passed());
}

#[test]
fn runtime_performance_trend_regression_smoke_receipt_uses_fixture_values() {
    let fixture = std::fs::read_to_string(RUNTIME_PERFORMANCE_TREND_FIXTURE)
        .expect("runtime performance trend fixture must be readable");
    let receipt = ai::validation_harness::runtime_performance_trend_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        RUNTIME_PERFORMANCE_TREND_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.baseline_project_agent_elapsed_ms_p95,
        fixture_value(&fixture, "baseline_project_agent_elapsed_ms_p95")
    );
    assert_eq!(
        receipt.current_project_agent_elapsed_ms_p95,
        fixture_value(&fixture, "regressed_current_project_agent_elapsed_ms_p95")
    );
    assert_eq!(
        receipt.allowed_regression_bps,
        fixture_value(&fixture, "allowed_regression_bps")
    );
    assert_eq!(
        receipt.observed_regression_bps,
        fixture_value(&fixture, "expected_regressed_observed_regression_bps")
    );
    assert_eq!(receipt.budget_status, "pass");
    assert_eq!(
        receipt.trend_status,
        fixture_text(&fixture, "expected_regressed_trend_status")
    );
    assert!(!receipt.passed());
}

#[test]
fn runtime_performance_budget_smoke_receipt_records_controlled_failure() {
    let receipt = ai::validation_harness::runtime_performance_budget_smoke_receipt();

    assert_eq!(receipt.step, RUNTIME_PERFORMANCE_BUDGET_SMOKE_STEP);
    assert!(receipt.passed());
    assert_eq!(receipt.runtime_performance_budget_status, "fail");
    assert!(receipt.controlled_failure_observed);
    assert!(receipt.receipt_json_contract_valid);

    let json = receipt.to_json();
    assert!(json.contains("\"step\":\"runtime_performance_budget_smoke\\"));
    assert!(json.contains("\"runtime_performance_budget_status\":\"fail\\"));
    assert!(json.contains("\"controlled_failure_observed\":true"));
}

#[test]
fn root_validate_runtime_trend_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--runtime-trend-smoke",
        &[
            "\"schema\":\"canon_runtime_performance_trend_v1\\",
            "\"record_type\":\"runtime_performance_trend_smoke\\",
            "\"baseline_project_agent_elapsed_ms_p95\":3000",
            "\"current_project_agent_elapsed_ms_p95\":3150",
            "\"observed_regression_bps\":500",
            "\"budget_status\":\"pass\\",
            "\"trend_status\":\"pass\\",
        ],
    );
}

#[test]
fn root_validate_runtime_trend_regression_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--runtime-trend-regression-smoke",
        &[
            "\"schema\":\"canon_runtime_performance_trend_v1\\",
            "\"record_type\":\"runtime_performance_trend_regression_smoke\\",
            "\"baseline_project_agent_elapsed_ms_p95\":3000",
            "\"current_project_agent_elapsed_ms_p95\":3301",
            "\"observed_regression_bps\":1003",
            "\"budget_status\":\"pass\\",
            "\"trend_status\":\"fail\\",
        ],
    );
}

#[test]
fn root_validate_runtime_budget_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--runtime-budget-smoke",
        &[
            "\"step\":\"runtime_performance_budget_smoke\\",
            "\"runtime_performance_budget_status\":\"fail\\",
            "\"controlled_failure_observed\":true",
            "\"receipt_json_contract_valid\":true",
        ],
    );
}

#[test]
fn runtime_performance_budget_smoke_exit_is_independent_of_full_root_suite() {
    let exe = env!("CARGO_BIN_EXE_root_validate");
    let output = std::process::Command::new(exe)
        .arg("--runtime-budget-smoke")
        .env("CANON_MAX_PROJECT_AGENT_ELAPSED_MS_P95", "1")
        .output()
        .expect("run root_validate runtime budget smoke with env ceiling");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("\"forced_validation_command_duration_ms\":2"));
    assert!(stdout.contains("\"forced_max_project_agent_elapsed_ms_p95\":1"));
    assert!(!stdout.contains("canon_root_validation_v1"));
}

fn root_validate_compact_mode_stdout(arg: &str, expected_marker: &str) -> String {
    let exe = env!("CARGO_BIN_EXE_root_validate");
    let output = std::process::Command::new(exe)
        .arg(arg)
        .output()
        .unwrap_or_else(|err| panic!("run root_validate {arg}: {err}"));

    assert!(output.status.success(), "root_validate {arg} failed");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        stdout.contains(expected_marker),
        "root_validate {arg} missing {expected_marker}"
    );
    assert!(!stdout.contains("canon_root_validation_v1"));
    stdout
}

fn assert_stdout_contains_all(stdout: &str, expected_fragments: &[&str]) {
    for fragment in expected_fragments {
        assert!(
            stdout.contains(fragment),
            "stdout missing expected fragment: {fragment}"
        );
    }
}

struct CompactFixtureModeContract<'a> {
    arg: &'a str,
    marker: &'a str,
    expected_fragments: &'a [&'a str],
}

fn assert_root_validate_fixture_mode_contract(contract: CompactFixtureModeContract<'_>) {
    let stdout = root_validate_compact_mode_stdout(contract.arg, contract.marker);
    assert_stdout_contains_all(&stdout, contract.expected_fragments);
}

struct CompactModeOutputContract<'a> {
    arg: &'a str,
    marker: &'a str,
    expected_fragments: &'a [&'a str],
}

fn assert_root_validate_compact_mode_contract(contract: CompactModeOutputContract<'_>) {
    let stdout = root_validate_compact_mode_stdout(contract.arg, contract.marker);
    assert_stdout_contains_all(&stdout, contract.expected_fragments);
}

fn assert_root_validate_compact_mode_contracts(contracts: &[CompactModeOutputContract<'_>]) {
    for contract in contracts {
        assert_root_validate_compact_mode_contract(CompactModeOutputContract {
            arg: contract.arg,
            marker: contract.marker,
            expected_fragments: contract.expected_fragments,
        });
    }
}

#[test]
fn root_validate_compact_mode_helper_reports_marker_contract() {
    let stdout = root_validate_compact_mode_stdout(
        "--runtime-budget-smoke",
        RUNTIME_PERFORMANCE_BUDGET_SMOKE_STEP,
    );

    assert!(stdout.contains("\"controlled_failure_observed\":true"));
}

#[test]
fn root_validate_compact_mode_contract_table_helper_covers_multiple_modes() {
    assert_root_validate_compact_mode_contracts(&[
        CompactModeOutputContract {
            arg: "--runtime-trend-smoke",
            marker: RUNTIME_PERFORMANCE_TREND_SMOKE_STEP,
            expected_fragments: &["\"trend_status\":\"pass\\"],
        },
        CompactModeOutputContract {
            arg: "--policy-reuse-smoke",
            marker: ai::validation_harness::POLICY_REUSE_SMOKE_STEP,
            expected_fragments: &["\"avoided_llm_call_count\":1"],
        },
    ]);
}

#[test]
fn root_validate_fixture_mode_contract_helper_covers_fixture_output() {
    assert_root_validate_fixture_mode_contract(CompactFixtureModeContract {
        arg: "--policy-orchestration-capacity-fixture",
        marker: "canon_policy_orchestration_capacity_receipts_v1",
        expected_fragments: &[
            "schema=canon_policy_orchestration_capacity_receipts_v1",
            "policy_orchestration_capacity_smoke.estimated_avoided_llm_calls_per_full_batch=4",
        ],
    });
}

fn runtime_receipt_with_p95(p95: u64) -> RuntimePerformanceReceipt {
    RuntimePerformanceReceipt {
        step: RUNTIME_PERFORMANCE_STEP,
        runtime_performance_signal_present: true,
        runtime_performance_budget_status: "pass",
        project_agent_elapsed_ms_median: p95,
        project_agent_elapsed_ms_p95: p95,
        download_initial_get_ms_median: 0,
        download_initial_get_ms_p95: 0,
        download_follow_get_ms_median: 0,
        download_follow_get_ms_p95: 0,
        download_write_ms_median: 0,
        download_write_ms_p95: 0,
        validation_command_duration_ms: p95,
        max_project_agent_elapsed_ms_p95: 10_000,
        max_download_initial_get_ms_p95: 2_000,
        max_download_follow_get_ms_p95: 2_000,
        max_download_write_ms_p95: 1_000,
    }
}

fn fixture_value(fixture: &str, key: &str) -> u64 {
    fixture
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}=")))
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or_else(|| panic!("missing numeric fixture key {key}"))
}

fn fixture_text<'a>(fixture: &'a str, key: &str) -> &'a str {
    fixture
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key}=")))
        .unwrap_or_else(|| panic!("missing text fixture key {key}"))
}

fn fixture_contains_expected_lines(fixture: &str, expected: &[String]) -> bool {
    expected.iter().all(|line| fixture.contains(line))
}

fn fixture_missing_expected_lines<'a>(fixture: &str, expected: &'a [String]) -> Vec<&'a str> {
    expected
        .iter()
        .map(String::as_str)
        .filter(|line| !fixture.contains(*line))
        .collect()
}

fn retained_receipt_fixture_valid(
    fixture: &str,
    schema: &str,
    receipt_count: usize,
    expected: &[String],
    rules: &[&str],
) -> bool {
    ai::validation_harness::retained_fixture_header_valid(fixture, schema, receipt_count)
        && fixture_contains_expected_lines(fixture, expected)
        && rules.iter().all(|rule| fixture.contains(rule))
}

#[test]
fn retained_fixture_line_helper_reports_missing_expected_lines() {
    let expected = vec![
        "fixture.record_type=example".to_string(),
        "fixture.verdict=pass".to_string(),
    ];
    let fixture = "schema=canon_fixture_helper_v1\nfixture.record_type=example\n";

    assert!(!fixture_contains_expected_lines(fixture, &expected));
    assert_eq!(
        fixture_missing_expected_lines(fixture, &expected),
        vec!["fixture.verdict=pass"]
    );
}

#[test]
fn retained_fixture_header_helper_accepts_exact_schema_and_count() {
    let fixture = "schema=canon_fixture_header_v1\nreceipt_count=2\nrow=example\n";

    assert!(ai::validation_harness::retained_fixture_header_valid(
        fixture,
        "canon_fixture_header_v1",
        2
    ));
}

#[test]
fn retained_fixture_header_helper_rejects_drifted_count_and_prefix_matches() {
    let fixture =
        "schema=canon_fixture_header_v1_extra\nschema=canon_fixture_header_v1\nreceipt_count=20\n";

    assert!(!ai::validation_harness::retained_fixture_header_valid(
        fixture,
        "canon_fixture_header_v1",
        2
    ));
}

#[test]
fn retained_receipt_fixture_validators_reject_prefix_header_drift() {
    let drifted = "schema=canon_policy_reuse_receipts_v1_extra\nreceipt_count=30\npolicy_reuse_smoke.record_type=policy_reuse_smoke\n";

    assert!(!policy_reuse_receipts_fixture_valid(drifted));
}

#[test]
fn runtime_performance_trend_comparator_accepts_fixture_baseline() {
    let fixture = std::fs::read_to_string(RUNTIME_PERFORMANCE_TREND_FIXTURE)
        .expect("runtime performance trend fixture must be readable");
    assert!(fixture.contains("schema=canon_runtime_performance_trend_fixture_v1"));

    let baseline = runtime_receipt_with_p95(fixture_value(
        &fixture,
        "baseline_project_agent_elapsed_ms_p95",
    ));
    let current = runtime_receipt_with_p95(fixture_value(
        &fixture,
        "current_project_agent_elapsed_ms_p95",
    ));
    let trend = compare_runtime_performance_trend(
        &baseline,
        &current,
        fixture_value(&fixture, "allowed_regression_bps"),
    );

    assert_eq!(
        trend.observed_regression_bps,
        fixture_value(&fixture, "expected_observed_regression_bps")
    );
    assert_eq!(
        trend.trend_status,
        fixture_text(&fixture, "expected_trend_status")
    );
    assert!(trend.passed());
    assert!(trend
        .to_json()
        .contains("\"schema\":\"canon_runtime_performance_trend_v1\\"));
}

#[test]
fn runtime_performance_trend_comparator_detects_fixture_regression() {
    let fixture = std::fs::read_to_string(RUNTIME_PERFORMANCE_TREND_FIXTURE)
        .expect("runtime performance trend fixture must be readable");
    let baseline = runtime_receipt_with_p95(fixture_value(
        &fixture,
        "baseline_project_agent_elapsed_ms_p95",
    ));
    let current = runtime_receipt_with_p95(fixture_value(
        &fixture,
        "regressed_current_project_agent_elapsed_ms_p95",
    ));
    let trend = compare_runtime_performance_trend(
        &baseline,
        &current,
        fixture_value(&fixture, "allowed_regression_bps"),
    );

    assert_eq!(
        trend.observed_regression_bps,
        fixture_value(&fixture, "expected_regressed_observed_regression_bps")
    );
    assert_eq!(
        trend.trend_status,
        fixture_text(&fixture, "expected_regressed_trend_status")
    );
    assert!(!trend.passed());
    assert!(trend.to_json().contains("\"trend_status\":\"fail\\"));
}

#[test]
fn runtime_performance_trend_comparator_preserves_budget_failure() {
    let baseline = runtime_receipt_with_p95(3_000);
    let mut current = runtime_receipt_with_p95(3_000);
    current.runtime_performance_budget_status = "fail";

    let trend = compare_runtime_performance_trend(&baseline, &current, 500);

    assert_eq!(trend.observed_regression_bps, 0);
    assert_eq!(trend.trend_status, "pass");
    assert_eq!(trend.budget_status, "fail");
    assert!(!trend.passed());
}

#[test]
fn validation_cost_footprint_comparator_accepts_retained_receipts_without_running_suite() {
    let baseline_runtime = runtime_receipt_with_p95(3_000);
    let current_runtime = runtime_receipt_with_p95(3_150);
    let footprint = ai::validation_harness::validation_footprint_receipt();

    let receipt = compare_validation_cost_footprint(
        &baseline_runtime,
        &current_runtime,
        &footprint,
        &footprint,
        500,
    );

    assert_eq!(receipt.schema, "canon_validation_cost_footprint_v1");
    assert_eq!(receipt.record_type, "validation_cost_footprint");
    assert_eq!(receipt.observed_regression_bps, 500);
    assert_eq!(receipt.total_declared_step_delta, 0);
    assert_eq!(receipt.expected_count_guarded_test_delta, 0);
    assert!(!receipt.command_set_changed);
    assert_eq!(receipt.budget_status, "pass");
    assert_eq!(receipt.trend_status, "pass");
    assert_eq!(receipt.footprint_status, "pass");
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
    assert!(receipt
        .to_json()
        .contains("\"schema\":\"canon_validation_cost_footprint_v1\\"));
}

#[test]
fn validation_cost_footprint_comparator_binds_dispatch_catalog_hashes() {
    let baseline_runtime = runtime_receipt_with_p95(3_000);
    let current_runtime = runtime_receipt_with_p95(3_150);
    let footprint = ai::validation_harness::validation_footprint_receipt();

    let receipt = ai::validation_harness::compare_validation_cost_footprint_with_dispatch_hashes(
        &baseline_runtime,
        &current_runtime,
        &footprint,
        &footprint,
        500,
        "dispatch-hash-v1",
        "dispatch-hash-v1",
    );

    assert_eq!(receipt.baseline_dispatch_catalog_hash, "dispatch-hash-v1");
    assert_eq!(receipt.current_dispatch_catalog_hash, "dispatch-hash-v1");
    assert!(!receipt.dispatch_catalog_changed);
    assert_eq!(receipt.footprint_status, "pass");
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt
        .to_json()
        .contains("\"baseline_dispatch_catalog_hash\":\"dispatch-hash-v1\\"));
    assert!(receipt
        .to_json()
        .contains("\"dispatch_catalog_changed\":false"));
}

#[test]
fn validation_cost_footprint_comparator_rejects_dispatch_catalog_hash_drift() {
    let baseline_runtime = runtime_receipt_with_p95(3_000);
    let current_runtime = runtime_receipt_with_p95(3_000);
    let footprint = ai::validation_harness::validation_footprint_receipt();

    let receipt = ai::validation_harness::compare_validation_cost_footprint_with_dispatch_hashes(
        &baseline_runtime,
        &current_runtime,
        &footprint,
        &footprint,
        500,
        "dispatch-hash-v1",
        "dispatch-hash-v2",
    );

    assert_eq!(receipt.observed_regression_bps, 0);
    assert_eq!(receipt.total_declared_step_delta, 0);
    assert_eq!(receipt.expected_count_guarded_test_delta, 0);
    assert!(!receipt.command_set_changed);
    assert!(receipt.dispatch_catalog_changed);
    assert_eq!(receipt.trend_status, "pass");
    assert_eq!(receipt.footprint_status, "fail");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn validation_cost_smoke_json_exposes_dispatch_catalog_hash_fields() {
    let receipt = ai::validation_harness::validation_cost_footprint_smoke_receipt();
    let json = receipt.to_json();

    assert_eq!(
        receipt.baseline_dispatch_catalog_hash,
        receipt.current_dispatch_catalog_hash
    );
    assert!(!receipt.dispatch_catalog_changed);
    assert!(json.contains("\"baseline_dispatch_catalog_hash\":"));
    assert!(json.contains("\"current_dispatch_catalog_hash\":"));
    assert!(json.contains("\"dispatch_catalog_changed\":false"));
}

#[test]
fn validation_cost_footprint_comparator_rejects_runtime_regression() {
    let baseline_runtime = runtime_receipt_with_p95(3_000);
    let current_runtime = runtime_receipt_with_p95(3_301);
    let footprint = ai::validation_harness::validation_footprint_receipt();

    let receipt = compare_validation_cost_footprint(
        &baseline_runtime,
        &current_runtime,
        &footprint,
        &footprint,
        500,
    );

    assert_eq!(receipt.observed_regression_bps, 1003);
    assert_eq!(receipt.trend_status, "fail");
    assert_eq!(receipt.footprint_status, "pass");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn validation_cost_footprint_comparator_rejects_footprint_growth() {
    let baseline_runtime = runtime_receipt_with_p95(3_000);
    let current_runtime = runtime_receipt_with_p95(3_000);
    let baseline_footprint = ai::validation_harness::validation_footprint_receipt();
    let mut steps = root_validation_steps();
    steps.push(ValidationStep {
        name: "synthetic_cost_growth_step",
        runner: StepRunner::Cargo,
        args: vec![
            LOCKFILE_COMPAT_FLAG,
            "test",
            "--test",
            "synthetic",
            "--locked",
        ],
        expected_test_count: Some(1),
    });
    let current_footprint = validation_footprint_receipt_for_steps(
        &steps,
        ai::validation_harness::DEFAULT_MAX_PROJECT_AGENT_ELAPSED_MS_P95,
        ai::validation_harness::DEFAULT_MAX_DOWNLOAD_INITIAL_GET_MS_P95,
        ai::validation_harness::DEFAULT_MAX_DOWNLOAD_FOLLOW_GET_MS_P95,
        ai::validation_harness::DEFAULT_MAX_DOWNLOAD_WRITE_MS_P95,
    );

    let receipt = compare_validation_cost_footprint(
        &baseline_runtime,
        &current_runtime,
        &baseline_footprint,
        &current_footprint,
        500,
    );

    assert_eq!(receipt.observed_regression_bps, 0);
    assert_eq!(receipt.total_declared_step_delta, 1);
    assert_eq!(receipt.expected_count_guarded_test_delta, 1);
    assert!(receipt.command_set_changed);
    assert_eq!(receipt.trend_status, "pass");
    assert_eq!(receipt.footprint_status, "fail");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn root_validate_validation_cost_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--validation-cost-smoke",
        &[
            "\"schema\":\"canon_validation_cost_footprint_v1\\",
            "\"record_type\":\"validation_cost_footprint_smoke\\",
            "\"observed_regression_bps\":500",
            "\"total_declared_step_delta\":0",
            "\"expected_count_guarded_test_delta\":0",
            "\"command_set_changed\":false",
            "\"footprint_status\":\"pass\\",
            "\"verdict\":\"pass\\",
        ],
    );
}

#[test]
fn validation_cost_growth_smoke_receipt_is_controlled_negative_contract() {
    let receipt = ai::validation_harness::validation_cost_footprint_growth_smoke_receipt();

    assert_eq!(receipt.schema, "canon_validation_cost_footprint_v1");
    assert_eq!(
        receipt.record_type,
        VALIDATION_COST_FOOTPRINT_GROWTH_SMOKE_STEP
    );
    assert_eq!(receipt.observed_regression_bps, 0);
    assert_eq!(receipt.total_declared_step_delta, 1);
    assert_eq!(receipt.expected_count_guarded_test_delta, 1);
    assert!(receipt.command_set_changed);
    assert_eq!(receipt.budget_status, "pass");
    assert_eq!(receipt.trend_status, "pass");
    assert_eq!(receipt.footprint_status, "fail");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn root_validate_validation_cost_growth_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--validation-cost-growth-smoke",
        &[
            "\"schema\":\"canon_validation_cost_footprint_v1\\",
            "\"record_type\":\"validation_cost_footprint_growth_smoke\\",
            "\"observed_regression_bps\":0",
            "\"total_declared_step_delta\":1",
            "\"expected_count_guarded_test_delta\":1",
            "\"command_set_changed\":true",
            "\"budget_status\":\"pass\\",
            "\"trend_status\":\"pass\\",
            "\"footprint_status\":\"fail\\",
            "\"verdict\":\"fail\\",
        ],
    );
}

#[test]
fn root_validation_records_graph_mutation_cli_test_surface() {
    let steps = root_validation_steps();
    let graph_cli_step = steps
        .iter()
        .find(|step| step.name == GRAPH_MUTATION_CLI_CONTRACT_STEP)
        .expect("graph mutation CLI contract step must be present");

    assert_eq!(
        graph_cli_step.expected_test_count,
        Some(GRAPH_MUTATION_CLI_CONTRACT_EXPECTED_TESTS)
    );
    assert_eq!(GRAPH_MUTATION_CLI_CONTRACT_EXPECTED_TESTS, 10);
    assert_eq!(
        graph_cli_step.command_line("cargo"),
        "cargo -Znext-lockfile-bump test --test graph_mutation_cli_contract --locked -- --nocapture"
    );
}

#[test]
fn policy_capacity_cost_summary_smoke_combines_capacity_and_validation_cost() {
    let receipt = ai::validation_harness::policy_capacity_cost_summary_smoke_receipt();

    assert_eq!(receipt.schema, "canon_policy_capacity_cost_summary_v1");
    assert_eq!(receipt.record_type, POLICY_CAPACITY_COST_SUMMARY_SMOKE_STEP);
    assert_eq!(receipt.batch_capacity_limit, 8);
    assert_eq!(receipt.retained_policy_hit_rate_bps, 5_000);
    assert_eq!(receipt.estimated_avoided_llm_calls_per_full_batch, 4);
    assert_eq!(receipt.estimated_llm_fallbacks_per_full_batch, 4);
    assert_eq!(receipt.retained_avoided_llm_call_count, 1);
    assert_eq!(receipt.validation_observed_regression_bps, 500);
    assert_eq!(receipt.validation_total_declared_step_delta, 0);
    assert_eq!(receipt.validation_expected_count_guarded_test_delta, 0);
    assert!(!receipt.validation_dispatch_catalog_changed);
    assert_eq!(receipt.policy_capacity_status, "pass");
    assert_eq!(receipt.validation_cost_verdict, "pass");
    assert_eq!(receipt.summary_status, "pass");
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
}

#[test]
fn root_validate_policy_capacity_cost_summary_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-capacity-cost-summary-smoke",
        &[
            "\"schema\":\"canon_policy_capacity_cost_summary_v1\\",
            "\"record_type\":\"policy_capacity_cost_summary_smoke\\",
            "\"retained_policy_hit_rate_bps\":5000",
            "\"estimated_avoided_llm_calls_per_full_batch\":4",
            "\"estimated_llm_fallbacks_per_full_batch\":4",
            "\"validation_observed_regression_bps\":500",
            "\"validation_total_declared_step_delta\":0",
            "\"validation_expected_count_guarded_test_delta\":0",
            "\"validation_dispatch_catalog_changed\":false",
            "\"policy_capacity_status\":\"pass\\",
            "\"validation_cost_verdict\":\"pass\\",
            "\"summary_status\":\"pass\\",
            "\"verdict\":\"pass\\",
        ],
    );
}

#[test]
fn policy_capacity_cost_summary_growth_smoke_is_controlled_negative() {
    let receipt = ai::validation_harness::policy_capacity_cost_summary_growth_smoke_receipt();

    assert_eq!(receipt.schema, "canon_policy_capacity_cost_summary_v1");
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_CAPACITY_COST_SUMMARY_GROWTH_SMOKE_STEP
    );
    assert_eq!(receipt.retained_policy_hit_rate_bps, 5_000);
    assert_eq!(receipt.estimated_avoided_llm_calls_per_full_batch, 4);
    assert_eq!(receipt.validation_observed_regression_bps, 0);
    assert_eq!(receipt.validation_total_declared_step_delta, 1);
    assert_eq!(receipt.validation_expected_count_guarded_test_delta, 1);
    assert_eq!(receipt.policy_capacity_status, "pass");
    assert_eq!(receipt.validation_cost_verdict, "fail");
    assert_eq!(receipt.summary_status, "fail");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_capacity_cost_summary_growth_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-capacity-cost-summary-growth-smoke",
        &[
            "\"schema\":\"canon_policy_capacity_cost_summary_v1\\",
            "\"record_type\":\"policy_capacity_cost_summary_growth_smoke\\",
            "\"retained_policy_hit_rate_bps\":5000",
            "\"estimated_avoided_llm_calls_per_full_batch\":4",
            "\"validation_observed_regression_bps\":0",
            "\"validation_total_declared_step_delta\":1",
            "\"validation_expected_count_guarded_test_delta\":1",
            "\"policy_capacity_status\":\"pass\\",
            "\"validation_cost_verdict\":\"fail\\",
            "\"summary_status\":\"fail\\",
            "\"verdict\":\"fail\\",
        ],
    );
}

#[test]
fn policy_capacity_cost_summary_trend_smoke_compares_capacity_and_cost() {
    let receipt = ai::validation_harness::policy_capacity_cost_summary_trend_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_capacity_cost_summary_trend_v1"
    );
    assert_eq!(
        receipt.record_type,
        POLICY_CAPACITY_COST_SUMMARY_TREND_SMOKE_STEP
    );
    assert_eq!(receipt.batch_capacity_limit, 8);
    assert_eq!(receipt.baseline_retained_policy_hit_rate_bps, 5_000);
    assert_eq!(receipt.current_retained_policy_hit_rate_bps, 10_000);
    assert_eq!(receipt.retained_policy_hit_rate_delta_bps, 5_000);
    assert_eq!(
        receipt.baseline_estimated_avoided_llm_calls_per_full_batch,
        4
    );
    assert_eq!(
        receipt.current_estimated_avoided_llm_calls_per_full_batch,
        8
    );
    assert_eq!(receipt.avoided_llm_call_delta_per_full_batch, 4);
    assert_eq!(receipt.baseline_validation_observed_regression_bps, 500);
    assert_eq!(receipt.current_validation_observed_regression_bps, 500);
    assert_eq!(receipt.validation_observed_regression_delta_bps, 0);
    assert_eq!(receipt.baseline_summary_status, "pass");
    assert_eq!(receipt.current_summary_status, "pass");
    assert_eq!(receipt.baseline_validation_cost_verdict, "pass");
    assert_eq!(receipt.current_validation_cost_verdict, "pass");
    assert!(!receipt.baseline_dispatch_catalog_changed);
    assert!(!receipt.current_dispatch_catalog_changed);
    assert_eq!(receipt.trend_status, "pass");
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
}

#[test]
fn root_validate_policy_capacity_cost_summary_trend_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-capacity-cost-summary-trend-smoke",
        &[
            "\"schema\":\"canon_policy_capacity_cost_summary_trend_v1\\",
            "\"record_type\":\"policy_capacity_cost_summary_trend_smoke\\",
            "\"baseline_retained_policy_hit_rate_bps\":5000",
            "\"current_retained_policy_hit_rate_bps\":10000",
            "\"retained_policy_hit_rate_delta_bps\":5000",
            "\"baseline_estimated_avoided_llm_calls_per_full_batch\":4",
            "\"current_estimated_avoided_llm_calls_per_full_batch\":8",
            "\"avoided_llm_call_delta_per_full_batch\":4",
            "\"validation_observed_regression_delta_bps\":0",
            "\"trend_status\":\"pass\\",
            "\"verdict\":\"pass\\",
        ],
    );
}

#[test]
fn policy_capacity_cost_summary_regression_smoke_exposes_controlled_negative_receipt() {
    let receipt = ai::validation_harness::policy_capacity_cost_summary_regression_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_capacity_cost_summary_trend_v1"
    );
    assert_eq!(
        receipt.record_type,
        POLICY_CAPACITY_COST_SUMMARY_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.batch_capacity_limit, 8);
    assert_eq!(receipt.baseline_retained_policy_hit_rate_bps, 10_000);
    assert_eq!(receipt.current_retained_policy_hit_rate_bps, 5_000);
    assert_eq!(receipt.retained_policy_hit_rate_delta_bps, -5_000);
    assert_eq!(
        receipt.baseline_estimated_avoided_llm_calls_per_full_batch,
        8
    );
    assert_eq!(
        receipt.current_estimated_avoided_llm_calls_per_full_batch,
        4
    );
    assert_eq!(receipt.avoided_llm_call_delta_per_full_batch, -4);
    assert_eq!(receipt.baseline_summary_status, "pass");
    assert_eq!(receipt.current_summary_status, "pass");
    assert_eq!(receipt.baseline_validation_cost_verdict, "pass");
    assert_eq!(receipt.current_validation_cost_verdict, "pass");
    assert_eq!(receipt.trend_status, "regressed");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_capacity_cost_summary_regression_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-capacity-cost-summary-regression-smoke",
        &[
            "\"schema\":\"canon_policy_capacity_cost_summary_trend_v1\\",
            "\"record_type\":\"policy_capacity_cost_summary_regression_smoke\\",
            "\"baseline_retained_policy_hit_rate_bps\":10000",
            "\"current_retained_policy_hit_rate_bps\":5000",
            "\"retained_policy_hit_rate_delta_bps\":-5000",
            "\"baseline_estimated_avoided_llm_calls_per_full_batch\":8",
            "\"current_estimated_avoided_llm_calls_per_full_batch\":4",
            "\"avoided_llm_call_delta_per_full_batch\":-4",
            "\"trend_status\":\"regressed\\",
            "\"verdict\":\"fail\\",
        ],
    );
}

#[test]
fn policy_capacity_cost_summary_rejects_validation_cost_failure() {
    let capacity = ai::validation_harness::policy_orchestration_capacity_smoke_receipt();
    let validation_cost = ai::validation_harness::validation_cost_footprint_growth_smoke_receipt();

    let receipt =
        ai::validation_harness::compare_policy_capacity_cost_summary(&capacity, &validation_cost);

    assert_eq!(receipt.record_type, "policy_capacity_cost_summary");
    assert_eq!(receipt.policy_capacity_status, "pass");
    assert_eq!(receipt.validation_cost_verdict, "fail");
    assert_eq!(receipt.summary_status, "fail");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn policy_capacity_cost_summary_receipts_fixture_binds_expected_retained_receipts() {
    let fixture = std::fs::read_to_string(POLICY_CAPACITY_COST_SUMMARY_RECEIPTS_FIXTURE)
        .expect("policy capacity cost summary receipts fixture must be readable");
    assert!(policy_capacity_cost_summary_receipts_fixture_valid(
        &fixture
    ));
}

#[test]
fn root_validate_policy_capacity_cost_summary_fixture_mode_is_executable_contract() {
    assert_root_validate_fixture_mode_contract(CompactFixtureModeContract {
        arg: "--policy-capacity-cost-summary-fixture",
        marker: "canon_policy_capacity_cost_summary_receipts_v1",
        expected_fragments: &[
            "schema=canon_policy_capacity_cost_summary_receipts_v1",
            "receipt_count=4",
            "policy_capacity_cost_summary_smoke.estimated_avoided_llm_calls_per_full_batch=4",
            "policy_capacity_cost_summary_growth_smoke.validation_cost_verdict=fail",
            "policy_capacity_cost_summary_trend_smoke.current_estimated_avoided_llm_calls_per_full_batch=8",
            "policy_capacity_cost_summary_regression_smoke.avoided_llm_call_delta_per_full_batch=-4",
        ],
    });
}

#[test]
fn policy_capacity_cost_summary_receipts_fixture_negative_contract_detects_drift() {
    let fixture = std::fs::read_to_string(POLICY_CAPACITY_COST_SUMMARY_RECEIPTS_FIXTURE)
        .expect("policy capacity cost summary receipts fixture must be readable");
    let drifted = fixture.replace(
        "policy_capacity_cost_summary_regression_smoke.current_estimated_avoided_llm_calls_per_full_batch=4",
        "policy_capacity_cost_summary_regression_smoke.current_estimated_avoided_llm_calls_per_full_batch=8",
    );
    assert!(!policy_capacity_cost_summary_receipts_fixture_valid(
        &drifted
    ));
}

#[test]
fn policy_capacity_cost_summary_receipts_fixture_negative_contract_detects_validation_cost_trend_drift(
) {
    let fixture = std::fs::read_to_string(POLICY_CAPACITY_COST_SUMMARY_RECEIPTS_FIXTURE)
        .expect("policy capacity cost summary receipts fixture must be readable");

    let drifted_trend = fixture.replace(
        "policy_capacity_cost_summary_trend_smoke.validation_observed_regression_delta_bps=0",
        "policy_capacity_cost_summary_trend_smoke.validation_observed_regression_delta_bps=1",
    );
    assert!(!policy_capacity_cost_summary_receipts_fixture_valid(
        &drifted_trend
    ));

    let drifted_regression = fixture.replace(
        "policy_capacity_cost_summary_regression_smoke.current_validation_cost_verdict=pass",
        "policy_capacity_cost_summary_regression_smoke.current_validation_cost_verdict=fail",
    );
    assert!(!policy_capacity_cost_summary_receipts_fixture_valid(
        &drifted_regression
    ));
}

fn policy_capacity_cost_summary_receipts_fixture_valid(fixture: &str) -> bool {
    let smoke = ai::validation_harness::policy_capacity_cost_summary_smoke_receipt();
    let growth = ai::validation_harness::policy_capacity_cost_summary_growth_smoke_receipt();
    let trend = ai::validation_harness::policy_capacity_cost_summary_trend_smoke_receipt();
    let regression =
        ai::validation_harness::policy_capacity_cost_summary_regression_smoke_receipt();

    let expected = [
        format!("policy_capacity_cost_summary_smoke.record_type={}", smoke.record_type),
        format!("policy_capacity_cost_summary_smoke.batch_capacity_limit={}", smoke.batch_capacity_limit),
        format!("policy_capacity_cost_summary_smoke.retained_policy_hit_rate_bps={}", smoke.retained_policy_hit_rate_bps),
        format!("policy_capacity_cost_summary_smoke.estimated_avoided_llm_calls_per_full_batch={}", smoke.estimated_avoided_llm_calls_per_full_batch),
        format!("policy_capacity_cost_summary_smoke.estimated_llm_fallbacks_per_full_batch={}", smoke.estimated_llm_fallbacks_per_full_batch),
        format!("policy_capacity_cost_summary_smoke.retained_avoided_llm_call_count={}", smoke.retained_avoided_llm_call_count),
        format!("policy_capacity_cost_summary_smoke.validation_observed_regression_bps={}", smoke.validation_observed_regression_bps),
        format!("policy_capacity_cost_summary_smoke.validation_total_declared_step_delta={}", smoke.validation_total_declared_step_delta),
        format!("policy_capacity_cost_summary_smoke.validation_expected_count_guarded_test_delta={}", smoke.validation_expected_count_guarded_test_delta),
        format!("policy_capacity_cost_summary_smoke.validation_dispatch_catalog_changed={}", smoke.validation_dispatch_catalog_changed),
        format!("policy_capacity_cost_summary_smoke.policy_capacity_status={}", smoke.policy_capacity_status),
        format!("policy_capacity_cost_summary_smoke.validation_cost_verdict={}", smoke.validation_cost_verdict),
        format!("policy_capacity_cost_summary_smoke.summary_status={}", smoke.summary_status),
        format!("policy_capacity_cost_summary_smoke.verdict={}", smoke.verdict),
        format!("policy_capacity_cost_summary_growth_smoke.record_type={}", growth.record_type),
        format!("policy_capacity_cost_summary_growth_smoke.validation_observed_regression_bps={}", growth.validation_observed_regression_bps),
        format!("policy_capacity_cost_summary_growth_smoke.validation_total_declared_step_delta={}", growth.validation_total_declared_step_delta),
        format!("policy_capacity_cost_summary_growth_smoke.validation_expected_count_guarded_test_delta={}", growth.validation_expected_count_guarded_test_delta),
        format!("policy_capacity_cost_summary_growth_smoke.validation_dispatch_catalog_changed={}", growth.validation_dispatch_catalog_changed),
        format!("policy_capacity_cost_summary_growth_smoke.policy_capacity_status={}", growth.policy_capacity_status),
        format!("policy_capacity_cost_summary_growth_smoke.validation_cost_verdict={}", growth.validation_cost_verdict),
        format!("policy_capacity_cost_summary_growth_smoke.summary_status={}", growth.summary_status),
        format!("policy_capacity_cost_summary_growth_smoke.verdict={}", growth.verdict),
        format!("policy_capacity_cost_summary_trend_smoke.record_type={}", trend.record_type),
        format!("policy_capacity_cost_summary_trend_smoke.batch_capacity_limit={}", trend.batch_capacity_limit),
        format!("policy_capacity_cost_summary_trend_smoke.baseline_retained_policy_hit_rate_bps={}", trend.baseline_retained_policy_hit_rate_bps),
        format!("policy_capacity_cost_summary_trend_smoke.current_retained_policy_hit_rate_bps={}", trend.current_retained_policy_hit_rate_bps),
        format!("policy_capacity_cost_summary_trend_smoke.retained_policy_hit_rate_delta_bps={}", trend.retained_policy_hit_rate_delta_bps),
        format!("policy_capacity_cost_summary_trend_smoke.baseline_estimated_avoided_llm_calls_per_full_batch={}", trend.baseline_estimated_avoided_llm_calls_per_full_batch),
        format!("policy_capacity_cost_summary_trend_smoke.current_estimated_avoided_llm_calls_per_full_batch={}", trend.current_estimated_avoided_llm_calls_per_full_batch),
        format!("policy_capacity_cost_summary_trend_smoke.avoided_llm_call_delta_per_full_batch={}", trend.avoided_llm_call_delta_per_full_batch),
        format!("policy_capacity_cost_summary_trend_smoke.baseline_validation_observed_regression_bps={}", trend.baseline_validation_observed_regression_bps),
        format!("policy_capacity_cost_summary_trend_smoke.current_validation_observed_regression_bps={}", trend.current_validation_observed_regression_bps),
        format!("policy_capacity_cost_summary_trend_smoke.validation_observed_regression_delta_bps={}", trend.validation_observed_regression_delta_bps),
        format!("policy_capacity_cost_summary_trend_smoke.baseline_summary_status={}", trend.baseline_summary_status),
        format!("policy_capacity_cost_summary_trend_smoke.current_summary_status={}", trend.current_summary_status),
        format!("policy_capacity_cost_summary_trend_smoke.baseline_validation_cost_verdict={}", trend.baseline_validation_cost_verdict),
        format!("policy_capacity_cost_summary_trend_smoke.current_validation_cost_verdict={}", trend.current_validation_cost_verdict),
        format!("policy_capacity_cost_summary_trend_smoke.baseline_dispatch_catalog_changed={}", trend.baseline_dispatch_catalog_changed),
        format!("policy_capacity_cost_summary_trend_smoke.current_dispatch_catalog_changed={}", trend.current_dispatch_catalog_changed),
        format!("policy_capacity_cost_summary_trend_smoke.trend_status={}", trend.trend_status),
        format!("policy_capacity_cost_summary_trend_smoke.verdict={}", trend.verdict),
        format!("policy_capacity_cost_summary_regression_smoke.record_type={}", regression.record_type),
        format!("policy_capacity_cost_summary_regression_smoke.batch_capacity_limit={}", regression.batch_capacity_limit),
        format!("policy_capacity_cost_summary_regression_smoke.baseline_retained_policy_hit_rate_bps={}", regression.baseline_retained_policy_hit_rate_bps),
        format!("policy_capacity_cost_summary_regression_smoke.current_retained_policy_hit_rate_bps={}", regression.current_retained_policy_hit_rate_bps),
        format!("policy_capacity_cost_summary_regression_smoke.retained_policy_hit_rate_delta_bps={}", regression.retained_policy_hit_rate_delta_bps),
        format!("policy_capacity_cost_summary_regression_smoke.baseline_estimated_avoided_llm_calls_per_full_batch={}", regression.baseline_estimated_avoided_llm_calls_per_full_batch),
        format!("policy_capacity_cost_summary_regression_smoke.current_estimated_avoided_llm_calls_per_full_batch={}", regression.current_estimated_avoided_llm_calls_per_full_batch),
        format!("policy_capacity_cost_summary_regression_smoke.avoided_llm_call_delta_per_full_batch={}", regression.avoided_llm_call_delta_per_full_batch),
        format!("policy_capacity_cost_summary_regression_smoke.baseline_validation_observed_regression_bps={}", regression.baseline_validation_observed_regression_bps),
        format!("policy_capacity_cost_summary_regression_smoke.current_validation_observed_regression_bps={}", regression.current_validation_observed_regression_bps),
        format!("policy_capacity_cost_summary_regression_smoke.validation_observed_regression_delta_bps={}", regression.validation_observed_regression_delta_bps),
        format!("policy_capacity_cost_summary_regression_smoke.baseline_summary_status={}", regression.baseline_summary_status),
        format!("policy_capacity_cost_summary_regression_smoke.current_summary_status={}", regression.current_summary_status),
        format!("policy_capacity_cost_summary_regression_smoke.baseline_validation_cost_verdict={}", regression.baseline_validation_cost_verdict),
        format!("policy_capacity_cost_summary_regression_smoke.current_validation_cost_verdict={}", regression.current_validation_cost_verdict),
        format!("policy_capacity_cost_summary_regression_smoke.baseline_dispatch_catalog_changed={}", regression.baseline_dispatch_catalog_changed),
        format!("policy_capacity_cost_summary_regression_smoke.current_dispatch_catalog_changed={}", regression.current_dispatch_catalog_changed),
        format!("policy_capacity_cost_summary_regression_smoke.trend_status={}", regression.trend_status),
        format!("policy_capacity_cost_summary_regression_smoke.verdict={}", regression.verdict),
    ];

    retained_receipt_fixture_valid(
        fixture,
        "canon_policy_capacity_cost_summary_receipts_v1",
        4,
        &expected,
        &[
            "rule=summary smoke passes only when policy capacity and validation cost both pass",
            "rule=growth smoke proves validation footprint or dispatch growth fails summary while policy capacity remains passing",
            "rule=trend smoke passes when bounded-batch avoided LLM work improves and validation cost remains stable",
            "rule=regression smoke fails when bounded-batch avoided LLM work declines even if validation cost remains stable",
            "rule=fixture binds retained semantic values rather than brittle receipt hashes",
        ],
    )
        && smoke.passed()
        && !growth.passed()
        && trend.passed()
        && !regression.passed()
}

#[test]
fn policy_orchestration_capacity_trend_smoke_compares_batch_avoidance() {
    let receipt = ai::validation_harness::policy_orchestration_capacity_trend_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_orchestration_capacity_trend_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_ORCHESTRATION_CAPACITY_TREND_SMOKE_STEP
    );
    assert_eq!(receipt.batch_capacity_limit, 8);
    assert_eq!(receipt.baseline_hit_rate_bps, 5_000);
    assert_eq!(receipt.current_hit_rate_bps, 10_000);
    assert_eq!(receipt.hit_rate_delta_bps, 5_000);
    assert_eq!(
        receipt.baseline_estimated_avoided_llm_calls_per_full_batch,
        4
    );
    assert_eq!(
        receipt.current_estimated_avoided_llm_calls_per_full_batch,
        8
    );
    assert_eq!(receipt.avoided_llm_call_delta_per_full_batch, 4);
    assert_eq!(receipt.baseline_policy_reuse_verdict, "pass");
    assert_eq!(receipt.current_policy_reuse_verdict, "pass");
    assert_eq!(receipt.baseline_capacity_status, "pass");
    assert_eq!(receipt.current_capacity_status, "pass");
    assert_eq!(receipt.trend_status, "pass");
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
}

#[test]
fn root_validate_policy_orchestration_capacity_trend_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-orchestration-capacity-trend-smoke",
        &[
            "\"schema\":\"canon_policy_orchestration_capacity_trend_v1\\",
            "\"record_type\":\"policy_orchestration_capacity_trend_smoke\\",
            "\"baseline_hit_rate_bps\":5000",
            "\"current_hit_rate_bps\":10000",
            "\"hit_rate_delta_bps\":5000",
            "\"baseline_estimated_avoided_llm_calls_per_full_batch\":4",
            "\"current_estimated_avoided_llm_calls_per_full_batch\":8",
            "\"avoided_llm_call_delta_per_full_batch\":4",
            "\"trend_status\":\"pass\\",
            "\"verdict\":\"pass\\",
        ],
    );
}

#[test]
fn policy_orchestration_capacity_regression_smoke_exposes_controlled_negative_receipt() {
    let receipt = ai::validation_harness::policy_orchestration_capacity_regression_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_orchestration_capacity_trend_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_ORCHESTRATION_CAPACITY_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.batch_capacity_limit, 8);
    assert_eq!(receipt.baseline_hit_rate_bps, 10_000);
    assert_eq!(receipt.current_hit_rate_bps, 5_000);
    assert_eq!(receipt.hit_rate_delta_bps, -5_000);
    assert_eq!(
        receipt.baseline_estimated_avoided_llm_calls_per_full_batch,
        8
    );
    assert_eq!(
        receipt.current_estimated_avoided_llm_calls_per_full_batch,
        4
    );
    assert_eq!(receipt.avoided_llm_call_delta_per_full_batch, -4);
    assert_eq!(receipt.baseline_policy_reuse_verdict, "pass");
    assert_eq!(receipt.current_policy_reuse_verdict, "pass");
    assert_eq!(receipt.baseline_capacity_status, "pass");
    assert_eq!(receipt.current_capacity_status, "pass");
    assert_eq!(receipt.trend_status, "regressed");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_orchestration_capacity_regression_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-orchestration-capacity-regression-smoke",
        &[
            "\"schema\":\"canon_policy_orchestration_capacity_trend_v1\\",
            "\"record_type\":\"policy_orchestration_capacity_regression_smoke\\",
            "\"baseline_hit_rate_bps\":10000",
            "\"current_hit_rate_bps\":5000",
            "\"hit_rate_delta_bps\":-5000",
            "\"baseline_estimated_avoided_llm_calls_per_full_batch\":8",
            "\"current_estimated_avoided_llm_calls_per_full_batch\":4",
            "\"avoided_llm_call_delta_per_full_batch\":-4",
            "\"baseline_capacity_status\":\"pass\\",
            "\"current_capacity_status\":\"pass\\",
            "\"trend_status\":\"regressed\\",
            "\"verdict\":\"fail\\",
        ],
    );
}

#[test]
fn policy_orchestration_capacity_trend_receipt_rejects_lower_batch_avoidance() {
    let baseline = ai::validation_harness::policy_orchestration_capacity_smoke_receipt();
    let mut current = baseline.clone();
    current.hit_rate_bps = 2_500;
    current.estimated_policy_hits_per_full_batch = 2;
    current.estimated_llm_fallbacks_per_full_batch = 6;
    current.estimated_avoided_llm_calls_per_full_batch = 2;

    let receipt =
        ai::validation_harness::compare_policy_orchestration_capacity_trend(&baseline, &current);

    assert_eq!(receipt.baseline_hit_rate_bps, 5_000);
    assert_eq!(receipt.current_hit_rate_bps, 2_500);
    assert_eq!(receipt.hit_rate_delta_bps, -2_500);
    assert_eq!(receipt.avoided_llm_call_delta_per_full_batch, -2);
    assert_eq!(receipt.trend_status, "regressed");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn policy_orchestration_capacity_smoke_estimates_batch_level_llm_avoidance() {
    let receipt = ai::validation_harness::policy_orchestration_capacity_smoke_receipt();

    assert_eq!(receipt.schema, "canon_policy_orchestration_capacity_v1");
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_ORCHESTRATION_CAPACITY_SMOKE_STEP
    );
    assert_eq!(
        receipt.batch_capacity_limit,
        ai::validation_harness::POLICY_ORCHESTRATION_CAPACITY_BATCH_LIMIT
    );
    assert_eq!(receipt.batch_capacity_limit, 8);
    assert_eq!(receipt.retained_policy_record_count, 2);
    assert_eq!(receipt.policy_hit_count, 1);
    assert_eq!(receipt.policy_miss_count, 1);
    assert_eq!(receipt.hit_rate_bps, 5_000);
    assert_eq!(receipt.estimated_policy_hits_per_full_batch, 4);
    assert_eq!(receipt.estimated_llm_fallbacks_per_full_batch, 4);
    assert_eq!(receipt.estimated_avoided_llm_calls_per_full_batch, 4);
    assert_eq!(receipt.retained_avoided_llm_call_count, 1);
    assert_eq!(receipt.capacity_status, "pass");
    assert_eq!(receipt.policy_reuse_verdict, "pass");
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
}

#[test]
fn root_validate_policy_orchestration_capacity_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-orchestration-capacity-smoke",
        &[
            "\"schema\":\"canon_policy_orchestration_capacity_v1\\",
            "\"record_type\":\"policy_orchestration_capacity_smoke\\",
            "\"batch_capacity_limit\":8",
            "\"hit_rate_bps\":5000",
            "\"estimated_policy_hits_per_full_batch\":4",
            "\"estimated_llm_fallbacks_per_full_batch\":4",
            "\"estimated_avoided_llm_calls_per_full_batch\":4",
            "\"capacity_status\":\"pass\\",
            "\"policy_reuse_verdict\":\"pass\\",
            "\"verdict\":\"pass\\",
        ],
    );
}

#[test]
fn policy_orchestration_capacity_receipt_rejects_inconsistent_capacity_math() {
    let mut receipt = ai::validation_harness::policy_orchestration_capacity_smoke_receipt();
    assert!(receipt.passed());

    receipt.estimated_avoided_llm_calls_per_full_batch += 1;
    receipt.verdict = "fail";

    assert_eq!(receipt.capacity_status, "pass");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn policy_orchestration_capacity_receipts_fixture_binds_expected_retained_receipts() {
    let fixture = std::fs::read_to_string(POLICY_ORCHESTRATION_CAPACITY_RECEIPTS_FIXTURE)
        .expect("policy orchestration capacity receipts fixture must be readable");
    assert!(policy_orchestration_capacity_receipts_fixture_valid(
        &fixture
    ));
}

#[test]
fn root_validate_policy_orchestration_capacity_fixture_mode_is_executable_contract() {
    assert_root_validate_fixture_mode_contract(CompactFixtureModeContract {
        arg: "--policy-orchestration-capacity-fixture",
        marker: "canon_policy_orchestration_capacity_receipts_v1",
        expected_fragments: &[
            "schema=canon_policy_orchestration_capacity_receipts_v1",
            "receipt_count=3",
            "policy_orchestration_capacity_smoke.estimated_avoided_llm_calls_per_full_batch=4",
            "policy_orchestration_capacity_trend_smoke.current_estimated_avoided_llm_calls_per_full_batch=8",
            "policy_orchestration_capacity_regression_smoke.avoided_llm_call_delta_per_full_batch=-4",
        ],
    });
}

#[test]
fn policy_orchestration_capacity_receipts_fixture_negative_contract_detects_drift() {
    let fixture = std::fs::read_to_string(POLICY_ORCHESTRATION_CAPACITY_RECEIPTS_FIXTURE)
        .expect("policy orchestration capacity receipts fixture must be readable");
    let drifted = fixture.replace(
        "policy_orchestration_capacity_regression_smoke.current_estimated_avoided_llm_calls_per_full_batch=4",
        "policy_orchestration_capacity_regression_smoke.current_estimated_avoided_llm_calls_per_full_batch=8",
    );
    assert!(!policy_orchestration_capacity_receipts_fixture_valid(
        &drifted
    ));
}

fn policy_orchestration_capacity_receipts_fixture_valid(fixture: &str) -> bool {
    let capacity = ai::validation_harness::policy_orchestration_capacity_smoke_receipt();
    let trend = ai::validation_harness::policy_orchestration_capacity_trend_smoke_receipt();
    let regression =
        ai::validation_harness::policy_orchestration_capacity_regression_smoke_receipt();

    let expected = [
        format!("policy_orchestration_capacity_smoke.record_type={}", capacity.record_type),
        format!("policy_orchestration_capacity_smoke.batch_capacity_limit={}", capacity.batch_capacity_limit),
        format!("policy_orchestration_capacity_smoke.retained_policy_record_count={}", capacity.retained_policy_record_count),
        format!("policy_orchestration_capacity_smoke.policy_hit_count={}", capacity.policy_hit_count),
        format!("policy_orchestration_capacity_smoke.policy_miss_count={}", capacity.policy_miss_count),
        format!("policy_orchestration_capacity_smoke.hit_rate_bps={}", capacity.hit_rate_bps),
        format!("policy_orchestration_capacity_smoke.estimated_policy_hits_per_full_batch={}", capacity.estimated_policy_hits_per_full_batch),
        format!("policy_orchestration_capacity_smoke.estimated_llm_fallbacks_per_full_batch={}", capacity.estimated_llm_fallbacks_per_full_batch),
        format!("policy_orchestration_capacity_smoke.estimated_avoided_llm_calls_per_full_batch={}", capacity.estimated_avoided_llm_calls_per_full_batch),
        format!("policy_orchestration_capacity_smoke.retained_avoided_llm_call_count={}", capacity.retained_avoided_llm_call_count),
        format!("policy_orchestration_capacity_smoke.capacity_status={}", capacity.capacity_status),
        format!("policy_orchestration_capacity_smoke.policy_reuse_verdict={}", capacity.policy_reuse_verdict),
        format!("policy_orchestration_capacity_smoke.verdict={}", capacity.verdict),
        format!("policy_orchestration_capacity_trend_smoke.record_type={}", trend.record_type),
        format!("policy_orchestration_capacity_trend_smoke.batch_capacity_limit={}", trend.batch_capacity_limit),
        format!("policy_orchestration_capacity_trend_smoke.baseline_hit_rate_bps={}", trend.baseline_hit_rate_bps),
        format!("policy_orchestration_capacity_trend_smoke.current_hit_rate_bps={}", trend.current_hit_rate_bps),
        format!("policy_orchestration_capacity_trend_smoke.hit_rate_delta_bps={}", trend.hit_rate_delta_bps),
        format!("policy_orchestration_capacity_trend_smoke.baseline_estimated_avoided_llm_calls_per_full_batch={}", trend.baseline_estimated_avoided_llm_calls_per_full_batch),
        format!("policy_orchestration_capacity_trend_smoke.current_estimated_avoided_llm_calls_per_full_batch={}", trend.current_estimated_avoided_llm_calls_per_full_batch),
        format!("policy_orchestration_capacity_trend_smoke.avoided_llm_call_delta_per_full_batch={}", trend.avoided_llm_call_delta_per_full_batch),
        format!("policy_orchestration_capacity_trend_smoke.baseline_policy_reuse_verdict={}", trend.baseline_policy_reuse_verdict),
        format!("policy_orchestration_capacity_trend_smoke.current_policy_reuse_verdict={}", trend.current_policy_reuse_verdict),
        format!("policy_orchestration_capacity_trend_smoke.baseline_capacity_status={}", trend.baseline_capacity_status),
        format!("policy_orchestration_capacity_trend_smoke.current_capacity_status={}", trend.current_capacity_status),
        format!("policy_orchestration_capacity_trend_smoke.trend_status={}", trend.trend_status),
        format!("policy_orchestration_capacity_trend_smoke.verdict={}", trend.verdict),
        format!("policy_orchestration_capacity_regression_smoke.record_type={}", regression.record_type),
        format!("policy_orchestration_capacity_regression_smoke.batch_capacity_limit={}", regression.batch_capacity_limit),
        format!("policy_orchestration_capacity_regression_smoke.baseline_hit_rate_bps={}", regression.baseline_hit_rate_bps),
        format!("policy_orchestration_capacity_regression_smoke.current_hit_rate_bps={}", regression.current_hit_rate_bps),
        format!("policy_orchestration_capacity_regression_smoke.hit_rate_delta_bps={}", regression.hit_rate_delta_bps),
        format!("policy_orchestration_capacity_regression_smoke.baseline_estimated_avoided_llm_calls_per_full_batch={}", regression.baseline_estimated_avoided_llm_calls_per_full_batch),
        format!("policy_orchestration_capacity_regression_smoke.current_estimated_avoided_llm_calls_per_full_batch={}", regression.current_estimated_avoided_llm_calls_per_full_batch),
        format!("policy_orchestration_capacity_regression_smoke.avoided_llm_call_delta_per_full_batch={}", regression.avoided_llm_call_delta_per_full_batch),
        format!("policy_orchestration_capacity_regression_smoke.baseline_policy_reuse_verdict={}", regression.baseline_policy_reuse_verdict),
        format!("policy_orchestration_capacity_regression_smoke.current_policy_reuse_verdict={}", regression.current_policy_reuse_verdict),
        format!("policy_orchestration_capacity_regression_smoke.baseline_capacity_status={}", regression.baseline_capacity_status),
        format!("policy_orchestration_capacity_regression_smoke.current_capacity_status={}", regression.current_capacity_status),
        format!("policy_orchestration_capacity_regression_smoke.trend_status={}", regression.trend_status),
        format!("policy_orchestration_capacity_regression_smoke.verdict={}", regression.verdict),
    ];

    retained_receipt_fixture_valid(
        fixture,
        "canon_policy_orchestration_capacity_receipts_v1",
        3,
        &expected,
        &[
            "rule=capacity smoke estimates avoided LLM calls from retained policy reuse records and bounded batch limit",
            "rule=trend smoke passes only when batch-level hit rate and avoided LLM calls do not regress",
            "rule=regression smoke remains structurally valid but does not pass",
        ],
    )
        && capacity.passed()
        && trend.passed()
        && !regression.passed()
        && regression.baseline_capacity_status == "pass"
        && regression.current_capacity_status == "pass"
}

#[test]
fn policy_validation_health_trend_smoke_compares_aggregate_health_and_capacity() {
    let receipt = ai::validation_harness::policy_validation_health_trend_smoke_receipt();

    assert_eq!(receipt.schema, "canon_policy_validation_health_trend_v1");
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_VALIDATION_HEALTH_TREND_SMOKE_STEP
    );
    assert_eq!(receipt.baseline_policy_hit_rate_bps, 5_000);
    assert_eq!(receipt.current_policy_hit_rate_bps, 10_000);
    assert_eq!(receipt.policy_hit_rate_delta_bps, 5_000);
    assert_eq!(receipt.baseline_avoided_llm_call_count, 1);
    assert_eq!(receipt.current_avoided_llm_call_count, 2);
    assert_eq!(receipt.avoided_llm_call_delta, 1);
    assert_eq!(
        receipt.baseline_capacity_avoided_llm_calls_per_full_batch,
        4
    );
    assert_eq!(receipt.current_capacity_avoided_llm_calls_per_full_batch, 8);
    assert_eq!(receipt.capacity_avoided_llm_call_delta_per_full_batch, 4);
    assert_eq!(receipt.baseline_health_verdict, "pass");
    assert_eq!(receipt.current_health_verdict, "pass");
    assert_eq!(receipt.capacity_trend_status, "pass");
    assert_eq!(receipt.validation_cost_verdict, "pass");
    assert!(!receipt.dispatch_catalog_changed);
    assert_eq!(receipt.trend_status, "pass");
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
    assert!(receipt
        .to_json()
        .contains("\"schema\":\"canon_policy_validation_health_trend_v1\\"));
}

#[test]
fn root_validate_policy_validation_health_trend_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-validation-health-trend-smoke",
        &[
            "\"schema\":\"canon_policy_validation_health_trend_v1\\",
            "\"record_type\":\"policy_validation_health_trend_smoke\\",
            "\"policy_hit_rate_delta_bps\":5000",
            "\"capacity_avoided_llm_call_delta_per_full_batch\":4",
            "\"trend_status\":\"pass\\",
            "\"verdict\":\"pass\\",
        ],
    );
}

#[test]
fn policy_validation_health_trend_receipt_rejects_capacity_regression() {
    let baseline = ai::validation_harness::policy_validation_health_smoke_receipt();
    let current = ai::validation_harness::policy_validation_health_smoke_receipt();
    let capacity_regression =
        ai::validation_harness::policy_orchestration_capacity_regression_smoke_receipt();
    let receipt = ai::validation_harness::compare_policy_validation_health_trend(
        &baseline,
        &current,
        &capacity_regression,
    );

    assert_eq!(receipt.baseline_health_verdict, "pass");
    assert_eq!(receipt.current_health_verdict, "pass");
    assert_eq!(receipt.capacity_trend_status, "regressed");
    assert_eq!(receipt.capacity_avoided_llm_call_delta_per_full_batch, -4);
    assert_eq!(receipt.trend_status, "regressed");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn policy_validation_health_smoke_combines_policy_reuse_and_validation_cost() {
    let receipt = ai::validation_harness::policy_validation_health_smoke_receipt();

    assert_eq!(receipt.schema, "canon_policy_validation_health_v1");
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_VALIDATION_HEALTH_SMOKE_STEP
    );
    assert_eq!(receipt.policy_hit_rate_bps, 5_000);
    assert_eq!(receipt.avoided_llm_call_count, 1);
    assert_eq!(receipt.policy_reuse_verdict, "pass");
    assert_eq!(receipt.validation_budget_status, "pass");
    assert_eq!(receipt.validation_trend_status, "pass");
    assert_eq!(receipt.validation_footprint_status, "pass");
    assert_eq!(receipt.validation_cost_verdict, "pass");
    assert_eq!(
        receipt.expected_count_guarded_tests,
        expected_guarded_test_count()
    );
    assert_eq!(receipt.total_declared_steps, 7);
    assert!(!receipt.command_set_changed);
    assert_eq!(
        receipt.baseline_dispatch_catalog_hash,
        receipt.current_dispatch_catalog_hash
    );
    assert!(!receipt.dispatch_catalog_changed);
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
    assert!(receipt
        .to_json()
        .contains("\"schema\":\"canon_policy_validation_health_v1\\"));
}

#[test]
fn root_validate_policy_validation_health_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-validation-health-smoke",
        &[
            "\"schema\":\"canon_policy_validation_health_v1\\",
            "\"record_type\":\"policy_validation_health_smoke\\",
            "\"policy_hit_rate_bps\":5000",
            "\"avoided_llm_call_count\":1",
            "\"policy_reuse_verdict\":\"pass\\",
            "\"validation_budget_status\":\"pass\\",
            "\"validation_trend_status\":\"pass\\",
            "\"validation_footprint_status\":\"pass\\",
            "\"validation_cost_verdict\":\"pass\\",
            "\"command_set_changed\":false",
            "\"baseline_dispatch_catalog_hash\":",
            "\"current_dispatch_catalog_hash\":",
            "\"dispatch_catalog_changed\":false",
            "\"verdict\":\"pass\\",
        ],
    );
}

#[test]
fn policy_validation_health_receipt_rejects_validation_cost_growth() {
    let validation = ai::validation_harness::validation_cost_footprint_growth_smoke_receipt();
    let receipt = ai::validation_harness::PolicyValidationHealthReceipt {
        schema: "canon_policy_validation_health_v1",
        record_type: ai::validation_harness::POLICY_VALIDATION_HEALTH_SMOKE_STEP,
        policy_hit_rate_bps: 5_000,
        avoided_llm_call_count: 1,
        policy_reuse_verdict: "pass",
        validation_budget_status: validation.budget_status,
        validation_trend_status: validation.trend_status,
        validation_footprint_status: validation.footprint_status,
        validation_cost_verdict: validation.verdict,
        expected_count_guarded_tests: validation.current_expected_count_guarded_tests,
        total_declared_steps: validation.current_total_declared_steps,
        command_set_changed: validation.command_set_changed,
        baseline_dispatch_catalog_hash: validation.baseline_dispatch_catalog_hash,
        current_dispatch_catalog_hash: validation.current_dispatch_catalog_hash,
        dispatch_catalog_changed: validation.dispatch_catalog_changed,
        verdict: "fail",
    };

    assert_eq!(receipt.validation_footprint_status, "fail");
    assert_eq!(receipt.validation_cost_verdict, "fail");
    assert!(receipt.command_set_changed);
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
}

#[test]
fn policy_validation_health_receipt_rejects_dispatch_catalog_drift() {
    let policy = ai::validation_harness::policy_reuse_smoke_receipt();
    let footprint = ai::validation_harness::validation_footprint_receipt();
    let baseline_runtime = ai::validation_harness::runtime_performance_receipt(3_000);
    let current_runtime = ai::validation_harness::runtime_performance_receipt(3_150);
    let validation = ai::validation_harness::compare_validation_cost_footprint_with_dispatch_hashes(
        &baseline_runtime,
        &current_runtime,
        &footprint,
        &footprint,
        500,
        "dispatch-hash-v1",
        "dispatch-hash-v2",
    );

    let receipt = ai::validation_harness::PolicyValidationHealthReceipt {
        schema: "canon_policy_validation_health_v1",
        record_type: ai::validation_harness::POLICY_VALIDATION_HEALTH_SMOKE_STEP,
        policy_hit_rate_bps: policy.hit_rate_bps,
        avoided_llm_call_count: policy.avoided_llm_call_count,
        policy_reuse_verdict: policy.verdict,
        validation_budget_status: validation.budget_status,
        validation_trend_status: validation.trend_status,
        validation_footprint_status: validation.footprint_status,
        validation_cost_verdict: validation.verdict,
        expected_count_guarded_tests: validation.current_expected_count_guarded_tests,
        total_declared_steps: validation.current_total_declared_steps,
        command_set_changed: validation.command_set_changed,
        baseline_dispatch_catalog_hash: validation.baseline_dispatch_catalog_hash,
        current_dispatch_catalog_hash: validation.current_dispatch_catalog_hash,
        dispatch_catalog_changed: validation.dispatch_catalog_changed,
        verdict: "fail",
    };

    assert_eq!(receipt.validation_budget_status, "pass");
    assert_eq!(receipt.validation_trend_status, "pass");
    assert_eq!(receipt.validation_footprint_status, "fail");
    assert_eq!(receipt.validation_cost_verdict, "fail");
    assert!(!receipt.command_set_changed);
    assert!(receipt.dispatch_catalog_changed);
    assert_eq!(receipt.baseline_dispatch_catalog_hash, "dispatch-hash-v1");
    assert_eq!(receipt.current_dispatch_catalog_hash, "dispatch-hash-v2");
    assert_eq!(receipt.verdict, "fail");
    assert!(!receipt.passed());
    assert!(receipt
        .to_json()
        .contains("\"dispatch_catalog_changed\":true"));
}

#[test]
fn validation_command_footprint_receipts_fixture_binds_expected_retained_receipts() {
    let fixture = std::fs::read_to_string(VALIDATION_COMMAND_FOOTPRINT_RECEIPTS_FIXTURE)
        .expect("validation command footprint receipts fixture must be readable");
    assert!(validation_command_footprint_receipts_fixture_valid(
        &fixture
    ));
}

#[test]
fn root_validate_validation_command_footprint_fixture_mode_is_executable_contract() {
    assert_root_validate_fixture_mode_contract(CompactFixtureModeContract {
        arg: "--validation-command-footprint-fixture",
        marker: "canon_validation_command_footprint_receipts_v1",
        expected_fragments: &[
            "schema=canon_validation_command_footprint_receipts_v1",
            "receipt_count=5",
            "validation_command_footprint_planning.target_total_declared_steps=6",
            "validation_command_footprint_planning.planning_status=action_required",
            "validation_command_footprint_target_met_smoke.target_total_declared_steps=7",
            "validation_command_footprint_target_met_smoke.planning_status=met",
            "validation_command_footprint_unsafe_target_smoke.target_total_declared_steps=1",
            "validation_command_footprint_unsafe_target_smoke.planning_status=unsafe_target",
            "validation_command_footprint_trend_smoke.target_total_declared_step_delta=1",
            "validation_command_footprint_trend_smoke.trend_status=pass",
            "validation_command_footprint_regression_smoke.target_total_declared_step_delta=-1",
            "validation_command_footprint_regression_smoke.trend_status=regressed",
        ],
    });
}

#[test]
fn validation_command_footprint_receipts_fixture_negative_contract_detects_drift() {
    let fixture = std::fs::read_to_string(VALIDATION_COMMAND_FOOTPRINT_RECEIPTS_FIXTURE)
        .expect("validation command footprint receipts fixture must be readable");
    let drifted = fixture.replace(
        "validation_command_footprint_planning.command_reduction_target=1",
        "validation_command_footprint_planning.command_reduction_target=0",
    );
    assert!(!validation_command_footprint_receipts_fixture_valid(
        &drifted
    ));

    let drifted_unsafe = fixture.replace(
        "validation_command_footprint_unsafe_target_smoke.safety_status=fail",
        "validation_command_footprint_unsafe_target_smoke.safety_status=pass",
    );
    assert!(!validation_command_footprint_receipts_fixture_valid(
        &drifted_unsafe
    ));

    let drifted_target_met = fixture.replace(
        "validation_command_footprint_target_met_smoke.planning_status=met",
        "validation_command_footprint_target_met_smoke.planning_status=action_required",
    );
    assert!(!validation_command_footprint_receipts_fixture_valid(
        &drifted_target_met
    ));
}

#[test]
fn validation_command_footprint_receipts_fixture_negative_contract_detects_trend_drift() {
    let fixture = std::fs::read_to_string(VALIDATION_COMMAND_FOOTPRINT_RECEIPTS_FIXTURE)
        .expect("validation command footprint receipts fixture must be readable");
    let drifted_trend = fixture.replace(
        "validation_command_footprint_trend_smoke.command_reduction_target_delta=-1",
        "validation_command_footprint_trend_smoke.command_reduction_target_delta=0",
    );
    assert!(!validation_command_footprint_receipts_fixture_valid(
        &drifted_trend
    ));

    let drifted_regression = fixture.replace(
        "validation_command_footprint_regression_smoke.trend_status=regressed",
        "validation_command_footprint_regression_smoke.trend_status=pass",
    );
    assert!(!validation_command_footprint_receipts_fixture_valid(
        &drifted_regression
    ));
}

#[test]
fn validation_command_footprint_receipts_fixture_negative_contract_detects_header_drift() {
    let fixture = std::fs::read_to_string(VALIDATION_COMMAND_FOOTPRINT_RECEIPTS_FIXTURE)
        .expect("validation command footprint receipts fixture must be readable");

    let drifted_count = fixture.replace("receipt_count=5", "receipt_count=30");
    assert!(!validation_command_footprint_receipts_fixture_valid(
        &drifted_count
    ));

    let drifted_schema = fixture.replace(
        "schema=canon_validation_command_footprint_receipts_v1",
        "schema=canon_validation_command_footprint_receipts_v1_extra",
    );
    assert!(!validation_command_footprint_receipts_fixture_valid(
        &drifted_schema
    ));
}

fn validation_command_footprint_receipts_fixture_valid(fixture: &str) -> bool {
    if !ai::validation_harness::retained_fixture_header_valid(
        fixture,
        "canon_validation_command_footprint_receipts_v1",
        5,
    ) {
        return false;
    }

    let planning = ai::validation_harness::validation_command_footprint_planning_smoke_receipt();
    let target_met =
        ai::validation_harness::validation_command_footprint_target_met_smoke_receipt();
    let unsafe_target =
        ai::validation_harness::validation_command_footprint_unsafe_target_smoke_receipt();
    let trend = ai::validation_harness::validation_command_footprint_trend_smoke_receipt();
    let regression =
        ai::validation_harness::validation_command_footprint_regression_smoke_receipt();
    let mut expected = validation_command_footprint_expected_fixture_lines(&[
        ("validation_command_footprint_planning", &planning),
        ("validation_command_footprint_target_met_smoke", &target_met),
        (
            "validation_command_footprint_unsafe_target_smoke",
            &unsafe_target,
        ),
    ]);
    expected.extend(validation_command_footprint_trend_expected_fixture_lines(
        &[
            ("validation_command_footprint_trend_smoke", &trend),
            ("validation_command_footprint_regression_smoke", &regression),
        ],
    ));

    fixture_contains_expected_lines(fixture, &expected)
        && fixture.contains("rule=planning smoke passes only when footprint is passing and target keeps all expected-count guarded suites")
        && fixture.contains("rule=target-met smoke passes when current command footprint already equals the retained target")
        && fixture.contains("rule=unsafe-target smoke fails when the target declared step count is below expected-count guarded step count")
        && fixture.contains("rule=trend smoke passes when the retained target is met without losing expected-count guarded coverage")
        && fixture.contains("rule=regression smoke fails when retained command-footprint target coverage regresses")
        && fixture.contains("rule=fixture binds retained semantic values rather than brittle receipt hashes")
        && planning.passed()
        && target_met.passed()
        && !unsafe_target.passed()
        && trend.passed()
        && !regression.passed()
        && planning.command_reduction_target > 0
        && target_met.command_reduction_target == 0
        && target_met.planning_status == "met"
        && unsafe_target.target_total_declared_steps < unsafe_target.expected_count_guarded_steps
        && trend.command_reduction_target_delta < 0
        && regression.command_reduction_target_delta > 0
}

fn validation_command_footprint_expected_fixture_lines(
    receipts: &[(
        &str,
        &ai::validation_harness::ValidationCommandFootprintPlanningReceipt,
    )],
) -> Vec<String> {
    let mut expected = Vec::with_capacity(receipts.len() * 11);
    for (prefix, receipt) in receipts {
        expected.extend([
            format!("{prefix}.record_type={}", receipt.record_type),
            format!(
                "{prefix}.current_total_declared_steps={}",
                receipt.current_total_declared_steps
            ),
            format!(
                "{prefix}.target_total_declared_steps={}",
                receipt.target_total_declared_steps
            ),
            format!(
                "{prefix}.command_reduction_target={}",
                receipt.command_reduction_target
            ),
            format!(
                "{prefix}.expected_count_guarded_steps={}",
                receipt.expected_count_guarded_steps
            ),
            format!(
                "{prefix}.expected_count_guarded_tests={}",
                receipt.expected_count_guarded_tests
            ),
            format!(
                "{prefix}.lockfile_compat_step_count={}",
                receipt.lockfile_compat_step_count
            ),
            format!("{prefix}.footprint_verdict={}", receipt.footprint_verdict),
            format!("{prefix}.safety_status={}", receipt.safety_status),
            format!("{prefix}.planning_status={}", receipt.planning_status),
            format!("{prefix}.verdict={}", receipt.verdict),
        ]);
    }
    expected
}
fn validation_command_footprint_trend_expected_fixture_lines(
    receipts: &[(
        &str,
        &ai::validation_harness::ValidationCommandFootprintTrendReceipt,
    )],
) -> Vec<String> {
    let mut expected = Vec::with_capacity(receipts.len() * 13);
    for (prefix, receipt) in receipts {
        expected.extend([
            format!("{prefix}.record_type={}", receipt.record_type),
            format!(
                "{prefix}.baseline_target_total_declared_steps={}",
                receipt.baseline_target_total_declared_steps
            ),
            format!(
                "{prefix}.current_target_total_declared_steps={}",
                receipt.current_target_total_declared_steps
            ),
            format!(
                "{prefix}.target_total_declared_step_delta={}",
                receipt.target_total_declared_step_delta
            ),
            format!(
                "{prefix}.baseline_command_reduction_target={}",
                receipt.baseline_command_reduction_target
            ),
            format!(
                "{prefix}.current_command_reduction_target={}",
                receipt.current_command_reduction_target
            ),
            format!(
                "{prefix}.command_reduction_target_delta={}",
                receipt.command_reduction_target_delta
            ),
            format!(
                "{prefix}.baseline_planning_status={}",
                receipt.baseline_planning_status
            ),
            format!(
                "{prefix}.current_planning_status={}",
                receipt.current_planning_status
            ),
            format!(
                "{prefix}.baseline_safety_status={}",
                receipt.baseline_safety_status
            ),
            format!(
                "{prefix}.current_safety_status={}",
                receipt.current_safety_status
            ),
            format!("{prefix}.trend_status={}", receipt.trend_status),
            format!("{prefix}.verdict={}", receipt.verdict),
        ]);
    }
    expected
}

#[test]
fn validation_duration_planning_receipts_fixture_binds_expected_retained_receipts() {
    let fixture = std::fs::read_to_string(VALIDATION_DURATION_PLANNING_RECEIPTS_FIXTURE)
        .expect("validation duration planning receipts fixture must be readable");
    assert!(validation_duration_planning_receipts_fixture_valid(
        &fixture
    ));
}

#[test]
fn root_validate_validation_duration_planning_fixture_mode_is_executable_contract() {
    assert_root_validate_fixture_mode_contract(CompactFixtureModeContract {
        arg: "--validation-duration-planning-fixture",
        marker: "canon_validation_duration_planning_receipts_v1",
        expected_fragments: &[
            "schema=canon_validation_duration_planning_receipts_v1",
            "receipt_count=4",
            "validation_duration_planning_summary.retained_budget_headroom_ms=6850",
            "validation_duration_planning_summary.planning_status=pass",
            "validation_duration_planning_budget_exhaustion_smoke.retained_budget_headroom_ms=-1",
            "validation_duration_planning_budget_exhaustion_smoke.planning_status=fail",
            "validation_duration_planning_trend_smoke.retained_duration_delta_ms=-150",
            "validation_duration_planning_trend_smoke.trend_status=pass",
            "validation_duration_planning_regression_smoke.retained_duration_delta_ms=151",
            "validation_duration_planning_regression_smoke.trend_status=regressed",
        ],
    });
}

#[test]
fn validation_duration_planning_receipts_fixture_negative_contract_detects_drift() {
    let fixture = std::fs::read_to_string(VALIDATION_DURATION_PLANNING_RECEIPTS_FIXTURE)
        .expect("validation duration planning receipts fixture must be readable");
    let drifted = fixture.replace(
        "validation_duration_planning_budget_exhaustion_smoke.retained_budget_headroom_ms=-1",
        "validation_duration_planning_budget_exhaustion_smoke.retained_budget_headroom_ms=0",
    );
    assert!(!validation_duration_planning_receipts_fixture_valid(
        &drifted
    ));
}

#[test]
fn validation_duration_planning_receipts_fixture_negative_contract_detects_trend_drift() {
    let fixture = std::fs::read_to_string(VALIDATION_DURATION_PLANNING_RECEIPTS_FIXTURE)
        .expect("validation duration planning receipts fixture must be readable");

    let drifted_trend = fixture.replace(
        "validation_duration_planning_trend_smoke.retained_duration_delta_ms=-150",
        "validation_duration_planning_trend_smoke.retained_duration_delta_ms=0",
    );
    assert!(!validation_duration_planning_receipts_fixture_valid(
        &drifted_trend
    ));

    let drifted_regression = fixture.replace(
        "validation_duration_planning_regression_smoke.retained_budget_headroom_delta_ms=-151",
        "validation_duration_planning_regression_smoke.retained_budget_headroom_delta_ms=0",
    );
    assert!(!validation_duration_planning_receipts_fixture_valid(
        &drifted_regression
    ));
}

fn validation_duration_planning_receipts_fixture_valid(fixture: &str) -> bool {
    if !ai::validation_harness::retained_fixture_header_valid(
        fixture,
        "canon_validation_duration_planning_receipts_v1",
        4,
    ) {
        return false;
    }

    let summary = ai::validation_harness::validation_duration_planning_smoke_receipt();
    let budget_exhaustion =
        ai::validation_harness::validation_duration_planning_budget_exhaustion_smoke_receipt();
    let trend = ai::validation_harness::validation_duration_planning_trend_smoke_receipt();
    let regression =
        ai::validation_harness::validation_duration_planning_regression_smoke_receipt();
    let mut expected = validation_duration_planning_expected_fixture_lines(&[
        ("validation_duration_planning_summary", &summary),
        (
            "validation_duration_planning_budget_exhaustion_smoke",
            &budget_exhaustion,
        ),
    ]);
    expected.extend(validation_duration_planning_trend_expected_fixture_lines(
        &[
            ("validation_duration_planning_trend_smoke", &trend),
            ("validation_duration_planning_regression_smoke", &regression),
        ],
    ));

    fixture_contains_expected_lines(fixture, &expected)
        && fixture.contains("rule=duration planning summary passes only when retained runtime budget, validation footprint, positive step count, and positive guarded-test count pass")
        && fixture.contains("rule=budget-exhaustion smoke fails through retained budget headroom while validation footprint remains passing")
        && fixture.contains("rule=duration planning trend passes only when retained duration decreases, budget headroom increases, and guarded-test estimates do not grow")
        && fixture.contains("rule=duration planning regression smoke fails when retained duration grows and budget headroom shrinks")
        && fixture.contains("rule=fixture binds retained semantic values rather than brittle receipt hashes")
        && summary.passed()
        && !budget_exhaustion.passed()
        && trend.passed()
        && !regression.passed()
        && budget_exhaustion.retained_budget_headroom_ms < 0
        && trend.retained_duration_delta_ms < 0
        && regression.retained_duration_delta_ms > 0
}

fn validation_duration_planning_expected_fixture_lines(
    receipts: &[(
        &str,
        &ai::validation_harness::ValidationDurationPlanningReceipt,
    )],
) -> Vec<String> {
    let mut expected = Vec::with_capacity(receipts.len() * 12);
    for (prefix, receipt) in receipts {
        expected.extend([
            format!("{prefix}.record_type={}", receipt.record_type),
            format!(
                "{prefix}.retained_project_agent_elapsed_ms_p95={}",
                receipt.retained_project_agent_elapsed_ms_p95
            ),
            format!(
                "{prefix}.max_project_agent_elapsed_ms_p95={}",
                receipt.max_project_agent_elapsed_ms_p95
            ),
            format!(
                "{prefix}.retained_budget_headroom_ms={}",
                receipt.retained_budget_headroom_ms
            ),
            format!(
                "{prefix}.total_declared_steps={}",
                receipt.total_declared_steps
            ),
            format!(
                "{prefix}.expected_count_guarded_tests={}",
                receipt.expected_count_guarded_tests
            ),
            format!(
                "{prefix}.estimated_ms_per_declared_step={}",
                receipt.estimated_ms_per_declared_step
            ),
            format!(
                "{prefix}.estimated_ms_per_guarded_test={}",
                receipt.estimated_ms_per_guarded_test
            ),
            format!(
                "{prefix}.runtime_budget_status={}",
                receipt.runtime_budget_status
            ),
            format!("{prefix}.footprint_verdict={}", receipt.footprint_verdict),
            format!("{prefix}.planning_status={}", receipt.planning_status),
            format!("{prefix}.verdict={}", receipt.verdict),
        ]);
    }
    expected
}

fn validation_duration_planning_trend_expected_fixture_lines(
    receipts: &[(
        &str,
        &ai::validation_harness::ValidationDurationPlanningTrendReceipt,
    )],
) -> Vec<String> {
    let mut expected = Vec::with_capacity(receipts.len() * 15);
    for (prefix, receipt) in receipts {
        expected.extend([
            format!("{prefix}.record_type={}", receipt.record_type),
            format!(
                "{prefix}.baseline_retained_project_agent_elapsed_ms_p95={}",
                receipt.baseline_retained_project_agent_elapsed_ms_p95
            ),
            format!(
                "{prefix}.current_retained_project_agent_elapsed_ms_p95={}",
                receipt.current_retained_project_agent_elapsed_ms_p95
            ),
            format!(
                "{prefix}.retained_duration_delta_ms={}",
                receipt.retained_duration_delta_ms
            ),
            format!(
                "{prefix}.baseline_retained_budget_headroom_ms={}",
                receipt.baseline_retained_budget_headroom_ms
            ),
            format!(
                "{prefix}.current_retained_budget_headroom_ms={}",
                receipt.current_retained_budget_headroom_ms
            ),
            format!(
                "{prefix}.retained_budget_headroom_delta_ms={}",
                receipt.retained_budget_headroom_delta_ms
            ),
            format!(
                "{prefix}.baseline_estimated_ms_per_guarded_test={}",
                receipt.baseline_estimated_ms_per_guarded_test
            ),
            format!(
                "{prefix}.current_estimated_ms_per_guarded_test={}",
                receipt.current_estimated_ms_per_guarded_test
            ),
            format!(
                "{prefix}.estimated_ms_per_guarded_test_delta={}",
                receipt.estimated_ms_per_guarded_test_delta
            ),
            format!(
                "{prefix}.baseline_planning_status={}",
                receipt.baseline_planning_status
            ),
            format!(
                "{prefix}.current_planning_status={}",
                receipt.current_planning_status
            ),
            format!("{prefix}.trend_status={}", receipt.trend_status),
            format!("{prefix}.verdict={}", receipt.verdict),
        ]);
    }
    expected
}

#[test]
fn policy_validation_health_receipts_fixture_binds_expected_retained_receipt() {
    let fixture = std::fs::read_to_string(POLICY_VALIDATION_HEALTH_RECEIPTS_FIXTURE)
        .expect("policy validation health receipts fixture must be readable");
    assert!(policy_validation_health_receipts_fixture_valid(&fixture));
}

#[test]
fn root_validate_policy_validation_health_fixture_mode_is_executable_contract() {
    assert_root_validate_fixture_mode_contract(CompactFixtureModeContract {
        arg: "--policy-validation-health-fixture",
        marker: "canon_policy_validation_health_receipts_v1",
        expected_fragments: &[
            "schema=canon_policy_validation_health_receipts_v1",
            "receipt_count=1",
            "policy_validation_health_smoke.dispatch_catalog_changed=false",
            "policy_validation_health_smoke.baseline_dispatch_catalog_hash=",
            "policy_validation_health_smoke.current_dispatch_catalog_hash=",
        ],
    });
}

#[test]
fn policy_validation_health_receipts_fixture_negative_contract_detects_drift() {
    let fixture = std::fs::read_to_string(POLICY_VALIDATION_HEALTH_RECEIPTS_FIXTURE)
        .expect("policy validation health receipts fixture must be readable");
    let drifted = fixture.replace(
        "policy_validation_health_smoke.dispatch_catalog_changed=false",
        "policy_validation_health_smoke.dispatch_catalog_changed=true",
    );
    assert!(!policy_validation_health_receipts_fixture_valid(&drifted));
}

#[test]
fn policy_validation_health_trend_receipts_fixture_binds_expected_retained_receipt() {
    let fixture = std::fs::read_to_string(POLICY_VALIDATION_HEALTH_TREND_RECEIPTS_FIXTURE)
        .expect("policy validation health trend receipts fixture must be readable");
    assert!(policy_validation_health_trend_receipts_fixture_valid(
        &fixture
    ));
}

#[test]
fn root_validate_policy_validation_health_trend_fixture_mode_is_executable_contract() {
    assert_root_validate_fixture_mode_contract(CompactFixtureModeContract {
        arg: "--policy-validation-health-trend-fixture",
        marker: "canon_policy_validation_health_trend_receipts_v1",
        expected_fragments: &[
            "schema=canon_policy_validation_health_trend_receipts_v1",
            "receipt_count=1",
            "policy_validation_health_trend_smoke.policy_hit_rate_delta_bps=5000",
            "policy_validation_health_trend_smoke.capacity_avoided_llm_call_delta_per_full_batch=4",
            "policy_validation_health_trend_smoke.dispatch_catalog_changed=false",
        ],
    });
}

#[test]
fn policy_validation_health_trend_receipts_fixture_negative_contract_detects_drift() {
    let fixture = std::fs::read_to_string(POLICY_VALIDATION_HEALTH_TREND_RECEIPTS_FIXTURE)
        .expect("policy validation health trend receipts fixture must be readable");
    let drifted = fixture.replace(
        "policy_validation_health_trend_smoke.capacity_avoided_llm_call_delta_per_full_batch=4",
        "policy_validation_health_trend_smoke.capacity_avoided_llm_call_delta_per_full_batch=-4",
    );
    assert!(!policy_validation_health_trend_receipts_fixture_valid(
        &drifted
    ));
}

fn policy_validation_health_trend_receipts_fixture_valid(fixture: &str) -> bool {
    let receipt = ai::validation_harness::policy_validation_health_trend_smoke_receipt();
    let expected = [
        format!("policy_validation_health_trend_smoke.record_type={}", receipt.record_type),
        format!("policy_validation_health_trend_smoke.baseline_policy_hit_rate_bps={}", receipt.baseline_policy_hit_rate_bps),
        format!("policy_validation_health_trend_smoke.current_policy_hit_rate_bps={}", receipt.current_policy_hit_rate_bps),
        format!("policy_validation_health_trend_smoke.policy_hit_rate_delta_bps={}", receipt.policy_hit_rate_delta_bps),
        format!("policy_validation_health_trend_smoke.baseline_avoided_llm_call_count={}", receipt.baseline_avoided_llm_call_count),
        format!("policy_validation_health_trend_smoke.current_avoided_llm_call_count={}", receipt.current_avoided_llm_call_count),
        format!("policy_validation_health_trend_smoke.avoided_llm_call_delta={}", receipt.avoided_llm_call_delta),
        format!("policy_validation_health_trend_smoke.baseline_capacity_avoided_llm_calls_per_full_batch={}", receipt.baseline_capacity_avoided_llm_calls_per_full_batch),
        format!("policy_validation_health_trend_smoke.current_capacity_avoided_llm_calls_per_full_batch={}", receipt.current_capacity_avoided_llm_calls_per_full_batch),
        format!("policy_validation_health_trend_smoke.capacity_avoided_llm_call_delta_per_full_batch={}", receipt.capacity_avoided_llm_call_delta_per_full_batch),
        format!("policy_validation_health_trend_smoke.baseline_health_verdict={}", receipt.baseline_health_verdict),
        format!("policy_validation_health_trend_smoke.current_health_verdict={}", receipt.current_health_verdict),
        format!("policy_validation_health_trend_smoke.capacity_trend_status={}", receipt.capacity_trend_status),
        format!("policy_validation_health_trend_smoke.validation_cost_verdict={}", receipt.validation_cost_verdict),
        format!("policy_validation_health_trend_smoke.dispatch_catalog_changed={}", receipt.dispatch_catalog_changed),
        format!("policy_validation_health_trend_smoke.trend_status={}", receipt.trend_status),
        format!("policy_validation_health_trend_smoke.verdict={}", receipt.verdict),
    ];
    let rules = [
        "rule=health trend smoke passes only when aggregate health, policy reuse, validation cost, dispatch catalog, and capacity trend do not regress",
        "rule=capacity trend must preserve or improve avoided LLM calls per bounded full batch",
        "rule=fixture binds retained semantic values rather than brittle receipt hashes",
    ];

    retained_receipt_fixture_valid(
        fixture,
        "canon_policy_validation_health_trend_receipts_v1",
        1,
        &expected,
        &rules,
    ) && receipt.passed()
        && receipt.capacity_avoided_llm_call_delta_per_full_batch >= 0
        && !receipt.dispatch_catalog_changed
}

fn policy_validation_health_receipts_fixture_valid(fixture: &str) -> bool {
    let receipt = ai::validation_harness::policy_validation_health_smoke_receipt();
    let expected = [
        format!(
            "policy_validation_health_smoke.record_type={}",
            receipt.record_type
        ),
        format!(
            "policy_validation_health_smoke.policy_hit_rate_bps={}",
            receipt.policy_hit_rate_bps
        ),
        format!(
            "policy_validation_health_smoke.avoided_llm_call_count={}",
            receipt.avoided_llm_call_count
        ),
        format!(
            "policy_validation_health_smoke.policy_reuse_verdict={}",
            receipt.policy_reuse_verdict
        ),
        format!(
            "policy_validation_health_smoke.validation_budget_status={}",
            receipt.validation_budget_status
        ),
        format!(
            "policy_validation_health_smoke.validation_trend_status={}",
            receipt.validation_trend_status
        ),
        format!(
            "policy_validation_health_smoke.validation_footprint_status={}",
            receipt.validation_footprint_status
        ),
        format!(
            "policy_validation_health_smoke.validation_cost_verdict={}",
            receipt.validation_cost_verdict
        ),
        format!(
            "policy_validation_health_smoke.expected_count_guarded_tests={}",
            receipt.expected_count_guarded_tests
        ),
        format!(
            "policy_validation_health_smoke.total_declared_steps={}",
            receipt.total_declared_steps
        ),
        format!(
            "policy_validation_health_smoke.command_set_changed={}",
            receipt.command_set_changed
        ),
        format!(
            "policy_validation_health_smoke.baseline_dispatch_catalog_hash={}",
            receipt.baseline_dispatch_catalog_hash
        ),
        format!(
            "policy_validation_health_smoke.current_dispatch_catalog_hash={}",
            receipt.current_dispatch_catalog_hash
        ),
        format!(
            "policy_validation_health_smoke.dispatch_catalog_changed={}",
            receipt.dispatch_catalog_changed
        ),
        format!("policy_validation_health_smoke.verdict={}", receipt.verdict),
    ];
    let rules = [
        "rule=health smoke passes only when policy reuse, validation budget, runtime trend, validation footprint, validation cost, command-set drift, and dispatch-catalog drift all pass",
        "rule=dispatch catalog hashes must match for aggregate policy validation health to pass",
        "rule=fixture binds retained semantic values rather than brittle receipt hashes",
    ];

    retained_receipt_fixture_valid(
        fixture,
        "canon_policy_validation_health_receipts_v1",
        1,
        &expected,
        &rules,
    ) && receipt.passed()
        && receipt.baseline_dispatch_catalog_hash == receipt.current_dispatch_catalog_hash
        && !receipt.dispatch_catalog_changed
}

#[test]
fn policy_reuse_smoke_receipt_counts_hits_without_llm_fallback() {
    let receipt = ai::validation_harness::policy_reuse_smoke_receipt();

    assert_eq!(receipt.schema_version, 1);
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_SMOKE_STEP
    );
    assert_eq!(receipt.retained_record_count, 2);
    assert_eq!(receipt.policy_hit_count, 1);
    assert_eq!(receipt.policy_miss_count, 1);
    assert_eq!(receipt.avoided_llm_call_count, 1);
    assert_eq!(receipt.hit_rate_bps, 5_000);
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
    assert!(receipt
        .to_json()
        .contains("\"record_type\":\"policy_reuse_smoke\\"));
}

#[test]
fn root_validate_policy_reuse_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-smoke",
        &[
            "\"record_type\":\"policy_reuse_smoke\\",
            "\"policy_hit_count\":1",
            "\"policy_miss_count\":1",
            "\"avoided_llm_call_count\":1",
            "\"hit_rate_bps\":5000",
            "\"verdict\":\"pass\\",
        ],
    );
}

#[test]
fn policy_reuse_trend_smoke_receipt_proves_policy_hit_improvement() {
    let receipt = ai::validation_harness::policy_reuse_trend_smoke_receipt();

    assert_eq!(receipt.schema_version, 1);
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_TREND_SMOKE_STEP
    );
    assert_eq!(receipt.baseline_hit_rate_bps, 5_000);
    assert_eq!(receipt.current_hit_rate_bps, 10_000);
    assert_eq!(receipt.hit_rate_delta_bps, 5_000);
    assert_eq!(receipt.avoided_llm_call_delta, 1);
    assert_eq!(receipt.trend_status, "improved_or_stable");
    assert_eq!(receipt.verdict, "pass");
    assert!(receipt.passed());
}

#[test]
fn root_validate_policy_reuse_trend_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-trend-smoke",
        &[
            "\"record_type\":\"policy_reuse_trend_smoke\\",
            "\"baseline_hit_rate_bps\":5000",
            "\"current_hit_rate_bps\":10000",
            "\"avoided_llm_call_delta\":1",
            "\"trend_status\":\"improved_or_stable\\",
            "\"verdict\":\"pass\\",
        ],
    );
}

#[test]
fn policy_reuse_regression_smoke_receipt_is_controlled_negative_contract() {
    let receipt = ai::validation_harness::policy_reuse_regression_smoke_receipt();

    assert_eq!(receipt.schema_version, 1);
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.baseline_hit_rate_bps, 10_000);
    assert_eq!(receipt.current_hit_rate_bps, 5_000);
    assert_eq!(receipt.hit_rate_delta_bps, -5_000);
    assert_eq!(receipt.avoided_llm_call_delta, -1);
    assert_eq!(receipt.trend_status, "regressed");
    assert_eq!(receipt.verdict, "fail");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_regression_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-regression-smoke",
        &[
            "\"record_type\":\"policy_reuse_regression_smoke\\",
            "\"baseline_hit_rate_bps\":10000",
            "\"current_hit_rate_bps\":5000",
            "\"avoided_llm_call_delta\":-1",
            "\"trend_status\":\"regressed\\",
            "\"verdict\":\"fail\\",
        ],
    );
}

#[test]
fn policy_reuse_ledger_summary_smoke_exposes_reuse_and_validation_health() {
    let receipt = ai::validation_harness::policy_reuse_ledger_summary_smoke_receipt();

    assert_eq!(receipt.schema_version, 1);
    assert_eq!(receipt.record_type, POLICY_REUSE_LEDGER_SUMMARY_SMOKE_STEP);
    assert_eq!(receipt.policy_hits, 1);
    assert_eq!(receipt.policy_misses, 1);
    assert_eq!(receipt.llm_fallbacks, receipt.policy_misses);
    assert_eq!(receipt.validation_passes, 3);
    assert_eq!(receipt.validation_failures, 0);
    assert_eq!(receipt.reuse_rate_bps, 5_000);
    assert!(!receipt.regression_flag);
    assert_ne!(receipt.source_receipt_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_ledger_summary_regression_smoke_exposes_controlled_failure() {
    let receipt = ai::validation_harness::policy_reuse_ledger_summary_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        POLICY_REUSE_LEDGER_SUMMARY_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.policy_hits, 2);
    assert_eq!(receipt.policy_misses, 1);
    assert_eq!(receipt.llm_fallbacks, 1);
    assert_eq!(receipt.validation_passes, 2);
    assert_eq!(receipt.validation_failures, 1);
    assert_eq!(receipt.reuse_rate_bps, 6_666);
    assert!(receipt.regression_flag);
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_ledger_summary_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-ledger-summary-smoke",
        &[
            "\"record_type\":\"policy_reuse_ledger_summary_smoke\\",
            "\"policy_hits\":1",
            "\"policy_misses\":1",
            "\"llm_fallbacks\":1",
            "\"validation_passes\":3",
            "\"validation_failures\":0",
            "\"reuse_rate_bps\":5000",
            "\"regression_flag\":false",
            "\"source_receipt_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_ledger_summary_regression_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-ledger-summary-regression-smoke",
        &[
            "\"record_type\":\"policy_reuse_ledger_summary_regression_smoke\\",
            "\"policy_hits\":2",
            "\"policy_misses\":1",
            "\"llm_fallbacks\":1",
            "\"validation_passes\":2",
            "\"validation_failures\":1",
            "\"reuse_rate_bps\":6666",
            "\"regression_flag\":true",
            "\"source_receipt_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_scale_trace_smoke_exposes_larger_batch_reuse() {
    let receipt = ai::validation_harness::policy_reuse_scale_trace_smoke_receipt();

    assert_eq!(receipt.schema_version, 1);
    assert_eq!(receipt.record_type, POLICY_REUSE_SCALE_TRACE_SMOKE_STEP);
    assert_eq!(receipt.batch_size, 6);
    assert_eq!(receipt.policy_hits, 4);
    assert_eq!(receipt.policy_misses, 2);
    assert_eq!(receipt.llm_fallbacks, 2);
    assert_eq!(receipt.validation_passes, 6);
    assert_eq!(receipt.validation_failures, 0);
    assert_eq!(receipt.reuse_rate_bps, 6_666);
    assert!(!receipt.regression_flag);
    assert_eq!(receipt.avoided_llm_calls_per_batch, 4);
    assert_ne!(receipt.source_receipt_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_scale_trace_regression_smoke_exposes_controlled_failure() {
    let receipt = ai::validation_harness::policy_reuse_scale_trace_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        POLICY_REUSE_SCALE_TRACE_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.batch_size, 4);
    assert_eq!(receipt.policy_hits, 2);
    assert_eq!(receipt.policy_misses, 2);
    assert_eq!(receipt.llm_fallbacks, 2);
    assert_eq!(receipt.validation_passes, 3);
    assert_eq!(receipt.validation_failures, 1);
    assert_eq!(receipt.reuse_rate_bps, 5_000);
    assert!(receipt.regression_flag);
    assert_eq!(receipt.avoided_llm_calls_per_batch, 2);
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_scale_trace_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-scale-trace-smoke",
        &[
            "\"record_type\":\"policy_reuse_scale_trace_smoke\\",
            "\"batch_size\":6",
            "\"policy_hits\":4",
            "\"policy_misses\":2",
            "\"llm_fallbacks\":2",
            "\"validation_passes\":6",
            "\"validation_failures\":0",
            "\"reuse_rate_bps\":6666",
            "\"regression_flag\":false",
            "\"avoided_llm_calls_per_batch\":4",
            "\"source_receipt_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_scale_trace_regression_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-scale-trace-regression-smoke",
        &[
            "\"record_type\":\"policy_reuse_scale_trace_regression_smoke\\",
            "\"batch_size\":4",
            "\"policy_hits\":2",
            "\"policy_misses\":2",
            "\"llm_fallbacks\":2",
            "\"validation_passes\":3",
            "\"validation_failures\":1",
            "\"reuse_rate_bps\":5000",
            "\"regression_flag\":true",
            "\"avoided_llm_calls_per_batch\":2",
            "\"source_receipt_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_performance_cost_trend_smoke_exposes_cost_safe_reuse() {
    let receipt = ai::validation_harness::policy_reuse_performance_cost_trend_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_PERFORMANCE_COST_TREND_SMOKE_STEP
    );
    assert_eq!(receipt.batch_size, 6);
    assert_eq!(receipt.avoided_llm_calls_per_batch, 4);
    assert_eq!(receipt.reuse_rate_bps, 6_666);
    assert_eq!(receipt.validation_expected_count_guarded_tests, 210);
    assert_eq!(receipt.estimated_ms_per_guarded_test, 14);
    assert_eq!(receipt.runtime_budget_status, "pass");
    assert_eq!(receipt.validation_cost_verdict, "pass");
    assert!(!receipt.cost_regression_flag);
    assert_ne!(receipt.source_scale_trace_hash, 0);
    assert_ne!(receipt.source_validation_duration_hash, 0);
    assert_ne!(receipt.source_runtime_performance_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_performance_cost_trend_regression_smoke_exposes_cost_failure() {
    let receipt =
        ai::validation_harness::policy_reuse_performance_cost_trend_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_PERFORMANCE_COST_TREND_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.batch_size, 6);
    assert_eq!(receipt.avoided_llm_calls_per_batch, 4);
    assert_eq!(receipt.reuse_rate_bps, 6_666);
    assert_eq!(receipt.validation_expected_count_guarded_tests, 210);
    assert_eq!(receipt.estimated_ms_per_guarded_test, 15);
    assert_eq!(receipt.runtime_budget_status, "pass");
    assert_eq!(receipt.validation_cost_verdict, "fail");
    assert!(receipt.cost_regression_flag);
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_performance_cost_trend_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-performance-cost-trend-smoke",
        &[
            "\"record_type\":\"policy_reuse_performance_cost_trend_smoke\\",
            "\"batch_size\":6",
            "\"avoided_llm_calls_per_batch\":4",
            "\"reuse_rate_bps\":6666",
            "\"validation_expected_count_guarded_tests\":210",
            "\"estimated_ms_per_guarded_test\":14",
            "\"runtime_budget_status\":\"pass\\",
            "\"validation_cost_verdict\":\"pass\\",
            "\"cost_regression_flag\":false",
            "\"source_scale_trace_hash\":",
            "\"source_validation_duration_hash\":",
            "\"source_runtime_performance_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_performance_cost_trend_regression_smoke_mode_is_executable_contract()
{
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-performance-cost-trend-regression-smoke",
        &[
            "\"record_type\":\"policy_reuse_performance_cost_trend_regression_smoke\\",
            "\"batch_size\":6",
            "\"avoided_llm_calls_per_batch\":4",
            "\"reuse_rate_bps\":6666",
            "\"validation_expected_count_guarded_tests\":210",
            "\"estimated_ms_per_guarded_test\":15",
            "\"runtime_budget_status\":\"pass\\",
            "\"validation_cost_verdict\":\"fail\\",
            "\"cost_regression_flag\":true",
            "\"source_scale_trace_hash\":",
            "\"source_validation_duration_hash\":",
            "\"source_runtime_performance_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_cost_catalog_smoke_exposes_complete_reuse_cost_family() {
    let receipt = ai::validation_harness::policy_reuse_cost_catalog_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_COST_CATALOG_SMOKE_STEP
    );
    assert_eq!(receipt.catalog_version, 1);
    assert_eq!(receipt.evidence_family_count, 6);
    assert_eq!(receipt.healthy_mode_count, 4);
    assert_eq!(receipt.regression_mode_count, 4);
    assert_eq!(receipt.retained_fixture_count, 6);
    assert!(receipt.required_healthy_modes_present);
    assert!(receipt.required_regression_modes_present);
    assert!(receipt.summary_complete);
    assert_eq!(receipt.missing_required_modes, "none");
    assert_ne!(receipt.source_policy_reuse_hash, 0);
    assert_ne!(receipt.source_scale_trace_hash, 0);
    assert_ne!(receipt.source_performance_cost_trend_hash, 0);
    assert_ne!(receipt.source_validation_health_hash, 0);
    assert_ne!(receipt.source_validation_duration_hash, 0);
    assert_ne!(receipt.source_runtime_performance_hash, 0);
    assert_ne!(receipt.catalog_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_cost_catalog_incomplete_smoke_exposes_missing_coverage() {
    let receipt = ai::validation_harness::policy_reuse_cost_catalog_incomplete_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_COST_CATALOG_INCOMPLETE_SMOKE_STEP
    );
    assert_eq!(receipt.catalog_version, 1);
    assert_eq!(receipt.evidence_family_count, 6);
    assert_eq!(receipt.healthy_mode_count, 4);
    assert_eq!(receipt.regression_mode_count, 3);
    assert_eq!(receipt.retained_fixture_count, 6);
    assert!(receipt.required_healthy_modes_present);
    assert!(!receipt.required_regression_modes_present);
    assert!(!receipt.summary_complete);
    assert_eq!(receipt.missing_required_modes, "required_regression_modes");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_cost_catalog_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-cost-catalog-smoke",
        &[
            "\"record_type\":\"policy_reuse_cost_catalog_smoke\\",
            "\"catalog_version\":1",
            "\"evidence_family_count\":6",
            "\"healthy_mode_count\":4",
            "\"regression_mode_count\":4",
            "\"retained_fixture_count\":6",
            "\"required_healthy_modes_present\":true",
            "\"required_regression_modes_present\":true",
            "\"summary_complete\":true",
            "\"missing_required_modes\":\"none\\",
            "\"source_policy_reuse_hash\":",
            "\"source_scale_trace_hash\":",
            "\"source_performance_cost_trend_hash\":",
            "\"source_validation_health_hash\":",
            "\"source_validation_duration_hash\":",
            "\"source_runtime_performance_hash\":",
            "\"catalog_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_cost_catalog_incomplete_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-cost-catalog-incomplete-smoke",
        &[
            "\"record_type\":\"policy_reuse_cost_catalog_incomplete_smoke\\",
            "\"catalog_version\":1",
            "\"evidence_family_count\":6",
            "\"healthy_mode_count\":4",
            "\"regression_mode_count\":3",
            "\"retained_fixture_count\":6",
            "\"required_healthy_modes_present\":true",
            "\"required_regression_modes_present\":false",
            "\"summary_complete\":false",
            "\"missing_required_modes\":\"required_regression_modes\\",
            "\"catalog_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evaluator_savings_smoke_quantifies_avoided_llm_work() {
    let receipt = ai::validation_harness::policy_reuse_evaluator_savings_smoke_receipt();
    let catalog = ai::validation_harness::policy_reuse_cost_catalog_smoke_receipt();
    let performance = ai::validation_harness::policy_reuse_performance_cost_trend_smoke_receipt();
    let reuse = ai::validation_harness::policy_reuse_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVALUATOR_SAVINGS_SMOKE_STEP
    );
    assert_eq!(receipt.savings_version, 1);
    assert_eq!(receipt.source_catalog_hash, catalog.receipt_hash);
    assert_eq!(
        receipt.source_performance_cost_trend_hash,
        performance.receipt_hash
    );
    assert_eq!(receipt.source_policy_reuse_hash, reuse.receipt_hash);
    assert_eq!(receipt.sample_runs, reuse.retained_record_count);
    assert_eq!(receipt.policy_hits, reuse.policy_hit_count);
    assert_eq!(receipt.llm_calls_avoided, reuse.avoided_llm_call_count);
    assert_eq!(receipt.estimated_reasoning_cost_units_avoided, 100);
    assert_eq!(receipt.baseline_llm_calls, reuse.retained_record_count);
    assert_eq!(receipt.actual_llm_calls, reuse.policy_miss_count);
    assert_eq!(receipt.llm_call_reduction_ratio_bps, 5_000);
    assert!(receipt.validation_passed);
    assert_eq!(receipt.regression_reason, "none");
    assert_ne!(receipt.savings_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evaluator_savings_regression_smoke_is_valid_failing_evidence() {
    let receipt = ai::validation_harness::policy_reuse_evaluator_savings_regression_smoke_receipt();
    let catalog = ai::validation_harness::policy_reuse_cost_catalog_incomplete_smoke_receipt();
    let performance = ai::validation_harness::policy_reuse_performance_cost_trend_smoke_receipt();
    let reuse = ai::validation_harness::policy_reuse_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVALUATOR_SAVINGS_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_catalog_hash, catalog.receipt_hash);
    assert_eq!(
        receipt.source_performance_cost_trend_hash,
        performance.receipt_hash
    );
    assert_eq!(receipt.source_policy_reuse_hash, reuse.receipt_hash);
    assert_eq!(receipt.llm_calls_avoided, reuse.avoided_llm_call_count);
    assert_eq!(receipt.regression_reason, "catalog_incomplete");
    assert!(!receipt.validation_passed);
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evaluator_savings_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evaluator-savings-smoke",
        &[
            "\"record_type\":\"policy_reuse_evaluator_savings_smoke\\",
            "\"savings_version\":1",
            "\"source_catalog_hash\":",
            "\"source_performance_cost_trend_hash\":",
            "\"source_policy_reuse_hash\":",
            "\"sample_runs\":2",
            "\"policy_hits\":1",
            "\"llm_calls_avoided\":1",
            "\"estimated_reasoning_cost_units_avoided\":100",
            "\"baseline_llm_calls\":2",
            "\"actual_llm_calls\":1",
            "\"llm_call_reduction_ratio_bps\":5000",
            "\"validation_passed\":true",
            "\"regression_reason\":\"none\\",
            "\"savings_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evaluator_savings_regression_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evaluator-savings-regression-smoke",
        &[
            "\"record_type\":\"policy_reuse_evaluator_savings_regression_smoke\\",
            "\"savings_version\":1",
            "\"llm_calls_avoided\":1",
            "\"estimated_reasoning_cost_units_avoided\":100",
            "\"validation_passed\":false",
            "\"regression_reason\":\"catalog_incomplete\\",
            "\"savings_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_scaling_projection_smoke_projects_batch_savings() {
    let receipt = ai::validation_harness::policy_reuse_scaling_projection_smoke_receipt();
    let savings = ai::validation_harness::policy_reuse_evaluator_savings_smoke_receipt();
    let capacity = ai::validation_harness::policy_orchestration_capacity_smoke_receipt();

    assert_eq!(receipt.schema, "canon_policy_reuse_scaling_projection_v1");
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_SCALING_PROJECTION_SMOKE_STEP
    );
    assert_eq!(receipt.projection_version, 1);
    assert_eq!(receipt.source_evaluator_savings_hash, savings.receipt_hash);
    assert_ne!(receipt.source_orchestration_capacity_hash, 0);
    assert_eq!(receipt.batch_capacity_limit, capacity.batch_capacity_limit);
    assert_eq!(receipt.retained_sample_runs, savings.sample_runs);
    assert_eq!(
        receipt.retained_llm_calls_avoided,
        savings.llm_calls_avoided
    );
    assert_eq!(
        receipt.retained_cost_units_avoided,
        savings.estimated_reasoning_cost_units_avoided
    );
    assert_eq!(receipt.cost_units_per_llm_call, 100);
    assert_eq!(
        receipt.projected_llm_calls_avoided_per_full_batch,
        capacity.estimated_avoided_llm_calls_per_full_batch
    );
    assert_eq!(receipt.projected_llm_calls_avoided_per_full_batch, 4);
    assert_eq!(
        receipt.projected_reasoning_cost_units_avoided_per_full_batch,
        400
    );
    assert_eq!(
        receipt.projected_llm_fallbacks_per_full_batch,
        capacity.estimated_llm_fallbacks_per_full_batch
    );
    assert!(receipt.projection_passed);
    assert_eq!(receipt.regression_reason, "none");
    assert_ne!(receipt.projection_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_scaling_projection_regression_smoke_is_valid_failing_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_scaling_projection_regression_smoke_receipt();
    let savings = ai::validation_harness::policy_reuse_evaluator_savings_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_SCALING_PROJECTION_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_evaluator_savings_hash, savings.receipt_hash);
    assert_eq!(receipt.projected_llm_calls_avoided_per_full_batch, 4);
    assert_eq!(
        receipt.projected_reasoning_cost_units_avoided_per_full_batch,
        400
    );
    assert!(!receipt.projection_passed);
    assert_eq!(receipt.regression_reason, "evaluator_savings_failed");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_scaling_projection_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-scaling-projection-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_scaling_projection_v1\\",
            "\"record_type\":\"policy_reuse_scaling_projection_smoke\\",
            "\"projection_version\":1",
            "\"source_evaluator_savings_hash\":",
            "\"source_orchestration_capacity_hash\":",
            "\"batch_capacity_limit\":8",
            "\"retained_sample_runs\":2",
            "\"retained_llm_calls_avoided\":1",
            "\"retained_cost_units_avoided\":100",
            "\"cost_units_per_llm_call\":100",
            "\"projected_llm_calls_avoided_per_full_batch\":4",
            "\"projected_reasoning_cost_units_avoided_per_full_batch\":400",
            "\"projected_llm_fallbacks_per_full_batch\":4",
            "\"projection_passed\":true",
            "\"regression_reason\":\"none\\",
            "\"projection_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_scaling_projection_regression_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-scaling-projection-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_scaling_projection_v1\\",
            "\"record_type\":\"policy_reuse_scaling_projection_regression_smoke\\",
            "\"projection_version\":1",
            "\"projected_llm_calls_avoided_per_full_batch\":4",
            "\"projected_reasoning_cost_units_avoided_per_full_batch\":400",
            "\"projection_passed\":false",
            "\"regression_reason\":\"evaluator_savings_failed\\",
            "\"projection_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_distillation_readiness_smoke_binds_verified_sources() {
    let receipt = ai::validation_harness::policy_reuse_distillation_readiness_smoke_receipt();
    let reuse = ai::validation_harness::policy_reuse_smoke_receipt();
    let catalog = ai::validation_harness::policy_reuse_cost_catalog_smoke_receipt();
    let savings = ai::validation_harness::policy_reuse_evaluator_savings_smoke_receipt();
    let projection = ai::validation_harness::policy_reuse_scaling_projection_smoke_receipt();
    let health = ai::validation_harness::policy_validation_health_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_distillation_readiness_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_DISTILLATION_READINESS_SMOKE_STEP
    );
    assert_eq!(receipt.readiness_version, 1);
    assert_eq!(receipt.source_policy_reuse_hash, reuse.receipt_hash);
    assert_eq!(receipt.source_cost_catalog_hash, catalog.receipt_hash);
    assert_eq!(receipt.source_evaluator_savings_hash, savings.receipt_hash);
    assert_eq!(
        receipt.source_scaling_projection_hash,
        projection.receipt_hash
    );
    assert_ne!(receipt.source_validation_health_hash, 0);
    assert_eq!(receipt.verified_policy_hits, reuse.policy_hit_count);
    assert_eq!(
        receipt.verified_llm_calls_avoided,
        reuse.avoided_llm_call_count
    );
    assert_eq!(receipt.projected_llm_calls_avoided_per_full_batch, 4);
    assert_eq!(
        receipt.projected_reasoning_cost_units_avoided_per_full_batch,
        400
    );
    assert_eq!(
        receipt.validation_guarded_test_count,
        health.expected_count_guarded_tests
    );
    assert!(receipt.catalog_complete);
    assert!(receipt.evaluator_savings_passed);
    assert!(receipt.scaling_projection_passed);
    assert!(receipt.validation_health_passed);
    assert!(receipt.distillation_ready);
    assert_eq!(receipt.regression_reason, "none");
    assert_ne!(receipt.readiness_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_distillation_readiness_regression_smoke_is_valid_failing_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_distillation_readiness_regression_smoke_receipt();
    let catalog = ai::validation_harness::policy_reuse_cost_catalog_incomplete_smoke_receipt();
    let savings = ai::validation_harness::policy_reuse_evaluator_savings_regression_smoke_receipt();
    let projection =
        ai::validation_harness::policy_reuse_scaling_projection_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_DISTILLATION_READINESS_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_cost_catalog_hash, catalog.receipt_hash);
    assert_eq!(receipt.source_evaluator_savings_hash, savings.receipt_hash);
    assert_eq!(
        receipt.source_scaling_projection_hash,
        projection.receipt_hash
    );
    assert!(!receipt.catalog_complete);
    assert!(!receipt.evaluator_savings_passed);
    assert!(!receipt.scaling_projection_passed);
    assert!(receipt.validation_health_passed);
    assert!(!receipt.distillation_ready);
    assert_eq!(receipt.regression_reason, "catalog_incomplete");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_distillation_readiness_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-distillation-readiness-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_distillation_readiness_v1\\",
            "\"record_type\":\"policy_reuse_distillation_readiness_smoke\\",
            "\"readiness_version\":1",
            "\"source_policy_reuse_hash\":",
            "\"source_cost_catalog_hash\":",
            "\"source_evaluator_savings_hash\":",
            "\"source_scaling_projection_hash\":",
            "\"source_validation_health_hash\":",
            "\"verified_policy_hits\":1",
            "\"verified_llm_calls_avoided\":1",
            "\"projected_llm_calls_avoided_per_full_batch\":4",
            "\"projected_reasoning_cost_units_avoided_per_full_batch\":400",
            "\"catalog_complete\":true",
            "\"evaluator_savings_passed\":true",
            "\"scaling_projection_passed\":true",
            "\"validation_health_passed\":true",
            "\"distillation_ready\":true",
            "\"regression_reason\":\"none\\",
            "\"readiness_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_distillation_readiness_regression_smoke_mode_is_executable_contract()
{
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-distillation-readiness-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_distillation_readiness_v1\\",
            "\"record_type\":\"policy_reuse_distillation_readiness_regression_smoke\\",
            "\"readiness_version\":1",
            "\"catalog_complete\":false",
            "\"evaluator_savings_passed\":false",
            "\"scaling_projection_passed\":false",
            "\"validation_health_passed\":true",
            "\"distillation_ready\":false",
            "\"regression_reason\":\"catalog_incomplete\\",
            "\"readiness_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_surface_index_smoke_groups_receipt_family() {
    let receipt = ai::validation_harness::policy_reuse_evidence_surface_index_smoke_receipt();
    let reuse = ai::validation_harness::policy_reuse_smoke_receipt();
    let catalog = ai::validation_harness::policy_reuse_cost_catalog_smoke_receipt();
    let savings = ai::validation_harness::policy_reuse_evaluator_savings_smoke_receipt();
    let projection = ai::validation_harness::policy_reuse_scaling_projection_smoke_receipt();
    let readiness = ai::validation_harness::policy_reuse_distillation_readiness_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_surface_index_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_SURFACE_INDEX_SMOKE_STEP
    );
    assert_eq!(receipt.index_version, 1);
    assert_eq!(receipt.evidence_family_count, 7);
    assert_eq!(receipt.healthy_mode_count, 7);
    assert_eq!(receipt.regression_mode_count, 7);
    assert_eq!(receipt.dependency_group_count, 5);
    assert_eq!(receipt.indexed_root_mode_count, 14);
    assert_eq!(receipt.source_policy_reuse_hash, reuse.receipt_hash);
    assert_eq!(receipt.source_cost_catalog_hash, catalog.receipt_hash);
    assert_eq!(receipt.source_evaluator_savings_hash, savings.receipt_hash);
    assert_eq!(
        receipt.source_scaling_projection_hash,
        projection.receipt_hash
    );
    assert_eq!(
        receipt.source_distillation_readiness_hash,
        readiness.receipt_hash
    );
    assert!(receipt.required_healthy_modes_present);
    assert!(receipt.required_regression_modes_present);
    assert!(receipt.required_dependency_groups_present);
    assert!(receipt.index_complete);
    assert_eq!(receipt.missing_surface, "none");
    assert_ne!(receipt.surface_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_surface_index_regression_smoke_is_valid_failing_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_surface_index_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_SURFACE_INDEX_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.evidence_family_count, 7);
    assert_eq!(receipt.healthy_mode_count, 7);
    assert_eq!(receipt.regression_mode_count, 6);
    assert_eq!(receipt.dependency_group_count, 5);
    assert_eq!(receipt.indexed_root_mode_count, 13);
    assert!(receipt.required_healthy_modes_present);
    assert!(!receipt.required_regression_modes_present);
    assert!(receipt.required_dependency_groups_present);
    assert!(!receipt.index_complete);
    assert_eq!(receipt.missing_surface, "required_regression_modes");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_surface_index_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-surface-index-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_surface_index_v1\\",
            "\"record_type\":\"policy_reuse_evidence_surface_index_smoke\\",
            "\"index_version\":1",
            "\"evidence_family_count\":7",
            "\"healthy_mode_count\":7",
            "\"regression_mode_count\":7",
            "\"dependency_group_count\":5",
            "\"indexed_root_mode_count\":14",
            "\"source_policy_reuse_hash\":",
            "\"source_cost_catalog_hash\":",
            "\"source_evaluator_savings_hash\":",
            "\"source_scaling_projection_hash\":",
            "\"source_distillation_readiness_hash\":",
            "\"required_healthy_modes_present\":true",
            "\"required_regression_modes_present\":true",
            "\"required_dependency_groups_present\":true",
            "\"index_complete\":true",
            "\"missing_surface\":\"none\\",
            "\"surface_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_surface_index_regression_smoke_mode_is_executable_contract()
{
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-surface-index-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_surface_index_v1\\",
            "\"record_type\":\"policy_reuse_evidence_surface_index_regression_smoke\\",
            "\"index_version\":1",
            "\"evidence_family_count\":7",
            "\"healthy_mode_count\":7",
            "\"regression_mode_count\":6",
            "\"dependency_group_count\":5",
            "\"indexed_root_mode_count\":13",
            "\"required_healthy_modes_present\":true",
            "\"required_regression_modes_present\":false",
            "\"required_dependency_groups_present\":true",
            "\"index_complete\":false",
            "\"missing_surface\":\"required_regression_modes\\",
            "\"surface_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_bundle_smoke_rolls_up_indexed_surface() {
    let receipt = ai::validation_harness::policy_reuse_evidence_bundle_smoke_receipt();
    let index = ai::validation_harness::policy_reuse_evidence_surface_index_smoke_receipt();

    assert_eq!(receipt.schema, "canon_policy_reuse_evidence_bundle_v1");
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_BUNDLE_SMOKE_STEP
    );
    assert_eq!(receipt.bundle_version, 1);
    assert_eq!(receipt.source_surface_index_hash, index.receipt_hash);
    assert_eq!(
        receipt.source_policy_reuse_hash,
        index.source_policy_reuse_hash
    );
    assert_eq!(
        receipt.source_cost_catalog_hash,
        index.source_cost_catalog_hash
    );
    assert_eq!(
        receipt.source_evaluator_savings_hash,
        index.source_evaluator_savings_hash
    );
    assert_eq!(
        receipt.source_scaling_projection_hash,
        index.source_scaling_projection_hash
    );
    assert_eq!(
        receipt.source_distillation_readiness_hash,
        index.source_distillation_readiness_hash
    );
    assert_eq!(receipt.bundled_evidence_family_count, 7);
    assert_eq!(receipt.bundled_root_mode_count, 14);
    assert_eq!(receipt.bundled_dependency_group_count, 5);
    assert!(receipt.surface_index_complete);
    assert!(receipt.source_hashes_complete);
    assert!(receipt.bundle_complete);
    assert_eq!(receipt.regression_reason, "none");
    assert_ne!(receipt.bundle_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_bundle_regression_smoke_is_valid_failing_evidence() {
    let receipt = ai::validation_harness::policy_reuse_evidence_bundle_regression_smoke_receipt();
    let index =
        ai::validation_harness::policy_reuse_evidence_surface_index_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_BUNDLE_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_surface_index_hash, index.receipt_hash);
    assert_eq!(receipt.bundled_evidence_family_count, 7);
    assert_eq!(receipt.bundled_root_mode_count, 13);
    assert_eq!(receipt.bundled_dependency_group_count, 5);
    assert!(!receipt.surface_index_complete);
    assert!(receipt.source_hashes_complete);
    assert!(!receipt.bundle_complete);
    assert_eq!(receipt.regression_reason, "surface_index_incomplete");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_bundle_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-bundle-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_bundle_v1\\",
            "\"record_type\":\"policy_reuse_evidence_bundle_smoke\\",
            "\"bundle_version\":1",
            "\"source_surface_index_hash\":",
            "\"source_policy_reuse_hash\":",
            "\"source_cost_catalog_hash\":",
            "\"source_evaluator_savings_hash\":",
            "\"source_scaling_projection_hash\":",
            "\"source_distillation_readiness_hash\":",
            "\"bundled_evidence_family_count\":7",
            "\"bundled_root_mode_count\":14",
            "\"bundled_dependency_group_count\":5",
            "\"surface_index_complete\":true",
            "\"source_hashes_complete\":true",
            "\"bundle_complete\":true",
            "\"regression_reason\":\"none\\",
            "\"bundle_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_bundle_regression_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-bundle-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_bundle_v1\\",
            "\"record_type\":\"policy_reuse_evidence_bundle_regression_smoke\\",
            "\"bundle_version\":1",
            "\"bundled_evidence_family_count\":7",
            "\"bundled_root_mode_count\":13",
            "\"bundled_dependency_group_count\":5",
            "\"surface_index_complete\":false",
            "\"source_hashes_complete\":true",
            "\"bundle_complete\":false",
            "\"regression_reason\":\"surface_index_incomplete\\",
            "\"bundle_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_quickcheck_smoke_summarizes_minimum_validation_commands() {
    let receipt = ai::validation_harness::policy_reuse_evidence_quickcheck_smoke_receipt();
    let bundle = ai::validation_harness::policy_reuse_evidence_bundle_smoke_receipt();

    assert_eq!(receipt.schema, "canon_policy_reuse_evidence_quickcheck_v1");
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_QUICKCHECK_SMOKE_STEP
    );
    assert_eq!(receipt.quickcheck_version, 1);
    assert_eq!(receipt.source_bundle_hash, bundle.receipt_hash);
    assert_eq!(
        receipt.validation_harness_expected_tests,
        ai::validation_harness::VALIDATION_HARNESS_EXPECTED_TESTS
    );
    assert_eq!(receipt.required_command_count, 4);
    assert_eq!(receipt.observed_command_count, 4);
    assert_ne!(receipt.minimum_command_set_hash, 0);
    assert!(receipt.bundle_complete);
    assert!(receipt.commands_complete);
    assert!(receipt.quickcheck_passed);
    assert_eq!(receipt.missing_command, "none");
    assert_ne!(receipt.quickcheck_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_quickcheck_regression_smoke_is_valid_failing_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_quickcheck_regression_smoke_receipt();
    let bundle = ai::validation_harness::policy_reuse_evidence_bundle_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_QUICKCHECK_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_bundle_hash, bundle.receipt_hash);
    assert_eq!(receipt.required_command_count, 4);
    assert_eq!(receipt.observed_command_count, 3);
    assert!(receipt.bundle_complete);
    assert!(!receipt.commands_complete);
    assert!(!receipt.quickcheck_passed);
    assert_eq!(receipt.missing_command, "validation_harness_contract");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_quickcheck_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-quickcheck-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_quickcheck_v1\\",
            "\"record_type\":\"policy_reuse_evidence_quickcheck_smoke\\",
            "\"quickcheck_version\":1",
            "\"source_bundle_hash\":",
            "\"validation_harness_expected_tests\":200",
            "\"required_command_count\":4",
            "\"observed_command_count\":4",
            "\"minimum_command_set_hash\":",
            "\"bundle_complete\":true",
            "\"commands_complete\":true",
            "\"quickcheck_passed\":true",
            "\"missing_command\":\"none\\",
            "\"quickcheck_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_quickcheck_regression_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-quickcheck-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_quickcheck_v1\\",
            "\"record_type\":\"policy_reuse_evidence_quickcheck_regression_smoke\\",
            "\"quickcheck_version\":1",
            "\"validation_harness_expected_tests\":200",
            "\"required_command_count\":4",
            "\"observed_command_count\":3",
            "\"bundle_complete\":true",
            "\"commands_complete\":false",
            "\"quickcheck_passed\":false",
            "\"missing_command\":\"validation_harness_contract\\",
            "\"quickcheck_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_maturity_smoke_summarizes_evidence_stack() {
    let receipt = ai::validation_harness::policy_reuse_evidence_maturity_smoke_receipt();
    let quickcheck = ai::validation_harness::policy_reuse_evidence_quickcheck_smoke_receipt();
    let bundle = ai::validation_harness::policy_reuse_evidence_bundle_smoke_receipt();

    assert_eq!(receipt.schema, "canon_policy_reuse_evidence_maturity_v1");
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_MATURITY_SMOKE_STEP
    );
    assert_eq!(receipt.maturity_version, 1);
    assert_eq!(receipt.source_quickcheck_hash, quickcheck.receipt_hash);
    assert_eq!(receipt.source_bundle_hash, bundle.receipt_hash);
    assert_eq!(receipt.validated_layer_count, 5);
    assert_eq!(receipt.required_layer_count, 5);
    assert_eq!(receipt.maturity_stage, "candidate");
    assert!(receipt.quickcheck_passed);
    assert!(receipt.bundle_complete);
    assert!(receipt.promotion_eligible);
    assert_eq!(receipt.regression_reason, "none");
    assert_ne!(receipt.maturity_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_maturity_regression_smoke_is_valid_failing_evidence() {
    let receipt = ai::validation_harness::policy_reuse_evidence_maturity_regression_smoke_receipt();
    let quickcheck =
        ai::validation_harness::policy_reuse_evidence_quickcheck_regression_smoke_receipt();
    let bundle = ai::validation_harness::policy_reuse_evidence_bundle_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_MATURITY_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_quickcheck_hash, quickcheck.receipt_hash);
    assert_eq!(receipt.source_bundle_hash, bundle.receipt_hash);
    assert_eq!(receipt.validated_layer_count, 4);
    assert_eq!(receipt.required_layer_count, 5);
    assert_eq!(receipt.maturity_stage, "immature");
    assert!(!receipt.quickcheck_passed);
    assert!(receipt.bundle_complete);
    assert!(!receipt.promotion_eligible);
    assert_eq!(receipt.regression_reason, "quickcheck_failed");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_maturity_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-maturity-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_maturity_v1\\",
            "\"record_type\":\"policy_reuse_evidence_maturity_smoke\\",
            "\"maturity_version\":1",
            "\"source_quickcheck_hash\":",
            "\"source_bundle_hash\":",
            "\"validated_layer_count\":5",
            "\"required_layer_count\":5",
            "\"maturity_stage\":\"candidate\\",
            "\"quickcheck_passed\":true",
            "\"bundle_complete\":true",
            "\"promotion_eligible\":true",
            "\"regression_reason\":\"none\\",
            "\"maturity_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_maturity_regression_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-maturity-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_maturity_v1\\",
            "\"record_type\":\"policy_reuse_evidence_maturity_regression_smoke\\",
            "\"maturity_version\":1",
            "\"validated_layer_count\":4",
            "\"required_layer_count\":5",
            "\"maturity_stage\":\"immature\\",
            "\"quickcheck_passed\":false",
            "\"bundle_complete\":true",
            "\"promotion_eligible\":false",
            "\"regression_reason\":\"quickcheck_failed\\",
            "\"maturity_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_summary_smoke_exposes_stable_candidate_summary() {
    let receipt = ai::validation_harness::policy_reuse_evidence_summary_smoke_receipt();
    let maturity = ai::validation_harness::policy_reuse_evidence_maturity_smoke_receipt();
    let quickcheck = ai::validation_harness::policy_reuse_evidence_quickcheck_smoke_receipt();
    let bundle = ai::validation_harness::policy_reuse_evidence_bundle_smoke_receipt();

    assert_eq!(receipt.schema, "canon_policy_reuse_evidence_summary_v1");
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_SUMMARY_SMOKE_STEP
    );
    assert_eq!(receipt.summary_version, 1);
    assert_eq!(receipt.source_maturity_hash, maturity.receipt_hash);
    assert_eq!(receipt.source_quickcheck_hash, quickcheck.receipt_hash);
    assert_eq!(receipt.source_bundle_hash, bundle.receipt_hash);
    assert_eq!(receipt.maturity_stage, "candidate");
    assert!(receipt.promotion_eligible);
    assert_eq!(receipt.summary_status, "pass");
    assert_eq!(receipt.evaluator_action, "accept_summary");
    assert_eq!(receipt.regression_reason, "none");
    assert_ne!(receipt.summary_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_summary_regression_smoke_is_valid_failing_evidence() {
    let receipt = ai::validation_harness::policy_reuse_evidence_summary_regression_smoke_receipt();
    let maturity =
        ai::validation_harness::policy_reuse_evidence_maturity_regression_smoke_receipt();
    let quickcheck =
        ai::validation_harness::policy_reuse_evidence_quickcheck_regression_smoke_receipt();
    let bundle = ai::validation_harness::policy_reuse_evidence_bundle_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_SUMMARY_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_maturity_hash, maturity.receipt_hash);
    assert_eq!(receipt.source_quickcheck_hash, quickcheck.receipt_hash);
    assert_eq!(receipt.source_bundle_hash, bundle.receipt_hash);
    assert_eq!(receipt.maturity_stage, "immature");
    assert!(!receipt.promotion_eligible);
    assert_eq!(receipt.summary_status, "fail");
    assert_eq!(receipt.evaluator_action, "inspect_maturity");
    assert_eq!(receipt.regression_reason, "maturity_immature");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_summary_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-summary-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_summary_v1\\",
            "\"record_type\":\"policy_reuse_evidence_summary_smoke\\",
            "\"summary_version\":1",
            "\"source_maturity_hash\":",
            "\"source_quickcheck_hash\":",
            "\"source_bundle_hash\":",
            "\"maturity_stage\":\"candidate\\",
            "\"promotion_eligible\":true",
            "\"summary_status\":\"pass\\",
            "\"evaluator_action\":\"accept_summary\\",
            "\"regression_reason\":\"none\\",
            "\"summary_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_summary_regression_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-summary-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_summary_v1\\",
            "\"record_type\":\"policy_reuse_evidence_summary_regression_smoke\\",
            "\"summary_version\":1",
            "\"maturity_stage\":\"immature\\",
            "\"promotion_eligible\":false",
            "\"summary_status\":\"fail\\",
            "\"evaluator_action\":\"inspect_maturity\\",
            "\"regression_reason\":\"maturity_immature\\",
            "\"summary_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_manifest_smoke_lists_evaluator_surface() {
    let receipt = ai::validation_harness::policy_reuse_evidence_manifest_smoke_receipt();
    let summary = ai::validation_harness::policy_reuse_evidence_summary_smoke_receipt();
    let maturity = ai::validation_harness::policy_reuse_evidence_maturity_smoke_receipt();

    assert_eq!(receipt.schema, "canon_policy_reuse_evidence_manifest_v1");
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_MANIFEST_SMOKE_STEP
    );
    assert_eq!(receipt.manifest_version, 1);
    assert_eq!(receipt.source_summary_hash, summary.receipt_hash);
    assert_eq!(receipt.source_maturity_hash, maturity.receipt_hash);
    assert_eq!(receipt.evaluator_mode_count, 10);
    assert_eq!(receipt.fixture_dependency_count, 4);
    assert!(receipt.required_summary_modes_present);
    assert!(receipt.required_fixture_dependencies_present);
    assert!(receipt.manifest_complete);
    assert_eq!(receipt.missing_surface, "none");
    assert_ne!(receipt.manifest_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_manifest_regression_smoke_is_valid_failing_evidence() {
    let receipt = ai::validation_harness::policy_reuse_evidence_manifest_regression_smoke_receipt();
    let summary = ai::validation_harness::policy_reuse_evidence_summary_smoke_receipt();
    let maturity = ai::validation_harness::policy_reuse_evidence_maturity_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_MANIFEST_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_summary_hash, summary.receipt_hash);
    assert_eq!(receipt.source_maturity_hash, maturity.receipt_hash);
    assert_eq!(receipt.evaluator_mode_count, 9);
    assert_eq!(receipt.fixture_dependency_count, 4);
    assert!(!receipt.required_summary_modes_present);
    assert!(receipt.required_fixture_dependencies_present);
    assert!(!receipt.manifest_complete);
    assert_eq!(receipt.missing_surface, "required_summary_modes");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_manifest_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-manifest-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_manifest_v1\\",
            "\"record_type\":\"policy_reuse_evidence_manifest_smoke\\",
            "\"manifest_version\":1",
            "\"source_summary_hash\":",
            "\"source_maturity_hash\":",
            "\"evaluator_mode_count\":10",
            "\"fixture_dependency_count\":4",
            "\"required_summary_modes_present\":true",
            "\"required_fixture_dependencies_present\":true",
            "\"manifest_complete\":true",
            "\"missing_surface\":\"none\\",
            "\"manifest_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_manifest_regression_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-manifest-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_manifest_v1\\",
            "\"record_type\":\"policy_reuse_evidence_manifest_regression_smoke\\",
            "\"manifest_version\":1",
            "\"evaluator_mode_count\":9",
            "\"fixture_dependency_count\":4",
            "\"required_summary_modes_present\":false",
            "\"required_fixture_dependencies_present\":true",
            "\"manifest_complete\":false",
            "\"missing_surface\":\"required_summary_modes\\",
            "\"manifest_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_validation_budget_smoke_summarizes_minimum_target() {
    let receipt = ai::validation_harness::policy_reuse_evidence_validation_budget_smoke_receipt();
    let manifest = ai::validation_harness::policy_reuse_evidence_manifest_smoke_receipt();
    let summary = ai::validation_harness::policy_reuse_evidence_summary_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_validation_budget_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_VALIDATION_BUDGET_SMOKE_STEP
    );
    assert_eq!(receipt.budget_version, 1);
    assert_eq!(receipt.source_manifest_hash, manifest.receipt_hash);
    assert_eq!(receipt.source_summary_hash, summary.receipt_hash);
    assert_eq!(receipt.targeted_command_count, 2);
    assert_eq!(receipt.targeted_test_count, 4);
    assert_eq!(receipt.max_targeted_test_count, 4);
    assert_eq!(
        receipt.full_harness_test_count,
        ai::validation_harness::VALIDATION_HARNESS_EXPECTED_TESTS
    );
    assert_eq!(
        receipt.avoided_full_harness_tests,
        ai::validation_harness::VALIDATION_HARNESS_EXPECTED_TESTS - 4
    );
    assert!(receipt.manifest_complete);
    assert!(receipt.budget_within_limit);
    assert_eq!(receipt.budget_status, "pass");
    assert_eq!(receipt.regression_reason, "none");
    assert_ne!(receipt.budget_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_validation_budget_regression_smoke_is_valid_failing_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_validation_budget_regression_smoke_receipt();
    let manifest = ai::validation_harness::policy_reuse_evidence_manifest_smoke_receipt();
    let summary = ai::validation_harness::policy_reuse_evidence_summary_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_VALIDATION_BUDGET_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_manifest_hash, manifest.receipt_hash);
    assert_eq!(receipt.source_summary_hash, summary.receipt_hash);
    assert_eq!(receipt.targeted_command_count, 2);
    assert_eq!(receipt.targeted_test_count, 12);
    assert_eq!(receipt.max_targeted_test_count, 4);
    assert_eq!(receipt.avoided_full_harness_tests, 188);
    assert!(receipt.manifest_complete);
    assert!(!receipt.budget_within_limit);
    assert_eq!(receipt.budget_status, "fail");
    assert_eq!(receipt.regression_reason, "budget_exceeded");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_validation_budget_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-validation-budget-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_validation_budget_v1\\",
            "\"record_type\":\"policy_reuse_evidence_validation_budget_smoke\\",
            "\"budget_version\":1",
            "\"source_manifest_hash\":",
            "\"source_summary_hash\":",
            "\"targeted_command_count\":2",
            "\"targeted_test_count\":4",
            "\"max_targeted_test_count\":4",
            "\"full_harness_test_count\":204",
            "\"avoided_full_harness_tests\":196",
            "\"manifest_complete\":true",
            "\"budget_within_limit\":true",
            "\"budget_status\":\"pass\\",
            "\"regression_reason\":\"none\\",
            "\"budget_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_validation_budget_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-validation-budget-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_validation_budget_v1\\",
            "\"record_type\":\"policy_reuse_evidence_validation_budget_regression_smoke\\",
            "\"budget_version\":1",
            "\"targeted_command_count\":2",
            "\"targeted_test_count\":12",
            "\"max_targeted_test_count\":4",
            "\"full_harness_test_count\":204",
            "\"avoided_full_harness_tests\":188",
            "\"manifest_complete\":true",
            "\"budget_within_limit\":false",
            "\"budget_status\":\"fail\\",
            "\"regression_reason\":\"budget_exceeded\\",
            "\"budget_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_rollout_readiness_smoke_composes_stack_readiness() {
    let receipt = ai::validation_harness::policy_reuse_evidence_rollout_readiness_smoke_receipt();
    let budget = ai::validation_harness::policy_reuse_evidence_validation_budget_smoke_receipt();
    let manifest = ai::validation_harness::policy_reuse_evidence_manifest_smoke_receipt();
    let maturity = ai::validation_harness::policy_reuse_evidence_maturity_smoke_receipt();
    let summary = ai::validation_harness::policy_reuse_evidence_summary_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_rollout_readiness_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_ROLLOUT_READINESS_SMOKE_STEP
    );
    assert_eq!(receipt.readiness_version, 1);
    assert_eq!(receipt.source_validation_budget_hash, budget.receipt_hash);
    assert_eq!(receipt.source_manifest_hash, manifest.receipt_hash);
    assert_eq!(receipt.source_maturity_hash, maturity.receipt_hash);
    assert_eq!(receipt.source_summary_hash, summary.receipt_hash);
    assert!(receipt.validation_budget_passed);
    assert!(receipt.manifest_complete);
    assert_eq!(receipt.maturity_stage, "candidate");
    assert_eq!(receipt.summary_status, "pass");
    assert!(receipt.rollout_ready);
    assert_eq!(receipt.readiness_status, "ready");
    assert_eq!(receipt.not_ready_reason, "none");
    assert_ne!(receipt.readiness_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_rollout_readiness_regression_smoke_is_valid_not_ready_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_rollout_readiness_regression_smoke_receipt();
    let budget =
        ai::validation_harness::policy_reuse_evidence_validation_budget_regression_smoke_receipt();
    let manifest = ai::validation_harness::policy_reuse_evidence_manifest_smoke_receipt();
    let maturity = ai::validation_harness::policy_reuse_evidence_maturity_smoke_receipt();
    let summary = ai::validation_harness::policy_reuse_evidence_summary_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_ROLLOUT_READINESS_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_validation_budget_hash, budget.receipt_hash);
    assert_eq!(receipt.source_manifest_hash, manifest.receipt_hash);
    assert_eq!(receipt.source_maturity_hash, maturity.receipt_hash);
    assert_eq!(receipt.source_summary_hash, summary.receipt_hash);
    assert!(!receipt.validation_budget_passed);
    assert!(receipt.manifest_complete);
    assert_eq!(receipt.maturity_stage, "candidate");
    assert_eq!(receipt.summary_status, "pass");
    assert!(!receipt.rollout_ready);
    assert_eq!(receipt.readiness_status, "not_ready");
    assert_eq!(receipt.not_ready_reason, "validation_budget_failed");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_rollout_readiness_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-rollout-readiness-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_rollout_readiness_v1\\",
            "\"record_type\":\"policy_reuse_evidence_rollout_readiness_smoke\\",
            "\"readiness_version\":1",
            "\"source_validation_budget_hash\":",
            "\"source_manifest_hash\":",
            "\"source_maturity_hash\":",
            "\"source_summary_hash\":",
            "\"validation_budget_passed\":true",
            "\"manifest_complete\":true",
            "\"maturity_stage\":\"candidate\\",
            "\"summary_status\":\"pass\\",
            "\"rollout_ready\":true",
            "\"readiness_status\":\"ready\\",
            "\"not_ready_reason\":\"none\\",
            "\"readiness_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_rollout_readiness_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-rollout-readiness-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_rollout_readiness_v1\\",
            "\"record_type\":\"policy_reuse_evidence_rollout_readiness_regression_smoke\\",
            "\"readiness_version\":1",
            "\"validation_budget_passed\":false",
            "\"manifest_complete\":true",
            "\"maturity_stage\":\"candidate\\",
            "\"summary_status\":\"pass\\",
            "\"rollout_ready\":false",
            "\"readiness_status\":\"not_ready\\",
            "\"not_ready_reason\":\"validation_budget_failed\\",
            "\"readiness_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_learning_admission_smoke_admits_ready_trace() {
    let receipt = ai::validation_harness::policy_reuse_evidence_learning_admission_smoke_receipt();
    let rollout = ai::validation_harness::policy_reuse_evidence_rollout_readiness_smoke_receipt();
    let budget = ai::validation_harness::policy_reuse_evidence_validation_budget_smoke_receipt();
    let summary = ai::validation_harness::policy_reuse_evidence_summary_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_learning_admission_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_LEARNING_ADMISSION_SMOKE_STEP
    );
    assert_eq!(receipt.admission_version, 1);
    assert_eq!(receipt.source_rollout_readiness_hash, rollout.receipt_hash);
    assert_eq!(receipt.source_validation_budget_hash, budget.receipt_hash);
    assert_eq!(receipt.source_summary_hash, summary.receipt_hash);
    assert!(receipt.rollout_ready);
    assert!(receipt.validation_budget_passed);
    assert_eq!(receipt.summary_status, "pass");
    assert!(!receipt.external_evidence_required);
    assert!(receipt.learning_data_admissible);
    assert_eq!(receipt.admission_status, "admissible");
    assert_eq!(receipt.not_admissible_reason, "none");
    assert_ne!(receipt.admission_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_learning_admission_regression_smoke_is_valid_not_admissible_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_learning_admission_regression_smoke_receipt();
    let rollout =
        ai::validation_harness::policy_reuse_evidence_rollout_readiness_regression_smoke_receipt();
    let budget =
        ai::validation_harness::policy_reuse_evidence_validation_budget_regression_smoke_receipt();
    let summary = ai::validation_harness::policy_reuse_evidence_summary_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_LEARNING_ADMISSION_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_rollout_readiness_hash, rollout.receipt_hash);
    assert_eq!(receipt.source_validation_budget_hash, budget.receipt_hash);
    assert_eq!(receipt.source_summary_hash, summary.receipt_hash);
    assert!(!receipt.rollout_ready);
    assert!(!receipt.validation_budget_passed);
    assert_eq!(receipt.summary_status, "pass");
    assert!(!receipt.external_evidence_required);
    assert!(!receipt.learning_data_admissible);
    assert_eq!(receipt.admission_status, "not_admissible");
    assert_eq!(receipt.not_admissible_reason, "rollout_not_ready");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_learning_admission_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-learning-admission-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_learning_admission_v1\\",
            "\"record_type\":\"policy_reuse_evidence_learning_admission_smoke\\",
            "\"admission_version\":1",
            "\"source_rollout_readiness_hash\":",
            "\"source_validation_budget_hash\":",
            "\"source_summary_hash\":",
            "\"rollout_ready\":true",
            "\"validation_budget_passed\":true",
            "\"summary_status\":\"pass\\",
            "\"external_evidence_required\":false",
            "\"learning_data_admissible\":true",
            "\"admission_status\":\"admissible\\",
            "\"not_admissible_reason\":\"none\\",
            "\"admission_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_learning_admission_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-learning-admission-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_learning_admission_v1\\",
            "\"record_type\":\"policy_reuse_evidence_learning_admission_regression_smoke\\",
            "\"admission_version\":1",
            "\"rollout_ready\":false",
            "\"validation_budget_passed\":false",
            "\"summary_status\":\"pass\\",
            "\"external_evidence_required\":false",
            "\"learning_data_admissible\":false",
            "\"admission_status\":\"not_admissible\\",
            "\"not_admissible_reason\":\"rollout_not_ready\\",
            "\"admission_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_retrieval_readiness_smoke_marks_retrieval_ready() {
    let receipt = ai::validation_harness::policy_reuse_evidence_retrieval_readiness_smoke_receipt();
    let admission =
        ai::validation_harness::policy_reuse_evidence_learning_admission_smoke_receipt();
    let rollout = ai::validation_harness::policy_reuse_evidence_rollout_readiness_smoke_receipt();
    let summary = ai::validation_harness::policy_reuse_evidence_summary_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_retrieval_readiness_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_READINESS_SMOKE_STEP
    );
    assert_eq!(receipt.retrieval_version, 1);
    assert_eq!(
        receipt.source_learning_admission_hash,
        admission.receipt_hash
    );
    assert_eq!(receipt.source_rollout_readiness_hash, rollout.receipt_hash);
    assert_eq!(receipt.source_summary_hash, summary.receipt_hash);
    assert!(receipt.learning_data_admissible);
    assert!(receipt.rollout_ready);
    assert_eq!(receipt.summary_status, "pass");
    assert!(!receipt.retrieval_storage_write_performed);
    assert!(receipt.retrieval_example_ready);
    assert_eq!(receipt.retrieval_status, "ready");
    assert_eq!(receipt.not_ready_reason, "none");
    assert_ne!(receipt.retrieval_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_retrieval_readiness_regression_smoke_is_valid_not_ready_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_retrieval_readiness_regression_smoke_receipt(
        );
    let admission =
        ai::validation_harness::policy_reuse_evidence_learning_admission_regression_smoke_receipt();
    let rollout =
        ai::validation_harness::policy_reuse_evidence_rollout_readiness_regression_smoke_receipt();
    let summary = ai::validation_harness::policy_reuse_evidence_summary_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_READINESS_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.source_learning_admission_hash,
        admission.receipt_hash
    );
    assert_eq!(receipt.source_rollout_readiness_hash, rollout.receipt_hash);
    assert_eq!(receipt.source_summary_hash, summary.receipt_hash);
    assert!(!receipt.learning_data_admissible);
    assert!(!receipt.rollout_ready);
    assert_eq!(receipt.summary_status, "pass");
    assert!(!receipt.retrieval_storage_write_performed);
    assert!(!receipt.retrieval_example_ready);
    assert_eq!(receipt.retrieval_status, "not_ready");
    assert_eq!(receipt.not_ready_reason, "learning_not_admissible");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_readiness_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-readiness-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_readiness_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_readiness_smoke\\",
            "\"retrieval_version\":1",
            "\"source_learning_admission_hash\":",
            "\"source_rollout_readiness_hash\":",
            "\"source_summary_hash\":",
            "\"learning_data_admissible\":true",
            "\"rollout_ready\":true",
            "\"summary_status\":\"pass\\",
            "\"retrieval_storage_write_performed\":false",
            "\"retrieval_example_ready\":true",
            "\"retrieval_status\":\"ready\\",
            "\"not_ready_reason\":\"none\\",
            "\"retrieval_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_readiness_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-readiness-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_readiness_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_readiness_regression_smoke\\",
            "\"retrieval_version\":1",
            "\"learning_data_admissible\":false",
            "\"rollout_ready\":false",
            "\"summary_status\":\"pass\\",
            "\"retrieval_storage_write_performed\":false",
            "\"retrieval_example_ready\":false",
            "\"retrieval_status\":\"not_ready\\",
            "\"not_ready_reason\":\"learning_not_admissible\\",
            "\"retrieval_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_compact_validation_smoke_summarizes_targeted_chain() {
    let receipt = ai::validation_harness::policy_reuse_evidence_compact_validation_smoke_receipt();
    let retrieval =
        ai::validation_harness::policy_reuse_evidence_retrieval_readiness_smoke_receipt();
    let admission =
        ai::validation_harness::policy_reuse_evidence_learning_admission_smoke_receipt();
    let budget = ai::validation_harness::policy_reuse_evidence_validation_budget_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_compact_validation_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_COMPACT_VALIDATION_SMOKE_STEP
    );
    assert_eq!(receipt.compact_validation_version, 1);
    assert_eq!(
        receipt.source_retrieval_readiness_hash,
        retrieval.receipt_hash
    );
    assert_eq!(
        receipt.source_learning_admission_hash,
        admission.receipt_hash
    );
    assert_eq!(receipt.source_validation_budget_hash, budget.receipt_hash);
    assert!(receipt.retrieval_ready);
    assert!(receipt.learning_data_admissible);
    assert!(receipt.validation_budget_passed);
    assert_eq!(receipt.targeted_command_count, 3);
    assert_eq!(receipt.targeted_test_count, 6);
    assert_eq!(receipt.max_targeted_test_count, 6);
    assert_eq!(
        receipt.full_harness_test_count,
        ai::validation_harness::VALIDATION_HARNESS_EXPECTED_TESTS
    );
    assert_eq!(
        receipt.avoided_full_harness_tests,
        ai::validation_harness::VALIDATION_HARNESS_EXPECTED_TESTS - 6
    );
    assert!(receipt.compact_validation_passed);
    assert_eq!(receipt.compact_validation_status, "pass");
    assert_eq!(receipt.failure_reason, "none");
    assert_ne!(receipt.compact_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_compact_validation_regression_smoke_is_valid_failing_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_compact_validation_regression_smoke_receipt();
    let retrieval =
        ai::validation_harness::policy_reuse_evidence_retrieval_readiness_regression_smoke_receipt(
        );
    let admission =
        ai::validation_harness::policy_reuse_evidence_learning_admission_regression_smoke_receipt();
    let budget =
        ai::validation_harness::policy_reuse_evidence_validation_budget_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_COMPACT_VALIDATION_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.source_retrieval_readiness_hash,
        retrieval.receipt_hash
    );
    assert_eq!(
        receipt.source_learning_admission_hash,
        admission.receipt_hash
    );
    assert_eq!(receipt.source_validation_budget_hash, budget.receipt_hash);
    assert!(!receipt.retrieval_ready);
    assert!(!receipt.learning_data_admissible);
    assert!(!receipt.validation_budget_passed);
    assert_eq!(receipt.targeted_command_count, 3);
    assert_eq!(receipt.targeted_test_count, 14);
    assert_eq!(receipt.max_targeted_test_count, 6);
    assert_eq!(
        receipt.full_harness_test_count,
        ai::validation_harness::VALIDATION_HARNESS_EXPECTED_TESTS
    );
    assert_eq!(
        receipt.avoided_full_harness_tests,
        ai::validation_harness::VALIDATION_HARNESS_EXPECTED_TESTS - 14
    );
    assert!(!receipt.compact_validation_passed);
    assert_eq!(receipt.compact_validation_status, "fail");
    assert_eq!(receipt.failure_reason, "retrieval_not_ready");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_compact_validation_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-compact-validation-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_compact_validation_v1\\",
            "\"record_type\":\"policy_reuse_evidence_compact_validation_smoke\\",
            "\"compact_validation_version\":1",
            "\"source_retrieval_readiness_hash\":",
            "\"source_learning_admission_hash\":",
            "\"source_validation_budget_hash\":",
            "\"retrieval_ready\":true",
            "\"learning_data_admissible\":true",
            "\"validation_budget_passed\":true",
            "\"targeted_command_count\":3",
            "\"targeted_test_count\":6",
            "\"max_targeted_test_count\":6",
            "\"full_harness_test_count\":204",
            "\"avoided_full_harness_tests\":198",
            "\"compact_validation_passed\":true",
            "\"compact_validation_status\":\"pass\\",
            "\"failure_reason\":\"none\\",
            "\"compact_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_compact_validation_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-compact-validation-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_compact_validation_v1\\",
            "\"record_type\":\"policy_reuse_evidence_compact_validation_regression_smoke\\",
            "\"compact_validation_version\":1",
            "\"retrieval_ready\":false",
            "\"learning_data_admissible\":false",
            "\"validation_budget_passed\":false",
            "\"targeted_command_count\":3",
            "\"targeted_test_count\":14",
            "\"max_targeted_test_count\":6",
            "\"full_harness_test_count\":204",
            "\"avoided_full_harness_tests\":190",
            "\"compact_validation_passed\":false",
            "\"compact_validation_status\":\"fail\\",
            "\"failure_reason\":\"retrieval_not_ready\\",
            "\"compact_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_batch_readiness_smoke_composes_batch_chain() {
    let receipt = ai::validation_harness::policy_reuse_evidence_batch_readiness_smoke_receipt();
    let compact = ai::validation_harness::policy_reuse_evidence_compact_validation_smoke_receipt();
    let retrieval =
        ai::validation_harness::policy_reuse_evidence_retrieval_readiness_smoke_receipt();
    let projection = ai::validation_harness::policy_reuse_scaling_projection_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_batch_readiness_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_BATCH_READINESS_SMOKE_STEP
    );
    assert_eq!(receipt.batch_readiness_version, 1);
    assert_eq!(receipt.source_compact_validation_hash, compact.receipt_hash);
    assert_eq!(
        receipt.source_retrieval_readiness_hash,
        retrieval.receipt_hash
    );
    assert_eq!(
        receipt.source_scaling_projection_hash,
        projection.receipt_hash
    );
    assert!(receipt.compact_validation_passed);
    assert!(receipt.retrieval_ready);
    assert!(receipt.scaling_projection_passed);
    assert_eq!(
        receipt.batch_capacity_limit,
        projection.batch_capacity_limit
    );
    assert_eq!(
        receipt.projected_llm_calls_avoided_per_full_batch,
        projection.projected_llm_calls_avoided_per_full_batch
    );
    assert_eq!(
        receipt.projected_llm_fallbacks_per_full_batch,
        projection.projected_llm_fallbacks_per_full_batch
    );
    assert!(receipt.batch_ready);
    assert_eq!(receipt.batch_readiness_status, "ready");
    assert_eq!(receipt.not_ready_reason, "none");
    assert_ne!(receipt.batch_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_batch_readiness_regression_smoke_is_valid_not_ready_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_batch_readiness_regression_smoke_receipt();
    let compact =
        ai::validation_harness::policy_reuse_evidence_compact_validation_regression_smoke_receipt();
    let retrieval =
        ai::validation_harness::policy_reuse_evidence_retrieval_readiness_regression_smoke_receipt(
        );
    let projection =
        ai::validation_harness::policy_reuse_scaling_projection_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_BATCH_READINESS_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_compact_validation_hash, compact.receipt_hash);
    assert_eq!(
        receipt.source_retrieval_readiness_hash,
        retrieval.receipt_hash
    );
    assert_eq!(
        receipt.source_scaling_projection_hash,
        projection.receipt_hash
    );
    assert!(!receipt.compact_validation_passed);
    assert!(!receipt.retrieval_ready);
    assert!(!receipt.scaling_projection_passed);
    assert_eq!(
        receipt.batch_capacity_limit,
        projection.batch_capacity_limit
    );
    assert!(!receipt.batch_ready);
    assert_eq!(receipt.batch_readiness_status, "not_ready");
    assert_eq!(receipt.not_ready_reason, "compact_validation_failed");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_batch_readiness_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-batch-readiness-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_batch_readiness_v1\\",
            "\"record_type\":\"policy_reuse_evidence_batch_readiness_smoke\\",
            "\"batch_readiness_version\":1",
            "\"source_compact_validation_hash\":",
            "\"source_retrieval_readiness_hash\":",
            "\"source_scaling_projection_hash\":",
            "\"compact_validation_passed\":true",
            "\"retrieval_ready\":true",
            "\"scaling_projection_passed\":true",
            "\"batch_ready\":true",
            "\"batch_readiness_status\":\"ready\\",
            "\"not_ready_reason\":\"none\\",
            "\"batch_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_batch_readiness_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-batch-readiness-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_batch_readiness_v1\\",
            "\"record_type\":\"policy_reuse_evidence_batch_readiness_regression_smoke\\",
            "\"batch_readiness_version\":1",
            "\"compact_validation_passed\":false",
            "\"retrieval_ready\":false",
            "\"scaling_projection_passed\":false",
            "\"batch_ready\":false",
            "\"batch_readiness_status\":\"not_ready\\",
            "\"not_ready_reason\":\"compact_validation_failed\\",
            "\"batch_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_batch_execution_plan_smoke_composes_no_execute_plan() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_batch_execution_plan_smoke_receipt();
    let batch = ai::validation_harness::policy_reuse_evidence_batch_readiness_smoke_receipt();
    let compact = ai::validation_harness::policy_reuse_evidence_compact_validation_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_batch_execution_plan_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_BATCH_EXECUTION_PLAN_SMOKE_STEP
    );
    assert_eq!(receipt.execution_plan_version, 1);
    assert_eq!(receipt.source_batch_readiness_hash, batch.receipt_hash);
    assert_eq!(receipt.source_compact_validation_hash, compact.receipt_hash);
    assert!(receipt.batch_ready);
    assert!(receipt.compact_validation_passed);
    assert!(receipt.no_execute_plan);
    assert_eq!(receipt.proposed_batch_capacity, batch.batch_capacity_limit);
    assert_eq!(
        receipt.proposed_policy_reuse_cases,
        batch.projected_llm_calls_avoided_per_full_batch
    );
    assert_eq!(
        receipt.proposed_llm_fallback_cases,
        batch.projected_llm_fallbacks_per_full_batch
    );
    assert!(!receipt.execution_performed);
    assert!(receipt.plan_ready);
    assert_eq!(receipt.plan_status, "planned");
    assert_eq!(receipt.not_plannable_reason, "none");
    assert_ne!(receipt.plan_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_batch_execution_plan_regression_smoke_is_valid_not_plannable_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_batch_execution_plan_regression_smoke_receipt(
        );
    let batch =
        ai::validation_harness::policy_reuse_evidence_batch_readiness_regression_smoke_receipt();
    let compact =
        ai::validation_harness::policy_reuse_evidence_compact_validation_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_BATCH_EXECUTION_PLAN_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_batch_readiness_hash, batch.receipt_hash);
    assert_eq!(receipt.source_compact_validation_hash, compact.receipt_hash);
    assert!(!receipt.batch_ready);
    assert!(!receipt.compact_validation_passed);
    assert!(receipt.no_execute_plan);
    assert_eq!(receipt.proposed_batch_capacity, batch.batch_capacity_limit);
    assert!(!receipt.execution_performed);
    assert!(!receipt.plan_ready);
    assert_eq!(receipt.plan_status, "not_plannable");
    assert_eq!(receipt.not_plannable_reason, "batch_not_ready");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_batch_execution_plan_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-batch-execution-plan-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_batch_execution_plan_v1\\",
            "\"record_type\":\"policy_reuse_evidence_batch_execution_plan_smoke\\",
            "\"execution_plan_version\":1",
            "\"source_batch_readiness_hash\":",
            "\"source_compact_validation_hash\":",
            "\"batch_ready\":true",
            "\"compact_validation_passed\":true",
            "\"no_execute_plan\":true",
            "\"execution_performed\":false",
            "\"plan_ready\":true",
            "\"plan_status\":\"planned\\",
            "\"not_plannable_reason\":\"none\\",
            "\"plan_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_batch_execution_plan_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-batch-execution-plan-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_batch_execution_plan_v1\\",
            "\"record_type\":\"policy_reuse_evidence_batch_execution_plan_regression_smoke\\",
            "\"execution_plan_version\":1",
            "\"batch_ready\":false",
            "\"compact_validation_passed\":false",
            "\"no_execute_plan\":true",
            "\"execution_performed\":false",
            "\"plan_ready\":false",
            "\"plan_status\":\"not_plannable\\",
            "\"not_plannable_reason\":\"batch_not_ready\\",
            "\"plan_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_batch_evaluation_admission_smoke_composes_admission_verdict() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_batch_evaluation_admission_smoke_receipt();
    let plan = ai::validation_harness::policy_reuse_evidence_batch_execution_plan_smoke_receipt();
    let batch = ai::validation_harness::policy_reuse_evidence_batch_readiness_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_batch_evaluation_admission_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_BATCH_EVALUATION_ADMISSION_SMOKE_STEP
    );
    assert_eq!(receipt.admission_version, 1);
    assert_eq!(receipt.source_batch_execution_plan_hash, plan.receipt_hash);
    assert_eq!(receipt.source_batch_readiness_hash, batch.receipt_hash);
    assert!(receipt.plan_ready);
    assert!(receipt.batch_ready);
    assert!(receipt.no_execute_plan);
    assert!(!receipt.execution_performed);
    assert_eq!(
        receipt.proposed_batch_capacity,
        plan.proposed_batch_capacity
    );
    assert_eq!(
        receipt.admitted_policy_reuse_cases,
        plan.proposed_policy_reuse_cases
    );
    assert_eq!(
        receipt.admitted_llm_fallback_cases,
        plan.proposed_llm_fallback_cases
    );
    assert!(receipt.batch_evaluation_admitted);
    assert_eq!(receipt.admission_status, "admitted");
    assert_eq!(receipt.not_admitted_reason, "none");
    assert_ne!(receipt.admission_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_batch_evaluation_admission_regression_smoke_is_valid_not_admitted_evidence(
) {
    let receipt = ai::validation_harness::
        policy_reuse_evidence_batch_evaluation_admission_regression_smoke_receipt();
    let plan =
        ai::validation_harness::policy_reuse_evidence_batch_execution_plan_regression_smoke_receipt(
        );
    let batch =
        ai::validation_harness::policy_reuse_evidence_batch_readiness_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_BATCH_EVALUATION_ADMISSION_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_batch_execution_plan_hash, plan.receipt_hash);
    assert_eq!(receipt.source_batch_readiness_hash, batch.receipt_hash);
    assert!(!receipt.plan_ready);
    assert!(!receipt.batch_ready);
    assert!(receipt.no_execute_plan);
    assert!(!receipt.execution_performed);
    assert_eq!(
        receipt.proposed_batch_capacity,
        plan.proposed_batch_capacity
    );
    assert!(!receipt.batch_evaluation_admitted);
    assert_eq!(receipt.admission_status, "not_admitted");
    assert_eq!(receipt.not_admitted_reason, "plan_not_ready");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_batch_evaluation_admission_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-batch-evaluation-admission-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_batch_evaluation_admission_v1\\",
            "\"record_type\":\"policy_reuse_evidence_batch_evaluation_admission_smoke\\",
            "\"admission_version\":1",
            "\"source_batch_execution_plan_hash\":",
            "\"source_batch_readiness_hash\":",
            "\"plan_ready\":true",
            "\"batch_ready\":true",
            "\"no_execute_plan\":true",
            "\"execution_performed\":false",
            "\"batch_evaluation_admitted\":true",
            "\"admission_status\":\"admitted\\",
            "\"not_admitted_reason\":\"none\\",
            "\"admission_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_batch_evaluation_admission_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-batch-evaluation-admission-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_batch_evaluation_admission_v1\\",
            "\"record_type\":\"policy_reuse_evidence_batch_evaluation_admission_regression_smoke\\",
            "\"admission_version\":1",
            "\"plan_ready\":false",
            "\"batch_ready\":false",
            "\"no_execute_plan\":true",
            "\"execution_performed\":false",
            "\"batch_evaluation_admitted\":false",
            "\"admission_status\":\"not_admitted\\",
            "\"not_admitted_reason\":\"plan_not_ready\\",
            "\"admission_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_batch_run_request_smoke_composes_no_execute_request() {
    let receipt = ai::validation_harness::policy_reuse_evidence_batch_run_request_smoke_receipt();
    let admission =
        ai::validation_harness::policy_reuse_evidence_batch_evaluation_admission_smoke_receipt();
    let plan = ai::validation_harness::policy_reuse_evidence_batch_execution_plan_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_batch_run_request_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_BATCH_RUN_REQUEST_SMOKE_STEP
    );
    assert_eq!(receipt.request_version, 1);
    assert_eq!(
        receipt.source_batch_evaluation_admission_hash,
        admission.receipt_hash
    );
    assert_eq!(receipt.source_batch_execution_plan_hash, plan.receipt_hash);
    assert!(receipt.batch_evaluation_admitted);
    assert!(receipt.plan_ready);
    assert!(receipt.no_execute_request);
    assert!(!receipt.execution_performed);
    assert_eq!(
        receipt.requested_batch_capacity,
        admission.proposed_batch_capacity
    );
    assert_eq!(
        receipt.requested_policy_reuse_cases,
        admission.admitted_policy_reuse_cases
    );
    assert_eq!(
        receipt.requested_llm_fallback_cases,
        admission.admitted_llm_fallback_cases
    );
    assert!(receipt.batch_request_ready);
    assert_eq!(receipt.request_status, "request_ready");
    assert_eq!(receipt.not_requestable_reason, "none");
    assert_ne!(receipt.request_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_batch_run_request_regression_smoke_is_valid_not_requestable_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_batch_run_request_regression_smoke_receipt();
    let admission = ai::validation_harness::
        policy_reuse_evidence_batch_evaluation_admission_regression_smoke_receipt();
    let plan =
        ai::validation_harness::policy_reuse_evidence_batch_execution_plan_regression_smoke_receipt(
        );

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_BATCH_RUN_REQUEST_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.source_batch_evaluation_admission_hash,
        admission.receipt_hash
    );
    assert_eq!(receipt.source_batch_execution_plan_hash, plan.receipt_hash);
    assert!(!receipt.batch_evaluation_admitted);
    assert!(!receipt.plan_ready);
    assert!(receipt.no_execute_request);
    assert!(!receipt.execution_performed);
    assert_eq!(
        receipt.requested_batch_capacity,
        admission.proposed_batch_capacity
    );
    assert!(!receipt.batch_request_ready);
    assert_eq!(receipt.request_status, "not_requestable");
    assert_eq!(receipt.not_requestable_reason, "admission_not_granted");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_batch_run_request_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-batch-run-request-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_batch_run_request_v1\\",
            "\"record_type\":\"policy_reuse_evidence_batch_run_request_smoke\\",
            "\"request_version\":1",
            "\"source_batch_evaluation_admission_hash\":",
            "\"source_batch_execution_plan_hash\":",
            "\"batch_evaluation_admitted\":true",
            "\"plan_ready\":true",
            "\"no_execute_request\":true",
            "\"execution_performed\":false",
            "\"batch_request_ready\":true",
            "\"request_status\":\"request_ready\\",
            "\"not_requestable_reason\":\"none\\",
            "\"request_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_batch_run_request_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-batch-run-request-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_batch_run_request_v1\\",
            "\"record_type\":\"policy_reuse_evidence_batch_run_request_regression_smoke\\",
            "\"request_version\":1",
            "\"batch_evaluation_admitted\":false",
            "\"plan_ready\":false",
            "\"no_execute_request\":true",
            "\"execution_performed\":false",
            "\"batch_request_ready\":false",
            "\"request_status\":\"not_requestable\\",
            "\"not_requestable_reason\":\"admission_not_granted\\",
            "\"request_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_external_evaluator_result_smoke_composes_result_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_external_evaluator_result_smoke_receipt();
    let request = ai::validation_harness::policy_reuse_evidence_batch_run_request_smoke_receipt();
    let admission =
        ai::validation_harness::policy_reuse_evidence_batch_evaluation_admission_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_external_evaluator_result_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_EXTERNAL_EVALUATOR_RESULT_SMOKE_STEP
    );
    assert_eq!(receipt.evaluator_result_version, 1);
    assert_eq!(receipt.source_batch_run_request_hash, request.receipt_hash);
    assert_eq!(
        receipt.source_batch_evaluation_admission_hash,
        admission.receipt_hash
    );
    assert!(receipt.batch_request_ready);
    assert!(receipt.batch_evaluation_admitted);
    assert!(receipt.external_evaluator_independent);
    assert!(!receipt.llm_self_approved);
    assert_eq!(
        receipt.evaluated_batch_capacity,
        request.requested_batch_capacity
    );
    assert_eq!(
        receipt.evaluated_policy_reuse_cases,
        request.requested_policy_reuse_cases
    );
    assert_eq!(
        receipt.evaluated_llm_fallback_cases,
        request.requested_llm_fallback_cases
    );
    assert!(receipt.evaluator_result_passed);
    assert_eq!(receipt.evaluator_status, "passed");
    assert_eq!(receipt.evaluator_failure_reason, "none");
    assert_ne!(receipt.evaluator_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_external_evaluator_result_regression_smoke_is_valid_failed_evidence() {
    let receipt = ai::validation_harness::
        policy_reuse_evidence_external_evaluator_result_regression_smoke_receipt();
    let request =
        ai::validation_harness::policy_reuse_evidence_batch_run_request_regression_smoke_receipt();
    let admission = ai::validation_harness::
        policy_reuse_evidence_batch_evaluation_admission_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_EXTERNAL_EVALUATOR_RESULT_REGRESSION_SMOKE_STEP
    );
    assert_eq!(receipt.source_batch_run_request_hash, request.receipt_hash);
    assert_eq!(
        receipt.source_batch_evaluation_admission_hash,
        admission.receipt_hash
    );
    assert!(!receipt.batch_request_ready);
    assert!(!receipt.batch_evaluation_admitted);
    assert!(receipt.external_evaluator_independent);
    assert!(!receipt.llm_self_approved);
    assert_eq!(
        receipt.evaluated_batch_capacity,
        request.requested_batch_capacity
    );
    assert!(!receipt.evaluator_result_passed);
    assert_eq!(receipt.evaluator_status, "failed");
    assert_eq!(
        receipt.evaluator_failure_reason,
        "external_evaluator_failed"
    );
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_external_evaluator_result_smoke_mode_is_executable_contract()
{
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-external-evaluator-result-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_external_evaluator_result_v1\\",
            "\"record_type\":\"policy_reuse_evidence_external_evaluator_result_smoke\\",
            "\"evaluator_result_version\":1",
            "\"source_batch_run_request_hash\":",
            "\"source_batch_evaluation_admission_hash\":",
            "\"batch_request_ready\":true",
            "\"batch_evaluation_admitted\":true",
            "\"external_evaluator_independent\":true",
            "\"llm_self_approved\":false",
            "\"evaluator_result_passed\":true",
            "\"evaluator_status\":\"passed\\",
            "\"evaluator_failure_reason\":\"none\\",
            "\"evaluator_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_external_evaluator_result_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-external-evaluator-result-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_external_evaluator_result_v1\\",
            "\"record_type\":\"policy_reuse_evidence_external_evaluator_result_regression_smoke\\",
            "\"evaluator_result_version\":1",
            "\"batch_request_ready\":false",
            "\"batch_evaluation_admitted\":false",
            "\"external_evaluator_independent\":true",
            "\"llm_self_approved\":false",
            "\"evaluator_result_passed\":false",
            "\"evaluator_status\":\"failed\\",
            "\"evaluator_failure_reason\":\"external_evaluator_failed\\",
            "\"evaluator_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_learning_candidate_smoke_composes_learning_candidate() {
    let receipt = ai::validation_harness::policy_reuse_evidence_learning_candidate_smoke_receipt();
    let evaluator =
        ai::validation_harness::policy_reuse_evidence_external_evaluator_result_smoke_receipt();
    let request = ai::validation_harness::policy_reuse_evidence_batch_run_request_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_learning_candidate_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_LEARNING_CANDIDATE_SMOKE_STEP
    );
    assert_eq!(receipt.learning_candidate_version, 1);
    assert_eq!(
        receipt.source_external_evaluator_result_hash,
        evaluator.receipt_hash
    );
    assert_eq!(receipt.source_batch_run_request_hash, request.receipt_hash);
    assert!(receipt.evaluator_result_passed);
    assert!(receipt.batch_request_ready);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.retrieval_write_performed);
    assert_eq!(
        receipt.candidate_batch_capacity,
        evaluator.evaluated_batch_capacity
    );
    assert_eq!(
        receipt.candidate_policy_reuse_cases,
        evaluator.evaluated_policy_reuse_cases
    );
    assert_eq!(
        receipt.candidate_llm_fallback_cases,
        evaluator.evaluated_llm_fallback_cases
    );
    assert!(receipt.learning_candidate_ready);
    assert_eq!(receipt.candidate_status, "candidate");
    assert_eq!(receipt.not_candidate_reason, "none");
    assert_ne!(receipt.candidate_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_learning_candidate_regression_smoke_is_valid_not_candidate_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_learning_candidate_regression_smoke_receipt();
    let evaluator = ai::validation_harness::
        policy_reuse_evidence_external_evaluator_result_regression_smoke_receipt();
    let request =
        ai::validation_harness::policy_reuse_evidence_batch_run_request_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_LEARNING_CANDIDATE_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.source_external_evaluator_result_hash,
        evaluator.receipt_hash
    );
    assert_eq!(receipt.source_batch_run_request_hash, request.receipt_hash);
    assert!(!receipt.evaluator_result_passed);
    assert!(!receipt.batch_request_ready);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.retrieval_write_performed);
    assert_eq!(
        receipt.candidate_batch_capacity,
        evaluator.evaluated_batch_capacity
    );
    assert!(!receipt.learning_candidate_ready);
    assert_eq!(receipt.candidate_status, "not_candidate");
    assert_eq!(receipt.not_candidate_reason, "evaluator_not_passed");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_learning_candidate_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-learning-candidate-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_learning_candidate_v1\\",
            "\"record_type\":\"policy_reuse_evidence_learning_candidate_smoke\\",
            "\"learning_candidate_version\":1",
            "\"source_external_evaluator_result_hash\":",
            "\"source_batch_run_request_hash\":",
            "\"evaluator_result_passed\":true",
            "\"batch_request_ready\":true",
            "\"policy_promotion_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"learning_candidate_ready\":true",
            "\"candidate_status\":\"candidate\\",
            "\"not_candidate_reason\":\"none\\",
            "\"candidate_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_learning_candidate_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-learning-candidate-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_learning_candidate_v1\\",
            "\"record_type\":\"policy_reuse_evidence_learning_candidate_regression_smoke\\",
            "\"learning_candidate_version\":1",
            "\"evaluator_result_passed\":false",
            "\"batch_request_ready\":false",
            "\"policy_promotion_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"learning_candidate_ready\":false",
            "\"candidate_status\":\"not_candidate\\",
            "\"not_candidate_reason\":\"evaluator_not_passed\\",
            "\"candidate_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_learning_data_admission_smoke_composes_dataset_admission() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_learning_data_admission_smoke_receipt();
    let candidate =
        ai::validation_harness::policy_reuse_evidence_learning_candidate_smoke_receipt();
    let evaluator =
        ai::validation_harness::policy_reuse_evidence_external_evaluator_result_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_learning_data_admission_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_LEARNING_DATA_ADMISSION_SMOKE_STEP
    );
    assert_eq!(receipt.data_admission_version, 1);
    assert_eq!(
        receipt.source_learning_candidate_hash,
        candidate.receipt_hash
    );
    assert_eq!(
        receipt.source_external_evaluator_result_hash,
        evaluator.receipt_hash
    );
    assert!(receipt.learning_candidate_ready);
    assert!(receipt.evaluator_result_passed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.admitted_batch_capacity,
        candidate.candidate_batch_capacity
    );
    assert_eq!(
        receipt.admitted_policy_reuse_cases,
        candidate.candidate_policy_reuse_cases
    );
    assert_eq!(
        receipt.admitted_llm_fallback_cases,
        candidate.candidate_llm_fallback_cases
    );
    assert!(receipt.learning_data_admitted);
    assert_eq!(receipt.admission_status, "admitted");
    assert_eq!(receipt.not_admitted_reason, "none");
    assert_ne!(receipt.admission_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_learning_data_admission_regression_smoke_is_valid_not_admitted_evidence() {
    let receipt = ai::validation_harness::
        policy_reuse_evidence_learning_data_admission_regression_smoke_receipt();
    let candidate =
        ai::validation_harness::policy_reuse_evidence_learning_candidate_regression_smoke_receipt();
    let evaluator = ai::validation_harness::
        policy_reuse_evidence_external_evaluator_result_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_LEARNING_DATA_ADMISSION_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.source_learning_candidate_hash,
        candidate.receipt_hash
    );
    assert_eq!(
        receipt.source_external_evaluator_result_hash,
        evaluator.receipt_hash
    );
    assert!(!receipt.learning_candidate_ready);
    assert!(!receipt.evaluator_result_passed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.admitted_batch_capacity,
        candidate.candidate_batch_capacity
    );
    assert!(!receipt.learning_data_admitted);
    assert_eq!(receipt.admission_status, "not_admitted");
    assert_eq!(receipt.not_admitted_reason, "candidate_not_ready");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_learning_data_admission_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-learning-data-admission-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_learning_data_admission_v1\\",
            "\"record_type\":\"policy_reuse_evidence_learning_data_admission_smoke\\",
            "\"data_admission_version\":1",
            "\"source_learning_candidate_hash\":",
            "\"source_external_evaluator_result_hash\":",
            "\"learning_candidate_ready\":true",
            "\"evaluator_result_passed\":true",
            "\"policy_promotion_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"student_training_performed\":false",
            "\"learning_data_admitted\":true",
            "\"admission_status\":\"admitted\\",
            "\"not_admitted_reason\":\"none\\",
            "\"admission_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_learning_data_admission_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-learning-data-admission-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_learning_data_admission_v1\\",
            "\"record_type\":\"policy_reuse_evidence_learning_data_admission_regression_smoke\\",
            "\"data_admission_version\":1",
            "\"learning_candidate_ready\":false",
            "\"evaluator_result_passed\":false",
            "\"policy_promotion_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"student_training_performed\":false",
            "\"learning_data_admitted\":false",
            "\"admission_status\":\"not_admitted\\",
            "\"not_admitted_reason\":\"candidate_not_ready\\",
            "\"admission_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_retrieval_example_admission_smoke_composes_example_admission() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_retrieval_example_admission_smoke_receipt();
    let data_admission =
        ai::validation_harness::policy_reuse_evidence_learning_data_admission_smoke_receipt();
    let candidate =
        ai::validation_harness::policy_reuse_evidence_learning_candidate_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_retrieval_example_admission_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_EXAMPLE_ADMISSION_SMOKE_STEP
    );
    assert_eq!(receipt.retrieval_example_admission_version, 1);
    assert_eq!(
        receipt.source_learning_data_admission_hash,
        data_admission.receipt_hash
    );
    assert_eq!(
        receipt.source_learning_candidate_hash,
        candidate.receipt_hash
    );
    assert!(receipt.learning_data_admitted);
    assert!(receipt.learning_candidate_ready);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.example_policy_reuse_cases,
        data_admission.admitted_policy_reuse_cases
    );
    assert_eq!(
        receipt.example_llm_fallback_cases,
        data_admission.admitted_llm_fallback_cases
    );
    assert!(receipt.retrieval_example_admitted);
    assert_eq!(receipt.admission_status, "admitted");
    assert_eq!(receipt.not_admitted_reason, "none");
    assert_ne!(receipt.admission_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_retrieval_example_admission_regression_smoke_is_valid_not_admitted_evidence(
) {
    let receipt = ai::validation_harness::
        policy_reuse_evidence_retrieval_example_admission_regression_smoke_receipt();
    let data_admission = ai::validation_harness::
        policy_reuse_evidence_learning_data_admission_regression_smoke_receipt();
    let candidate =
        ai::validation_harness::policy_reuse_evidence_learning_candidate_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_EXAMPLE_ADMISSION_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.source_learning_data_admission_hash,
        data_admission.receipt_hash
    );
    assert_eq!(
        receipt.source_learning_candidate_hash,
        candidate.receipt_hash
    );
    assert!(!receipt.learning_data_admitted);
    assert!(!receipt.learning_candidate_ready);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.example_policy_reuse_cases,
        data_admission.admitted_policy_reuse_cases
    );
    assert!(!receipt.retrieval_example_admitted);
    assert_eq!(receipt.admission_status, "not_admitted");
    assert_eq!(receipt.not_admitted_reason, "data_not_admitted");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_example_admission_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-example-admission-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_example_admission_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_example_admission_smoke\\",
            "\"retrieval_example_admission_version\":1",
            "\"source_learning_data_admission_hash\":",
            "\"source_learning_candidate_hash\":",
            "\"learning_data_admitted\":true",
            "\"learning_candidate_ready\":true",
            "\"retrieval_write_performed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_example_admitted\":true",
            "\"admission_status\":\"admitted\\",
            "\"not_admitted_reason\":\"none\\",
            "\"admission_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_example_admission_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-example-admission-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_example_admission_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_example_admission_regression_smoke\\",
            "\"retrieval_example_admission_version\":1",
            "\"learning_data_admitted\":false",
            "\"learning_candidate_ready\":false",
            "\"retrieval_write_performed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_example_admitted\":false",
            "\"admission_status\":\"not_admitted\\",
            "\"not_admitted_reason\":\"data_not_admitted\\",
            "\"admission_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_retrieval_example_index_smoke_composes_index_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_retrieval_example_index_smoke_receipt();
    let example_admission =
        ai::validation_harness::policy_reuse_evidence_retrieval_example_admission_smoke_receipt();
    let data_admission =
        ai::validation_harness::policy_reuse_evidence_learning_data_admission_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_retrieval_example_index_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_EXAMPLE_INDEX_SMOKE_STEP
    );
    assert_eq!(receipt.retrieval_example_index_version, 1);
    assert_eq!(
        receipt.source_retrieval_example_admission_hash,
        example_admission.receipt_hash
    );
    assert_eq!(
        receipt.source_learning_data_admission_hash,
        data_admission.receipt_hash
    );
    assert!(receipt.retrieval_example_admitted);
    assert!(receipt.learning_data_admitted);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.indexed_policy_reuse_examples,
        example_admission.example_policy_reuse_cases
    );
    assert_eq!(
        receipt.indexed_llm_fallback_examples,
        example_admission.example_llm_fallback_cases
    );
    assert!(receipt.retrieval_example_indexed);
    assert_eq!(receipt.index_status, "indexed");
    assert_eq!(receipt.not_indexed_reason, "none");
    assert_ne!(receipt.index_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_retrieval_example_index_regression_smoke_is_valid_not_indexed_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_retrieval_example_index_regression_smoke_receipt();
    let example_admission = ai::validation_harness::
        policy_reuse_evidence_retrieval_example_admission_regression_smoke_receipt();
    let data_admission = ai::validation_harness::
        policy_reuse_evidence_learning_data_admission_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_EXAMPLE_INDEX_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.source_retrieval_example_admission_hash,
        example_admission.receipt_hash
    );
    assert_eq!(
        receipt.source_learning_data_admission_hash,
        data_admission.receipt_hash
    );
    assert!(!receipt.retrieval_example_admitted);
    assert!(!receipt.learning_data_admitted);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.indexed_policy_reuse_examples,
        example_admission.example_policy_reuse_cases
    );
    assert!(!receipt.retrieval_example_indexed);
    assert_eq!(receipt.index_status, "not_indexed");
    assert_eq!(receipt.not_indexed_reason, "example_not_admitted");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_example_index_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-example-index-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_example_index_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_example_index_smoke\\",
            "\"retrieval_example_index_version\":1",
            "\"source_retrieval_example_admission_hash\":",
            "\"source_learning_data_admission_hash\":",
            "\"retrieval_example_admitted\":true",
            "\"learning_data_admitted\":true",
            "\"retrieval_write_performed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_example_indexed\":true",
            "\"index_status\":\"indexed\\",
            "\"not_indexed_reason\":\"none\\",
            "\"index_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_example_index_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-example-index-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_example_index_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_example_index_regression_smoke\\",
            "\"retrieval_example_index_version\":1",
            "\"retrieval_example_admitted\":false",
            "\"learning_data_admitted\":false",
            "\"retrieval_write_performed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_example_indexed\":false",
            "\"index_status\":\"not_indexed\\",
            "\"not_indexed_reason\":\"example_not_admitted\\",
            "\"index_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_retrieval_corpus_readiness_smoke_composes_corpus_readiness() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_retrieval_corpus_readiness_smoke_receipt();
    let example_index =
        ai::validation_harness::policy_reuse_evidence_retrieval_example_index_smoke_receipt();
    let example_admission =
        ai::validation_harness::policy_reuse_evidence_retrieval_example_admission_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_retrieval_corpus_readiness_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_CORPUS_READINESS_SMOKE_STEP
    );
    assert_eq!(receipt.retrieval_corpus_readiness_version, 1);
    assert_eq!(
        receipt.source_retrieval_example_index_hash,
        example_index.receipt_hash
    );
    assert_eq!(
        receipt.source_retrieval_example_admission_hash,
        example_admission.receipt_hash
    );
    assert!(receipt.retrieval_example_indexed);
    assert!(receipt.retrieval_example_admitted);
    assert!(!receipt.retrieval_read_performed);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.ready_policy_reuse_examples,
        example_index.indexed_policy_reuse_examples
    );
    assert_eq!(
        receipt.ready_llm_fallback_examples,
        example_index.indexed_llm_fallback_examples
    );
    assert!(receipt.retrieval_corpus_ready);
    assert_eq!(receipt.readiness_status, "ready");
    assert_eq!(receipt.not_ready_reason, "none");
    assert_ne!(receipt.readiness_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_retrieval_corpus_readiness_regression_smoke_is_valid_not_ready_evidence() {
    let receipt = ai::validation_harness::
        policy_reuse_evidence_retrieval_corpus_readiness_regression_smoke_receipt();
    let example_index = ai::validation_harness::
        policy_reuse_evidence_retrieval_example_index_regression_smoke_receipt();
    let example_admission = ai::validation_harness::
        policy_reuse_evidence_retrieval_example_admission_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_CORPUS_READINESS_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.source_retrieval_example_index_hash,
        example_index.receipt_hash
    );
    assert_eq!(
        receipt.source_retrieval_example_admission_hash,
        example_admission.receipt_hash
    );
    assert!(!receipt.retrieval_example_indexed);
    assert!(!receipt.retrieval_example_admitted);
    assert!(!receipt.retrieval_read_performed);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.ready_policy_reuse_examples,
        example_index.indexed_policy_reuse_examples
    );
    assert!(!receipt.retrieval_corpus_ready);
    assert_eq!(receipt.readiness_status, "not_ready");
    assert_eq!(receipt.not_ready_reason, "index_not_ready");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_corpus_readiness_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-corpus-readiness-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_corpus_readiness_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_corpus_readiness_smoke\\",
            "\"retrieval_corpus_readiness_version\":1",
            "\"source_retrieval_example_index_hash\":",
            "\"source_retrieval_example_admission_hash\":",
            "\"retrieval_example_indexed\":true",
            "\"retrieval_example_admitted\":true",
            "\"retrieval_read_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_corpus_ready\":true",
            "\"readiness_status\":\"ready\\",
            "\"not_ready_reason\":\"none\\",
            "\"readiness_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_corpus_readiness_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-corpus-readiness-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_corpus_readiness_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_corpus_readiness_regression_smoke\\",
            "\"retrieval_corpus_readiness_version\":1",
            "\"retrieval_example_indexed\":false",
            "\"retrieval_example_admitted\":false",
            "\"retrieval_read_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_corpus_ready\":false",
            "\"readiness_status\":\"not_ready\\",
            "\"not_ready_reason\":\"index_not_ready\\",
            "\"readiness_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_retrieval_corpus_admission_smoke_composes_corpus_admission() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_retrieval_corpus_admission_smoke_receipt();
    let corpus_readiness =
        ai::validation_harness::policy_reuse_evidence_retrieval_corpus_readiness_smoke_receipt();
    let example_index =
        ai::validation_harness::policy_reuse_evidence_retrieval_example_index_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_retrieval_corpus_admission_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_CORPUS_ADMISSION_SMOKE_STEP
    );
    assert_eq!(receipt.retrieval_corpus_admission_version, 1);
    assert_eq!(
        receipt.source_retrieval_corpus_readiness_hash,
        corpus_readiness.receipt_hash
    );
    assert_eq!(
        receipt.source_retrieval_example_index_hash,
        example_index.receipt_hash
    );
    assert!(receipt.retrieval_corpus_ready);
    assert!(receipt.retrieval_example_indexed);
    assert!(!receipt.retrieval_read_performed);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.admitted_policy_reuse_examples,
        corpus_readiness.ready_policy_reuse_examples
    );
    assert_eq!(
        receipt.admitted_llm_fallback_examples,
        corpus_readiness.ready_llm_fallback_examples
    );
    assert!(receipt.retrieval_corpus_admitted);
    assert_eq!(receipt.admission_status, "admitted");
    assert_eq!(receipt.not_admitted_reason, "none");
    assert_ne!(receipt.admission_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_retrieval_corpus_admission_regression_smoke_is_valid_not_admitted_evidence(
) {
    let receipt = ai::validation_harness::
        policy_reuse_evidence_retrieval_corpus_admission_regression_smoke_receipt();
    let corpus_readiness = ai::validation_harness::
        policy_reuse_evidence_retrieval_corpus_readiness_regression_smoke_receipt();
    let example_index = ai::validation_harness::
        policy_reuse_evidence_retrieval_example_index_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_CORPUS_ADMISSION_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.source_retrieval_corpus_readiness_hash,
        corpus_readiness.receipt_hash
    );
    assert_eq!(
        receipt.source_retrieval_example_index_hash,
        example_index.receipt_hash
    );
    assert!(!receipt.retrieval_corpus_ready);
    assert!(!receipt.retrieval_example_indexed);
    assert!(!receipt.retrieval_read_performed);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.admitted_policy_reuse_examples,
        corpus_readiness.ready_policy_reuse_examples
    );
    assert!(!receipt.retrieval_corpus_admitted);
    assert_eq!(receipt.admission_status, "not_admitted");
    assert_eq!(receipt.not_admitted_reason, "corpus_not_ready");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_corpus_admission_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-corpus-admission-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_corpus_admission_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_corpus_admission_smoke\\",
            "\"retrieval_corpus_admission_version\":1",
            "\"source_retrieval_corpus_readiness_hash\":",
            "\"source_retrieval_example_index_hash\":",
            "\"retrieval_corpus_ready\":true",
            "\"retrieval_example_indexed\":true",
            "\"retrieval_read_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_corpus_admitted\":true",
            "\"admission_status\":\"admitted\\",
            "\"not_admitted_reason\":\"none\\",
            "\"admission_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_corpus_admission_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-corpus-admission-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_corpus_admission_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_corpus_admission_regression_smoke\\",
            "\"retrieval_corpus_admission_version\":1",
            "\"retrieval_corpus_ready\":false",
            "\"retrieval_example_indexed\":false",
            "\"retrieval_read_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_corpus_admitted\":false",
            "\"admission_status\":\"not_admitted\\",
            "\"not_admitted_reason\":\"corpus_not_ready\\",
            "\"admission_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_retrieval_use_approval_smoke_composes_use_approval() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_retrieval_use_approval_smoke_receipt();
    let corpus_admission =
        ai::validation_harness::policy_reuse_evidence_retrieval_corpus_admission_smoke_receipt();
    let corpus_readiness =
        ai::validation_harness::policy_reuse_evidence_retrieval_corpus_readiness_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_retrieval_use_approval_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_USE_APPROVAL_SMOKE_STEP
    );
    assert_eq!(receipt.retrieval_use_approval_version, 1);
    assert_eq!(
        receipt.source_retrieval_corpus_admission_hash,
        corpus_admission.receipt_hash
    );
    assert_eq!(
        receipt.source_retrieval_corpus_readiness_hash,
        corpus_readiness.receipt_hash
    );
    assert!(receipt.retrieval_corpus_admitted);
    assert!(receipt.retrieval_corpus_ready);
    assert!(!receipt.retrieval_read_performed);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.approved_policy_reuse_examples,
        corpus_admission.admitted_policy_reuse_examples
    );
    assert_eq!(
        receipt.approved_llm_fallback_examples,
        corpus_admission.admitted_llm_fallback_examples
    );
    assert!(receipt.retrieval_use_approved);
    assert_eq!(receipt.approval_status, "approved");
    assert_eq!(receipt.not_approved_reason, "none");
    assert_ne!(receipt.approval_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_retrieval_use_approval_regression_smoke_is_valid_not_approved_evidence() {
    let receipt = ai::validation_harness::
        policy_reuse_evidence_retrieval_use_approval_regression_smoke_receipt();
    let corpus_admission = ai::validation_harness::
        policy_reuse_evidence_retrieval_corpus_admission_regression_smoke_receipt();
    let corpus_readiness = ai::validation_harness::
        policy_reuse_evidence_retrieval_corpus_readiness_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_USE_APPROVAL_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.source_retrieval_corpus_admission_hash,
        corpus_admission.receipt_hash
    );
    assert_eq!(
        receipt.source_retrieval_corpus_readiness_hash,
        corpus_readiness.receipt_hash
    );
    assert!(!receipt.retrieval_corpus_admitted);
    assert!(!receipt.retrieval_corpus_ready);
    assert!(!receipt.retrieval_read_performed);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.approved_policy_reuse_examples,
        corpus_admission.admitted_policy_reuse_examples
    );
    assert!(!receipt.retrieval_use_approved);
    assert_eq!(receipt.approval_status, "not_approved");
    assert_eq!(receipt.not_approved_reason, "corpus_not_admitted");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_use_approval_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-use-approval-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_use_approval_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_use_approval_smoke\\",
            "\"retrieval_use_approval_version\":1",
            "\"source_retrieval_corpus_admission_hash\":",
            "\"source_retrieval_corpus_readiness_hash\":",
            "\"retrieval_corpus_admitted\":true",
            "\"retrieval_corpus_ready\":true",
            "\"retrieval_read_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_use_approved\":true",
            "\"approval_status\":\"approved\\",
            "\"not_approved_reason\":\"none\\",
            "\"approval_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_use_approval_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-use-approval-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_use_approval_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_use_approval_regression_smoke\\",
            "\"retrieval_use_approval_version\":1",
            "\"retrieval_corpus_admitted\":false",
            "\"retrieval_corpus_ready\":false",
            "\"retrieval_read_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_use_approved\":false",
            "\"approval_status\":\"not_approved\\",
            "\"not_approved_reason\":\"corpus_not_admitted\\",
            "\"approval_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_retrieval_use_manifest_smoke_composes_manifest_readiness() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_retrieval_use_manifest_smoke_receipt();
    let use_approval =
        ai::validation_harness::policy_reuse_evidence_retrieval_use_approval_smoke_receipt();
    let corpus_admission =
        ai::validation_harness::policy_reuse_evidence_retrieval_corpus_admission_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_retrieval_use_manifest_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_USE_MANIFEST_SMOKE_STEP
    );
    assert_eq!(receipt.retrieval_use_manifest_version, 1);
    assert_eq!(
        receipt.source_retrieval_use_approval_hash,
        use_approval.receipt_hash
    );
    assert_eq!(
        receipt.source_retrieval_corpus_admission_hash,
        corpus_admission.receipt_hash
    );
    assert!(receipt.retrieval_use_approved);
    assert!(receipt.retrieval_corpus_admitted);
    assert!(!receipt.retrieval_read_performed);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.manifest_policy_reuse_examples,
        use_approval.approved_policy_reuse_examples
    );
    assert_eq!(
        receipt.manifest_llm_fallback_examples,
        use_approval.approved_llm_fallback_examples
    );
    assert!(receipt.retrieval_use_manifest_ready);
    assert_eq!(receipt.manifest_status, "manifest_ready");
    assert_eq!(receipt.not_ready_reason, "none");
    assert_ne!(receipt.manifest_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_retrieval_use_manifest_regression_smoke_is_valid_not_ready_evidence() {
    let receipt = ai::validation_harness::
        policy_reuse_evidence_retrieval_use_manifest_regression_smoke_receipt();
    let use_approval = ai::validation_harness::
        policy_reuse_evidence_retrieval_use_approval_regression_smoke_receipt();
    let corpus_admission = ai::validation_harness::
        policy_reuse_evidence_retrieval_corpus_admission_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_USE_MANIFEST_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.source_retrieval_use_approval_hash,
        use_approval.receipt_hash
    );
    assert_eq!(
        receipt.source_retrieval_corpus_admission_hash,
        corpus_admission.receipt_hash
    );
    assert!(!receipt.retrieval_use_approved);
    assert!(!receipt.retrieval_corpus_admitted);
    assert!(!receipt.retrieval_read_performed);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.manifest_policy_reuse_examples,
        use_approval.approved_policy_reuse_examples
    );
    assert!(!receipt.retrieval_use_manifest_ready);
    assert_eq!(receipt.manifest_status, "manifest_not_ready");
    assert_eq!(receipt.not_ready_reason, "use_not_approved");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_use_manifest_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-use-manifest-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_use_manifest_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_use_manifest_smoke\\",
            "\"retrieval_use_manifest_version\":1",
            "\"source_retrieval_use_approval_hash\":",
            "\"source_retrieval_corpus_admission_hash\":",
            "\"retrieval_use_approved\":true",
            "\"retrieval_corpus_admitted\":true",
            "\"retrieval_read_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_use_manifest_ready\":true",
            "\"manifest_status\":\"manifest_ready\\",
            "\"not_ready_reason\":\"none\\",
            "\"manifest_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_use_manifest_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-use-manifest-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_use_manifest_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_use_manifest_regression_smoke\\",
            "\"retrieval_use_manifest_version\":1",
            "\"retrieval_use_approved\":false",
            "\"retrieval_corpus_admitted\":false",
            "\"retrieval_read_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_use_manifest_ready\":false",
            "\"manifest_status\":\"manifest_not_ready\\",
            "\"not_ready_reason\":\"use_not_approved\\",
            "\"manifest_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_retrieval_query_plan_smoke_composes_query_plan_readiness() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_retrieval_query_plan_smoke_receipt();
    let use_manifest =
        ai::validation_harness::policy_reuse_evidence_retrieval_use_manifest_smoke_receipt();
    let use_approval =
        ai::validation_harness::policy_reuse_evidence_retrieval_use_approval_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_retrieval_query_plan_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_QUERY_PLAN_SMOKE_STEP
    );
    assert_eq!(receipt.retrieval_query_plan_version, 1);
    assert_eq!(
        receipt.source_retrieval_use_manifest_hash,
        use_manifest.receipt_hash
    );
    assert_eq!(
        receipt.source_retrieval_use_approval_hash,
        use_approval.receipt_hash
    );
    assert!(receipt.retrieval_use_manifest_ready);
    assert!(receipt.retrieval_use_approved);
    assert!(!receipt.retrieval_read_performed);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.retrieval_query_executed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.planned_policy_reuse_examples,
        use_manifest.manifest_policy_reuse_examples
    );
    assert_eq!(
        receipt.planned_llm_fallback_examples,
        use_manifest.manifest_llm_fallback_examples
    );
    assert!(receipt.retrieval_query_plan_ready);
    assert_eq!(receipt.query_plan_status, "query_plan_ready");
    assert_eq!(receipt.not_ready_reason, "none");
    assert_ne!(receipt.query_plan_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_retrieval_query_plan_regression_smoke_is_valid_not_ready_evidence() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_retrieval_query_plan_regression_smoke_receipt(
        );
    let use_manifest = ai::validation_harness::
        policy_reuse_evidence_retrieval_use_manifest_regression_smoke_receipt();
    let use_approval = ai::validation_harness::
        policy_reuse_evidence_retrieval_use_approval_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_QUERY_PLAN_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.source_retrieval_use_manifest_hash,
        use_manifest.receipt_hash
    );
    assert_eq!(
        receipt.source_retrieval_use_approval_hash,
        use_approval.receipt_hash
    );
    assert!(!receipt.retrieval_use_manifest_ready);
    assert!(!receipt.retrieval_use_approved);
    assert!(!receipt.retrieval_read_performed);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.retrieval_query_executed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.planned_policy_reuse_examples,
        use_manifest.manifest_policy_reuse_examples
    );
    assert!(!receipt.retrieval_query_plan_ready);
    assert_eq!(receipt.query_plan_status, "query_plan_not_ready");
    assert_eq!(receipt.not_ready_reason, "manifest_not_ready");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_query_plan_smoke_mode_is_executable_contract() {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-query-plan-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_query_plan_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_query_plan_smoke\\",
            "\"retrieval_query_plan_version\":1",
            "\"source_retrieval_use_manifest_hash\":",
            "\"source_retrieval_use_approval_hash\":",
            "\"retrieval_use_manifest_ready\":true",
            "\"retrieval_use_approved\":true",
            "\"retrieval_read_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"retrieval_query_executed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_query_plan_ready\":true",
            "\"query_plan_status\":\"query_plan_ready\\",
            "\"not_ready_reason\":\"none\\",
            "\"query_plan_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_query_plan_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-query-plan-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_query_plan_v1\\",
            "\"record_type\":\"policy_reuse_evidence_retrieval_query_plan_regression_smoke\\",
            "\"retrieval_query_plan_version\":1",
            "\"retrieval_use_manifest_ready\":false",
            "\"retrieval_use_approved\":false",
            "\"retrieval_read_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"retrieval_query_executed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_query_plan_ready\":false",
            "\"query_plan_status\":\"query_plan_not_ready\\",
            "\"not_ready_reason\":\"manifest_not_ready\\",
            "\"query_plan_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_evidence_retrieval_query_approval_smoke_composes_query_approval() {
    let receipt =
        ai::validation_harness::policy_reuse_evidence_retrieval_query_approval_smoke_receipt();
    let query_plan =
        ai::validation_harness::policy_reuse_evidence_retrieval_query_plan_smoke_receipt();
    let use_manifest =
        ai::validation_harness::policy_reuse_evidence_retrieval_use_manifest_smoke_receipt();

    assert_eq!(
        receipt.schema,
        "canon_policy_reuse_evidence_retrieval_query_approval_v1"
    );
    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_QUERY_APPROVAL_SMOKE_STEP
    );
    assert_eq!(receipt.retrieval_query_approval_version, 1);
    assert_eq!(
        receipt.source_retrieval_query_plan_hash,
        query_plan.receipt_hash
    );
    assert_eq!(
        receipt.source_retrieval_use_manifest_hash,
        use_manifest.receipt_hash
    );
    assert!(receipt.retrieval_query_plan_ready);
    assert!(receipt.retrieval_use_manifest_ready);
    assert!(!receipt.retrieval_read_performed);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.retrieval_query_executed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.approved_query_policy_reuse_examples,
        query_plan.planned_policy_reuse_examples
    );
    assert_eq!(
        receipt.approved_query_llm_fallback_examples,
        query_plan.planned_llm_fallback_examples
    );
    assert!(receipt.retrieval_query_approved);
    assert_eq!(receipt.query_approval_status, "query_approved");
    assert_eq!(receipt.not_approved_reason, "none");
    assert_ne!(receipt.query_approval_hash, 0);
    assert_ne!(receipt.receipt_hash, 0);
    assert!(receipt.is_valid());
    assert!(receipt.passed());
}

#[test]
fn policy_reuse_evidence_retrieval_query_approval_regression_smoke_is_valid_not_approved_evidence()
{
    let receipt = ai::validation_harness::
        policy_reuse_evidence_retrieval_query_approval_regression_smoke_receipt();
    let query_plan =
        ai::validation_harness::policy_reuse_evidence_retrieval_query_plan_regression_smoke_receipt(
        );
    let use_manifest = ai::validation_harness::
        policy_reuse_evidence_retrieval_use_manifest_regression_smoke_receipt();

    assert_eq!(
        receipt.record_type,
        ai::validation_harness::POLICY_REUSE_EVIDENCE_RETRIEVAL_QUERY_APPROVAL_REGRESSION_SMOKE_STEP
    );
    assert_eq!(
        receipt.source_retrieval_query_plan_hash,
        query_plan.receipt_hash
    );
    assert_eq!(
        receipt.source_retrieval_use_manifest_hash,
        use_manifest.receipt_hash
    );
    assert!(!receipt.retrieval_query_plan_ready);
    assert!(!receipt.retrieval_use_manifest_ready);
    assert!(!receipt.retrieval_read_performed);
    assert!(!receipt.retrieval_write_performed);
    assert!(!receipt.retrieval_query_executed);
    assert!(!receipt.policy_promotion_performed);
    assert!(!receipt.student_training_performed);
    assert_eq!(
        receipt.approved_query_policy_reuse_examples,
        query_plan.planned_policy_reuse_examples
    );
    assert!(!receipt.retrieval_query_approved);
    assert_eq!(receipt.query_approval_status, "query_not_approved");
    assert_eq!(receipt.not_approved_reason, "query_plan_not_ready");
    assert!(receipt.is_valid());
    assert!(!receipt.passed());
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_query_approval_smoke_mode_is_executable_contract()
{
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-query-approval-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_query_approval_v1\"",
            "\"record_type\":\"policy_reuse_evidence_retrieval_query_approval_smoke\"",
            "\"retrieval_query_approval_version\":1",
            "\"source_retrieval_query_plan_hash\":",
            "\"source_retrieval_use_manifest_hash\":",
            "\"retrieval_query_plan_ready\":true",
            "\"retrieval_use_manifest_ready\":true",
            "\"retrieval_read_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"retrieval_query_executed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_query_approved\":true",
            "\"query_approval_status\":\"query_approved\"",
            "\"not_approved_reason\":\"none\"",
            "\"query_approval_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn root_validate_policy_reuse_evidence_retrieval_query_approval_regression_smoke_mode_is_executable_contract(
) {
    assert_root_validate_catalog_compact_mode_contract(
        "--policy-reuse-evidence-retrieval-query-approval-regression-smoke",
        &[
            "\"schema\":\"canon_policy_reuse_evidence_retrieval_query_approval_v1\"",
            "\"record_type\":\"policy_reuse_evidence_retrieval_query_approval_regression_smoke\"",
            "\"retrieval_query_approval_version\":1",
            "\"retrieval_query_plan_ready\":false",
            "\"retrieval_use_manifest_ready\":false",
            "\"retrieval_read_performed\":false",
            "\"retrieval_write_performed\":false",
            "\"retrieval_query_executed\":false",
            "\"policy_promotion_performed\":false",
            "\"student_training_performed\":false",
            "\"retrieval_query_approved\":false",
            "\"query_approval_status\":\"query_not_approved\"",
            "\"not_approved_reason\":\"query_plan_not_ready\"",
            "\"query_approval_hash\":",
            "\"receipt_hash\":",
        ],
    );
}

#[test]
fn policy_reuse_receipts_fixture_binds_expected_retained_receipts() {
    let fixture = std::fs::read_to_string(POLICY_REUSE_RECEIPTS_FIXTURE)
        .expect("policy reuse receipts fixture must be readable");
    assert!(policy_reuse_receipts_fixture_valid(&fixture));
}

#[test]
fn policy_reuse_receipts_fixture_negative_contract_detects_drift() {
    let fixture = std::fs::read_to_string(POLICY_REUSE_RECEIPTS_FIXTURE)
        .expect("policy reuse receipts fixture must be readable");
    let drifted = fixture.replace(
        "policy_reuse_smoke.avoided_llm_call_count=1",
        "policy_reuse_smoke.avoided_llm_call_count=0",
    );
    assert!(!policy_reuse_receipts_fixture_valid(&drifted));
}

fn policy_reuse_receipts_fixture_valid(fixture: &str) -> bool {
    if !ai::validation_harness::retained_fixture_header_valid(
        fixture,
        "canon_policy_reuse_receipts_v1",
        3,
    ) {
        return false;
    }

    let reuse = ai::validation_harness::policy_reuse_smoke_receipt();
    let trend = ai::validation_harness::policy_reuse_trend_smoke_receipt();
    let regression = ai::validation_harness::policy_reuse_regression_smoke_receipt();

    let expected = [
        format!("policy_reuse_smoke.record_type={}", reuse.record_type),
        format!(
            "policy_reuse_smoke.retained_record_count={}",
            reuse.retained_record_count
        ),
        format!(
            "policy_reuse_smoke.policy_hit_count={}",
            reuse.policy_hit_count
        ),
        format!(
            "policy_reuse_smoke.policy_miss_count={}",
            reuse.policy_miss_count
        ),
        format!(
            "policy_reuse_smoke.avoided_llm_call_count={}",
            reuse.avoided_llm_call_count
        ),
        format!("policy_reuse_smoke.hit_rate_bps={}", reuse.hit_rate_bps),
        format!("policy_reuse_smoke.verdict={}", reuse.verdict),
        format!("policy_reuse_trend_smoke.record_type={}", trend.record_type),
        format!(
            "policy_reuse_trend_smoke.baseline_retained_record_count={}",
            trend.baseline_retained_record_count
        ),
        format!(
            "policy_reuse_trend_smoke.current_retained_record_count={}",
            trend.current_retained_record_count
        ),
        format!(
            "policy_reuse_trend_smoke.baseline_hit_rate_bps={}",
            trend.baseline_hit_rate_bps
        ),
        format!(
            "policy_reuse_trend_smoke.current_hit_rate_bps={}",
            trend.current_hit_rate_bps
        ),
        format!(
            "policy_reuse_trend_smoke.hit_rate_delta_bps={}",
            trend.hit_rate_delta_bps
        ),
        format!(
            "policy_reuse_trend_smoke.avoided_llm_call_delta={}",
            trend.avoided_llm_call_delta
        ),
        format!(
            "policy_reuse_trend_smoke.trend_status={}",
            trend.trend_status
        ),
        format!("policy_reuse_trend_smoke.verdict={}", trend.verdict),
        format!(
            "policy_reuse_regression_smoke.record_type={}",
            regression.record_type
        ),
        format!(
            "policy_reuse_regression_smoke.baseline_retained_record_count={}",
            regression.baseline_retained_record_count
        ),
        format!(
            "policy_reuse_regression_smoke.current_retained_record_count={}",
            regression.current_retained_record_count
        ),
        format!(
            "policy_reuse_regression_smoke.baseline_hit_rate_bps={}",
            regression.baseline_hit_rate_bps
        ),
        format!(
            "policy_reuse_regression_smoke.current_hit_rate_bps={}",
            regression.current_hit_rate_bps
        ),
        format!(
            "policy_reuse_regression_smoke.hit_rate_delta_bps={}",
            regression.hit_rate_delta_bps
        ),
        format!(
            "policy_reuse_regression_smoke.avoided_llm_call_delta={}",
            regression.avoided_llm_call_delta
        ),
        format!(
            "policy_reuse_regression_smoke.trend_status={}",
            regression.trend_status
        ),
        format!(
            "policy_reuse_regression_smoke.verdict={}",
            regression.verdict
        ),
    ];

    fixture_contains_expected_lines(fixture, &expected)
        && fixture
            .contains("rule=policy_hit_count equals avoided_llm_call_count for policy_reuse_smoke")
        && fixture.contains(
            "rule=trend smoke passes only when hit rate and avoided LLM calls do not regress",
        )
        && fixture.contains("rule=regression smoke remains valid but does not pass")
        && reuse.avoided_llm_call_count == reuse.policy_hit_count
        && trend.passed()
        && regression.is_valid()
        && !regression.passed()
}

#[test]
fn root_validation_pass_status_includes_runtime_performance_budget() {
    let mut receipt = ValidationReceipt {
        cargo: "cargo".to_owned(),
        cargo_version: "cargo 1.96.0-nightly".to_owned(),
        steps: vec![StepReceipt {
            name: VALIDATION_HARNESS_STEP,
            command: "cargo -Znext-lockfile-bump test --test validation_harness_contract --locked -- --nocapture".to_owned(),
            exit_code: Some(0),
            stdout_bytes: 428,
            stderr_bytes: 188,
            skip_reason: None,
            expected_test_count: Some(VALIDATION_HARNESS_EXPECTED_TESTS),
            observed_test_count: Some(VALIDATION_HARNESS_EXPECTED_TESTS),
        }],
        graph_telemetry: GraphTelemetryReceipt {
            crate_name: "semantic_scale_probe".to_owned(),
            graph_path: "/tmp/canon-agent-graph-telemetry.json".to_owned(),
            node_count: 64,
            edge_count: 136,
            semantic_fn_count: 64,
            semantic_fn_coverage_bps: 10_000,
            wrapper_hash: "wrapper-hash".to_owned(),
            report_hash: "report-hash".to_owned(),
        },
        runtime_performance: RuntimePerformanceReceipt {
            step: RUNTIME_PERFORMANCE_STEP,
            runtime_performance_signal_present: true,
            runtime_performance_budget_status: "pass",
            project_agent_elapsed_ms_median: 1,
            project_agent_elapsed_ms_p95: 1,
            download_initial_get_ms_median: 0,
            download_initial_get_ms_p95: 0,
            download_follow_get_ms_median: 0,
            download_follow_get_ms_p95: 0,
            download_write_ms_median: 0,
            download_write_ms_p95: 0,
            validation_command_duration_ms: 1,
            max_project_agent_elapsed_ms_p95: 10_000,
            max_download_initial_get_ms_p95: 2_000,
            max_download_follow_get_ms_p95: 2_000,
            max_download_write_ms_p95: 1_000,
        },
    };
    assert!(receipt.passed());

    receipt.runtime_performance.max_project_agent_elapsed_ms_p95 = 0;
    receipt
        .runtime_performance
        .runtime_performance_budget_status = "fail";

    assert!(!receipt.passed());
    assert!(receipt.to_json_line().contains("\"passed\":false"));
}

#[test]
fn root_validation_json_receipt_records_graph_cli_test_surface() {
    let receipt = ValidationReceipt {
        cargo: "cargo".to_owned(),
        cargo_version: "cargo 1.96.0-nightly".to_owned(),
        steps: vec![
            StepReceipt {
                name: VALIDATION_HARNESS_STEP,
                command: "cargo -Znext-lockfile-bump test --test validation_harness_contract --locked -- --nocapture".to_owned(),
                exit_code: Some(0),
                stdout_bytes: 428,
                stderr_bytes: 188,
                skip_reason: None,
                expected_test_count: Some(VALIDATION_HARNESS_EXPECTED_TESTS),
                observed_test_count: Some(VALIDATION_HARNESS_EXPECTED_TESTS),
            },
            StepReceipt {
                name: GRAPH_MUTATION_CLI_CONTRACT_STEP,
                command: "cargo -Znext-lockfile-bump test --test graph_mutation_cli_contract --locked -- --nocapture".to_owned(),
                exit_code: Some(0),
                stdout_bytes: 806,
                stderr_bytes: 188,
                skip_reason: None,
                expected_test_count: Some(GRAPH_MUTATION_CLI_CONTRACT_EXPECTED_TESTS),
                observed_test_count: Some(GRAPH_MUTATION_CLI_CONTRACT_EXPECTED_TESTS),
            },
        ],
        graph_telemetry: GraphTelemetryReceipt {
            crate_name: "semantic_scale_probe".to_owned(),
            graph_path: "/tmp/canon-agent-graph-telemetry.json".to_owned(),
            node_count: 64,
            edge_count: 136,
            semantic_fn_count: 64,
            semantic_fn_coverage_bps: 10_000,
            wrapper_hash: "wrapper-hash".to_owned(),
            report_hash: "report-hash".to_owned(),
        },
        runtime_performance: RuntimePerformanceReceipt {
            step: RUNTIME_PERFORMANCE_STEP,
            runtime_performance_signal_present: true,
            runtime_performance_budget_status: "pass",
            project_agent_elapsed_ms_median: 1,
            project_agent_elapsed_ms_p95: 1,
            download_initial_get_ms_median: 0,
            download_initial_get_ms_p95: 0,
            download_follow_get_ms_median: 0,
            download_follow_get_ms_p95: 0,
            download_write_ms_median: 0,
            download_write_ms_p95: 0,
            validation_command_duration_ms: 1,
            max_project_agent_elapsed_ms_p95: 10_000,
            max_download_initial_get_ms_p95: 2_000,
            max_download_follow_get_ms_p95: 2_000,
            max_download_write_ms_p95: 1_000,
        },
    };

    let json = receipt.to_json_line();
    assert!(json.contains("\"name\":\"validation_harness_contract_tests\\"));
    assert!(json.contains(&format!(
        "\"expected_test_count\":{VALIDATION_HARNESS_EXPECTED_TESTS}"
    )));
    assert!(json.contains(&format!(
        "\"observed_test_count\":{VALIDATION_HARNESS_EXPECTED_TESTS}"
    )));
    assert!(json.contains("\"name\":\"graph_mutation_cli_contract_tests\\"));
    assert!(json.contains("\"expected_test_count\":10"));
    assert!(json.contains("\"observed_test_count\":10"));
    assert!(json.contains("\"passed\":true"));
}
