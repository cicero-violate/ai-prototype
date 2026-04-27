//! Runtime reducer and in-memory execution loop.
//!
//! This layer owns phase advancement policy. Capabilities submit evidence;
//! the frozen kernel only carries the evidence token and gate state.

use crate::kernel::{
    Cause, Decision, EventKind, Evidence, FailureClass, GateId, GateSet, GateStatus,
    Packet, Phase, RecoveryAction, RuntimeConfig, SemanticDelta,
    State, TLog, GATE_ORDER, PHASES,
};

pub(crate) mod diff;
pub mod durable;
pub(crate) mod recovery_policy;
pub(crate) mod reducer;
pub(crate) mod transition_table;
pub mod verify;
pub(crate) mod writer;

pub use self::diff::semantic_diff;
pub use self::durable::{run_until_done_durable, tick_durable};
pub use self::verify::{legal_transition, replay_tlog_ndjson, verify_tlog, verify_tlog_from};

use self::recovery_policy::{evidence_for_gate, recovery_policy_coverage_count};
pub(crate) use self::reducer::reduce;
use self::transition_table::TRANSITIONS;
pub(crate) use self::writer::CanonicalWriter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Outcome {
    pub(crate) state: State,
    pub(crate) kind: EventKind,
    pub(crate) cause: Cause,
    pub(crate) evidence: Evidence,
    pub(crate) decision: Decision,
    pub(crate) failure: Option<FailureClass>,
    pub(crate) recovery_action: Option<RecoveryAction>,
    pub(crate) affected_gate: Option<GateId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonError {
    IllegalEvent {
        from: Phase,
        to: Phase,
        kind: EventKind,
    },
    MissingFailureClass,
    UnexpectedFailureClass,
    MissingRecoveryAction,
    UnexpectedRecoveryAction,
    InvalidLearnTarget,
    InvalidRepairTarget,
    InvalidCompletion,
    InvalidStateContinuity,
    InvalidPacketContinuity,
    InvalidSemanticDelta,
    InvalidHashChain,
    InvalidReplay,
    TlogIo,
    InvalidTlogRecord,
    MissingAffectedGate,
    UnexpectedAffectedGate,
}

impl core::fmt::Display for CanonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for CanonError {}

pub fn tick(state: &mut State, tlog: &mut TLog, cfg: RuntimeConfig) -> Result<(), CanonError> {
    let before = *state;
    let outcome = reduce(before, cfg);
    let event = CanonicalWriter::append(tlog, before, outcome, cfg)?;
    *state = event.state_after;
    Ok(())
}

pub fn run_until_done(mut state: State, cfg: RuntimeConfig) -> Result<(State, TLog), CanonError> {
    let mut tlog = Vec::new();

    for _ in 0..cfg.max_steps {
        if state.phase == Phase::Done {
            return Ok((state, tlog));
        }

        tick(&mut state, &mut tlog, cfg)?;
    }

    append_convergence_failure(&mut state, &mut tlog, cfg)?;
    Ok((state, tlog))
}

pub(crate) fn convergence_outcome(before: State) -> Outcome {
    let mut state = before;
    state.phase = Phase::Done;
    state.failure = Some(FailureClass::ConvergenceFailed);
    state.recovery_action = Some(RecoveryAction::Escalate);

    Outcome {
        state,
        kind: EventKind::Failed,
        cause: Cause::MaxSteps,
        evidence: Evidence::ConvergenceLimit,
        decision: Decision::Halt,
        failure: Some(FailureClass::ConvergenceFailed),
        recovery_action: Some(RecoveryAction::Escalate),
        affected_gate: None,
    }
}

fn append_convergence_failure(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
) -> Result<(), CanonError> {
    let before = *state;
    let event = CanonicalWriter::append(tlog, before, convergence_outcome(before), cfg)?;
    *state = event.state_after;
    Ok(())
}

pub fn touch_all_surfaces() -> usize {
    let statuses = [GateStatus::Unknown, GateStatus::Pass, GateStatus::Fail];
    let evidences = [
        Evidence::Missing,
        Evidence::DeltaComputed,
        Evidence::InvariantProof,
        Evidence::AnalysisReport,
        Evidence::JudgmentRecord,
        Evidence::PlanRecord,
        Evidence::TaskReady,
        Evidence::ExecutionReceipt,
        Evidence::ArtifactReceipt,
        Evidence::VerificationReport,
        Evidence::LineageProof,
        Evidence::EvalScore,
        Evidence::RecoveryPolicy,
        Evidence::CompletionProof,
        Evidence::ConvergenceLimit,
        Evidence::PersistedRecord,
        Evidence::LearningRecord,
        Evidence::PolicyPromotion,
    ];
    let failures = [
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
    let actions = [
        RecoveryAction::RecheckInvariant,
        RecoveryAction::RunAnalysis,
        RecoveryAction::Rejudge,
        RecoveryAction::Replan,
        RecoveryAction::BindReadyTask,
        RecoveryAction::Reexecute,
        RecoveryAction::Reverify,
        RecoveryAction::RepairArtifactLineage,
        RecoveryAction::RecomputeEval,
        RecoveryAction::Escalate,
    ];
    let kinds = [
        EventKind::Advanced,
        EventKind::Blocked,
        EventKind::Failed,
        EventKind::Recovered,
        EventKind::Learned,
        EventKind::Completed,
        EventKind::Persisted,
    ];
    let causes = [
        Cause::Start,
        Cause::GatePassed,
        Cause::GateFailed,
        Cause::EvidenceMissing,
        Cause::JudgmentMade,
        Cause::PlanReady,
        Cause::ReadyQueueEmpty,
        Cause::ExecutionFinished,
        Cause::TaskReceiptMissing,
        Cause::VerificationPassed,
        Cause::ArtifactLineageBroken,
        Cause::EvalPassed,
        Cause::EvalFailed,
        Cause::RepairSelected,
        Cause::RepairApplied,
        Cause::RecoveryLimit,
        Cause::MaxSteps,
        Cause::Persisted,
        Cause::PolicyPromoted,
    ];
    let decisions = [
        Decision::Continue,
        Decision::Complete,
        Decision::Block,
        Decision::Fail,
        Decision::Repair,
        Decision::Halt,
    ];
    let deltas = [
        SemanticDelta::NoChange,
        SemanticDelta::PhaseAdvanced,
        SemanticDelta::FailureRaised,
        SemanticDelta::RepairSelected,
        SemanticDelta::RepairApplied,
        SemanticDelta::PayloadChanged,
        SemanticDelta::Completed,
        SemanticDelta::Halted,
        SemanticDelta::Persisted,
        SemanticDelta::LearningPromoted,
    ];
    let errors = [
        CanonError::IllegalEvent {
            from: Phase::Delta,
            to: Phase::Done,
            kind: EventKind::Failed,
        },
        CanonError::MissingFailureClass,
        CanonError::UnexpectedFailureClass,
        CanonError::MissingRecoveryAction,
        CanonError::UnexpectedRecoveryAction,
        CanonError::InvalidLearnTarget,
        CanonError::InvalidRepairTarget,
        CanonError::InvalidCompletion,
        CanonError::InvalidStateContinuity,
        CanonError::InvalidPacketContinuity,
        CanonError::InvalidSemanticDelta,
        CanonError::InvalidHashChain,
        CanonError::InvalidReplay,
        CanonError::MissingAffectedGate,
        CanonError::UnexpectedAffectedGate,
    ];

    let mut gates = GateSet::default();
    gates.set_fail(GateId::Eval, Evidence::EvalScore);
    gates.set_pass(GateId::Eval, evidence_for_gate(GateId::Eval));

    let mut packet = Packet::empty();
    packet.bind_ready_task();
    packet.materialize_artifact();
    packet.repair_lineage();
    packet.complete_objective();

    let error_score = errors
        .iter()
        .map(|e| match e {
            CanonError::IllegalEvent { from, to, kind } => {
                *from as usize + *to as usize + *kind as usize
            }
            CanonError::MissingFailureClass => 1,
            CanonError::UnexpectedFailureClass => 2,
            CanonError::MissingRecoveryAction => 3,
            CanonError::UnexpectedRecoveryAction => 4,
            CanonError::InvalidLearnTarget => 5,
            CanonError::InvalidRepairTarget => 6,
            CanonError::InvalidCompletion => 7,
            CanonError::InvalidStateContinuity => 8,
            CanonError::InvalidPacketContinuity => 9,
            CanonError::InvalidSemanticDelta => 10,
            CanonError::InvalidHashChain => 11,
            CanonError::InvalidReplay => 12,
            CanonError::MissingAffectedGate => 13,
            CanonError::UnexpectedAffectedGate => 14,
            CanonError::TlogIo => 15,
            CanonError::InvalidTlogRecord => 16,
        })
        .sum::<usize>();

    PHASES.len()
        + statuses.len()
        + GATE_ORDER.len()
        + TRANSITIONS.len()
        + recovery_policy_coverage_count()
        + evidences.len()
        + failures.len()
        + actions.len()
        + kinds.len()
        + causes.len()
        + decisions.len()
        + deltas.len()
        + gates.all_passed() as usize
        + packet.lineage_valid() as usize
        + error_score
}
