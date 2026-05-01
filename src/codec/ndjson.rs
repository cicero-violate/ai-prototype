//! NDJSON TLog codec.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::error::CanonError;
use crate::kernel::{
    CapabilityRegistryProjection, Cause, ControlEvent, Decision, EventKind, Evidence,
    FailureClass, Gate, GateId, GateSet, GateStatus, Packet, Phase, RecoveryAction,
    RuntimeConfig, SemanticDelta, State, TLog, GATE_ORDER,
};

pub const TLOG_SCHEMA_VERSION: u64 = 5;
pub const TLOG_RECORD_EVENT: u64 = 1;

pub fn append_tlog_ndjson(
    path: impl AsRef<Path>,
    event: &ControlEvent,
) -> Result<(), CanonError> {
    let path = path.as_ref();
    {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|_| CanonError::TlogIo)?;
        writeln!(file, "{}", encode_control_event_ndjson(event)).map_err(|_| CanonError::TlogIo)?;
        file.sync_all().map_err(|_| CanonError::TlogIo)?;
    }
    sync_parent_dir(path)
}

pub fn write_tlog_ndjson(path: impl AsRef<Path>, tlog: &[ControlEvent]) -> Result<(), CanonError> {
    let path = path.as_ref();
    let tmp_path = temporary_tlog_path(path);

    {
        let mut file = File::create(&tmp_path).map_err(|_| CanonError::TlogIo)?;
        for event in tlog {
            writeln!(file, "{}", encode_control_event_ndjson(event)).map_err(|_| CanonError::TlogIo)?;
        }
        file.sync_all().map_err(|_| CanonError::TlogIo)?;
    }

    fs::rename(&tmp_path, path).map_err(|_| CanonError::TlogIo)?;
    sync_parent_dir(path)
}

fn temporary_tlog_path(path: &Path) -> std::path::PathBuf {
    let mut tmp = path.to_path_buf();
    let suffix = match path.extension().and_then(|v| v.to_str()) {
        Some(ext) if !ext.is_empty() => format!("{ext}.tmp"),
        _ => "tmp".to_string(),
    };
    tmp.set_extension(suffix);
    tmp
}

fn sync_parent_dir(path: &Path) -> Result<(), CanonError> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }

    let dir = File::open(parent).map_err(|_| CanonError::TlogIo)?;
    dir.sync_all().map_err(|_| CanonError::TlogIo)
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
        if !is_control_event_record_line(&line)? {
            continue;
        }
        tlog.push(decode_control_event_ndjson(&line)?);
    }

    Ok(tlog)
}

pub fn encode_control_event_ndjson(event: &ControlEvent) -> String {
    let mut fields = Vec::with_capacity(92);
    fields.extend([TLOG_SCHEMA_VERSION, TLOG_RECORD_EVENT]);
    push_event(&mut fields, *event);
    let body = fields
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

pub fn decode_control_event_ndjson(line: &str) -> Result<ControlEvent, CanonError> {
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

pub fn encode_tlog_ndjson_string(tlog: &[ControlEvent]) -> String {
    let mut out = String::new();
    for event in tlog {
        out.push_str(&encode_control_event_ndjson(event));
        out.push('\n');
    }
    out
}

pub fn decode_tlog_ndjson_str(input: &str) -> Result<TLog, CanonError> {
    let mut tlog = Vec::new();
    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if !is_control_event_record_line(line)? {
            continue;
        }
        tlog.push(decode_control_event_ndjson(line)?);
    }
    Ok(tlog)
}

fn is_control_event_record_line(line: &str) -> Result<bool, CanonError> {
    let trimmed = line.trim();
    let body = trimmed
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or(CanonError::InvalidTlogRecord)?;
    let mut parts = body.split(',');
    let Some(version) = parts.next() else {
        return Err(CanonError::InvalidTlogRecord);
    };
    let Some(record) = parts.next() else {
        return Err(CanonError::InvalidTlogRecord);
    };
    let version = version
        .trim()
        .parse::<u64>()
        .map_err(|_| CanonError::InvalidTlogRecord)?;
    let record = record
        .trim()
        .parse::<u64>()
        .map_err(|_| CanonError::InvalidTlogRecord)?;
    Ok(version == TLOG_SCHEMA_VERSION && record == TLOG_RECORD_EVENT)
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
    push_registry_projection(out, event.capability_registry_projection);
    out.extend([
        event.api_command_id,
        event.api_command_hash,
        event.prev_hash,
        event.self_hash,
    ]);
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
    let capability_registry_projection = pop_registry_projection(cursor)?;
    let api_command_id = cursor.take()?;
    let api_command_hash = cursor.take()?;
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
        capability_registry_projection,
        api_command_id,
        api_command_hash,
        prev_hash,
        self_hash,
    })
}

fn push_registry_projection(out: &mut Vec<u64>, projection: CapabilityRegistryProjection) {
    out.extend([projection.route_count, projection.policy_hash]);
}

fn pop_registry_projection(
    cursor: &mut Cursor<'_>,
) -> Result<CapabilityRegistryProjection, CanonError> {
    let projection = CapabilityRegistryProjection::new(cursor.take()?, cursor.take()?);
    if !projection.is_valid() {
        return Err(CanonError::InvalidTlogRecord);
    }
    Ok(projection)
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

fn enum_from_u64<T: Copy>(value: u64, table: &[(u64, T)]) -> Result<T, CanonError> {
    table
        .iter()
        .find_map(|(tag, item)| (*tag == value).then_some(*item))
        .ok_or(CanonError::InvalidTlogRecord)
}

const PHASE_TAGS: &[(u64, Phase)] = &[
    (1, Phase::Delta),
    (2, Phase::Invariant),
    (3, Phase::Analysis),
    (4, Phase::Judgment),
    (5, Phase::Plan),
    (6, Phase::Execute),
    (7, Phase::Verify),
    (8, Phase::Eval),
    (9, Phase::Recovery),
    (10, Phase::Learn),
    (11, Phase::Persist),
    (12, Phase::Done),
];

const GATE_STATUS_TAGS: &[(u64, GateStatus)] = &[
    (1, GateStatus::Unknown),
    (2, GateStatus::Pass),
    (3, GateStatus::Fail),
];

const GATE_ID_TAGS: &[(u64, GateId)] = &[
    (1, GateId::Invariant),
    (2, GateId::Analysis),
    (3, GateId::Judgment),
    (4, GateId::Plan),
    (5, GateId::Execution),
    (6, GateId::Verification),
    (7, GateId::Eval),
    (8, GateId::Learning),
];

const EVIDENCE_TAGS: &[(u64, Evidence)] = &[
    (1, Evidence::Missing),
    (2, Evidence::DeltaComputed),
    (3, Evidence::InvariantProof),
    (4, Evidence::AnalysisReport),
    (5, Evidence::JudgmentRecord),
    (6, Evidence::PlanRecord),
    (7, Evidence::TaskReady),
    (8, Evidence::ExecutionReceipt),
    (9, Evidence::ArtifactReceipt),
    (10, Evidence::VerificationReport),
    (11, Evidence::LineageProof),
    (12, Evidence::EvalScore),
    (13, Evidence::RecoveryPolicy),
    (14, Evidence::CompletionProof),
    (15, Evidence::ConvergenceLimit),
    (16, Evidence::PersistedRecord),
    (17, Evidence::LearningRecord),
    (18, Evidence::PolicyPromotion),
];

const FAILURE_TAGS: &[(u64, FailureClass)] = &[
    (1, FailureClass::InvariantUnknown),
    (2, FailureClass::InvariantBlocked),
    (3, FailureClass::AnalysisMissing),
    (4, FailureClass::AnalysisFailed),
    (5, FailureClass::JudgmentMissing),
    (6, FailureClass::JudgmentFailed),
    (7, FailureClass::PlanMissing),
    (8, FailureClass::PlanFailed),
    (9, FailureClass::PlanReadyQueueEmpty),
    (10, FailureClass::ExecutionMissing),
    (11, FailureClass::ExecutionFailed),
    (12, FailureClass::TaskReceiptMissing),
    (13, FailureClass::VerificationUnknown),
    (14, FailureClass::VerificationFailed),
    (15, FailureClass::ArtifactLineageBroken),
    (16, FailureClass::EvalMissing),
    (17, FailureClass::EvalFailed),
    (18, FailureClass::RecoveryExhausted),
    (19, FailureClass::ConvergenceFailed),
    (20, FailureClass::LearningMissing),
    (21, FailureClass::LearningFailed),
];

const RECOVERY_TAGS: &[(u64, RecoveryAction)] = &[
    (1, RecoveryAction::RecheckInvariant),
    (2, RecoveryAction::RunAnalysis),
    (3, RecoveryAction::Rejudge),
    (4, RecoveryAction::Replan),
    (5, RecoveryAction::BindReadyTask),
    (6, RecoveryAction::Reexecute),
    (7, RecoveryAction::Reverify),
    (8, RecoveryAction::RepairArtifactLineage),
    (9, RecoveryAction::RecomputeEval),
    (10, RecoveryAction::Escalate),
];

const EVENT_KIND_TAGS: &[(u64, EventKind)] = &[
    (1, EventKind::Advanced),
    (2, EventKind::Blocked),
    (3, EventKind::Failed),
    (4, EventKind::Recovered),
    (5, EventKind::Learned),
    (6, EventKind::Completed),
    (7, EventKind::Persisted),
];

const CAUSE_TAGS: &[(u64, Cause)] = &[
    (1, Cause::Start),
    (2, Cause::GatePassed),
    (3, Cause::GateFailed),
    (4, Cause::EvidenceMissing),
    (5, Cause::JudgmentMade),
    (6, Cause::PlanReady),
    (7, Cause::ReadyQueueEmpty),
    (8, Cause::ExecutionFinished),
    (9, Cause::TaskReceiptMissing),
    (10, Cause::VerificationPassed),
    (11, Cause::ArtifactLineageBroken),
    (12, Cause::EvalPassed),
    (13, Cause::EvalFailed),
    (14, Cause::RepairSelected),
    (15, Cause::RepairApplied),
    (16, Cause::RecoveryLimit),
    (17, Cause::MaxSteps),
    (18, Cause::Persisted),
    (19, Cause::PolicyPromoted),
    (20, Cause::EvidenceSubmitted),
];

const DECISION_TAGS: &[(u64, Decision)] = &[
    (1, Decision::Continue),
    (2, Decision::Complete),
    (3, Decision::Block),
    (4, Decision::Fail),
    (5, Decision::Repair),
    (6, Decision::Halt),
];

const SEMANTIC_DELTA_TAGS: &[(u64, SemanticDelta)] = &[
    (1, SemanticDelta::NoChange),
    (2, SemanticDelta::PhaseAdvanced),
    (3, SemanticDelta::FailureRaised),
    (4, SemanticDelta::RepairSelected),
    (5, SemanticDelta::RepairApplied),
    (6, SemanticDelta::PayloadChanged),
    (7, SemanticDelta::Completed),
    (8, SemanticDelta::Halted),
    (9, SemanticDelta::Persisted),
    (10, SemanticDelta::LearningPromoted),
];

fn phase_from_u64(value: u64) -> Result<Phase, CanonError> {
    enum_from_u64(value, PHASE_TAGS)
}

fn gate_status_from_u64(value: u64) -> Result<GateStatus, CanonError> {
    enum_from_u64(value, GATE_STATUS_TAGS)
}

fn gate_id_from_u64(value: u64) -> Result<GateId, CanonError> {
    enum_from_u64(value, GATE_ID_TAGS)
}

fn evidence_from_u64(value: u64) -> Result<Evidence, CanonError> {
    enum_from_u64(value, EVIDENCE_TAGS)
}

fn failure_from_u64(value: u64) -> Result<FailureClass, CanonError> {
    enum_from_u64(value, FAILURE_TAGS)
}

fn recovery_from_u64(value: u64) -> Result<RecoveryAction, CanonError> {
    enum_from_u64(value, RECOVERY_TAGS)
}

fn event_kind_from_u64(value: u64) -> Result<EventKind, CanonError> {
    enum_from_u64(value, EVENT_KIND_TAGS)
}

fn cause_from_u64(value: u64) -> Result<Cause, CanonError> {
    enum_from_u64(value, CAUSE_TAGS)
}

fn decision_from_u64(value: u64) -> Result<Decision, CanonError> {
    enum_from_u64(value, DECISION_TAGS)
}

fn semantic_delta_from_u64(value: u64) -> Result<SemanticDelta, CanonError> {
    enum_from_u64(value, SEMANTIC_DELTA_TAGS)
}
