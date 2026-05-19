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
    RecoveryPolicyRule {
        failure: FailureClass::InvariantUnknown,
        action: RecoveryAction::RecheckInvariant,
    },
    RecoveryPolicyRule {
        failure: FailureClass::InvariantBlocked,
        action: RecoveryAction::RecheckInvariant,
    },
    RecoveryPolicyRule {
        failure: FailureClass::AnalysisMissing,
        action: RecoveryAction::RunAnalysis,
    },
    RecoveryPolicyRule {
        failure: FailureClass::AnalysisFailed,
        action: RecoveryAction::RunAnalysis,
    },
    RecoveryPolicyRule {
        failure: FailureClass::JudgmentMissing,
        action: RecoveryAction::Rejudge,
    },
    RecoveryPolicyRule {
        failure: FailureClass::JudgmentFailed,
        action: RecoveryAction::Rejudge,
    },
    RecoveryPolicyRule {
        failure: FailureClass::PlanMissing,
        action: RecoveryAction::BindReadyTask,
    },
    RecoveryPolicyRule {
        failure: FailureClass::PlanFailed,
        action: RecoveryAction::Replan,
    },
    RecoveryPolicyRule {
        failure: FailureClass::PlanReadyQueueEmpty,
        action: RecoveryAction::BindReadyTask,
    },
    RecoveryPolicyRule {
        failure: FailureClass::ExecutionMissing,
        action: RecoveryAction::Reexecute,
    },
    RecoveryPolicyRule {
        failure: FailureClass::ExecutionFailed,
        action: RecoveryAction::Reexecute,
    },
    RecoveryPolicyRule {
        failure: FailureClass::TaskReceiptMissing,
        action: RecoveryAction::Reexecute,
    },
    RecoveryPolicyRule {
        failure: FailureClass::VerificationUnknown,
        action: RecoveryAction::Reverify,
    },
    RecoveryPolicyRule {
        failure: FailureClass::VerificationFailed,
        action: RecoveryAction::Reverify,
    },
    RecoveryPolicyRule {
        failure: FailureClass::ArtifactLineageBroken,
        action: RecoveryAction::RepairArtifactLineage,
    },
    RecoveryPolicyRule {
        failure: FailureClass::EvalMissing,
        action: RecoveryAction::RecomputeEval,
    },
    RecoveryPolicyRule {
        failure: FailureClass::EvalFailed,
        action: RecoveryAction::RecomputeEval,
    },
    RecoveryPolicyRule {
        failure: FailureClass::RecoveryExhausted,
        action: RecoveryAction::Escalate,
    },
    RecoveryPolicyRule {
        failure: FailureClass::ConvergenceFailed,
        action: RecoveryAction::Escalate,
    },
    RecoveryPolicyRule {
        failure: FailureClass::LearningMissing,
        action: RecoveryAction::RecomputeEval,
    },
    RecoveryPolicyRule {
        failure: FailureClass::LearningFailed,
        action: RecoveryAction::RecomputeEval,
    },
];

pub(crate) fn recovery_policy_coverage_count() -> usize {
    RECOVERY_POLICY.len()
}

pub(crate) fn recovery_action_for(class: FailureClass) -> RecoveryAction {
    for rule in RECOVERY_POLICY {
        if rule.failure == class {
            return rule.action;
        }
    }

    RecoveryAction::Escalate
}

pub(crate) fn failure_for_gate(id: GateId, status: GateStatus) -> Option<FailureClass> {
    match status {
        GateStatus::Pass => None,
        GateStatus::Unknown => Some(unknown_failure_for_gate(id)),
        GateStatus::Fail => Some(failed_failure_for_gate(id)),
    }
}

fn unknown_failure_for_gate(id: GateId) -> FailureClass {
    match id {
        GateId::Invariant => FailureClass::InvariantUnknown,
        GateId::Analysis => FailureClass::AnalysisMissing,
        GateId::Judgment => FailureClass::JudgmentMissing,
        GateId::Plan => FailureClass::PlanMissing,
        GateId::Execution => FailureClass::ExecutionMissing,
        GateId::Verification => FailureClass::VerificationUnknown,
        GateId::Eval => FailureClass::EvalMissing,
        GateId::Learning => FailureClass::LearningMissing,
    }
}

fn failed_failure_for_gate(id: GateId) -> FailureClass {
    match id {
        GateId::Invariant => FailureClass::InvariantBlocked,
        GateId::Analysis => FailureClass::AnalysisFailed,
        GateId::Judgment => FailureClass::JudgmentFailed,
        GateId::Plan => FailureClass::PlanFailed,
        GateId::Execution => FailureClass::ExecutionFailed,
        GateId::Verification => FailureClass::VerificationFailed,
        GateId::Eval => FailureClass::EvalFailed,
        GateId::Learning => FailureClass::LearningFailed,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passing_gate_has_no_failure_class() {
        assert_eq!(failure_for_gate(GateId::Eval, GateStatus::Pass), None);
    }
}
