//! HTTP client for the supervisor task lifecycle API.
//!
//! This is an adapter client, not the task ownership authority. It keeps
//! endpoint details out of scheduler policy and worker lifecycle wrappers.

use crate::process::agent::loop_driver::http::{get_json_body_local, post_json_body_local};

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
pub struct TaskClient {
    supervisor_url: String,
}

impl TaskClient {
    pub fn new(supervisor_url: impl Into<String>) -> Self {
        Self {
            supervisor_url: supervisor_url.into(),
        }
    }

    pub fn next(&self) -> Result<Option<TaskAssignment>, String> {
        let url = format!("{}/v1/task/next", self.supervisor_url);
        let value = get_json_body_local(&url)?;
        if value.get("ok").and_then(|v| v.as_bool()) != Some(true) {
            return Ok(None);
        }
        let node_id = value
            .get("node_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if node_id.is_empty() {
            return Ok(None);
        }
        Ok(Some(TaskAssignment {
            node_id,
            title: value
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            description: value
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }))
    }

    pub fn claim(
        &self,
        node_id: &str,
        worker_id: &str,
        idempotency_key: u64,
        lease_ttl_ms: u64,
    ) -> Result<TaskClaim, String> {
        let url = format!("{}/v1/task/claim", self.supervisor_url);
        let body = serde_json::json!({
            "node_id": node_id,
            "worker_id": worker_id,
            "idempotency_key": idempotency_key,
            "lease_ttl_ms": lease_ttl_ms,
        });
        let (status, body_val) = post_json_body_local(&url, &body)?;
        if !(200..300).contains(&status) {
            return Err(task_error(status, &body_val));
        }
        Ok(TaskClaim {
            node_id: node_id.to_string(),
            claim_id: body_val
                .get("claim_id")
                .and_then(|v| v.as_u64())
                .unwrap_or(idempotency_key),
            expires_at_ms: body_val
                .get("expires_at_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            receipt_hash: body_val
                .get("receipt_hash")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            tlog_submitted: body_val
                .get("tlog_submitted")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    }

    pub fn heartbeat(
        &self,
        claim: &TaskClaim,
        worker_id: &str,
        lease_ttl_ms: u64,
    ) -> Result<TaskHeartbeat, String> {
        let url = format!("{}/v1/task/heartbeat", self.supervisor_url);
        let body = serde_json::json!({
            "node_id": claim.node_id,
            "worker_id": worker_id,
            "claim_id": claim.claim_id,
            "lease_ttl_ms": lease_ttl_ms,
        });
        let (status, body_val) = post_json_body_local(&url, &body)?;
        if !(200..300).contains(&status) {
            return Err(task_error(status, &body_val));
        }
        Ok(TaskHeartbeat {
            expires_at_ms: body_val
                .get("expires_at_ms")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            receipt_hash: body_val
                .get("receipt_hash")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            tlog_submitted: body_val
                .get("tlog_submitted")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    }

    pub fn complete(&self, claim: &TaskClaim, worker_id: &str) -> Result<TaskLifecycleAck, String> {
        let url = format!("{}/v1/task/complete", self.supervisor_url);
        let body = serde_json::json!({
            "node_id": claim.node_id,
            "worker_id": worker_id,
            "claim_id": claim.claim_id,
        });
        let (status, body_val) = post_json_body_local(&url, &body)?;
        if !(200..300).contains(&status) {
            return Err(task_error(status, &body_val));
        }
        Ok(TaskLifecycleAck {
            receipt_hash: body_val
                .get("receipt_hash")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            tlog_submitted: body_val
                .get("tlog_submitted")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    }

    pub fn fail(
        &self,
        claim: &TaskClaim,
        worker_id: &str,
        retry_after_ms: u64,
    ) -> Result<TaskLifecycleAck, String> {
        let url = format!("{}/v1/task/fail", self.supervisor_url);
        let body = serde_json::json!({
            "node_id": claim.node_id,
            "worker_id": worker_id,
            "claim_id": claim.claim_id,
            "retry_after_ms": retry_after_ms,
        });
        let (status, body_val) = post_json_body_local(&url, &body)?;
        if !(200..300).contains(&status) {
            return Err(task_error(status, &body_val));
        }
        Ok(TaskLifecycleAck {
            receipt_hash: body_val
                .get("receipt_hash")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            tlog_submitted: body_val
                .get("tlog_submitted")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    }
}

fn task_error(status: u16, body: &serde_json::Value) -> String {
    let err = body
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    format!("task lifecycle returned HTTP {status}: {err}")
}
