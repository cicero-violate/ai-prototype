//! MCP dispatch, receipt recording, and stateful tool execution.

use chrono::{DateTime, Utc};
use serde_json::{json, Value};

use super::{apply_patch, canon_read_mailbox, canon_send_agent_message, shell, WorkspaceView};
use crate::api::mcp::{
    dispatch_ai_mcp_plan, mcp_ok, result_with_warning, tool_error, AiMcpDispatchPlan,
};
use crate::api::protocol::Command as KernelCommand;
use crate::{CapabilityRegistry, McpCallReceipt, McpCallRequest};

#[allow(async_fn_in_trait)]
pub trait StatefulMcpHost {
    fn workspace(&self) -> WorkspaceView;

    async fn active_generation(&self) -> u64;

    fn record_session(&self, session_id: String, worker_generation: u64, created_at: DateTime<Utc>);

    async fn submit_kernel_command(&self, command: KernelCommand) -> Result<(), String>;

    async fn run_host_tool(&self, name: &str, args: &Value) -> Option<Value>;
}

pub async fn dispatch_ai_mcp<H: StatefulMcpHost>(
    method: &str,
    id: Value,
    params: Value,
    host: &H,
) -> (Value, Option<String>) {
    let workspace = host.workspace();
    match dispatch_ai_mcp_plan(method, id.clone(), params, Some(&workspace)) {
        AiMcpDispatchPlan::Initialize {
            response,
            session_id,
        } => {
            let worker_generation = host.active_generation().await;
            host.record_session(session_id.clone(), worker_generation, Utc::now());
            (response, Some(session_id))
        }
        AiMcpDispatchPlan::Immediate { response } => (response, None),
        AiMcpDispatchPlan::ToolCall { name, args } => {
            let result = execute_recorded_ai_mcp_tool(&name, args, host).await;
            (mcp_ok(id, result), None)
        }
    }
}

async fn execute_recorded_ai_mcp_tool<H: StatefulMcpHost>(
    name: &str,
    args: Value,
    host: &H,
) -> Value {
    if name == "shell" {
        return run_recorded_ai_mcp_shell(&args, host).await;
    }

    let args_json = serde_json::to_string(&args).unwrap_or_else(|_| "null".to_string());
    let timeout_ms = args
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .unwrap_or(180_000)
        .max(1);
    let max_output_bytes = args
        .get("max_output_bytes")
        .and_then(Value::as_u64)
        .unwrap_or(65_536)
        .max(1);
    let request = McpCallRequest::new(
        CapabilityRegistry::canonical(),
        "ai-native:/ai/mcp",
        name,
        &args_json,
        timeout_ms,
        max_output_bytes,
    );
    if let Err(error) = host
        .submit_kernel_command(KernelCommand::AuthorizeMcpCall(request))
        .await
    {
        return tool_error(format!(
            "MCP tool call denied before execution by ai worker: {error}"
        ));
    }

    let result = call_ai_mcp_tool(name, &args, host).await;
    let response_bytes = serde_json::to_vec(&result).unwrap_or_default();
    let exit_status = if result
        .get("isError")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        1
    } else {
        0
    };
    let receipt = McpCallReceipt::from_response(&request, &response_bytes, exit_status, false);
    if let Err(error) = host
        .submit_kernel_command(KernelCommand::SubmitMcpCallReceipt(receipt))
        .await
    {
        return result_with_warning(
            result,
            format!("MCP receipt recording failed in ai supervisor: {error}"),
        );
    }
    result
}

async fn call_ai_mcp_tool<H: StatefulMcpHost>(name: &str, args: &Value, host: &H) -> Value {
    match name {
        "echo" => json!({
            "content": [{ "type": "text", "text": args.get("text").and_then(Value::as_str).unwrap_or("") }],
            "isError": false
        }),
        "get_current_time" => json!({
            "content": [{ "type": "text", "text": Utc::now().to_rfc3339() }],
            "isError": false
        }),
        "apply_patch" => {
            let workspace = host.workspace();
            apply_patch::run(args, &workspace).await
        }
        "shell" => {
            let workspace = host.workspace();
            shell::run_unrecorded(args, &workspace).await
        }
        "canon_spawn_agent" => host
            .run_host_tool(name, args)
            .await
            .unwrap_or_else(|| tool_error(format!("Unknown host tool: {name}"))),
        "canon_send_agent_message" => {
            let root = host.workspace().root;
            canon_send_agent_message::run(args, &root)
        }
        "canon_read_mailbox" => {
            let root = host.workspace().root;
            canon_read_mailbox::run(args, &root)
        }
        _ => tool_error(format!("Unknown tool: {name}")),
    }
}

async fn run_recorded_ai_mcp_shell<H: StatefulMcpHost>(args: &Value, host: &H) -> Value {
    let workspace = host.workspace();
    let request = match shell::recorded_process_request(args, &workspace) {
        Ok(request) => request,
        Err(error) => return tool_error(error),
    };
    if let Err(error) = host
        .submit_kernel_command(KernelCommand::AuthorizeProcessCall(request))
        .await
    {
        return tool_error(format!(
            "process execution denied before execution by ai worker: {error}"
        ));
    }

    match shell::run_recorded_process(args, &workspace) {
        Ok((receipt, stdout, stderr)) => {
            let result = shell::render_recorded_response(&receipt, &stdout, &stderr);
            if let Err(error) = host
                .submit_kernel_command(KernelCommand::SubmitProcessReceipt(receipt))
                .await
            {
                return result_with_warning(
                    result,
                    format!("process receipt recording failed in ai worker: {error}"),
                );
            }
            result
        }
        Err(error) => tool_error(error),
    }
}
