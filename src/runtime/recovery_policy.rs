//! Total recovery and evidence policy tables.

use crate::kernel::{
    Decision, EventKind, Evidence, FailureClass, GateId, GateStatus, RecoveryAction,
};

#[derive(Clone, Copy)]
struct RecoveryPolicyRule {
    failure: FailureClass,
    action: RecoveryAction,
}

const RECOVERY_POLICY: [RecoveryPolicyRule; 21] = [
    RecoveryPolicyRule { failure: FailureClass::InvariantUnknown, action: RecoveryAction::RecheckInvariant },
    RecoveryPolicyRule { failure: FailureClass::InvariantBlocked, action: RecoveryAction::RecheckInvariant },
    RecoveryPolicyRule { failure: FailureClass::AnalysisMissing, action: RecoveryAction::RunAnalysis },
    RecoveryPolicyRule { failure: FailureClass::AnalysisFailed, action: RecoveryAction::RunAnalysis },
    RecoveryPolicyRule { failure: FailureClass::JudgmentMissing, action: RecoveryAction::Rejudge },
    RecoveryPolicyRule { failure: FailureClass::JudgmentFailed, action: RecoveryAction::Rejudge },
    RecoveryPolicyRule { failure: FailureClass::PlanMissing, action: RecoveryAction::BindReadyTask },
    RecoveryPolicyRule { failure: FailureClass::PlanFailed, action: RecoveryAction::Replan },
    RecoveryPolicyRule { failure: FailureClass::PlanReadyQueueEmpty, action: RecoveryAction::BindReadyTask },
    RecoveryPolicyRule { failure: FailureClass::ExecutionMissing, action: RecoveryAction::Reexecute },
    RecoveryPolicyRule { failure: FailureClass::ExecutionFailed, action: RecoveryAction::Reexecute },
    RecoveryPolicyRule { failure: FailureClass::TaskReceiptMissing, action: RecoveryAction::Reexecute },
    RecoveryPolicyRule { failure: FailureClass::VerificationUnknown, action: RecoveryAction::Reverify },
    RecoveryPolicyRule { failure: FailureClass::VerificationFailed, action: RecoveryAction::Reverify },
    RecoveryPolicyRule { failure: FailureClass::ArtifactLineageBroken, action: RecoveryAction::RepairArtifactLineage },
    RecoveryPolicyRule { failure: FailureClass::EvalMissing, action: RecoveryAction::RecomputeEval },
    RecoveryPolicyRule { failure: FailureClass::EvalFailed, action: RecoveryAction::RecomputeEval },
    RecoveryPolicyRule { failure: FailureClass::RecoveryExhausted, action: RecoveryAction::Escalate },
    RecoveryPolicyRule { failure: FailureClass::ConvergenceFailed, action: RecoveryAction::Escalate },
    RecoveryPolicyRule { failure: FailureClass::LearningMissing, action: RecoveryAction::RecomputeEval },
    RecoveryPolicyRule { failure: FailureClass::LearningFailed, action: RecoveryAction::RecomputeEval },
];

pub(crate) fn recovery_policy_coverage_count() -> usize {
    RECOVERY_POLICY.len()
}

pub(crate) fn recovery_action_for(class: FailureClass) -> RecoveryAction {
    RECOVERY_POLICY
        .iter()
        .find(|rule| rule.failure == class)
        .map(|rule| rule.action)
        .expect("recovery policy must cover every failure class")
}

pub(crate) fn failure_for_gate(id: GateId, status: GateStatus) -> FailureClass {
    match (id, status) {
        (GateId::Invariant, GateStatus::Unknown) => FailureClass::InvariantUnknown,
        (GateId::Invariant, GateStatus::Fail) => FailureClass::InvariantBlocked,

        (GateId::Analysis, GateStatus::Unknown) => FailureClass::AnalysisMissing,
        (GateId::Analysis, GateStatus::Fail) => FailureClass::AnalysisFailed,

        (GateId::Judgment, GateStatus::Unknown) => FailureClass::JudgmentMissing,
        (GateId::Judgment, GateStatus::Fail) => FailureClass::JudgmentFailed,

        (GateId::Plan, GateStatus::Unknown) => FailureClass::PlanMissing,
        (GateId::Plan, GateStatus::Fail) => FailureClass::PlanFailed,

        (GateId::Execution, GateStatus::Unknown) => FailureClass::ExecutionMissing,
        (GateId::Execution, GateStatus::Fail) => FailureClass::ExecutionFailed,

        (GateId::Verification, GateStatus::Unknown) => FailureClass::VerificationUnknown,
        (GateId::Verification, GateStatus::Fail) => FailureClass::VerificationFailed,

        (GateId::Eval, GateStatus::Unknown) => FailureClass::EvalMissing,
        (GateId::Eval, GateStatus::Fail) => FailureClass::EvalFailed,

        (GateId::Learning, GateStatus::Unknown) => FailureClass::LearningMissing,
        (GateId::Learning, GateStatus::Fail) => FailureClass::LearningFailed,

        (_, GateStatus::Pass) => unreachable!("passing gate cannot produce failure"),
    }
}

pub(crate) fn event_kind_for_failure(class: FailureClass) -> EventKind {
    match class {
        FailureClass::InvariantUnknown | FailureClass::InvariantBlocked => EventKind::Blocked,
        _ => EventKind::Failed,
    }
}

pub(crate) fn decision_for_failure(class: FailureClass) -> Decision {
    match class {
        FailureClass::InvariantUnknown | FailureClass::InvariantBlocked => Decision::Block,
        _ => Decision::Fail,
    }
}

pub(crate) fn evidence_for_gate(id: GateId) -> Evidence {
    match id {
        GateId::Invariant => Evidence::InvariantProof,
        GateId::Analysis => Evidence::AnalysisReport,
        GateId::Judgment => Evidence::JudgmentRecord,
        GateId::Plan => Evidence::TaskReady,
        GateId::Execution => Evidence::ArtifactReceipt,
        GateId::Verification => Evidence::LineageProof,
        GateId::Eval => Evidence::EvalScore,
        GateId::Learning => Evidence::PolicyPromotion,
    }
}
