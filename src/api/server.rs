//! Axum worker API surface.
//!
//! The worker server is an adapter around `ApiTransportSession`. Routes do not
//! mutate kernel state directly; command ingress is converted into an
//! `ApiTransportFrame` and handled by the existing deterministic transport
//! boundary.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use axum::extract::State as AxumState;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::api::protocol::{Command, CommandEnvelope};
use crate::api::transport::{ApiTransportDisposition, ApiTransportFrame, ApiTransportSession};
use crate::capability::execution::{
    ActionCallRequest as McpCallRequest, ActionReceipt as McpCallReceipt,
};
use crate::capability::execution::{
    Effect, SandboxProcessReceipt, SandboxProcessRequest, ToolEffectKind,
};
use crate::capability::observation::{
    ObservationCursor, ObservationIngressBatch, ObservationRecord,
};
use crate::capability::orchestration::{
    AgentCycleEvent, AgentCycleEventKind, ChildCompleteRecord, WaveRecord,
};
use crate::capability::{CapabilityId, CapabilityRegistry, EvidenceSubmission, PacketEffect};
use crate::codec::ndjson::append_tlog_events_ndjson;
use crate::kernel::CanonError;
use crate::kernel::{ControlEvent, Evidence, GateId, RuntimeConfig, State};
use crate::runtime::{MailboxMessageReceipt, MailboxMessageRequest};

#[derive(Clone)]
pub struct WorkerAppState {
    inner: Arc<Mutex<Option<WorkerSession>>>,
}

impl WorkerAppState {
    pub fn new(session: ApiTransportSession, tlog_path: impl Into<PathBuf>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(WorkerSession {
                session,
                tlog_path: tlog_path.into(),
                next_request_id: 1,
            }))),
        }
    }

    pub fn new_default(tlog_path: impl Into<PathBuf>) -> Self {
        Self::new(
            ApiTransportSession::new(State::default(), RuntimeConfig::default()),
            tlog_path,
        )
    }

    /// Create a state that serves the health endpoint immediately while session
    /// loading is still in progress. Call `set_ready` once loading completes.
    pub fn loading() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    /// Transition from loading to ready. Called from the background load task.
    pub fn set_ready(&self, session: ApiTransportSession, tlog_path: impl Into<PathBuf>) {
        if let Ok(mut guard) = self.inner.lock() {
            *guard = Some(WorkerSession {
                session,
                tlog_path: tlog_path.into(),
                next_request_id: 1,
            });
        }
    }

    pub fn is_ready(&self) -> Result<bool, ServerError> {
        let guard = self.inner.lock().map_err(|_| ServerError::LockPoisoned)?;
        Ok(guard.is_some())
    }

    pub fn snapshot(&self) -> Result<StateDto, ServerError> {
        let guard = self.inner.lock().map_err(|_| ServerError::LockPoisoned)?;
        let session = guard.as_ref().ok_or(ServerError::WorkerLoading)?;
        Ok(state_dto(&session.session))
    }
}

pub struct WorkerSession {
    session: ApiTransportSession,
    tlog_path: PathBuf,
    next_request_id: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandEnvelopeDto {
    pub command_id: u64,
    pub command_hash: u64,
    pub payload_tag: String,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommandResponseDto {
    pub ok: bool,
    pub request_id: u64,
    pub event_seq: u64,
    pub event_hash: u64,
    pub phase: String,
    pub disposition: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateDto {
    pub phase: String,
    pub tlog_len: usize,
    pub objective_id: u64,
    pub task_id: u64,
    pub failure: Option<String>,
    pub recovery_action: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceSubmissionDto {
    pub gate: String,
    pub evidence: String,
    pub passed: bool,
    pub effect: Option<String>,
    pub payload_hash: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpCallRequestDto {
    pub registry_policy_hash: u64,
    pub worker_url_hash: u64,
    pub tool_name_hash: u64,
    pub args_hash: u64,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct McpCallReceiptDto {
    pub request_hash: u64,
    pub registry_policy_hash: u64,
    pub worker_url_hash: u64,
    pub tool_name_hash: u64,
    pub args_hash: u64,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
    pub effect_kind: u64,
    pub effect_digest: u64,
    pub effect_metadata: u64,
    pub response_hash: u64,
    pub response_bytes: u64,
    pub exit_status: u64,
    pub timed_out: bool,
    pub receipt_hash: u64,
}

impl<'de> Deserialize<'de> for McpCallReceiptDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(Self {
            request_hash: dto_u64(&value, "request_hash")?,
            registry_policy_hash: dto_u64(&value, "registry_policy_hash")?,
            worker_url_hash: dto_u64(&value, "worker_url_hash")?,
            tool_name_hash: dto_u64(&value, "tool_name_hash")?,
            args_hash: dto_u64(&value, "args_hash")?,
            timeout_ms: dto_u64(&value, "timeout_ms")?,
            max_output_bytes: dto_u64(&value, "max_output_bytes")?,
            effect_kind: dto_u64(&value, "effect_kind")?,
            effect_digest: dto_u64(&value, "effect_digest")?,
            effect_metadata: dto_u64(&value, "effect_metadata")?,
            response_hash: dto_u64(&value, "response_hash")?,
            response_bytes: dto_u64(&value, "response_bytes")?,
            exit_status: dto_u64(&value, "exit_status")?,
            timed_out: dto_bool(&value, "timed_out")?,
            receipt_hash: dto_u64(&value, "receipt_hash")?,
        })
    }
}

fn dto_u64<E>(value: &serde_json::Value, field: &'static str) -> Result<u64, E>
where
    E: serde::de::Error,
{
    value
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| E::missing_field(field))
}

fn dto_bool<E>(value: &serde_json::Value, field: &'static str) -> Result<bool, E>
where
    E: serde::de::Error,
{
    value
        .get(field)
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| E::missing_field(field))
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SandboxProcessRequestDto {
    pub registry_policy_hash: u64,
    pub command_hash: u64,
    pub argv_hash: u64,
    pub cwd_hash: u64,
    pub env_hash: u64,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SandboxProcessReceiptDto {
    pub request_hash: u64,
    pub registry_policy_hash: u64,
    pub command_hash: u64,
    pub argv_hash: u64,
    pub cwd_hash: u64,
    pub env_hash: u64,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
    pub effect_kind: u64,
    pub effect_digest: u64,
    pub effect_metadata: u64,
    pub stdout_hash: u64,
    pub stderr_hash: u64,
    pub stdout_bytes: u64,
    pub stderr_bytes: u64,
    pub exit_status: u64,
    pub timed_out: bool,
    pub receipt_hash: u64,
}

impl<'de> Deserialize<'de> for SandboxProcessReceiptDto {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        Ok(Self {
            request_hash: dto_u64(&value, "request_hash")?,
            registry_policy_hash: dto_u64(&value, "registry_policy_hash")?,
            command_hash: dto_u64(&value, "command_hash")?,
            argv_hash: dto_u64(&value, "argv_hash")?,
            cwd_hash: dto_u64(&value, "cwd_hash")?,
            env_hash: dto_u64(&value, "env_hash")?,
            timeout_ms: dto_u64(&value, "timeout_ms")?,
            max_output_bytes: dto_u64(&value, "max_output_bytes")?,
            effect_kind: dto_u64(&value, "effect_kind")?,
            effect_digest: dto_u64(&value, "effect_digest")?,
            effect_metadata: dto_u64(&value, "effect_metadata")?,
            stdout_hash: dto_u64(&value, "stdout_hash")?,
            stderr_hash: dto_u64(&value, "stderr_hash")?,
            stdout_bytes: dto_u64(&value, "stdout_bytes")?,
            stderr_bytes: dto_u64(&value, "stderr_bytes")?,
            exit_status: dto_u64(&value, "exit_status")?,
            timed_out: dto_bool(&value, "timed_out")?,
            receipt_hash: dto_u64(&value, "receipt_hash")?,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MailboxMessageRequestDto {
    pub registry_policy_hash: u64,
    pub sender_hash: u64,
    pub target_hash: u64,
    pub kind_hash: u64,
    pub payload_hash: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MailboxMessageReceiptDto {
    pub request_hash: u64,
    pub registry_policy_hash: u64,
    pub sender_hash: u64,
    pub target_hash: u64,
    pub kind_hash: u64,
    pub payload_hash: u64,
    pub message_id_hash: u64,
    pub sent_at_hash: u64,
    pub record_hash: u64,
    pub receipt_hash: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ErrorDto {
    pub ok: bool,
    pub error: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObservationRecordDto {
    pub source_id: u64,
    pub sequence: u64,
    pub observed_hash: u64,
    pub received_at_tick: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObservationIngressBatchDto {
    pub source_id: u64,
    pub source_hash: u64,
    pub cursor_source_id: u64,
    pub cursor_last_sequence: u64,
    pub cursor_last_observed_hash: u64,
    pub backlog_len: u64,
    pub records: Vec<ObservationRecordDto>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ServerError {
    LockPoisoned,
    WorkerLoading,
    UnsupportedPayloadTag,
    InvalidPayload,
    InvalidCommand,
    Transport(CanonError),
    TlogIo,
}

pub fn build_router(state: WorkerAppState) -> Router {
    Router::new()
        .route("/v1/command", post(post_command))
        .route("/v1/state", get(get_state))
        .route("/health/worker", get(health))
        .with_state(state)
}

pub async fn health(
    AxumState(state): AxumState<WorkerAppState>,
) -> Result<&'static str, (StatusCode, Json<ErrorDto>)> {
    if state.is_ready().map_err(error_response)? {
        Ok("ok")
    } else {
        Err(error_response(ServerError::WorkerLoading))
    }
}

pub async fn get_state(
    AxumState(state): AxumState<WorkerAppState>,
) -> Result<Json<StateDto>, (StatusCode, Json<ErrorDto>)> {
    state.snapshot().map(Json).map_err(error_response)
}

pub async fn post_command(
    AxumState(state): AxumState<WorkerAppState>,
    Json(dto): Json<CommandEnvelopeDto>,
) -> Result<Json<CommandResponseDto>, (StatusCode, Json<ErrorDto>)> {
    let command = decode_command(&dto)?;
    let envelope = CommandEnvelope {
        schema_version: crate::api::protocol::API_PROTOCOL_SCHEMA_VERSION,
        command_id: dto.command_id,
        command_hash: dto.command_hash,
        command,
    };
    if !envelope.is_contract_valid() {
        return Err(ServerError::InvalidCommand.into());
    }

    let mut guard = state.inner.lock().map_err(|_| ServerError::LockPoisoned)?;
    let session = match &mut *guard {
        Some(s) => s,
        None => return Err(error_response(ServerError::WorkerLoading)),
    };
    let request_id = session.next_request_id;
    session.next_request_id = session
        .next_request_id
        .checked_add(1)
        .ok_or(ServerError::InvalidCommand)?;

    let old_tlog_len = session.session.tlog().len();
    let frame = ApiTransportFrame::new(request_id, envelope);
    let response = session
        .session
        .handle_frame(frame)
        .map_err(ServerError::Transport)?;
    persist_tlog_tail(&session.tlog_path, session.session.tlog(), old_tlog_len)
        .map_err(|_| ServerError::TlogIo)?;

    Ok(Json(CommandResponseDto {
        ok: true,
        request_id: response.request_id,
        event_seq: response.control.event.seq,
        event_hash: response.event_hash,
        phase: format!("{:?}", response.control.event.state_after.phase),
        disposition: disposition_string(response.disposition).to_string(),
    }))
}

fn persist_tlog_tail(
    tlog_path: &std::path::Path,
    tlog: &[ControlEvent],
    old_tlog_len: usize,
) -> Result<(), CanonError> {
    let events = if old_tlog_len > 0 && !tlog_path.exists() {
        tlog
    } else {
        &tlog[old_tlog_len..]
    };
    append_tlog_events_ndjson(tlog_path, events)
}

fn decode_command(dto: &CommandEnvelopeDto) -> Result<Command, ServerError> {
    let decoder = command_decoder(dto.payload_tag.as_str())?;
    decoder(dto.payload.clone())
}

type CommandDecoder = fn(serde_json::Value) -> Result<Command, ServerError>;

fn command_decoder(payload_tag: &str) -> Result<CommandDecoder, ServerError> {
    Ok(match payload_tag {
        "SubmitEvidence" => decode_submit_evidence_command,
        "SubmitObservationIngress" => decode_submit_observation_ingress_command,
        "SubmitEvidenceBatch" => decode_submit_evidence_batch_command,
        "AuthorizeActionCall" => decode_authorize_action_call_command,
        "AuthorizeMcpCall" => decode_authorize_mcp_call_command,
        "AuthorizeProcessCall" => decode_authorize_process_call_command,
        "AuthorizeMailboxMessage" => decode_authorize_mailbox_message_command,
        "SubmitActionReceipt" => decode_submit_action_receipt_command,
        "SubmitMcpCallReceipt" => decode_submit_mcp_call_receipt_command,
        "SubmitProcessReceipt" => decode_submit_process_receipt_command,
        "SubmitMailboxMessageReceipt" => decode_submit_mailbox_message_receipt_command,
        "SubmitProcessReceiptBatch" => decode_submit_process_receipt_batch_command,
        "SubmitAgentCycleEvent" => decode_submit_agent_cycle_event_command,
        "SubmitWaveDispatch" => decode_submit_wave_dispatch_command,
        "SubmitChildComplete" => decode_submit_child_complete_command,
        _ => return Err(ServerError::UnsupportedPayloadTag),
    })
}

fn decode_submit_evidence_command(payload: serde_json::Value) -> Result<Command, ServerError> {
    let submission = decode_submission(payload)?;
    // InvariantProof requires typed Observation path — reject raw submissions.
    if submission.gate == GateId::Invariant {
        return Err(ServerError::InvalidCommand);
    }
    Ok(Command::SubmitEvidence(submission))
}

fn decode_submit_observation_ingress_command(
    payload: serde_json::Value,
) -> Result<Command, ServerError> {
    Ok(Command::SubmitObservationIngress(
        decode_observation_ingress(payload)?,
    ))
}

fn decode_submit_evidence_batch_command(
    payload: serde_json::Value,
) -> Result<Command, ServerError> {
    let payloads: Vec<EvidenceSubmissionDto> =
        serde_json::from_value(payload).map_err(|_| ServerError::InvalidPayload)?;
    let submissions = payloads
        .into_iter()
        .map(submission_from_dto)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Command::SubmitEvidenceBatch(submissions))
}

fn decode_authorize_mcp_call_command(payload: serde_json::Value) -> Result<Command, ServerError> {
    Ok(Command::AuthorizeMcpCall(decode_mcp_call_request(payload)?))
}

fn decode_authorize_action_call_command(
    payload: serde_json::Value,
) -> Result<Command, ServerError> {
    Ok(Command::AuthorizeActionCall(decode_mcp_call_request(
        payload,
    )?))
}

fn decode_authorize_process_call_command(
    payload: serde_json::Value,
) -> Result<Command, ServerError> {
    Ok(Command::AuthorizeProcessCall(
        decode_sandbox_process_request(payload)?,
    ))
}

fn decode_authorize_mailbox_message_command(
    payload: serde_json::Value,
) -> Result<Command, ServerError> {
    Ok(Command::AuthorizeMailboxMessage(
        decode_mailbox_message_request(payload)?,
    ))
}

fn decode_submit_mcp_call_receipt_command(
    payload: serde_json::Value,
) -> Result<Command, ServerError> {
    Ok(Command::SubmitMcpCallReceipt(decode_mcp_call_receipt(
        payload,
    )?))
}

fn decode_submit_action_receipt_command(
    payload: serde_json::Value,
) -> Result<Command, ServerError> {
    Ok(Command::SubmitActionReceipt(decode_mcp_call_receipt(
        payload,
    )?))
}

fn decode_submit_process_receipt_command(
    payload: serde_json::Value,
) -> Result<Command, ServerError> {
    Ok(Command::SubmitProcessReceipt(
        decode_sandbox_process_receipt(payload)?,
    ))
}

fn decode_submit_mailbox_message_receipt_command(
    payload: serde_json::Value,
) -> Result<Command, ServerError> {
    Ok(Command::SubmitMailboxMessageReceipt(
        decode_mailbox_message_receipt(payload)?,
    ))
}

fn decode_submit_process_receipt_batch_command(
    payload: serde_json::Value,
) -> Result<Command, ServerError> {
    let payloads: Vec<SandboxProcessReceiptDto> =
        serde_json::from_value(payload).map_err(|_| ServerError::InvalidPayload)?;
    let receipts = payloads
        .into_iter()
        .map(sandbox_process_receipt_from_dto)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Command::SubmitProcessReceiptBatch(receipts))
}

fn decode_submit_agent_cycle_event_command(
    payload: serde_json::Value,
) -> Result<Command, ServerError> {
    let kind_u64 = payload
        .get("kind")
        .and_then(serde_json::Value::as_u64)
        .ok_or(ServerError::InvalidPayload)?;
    let kind = AgentCycleEventKind::from_route_u64(kind_u64).ok_or(ServerError::InvalidPayload)?;
    let agent_hash = payload
        .get("agent_hash")
        .and_then(serde_json::Value::as_u64)
        .ok_or(ServerError::InvalidPayload)?;
    let cycle = payload
        .get("cycle")
        .and_then(serde_json::Value::as_u64)
        .ok_or(ServerError::InvalidPayload)?;
    let label_hash = payload
        .get("label_hash")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let content_hash = payload
        .get("content_hash")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let event = AgentCycleEvent::new(kind, agent_hash, cycle, label_hash, content_hash);
    if !event.is_contract_valid() {
        return Err(ServerError::InvalidCommand);
    }
    Ok(Command::SubmitAgentCycleEvent(event))
}

fn decode_submit_wave_dispatch_command(payload: serde_json::Value) -> Result<Command, ServerError> {
    let wave_id = payload
        .get("wave_id")
        .and_then(serde_json::Value::as_u64)
        .ok_or(ServerError::InvalidPayload)?;
    let parent_hash = payload
        .get("parent_hash")
        .and_then(serde_json::Value::as_u64)
        .ok_or(ServerError::InvalidPayload)?;
    let cycle = payload
        .get("cycle")
        .and_then(serde_json::Value::as_u64)
        .ok_or(ServerError::InvalidPayload)?;
    let node_count = payload
        .get("node_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or(ServerError::InvalidPayload)? as u16;
    let node_ids_hash = payload
        .get("node_ids_hash")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    let record = WaveRecord::new(wave_id, parent_hash, cycle, node_count, node_ids_hash);
    if !record.is_contract_valid() {
        return Err(ServerError::InvalidCommand);
    }
    Ok(Command::SubmitWaveDispatch(record))
}

fn decode_submit_child_complete_command(
    payload: serde_json::Value,
) -> Result<Command, ServerError> {
    let wave_id = payload
        .get("wave_id")
        .and_then(serde_json::Value::as_u64)
        .ok_or(ServerError::InvalidPayload)?;
    let node_id_hash = payload
        .get("node_id_hash")
        .and_then(serde_json::Value::as_u64)
        .ok_or(ServerError::InvalidPayload)?;
    let panicked = payload
        .get("exit_status")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
        != 0;
    let record = ChildCompleteRecord::new(wave_id, node_id_hash, panicked);
    if !record.is_contract_valid() {
        return Err(ServerError::InvalidCommand);
    }
    Ok(Command::SubmitChildComplete(record))
}

fn decode_mcp_call_request(payload: serde_json::Value) -> Result<McpCallRequest, ServerError> {
    let dto: McpCallRequestDto = decode_mcp_call_payload(payload)?;
    let request = McpCallRequest {
        capability: CapabilityId::Tooling,
        registry_policy_hash: dto.registry_policy_hash,
        worker_url_hash: dto.worker_url_hash,
        tool_name_hash: dto.tool_name_hash,
        args_hash: dto.args_hash,
        timeout_ms: dto.timeout_ms,
        max_output_bytes: dto.max_output_bytes,
    };
    validate_mcp_registry_policy(request.registry_policy_hash)?;
    if !request.is_admissible() {
        return Err(ServerError::InvalidCommand);
    }
    Ok(request)
}

fn decode_mailbox_message_request(
    payload: serde_json::Value,
) -> Result<MailboxMessageRequest, ServerError> {
    let dto: MailboxMessageRequestDto =
        serde_json::from_value(payload).map_err(|_| ServerError::InvalidPayload)?;
    let request = MailboxMessageRequest {
        capability_id: crate::runtime::MAILBOX_MESSAGE_CAPABILITY_ID,
        registry_policy_hash: dto.registry_policy_hash,
        sender_hash: dto.sender_hash,
        target_hash: dto.target_hash,
        kind_hash: dto.kind_hash,
        payload_hash: dto.payload_hash,
    };
    if request.registry_policy_hash != CapabilityRegistry::canonical().policy_hash()
        || !request.is_admissible()
    {
        return Err(ServerError::InvalidCommand);
    }
    Ok(request)
}

fn decode_mailbox_message_receipt(
    payload: serde_json::Value,
) -> Result<MailboxMessageReceipt, ServerError> {
    let dto: MailboxMessageReceiptDto =
        serde_json::from_value(payload).map_err(|_| ServerError::InvalidPayload)?;
    let receipt = MailboxMessageReceipt {
        request_hash: dto.request_hash,
        registry_policy_hash: dto.registry_policy_hash,
        sender_hash: dto.sender_hash,
        target_hash: dto.target_hash,
        kind_hash: dto.kind_hash,
        payload_hash: dto.payload_hash,
        message_id_hash: dto.message_id_hash,
        sent_at_hash: dto.sent_at_hash,
        record_hash: dto.record_hash,
        receipt_hash: dto.receipt_hash,
    };
    if receipt.registry_policy_hash != CapabilityRegistry::canonical().policy_hash()
        || !receipt.is_contract_valid()
    {
        return Err(ServerError::InvalidCommand);
    }
    Ok(receipt)
}

fn decode_mcp_call_receipt(payload: serde_json::Value) -> Result<McpCallReceipt, ServerError> {
    let dto: McpCallReceiptDto = decode_mcp_call_payload(payload)?;
    let effect_kind = match dto.effect_kind {
        2 => ToolEffectKind::Process,
        _ => return Err(ServerError::InvalidPayload),
    };
    let receipt = McpCallReceipt {
        request_hash: dto.request_hash,
        registry_policy_hash: dto.registry_policy_hash,
        worker_url_hash: dto.worker_url_hash,
        tool_name_hash: dto.tool_name_hash,
        args_hash: dto.args_hash,
        timeout_ms: dto.timeout_ms,
        max_output_bytes: dto.max_output_bytes,
        effect: Effect {
            kind: effect_kind,
            digest: dto.effect_digest,
            metadata: dto.effect_metadata,
        },
        response_hash: dto.response_hash,
        response_bytes: dto.response_bytes,
        exit_status: dto.exit_status,
        timed_out: dto.timed_out,
        receipt_hash: dto.receipt_hash,
    };
    validate_mcp_registry_policy(receipt.registry_policy_hash)?;
    if !receipt.is_contract_valid() {
        return Err(ServerError::InvalidCommand);
    }
    Ok(receipt)
}

fn decode_mcp_call_payload<T>(payload: serde_json::Value) -> Result<T, ServerError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_value(payload).map_err(|_| ServerError::InvalidPayload)
}

fn validate_mcp_registry_policy(registry_policy_hash: u64) -> Result<(), ServerError> {
    if registry_policy_hash != CapabilityRegistry::canonical().policy_hash() {
        return Err(ServerError::InvalidCommand);
    }
    Ok(())
}

fn decode_sandbox_process_request(
    payload: serde_json::Value,
) -> Result<SandboxProcessRequest, ServerError> {
    let dto: SandboxProcessRequestDto =
        serde_json::from_value(payload).map_err(|_| ServerError::InvalidPayload)?;
    let request = SandboxProcessRequest {
        capability: CapabilityId::Tooling,
        registry_policy_hash: dto.registry_policy_hash,
        command_hash: dto.command_hash,
        argv_hash: dto.argv_hash,
        cwd_hash: dto.cwd_hash,
        env_hash: dto.env_hash,
        timeout_ms: dto.timeout_ms,
        max_output_bytes: dto.max_output_bytes,
    };
    if request.registry_policy_hash != CapabilityRegistry::canonical().policy_hash()
        || !request.is_admissible()
    {
        return Err(ServerError::InvalidCommand);
    }
    Ok(request)
}

fn decode_sandbox_process_receipt(
    payload: serde_json::Value,
) -> Result<SandboxProcessReceipt, ServerError> {
    let dto: SandboxProcessReceiptDto =
        serde_json::from_value(payload).map_err(|_| ServerError::InvalidPayload)?;
    sandbox_process_receipt_from_dto(dto)
}

fn sandbox_process_receipt_from_dto(
    dto: SandboxProcessReceiptDto,
) -> Result<SandboxProcessReceipt, ServerError> {
    let effect_kind = match dto.effect_kind {
        2 => ToolEffectKind::Process,
        _ => return Err(ServerError::InvalidPayload),
    };
    let receipt = SandboxProcessReceipt {
        request_hash: dto.request_hash,
        registry_policy_hash: dto.registry_policy_hash,
        command_hash: dto.command_hash,
        argv_hash: dto.argv_hash,
        cwd_hash: dto.cwd_hash,
        env_hash: dto.env_hash,
        timeout_ms: dto.timeout_ms,
        max_output_bytes: dto.max_output_bytes,
        effect: Effect {
            kind: effect_kind,
            digest: dto.effect_digest,
            metadata: dto.effect_metadata,
        },
        stdout_hash: dto.stdout_hash,
        stderr_hash: dto.stderr_hash,
        stdout_bytes: dto.stdout_bytes,
        stderr_bytes: dto.stderr_bytes,
        exit_status: dto.exit_status,
        timed_out: dto.timed_out,
        receipt_hash: dto.receipt_hash,
    };
    if receipt.registry_policy_hash != CapabilityRegistry::canonical().policy_hash()
        || !receipt.is_contract_valid()
    {
        return Err(ServerError::InvalidCommand);
    }
    Ok(receipt)
}

fn decode_submission(payload: serde_json::Value) -> Result<EvidenceSubmission, ServerError> {
    let dto: EvidenceSubmissionDto =
        serde_json::from_value(payload).map_err(|_| ServerError::InvalidPayload)?;
    submission_from_dto(dto)
}

fn decode_observation_ingress(
    payload: serde_json::Value,
) -> Result<ObservationIngressBatch, ServerError> {
    let dto: ObservationIngressBatchDto =
        serde_json::from_value(payload).map_err(|_| ServerError::InvalidPayload)?;
    let cursor = ObservationCursor {
        source_id: dto.cursor_source_id,
        last_sequence: dto.cursor_last_sequence,
        last_observed_hash: dto.cursor_last_observed_hash,
    };
    let records: Vec<ObservationRecord> = dto
        .records
        .into_iter()
        .map(|r| {
            ObservationRecord::new(r.source_id, r.sequence, r.observed_hash, r.received_at_tick)
        })
        .collect();
    let batch = if records.is_empty() {
        ObservationIngressBatch::empty(dto.source_id, dto.source_hash, cursor)
    } else {
        ObservationIngressBatch::accepted(
            dto.source_id,
            dto.source_hash,
            cursor,
            dto.backlog_len as usize,
            records,
        )
    };
    // Enforce capability: only Observation may submit InvariantProof.
    if !CapabilityRegistry::canonical().allows(CapabilityId::Observation, batch.submission()) {
        return Err(ServerError::InvalidCommand);
    }
    Ok(batch)
}

fn submission_from_dto(dto: EvidenceSubmissionDto) -> Result<EvidenceSubmission, ServerError> {
    let gate = gate_from_str(&dto.gate)?;
    let evidence = evidence_from_str(&dto.evidence)?;
    let effect = match dto.effect.as_deref() {
        None | Some("None") => PacketEffect::None,
        Some("BindReadyTask") => PacketEffect::BindReadyTask,
        Some("MaterializeArtifact") => PacketEffect::MaterializeArtifact,
        Some("RepairLineage") => PacketEffect::RepairLineage,
        Some("CompleteObjective") => PacketEffect::CompleteObjective,
        Some(_) => return Err(ServerError::InvalidPayload),
    };
    let submission = EvidenceSubmission::with_effect_payload(
        gate,
        evidence,
        dto.passed,
        effect,
        dto.payload_hash,
    );
    submission
        .is_contract_valid()
        .then_some(submission)
        .ok_or(ServerError::InvalidCommand)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ApiSubmissionToken {
    Gate(GateId),
    Evidence(Evidence),
}

fn api_submission_token_from_str(value: &str) -> Result<ApiSubmissionToken, ServerError> {
    match value {
        "Invariant" => Ok(ApiSubmissionToken::Gate(GateId::Invariant)),
        "Analysis" => Ok(ApiSubmissionToken::Gate(GateId::Analysis)),
        "Judgment" => Ok(ApiSubmissionToken::Gate(GateId::Judgment)),
        "Plan" => Ok(ApiSubmissionToken::Gate(GateId::Plan)),
        "Execution" => Ok(ApiSubmissionToken::Gate(GateId::Execution)),
        "Verification" => Ok(ApiSubmissionToken::Gate(GateId::Verification)),
        "Eval" => Ok(ApiSubmissionToken::Gate(GateId::Eval)),
        "Learning" => Ok(ApiSubmissionToken::Gate(GateId::Learning)),
        "InvariantProof" => Ok(ApiSubmissionToken::Evidence(Evidence::InvariantProof)),
        "AnalysisReport" => Ok(ApiSubmissionToken::Evidence(Evidence::AnalysisReport)),
        "JudgmentRecord" => Ok(ApiSubmissionToken::Evidence(Evidence::JudgmentRecord)),
        "PlanRecord" => Ok(ApiSubmissionToken::Evidence(Evidence::PlanRecord)),
        "TaskReady" => Ok(ApiSubmissionToken::Evidence(Evidence::TaskReady)),
        "ExecutionReceipt" => Ok(ApiSubmissionToken::Evidence(Evidence::ExecutionReceipt)),
        "ArtifactReceipt" => Ok(ApiSubmissionToken::Evidence(Evidence::ArtifactReceipt)),
        "VerificationReport" => Ok(ApiSubmissionToken::Evidence(Evidence::VerificationReport)),
        "LineageProof" => Ok(ApiSubmissionToken::Evidence(Evidence::LineageProof)),
        "EvalScore" => Ok(ApiSubmissionToken::Evidence(Evidence::EvalScore)),
        "PersistedRecord" => Ok(ApiSubmissionToken::Evidence(Evidence::PersistedRecord)),
        _ => Err(ServerError::InvalidPayload),
    }
}

fn extract_api_submission_token(
    value: &str,
    predicate: impl FnOnce(ApiSubmissionToken) -> Option<ApiSubmissionToken>,
) -> Result<ApiSubmissionToken, ServerError> {
    predicate(api_submission_token_from_str(value)?).ok_or(ServerError::InvalidPayload)
}

fn gate_from_str(value: &str) -> Result<GateId, ServerError> {
    match extract_api_submission_token(value, |token| match token {
        ApiSubmissionToken::Gate(_) => Some(token),
        ApiSubmissionToken::Evidence(_) => None,
    })? {
        ApiSubmissionToken::Gate(gate) => Ok(gate),
        ApiSubmissionToken::Evidence(_) => unreachable!("gate extractor only returns gate tokens"),
    }
}

fn evidence_from_str(value: &str) -> Result<Evidence, ServerError> {
    match extract_api_submission_token(value, |token| match token {
        ApiSubmissionToken::Gate(_) => None,
        ApiSubmissionToken::Evidence(_) => Some(token),
    })? {
        ApiSubmissionToken::Evidence(evidence) => Ok(evidence),
        ApiSubmissionToken::Gate(_) => {
            unreachable!("evidence extractor only returns evidence tokens")
        }
    }
}

fn state_dto(session: &ApiTransportSession) -> StateDto {
    let state = session.state();
    StateDto {
        phase: format!("{:?}", state.phase),
        tlog_len: session.tlog().len(),
        objective_id: state.packet.objective_id,
        task_id: state.packet.active_task_id,
        failure: state.failure.map(|failure| format!("{failure:?}")),
        recovery_action: state.recovery_action.map(|action| format!("{action:?}")),
    }
}

fn disposition_string(disposition: ApiTransportDisposition) -> &'static str {
    match disposition {
        ApiTransportDisposition::Accepted => "accepted",
        ApiTransportDisposition::Replayed => "replayed",
    }
}

fn error_response(error: ServerError) -> (StatusCode, Json<ErrorDto>) {
    let status = match error {
        ServerError::LockPoisoned | ServerError::TlogIo => StatusCode::INTERNAL_SERVER_ERROR,
        ServerError::WorkerLoading => StatusCode::SERVICE_UNAVAILABLE,
        ServerError::UnsupportedPayloadTag
        | ServerError::InvalidPayload
        | ServerError::InvalidCommand => StatusCode::BAD_REQUEST,
        ServerError::Transport(CanonError::InvalidReplay) => StatusCode::CONFLICT,
        ServerError::Transport(_) => StatusCode::BAD_REQUEST,
    };
    (
        status,
        Json(ErrorDto {
            ok: false,
            error: format!("{error:?}"),
        }),
    )
}

impl From<ServerError> for (StatusCode, Json<ErrorDto>) {
    fn from(error: ServerError) -> Self {
        error_response(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_submission_token_parser_preserves_gate_and_evidence_mappings() {
        let gate_cases = [
            ("Invariant", GateId::Invariant),
            ("Analysis", GateId::Analysis),
            ("Judgment", GateId::Judgment),
            ("Plan", GateId::Plan),
            ("Execution", GateId::Execution),
            ("Verification", GateId::Verification),
            ("Eval", GateId::Eval),
            ("Learning", GateId::Learning),
        ];
        for (input, expected) in gate_cases {
            assert_eq!(gate_from_str(input), Ok(expected));
        }
        assert_eq!(
            gate_from_str("UnknownGate"),
            Err(ServerError::InvalidPayload)
        );

        let evidence_cases = [
            ("InvariantProof", Evidence::InvariantProof),
            ("AnalysisReport", Evidence::AnalysisReport),
            ("JudgmentRecord", Evidence::JudgmentRecord),
            ("PlanRecord", Evidence::PlanRecord),
            ("TaskReady", Evidence::TaskReady),
            ("ExecutionReceipt", Evidence::ExecutionReceipt),
            ("ArtifactReceipt", Evidence::ArtifactReceipt),
            ("VerificationReport", Evidence::VerificationReport),
            ("LineageProof", Evidence::LineageProof),
            ("EvalScore", Evidence::EvalScore),
            ("PersistedRecord", Evidence::PersistedRecord),
        ];
        for (input, expected) in evidence_cases {
            assert_eq!(evidence_from_str(input), Ok(expected));
        }
        assert_eq!(
            evidence_from_str("UnknownEvidence"),
            Err(ServerError::InvalidPayload)
        );
    }

    #[test]
    fn api_submission_token_extractors_preserve_gate_evidence_and_error_boundaries() {
        let gate_cases = [
            ("Invariant", GateId::Invariant),
            ("Analysis", GateId::Analysis),
            ("Judgment", GateId::Judgment),
            ("Plan", GateId::Plan),
            ("Execution", GateId::Execution),
            ("Verification", GateId::Verification),
            ("Eval", GateId::Eval),
            ("Learning", GateId::Learning),
        ];
        for (input, expected) in gate_cases {
            assert_eq!(gate_from_str(input), Ok(expected));
            assert_eq!(evidence_from_str(input), Err(ServerError::InvalidPayload));
        }

        let evidence_cases = [
            ("InvariantProof", Evidence::InvariantProof),
            ("AnalysisReport", Evidence::AnalysisReport),
            ("JudgmentRecord", Evidence::JudgmentRecord),
            ("PlanRecord", Evidence::PlanRecord),
            ("TaskReady", Evidence::TaskReady),
            ("ExecutionReceipt", Evidence::ExecutionReceipt),
            ("ArtifactReceipt", Evidence::ArtifactReceipt),
            ("VerificationReport", Evidence::VerificationReport),
            ("LineageProof", Evidence::LineageProof),
            ("EvalScore", Evidence::EvalScore),
            ("PersistedRecord", Evidence::PersistedRecord),
        ];
        for (input, expected) in evidence_cases {
            assert_eq!(evidence_from_str(input), Ok(expected));
            assert_eq!(gate_from_str(input), Err(ServerError::InvalidPayload));
        }

        assert_eq!(
            gate_from_str("UnknownSubmissionToken"),
            Err(ServerError::InvalidPayload)
        );
        assert_eq!(
            evidence_from_str("UnknownSubmissionToken"),
            Err(ServerError::InvalidPayload)
        );

        let submission = submission_from_dto(EvidenceSubmissionDto {
            gate: "Plan".to_string(),
            evidence: "TaskReady".to_string(),
            passed: true,
            effect: Some("BindReadyTask".to_string()),
            payload_hash: 0xfeed_cafe,
        })
        .expect("valid plan/task-ready DTO should build a submission");

        assert_eq!(submission.gate, GateId::Plan);
        assert_eq!(submission.evidence, Evidence::TaskReady);
        assert!(submission.passed);
        assert_eq!(submission.effect, PacketEffect::BindReadyTask);
        assert_eq!(submission.payload_hash, 0xfeed_cafe);
        assert!(submission.is_contract_valid());
    }

    #[test]
    fn mcp_call_decoders_preserve_request_receipt_boundaries() {
        let request = McpCallRequest::new(
            CapabilityRegistry::canonical(),
            "http://127.0.0.1:38469/mcp_worker",
            "shell",
            r#"{"cwd":".","command":"true"}"#,
            1000,
            4096,
        );
        let request_payload = serde_json::json!({
            "registry_policy_hash": request.registry_policy_hash,
            "worker_url_hash": request.worker_url_hash,
            "tool_name_hash": request.tool_name_hash,
            "args_hash": request.args_hash,
            "timeout_ms": request.timeout_ms,
            "max_output_bytes": request.max_output_bytes,
        });

        let decoded_request = decode_mcp_call_request(request_payload.clone())
            .expect("valid MCP call request should decode");
        assert_eq!(decoded_request.capability, CapabilityId::Tooling);
        assert_eq!(
            decoded_request.registry_policy_hash,
            request.registry_policy_hash
        );
        assert_eq!(decoded_request.worker_url_hash, request.worker_url_hash);
        assert_eq!(decoded_request.tool_name_hash, request.tool_name_hash);
        assert_eq!(decoded_request.args_hash, request.args_hash);
        assert_eq!(decoded_request.timeout_ms, request.timeout_ms);
        assert_eq!(decoded_request.max_output_bytes, request.max_output_bytes);
        assert!(decoded_request.is_admissible());

        let receipt = McpCallReceipt::from_response(
            &request,
            br#"{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"ok"}]}}"#,
            0,
            false,
        );
        let receipt_payload = serde_json::json!({
            "request_hash": receipt.request_hash,
            "registry_policy_hash": receipt.registry_policy_hash,
            "worker_url_hash": receipt.worker_url_hash,
            "tool_name_hash": receipt.tool_name_hash,
            "args_hash": receipt.args_hash,
            "timeout_ms": receipt.timeout_ms,
            "max_output_bytes": receipt.max_output_bytes,
            "effect_kind": receipt.effect.kind as u64,
            "effect_digest": receipt.effect.digest,
            "effect_metadata": receipt.effect.metadata,
            "response_hash": receipt.response_hash,
            "response_bytes": receipt.response_bytes,
            "exit_status": receipt.exit_status,
            "timed_out": receipt.timed_out,
            "receipt_hash": receipt.receipt_hash,
        });

        let decoded_receipt = decode_mcp_call_receipt(receipt_payload.clone())
            .expect("valid MCP call receipt should decode");
        assert_eq!(decoded_receipt.request_hash, receipt.request_hash);
        assert_eq!(
            decoded_receipt.registry_policy_hash,
            receipt.registry_policy_hash
        );
        assert_eq!(decoded_receipt.worker_url_hash, receipt.worker_url_hash);
        assert_eq!(decoded_receipt.tool_name_hash, receipt.tool_name_hash);
        assert_eq!(decoded_receipt.args_hash, receipt.args_hash);
        assert_eq!(decoded_receipt.timeout_ms, receipt.timeout_ms);
        assert_eq!(decoded_receipt.max_output_bytes, receipt.max_output_bytes);
        assert_eq!(decoded_receipt.effect.kind, ToolEffectKind::Process);
        assert_eq!(decoded_receipt.effect.digest, receipt.effect.digest);
        assert_eq!(decoded_receipt.effect.metadata, receipt.effect.metadata);
        assert_eq!(decoded_receipt.response_hash, receipt.response_hash);
        assert_eq!(decoded_receipt.response_bytes, receipt.response_bytes);
        assert_eq!(decoded_receipt.exit_status, receipt.exit_status);
        assert_eq!(decoded_receipt.timed_out, receipt.timed_out);
        assert_eq!(decoded_receipt.receipt_hash, receipt.receipt_hash);
        assert!(decoded_receipt.is_contract_valid());

        assert_eq!(
            decode_mcp_call_request(serde_json::json!({"registry_policy_hash": "bad"})),
            Err(ServerError::InvalidPayload)
        );
        assert_eq!(
            decode_mcp_call_receipt(serde_json::json!({"request_hash": "bad"})),
            Err(ServerError::InvalidPayload)
        );

        let mut request_policy_mismatch = request_payload;
        request_policy_mismatch["registry_policy_hash"] = serde_json::json!(u64::MAX);
        assert_eq!(
            decode_mcp_call_request(request_policy_mismatch),
            Err(ServerError::InvalidCommand)
        );

        let mut receipt_policy_mismatch = receipt_payload.clone();
        receipt_policy_mismatch["registry_policy_hash"] = serde_json::json!(u64::MAX);
        assert_eq!(
            decode_mcp_call_receipt(receipt_policy_mismatch),
            Err(ServerError::InvalidCommand)
        );

        let mut invalid_effect_kind = receipt_payload;
        invalid_effect_kind["effect_kind"] = serde_json::json!(99_u64);
        assert_eq!(
            decode_mcp_call_receipt(invalid_effect_kind),
            Err(ServerError::InvalidPayload)
        );
    }

    #[test]
    fn agent_cycle_event_decoder_preserves_typed_route_contract() {
        let route_cases = [
            (AgentCycleEventKind::CycleStart, 1_u64),
            (AgentCycleEventKind::TurnComplete, 2_u64),
            (AgentCycleEventKind::TurnFailed, 3_u64),
            (AgentCycleEventKind::CycleEnd, 4_u64),
        ];
        for (kind, route_value) in route_cases {
            assert_eq!(kind.as_u64(), route_value);
            assert_eq!(AgentCycleEventKind::from_route_u64(route_value), Some(kind));

            let decoded = decode_submit_agent_cycle_event_command(serde_json::json!({
                "kind": route_value,
                "agent_hash": 0xa6e0_7001_u64,
                "cycle": 1_u64,
                "label_hash": 0x1abe_1001_u64,
                "content_hash": 0xc0de_1001_u64,
            }))
            .expect("valid typed route value should decode");

            match decoded {
                Command::SubmitAgentCycleEvent(event) => {
                    assert_eq!(event.kind, kind);
                    assert_eq!(event.kind.as_u64(), route_value);
                    assert!(event.is_contract_valid());
                }
                other => panic!("unexpected command decoded from agent cycle payload: {other:?}"),
            }
        }

        assert_eq!(AgentCycleEventKind::from_route_u64(99), None);
        assert_eq!(
            decode_submit_agent_cycle_event_command(serde_json::json!({
                "kind": 99_u64,
                "agent_hash": 0xa6e0_7001_u64,
                "cycle": 1_u64,
            })),
            Err(ServerError::InvalidPayload)
        );
    }

    #[test]
    fn invalid_replay_transport_error_maps_to_conflict_status() {
        let (status, Json(body)) =
            error_response(ServerError::Transport(CanonError::InvalidReplay));

        assert_eq!(status, StatusCode::CONFLICT);
        assert!(!body.ok);
        assert!(body.error.contains("InvalidReplay"));
    }

    #[test]
    fn invalid_api_transport_error_maps_to_bad_request_status() {
        let (status, Json(body)) =
            error_response(ServerError::Transport(CanonError::InvalidApiCommand));

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(!body.ok);
        assert!(body.error.contains("InvalidApiCommand"));
    }
}
