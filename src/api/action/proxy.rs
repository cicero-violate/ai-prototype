//! Action command proxying and receipt payload serialization.

use serde::Serialize;
use serde_json::{json, Value};

use crate::api::protocol::{Command as KernelCommand, CommandEnvelope};
use crate::capability::execution::{SandboxProcessReceipt, SandboxProcessRequest};
use crate::runtime::{MailboxMessageReceipt, MailboxMessageRequest};

pub fn action_kernel_command_payload_tag(command: &KernelCommand) -> &'static str {
    match command {
        KernelCommand::AuthorizeActionCall(_) => "AuthorizeActionCall",
        KernelCommand::AuthorizeMcpCall(_) => "AuthorizeMcpCall",
        KernelCommand::AuthorizeProcessCall(_) => "AuthorizeProcessCall",
        KernelCommand::AuthorizeMailboxMessage(_) => "AuthorizeMailboxMessage",
        KernelCommand::SubmitActionReceipt(_) => "SubmitActionReceipt",
        KernelCommand::SubmitMcpCallReceipt(_) => "SubmitMcpCallReceipt",
        KernelCommand::SubmitProcessReceipt(_) => "SubmitProcessReceipt",
        KernelCommand::SubmitMailboxMessageReceipt(_) => "SubmitMailboxMessageReceipt",
        KernelCommand::SubmitEvidence(_)
        | KernelCommand::SubmitEvidenceBatch(_)
        | KernelCommand::SubmitObservationIngress(_)
        | KernelCommand::SubmitProcessReceiptBatch(_)
        | KernelCommand::SubmitAgentCycleEvent(_)
        | KernelCommand::SubmitWaveDispatch(_)
        | KernelCommand::SubmitChildComplete(_)
        | KernelCommand::SubmitPlanPatch(_) => "Unsupported",
    }
}

// Legacy MCP name alias.
pub use self::action_kernel_command_payload_tag as kernel_command_payload_tag;

pub fn action_kernel_command_payload(command: &KernelCommand) -> Result<Value, String> {
    match command {
        KernelCommand::AuthorizeActionCall(request) => Ok(json!({
            "registry_policy_hash": request.registry_policy_hash,
            "worker_url_hash": request.worker_url_hash,
            "tool_name_hash": request.tool_name_hash,
            "args_hash": request.args_hash,
            "timeout_ms": request.timeout_ms,
            "max_output_bytes": request.max_output_bytes
        })),
        KernelCommand::AuthorizeMcpCall(request) => Ok(json!({
            "registry_policy_hash": request.registry_policy_hash,
            "worker_url_hash": request.worker_url_hash,
            "tool_name_hash": request.tool_name_hash,
            "args_hash": request.args_hash,
            "timeout_ms": request.timeout_ms,
            "max_output_bytes": request.max_output_bytes
        })),
        KernelCommand::AuthorizeProcessCall(request) => {
            Ok(sandbox_process_request_payload(request))
        }
        KernelCommand::AuthorizeMailboxMessage(request) => {
            Ok(mailbox_message_request_payload(request))
        }
        KernelCommand::SubmitActionReceipt(receipt) => Ok(json!({
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
            "receipt_hash": receipt.receipt_hash
        })),
        KernelCommand::SubmitMcpCallReceipt(receipt) => Ok(json!({
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
            "receipt_hash": receipt.receipt_hash
        })),
        KernelCommand::SubmitProcessReceipt(receipt) => {
            Ok(sandbox_process_receipt_payload(receipt))
        }
        KernelCommand::SubmitMailboxMessageReceipt(receipt) => {
            Ok(mailbox_message_receipt_payload(receipt))
        }
        KernelCommand::SubmitEvidence(_)
        | KernelCommand::SubmitEvidenceBatch(_)
        | KernelCommand::SubmitObservationIngress(_)
        | KernelCommand::SubmitProcessReceiptBatch(_)
        | KernelCommand::SubmitAgentCycleEvent(_)
        | KernelCommand::SubmitWaveDispatch(_)
        | KernelCommand::SubmitChildComplete(_)
        | KernelCommand::SubmitPlanPatch(_) => Err("unsupported action kernel command".to_string()),
    }
}

// Legacy MCP name alias.
pub use self::action_kernel_command_payload as kernel_command_payload;

pub async fn submit_action_kernel_command(
    command_id: u64,
    worker_port: u16,
    command: KernelCommand,
) -> Result<(), String> {
    let payload = action_kernel_command_payload(&command)?;
    let payload_tag = action_kernel_command_payload_tag(&command);
    let envelope = CommandEnvelope::new(command_id, command);
    let body = json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": payload_tag,
        "payload": payload,
        "source": "ai-action"
    });
    let body_str = body.to_string();
    let url = format!("http://127.0.0.1:{worker_port}/v1/command");
    let mut last_err = String::new();
    for attempt in 0u32..3 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(300 * u64::from(attempt))).await;
        }
        let result = reqwest::Client::new()
            .post(&url)
            .header("content-type", "application/json")
            .body(body_str.clone())
            .send()
            .await;
        match result {
            Err(err) => {
                last_err = format!("worker command proxy failed: {err}");
                continue;
            }
            Ok(response) => {
                let status = response.status();
                let bytes = response
                    .bytes()
                    .await
                    .map_err(|err| format!("worker command body read failed: {err}"))?;
                if status.is_success() {
                    return Ok(());
                }
                last_err = format!(
                    "worker returned HTTP {}: {}",
                    status.as_u16(),
                    String::from_utf8_lossy(&bytes)
                );
                if status.as_u16() < 500 {
                    return Err(last_err);
                }
            }
        }
    }
    Err(last_err)
}

// Legacy MCP name alias.
pub use self::submit_action_kernel_command as submit_mcp_kernel_command;

fn sandbox_process_request_payload(request: &SandboxProcessRequest) -> Value {
    sandbox_process_base_payload(
        request.registry_policy_hash,
        request.command_hash,
        request.argv_hash,
        request.cwd_hash,
        request.env_hash,
        request.timeout_ms,
        request.max_output_bytes,
    )
}

fn sandbox_process_receipt_payload(receipt: &SandboxProcessReceipt) -> Value {
    let mut payload = sandbox_process_base_payload(
        receipt.registry_policy_hash,
        receipt.command_hash,
        receipt.argv_hash,
        receipt.cwd_hash,
        receipt.env_hash,
        receipt.timeout_ms,
        receipt.max_output_bytes,
    );
    payload["request_hash"] = json!(receipt.request_hash);
    payload["effect_kind"] = json!(receipt.effect.kind as u64);
    payload["effect_digest"] = json!(receipt.effect.digest);
    payload["effect_metadata"] = json!(receipt.effect.metadata);
    payload["stdout_hash"] = json!(receipt.stdout_hash);
    payload["stderr_hash"] = json!(receipt.stderr_hash);
    payload["stdout_bytes"] = json!(receipt.stdout_bytes);
    payload["stderr_bytes"] = json!(receipt.stderr_bytes);
    payload["exit_status"] = json!(receipt.exit_status);
    payload["timed_out"] = json!(receipt.timed_out);
    payload["receipt_hash"] = json!(receipt.receipt_hash);
    payload
}

fn sandbox_process_base_payload(
    registry_policy_hash: impl Serialize,
    command_hash: impl Serialize,
    argv_hash: impl Serialize,
    cwd_hash: impl Serialize,
    env_hash: impl Serialize,
    timeout_ms: impl Serialize,
    max_output_bytes: impl Serialize,
) -> Value {
    json!({
        "registry_policy_hash": registry_policy_hash,
        "command_hash": command_hash,
        "argv_hash": argv_hash,
        "cwd_hash": cwd_hash,
        "env_hash": env_hash,
        "timeout_ms": timeout_ms,
        "max_output_bytes": max_output_bytes
    })
}

fn mailbox_message_request_payload(request: &MailboxMessageRequest) -> Value {
    mailbox_message_base_payload(
        request.registry_policy_hash,
        request.sender_hash,
        request.target_hash,
        request.kind_hash,
        request.payload_hash,
    )
}

fn mailbox_message_receipt_payload(receipt: &MailboxMessageReceipt) -> Value {
    let mut payload = mailbox_message_base_payload(
        receipt.registry_policy_hash,
        receipt.sender_hash,
        receipt.target_hash,
        receipt.kind_hash,
        receipt.payload_hash,
    );
    payload["request_hash"] = json!(receipt.request_hash);
    payload["message_id_hash"] = json!(receipt.message_id_hash);
    payload["sent_at_hash"] = json!(receipt.sent_at_hash);
    payload["record_hash"] = json!(receipt.record_hash);
    payload["receipt_hash"] = json!(receipt.receipt_hash);
    payload
}

fn mailbox_message_base_payload(
    registry_policy_hash: impl Serialize,
    sender_hash: impl Serialize,
    target_hash: impl Serialize,
    kind_hash: impl Serialize,
    payload_hash: impl Serialize,
) -> Value {
    json!({
        "registry_policy_hash": registry_policy_hash,
        "sender_hash": sender_hash,
        "target_hash": target_hash,
        "kind_hash": kind_hash,
        "payload_hash": payload_hash
    })
}
