//! HTTP client for the supervisor task lifecycle API.
//!
//! This is an adapter client, not the task ownership authority. It keeps
//! endpoint details out of scheduler policy and worker lifecycle wrappers.

use std::fmt;

use serde::{Deserialize, Deserializer};

use crate::service::agent::loop_driver::http::{get_json_body_local, post_json_body_local};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskClaim {
    pub node_id: String,
    pub claim_id: u64,
    pub expires_at_ms: u64,
    pub receipt_hash: u64,
    pub tlog_submitted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskHeartbeat {
    pub expires_at_ms: u64,
    pub receipt_hash: u64,
    pub tlog_submitted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskLifecycleAck {
    pub receipt_hash: u64,
    pub tlog_submitted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskAssignment {
    pub node_id: String,
    pub title: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskApiError {
    pub status: u16,
    pub kind: TaskApiErrorKind,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskApiErrorKind {
    Retryable,
    Fatal,
    Malformed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskClientError {
    Transport(String),
    Api(TaskApiError),
    Decode(String),
}

impl fmt::Display for TaskClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(err) => write!(f, "task lifecycle transport failed: {err}"),
            Self::Api(err) => write!(
                f,
                "task lifecycle returned HTTP {}: {}",
                err.status, err.message
            ),
            Self::Decode(err) => write!(f, "task lifecycle response decode failed: {err}"),
        }
    }
}

impl std::error::Error for TaskClientError {}

impl From<String> for TaskClientError {
    fn from(value: String) -> Self {
        Self::Transport(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskClient {
    supervisor_url: String,
}

type TaskClientResult<T> = Result<T, TaskClientError>;

impl TaskClient {
    pub fn new(supervisor_url: impl Into<String>) -> Self {
        Self {
            supervisor_url: supervisor_url.into(),
        }
    }

    pub fn next(&self) -> TaskClientResult<Option<TaskAssignment>> {
        let url = format!("{}/v1/task/next", self.supervisor_url);
        decode_next_response(get_json_body_local(&url).map_err(TaskClientError::Transport)?)
    }

    pub fn claim(
        &self,
        node_id: &str,
        worker_id: &str,
        idempotency_key: u64,
        lease_ttl_ms: u64,
    ) -> TaskClientResult<TaskClaim> {
        let url = format!("{}/v1/task/claim", self.supervisor_url);
        let body = serde_json::json!({
            "node_id": node_id,
            "worker_id": worker_id,
            "idempotency_key": idempotency_key,
            "lease_ttl_ms": lease_ttl_ms,
        });
        let (status, body_val) =
            post_json_body_local(&url, &body).map_err(TaskClientError::Transport)?;
        match decode_claim_response(status, body_val)? {
            TaskClaimResponse::Claimed(claim) => Ok(claim),
            TaskClaimResponse::Rejected(error) => Err(TaskClientError::Api(error)),
        }
    }

    pub fn heartbeat(
        &self,
        claim: &TaskClaim,
        worker_id: &str,
        lease_ttl_ms: u64,
    ) -> TaskClientResult<TaskHeartbeat> {
        let url = format!("{}/v1/task/heartbeat", self.supervisor_url);
        let body = serde_json::json!({
            "node_id": claim.node_id,
            "worker_id": worker_id,
            "claim_id": claim.claim_id,
            "lease_ttl_ms": lease_ttl_ms,
        });
        let (status, body_val) =
            post_json_body_local(&url, &body).map_err(TaskClientError::Transport)?;
        match decode_heartbeat_response(status, body_val)? {
            TaskHeartbeatResponse::Renewed(heartbeat) => Ok(heartbeat),
            TaskHeartbeatResponse::Rejected(error) => Err(TaskClientError::Api(error)),
        }
    }

    pub fn complete(
        &self,
        claim: &TaskClaim,
        worker_id: &str,
    ) -> TaskClientResult<TaskLifecycleAck> {
        let url = format!("{}/v1/task/complete", self.supervisor_url);
        let body = serde_json::json!({
            "node_id": claim.node_id,
            "worker_id": worker_id,
            "claim_id": claim.claim_id,
        });
        let (status, body_val) =
            post_json_body_local(&url, &body).map_err(TaskClientError::Transport)?;
        match decode_complete_response(status, body_val)? {
            TaskCompleteResponse::Completed(ack) => Ok(ack),
            TaskCompleteResponse::Rejected(error) => Err(TaskClientError::Api(error)),
        }
    }

    pub fn fail(
        &self,
        claim: &TaskClaim,
        worker_id: &str,
        retry_after_ms: u64,
    ) -> TaskClientResult<TaskLifecycleAck> {
        let url = format!("{}/v1/task/fail", self.supervisor_url);
        let body = serde_json::json!({
            "node_id": claim.node_id,
            "worker_id": worker_id,
            "claim_id": claim.claim_id,
            "retry_after_ms": retry_after_ms,
        });
        let (status, body_val) =
            post_json_body_local(&url, &body).map_err(TaskClientError::Transport)?;
        match decode_fail_response(status, body_val)? {
            TaskFailResponse::Recorded(ack) => Ok(ack),
            TaskFailResponse::Rejected(error) => Err(TaskClientError::Api(error)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TaskClaimResponse {
    Claimed(TaskClaim),
    Rejected(TaskApiError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TaskHeartbeatResponse {
    Renewed(TaskHeartbeat),
    Rejected(TaskApiError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TaskCompleteResponse {
    Completed(TaskLifecycleAck),
    Rejected(TaskApiError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TaskFailResponse {
    Recorded(TaskLifecycleAck),
    Rejected(TaskApiError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TrueResponse;

impl<'de> Deserialize<'de> for TrueResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match bool::deserialize(deserializer)? {
            true => Ok(Self),
            false => Err(serde::de::Error::custom("expected ok=true")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FalseResponse;

impl<'de> Deserialize<'de> for FalseResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match bool::deserialize(deserializer)? {
            true => Err(serde::de::Error::custom("expected ok=false")),
            false => Ok(Self),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct TaskAssignmentSuccessDto {
    #[serde(rename = "ok")]
    _ok: TrueResponse,
    node_id: String,
    title: String,
    description: String,
}

impl From<TaskAssignmentSuccessDto> for TaskAssignment {
    fn from(value: TaskAssignmentSuccessDto) -> Self {
        Self {
            node_id: value.node_id,
            title: value.title,
            description: value.description,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct TaskClaimSuccessDto {
    #[serde(rename = "ok")]
    _ok: TrueResponse,
    node_id: String,
    claim_id: u64,
    expires_at_ms: u64,
    receipt_hash: u64,
    tlog_submitted: bool,
}

impl From<TaskClaimSuccessDto> for TaskClaim {
    fn from(value: TaskClaimSuccessDto) -> Self {
        Self {
            node_id: value.node_id,
            claim_id: value.claim_id,
            expires_at_ms: value.expires_at_ms,
            receipt_hash: value.receipt_hash,
            tlog_submitted: value.tlog_submitted,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct TaskHeartbeatSuccessDto {
    #[serde(rename = "ok")]
    _ok: TrueResponse,
    expires_at_ms: u64,
    receipt_hash: u64,
    tlog_submitted: bool,
}

impl From<TaskHeartbeatSuccessDto> for TaskHeartbeat {
    fn from(value: TaskHeartbeatSuccessDto) -> Self {
        Self {
            expires_at_ms: value.expires_at_ms,
            receipt_hash: value.receipt_hash,
            tlog_submitted: value.tlog_submitted,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct TaskAckSuccessDto {
    #[serde(rename = "ok")]
    _ok: TrueResponse,
    receipt_hash: u64,
    tlog_submitted: bool,
}

impl From<TaskAckSuccessDto> for TaskLifecycleAck {
    fn from(value: TaskAckSuccessDto) -> Self {
        Self {
            receipt_hash: value.receipt_hash,
            tlog_submitted: value.tlog_submitted,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
struct TaskApiErrorDto {
    #[serde(rename = "ok")]
    _ok: FalseResponse,
    error: String,
}

fn decode_next_response(value: serde_json::Value) -> TaskClientResult<Option<TaskAssignment>> {
    if value.is_null() {
        return Ok(None);
    }
    match serde_json::from_value::<TaskAssignmentSuccessDto>(value.clone()) {
        Ok(dto) => Ok(Some(dto.into())),
        Err(success_error) => match serde_json::from_value::<TaskApiErrorDto>(value) {
            Ok(_) => Ok(None),
            Err(error_decode) => Err(TaskClientError::Decode(format!(
                "expected task assignment or no-task error; assignment={success_error}; error={error_decode}"
            ))),
        },
    }
}

fn decode_claim_response(
    status: u16,
    value: serde_json::Value,
) -> TaskClientResult<TaskClaimResponse> {
    decode_task_response(
        status,
        value,
        |dto: TaskClaimSuccessDto| TaskClaimResponse::Claimed(dto.into()),
        TaskClaimResponse::Rejected,
    )
}

fn decode_heartbeat_response(
    status: u16,
    value: serde_json::Value,
) -> TaskClientResult<TaskHeartbeatResponse> {
    decode_task_response(
        status,
        value,
        |dto: TaskHeartbeatSuccessDto| TaskHeartbeatResponse::Renewed(dto.into()),
        TaskHeartbeatResponse::Rejected,
    )
}

fn decode_complete_response(
    status: u16,
    value: serde_json::Value,
) -> TaskClientResult<TaskCompleteResponse> {
    decode_task_response(
        status,
        value,
        |dto: TaskAckSuccessDto| TaskCompleteResponse::Completed(dto.into()),
        TaskCompleteResponse::Rejected,
    )
}

fn decode_fail_response(
    status: u16,
    value: serde_json::Value,
) -> TaskClientResult<TaskFailResponse> {
    decode_task_response(
        status,
        value,
        |dto: TaskAckSuccessDto| TaskFailResponse::Recorded(dto.into()),
        TaskFailResponse::Rejected,
    )
}

fn decode_task_response<T, O, S, E>(
    status: u16,
    value: serde_json::Value,
    success: S,
    rejected: E,
) -> TaskClientResult<O>
where
    T: for<'de> Deserialize<'de>,
    S: FnOnce(T) -> O,
    E: FnOnce(TaskApiError) -> O,
{
    if (200..300).contains(&status) {
        let dto = serde_json::from_value::<T>(value)
            .map_err(|err| TaskClientError::Decode(err.to_string()))?;
        return Ok(success(dto));
    }
    Ok(rejected(decode_api_error(status, value)))
}

fn decode_api_error(status: u16, value: serde_json::Value) -> TaskApiError {
    match serde_json::from_value::<TaskApiErrorDto>(value) {
        Ok(dto) => TaskApiError {
            status,
            kind: classify_api_error(status),
            message: dto.error,
        },
        Err(err) => TaskApiError {
            status,
            kind: TaskApiErrorKind::Malformed,
            message: format!("malformed error response: {err}"),
        },
    }
}

fn classify_api_error(status: u16) -> TaskApiErrorKind {
    match status {
        409 | 423 | 429 | 500..=599 => TaskApiErrorKind::Retryable,
        _ => TaskApiErrorKind::Fatal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn next_response_decodes_ready_assignment_as_typed_outcome() {
        let decoded = decode_next_response(json!({
            "ok": true,
            "node_id": "node-1",
            "title": "Node 1",
            "description": "Do work",
            "ready_count": 3
        }))
        .expect("decode next response")
        .expect("ready task");

        assert_eq!(
            decoded,
            TaskAssignment {
                node_id: "node-1".to_string(),
                title: "Node 1".to_string(),
                description: "Do work".to_string(),
            }
        );
    }

    #[test]
    fn next_response_decodes_no_task_error_as_none() {
        let decoded = decode_next_response(json!({
            "ok": false,
            "error": "no ready tasks"
        }))
        .expect("decode no-task response");

        assert_eq!(decoded, None);
    }

    #[test]
    fn claim_response_requires_typed_success_shape() {
        let claim = match decode_claim_response(
            200,
            json!({
                "ok": true,
                "node_id": "node-1",
                "claim_id": 42,
                "expires_at_ms": 5000,
                "receipt_hash": 99,
                "tlog_submitted": true
            }),
        )
        .expect("typed claim success")
        {
            TaskClaimResponse::Claimed(claim) => claim,
            TaskClaimResponse::Rejected(error) => panic!("unexpected claim rejection: {error:?}"),
        };

        assert_eq!(claim.node_id, "node-1");
        assert_eq!(claim.claim_id, 42);
        assert_eq!(claim.expires_at_ms, 5000);
        assert_eq!(claim.receipt_hash, 99);
        assert!(claim.tlog_submitted);
    }

    #[test]
    fn malformed_success_response_is_decode_error() {
        let err = decode_claim_response(
            200,
            json!({
                "ok": true,
                "node_id": "node-1",
                "claim_id": 42,
                "expires_at_ms": 5000,
                "tlog_submitted": true
            }),
        )
        .expect_err("missing receipt_hash must fail decode");

        assert!(matches!(err, TaskClientError::Decode(_)));
    }

    #[test]
    fn server_error_response_is_typed_api_error() {
        let response = decode_claim_response(
            409,
            json!({
                "ok": false,
                "error": "node x is already claimed"
            }),
        )
        .expect("non-2xx response is typed rejection");

        assert_eq!(
            response,
            TaskClaimResponse::Rejected(TaskApiError {
                status: 409,
                kind: TaskApiErrorKind::Retryable,
                message: "node x is already claimed".to_string(),
            })
        );
    }

    #[test]
    fn heartbeat_and_ack_decode_typed_receipt_fields() {
        let heartbeat = match decode_heartbeat_response(
            200,
            json!({
                "ok": true,
                "expires_at_ms": 6000,
                "receipt_hash": 100,
                "tlog_submitted": false
            }),
        )
        .expect("typed heartbeat")
        {
            TaskHeartbeatResponse::Renewed(heartbeat) => heartbeat,
            TaskHeartbeatResponse::Rejected(error) => {
                panic!("unexpected heartbeat rejection: {error:?}")
            }
        };
        assert_eq!(heartbeat.expires_at_ms, 6000);
        assert_eq!(heartbeat.receipt_hash, 100);
        assert!(!heartbeat.tlog_submitted);

        let complete_ack = match decode_complete_response(
            200,
            json!({
                "ok": true,
                "receipt_hash": 101,
                "tlog_submitted": true
            }),
        )
        .expect("typed complete ack")
        {
            TaskCompleteResponse::Completed(ack) => ack,
            TaskCompleteResponse::Rejected(error) => {
                panic!("unexpected complete rejection: {error:?}")
            }
        };
        assert_eq!(complete_ack.receipt_hash, 101);
        assert!(complete_ack.tlog_submitted);

        let fail_ack = match decode_fail_response(
            200,
            json!({
                "ok": true,
                "receipt_hash": 102,
                "tlog_submitted": false
            }),
        )
        .expect("typed fail ack")
        {
            TaskFailResponse::Recorded(ack) => ack,
            TaskFailResponse::Rejected(error) => panic!("unexpected fail rejection: {error:?}"),
        };
        assert_eq!(fail_ack.receipt_hash, 102);
        assert!(!fail_ack.tlog_submitted);
    }

    #[test]
    fn malformed_error_response_is_typed_malformed_server_error() {
        let response = decode_fail_response(
            500,
            json!({
                "ok": false,
                "message": "missing canonical error field"
            }),
        )
        .expect("non-2xx response remains typed");

        match response {
            TaskFailResponse::Rejected(error) => {
                assert_eq!(error.status, 500);
                assert_eq!(error.kind, TaskApiErrorKind::Malformed);
                assert!(error.message.contains("malformed error response"));
            }
            TaskFailResponse::Recorded(ack) => panic!("unexpected fail ack: {ack:?}"),
        }
    }
}
