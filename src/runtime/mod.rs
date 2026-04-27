//! Runtime reducer and in-memory execution loop.
//!
//! This layer owns phase advancement policy. Capabilities submit evidence;
//! the frozen kernel only carries the evidence token and gate state.

use std::path::Path;

use crate::codec::ndjson::append_tlog_ndjson;
use crate::kernel::{
    Cause, ControlEvent, Decision, EventKind, Evidence, FailureClass, Gate, GateId,
    GateSet, GateStatus, Packet, Phase, RecoveryAction, RuntimeConfig, SemanticDelta,
    State, TLog, GATE_ORDER, PHASES,
};

pub mod durable;
pub mod verify;

pub use self::durable::{run_until_done_durable, tick_durable};
pub use self::verify::{legal_transition, replay_tlog_ndjson, verify_tlog, verify_tlog_from};

use self::verify::{hash_event, validate_event, EventHashInput, EventView};


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Outcome {
    state: State,
    kind: EventKind,
    cause: Cause,
    evidence: Evidence,
    decision: Decision,
    failure: Option<FailureClass>,
    recovery_action: Option<RecoveryAction>,
    affected_gate: Option<GateId>,
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

#[derive(Clone, Copy)]
struct Transition {
    from: Phase,
    to: Phase,
    kind: EventKind,
    cause: Cause,
}

const TRANSITIONS: [Transition; 38] = [
    Transition { from: Phase::Delta, to: Phase::Invariant, kind: EventKind::Advanced, cause: Cause::Start },
    Transition { from: Phase::Invariant, to: Phase::Analysis, kind: EventKind::Advanced, cause: Cause::GatePassed },
    Transition { from: Phase::Invariant, to: Phase::Recovery, kind: EventKind::Blocked, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Invariant, to: Phase::Recovery, kind: EventKind::Blocked, cause: Cause::GateFailed },
    Transition { from: Phase::Analysis, to: Phase::Judgment, kind: EventKind::Advanced, cause: Cause::GatePassed },
    Transition { from: Phase::Analysis, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Analysis, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Judgment, to: Phase::Plan, kind: EventKind::Advanced, cause: Cause::JudgmentMade },
    Transition { from: Phase::Judgment, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Judgment, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Plan, to: Phase::Execute, kind: EventKind::Advanced, cause: Cause::PlanReady },
    Transition { from: Phase::Plan, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Plan, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Plan, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::ReadyQueueEmpty },
    Transition { from: Phase::Execute, to: Phase::Verify, kind: EventKind::Advanced, cause: Cause::ExecutionFinished },
    Transition { from: Phase::Execute, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Execute, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Execute, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::TaskReceiptMissing },
    Transition { from: Phase::Verify, to: Phase::Eval, kind: EventKind::Advanced, cause: Cause::VerificationPassed },
    Transition { from: Phase::Verify, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Verify, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Verify, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::ArtifactLineageBroken },
    Transition { from: Phase::Eval, to: Phase::Persist, kind: EventKind::Advanced, cause: Cause::EvalPassed },
    Transition { from: Phase::Eval, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Eval, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Eval, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvalFailed },
    Transition { from: Phase::Recovery, to: Phase::Persist, kind: EventKind::Recovered, cause: Cause::RepairSelected },
    Transition { from: Phase::Recovery, to: Phase::Done, kind: EventKind::Failed, cause: Cause::RecoveryLimit },
    Transition { from: Phase::Persist, to: Phase::Invariant, kind: EventKind::Persisted, cause: Cause::RepairApplied },
    Transition { from: Phase::Persist, to: Phase::Analysis, kind: EventKind::Persisted, cause: Cause::RepairApplied },
    Transition { from: Phase::Persist, to: Phase::Judgment, kind: EventKind::Persisted, cause: Cause::RepairApplied },
    Transition { from: Phase::Persist, to: Phase::Plan, kind: EventKind::Persisted, cause: Cause::RepairApplied },
    Transition { from: Phase::Persist, to: Phase::Execute, kind: EventKind::Persisted, cause: Cause::RepairApplied },
    Transition { from: Phase::Persist, to: Phase::Verify, kind: EventKind::Persisted, cause: Cause::RepairApplied },
    Transition { from: Phase::Persist, to: Phase::Eval, kind: EventKind::Persisted, cause: Cause::RepairApplied },
    Transition { from: Phase::Persist, to: Phase::Learn, kind: EventKind::Persisted, cause: Cause::Persisted },
    Transition { from: Phase::Learn, to: Phase::Done, kind: EventKind::Learned, cause: Cause::PolicyPromoted },
    Transition { from: Phase::Done, to: Phase::Done, kind: EventKind::Completed, cause: Cause::EvalPassed },
];

pub(crate) struct CanonicalWriter;

impl CanonicalWriter {
    pub(crate) fn build(
        tlog: &[ControlEvent],
        before: State,
        outcome: Outcome,
        cfg: RuntimeConfig,
    ) -> Result<ControlEvent, CanonError> {
        let after = outcome.state;
        let delta = semantic_diff(before, after);

        validate_event(EventView {
            from: before.phase,
            to: after.phase,
            kind: outcome.kind,
            cause: outcome.cause,
            decision: outcome.decision,
            failure: outcome.failure,
            recovery_action: outcome.recovery_action,
            affected_gate: outcome.affected_gate,
        })?;

        if after.phase == Phase::Done && after.failure.is_none() && !after.is_success() {
            return Err(CanonError::InvalidCompletion);
        }

        let seq = tlog.len() as u64 + 1;
        let prev_hash = tlog.last().map(|e| e.self_hash).unwrap_or(0);
        let self_hash = hash_event(EventHashInput {
            seq,
            prev_hash,
            from: before.phase,
            to: after.phase,
            kind: outcome.kind,
            cause: outcome.cause,
            delta,
            evidence: outcome.evidence,
            decision: outcome.decision,
            failure: outcome.failure,
            recovery_action: outcome.recovery_action,
            affected_gate: outcome.affected_gate,
            runtime_config: cfg,
            state_before: before,
            state_after: after,
        });

        Ok(ControlEvent {
            seq,
            from: before.phase,
            to: after.phase,
            kind: outcome.kind,
            cause: outcome.cause,
            delta,
            evidence: outcome.evidence,
            decision: outcome.decision,
            failure: outcome.failure,
            recovery_action: outcome.recovery_action,
            affected_gate: outcome.affected_gate,
            runtime_config: cfg,
            state_before: before,
            state_after: after,
            prev_hash,
            self_hash,
        })
    }

    fn append(
        tlog: &mut TLog,
        before: State,
        outcome: Outcome,
        cfg: RuntimeConfig,
    ) -> Result<ControlEvent, CanonError> {
        let event = Self::build(tlog, before, outcome, cfg)?;
        tlog.push(event);
        Ok(event)
    }

    pub(crate) fn append_durable(
        tlog: &mut TLog,
        tlog_path: impl AsRef<Path>,
        before: State,
        outcome: Outcome,
        cfg: RuntimeConfig,
    ) -> Result<ControlEvent, CanonError> {
        let event = Self::build(tlog, before, outcome, cfg)?;
        append_tlog_ndjson(tlog_path, &event)?;
        tlog.push(event);
        Ok(event)
    }
}
pub(crate) fn reduce(input: State, cfg: RuntimeConfig) -> Outcome {
    match input.phase {
        Phase::Delta => {
            let mut s = input;
            advance(
                &mut s,
                Phase::Invariant,
                Cause::Start,
                Evidence::DeltaComputed,
            )
        }
        Phase::Invariant => gate_step(input, GateId::Invariant, Phase::Analysis, Cause::GatePassed),
        Phase::Analysis => gate_step(input, GateId::Analysis, Phase::Judgment, Cause::GatePassed),
        Phase::Judgment => gate_step(input, GateId::Judgment, Phase::Plan, Cause::JudgmentMade),
        Phase::Plan => plan_step(input),
        Phase::Execute => execute_step(input),
        Phase::Verify => verify_step(input),
        Phase::Eval => eval_step(input),
        Phase::Recovery => recover(input, cfg),
        Phase::Persist => persist(input),
        Phase::Learn => learn(input),
        Phase::Done => {
            let mut s = input;
            complete(&mut s)
        }
    }
}

fn gate_step(input: State, gate_id: GateId, next: Phase, pass_cause: Cause) -> Outcome {
    let mut s = input;
    let gate = s.gates.get(gate_id);

    match gate.status {
        GateStatus::Pass => advance(&mut s, next, pass_cause, gate.evidence),
        GateStatus::Unknown | GateStatus::Fail => raise_gate_failure(&mut s, gate_id, gate),
    }
}

fn plan_step(input: State) -> Outcome {
    let mut s = input;
    let gate = s.gates.plan;

    match gate.status {
        GateStatus::Pass if s.packet.has_ready_task() || s.packet.objective_complete() => {
            advance(&mut s, Phase::Execute, Cause::PlanReady, gate.evidence)
        }
        GateStatus::Pass => raise_domain_failure(
            &mut s,
            FailureClass::PlanReadyQueueEmpty,
            Cause::ReadyQueueEmpty,
            Evidence::Missing,
            GateId::Plan,
        ),
        GateStatus::Unknown | GateStatus::Fail => raise_gate_failure(&mut s, GateId::Plan, gate),
    }
}

fn execute_step(input: State) -> Outcome {
    let mut s = input;
    let gate = s.gates.execution;

    match gate.status {
        GateStatus::Pass if s.packet.artifact_receipt_valid() => {
            advance(&mut s, Phase::Verify, Cause::ExecutionFinished, gate.evidence)
        }
        GateStatus::Pass => raise_domain_failure(
            &mut s,
            FailureClass::TaskReceiptMissing,
            Cause::TaskReceiptMissing,
            Evidence::Missing,
            GateId::Execution,
        ),
        GateStatus::Unknown | GateStatus::Fail => {
            raise_gate_failure(&mut s, GateId::Execution, gate)
        }
    }
}

fn verify_step(input: State) -> Outcome {
    let mut s = input;
    let gate = s.gates.verification;

    match gate.status {
        GateStatus::Pass if s.packet.lineage_valid() => {
            advance(&mut s, Phase::Eval, Cause::VerificationPassed, gate.evidence)
        }
        GateStatus::Pass => raise_domain_failure(
            &mut s,
            FailureClass::ArtifactLineageBroken,
            Cause::ArtifactLineageBroken,
            Evidence::Missing,
            GateId::Verification,
        ),
        GateStatus::Unknown | GateStatus::Fail => {
            raise_gate_failure(&mut s, GateId::Verification, gate)
        }
    }
}

fn eval_step(input: State) -> Outcome {
    let mut s = input;

    if let Some((gate_id, gate)) = s.gates.first_execution_non_pass() {
        return raise_gate_failure(&mut s, gate_id, gate);
    }

    if !s.packet.objective_complete() {
        return raise_domain_failure(
            &mut s,
            FailureClass::EvalFailed,
            Cause::EvalFailed,
            Evidence::Missing,
            GateId::Eval,
        );
    }

    advance(&mut s, Phase::Persist, Cause::EvalPassed, Evidence::EvalScore)
}

fn advance(s: &mut State, to: Phase, cause: Cause, evidence: Evidence) -> Outcome {
    s.phase = to;
    s.failure = None;
    s.recovery_action = None;

    Outcome {
        state: *s,
        kind: EventKind::Advanced,
        cause,
        evidence,
        decision: Decision::Continue,
        failure: None,
        recovery_action: None,
        affected_gate: None,
    }
}

fn complete(s: &mut State) -> Outcome {
    s.phase = Phase::Done;
    s.failure = None;
    s.recovery_action = None;

    Outcome {
        state: *s,
        kind: EventKind::Completed,
        cause: Cause::EvalPassed,
        evidence: Evidence::CompletionProof,
        decision: Decision::Complete,
        failure: None,
        recovery_action: None,
        affected_gate: None,
    }
}

fn raise_gate_failure(s: &mut State, gate_id: GateId, gate: Gate) -> Outcome {
    let class = failure_for_gate(gate_id, gate.status);
    let kind = event_kind_for_failure(class);
    let decision = decision_for_failure(class);
    let cause = match gate.status {
        GateStatus::Unknown => Cause::EvidenceMissing,
        GateStatus::Fail => Cause::GateFailed,
        GateStatus::Pass => unreachable!("passing gate cannot raise failure"),
    };
    let evidence = match gate.status {
        GateStatus::Unknown => Evidence::Missing,
        GateStatus::Fail => gate.evidence,
        GateStatus::Pass => unreachable!("passing gate cannot raise failure"),
    };

    s.phase = Phase::Recovery;
    s.failure = Some(class);
    s.recovery_action = None;

    Outcome {
        state: *s,
        kind,
        cause,
        evidence,
        decision,
        failure: Some(class),
        recovery_action: None,
        affected_gate: Some(gate_id),
    }
}

fn raise_domain_failure(
    s: &mut State,
    class: FailureClass,
    cause: Cause,
    evidence: Evidence,
    gate_id: GateId,
) -> Outcome {
    s.phase = Phase::Recovery;
    s.failure = Some(class);
    s.recovery_action = None;

    Outcome {
        state: *s,
        kind: EventKind::Failed,
        cause,
        evidence,
        decision: Decision::Fail,
        failure: Some(class),
        recovery_action: None,
        affected_gate: Some(gate_id),
    }
}

fn recover(input: State, cfg: RuntimeConfig) -> Outcome {
    let mut s = input;

    if input.recovery_attempts >= cfg.max_recovery_attempts {
        return halt_recovery(
            &mut s,
            FailureClass::RecoveryExhausted,
            Cause::RecoveryLimit,
        );
    }

    let failure = input.failure.unwrap_or(FailureClass::RecoveryExhausted);
    let action = recovery_action_for(failure);

    if action == RecoveryAction::Escalate {
        return halt_recovery(&mut s, failure, Cause::RecoveryLimit);
    }

    s.phase = Phase::Persist;
    s.recovery_attempts = s.recovery_attempts.saturating_add(1);
    s.recovery_action = Some(action);

    Outcome {
        state: s,
        kind: EventKind::Recovered,
        cause: Cause::RepairSelected,
        evidence: Evidence::RecoveryPolicy,
        decision: Decision::Repair,
        failure: Some(failure),
        recovery_action: Some(action),
        affected_gate: None,
    }
}

fn halt_recovery(s: &mut State, class: FailureClass, cause: Cause) -> Outcome {
    s.phase = Phase::Done;
    s.failure = Some(class);
    s.recovery_action = Some(RecoveryAction::Escalate);

    Outcome {
        state: *s,
        kind: EventKind::Failed,
        cause,
        evidence: Evidence::ConvergenceLimit,
        decision: Decision::Halt,
        failure: Some(class),
        recovery_action: Some(RecoveryAction::Escalate),
        affected_gate: None,
    }
}

fn persist(input: State) -> Outcome {
    let mut s = input;

    let Some(action) = input.recovery_action else {
        s.phase = Phase::Learn;

        return Outcome {
            state: s,
            kind: EventKind::Persisted,
            cause: Cause::Persisted,
            evidence: Evidence::PersistedRecord,
            decision: Decision::Continue,
            failure: None,
            recovery_action: None,
            affected_gate: None,
        };
    };

    if action == RecoveryAction::Escalate {
        s.phase = Phase::Done;
        s.failure = Some(input.failure.unwrap_or(FailureClass::RecoveryExhausted));

        return Outcome {
            state: s,
            kind: EventKind::Persisted,
            cause: Cause::RepairApplied,
            evidence: Evidence::ConvergenceLimit,
            decision: Decision::Halt,
            failure: s.failure,
            recovery_action: Some(action),
            affected_gate: None,
        };
    }

    apply_repair(&mut s, action);

    let gate = action
        .repaired_gate()
        .expect("non-escalation repair action must target a gate");
    let evidence = action
        .produced_evidence()
        .expect("non-escalation repair action must produce evidence");

    s.gates.set_pass(gate, evidence);
    s.phase = action.target();
    s.failure = None;
    s.recovery_action = None;

    Outcome {
        state: s,
        kind: EventKind::Persisted,
        cause: Cause::RepairApplied,
        evidence,
        decision: Decision::Continue,
        failure: input.failure,
        recovery_action: Some(action),
        affected_gate: Some(gate),
    }
}

fn learn(input: State) -> Outcome {
    let mut s = input;

    s.gates.set_pass(GateId::Learning, Evidence::PolicyPromotion);
    s.phase = Phase::Done;
    s.failure = None;
    s.recovery_action = None;

    Outcome {
        state: s,
        kind: EventKind::Learned,
        cause: Cause::PolicyPromoted,
        evidence: Evidence::PolicyPromotion,
        decision: Decision::Complete,
        failure: None,
        recovery_action: None,
        affected_gate: Some(GateId::Learning),
    }
}

fn apply_repair(s: &mut State, action: RecoveryAction) {
    match action {
        RecoveryAction::RecheckInvariant => {
            s.packet.objective_id = s.packet.objective_id.max(1);
            s.packet.objective_required_tasks = s.packet.objective_required_tasks.max(1);
        }
        RecoveryAction::RunAnalysis | RecoveryAction::Rejudge => {
            s.packet.revision = s.packet.revision.saturating_add(1);
        }
        RecoveryAction::Replan | RecoveryAction::BindReadyTask => {
            s.packet.bind_ready_task();
        }
        RecoveryAction::Reexecute => {
            s.packet.materialize_artifact();
        }
        RecoveryAction::Reverify | RecoveryAction::RepairArtifactLineage => {
            s.packet.repair_lineage();
        }
        RecoveryAction::RecomputeEval => {
            s.packet.complete_objective();
        }
        RecoveryAction::Escalate => {}
    }
}

fn recovery_action_for(class: FailureClass) -> RecoveryAction {
    match class {
        FailureClass::InvariantUnknown | FailureClass::InvariantBlocked => {
            RecoveryAction::RecheckInvariant
        }
        FailureClass::AnalysisMissing | FailureClass::AnalysisFailed => RecoveryAction::RunAnalysis,
        FailureClass::JudgmentMissing | FailureClass::JudgmentFailed => RecoveryAction::Rejudge,
        FailureClass::PlanMissing => RecoveryAction::BindReadyTask,
        FailureClass::PlanFailed => RecoveryAction::Replan,
        FailureClass::PlanReadyQueueEmpty => RecoveryAction::BindReadyTask,
        FailureClass::ExecutionMissing
        | FailureClass::ExecutionFailed
        | FailureClass::TaskReceiptMissing => RecoveryAction::Reexecute,
        FailureClass::VerificationUnknown | FailureClass::VerificationFailed => {
            RecoveryAction::Reverify
        }
        FailureClass::ArtifactLineageBroken => RecoveryAction::RepairArtifactLineage,
        FailureClass::EvalMissing | FailureClass::EvalFailed => RecoveryAction::RecomputeEval,
        FailureClass::LearningMissing | FailureClass::LearningFailed => RecoveryAction::RecomputeEval,
        FailureClass::RecoveryExhausted | FailureClass::ConvergenceFailed => {
            RecoveryAction::Escalate
        }
    }
}

fn failure_for_gate(id: GateId, status: GateStatus) -> FailureClass {
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

fn event_kind_for_failure(class: FailureClass) -> EventKind {
    match class {
        FailureClass::InvariantUnknown | FailureClass::InvariantBlocked => EventKind::Blocked,
        _ => EventKind::Failed,
    }
}

fn decision_for_failure(class: FailureClass) -> Decision {
    match class {
        FailureClass::InvariantUnknown | FailureClass::InvariantBlocked => Decision::Block,
        _ => Decision::Fail,
    }
}

fn evidence_for_gate(id: GateId) -> Evidence {
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

pub fn semantic_diff(a: State, b: State) -> SemanticDelta {
    if a == b {
        return SemanticDelta::NoChange;
    }
    if a.phase == Phase::Learn && b.phase == Phase::Done {
        return SemanticDelta::LearningPromoted;
    }
    if b.phase == Phase::Done && b.failure.is_none() {
        return SemanticDelta::Completed;
    }
    if b.phase == Phase::Done && b.failure.is_some() {
        return SemanticDelta::Halted;
    }
    if a.phase == Phase::Recovery && b.phase == Phase::Persist {
        return SemanticDelta::RepairSelected;
    }
    if a.phase == Phase::Persist && a.recovery_action.is_some() {
        return SemanticDelta::RepairApplied;
    }
    if a.phase == Phase::Persist && b.phase == Phase::Learn {
        return SemanticDelta::Persisted;
    }
    if b.failure.is_some() && b.phase == Phase::Recovery {
        return SemanticDelta::FailureRaised;
    }
    if a.packet != b.packet {
        return SemanticDelta::PayloadChanged;
    }
    if a.phase != b.phase {
        return SemanticDelta::PhaseAdvanced;
    }
    SemanticDelta::NoChange
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
