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
use crate::capability::observation::{
    ObservationCursor, ObservationIngressBatch, ObservationRecord,
};
use crate::capability::tooling::{
    Effect, McpCallReceipt, McpCallRequest, SandboxProcessReceipt, ToolEffectKind,
};
use crate::capability::{CapabilityId, CapabilityRegistry, EvidenceSubmission, PacketEffect};
use crate::error::CanonError;
use crate::kernel::{Evidence, GateId, RuntimeConfig, State};
use crate::runtime::CanonicalWriter;

#[derive(Clone)]
pub struct WorkerAppState {
    inner: Arc<Mutex<WorkerSession>>,
}

impl WorkerAppState {
    pub fn new(session: ApiTransportSession, tlog_path: impl Into<PathBuf>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(WorkerSession {
                session,
                tlog_path: tlog_path.into(),
                next_request_id: 1,
            })),
        }
    }

    pub fn new_default(tlog_path: impl Into<PathBuf>) -> Self {
        Self::new(
            ApiTransportSession::new(State::default(), RuntimeConfig::default()),
            tlog_path,
        )
    }

    pub fn snapshot(&self) -> Result<StateDto, ServerError> {
        let guard = self.inner.lock().map_err(|_| ServerError::LockPoisoned)?;
        Ok(state_dto(&guard.session))
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

pub async fn health() -> &'static str {
    "ok"
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
    let request_id = guard.next_request_id;
    guard.next_request_id = guard
        .next_request_id
        .checked_add(1)
        .ok_or(ServerError::InvalidCommand)?;

    let frame = ApiTransportFrame::new(request_id, envelope);
    let response = guard
        .session
        .handle_frame(frame)
        .map_err(ServerError::Transport)?;
    CanonicalWriter::persist_snapshot(&guard.tlog_path, guard.session.tlog())
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

fn decode_command(dto: &CommandEnvelopeDto) -> Result<Command, ServerError> {
    match dto.payload_tag.as_str() {
        "SubmitEvidence" => {
            let submission = decode_submission(dto.payload.clone())?;
            // InvariantProof requires typed Observation path — reject raw submissions.
            if submission.gate == GateId::Invariant {
                return Err(ServerError::InvalidCommand);
            }
            Ok(Command::SubmitEvidence(submission))
        }
        "SubmitObservationIngress" => {
            let batch = decode_observation_ingress(dto.payload.clone())?;
            Ok(Command::SubmitObservationIngress(batch))
        }
        "SubmitEvidenceBatch" => {
            let payloads: Vec<EvidenceSubmissionDto> = serde_json::from_value(dto.payload.clone())
                .map_err(|_| ServerError::InvalidPayload)?;
            let submissions = payloads
                .into_iter()
                .map(submission_from_dto)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Command::SubmitEvidenceBatch(submissions))
        }
        "AuthorizeMcpCall" => {
            let request = decode_mcp_call_request(dto.payload.clone())?;
            Ok(Command::AuthorizeMcpCall(request))
        }
        "SubmitMcpCallReceipt" => {
            let receipt = decode_mcp_call_receipt(dto.payload.clone())?;
            Ok(Command::SubmitMcpCallReceipt(receipt))
        }
        "SubmitProcessReceipt" => {
            let receipt = decode_sandbox_process_receipt(dto.payload.clone())?;
            Ok(Command::SubmitProcessReceipt(receipt))
        }
        "SubmitProcessReceiptBatch" => {
            let payloads: Vec<SandboxProcessReceiptDto> =
                serde_json::from_value(dto.payload.clone())
                    .map_err(|_| ServerError::InvalidPayload)?;
            let receipts = payloads
                .into_iter()
                .map(sandbox_process_receipt_from_dto)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Command::SubmitProcessReceiptBatch(receipts))
        }
        _ => Err(ServerError::UnsupportedPayloadTag),
    }
}

fn decode_mcp_call_request(payload: serde_json::Value) -> Result<McpCallRequest, ServerError> {
    let dto: McpCallRequestDto =
        serde_json::from_value(payload).map_err(|_| ServerError::InvalidPayload)?;
    let request = McpCallRequest {
        capability: CapabilityId::Tooling,
        registry_policy_hash: dto.registry_policy_hash,
        worker_url_hash: dto.worker_url_hash,
        tool_name_hash: dto.tool_name_hash,
        args_hash: dto.args_hash,
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

fn decode_mcp_call_receipt(payload: serde_json::Value) -> Result<McpCallReceipt, ServerError> {
    let dto: McpCallReceiptDto =
        serde_json::from_value(payload).map_err(|_| ServerError::InvalidPayload)?;
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
    if receipt.registry_policy_hash != CapabilityRegistry::canonical().policy_hash()
        || !receipt.is_contract_valid()
    {
        return Err(ServerError::InvalidCommand);
    }
    Ok(receipt)
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

fn gate_from_str(value: &str) -> Result<GateId, ServerError> {
    match api_submission_token_from_str(value)? {
        ApiSubmissionToken::Gate(gate) => Ok(gate),
        ApiSubmissionToken::Evidence(_) => Err(ServerError::InvalidPayload),
    }
}

fn evidence_from_str(value: &str) -> Result<Evidence, ServerError> {
    match api_submission_token_from_str(value)? {
        ApiSubmissionToken::Evidence(evidence) => Ok(evidence),
        ApiSubmissionToken::Gate(_) => Err(ServerError::InvalidPayload),
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
