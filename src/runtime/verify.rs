//! TLog validation, replay, and deterministic event hashing.

use std::{collections::BTreeSet, path::Path};

use crate::codec::ndjson::load_tlog_ndjson;
use crate::kernel::{
    mix, CapabilityRegistryProjection, Cause, ControlEvent, Decision, EventKind, Evidence,
    FailureClass, GateId, GateSet, GateStatus, Packet, Phase, RecoveryAction, RuntimeConfig,
    SemanticDelta, State, GATE_ORDER,
};

use super::transition_table::TRANSITIONS;
use super::{convergence_outcome, reduce, semantic_diff, CanonError, CanonicalWriter, Outcome};

// Expose the full transition table for integration-test coverage contracts.
pub use super::transition_table::Transition as TransitionEntry;
pub use super::transition_table::TRANSITIONS as TRANSITIONS_FOR_TEST;

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
    validate_event_transition(event)?;
    validate_event_failure_class(event)?;
    validate_event_recovery_action(event)?;
    validate_event_affected_gate(event)?;
    validate_persisted_event(event)?;
    validate_learned_event(event)?;
    validate_halt_decision(event)?;

    Ok(())
}

fn validate_event_transition(event: EventView) -> Result<(), CanonError> {
    if !legal_transition(event.from, event.to, event.kind, event.cause) {
        return Err(CanonError::IllegalEvent {
            from: event.from,
            to: event.to,
            kind: event.kind,
        });
    }

    Ok(())
}

fn validate_event_failure_class(event: EventView) -> Result<(), CanonError> {
    if matches!(
        event.kind,
        EventKind::Blocked | EventKind::Failed | EventKind::Recovered
    ) && event.failure.is_none()
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

    Ok(())
}

fn validate_event_recovery_action(event: EventView) -> Result<(), CanonError> {
    if event.kind == EventKind::Recovered && event.recovery_action.is_none() {
        return Err(CanonError::MissingRecoveryAction);
    }

    if matches!(
        event.kind,
        EventKind::Advanced | EventKind::Completed | EventKind::Learned
    ) && event.recovery_action.is_some()
    {
        return Err(CanonError::UnexpectedRecoveryAction);
    }

    Ok(())
}

fn validate_event_affected_gate(event: EventView) -> Result<(), CanonError> {
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

    if matches!(
        event.kind,
        EventKind::Advanced | EventKind::Completed | EventKind::Recovered
    ) && event.affected_gate.is_some()
    {
        return Err(CanonError::UnexpectedAffectedGate);
    }

    Ok(())
}

fn validate_persisted_event(event: EventView) -> Result<(), CanonError> {
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

    if event.kind == EventKind::Persisted
        && event.cause == Cause::Persisted
        && (event.to != Phase::Learn
            || event.failure.is_some()
            || event.recovery_action.is_some()
            || event.affected_gate.is_some())
    {
        return Err(CanonError::InvalidLearnTarget);
    }

    Ok(())
}

fn validate_learned_event(event: EventView) -> Result<(), CanonError> {
    if event.kind == EventKind::Learned
        && (event.to != Phase::Done || event.affected_gate != Some(GateId::Learning))
    {
        return Err(CanonError::InvalidLearnTarget);
    }

    Ok(())
}

fn validate_halt_decision(event: EventView) -> Result<(), CanonError> {
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

    // Observational events: any phase self-loops with Persisted (no gate mutation).
    if from == to
        && kind == EventKind::Persisted
        && matches!(
            cause,
            Cause::AgentCycleEventSubmitted | Cause::WaveDispatched | Cause::ChildTaskCompleted
        )
    {
        return true;
    }

    TRANSITIONS.iter().any(|transition| {
        transition.from == from
            && transition.to == to
            && transition.kind == kind
            && transition.cause == cause
    })
}

fn evidence_submission_outcome(before: State, event: &ControlEvent) -> Result<Outcome, CanonError> {
    if event.from != event.to || event.failure.is_some() || event.recovery_action.is_some() {
        return Err(CanonError::InvalidReplay);
    }

    let gate_id = event.affected_gate.ok_or(CanonError::MissingAffectedGate)?;
    let gate_after = event.state_after.gates.get(gate_id);
    let passed = match gate_after.status {
        GateStatus::Pass => true,
        GateStatus::Fail => false,
        GateStatus::Unknown => return Err(CanonError::InvalidReplay),
    };

    if gate_after.evidence != event.evidence {
        return Err(CanonError::InvalidReplay);
    }

    let mut expected = before;
    if passed {
        apply_expected_packet_effect(&mut expected, gate_id, event.evidence);
    }
    expected.apply_evidence(gate_id, event.evidence, passed);

    if expected != event.state_after {
        return Err(CanonError::InvalidReplay);
    }

    Ok(Outcome {
        state: expected,
        kind: EventKind::Persisted,
        cause: Cause::EvidenceSubmitted,
        evidence: event.evidence,
        decision: if passed {
            Decision::Continue
        } else {
            Decision::Block
        },
        failure: None,
        recovery_action: None,
        affected_gate: Some(gate_id),
    })
}

fn apply_expected_packet_effect(state: &mut State, gate_id: GateId, evidence: Evidence) {
    match (gate_id, evidence) {
        (GateId::Plan, Evidence::TaskReady | Evidence::PlanRecord) => {
            state.packet.bind_ready_task();
        }
        (GateId::Execution, Evidence::ArtifactReceipt) => {
            state.packet.materialize_artifact();
        }
        (GateId::Execution, Evidence::ExecutionReceipt) => {}
        (GateId::Verification, Evidence::LineageProof | Evidence::VerificationReport) => {
            state.packet.repair_lineage();
        }
        (GateId::Eval, Evidence::EvalScore) => {
            state.packet.complete_objective();
        }
        _ => {}
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplayReport {
    pub initial_state: State,
    pub final_state: State,
    pub event_count: usize,
    pub first_seq: Option<u64>,
    pub last_seq: Option<u64>,
    pub final_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandCausalityReport {
    pub event_count: usize,
    pub command_event_count: usize,
    pub authorization_event_count: usize,
    pub receipt_event_count: usize,
    pub first_authorization_seq: Option<u64>,
    pub last_receipt_seq: Option<u64>,
    pub final_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TemporalEventType {
    kind: EventKind,
    cause: Cause,
}

impl TemporalEventType {
    const fn new(kind: EventKind, cause: Cause) -> Self {
        Self { kind, cause }
    }

    const fn from_event(event: &ControlEvent) -> Self {
        Self {
            kind: event.kind,
            cause: event.cause,
        }
    }
}

const PERSISTED_EVIDENCE_SUBMITTED: TemporalEventType =
    TemporalEventType::new(EventKind::Persisted, Cause::EvidenceSubmitted);

const EVIDENCE_SUBMITTED_TEMPORAL_SUCCESSORS: [TemporalEventType; 4] = [
    TemporalEventType::new(EventKind::Recovered, Cause::RepairSelected),
    TemporalEventType::new(EventKind::Persisted, Cause::RepairApplied),
    TemporalEventType::new(EventKind::Failed, Cause::GateFailed),
    TemporalEventType::new(EventKind::Failed, Cause::EvidenceMissing),
];

#[derive(Default)]
struct EvidenceSubmittedTemporalOrder {
    command_ids_with_evidence: BTreeSet<u64>,
}

impl EvidenceSubmittedTemporalOrder {
    fn validate_and_record(&mut self, event: &ControlEvent) -> Result<(), CanonError> {
        if event.api_command_id == 0 {
            return Ok(());
        }

        let event_type = TemporalEventType::from_event(event);
        if EVIDENCE_SUBMITTED_TEMPORAL_SUCCESSORS.contains(&event_type)
            && !self
                .command_ids_with_evidence
                .contains(&event.api_command_id)
        {
            return Err(CanonError::InvalidReplay);
        }

        if event_type == PERSISTED_EVIDENCE_SUBMITTED {
            self.command_ids_with_evidence.insert(event.api_command_id);
        }

        Ok(())
    }
}

pub fn replay_report_ndjson(
    initial: State,
    path: impl AsRef<Path>,
) -> Result<ReplayReport, CanonError> {
    let tlog = load_tlog_ndjson(path)?;
    replay_report_from(initial, &tlog)
}

pub fn replay_report_from(
    initial: State,
    tlog: &[ControlEvent],
) -> Result<ReplayReport, CanonError> {
    let final_state = verify_tlog_from(initial, tlog)?;
    Ok(ReplayReport {
        initial_state: initial,
        final_state,
        event_count: tlog.len(),
        first_seq: tlog.first().map(|event| event.seq),
        last_seq: tlog.last().map(|event| event.seq),
        final_hash: tlog.last().map(|event| event.self_hash).unwrap_or(0),
    })
}

pub fn command_causality_report_from(
    initial: State,
    tlog: &[ControlEvent],
) -> Result<CommandCausalityReport, CanonError> {
    verify_tlog_from(initial, tlog)?;

    let mut command_event_count = 0usize;
    let mut authorization_event_count = 0usize;
    let mut receipt_event_count = 0usize;
    let mut first_authorization_seq = None;
    let mut last_receipt_seq = None;

    for event in tlog {
        if event.api_command_id == 0 && event.api_command_hash == 0 {
            continue;
        }

        command_event_count += 1;

        if is_authorization_projection_event(event) {
            authorization_event_count += 1;
            first_authorization_seq.get_or_insert(event.seq);
        }

        if is_receipt_projection_event(event) {
            receipt_event_count += 1;
            last_receipt_seq = Some(event.seq);
        }
    }

    Ok(CommandCausalityReport {
        event_count: tlog.len(),
        command_event_count,
        authorization_event_count,
        receipt_event_count,
        first_authorization_seq,
        last_receipt_seq,
        final_hash: tlog.last().map(|event| event.self_hash).unwrap_or(0),
    })
}

fn is_authorization_projection_event(event: &ControlEvent) -> bool {
    event.cause == Cause::EvidenceSubmitted
        && event.affected_gate == Some(GateId::Plan)
        && event.evidence == Evidence::TaskReady
        && event.api_command_id != 0
        && event.api_command_hash != 0
}

fn is_receipt_projection_event(event: &ControlEvent) -> bool {
    event.cause == Cause::EvidenceSubmitted
        && event.affected_gate == Some(GateId::Execution)
        && event.evidence == Evidence::ExecutionReceipt
        && event.api_command_id != 0
        && event.api_command_hash != 0
}

pub fn replay_tlog_ndjson(initial: State, path: impl AsRef<Path>) -> Result<State, CanonError> {
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
    validate_replay_initial_state(initial)?;

    let mut state = initial;
    let mut prev_hash = 0;
    let mut evidence_submitted_order = EvidenceSubmittedTemporalOrder::default();

    for (i, event) in tlog.iter().enumerate() {
        validate_replay_event_shape(event)?;
        validate_replay_hash_link(event, i, prev_hash)?;
        validate_replay_api_command(event)?;
        validate_replay_registry_projection(event)?;
        validate_replay_state_continuity(event, state)?;
        validate_replay_semantic_delta(event)?;
        validate_event(event_view(event))?;
        evidence_submitted_order.validate_and_record(event)?;
        validate_replay_self_hash(event)?;
        validate_replay_writer_identity(tlog, i, state, event)?;

        state = event.state_after;
        prev_hash = event.self_hash;
    }

    Ok(state)
}

fn validate_replay_initial_state(initial: State) -> Result<(), CanonError> {
    if !initial.is_structurally_valid() {
        return Err(CanonError::InvalidStateInvariant);
    }

    Ok(())
}

fn validate_replay_event_shape(event: &ControlEvent) -> Result<(), CanonError> {
    if !event.runtime_config.is_structurally_valid() {
        return Err(CanonError::InvalidRuntimeConfig);
    }

    if !event.state_before.is_structurally_valid() || !event.state_after.is_structurally_valid() {
        return Err(CanonError::InvalidStateInvariant);
    }

    Ok(())
}

fn validate_replay_hash_link(
    event: &ControlEvent,
    event_index: usize,
    prev_hash: u64,
) -> Result<(), CanonError> {
    if event.seq != event_index as u64 + 1 || event.prev_hash != prev_hash {
        return Err(CanonError::InvalidHashChain);
    }

    Ok(())
}

fn validate_replay_api_command(event: &ControlEvent) -> Result<(), CanonError> {
    if (event.api_command_id == 0) != (event.api_command_hash == 0) {
        return Err(CanonError::InvalidApiCommand);
    }

    Ok(())
}

fn validate_replay_registry_projection(event: &ControlEvent) -> Result<(), CanonError> {
    let projection = event.capability_registry_projection;
    let is_observational = matches!(
        event.cause,
        Cause::AgentCycleEventSubmitted | Cause::WaveDispatched | Cause::ChildTaskCompleted
    );

    if !projection.is_valid()
        || (event.cause == Cause::EvidenceSubmitted && projection.is_empty())
        || (event.cause != Cause::EvidenceSubmitted && !is_observational && !projection.is_empty())
        || (is_observational && !projection.is_empty())
    {
        return Err(CanonError::InvalidReplay);
    }

    Ok(())
}

fn validate_replay_state_continuity(event: &ControlEvent, state: State) -> Result<(), CanonError> {
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

    Ok(())
}

fn validate_replay_semantic_delta(event: &ControlEvent) -> Result<(), CanonError> {
    if event.delta != semantic_diff(event.state_before, event.state_after) {
        return Err(CanonError::InvalidSemanticDelta);
    }

    Ok(())
}

fn event_view(event: &ControlEvent) -> EventView {
    EventView {
        from: event.from,
        to: event.to,
        kind: event.kind,
        cause: event.cause,
        decision: event.decision,
        failure: event.failure,
        recovery_action: event.recovery_action,
        affected_gate: event.affected_gate,
    }
}

fn validate_replay_self_hash(event: &ControlEvent) -> Result<(), CanonError> {
    if expected_event_hash(event) != event.self_hash {
        return Err(CanonError::InvalidHashChain);
    }

    Ok(())
}

fn expected_event_hash(event: &ControlEvent) -> u64 {
    hash_event(EventHashInput {
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
        capability_registry_projection: event.capability_registry_projection,
        api_command_id: event.api_command_id,
        api_command_hash: event.api_command_hash,
    })
}

fn validate_replay_writer_identity(
    tlog: &[ControlEvent],
    event_index: usize,
    state: State,
    event: &ControlEvent,
) -> Result<(), CanonError> {
    let expected_event = CanonicalWriter::build_with_command_and_registry_projection(
        &tlog[..event_index],
        state,
        expected_replay_outcome(state, event)?,
        event.runtime_config,
        event.api_command_id,
        event.api_command_hash,
        event.capability_registry_projection,
    )?;

    if *event != expected_event {
        return Err(CanonError::InvalidReplay);
    }

    Ok(())
}

fn expected_replay_outcome(state: State, event: &ControlEvent) -> Result<Outcome, CanonError> {
    if event.cause == Cause::MaxSteps {
        Ok(convergence_outcome(state))
    } else if event.cause == Cause::EvidenceSubmitted {
        evidence_submission_outcome(state, event)
    } else if matches!(
        event.cause,
        Cause::AgentCycleEventSubmitted | Cause::WaveDispatched | Cause::ChildTaskCompleted
    ) {
        observational_outcome(state, event)
    } else {
        Ok(reduce(state, event.runtime_config))
    }
}

fn observational_outcome(state: State, event: &ControlEvent) -> Result<Outcome, CanonError> {
    if event.from != event.to {
        return Err(CanonError::InvalidReplay);
    }

    // Wave events mutate wave_pending; verify the expected delta and that all
    // other state fields are unchanged.
    let state_after = match event.cause {
        Cause::WaveDispatched => {
            let mut expected = state;
            expected.wave_pending = event.state_after.wave_pending;
            if expected != event.state_after {
                return Err(CanonError::InvalidReplay);
            }
            event.state_after
        }
        Cause::ChildTaskCompleted => {
            let mut expected = state;
            expected.wave_pending = state
                .wave_pending
                .checked_sub(1)
                .ok_or(CanonError::InvalidReplay)?;
            if expected != event.state_after {
                return Err(CanonError::InvalidReplay);
            }
            event.state_after
        }
        Cause::Start
        | Cause::GatePassed
        | Cause::GateFailed
        | Cause::EvidenceMissing
        | Cause::JudgmentMade
        | Cause::PlanReady
        | Cause::ReadyQueueEmpty
        | Cause::ExecutionFinished
        | Cause::TaskReceiptMissing
        | Cause::VerificationPassed
        | Cause::ArtifactLineageBroken
        | Cause::EvalPassed
        | Cause::EvalFailed
        | Cause::RepairSelected
        | Cause::RepairApplied
        | Cause::RecoveryLimit
        | Cause::MaxSteps
        | Cause::Persisted
        | Cause::PolicyPromoted
        | Cause::EvidenceSubmitted
        | Cause::AgentCycleEventSubmitted
        | Cause::SymbolMutationObserved
        | Cause::ArchitecturalDecisionMade
        | Cause::CostGateEvaluated
        | Cause::SystemRestart => {
            // Pure-observational causes: state must be completely unchanged.
            if event.state_after != state {
                return Err(CanonError::InvalidReplay);
            }
            state
        }
    };

    Ok(Outcome {
        state: state_after,
        kind: EventKind::Persisted,
        cause: event.cause,
        evidence: event.evidence,
        decision: Decision::Continue,
        failure: None,
        recovery_action: None,
        affected_gate: None,
    })
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
    pub(crate) capability_registry_projection: CapabilityRegistryProjection,
    pub(crate) api_command_id: u64,
    pub(crate) api_command_hash: u64,
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
    h = mix(h, input.capability_registry_projection.route_count);
    h = mix(h, input.capability_registry_projection.policy_hash);
    h = mix(h, input.api_command_id);
    h = mix(h, input.api_command_hash);
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
    h = mix(h, state.wave_pending as u64);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::{CapabilityRegistryProjection, FailureClass};

    const COMMAND_ID: u64 = 177;
    const COMMAND_HASH: u64 = 0xA11CE;

    #[test]
    fn evidence_submission_precedes_repair_selected_per_command_trace() {
        assert_target_requires_prior_evidence(
            repair_selected_state(),
            EventKind::Recovered,
            Cause::RepairSelected,
        );
    }

    #[test]
    fn evidence_submission_precedes_repair_applied_per_command_trace() {
        assert_target_requires_prior_evidence(
            repair_applied_state(),
            EventKind::Persisted,
            Cause::RepairApplied,
        );
    }

    #[test]
    fn evidence_submission_precedes_failed_gate_failed_per_command_trace() {
        assert_target_requires_prior_evidence(
            failed_gate_failed_state(),
            EventKind::Failed,
            Cause::GateFailed,
        );
    }

    #[test]
    fn evidence_submission_precedes_failed_evidence_missing_per_command_trace() {
        assert_target_requires_prior_evidence(
            failed_evidence_missing_state(),
            EventKind::Failed,
            Cause::EvidenceMissing,
        );
    }

    #[test]
    #[test]
    fn prior_evidence_from_another_command_does_not_satisfy_temporal_order() {
        let cfg = RuntimeConfig::default();
        let cases = [
            (
                repair_selected_state(),
                EventKind::Recovered,
                Cause::RepairSelected,
            ),
            (
                repair_applied_state(),
                EventKind::Persisted,
                Cause::RepairApplied,
            ),
            (
                failed_gate_failed_state(),
                EventKind::Failed,
                Cause::GateFailed,
            ),
            (
                failed_evidence_missing_state(),
                EventKind::Failed,
                Cause::EvidenceMissing,
            ),
        ];

        for (initial, expected_kind, expected_cause) in cases {
            let unrelated_command_evidence = evidence_submission_event(&[], initial, cfg, 501, 0x501);
            let target = reducer_event(
                &[unrelated_command_evidence],
                unrelated_command_evidence.state_after,
                cfg,
                502,
                0x502,
            );
            assert_event_type(&target, expected_kind, expected_cause);

            assert_eq!(
                verify_tlog_from(initial, &[unrelated_command_evidence, target]),
                Err(CanonError::InvalidReplay),
                "unrelated command evidence unexpectedly satisfied {:?}::{:?}",
                expected_kind,
                expected_cause
            );
        }
    }