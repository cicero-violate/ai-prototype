//! NDJSON TLog codec.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Controls when `fsync` is called during append operations.
///
/// `SyncEveryBatch` (default) calls `sync_all` once after writing all events in a
/// call, matching the current behavior of `append_tlog_events_ndjson`. Use
/// `SyncEveryAppend` only when each individual record must survive a crash
/// independently; it incurs one additional `sync_all` per event.
///
/// The tradeoff: `SyncEveryBatch` reduces fsync overhead at the cost of losing the
/// last partial batch on a crash between events in the same call. For canonical TLog
/// append, each `/v1/command` call is one batch, so `SyncEveryBatch` is safe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppendPolicy {
    SyncEveryBatch,
    SyncEveryAppend,
}

use crate::capability::planning::{
    AcceptedPlanPatchRecord, PlanAssigneeChangePatch, PlanEdgePatch, PlanEvidenceAppendPatch,
    PlanFullImportPatch, PlanNodeRemovePatch, PlanNodeStatus, PlanNodeUpsertPatch, PlanPatchKind,
    PlanPatchPayload, PlanPatchRecord, PlanStatusChangePatch, RejectedPlanPatchRecord,
    PLAN_PATCH_ACCEPTED_RECORD, PLAN_PATCH_RECORD, PLAN_PATCH_REJECTED_RECORD,
    PLAN_PATCH_SCHEMA_VERSION,
};
use crate::kernel::CanonError;
use crate::kernel::{
    CapabilityRegistryProjection, Cause, ControlEvent, Decision, EventKind, Evidence, FailureClass,
    Gate, GateId, GateSet, GateStatus, Packet, Phase, RecoveryAction, RuntimeConfig, SemanticDelta,
    State, TLog, GATE_ORDER,
};

pub const TLOG_SCHEMA_VERSION: u64 = 6;
pub const TLOG_RECORD_EVENT: u64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanPatchTlogRecord {
    Patch(PlanPatchRecord),
    Accepted(AcceptedPlanPatchRecord),
    Rejected(RejectedPlanPatchRecord),
}

pub fn append_tlog_ndjson(path: impl AsRef<Path>, event: &ControlEvent) -> Result<(), CanonError> {
    append_tlog_events_ndjson(path, std::slice::from_ref(event))
}

pub fn append_tlog_events_ndjson(
    path: impl AsRef<Path>,
    events: &[ControlEvent],
) -> Result<(), CanonError> {
    append_tlog_events_ndjson_with_policy(path, events, AppendPolicy::SyncEveryBatch)
}

/// Append `events` to the NDJSON TLog at `path` using the given sync policy.
///
/// `SyncEveryBatch` issues one `sync_all` after writing all events (the default).
/// `SyncEveryAppend` issues one `sync_all` per event — use only when each record
/// must survive a crash independently.
pub fn append_tlog_events_ndjson_with_policy(
    path: impl AsRef<Path>,
    events: &[ControlEvent],
    policy: AppendPolicy,
) -> Result<(), CanonError> {
    if events.is_empty() {
        return Ok(());
    }

    let path = path.as_ref();
    {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|_| CanonError::TlogIo)?;
        for event in events {
            writeln!(file, "{}", encode_control_event_ndjson(event))
                .map_err(|_| CanonError::TlogIo)?;
            if policy == AppendPolicy::SyncEveryAppend {
                file.sync_all().map_err(|_| CanonError::TlogIo)?;
            }
        }
        if policy == AppendPolicy::SyncEveryBatch {
            file.sync_all().map_err(|_| CanonError::TlogIo)?;
        }
    }
    sync_parent_dir(path)
}

pub fn write_tlog_ndjson(path: impl AsRef<Path>, tlog: &[ControlEvent]) -> Result<(), CanonError> {
    let path = path.as_ref();
    let tmp_path = temporary_tlog_path(path);

    {
        let mut file = File::create(&tmp_path).map_err(|_| CanonError::TlogIo)?;
        for event in tlog {
            writeln!(file, "{}", encode_control_event_ndjson(event))
                .map_err(|_| CanonError::TlogIo)?;
        }
        file.sync_all().map_err(|_| CanonError::TlogIo)?;
    }

    fs::rename(&tmp_path, path).map_err(|_| CanonError::TlogIo)?;
    sync_parent_dir(path)
}

pub fn append_plan_patch_record_ndjson(
    path: impl AsRef<Path>,
    record: PlanPatchTlogRecord,
) -> Result<(), CanonError> {
    append_canonical_record_line(path, &encode_plan_patch_record_ndjson(record))
}

fn append_canonical_record_line(path: impl AsRef<Path>, line: &str) -> Result<(), CanonError> {
    let path = path.as_ref();
    {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|_| CanonError::TlogIo)?;
        writeln!(file, "{line}").map_err(|_| CanonError::TlogIo)?;
        file.sync_all().map_err(|_| CanonError::TlogIo)?;
    }
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

pub fn load_plan_patch_records_ndjson(
    path: impl AsRef<Path>,
) -> Result<Vec<PlanPatchTlogRecord>, CanonError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path).map_err(|_| CanonError::TlogIo)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|_| CanonError::TlogIo)?;
        if line.trim().is_empty() {
            continue;
        }
        if !is_plan_patch_record_line(&line)? {
            continue;
        }
        records.push(decode_plan_patch_record_ndjson(&line)?);
    }

    Ok(records)
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

pub fn encode_plan_patch_record_ndjson(record: PlanPatchTlogRecord) -> String {
    let mut fields = Vec::with_capacity(20);
    push_plan_patch_record(&mut fields, record);
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
            fields.push(
                raw.trim()
                    .parse::<u64>()
                    .map_err(|_| CanonError::InvalidTlogRecord)?,
            );
        }
    }
    let mut cursor = Cursor {
        fields: &fields,
        pos: 0,
    };
    if cursor.take()? != TLOG_SCHEMA_VERSION || cursor.take()? != TLOG_RECORD_EVENT {
        return Err(CanonError::InvalidTlogRecord);
    }
    let event = pop_event(&mut cursor)?;
    if cursor.pos != fields.len() {
        return Err(CanonError::InvalidTlogRecord);
    }
    Ok(event)
}

pub fn decode_plan_patch_record_ndjson(line: &str) -> Result<PlanPatchTlogRecord, CanonError> {
    let mut cursor = cursor_from_ndjson_line(line)?;
    let schema_version = cursor.take()?;
    if schema_version != PLAN_PATCH_SCHEMA_VERSION {
        return Err(CanonError::InvalidTlogRecord);
    }
    let record_type = cursor.take()?;
    let record = match record_type {
        PLAN_PATCH_RECORD => PlanPatchTlogRecord::Patch(pop_plan_patch_record(&mut cursor)?),
        PLAN_PATCH_ACCEPTED_RECORD => {
            PlanPatchTlogRecord::Accepted(pop_accepted_plan_patch_record(&mut cursor)?)
        }
        PLAN_PATCH_REJECTED_RECORD => {
            PlanPatchTlogRecord::Rejected(pop_rejected_plan_patch_record(&mut cursor)?)
        }
        _ => return Err(CanonError::InvalidTlogRecord),
    };
    if cursor.pos != cursor.fields.len() || !plan_patch_tlog_record_is_consistent(record) {
        return Err(CanonError::InvalidTlogRecord);
    }
    Ok(record)
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
    let Some((version, record)) = record_header(line)? else {
        return Ok(false);
    };
    Ok(version == TLOG_SCHEMA_VERSION && record == TLOG_RECORD_EVENT)
}

fn is_plan_patch_record_line(line: &str) -> Result<bool, CanonError> {
    let Some((version, record)) = record_header(line)? else {
        return Ok(false);
    };
    Ok(version == PLAN_PATCH_SCHEMA_VERSION
        && matches!(
            record,
            PLAN_PATCH_RECORD | PLAN_PATCH_ACCEPTED_RECORD | PLAN_PATCH_REJECTED_RECORD
        ))
}

fn record_header(line: &str) -> Result<Option<(u64, u64)>, CanonError> {
    let trimmed = line.trim();
    if !trimmed.starts_with('[') {
        return Ok(None);
    }
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
    Ok(Some((version, record)))
}

fn cursor_from_ndjson_line(line: &str) -> Result<Cursor<'static>, CanonError> {
    let trimmed = line.trim();
    let body = trimmed
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or(CanonError::InvalidTlogRecord)?;
    let mut fields = Vec::new();
    if !body.trim().is_empty() {
        for raw in body.split(',') {
            fields.push(
                raw.trim()
                    .parse::<u64>()
                    .map_err(|_| CanonError::InvalidTlogRecord)?,
            );
        }
    }
    Ok(Cursor {
        fields: Box::leak(fields.into_boxed_slice()),
        pos: 0,
    })
}

struct Cursor<'a> {
    fields: &'a [u64],
    pos: usize,
}

impl Cursor<'_> {
    fn take(&mut self) -> Result<u64, CanonError> {
        let value = *self
            .fields
            .get(self.pos)
            .ok_or(CanonError::InvalidTlogRecord)?;
        self.pos += 1;
        Ok(value)
    }
}

/// Encode a ControlEvent into the canonical u64 field array (schema header included).
/// Used by binary_tlog to write the same record format without JSON text overhead.
#[cfg_attr(not(feature = "binary-tlog"), allow(dead_code))]
pub(crate) fn control_event_to_fields(event: &ControlEvent) -> Vec<u64> {
    let mut fields = Vec::with_capacity(92);
    fields.extend([TLOG_SCHEMA_VERSION, TLOG_RECORD_EVENT]);
    push_event(&mut fields, *event);
    fields
}

/// Decode a ControlEvent from a u64 field slice (schema header included).
#[cfg_attr(not(feature = "binary-tlog"), allow(dead_code))]
pub(crate) fn control_event_from_fields(fields: &[u64]) -> Result<ControlEvent, CanonError> {
    let mut cursor = Cursor { fields, pos: 0 };
    if cursor.take()? != TLOG_SCHEMA_VERSION || cursor.take()? != TLOG_RECORD_EVENT {
        return Err(CanonError::InvalidTlogRecord);
    }
    let event = pop_event(&mut cursor)?;
    if cursor.pos != fields.len() {
        return Err(CanonError::InvalidTlogRecord);
    }
    Ok(event)
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
    let phases = pop_event_phases(cursor)?;
    let semantics = pop_event_semantics(cursor)?;
    let runtime_config = pop_runtime_config(cursor)?;
    let state_before = pop_state(cursor)?;
    let state_after = pop_state(cursor)?;
    let capability_registry_projection = pop_registry_projection(cursor)?;
    let command_hashes = pop_event_command_hashes(cursor)?;

    Ok(ControlEvent {
        seq,
        from: phases.from,
        to: phases.to,
        kind: semantics.kind,
        cause: semantics.cause,
        delta: semantics.delta,
        evidence: semantics.evidence,
        decision: semantics.decision,
        failure: semantics.failure,
        recovery_action: semantics.recovery_action,
        affected_gate: semantics.affected_gate,
        runtime_config,
        state_before,
        state_after,
        capability_registry_projection,
        api_command_id: command_hashes.api_command_id,
        api_command_hash: command_hashes.api_command_hash,
        prev_hash: command_hashes.prev_hash,
        self_hash: command_hashes.self_hash,
    })
}

struct EventPhases {
    from: Phase,
    to: Phase,
}

fn pop_event_phases(cursor: &mut Cursor<'_>) -> Result<EventPhases, CanonError> {
    Ok(EventPhases {
        from: phase_from_u64(cursor.take()?)?,
        to: phase_from_u64(cursor.take()?)?,
    })
}

struct EventSemantics {
    kind: EventKind,
    cause: Cause,
    delta: SemanticDelta,
    evidence: Evidence,
    decision: Decision,
    failure: Option<FailureClass>,
    recovery_action: Option<RecoveryAction>,
    affected_gate: Option<GateId>,
}

fn pop_event_semantics(cursor: &mut Cursor<'_>) -> Result<EventSemantics, CanonError> {
    Ok(EventSemantics {
        kind: event_kind_from_u64(cursor.take()?)?,
        cause: cause_from_u64(cursor.take()?)?,
        delta: semantic_delta_from_u64(cursor.take()?)?,
        evidence: evidence_from_u64(cursor.take()?)?,
        decision: decision_from_u64(cursor.take()?)?,
        failure: opt_failure_from_u64(cursor.take()?)?,
        recovery_action: opt_recovery_from_u64(cursor.take()?)?,
        affected_gate: opt_gate_from_u64(cursor.take()?)?,
    })
}

fn pop_runtime_config(cursor: &mut Cursor<'_>) -> Result<RuntimeConfig, CanonError> {
    Ok(RuntimeConfig {
        max_steps: cursor.take()?,
        max_recovery_attempts: u8_from_u64(cursor.take()?)?,
    })
}

struct EventCommandHashes {
    api_command_id: u64,
    api_command_hash: u64,
    prev_hash: u64,
    self_hash: u64,
}

fn pop_event_command_hashes(cursor: &mut Cursor<'_>) -> Result<EventCommandHashes, CanonError> {
    Ok(EventCommandHashes {
        api_command_id: cursor.take()?,
        api_command_hash: cursor.take()?,
        prev_hash: cursor.take()?,
        self_hash: cursor.take()?,
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
    out.push(state.wave_pending as u64);
    out.push(state.plan_state_hash);
}

fn pop_state(cursor: &mut Cursor<'_>) -> Result<State, CanonError> {
    Ok(State {
        phase: phase_from_u64(cursor.take()?)?,
        gates: pop_gates(cursor)?,
        packet: pop_packet(cursor)?,
        failure: opt_failure_from_u64(cursor.take()?)?,
        recovery_action: opt_recovery_from_u64(cursor.take()?)?,
        recovery_attempts: u8_from_u64(cursor.take()?)?,
        wave_pending: u16_from_u64(cursor.take()?)?,
        plan_state_hash: cursor.take()?,
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

fn push_plan_patch_record(out: &mut Vec<u64>, record: PlanPatchTlogRecord) {
    match record {
        PlanPatchTlogRecord::Patch(record) => {
            out.extend([
                record.schema_version,
                record.record_type,
                record.source_hash,
                record.cycle_id,
                record.patch_seq,
                record.contract_hash,
                record.payload.kind() as u64,
            ]);
            push_plan_patch_payload(out, record.payload);
            out.extend([record.payload_hash, record.patch_hash]);
        }
        PlanPatchTlogRecord::Accepted(record) => out.extend([
            record.schema_version,
            record.record_type,
            record.source_hash,
            record.cycle_id,
            record.patch_seq,
            record.contract_hash,
            record.patch_hash,
            record.applied_revision,
            record.acceptance_hash,
        ]),
        PlanPatchTlogRecord::Rejected(record) => out.extend([
            record.schema_version,
            record.record_type,
            record.source_hash,
            record.cycle_id,
            record.patch_seq,
            record.contract_hash,
            record.patch_hash,
            record.reason_hash,
            record.rejection_hash,
        ]),
    }
}

fn push_plan_patch_payload(out: &mut Vec<u64>, payload: PlanPatchPayload) {
    match payload {
        PlanPatchPayload::NodeUpsert(payload) => out.extend([
            payload.node_id_hash,
            payload.title_hash,
            payload.description_hash,
            payload.status as u64,
            payload.assignee_hash,
            payload.score_axes_hash,
            payload.files_hash,
        ]),
        PlanPatchPayload::EdgeAdd(payload) | PlanPatchPayload::EdgeRemove(payload) => {
            out.extend([payload.from_node_hash, payload.to_node_hash]);
        }
        PlanPatchPayload::NodeRemove(payload) => out.push(payload.node_id_hash),
        PlanPatchPayload::StatusChange(payload) => {
            out.extend([payload.node_id_hash, payload.status as u64]);
        }
        PlanPatchPayload::AssigneeChange(payload) => {
            out.extend([payload.node_id_hash, payload.assignee_hash]);
        }
        PlanPatchPayload::EvidenceAppend(payload) => out.extend([
            payload.node_id_hash,
            payload.path_hash,
            payload.kind_hash,
            payload.summary_hash,
        ]),
        PlanPatchPayload::FullImport(payload) => out.extend([
            payload.node_count,
            payload.edge_count,
            payload.nodes_hash,
            payload.edges_hash,
        ]),
    }
}

fn pop_plan_patch_record(cursor: &mut Cursor<'_>) -> Result<PlanPatchRecord, CanonError> {
    let source_hash = cursor.take()?;
    let cycle_id = cursor.take()?;
    let patch_seq = cursor.take()?;
    let contract_hash = cursor.take()?;
    let kind = plan_patch_kind_from_u64(cursor.take()?)?;
    let payload = pop_plan_patch_payload(cursor, kind)?;
    Ok(PlanPatchRecord {
        schema_version: PLAN_PATCH_SCHEMA_VERSION,
        record_type: PLAN_PATCH_RECORD,
        source_hash,
        cycle_id,
        patch_seq,
        contract_hash,
        payload,
        payload_hash: cursor.take()?,
        patch_hash: cursor.take()?,
    })
}

fn pop_accepted_plan_patch_record(
    cursor: &mut Cursor<'_>,
) -> Result<AcceptedPlanPatchRecord, CanonError> {
    Ok(AcceptedPlanPatchRecord {
        schema_version: PLAN_PATCH_SCHEMA_VERSION,
        record_type: PLAN_PATCH_ACCEPTED_RECORD,
        source_hash: cursor.take()?,
        cycle_id: cursor.take()?,
        patch_seq: cursor.take()?,
        contract_hash: cursor.take()?,
        patch_hash: cursor.take()?,
        applied_revision: cursor.take()?,
        acceptance_hash: cursor.take()?,
    })
}

fn pop_rejected_plan_patch_record(
    cursor: &mut Cursor<'_>,
) -> Result<RejectedPlanPatchRecord, CanonError> {
    Ok(RejectedPlanPatchRecord {
        schema_version: PLAN_PATCH_SCHEMA_VERSION,
        record_type: PLAN_PATCH_REJECTED_RECORD,
        source_hash: cursor.take()?,
        cycle_id: cursor.take()?,
        patch_seq: cursor.take()?,
        contract_hash: cursor.take()?,
        patch_hash: cursor.take()?,
        reason_hash: cursor.take()?,
        rejection_hash: cursor.take()?,
    })
}

fn pop_plan_patch_payload(
    cursor: &mut Cursor<'_>,
    kind: PlanPatchKind,
) -> Result<PlanPatchPayload, CanonError> {
    Ok(match kind {
        PlanPatchKind::NodeUpsert => PlanPatchPayload::NodeUpsert(PlanNodeUpsertPatch {
            node_id_hash: cursor.take()?,
            title_hash: cursor.take()?,
            description_hash: cursor.take()?,
            status: plan_node_status_from_u64(cursor.take()?)?,
            assignee_hash: cursor.take()?,
            score_axes_hash: cursor.take()?,
            files_hash: cursor.take()?,
        }),
        PlanPatchKind::EdgeAdd => PlanPatchPayload::EdgeAdd(PlanEdgePatch {
            from_node_hash: cursor.take()?,
            to_node_hash: cursor.take()?,
        }),
        PlanPatchKind::EdgeRemove => PlanPatchPayload::EdgeRemove(PlanEdgePatch {
            from_node_hash: cursor.take()?,
            to_node_hash: cursor.take()?,
        }),
        PlanPatchKind::NodeRemove => PlanPatchPayload::NodeRemove(PlanNodeRemovePatch {
            node_id_hash: cursor.take()?,
        }),
        PlanPatchKind::StatusChange => PlanPatchPayload::StatusChange(PlanStatusChangePatch {
            node_id_hash: cursor.take()?,
            status: plan_node_status_from_u64(cursor.take()?)?,
        }),
        PlanPatchKind::AssigneeChange => {
            PlanPatchPayload::AssigneeChange(PlanAssigneeChangePatch {
                node_id_hash: cursor.take()?,
                assignee_hash: cursor.take()?,
            })
        }
        PlanPatchKind::EvidenceAppend => {
            PlanPatchPayload::EvidenceAppend(PlanEvidenceAppendPatch {
                node_id_hash: cursor.take()?,
                path_hash: cursor.take()?,
                kind_hash: cursor.take()?,
                summary_hash: cursor.take()?,
            })
        }
        PlanPatchKind::FullImport => PlanPatchPayload::FullImport(PlanFullImportPatch {
            node_count: cursor.take()?,
            edge_count: cursor.take()?,
            nodes_hash: cursor.take()?,
            edges_hash: cursor.take()?,
        }),
    })
}

fn plan_patch_tlog_record_is_consistent(record: PlanPatchTlogRecord) -> bool {
    match record {
        PlanPatchTlogRecord::Patch(record) => record.is_self_consistent(),
        PlanPatchTlogRecord::Accepted(record) => record.is_self_consistent(),
        PlanPatchTlogRecord::Rejected(record) => record.is_self_consistent(),
    }
}

fn u8_from_u64(value: u64) -> Result<u8, CanonError> {
    u8::try_from(value).map_err(|_| CanonError::InvalidTlogRecord)
}

fn u16_from_u64(value: u64) -> Result<u16, CanonError> {
    u16::try_from(value).map_err(|_| CanonError::InvalidTlogRecord)
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
    if value == 0 {
        Ok(None)
    } else {
        failure_from_u64(value).map(Some)
    }
}

fn opt_recovery_from_u64(value: u64) -> Result<Option<RecoveryAction>, CanonError> {
    if value == 0 {
        Ok(None)
    } else {
        recovery_from_u64(value).map(Some)
    }
}

fn opt_gate_from_u64(value: u64) -> Result<Option<GateId>, CanonError> {
    if value == 0 {
        Ok(None)
    } else {
        gate_id_from_u64(value).map(Some)
    }
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
    (19, Evidence::AgentCycleEvent),
    (20, Evidence::WaveDispatched),
    (21, Evidence::ChildTaskComplete),
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
    (21, Cause::AgentCycleEventSubmitted),
    (22, Cause::WaveDispatched),
    (23, Cause::ChildTaskCompleted),
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

const PLAN_NODE_STATUS_TAGS: &[(u64, PlanNodeStatus)] = &[
    (1, PlanNodeStatus::Pending),
    (2, PlanNodeStatus::Running),
    (3, PlanNodeStatus::Done),
    (4, PlanNodeStatus::Failed),
    (5, PlanNodeStatus::Skipped),
];

const PLAN_PATCH_KIND_TAGS: &[(u64, PlanPatchKind)] = &[
    (1, PlanPatchKind::NodeUpsert),
    (2, PlanPatchKind::EdgeAdd),
    (3, PlanPatchKind::EdgeRemove),
    (4, PlanPatchKind::NodeRemove),
    (5, PlanPatchKind::StatusChange),
    (6, PlanPatchKind::AssigneeChange),
    (7, PlanPatchKind::EvidenceAppend),
    (8, PlanPatchKind::FullImport),
];

macro_rules! decode_enum_from_u64 {
    ($name:ident, $ty:ty, $table:ident) => {
        fn $name(value: u64) -> Result<$ty, CanonError> {
            enum_from_u64(value, $table)
        }
    };
}

decode_enum_from_u64!(phase_from_u64, Phase, PHASE_TAGS);
decode_enum_from_u64!(gate_status_from_u64, GateStatus, GATE_STATUS_TAGS);
decode_enum_from_u64!(gate_id_from_u64, GateId, GATE_ID_TAGS);
decode_enum_from_u64!(evidence_from_u64, Evidence, EVIDENCE_TAGS);
decode_enum_from_u64!(failure_from_u64, FailureClass, FAILURE_TAGS);
decode_enum_from_u64!(recovery_from_u64, RecoveryAction, RECOVERY_TAGS);
decode_enum_from_u64!(event_kind_from_u64, EventKind, EVENT_KIND_TAGS);
decode_enum_from_u64!(cause_from_u64, Cause, CAUSE_TAGS);
decode_enum_from_u64!(decision_from_u64, Decision, DECISION_TAGS);
decode_enum_from_u64!(semantic_delta_from_u64, SemanticDelta, SEMANTIC_DELTA_TAGS);
decode_enum_from_u64!(
    plan_node_status_from_u64,
    PlanNodeStatus,
    PLAN_NODE_STATUS_TAGS
);
decode_enum_from_u64!(
    plan_patch_kind_from_u64,
    PlanPatchKind,
    PLAN_PATCH_KIND_TAGS
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::{RuntimeConfig, State, TLog};
    use crate::runtime::{tick, verify_tlog};

    fn tmp_tlog_path(name: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let dir = std::path::PathBuf::from("target/test-tmp/ndjson-codec");
        std::fs::create_dir_all(&dir).expect("test dir should exist");
        dir.join(format!(
            "codec-{name}-{}-{nanos}.ndjson",
            std::process::id()
        ))
    }

    fn make_events(n: usize) -> (State, TLog) {
        let cfg = RuntimeConfig::default();
        let mut state = State::default();
        let mut tlog = TLog::default();
        for _ in 0..n {
            tick(&mut state, &mut tlog, cfg).expect("tick should succeed");
        }
        (state, tlog)
    }

    #[test]
    fn empty_event_slice_is_no_op() {
        let path = tmp_tlog_path("empty-noop");
        let _ = std::fs::remove_file(&path);
        append_tlog_events_ndjson(&path, &[]).expect("empty append should succeed");
        assert!(
            !path.exists(),
            "no file should be created for an empty event slice"
        );
    }

    #[test]
    fn empty_event_slice_with_policy_is_no_op() {
        let path = tmp_tlog_path("empty-noop-policy");
        let _ = std::fs::remove_file(&path);
        append_tlog_events_ndjson_with_policy(&path, &[], AppendPolicy::SyncEveryAppend)
            .expect("empty append should succeed");
        assert!(!path.exists());
    }

    #[test]
    fn multi_event_append_writes_all_events_in_order() {
        let path = tmp_tlog_path("multi-event-order");
        let _ = std::fs::remove_file(&path);
        let (_, tlog) = make_events(3);

        append_tlog_events_ndjson(&path, &tlog).expect("append should succeed");

        let loaded = load_tlog_ndjson(&path).expect("loaded tlog should parse");
        assert_eq!(loaded.len(), tlog.len(), "all events should be written");
        for (written, loaded) in tlog.iter().zip(loaded.iter()) {
            assert_eq!(written.seq, loaded.seq, "event order should be preserved");
            assert_eq!(
                written.self_hash, loaded.self_hash,
                "event hash should round-trip"
            );
        }
        verify_tlog(&loaded).expect("appended tlog should verify");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn batch_policy_calls_sync_once_and_is_equivalent_to_default() {
        let path_batch = tmp_tlog_path("policy-batch");
        let path_default = tmp_tlog_path("policy-default");
        let _ = std::fs::remove_file(&path_batch);
        let _ = std::fs::remove_file(&path_default);
        let (_, tlog) = make_events(4);

        append_tlog_events_ndjson_with_policy(&path_batch, &tlog, AppendPolicy::SyncEveryBatch)
            .expect("batch policy append should succeed");
        append_tlog_events_ndjson(&path_default, &tlog).expect("default append should succeed");

        let loaded_batch = load_tlog_ndjson(&path_batch).expect("batch tlog should load");
        let loaded_default = load_tlog_ndjson(&path_default).expect("default tlog should load");

        assert_eq!(
            loaded_batch.len(),
            loaded_default.len(),
            "batch and default should write the same number of events"
        );
        for (a, b) in loaded_batch.iter().zip(loaded_default.iter()) {
            assert_eq!(a.self_hash, b.self_hash, "event hashes should match");
        }
        let _ = std::fs::remove_file(path_batch);
        let _ = std::fs::remove_file(path_default);
    }

    #[test]
    fn incremental_appends_produce_same_result_as_single_batch() {
        let path_incremental = tmp_tlog_path("incremental");
        let path_batch = tmp_tlog_path("batch");
        let _ = std::fs::remove_file(&path_incremental);
        let _ = std::fs::remove_file(&path_batch);
        let (_, tlog) = make_events(3);

        for event in &tlog {
            append_tlog_events_ndjson(&path_incremental, std::slice::from_ref(event))
                .expect("incremental append should succeed");
        }
        append_tlog_events_ndjson(&path_batch, &tlog).expect("batch append should succeed");

        let incremental =
            load_tlog_ndjson(&path_incremental).expect("incremental tlog should load");
        let batch = load_tlog_ndjson(&path_batch).expect("batch tlog should load");

        assert_eq!(incremental.len(), batch.len());
        for (a, b) in incremental.iter().zip(batch.iter()) {
            assert_eq!(a.self_hash, b.self_hash);
        }
        verify_tlog(&incremental).expect("incremental tlog should verify");
        let _ = std::fs::remove_file(path_incremental);
        let _ = std::fs::remove_file(path_batch);
    }
}
