//! MCP worker command proxying and receipt payload serialization.

use serde_json::{json, Value};

use crate::api::protocol::{Command as KernelCommand, CommandEnvelope};
use crate::capability::tooling::{SandboxProcessReceipt, SandboxProcessRequest};

pub fn kernel_command_payload_tag(command: &KernelCommand) -> &'static str {
    match command {
        KernelCommand::AuthorizeMcpCall(_) => "AuthorizeMcpCall",
        KernelCommand::AuthorizeProcessCall(_) => "AuthorizeProcessCall",
        KernelCommand::SubmitMcpCallReceipt(_) => "SubmitMcpCallReceipt",
        KernelCommand::SubmitProcessReceipt(_) => "SubmitProcessReceipt",
        _ => "Unsupported",
    }
}

pub fn kernel_command_payload(command: &KernelCommand) -> Result<Value, String> {
    match command {
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
        _ => Err("unsupported ai mcp kernel command".to_string()),
    }
}

pub async fn submit_mcp_kernel_command(
    command_id: u64,
    worker_port: u16,
    command: KernelCommand,
) -> Result<(), String> {
    let payload = kernel_command_payload(&command)?;
    let payload_tag = kernel_command_payload_tag(&command);
    let envelope = CommandEnvelope::new(command_id, command);
    let body = json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": payload_tag,
        "payload": payload,
        "source": "ai-mcp"
    });
    let response = reqwest::Client::new()
        .post(format!("http://127.0.0.1:{worker_port}/v1/command"))
        .header("content-type", "application/json")
        .body(body.to_string())
        .send()
        .await
        .map_err(|err| format!("worker command proxy failed: {err}"))?;
    let status = response.status();
    let bytes = response
        .bytes()
        .await
        .map_err(|err| format!("worker command body read failed: {err}"))?;
    if status.is_success() {
        Ok(())
    } else {
        Err(format!(
            "worker returned HTTP {}: {}",
            status.as_u16(),
            String::from_utf8_lossy(&bytes)
        ))
    }
}

fn sandbox_process_request_payload(request: &SandboxProcessRequest) -> Value {
    json!({
        "registry_policy_hash": request.registry_policy_hash,
        "command_hash": request.command_hash,
        "argv_hash": request.argv_hash,
        "cwd_hash": request.cwd_hash,
        "env_hash": request.env_hash,
        "timeout_ms": request.timeout_ms,
        "max_output_bytes": request.max_output_bytes
    })
}

fn sandbox_process_receipt_payload(receipt: &SandboxProcessReceipt) -> Value {
    json!({
        "request_hash": receipt.request_hash,
        "registry_policy_hash": receipt.registry_policy_hash,
        "command_hash": receipt.command_hash,
        "argv_hash": receipt.argv_hash,
        "cwd_hash": receipt.cwd_hash,
        "env_hash": receipt.env_hash,
        "timeout_ms": receipt.timeout_ms,
        "max_output_bytes": receipt.max_output_bytes,
        "effect_kind": receipt.effect.kind as u64,
        "effect_digest": receipt.effect.digest,
        "effect_metadata": receipt.effect.metadata,
        "stdout_hash": receipt.stdout_hash,
        "stderr_hash": receipt.stderr_hash,
        "stdout_bytes": receipt.stdout_bytes,
        "stderr_bytes": receipt.stderr_bytes,
        "exit_status": receipt.exit_status,
        "timed_out": receipt.timed_out,
        "receipt_hash": receipt.receipt_hash
    })
}
