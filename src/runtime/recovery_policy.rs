//! Total recovery and evidence policy tables.

use crate::kernel::{
    Decision, EventKind, Evidence, FailureClass, GateId, GateStatus, RecoveryAction,
    FAILURE_CLASSES,
};

pub(crate) fn recovery_policy_coverage_count() -> usize {
    FAILURE_CLASSES.len()
}

pub(crate) fn recovery_action_for(class: FailureClass) -> RecoveryAction {
    match class {
        FailureClass::InvariantUnknown | FailureClass::InvariantBlocked => {
            RecoveryAction::RecheckInvariant
        }
        FailureClass::AnalysisMissing | FailureClass::AnalysisFailed => RecoveryAction::RunAnalysis,
        FailureClass::JudgmentMissing | FailureClass::JudgmentFailed => RecoveryAction::Rejudge,
        FailureClass::PlanMissing | FailureClass::PlanReadyQueueEmpty => {
            RecoveryAction::BindReadyTask
        }
        FailureClass::PlanFailed => RecoveryAction::Replan,
        FailureClass::ExecutionMissing
        | FailureClass::ExecutionFailed
        | FailureClass::TaskReceiptMissing => RecoveryAction::Reexecute,
        FailureClass::VerificationUnknown | FailureClass::VerificationFailed => {
            RecoveryAction::Reverify
        }
        FailureClass::ArtifactLineageBroken => RecoveryAction::RepairArtifactLineage,
        FailureClass::EvalMissing
        | FailureClass::EvalFailed
        | FailureClass::LearningMissing
        | FailureClass::LearningFailed => RecoveryAction::RecomputeEval,
        FailureClass::RecoveryExhausted | FailureClass::ConvergenceFailed => {
            RecoveryAction::Escalate
        }
    }
}

pub(crate) fn failure_for_gate(id: GateId, status: GateStatus) -> Option<FailureClass> {
    match (id, status) {
        (GateId::Invariant, GateStatus::Unknown) => Some(FailureClass::InvariantUnknown),
        (GateId::Invariant, GateStatus::Fail) => Some(FailureClass::InvariantBlocked),

        (GateId::Analysis, GateStatus::Unknown) => Some(FailureClass::AnalysisMissing),
        (GateId::Analysis, GateStatus::Fail) => Some(FailureClass::AnalysisFailed),

        (GateId::Judgment, GateStatus::Unknown) => Some(FailureClass::JudgmentMissing),
        (GateId::Judgment, GateStatus::Fail) => Some(FailureClass::JudgmentFailed),

        (GateId::Plan, GateStatus::Unknown) => Some(FailureClass::PlanMissing),
        (GateId::Plan, GateStatus::Fail) => Some(FailureClass::PlanFailed),

        (GateId::Execution, GateStatus::Unknown) => Some(FailureClass::ExecutionMissing),
        (GateId::Execution, GateStatus::Fail) => Some(FailureClass::ExecutionFailed),

        (GateId::Verification, GateStatus::Unknown) => Some(FailureClass::VerificationUnknown),
        (GateId::Verification, GateStatus::Fail) => Some(FailureClass::VerificationFailed),

        (GateId::Eval, GateStatus::Unknown) => Some(FailureClass::EvalMissing),
        (GateId::Eval, GateStatus::Fail) => Some(FailureClass::EvalFailed),

        (GateId::Learning, GateStatus::Unknown) => Some(FailureClass::LearningMissing),
        (GateId::Learning, GateStatus::Fail) => Some(FailureClass::LearningFailed),

        (_, GateStatus::Pass) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passing_gate_has_no_failure_class() {
        assert_eq!(failure_for_gate(GateId::Eval, GateStatus::Pass), None);
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
