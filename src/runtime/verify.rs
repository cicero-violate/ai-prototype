//! TLog validation, replay, and deterministic event hashing.

use std::path::Path;

use crate::codec::ndjson::load_tlog_ndjson;
use crate::kernel::{
    mix, Cause, ControlEvent, Decision, EventKind, Evidence, FailureClass, GateId, GateSet,
    Packet, Phase, RecoveryAction, RuntimeConfig, SemanticDelta, State, GATE_ORDER,
};

use super::{convergence_outcome, reduce, semantic_diff, CanonError, CanonicalWriter, TRANSITIONS};

#[derive(Clone, Copy)]
pub(crate) struct EventView {
    pub(crate) from: Phase,
    pub(crate) to: Phase,
    pub(crate) kind: EventKind,
    pub(crate) cause: Cause,
    pub(crate) decision: Decision,
    pub(crate) failure: Option<FailureClass>,
    pub(crate) recovery_action: Option<RecoveryAction>,
    pub(crate) affected_gate: Option<GateId>,
}

pub(crate) fn validate_event(event: EventView) -> Result<(), CanonError> {
    if !legal_transition(event.from, event.to, event.kind, event.cause) {
        return Err(CanonError::IllegalEvent {
            from: event.from,
            to: event.to,
            kind: event.kind,
        });
    }

    if matches!(event.kind, EventKind::Blocked | EventKind::Failed | EventKind::Recovered)
        && event.failure.is_none()
    {
        return Err(CanonError::MissingFailureClass);
    }

    if matches!(
        event.kind,
        EventKind::Advanced | EventKind::Completed | EventKind::Learned
    ) && event.failure.is_some()
    {
        return Err(CanonError::UnexpectedFailureClass);
    }

    if event.kind == EventKind::Recovered && event.recovery_action.is_none() {
        return Err(CanonError::MissingRecoveryAction);
    }

    if matches!(event.kind, EventKind::Advanced | EventKind::Completed | EventKind::Learned)
        && event.recovery_action.is_some()
    {
        return Err(CanonError::UnexpectedRecoveryAction);
    }

    if event.kind == EventKind::Recovered && event.affected_gate.is_some() {
        return Err(CanonError::InvalidRepairTarget);
    }

    if matches!(event.kind, EventKind::Blocked | EventKind::Failed) {
        let Some(class) = event.failure else {
            return Err(CanonError::MissingFailureClass);
        };

        if failure_requires_gate(class) && event.affected_gate.is_none() {
            return Err(CanonError::MissingAffectedGate);
        }

        if !failure_requires_gate(class) && event.affected_gate.is_some() {
            return Err(CanonError::UnexpectedAffectedGate);
        }
    }

    if matches!(event.kind, EventKind::Advanced | EventKind::Completed | EventKind::Recovered)
        && event.affected_gate.is_some()
    {
        return Err(CanonError::UnexpectedAffectedGate);
    }

    if event.kind == EventKind::Persisted && event.cause == Cause::RepairApplied {
        let Some(action) = event.recovery_action else {
            return Err(CanonError::MissingRecoveryAction);
        };

        if action.target() != event.to {
            return Err(CanonError::InvalidLearnTarget);
        }

        if action.repaired_gate() != event.affected_gate {
            return Err(CanonError::InvalidRepairTarget);
        }
    }

    if event.kind == EventKind::Persisted && event.cause == Cause::Persisted {
        if event.to != Phase::Learn
            || event.failure.is_some()
            || event.recovery_action.is_some()
            || event.affected_gate.is_some()
        {
            return Err(CanonError::InvalidLearnTarget);
        }
    }

    if event.kind == EventKind::Learned {
        if event.to != Phase::Done || event.affected_gate != Some(GateId::Learning) {
            return Err(CanonError::InvalidLearnTarget);
        }
    }

    if event.decision == Decision::Halt && event.recovery_action != Some(RecoveryAction::Escalate) {
        return Err(CanonError::MissingRecoveryAction);
    }

    Ok(())
}

fn failure_requires_gate(class: FailureClass) -> bool {
    !matches!(
        class,
        FailureClass::RecoveryExhausted | FailureClass::ConvergenceFailed
    )
}

pub fn legal_transition(from: Phase, to: Phase, kind: EventKind, cause: Cause) -> bool {
    if to == Phase::Done && kind == EventKind::Failed && cause == Cause::MaxSteps {
        return true;
    }

    TRANSITIONS.iter().any(|transition| {
        transition.from == from
            && transition.to == to
            && transition.kind == kind
            && transition.cause == cause
    })
}

pub fn replay_tlog_ndjson(
    initial: State,
    path: impl AsRef<Path>,
) -> Result<State, CanonError> {
    let tlog = load_tlog_ndjson(path)?;
    verify_tlog_from(initial, &tlog)
}

pub fn verify_tlog(tlog: &[ControlEvent]) -> Result<(), CanonError> {
    let Some(first) = tlog.first() else {
        return Ok(());
    };

    verify_tlog_from(first.state_before, tlog).map(|_| ())
}

pub fn verify_tlog_from(initial: State, tlog: &[ControlEvent]) -> Result<State, CanonError> {
    let mut state = initial;
    let mut prev_hash = 0;

    for (i, event) in tlog.iter().enumerate() {
        if event.seq != i as u64 + 1 || event.prev_hash != prev_hash {
            return Err(CanonError::InvalidHashChain);
        }

        if event.from != state.phase || event.state_before.phase != state.phase {
            return Err(CanonError::InvalidStateContinuity);
        }

        if event.state_before.packet != state.packet {
            return Err(CanonError::InvalidPacketContinuity);
        }

        if event.state_after.phase != event.to {
            return Err(CanonError::InvalidStateContinuity);
        }

        if event.state_before != state {
            return Err(CanonError::InvalidReplay);
        }

        if event.delta != semantic_diff(event.state_before, event.state_after) {
            return Err(CanonError::InvalidSemanticDelta);
        }

        validate_event(EventView {
            from: event.from,
            to: event.to,
            kind: event.kind,
            cause: event.cause,
            decision: event.decision,
            failure: event.failure,
            recovery_action: event.recovery_action,
            affected_gate: event.affected_gate,
        })?;

        let expected = hash_event(EventHashInput {
            seq: event.seq,
            prev_hash: event.prev_hash,
            from: event.from,
            to: event.to,
            kind: event.kind,
            cause: event.cause,
            delta: event.delta,
            evidence: event.evidence,
            decision: event.decision,
            failure: event.failure,
            recovery_action: event.recovery_action,
            affected_gate: event.affected_gate,
            runtime_config: event.runtime_config,
            state_before: event.state_before,
            state_after: event.state_after,
        });

        if expected != event.self_hash {
            return Err(CanonError::InvalidHashChain);
        }

        let expected_outcome = if event.cause == Cause::MaxSteps {
            convergence_outcome(state)
        } else {
            reduce(state, event.runtime_config)
        };
        let expected_event =
            CanonicalWriter::build(&tlog[..i], state, expected_outcome, event.runtime_config)?;
        if *event != expected_event {
            return Err(CanonError::InvalidReplay);
        }

        state = event.state_after;
        prev_hash = event.self_hash;
    }

    Ok(state)
}

#[derive(Clone, Copy)]
pub(crate) struct EventHashInput {
    pub(crate) seq: u64,
    pub(crate) prev_hash: u64,
    pub(crate) from: Phase,
    pub(crate) to: Phase,
    pub(crate) kind: EventKind,
    pub(crate) cause: Cause,
    pub(crate) delta: SemanticDelta,
    pub(crate) evidence: Evidence,
    pub(crate) decision: Decision,
    pub(crate) failure: Option<FailureClass>,
    pub(crate) recovery_action: Option<RecoveryAction>,
    pub(crate) affected_gate: Option<GateId>,
    pub(crate) runtime_config: RuntimeConfig,
    pub(crate) state_before: State,
    pub(crate) state_after: State,
}

pub(crate) fn hash_event(input: EventHashInput) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    h = mix(h, input.seq);
    h = mix(h, input.prev_hash);
    h = mix(h, input.from as u64);
    h = mix(h, input.to as u64);
    h = mix(h, input.kind as u64);
    h = mix(h, input.cause as u64);
    h = mix(h, input.delta as u64);
    h = mix(h, input.evidence as u64);
    h = mix(h, input.decision as u64);
    h = mix_option_failure(h, input.failure);
    h = mix_option_recovery(h, input.recovery_action);
    h = mix_option_gate(h, input.affected_gate);
    h = mix(h, input.runtime_config.max_steps);
    h = mix(h, input.runtime_config.max_recovery_attempts as u64);
    h = mix(h, state_hash(input.state_before));
    h = mix(h, state_hash(input.state_after));
    h
}

fn state_hash(state: State) -> u64 {
    let mut h = 0x84222325cbf29ce4u64;
    h = mix(h, state.phase as u64);
    h = mix(h, gates_hash(state.gates));
    h = mix(h, packet_hash(state.packet));
    h = mix_option_failure(h, state.failure);
    h = mix_option_recovery(h, state.recovery_action);
    h = mix(h, state.recovery_attempts as u64);
    h
}

fn gates_hash(gates: GateSet) -> u64 {
    let mut h = 0x517cc1b727220a95u64;
    for id in GATE_ORDER {
        let gate = gates.get(id);
        h = mix(h, id as u64);
        h = mix(h, gate.status as u64);
        h = mix(h, gate.evidence as u64);
        h = mix(h, gate.version);
    }
    h
}

fn packet_hash(packet: Packet) -> u64 {
    let mut h = 0x94d049bb133111ebu64;
    h = mix(h, packet.objective_id);
    h = mix(h, packet.objective_required_tasks as u64);
    h = mix(h, packet.objective_done_tasks as u64);
    h = mix(h, packet.ready_tasks as u64);
    h = mix(h, packet.active_task_id);
    h = mix(h, packet.artifact_id);
    h = mix(h, packet.parent_artifact_id);
    h = mix(h, packet.artifact_bytes);
    h = mix(h, packet.artifact_receipt_hash);
    h = mix(h, packet.artifact_lineage_hash);
    h = mix(h, packet.revision);
    h
}


fn mix_option_failure(h: u64, value: Option<FailureClass>) -> u64 {
    match value {
        Some(v) => mix(mix(h, 1), v as u64),
        None => mix(h, 0),
    }
}

fn mix_option_recovery(h: u64, value: Option<RecoveryAction>) -> u64 {
    match value {
        Some(v) => mix(mix(h, 1), v as u64),
        None => mix(h, 0),
    }
}

fn mix_option_gate(h: u64, value: Option<GateId>) -> u64 {
    match value {
        Some(v) => mix(mix(h, 1), v as u64),
        None => mix(h, 0),
    }
}
