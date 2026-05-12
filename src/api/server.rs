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
use crate::capability::{EvidenceSubmission, PacketEffect};
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
pub struct ErrorDto {
    pub ok: bool,
    pub error: String,
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
            Ok(Command::SubmitEvidence(submission))
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
        _ => Err(ServerError::UnsupportedPayloadTag),
    }
}

fn decode_submission(payload: serde_json::Value) -> Result<EvidenceSubmission, ServerError> {
    let dto: EvidenceSubmissionDto =
        serde_json::from_value(payload).map_err(|_| ServerError::InvalidPayload)?;
    submission_from_dto(dto)
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

fn gate_from_str(value: &str) -> Result<GateId, ServerError> {
    match value {
        "Invariant" => Ok(GateId::Invariant),
        "Analysis" => Ok(GateId::Analysis),
        "Judgment" => Ok(GateId::Judgment),
        "Plan" => Ok(GateId::Plan),
        "Execution" => Ok(GateId::Execution),
        "Verification" => Ok(GateId::Verification),
        "Eval" => Ok(GateId::Eval),
        "Learning" => Ok(GateId::Learning),
        _ => Err(ServerError::InvalidPayload),
    }
}

fn evidence_from_str(value: &str) -> Result<Evidence, ServerError> {
    match value {
        "InvariantProof" => Ok(Evidence::InvariantProof),
        "AnalysisReport" => Ok(Evidence::AnalysisReport),
        "JudgmentRecord" => Ok(Evidence::JudgmentRecord),
        "PlanRecord" => Ok(Evidence::PlanRecord),
        "TaskReady" => Ok(Evidence::TaskReady),
        "ExecutionReceipt" => Ok(Evidence::ExecutionReceipt),
        "ArtifactReceipt" => Ok(Evidence::ArtifactReceipt),
        "VerificationReport" => Ok(Evidence::VerificationReport),
        "LineageProof" => Ok(Evidence::LineageProof),
        "EvalScore" => Ok(Evidence::EvalScore),
        "PersistedRecord" => Ok(Evidence::PersistedRecord),
        _ => Err(ServerError::InvalidPayload),
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
