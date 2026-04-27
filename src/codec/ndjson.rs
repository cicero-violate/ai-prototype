//! NDJSON TLog codec.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::kernel::{
    Cause, ControlEvent, Decision, EventKind, Evidence, FailureClass, Gate, GateId,
    GateSet, GateStatus, Packet, Phase, RecoveryAction, RuntimeConfig, SemanticDelta,
    State, TLog, GATE_ORDER,
};
use crate::runtime::CanonError;

const TLOG_SCHEMA_VERSION: u64 = 3;
const TLOG_RECORD_EVENT: u64 = 1;

pub fn append_tlog_ndjson(
    path: impl AsRef<Path>,
    event: &ControlEvent,
) -> Result<(), CanonError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|_| CanonError::TlogIo)?;
    writeln!(file, "{}", encode_event_ndjson(event)).map_err(|_| CanonError::TlogIo)?;
    file.sync_all().map_err(|_| CanonError::TlogIo)
}

pub fn write_tlog_ndjson(path: impl AsRef<Path>, tlog: &[ControlEvent]) -> Result<(), CanonError> {
    let mut file = File::create(path).map_err(|_| CanonError::TlogIo)?;
    for event in tlog {
        writeln!(file, "{}", encode_event_ndjson(event)).map_err(|_| CanonError::TlogIo)?;
    }
    file.sync_all().map_err(|_| CanonError::TlogIo)
}

pub fn load_tlog_ndjson(path: impl AsRef<Path>) -> Result<TLog, CanonError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path).map_err(|_| CanonError::TlogIo)?;
    let reader = BufReader::new(file);
    let mut tlog = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|_| CanonError::TlogIo)?;
        if line.trim().is_empty() {
            continue;
        }
        tlog.push(decode_event_ndjson(&line)?);
    }

    Ok(tlog)
}

fn encode_event_ndjson(event: &ControlEvent) -> String {
    let mut fields = Vec::with_capacity(90);
    fields.extend([TLOG_SCHEMA_VERSION, TLOG_RECORD_EVENT]);
    push_event(&mut fields, *event);
    let body = fields
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

fn decode_event_ndjson(line: &str) -> Result<ControlEvent, CanonError> {
    let trimmed = line.trim();
    let body = trimmed
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or(CanonError::InvalidTlogRecord)?;
    let mut fields = Vec::new();
    if !body.trim().is_empty() {
        for raw in body.split(',') {
            fields.push(raw.trim().parse::<u64>().map_err(|_| CanonError::InvalidTlogRecord)?);
        }
    }
    let mut cursor = Cursor { fields: &fields, pos: 0 };
    if cursor.take()? != TLOG_SCHEMA_VERSION || cursor.take()? != TLOG_RECORD_EVENT {
        return Err(CanonError::InvalidTlogRecord);
    }
    let event = pop_event(&mut cursor)?;
    if cursor.pos != fields.len() {
        return Err(CanonError::InvalidTlogRecord);
    }
    Ok(event)
}

struct Cursor<'a> {
    fields: &'a [u64],
    pos: usize,
}

impl Cursor<'_> {
    fn take(&mut self) -> Result<u64, CanonError> {
        let value = *self.fields.get(self.pos).ok_or(CanonError::InvalidTlogRecord)?;
        self.pos += 1;
        Ok(value)
    }
}

fn push_event(out: &mut Vec<u64>, event: ControlEvent) {
    out.extend([
        event.seq,
        event.from as u64,
        event.to as u64,
        event.kind as u64,
        event.cause as u64,
        event.delta as u64,
        event.evidence as u64,
        event.decision as u64,
        opt_failure_to_u64(event.failure),
        opt_recovery_to_u64(event.recovery_action),
        opt_gate_to_u64(event.affected_gate),
        event.runtime_config.max_steps,
        event.runtime_config.max_recovery_attempts as u64,
    ]);
    push_state(out, event.state_before);
    push_state(out, event.state_after);
    out.extend([event.prev_hash, event.self_hash]);
}

fn pop_event(cursor: &mut Cursor<'_>) -> Result<ControlEvent, CanonError> {
    let seq = cursor.take()?;
    let from = phase_from_u64(cursor.take()?)?;
    let to = phase_from_u64(cursor.take()?)?;
    let kind = event_kind_from_u64(cursor.take()?)?;
    let cause = cause_from_u64(cursor.take()?)?;
    let delta = semantic_delta_from_u64(cursor.take()?)?;
    let evidence = evidence_from_u64(cursor.take()?)?;
    let decision = decision_from_u64(cursor.take()?)?;
    let failure = opt_failure_from_u64(cursor.take()?)?;
    let recovery_action = opt_recovery_from_u64(cursor.take()?)?;
    let affected_gate = opt_gate_from_u64(cursor.take()?)?;
    let runtime_config = RuntimeConfig {
        max_steps: cursor.take()?,
        max_recovery_attempts: u8_from_u64(cursor.take()?)?,
    };
    let state_before = pop_state(cursor)?;
    let state_after = pop_state(cursor)?;
    let prev_hash = cursor.take()?;
    let self_hash = cursor.take()?;

    Ok(ControlEvent {
        seq,
        from,
        to,
        kind,
        cause,
        delta,
        evidence,
        decision,
        failure,
        recovery_action,
        affected_gate,
        runtime_config,
        state_before,
        state_after,
        prev_hash,
        self_hash,
    })
}

fn push_state(out: &mut Vec<u64>, state: State) {
    out.push(state.phase as u64);
    push_gates(out, state.gates);
    push_packet(out, state.packet);
    out.push(opt_failure_to_u64(state.failure));
    out.push(opt_recovery_to_u64(state.recovery_action));
    out.push(state.recovery_attempts as u64);
}

fn pop_state(cursor: &mut Cursor<'_>) -> Result<State, CanonError> {
    Ok(State {
        phase: phase_from_u64(cursor.take()?)?,
        gates: pop_gates(cursor)?,
        packet: pop_packet(cursor)?,
        failure: opt_failure_from_u64(cursor.take()?)?,
        recovery_action: opt_recovery_from_u64(cursor.take()?)?,
        recovery_attempts: u8_from_u64(cursor.take()?)?,
    })
}

fn push_gates(out: &mut Vec<u64>, gates: GateSet) {
    for id in GATE_ORDER {
        let gate = gates.get(id);
        out.extend([gate.status as u64, gate.evidence as u64, gate.version]);
    }
}

fn pop_gates(cursor: &mut Cursor<'_>) -> Result<GateSet, CanonError> {
    let mut gates = GateSet::default();
    for id in GATE_ORDER {
        *gates.get_mut(id) = Gate {
            status: gate_status_from_u64(cursor.take()?)?,
            evidence: evidence_from_u64(cursor.take()?)?,
            version: cursor.take()?,
        };
    }
    Ok(gates)
}

fn push_packet(out: &mut Vec<u64>, packet: Packet) {
    out.extend([
        packet.objective_id,
        packet.objective_required_tasks as u64,
        packet.objective_done_tasks as u64,
        packet.ready_tasks as u64,
        packet.active_task_id,
        packet.artifact_id,
        packet.parent_artifact_id,
        packet.artifact_bytes,
        packet.artifact_receipt_hash,
        packet.artifact_lineage_hash,
        packet.revision,
    ]);
}

fn pop_packet(cursor: &mut Cursor<'_>) -> Result<Packet, CanonError> {
    Ok(Packet {
        objective_id: cursor.take()?,
        objective_required_tasks: u8_from_u64(cursor.take()?)?,
        objective_done_tasks: u8_from_u64(cursor.take()?)?,
        ready_tasks: u8_from_u64(cursor.take()?)?,
        active_task_id: cursor.take()?,
        artifact_id: cursor.take()?,
        parent_artifact_id: cursor.take()?,
        artifact_bytes: cursor.take()?,
        artifact_receipt_hash: cursor.take()?,
        artifact_lineage_hash: cursor.take()?,
        revision: cursor.take()?,
    })
}

fn u8_from_u64(value: u64) -> Result<u8, CanonError> {
    u8::try_from(value).map_err(|_| CanonError::InvalidTlogRecord)
}

fn opt_failure_to_u64(value: Option<FailureClass>) -> u64 {
    value.map(|v| v as u64).unwrap_or(0)
}

fn opt_recovery_to_u64(value: Option<RecoveryAction>) -> u64 {
    value.map(|v| v as u64).unwrap_or(0)
}

fn opt_gate_to_u64(value: Option<GateId>) -> u64 {
    value.map(|v| v as u64).unwrap_or(0)
}

fn opt_failure_from_u64(value: u64) -> Result<Option<FailureClass>, CanonError> {
    if value == 0 { Ok(None) } else { failure_from_u64(value).map(Some) }
}

fn opt_recovery_from_u64(value: u64) -> Result<Option<RecoveryAction>, CanonError> {
    if value == 0 { Ok(None) } else { recovery_from_u64(value).map(Some) }
}

fn opt_gate_from_u64(value: u64) -> Result<Option<GateId>, CanonError> {
    if value == 0 { Ok(None) } else { gate_id_from_u64(value).map(Some) }
}

fn phase_from_u64(value: u64) -> Result<Phase, CanonError> {
    match value {
        1 => Ok(Phase::Delta), 2 => Ok(Phase::Invariant), 3 => Ok(Phase::Analysis),
        4 => Ok(Phase::Judgment), 5 => Ok(Phase::Plan), 6 => Ok(Phase::Execute),
        7 => Ok(Phase::Verify), 8 => Ok(Phase::Eval), 9 => Ok(Phase::Recovery),
        10 => Ok(Phase::Learn), 11 => Ok(Phase::Persist), 12 => Ok(Phase::Done),
        _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn gate_status_from_u64(value: u64) -> Result<GateStatus, CanonError> {
    match value { 1 => Ok(GateStatus::Unknown), 2 => Ok(GateStatus::Pass), 3 => Ok(GateStatus::Fail), _ => Err(CanonError::InvalidTlogRecord) }
}

fn gate_id_from_u64(value: u64) -> Result<GateId, CanonError> {
    match value {
        1 => Ok(GateId::Invariant), 2 => Ok(GateId::Analysis), 3 => Ok(GateId::Judgment),
        4 => Ok(GateId::Plan), 5 => Ok(GateId::Execution), 6 => Ok(GateId::Verification),
        7 => Ok(GateId::Eval), 8 => Ok(GateId::Learning), _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn evidence_from_u64(value: u64) -> Result<Evidence, CanonError> {
    match value {
        1 => Ok(Evidence::Missing), 2 => Ok(Evidence::DeltaComputed), 3 => Ok(Evidence::InvariantProof),
        4 => Ok(Evidence::AnalysisReport), 5 => Ok(Evidence::JudgmentRecord), 6 => Ok(Evidence::PlanRecord),
        7 => Ok(Evidence::TaskReady), 8 => Ok(Evidence::ExecutionReceipt), 9 => Ok(Evidence::ArtifactReceipt),
        10 => Ok(Evidence::VerificationReport), 11 => Ok(Evidence::LineageProof), 12 => Ok(Evidence::EvalScore),
        13 => Ok(Evidence::RecoveryPolicy), 14 => Ok(Evidence::CompletionProof), 15 => Ok(Evidence::ConvergenceLimit),
        16 => Ok(Evidence::PersistedRecord), 17 => Ok(Evidence::LearningRecord), 18 => Ok(Evidence::PolicyPromotion),
        _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn failure_from_u64(value: u64) -> Result<FailureClass, CanonError> {
    match value {
        1 => Ok(FailureClass::InvariantUnknown), 2 => Ok(FailureClass::InvariantBlocked),
        3 => Ok(FailureClass::AnalysisMissing), 4 => Ok(FailureClass::AnalysisFailed),
        5 => Ok(FailureClass::JudgmentMissing), 6 => Ok(FailureClass::JudgmentFailed),
        7 => Ok(FailureClass::PlanMissing), 8 => Ok(FailureClass::PlanFailed),
        9 => Ok(FailureClass::PlanReadyQueueEmpty), 10 => Ok(FailureClass::ExecutionMissing),
        11 => Ok(FailureClass::ExecutionFailed), 12 => Ok(FailureClass::TaskReceiptMissing),
        13 => Ok(FailureClass::VerificationUnknown), 14 => Ok(FailureClass::VerificationFailed),
        15 => Ok(FailureClass::ArtifactLineageBroken), 16 => Ok(FailureClass::EvalMissing),
        17 => Ok(FailureClass::EvalFailed), 18 => Ok(FailureClass::RecoveryExhausted),
        19 => Ok(FailureClass::ConvergenceFailed), 20 => Ok(FailureClass::LearningMissing),
        21 => Ok(FailureClass::LearningFailed), _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn recovery_from_u64(value: u64) -> Result<RecoveryAction, CanonError> {
    match value {
        1 => Ok(RecoveryAction::RecheckInvariant), 2 => Ok(RecoveryAction::RunAnalysis),
        3 => Ok(RecoveryAction::Rejudge), 4 => Ok(RecoveryAction::Replan),
        5 => Ok(RecoveryAction::BindReadyTask), 6 => Ok(RecoveryAction::Reexecute),
        7 => Ok(RecoveryAction::Reverify), 8 => Ok(RecoveryAction::RepairArtifactLineage),
        9 => Ok(RecoveryAction::RecomputeEval), 10 => Ok(RecoveryAction::Escalate),
        _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn event_kind_from_u64(value: u64) -> Result<EventKind, CanonError> {
    match value {
        1 => Ok(EventKind::Advanced), 2 => Ok(EventKind::Blocked), 3 => Ok(EventKind::Failed),
        4 => Ok(EventKind::Recovered), 5 => Ok(EventKind::Learned), 6 => Ok(EventKind::Completed),
        7 => Ok(EventKind::Persisted),
        _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn cause_from_u64(value: u64) -> Result<Cause, CanonError> {
    match value {
        1 => Ok(Cause::Start), 2 => Ok(Cause::GatePassed), 3 => Ok(Cause::GateFailed),
        4 => Ok(Cause::EvidenceMissing), 5 => Ok(Cause::JudgmentMade), 6 => Ok(Cause::PlanReady),
        7 => Ok(Cause::ReadyQueueEmpty), 8 => Ok(Cause::ExecutionFinished), 9 => Ok(Cause::TaskReceiptMissing),
        10 => Ok(Cause::VerificationPassed), 11 => Ok(Cause::ArtifactLineageBroken),
        12 => Ok(Cause::EvalPassed), 13 => Ok(Cause::EvalFailed), 14 => Ok(Cause::RepairSelected),
        15 => Ok(Cause::RepairApplied), 16 => Ok(Cause::RecoveryLimit), 17 => Ok(Cause::MaxSteps),
        18 => Ok(Cause::Persisted), 19 => Ok(Cause::PolicyPromoted),
        _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn decision_from_u64(value: u64) -> Result<Decision, CanonError> {
    match value {
        1 => Ok(Decision::Continue), 2 => Ok(Decision::Complete), 3 => Ok(Decision::Block),
        4 => Ok(Decision::Fail), 5 => Ok(Decision::Repair), 6 => Ok(Decision::Halt),
        _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn semantic_delta_from_u64(value: u64) -> Result<SemanticDelta, CanonError> {
    match value {
        1 => Ok(SemanticDelta::NoChange), 2 => Ok(SemanticDelta::PhaseAdvanced),
        3 => Ok(SemanticDelta::FailureRaised), 4 => Ok(SemanticDelta::RepairSelected),
        5 => Ok(SemanticDelta::RepairApplied), 6 => Ok(SemanticDelta::PayloadChanged),
        7 => Ok(SemanticDelta::Completed), 8 => Ok(SemanticDelta::Halted),
        9 => Ok(SemanticDelta::Persisted), 10 => Ok(SemanticDelta::LearningPromoted),
        _ => Err(CanonError::InvalidTlogRecord),
    }
}
