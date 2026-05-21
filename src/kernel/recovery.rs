use super::{Evidence, GateId, Phase};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FailureClass {
    InvariantUnknown = 1,
    InvariantBlocked = 2,
    AnalysisMissing = 3,
    AnalysisFailed = 4,
    JudgmentMissing = 5,
    JudgmentFailed = 6,
    PlanMissing = 7,
    PlanFailed = 8,
    PlanReadyQueueEmpty = 9,
    ExecutionMissing = 10,
    ExecutionFailed = 11,
    TaskReceiptMissing = 12,
    VerificationUnknown = 13,
    VerificationFailed = 14,
    ArtifactLineageBroken = 15,
    EvalMissing = 16,
    EvalFailed = 17,
    RecoveryExhausted = 18,
    ConvergenceFailed = 19,
    LearningMissing = 20,
    LearningFailed = 21,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RecoveryAction {
    RecheckInvariant = 1,
    RunAnalysis = 2,
    Rejudge = 3,
    Replan = 4,
    BindReadyTask = 5,
    Reexecute = 6,
    Reverify = 7,
    RepairArtifactLineage = 8,
    RecomputeEval = 9,
    Escalate = 10,
}

impl FailureClass {
    pub const ALL: [FailureClass; 21] = [
        FailureClass::InvariantUnknown,
        FailureClass::InvariantBlocked,
        FailureClass::AnalysisMissing,
        FailureClass::AnalysisFailed,
        FailureClass::JudgmentMissing,
        FailureClass::JudgmentFailed,
        FailureClass::PlanMissing,
        FailureClass::PlanFailed,
        FailureClass::PlanReadyQueueEmpty,
        FailureClass::ExecutionMissing,
        FailureClass::ExecutionFailed,
        FailureClass::TaskReceiptMissing,
        FailureClass::VerificationUnknown,
        FailureClass::VerificationFailed,
        FailureClass::ArtifactLineageBroken,
        FailureClass::EvalMissing,
        FailureClass::EvalFailed,
        FailureClass::RecoveryExhausted,
        FailureClass::ConvergenceFailed,
        FailureClass::LearningMissing,
        FailureClass::LearningFailed,
    ];

    pub fn name(self) -> &'static str {
        match self {
            FailureClass::InvariantUnknown => "InvariantUnknown",
            FailureClass::InvariantBlocked => "InvariantBlocked",
            FailureClass::AnalysisMissing => "AnalysisMissing",
            FailureClass::AnalysisFailed => "AnalysisFailed",
            FailureClass::JudgmentMissing => "JudgmentMissing",
            FailureClass::JudgmentFailed => "JudgmentFailed",
            FailureClass::PlanMissing => "PlanMissing",
            FailureClass::PlanFailed => "PlanFailed",
            FailureClass::PlanReadyQueueEmpty => "PlanReadyQueueEmpty",
            FailureClass::ExecutionMissing => "ExecutionMissing",
            FailureClass::ExecutionFailed => "ExecutionFailed",
            FailureClass::TaskReceiptMissing => "TaskReceiptMissing",
            FailureClass::VerificationUnknown => "VerificationUnknown",
            FailureClass::VerificationFailed => "VerificationFailed",
            FailureClass::ArtifactLineageBroken => "ArtifactLineageBroken",
            FailureClass::EvalMissing => "EvalMissing",
            FailureClass::EvalFailed => "EvalFailed",
            FailureClass::RecoveryExhausted => "RecoveryExhausted",
            FailureClass::ConvergenceFailed => "ConvergenceFailed",
            FailureClass::LearningMissing => "LearningMissing",
            FailureClass::LearningFailed => "LearningFailed",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|class| class.name() == name)
    }
}

impl RecoveryAction {
    pub fn name(self) -> &'static str {
        match self {
            RecoveryAction::RecheckInvariant => "RecheckInvariant",
            RecoveryAction::RunAnalysis => "RunAnalysis",
            RecoveryAction::Rejudge => "Rejudge",
            RecoveryAction::Replan => "Replan",
            RecoveryAction::BindReadyTask => "BindReadyTask",
            RecoveryAction::Reexecute => "Reexecute",
            RecoveryAction::Reverify => "Reverify",
            RecoveryAction::RepairArtifactLineage => "RepairArtifactLineage",
            RecoveryAction::RecomputeEval => "RecomputeEval",
            RecoveryAction::Escalate => "Escalate",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "RecheckInvariant" => Some(RecoveryAction::RecheckInvariant),
            "RunAnalysis" => Some(RecoveryAction::RunAnalysis),
            "Rejudge" => Some(RecoveryAction::Rejudge),
            "Replan" => Some(RecoveryAction::Replan),
            "BindReadyTask" => Some(RecoveryAction::BindReadyTask),
            "Reexecute" => Some(RecoveryAction::Reexecute),
            "Reverify" => Some(RecoveryAction::Reverify),
            "RepairArtifactLineage" => Some(RecoveryAction::RepairArtifactLineage),
            "RecomputeEval" => Some(RecoveryAction::RecomputeEval),
            "Escalate" => Some(RecoveryAction::Escalate),
            _ => None,
        }
    }
}

impl RecoveryAction {
    pub fn target(self) -> Phase {
        match self {
            RecoveryAction::RecheckInvariant => Phase::Invariant,
            RecoveryAction::RunAnalysis => Phase::Analysis,
            RecoveryAction::Rejudge => Phase::Judgment,
            RecoveryAction::Replan | RecoveryAction::BindReadyTask => Phase::Plan,
            RecoveryAction::Reexecute => Phase::Execute,
            RecoveryAction::Reverify | RecoveryAction::RepairArtifactLineage => Phase::Verify,
            RecoveryAction::RecomputeEval => Phase::Eval,
            RecoveryAction::Escalate => Phase::Done,
        }
    }

    pub fn repaired_gate(self) -> Option<GateId> {
        match self {
            RecoveryAction::RecheckInvariant => Some(GateId::Invariant),
            RecoveryAction::RunAnalysis => Some(GateId::Analysis),
            RecoveryAction::Rejudge => Some(GateId::Judgment),
            RecoveryAction::Replan | RecoveryAction::BindReadyTask => Some(GateId::Plan),
            RecoveryAction::Reexecute => Some(GateId::Execution),
            RecoveryAction::Reverify | RecoveryAction::RepairArtifactLineage => {
                Some(GateId::Verification)
            }
            RecoveryAction::RecomputeEval => Some(GateId::Eval),
            RecoveryAction::Escalate => None,
        }
    }

    pub fn produced_evidence(self) -> Option<Evidence> {
        match self {
            RecoveryAction::RecheckInvariant => Some(Evidence::InvariantProof),
            RecoveryAction::RunAnalysis => Some(Evidence::AnalysisReport),
            RecoveryAction::Rejudge => Some(Evidence::JudgmentRecord),
            RecoveryAction::Replan => Some(Evidence::PlanRecord),
            RecoveryAction::BindReadyTask => Some(Evidence::TaskReady),
            RecoveryAction::Reexecute => Some(Evidence::ArtifactReceipt),
            RecoveryAction::Reverify => Some(Evidence::VerificationReport),
            RecoveryAction::RepairArtifactLineage => Some(Evidence::LineageProof),
            RecoveryAction::RecomputeEval => Some(Evidence::EvalScore),
            RecoveryAction::Escalate => None,
        }
    }
}
