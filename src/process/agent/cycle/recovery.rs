//! Recovery routing helpers for AgentCycle.

#[derive(Clone, Copy)]
pub(super) struct RecoveryActionSpec {
    pub(super) gate: Option<(&'static str, &'static str)>,
    pub(super) target_phase: &'static str,
}

pub(super) struct RecoveryRoute {
    pub(super) action: &'static str,
    pub(super) failures: &'static [&'static str],
    pub(super) spec: RecoveryActionSpec,
}

const RECOVERY_ROUTES: &[RecoveryRoute] = &[
    RecoveryRoute {
        action: "RecheckInvariant",
        failures: &["InvariantUnknown", "InvariantBlocked"],
        spec: RecoveryActionSpec {
            gate: Some(("Invariant", "InvariantProof")),
            target_phase: "Invariant",
        },
    },
    RecoveryRoute {
        action: "RunAnalysis",
        failures: &["AnalysisMissing", "AnalysisFailed"],
        spec: RecoveryActionSpec {
            gate: Some(("Analysis", "AnalysisReport")),
            target_phase: "Analysis",
        },
    },
    RecoveryRoute {
        action: "Rejudge",
        failures: &["JudgmentMissing", "JudgmentFailed"],
        spec: RecoveryActionSpec {
            gate: Some(("Judgment", "JudgmentRecord")),
            target_phase: "Judgment",
        },
    },
    RecoveryRoute {
        action: "Replan",
        failures: &["PlanMissing", "PlanFailed"],
        spec: RecoveryActionSpec {
            gate: Some(("Plan", "PlanRecord")),
            target_phase: "Plan",
        },
    },
    RecoveryRoute {
        action: "BindReadyTask",
        failures: &["PlanReadyQueueEmpty"],
        spec: RecoveryActionSpec {
            gate: Some(("Plan", "TaskReady")),
            target_phase: "Plan",
        },
    },
    RecoveryRoute {
        action: "Reexecute",
        failures: &["ExecutionMissing", "ExecutionFailed", "TaskReceiptMissing"],
        spec: RecoveryActionSpec {
            gate: Some(("Execution", "ArtifactReceipt")),
            target_phase: "Execute",
        },
    },
    RecoveryRoute {
        action: "Reverify",
        failures: &["VerificationUnknown", "VerificationFailed"],
        spec: RecoveryActionSpec {
            gate: Some(("Verification", "VerificationReport")),
            target_phase: "Verify",
        },
    },
    RecoveryRoute {
        action: "RepairArtifactLineage",
        failures: &["ArtifactLineageBroken"],
        spec: RecoveryActionSpec {
            gate: Some(("Verification", "LineageProof")),
            target_phase: "Verify",
        },
    },
    RecoveryRoute {
        action: "RecomputeEval",
        failures: &["EvalMissing", "EvalFailed"],
        spec: RecoveryActionSpec {
            gate: Some(("Eval", "EvalScore")),
            target_phase: "Eval",
        },
    },
    RecoveryRoute {
        action: "Escalate",
        failures: &["RecoveryExhausted", "ConvergenceFailed"],
        spec: RecoveryActionSpec {
            gate: None,
            target_phase: "Done",
        },
    },
];

pub(super) fn recovery_route_matching(
    predicate: impl Fn(&RecoveryRoute) -> bool,
) -> Option<&'static RecoveryRoute> {
    RECOVERY_ROUTES.iter().find(|route| predicate(route))
}

pub(super) fn recovery_route_for_action(action: &str) -> Option<&'static RecoveryRoute> {
    recovery_route_matching(|route| route.action == action)
}

pub(super) fn recovery_route_for_failure(failure: &str) -> Option<&'static RecoveryRoute> {
    recovery_route_matching(|route| route.failures.contains(&failure))
}

pub(super) fn recovery_action_for_failure(failure: &str) -> Option<&'static str> {
    recovery_route_for_failure(failure).map(|route| route.action)
}

pub(super) fn recovery_action_spec(action: &str) -> Option<RecoveryActionSpec> {
    recovery_route_for_action(action).map(|route| route.spec)
}
