//! MCP dispatch, receipt recording, and MCP tool execution.

use chrono::{DateTime, Utc};
use serde_json::{json, Map, Value};

use super::{
    apply_patch, canon_graph_editor, canon_read_mailbox, canon_send_agent_message, landmarks, shell,
};
use crate::api::mcp::{
    dispatch_ai_mcp_plan, mcp_ok, result_with_warning, tool_error, AiMcpDispatchPlan,
};
use crate::api::protocol::Command as KernelCommand;
use crate::runtime::{append_mcp_transcript, WorkspaceView};
use crate::{CapabilityRegistry, McpCallReceipt, McpCallRequest};

#[allow(async_fn_in_trait)]
pub trait McpToolHost {
    fn workspace(&self) -> WorkspaceView;

    async fn active_generation(&self) -> u64;

    fn record_session(&self, session_id: String, worker_generation: u64, created_at: DateTime<Utc>);

    async fn submit_kernel_command(&self, command: KernelCommand) -> Result<(), String>;

    async fn run_host_tool(&self, name: &str, args: &Value) -> Option<Value>;
}

pub async fn dispatch_ai_mcp<H: McpToolHost>(
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

async fn execute_recorded_ai_mcp_tool<H: McpToolHost>(name: &str, args: Value, host: &H) -> Value {
    if landmarks::is_gateway_tool(name) {
        return execute_recorded_gateway_tool(name, args, host).await;
    }
    execute_recorded_native_ai_mcp_tool(name, args, host).await
}

async fn execute_recorded_gateway_tool<H: McpToolHost>(name: &str, args: Value, host: &H) -> Value {
    eprintln!("[canon-ai-mcp] gateway tool start name={name}");
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

    let result = call_gateway_tool(name, &args, host).await;
    eprintln!(
        "[canon-ai-mcp] gateway tool finish name={} is_error={}",
        name,
        result
            .get("isError")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    );
    let response_bytes = serde_json::to_vec(&result).unwrap_or_default();
    let response_json =
        String::from_utf8(response_bytes.clone()).unwrap_or_else(|_| "null".to_string());
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
    let workspace = host.workspace();
    let transcript_warning = match append_mcp_transcript(
        &workspace.root,
        name,
        &args_json,
        &response_json,
        &request,
        &receipt,
    ) {
        Ok(_) => None,
        Err(error) => Some(format!(
            "MCP transcript recording failed in runtime: {error}"
        )),
    };
    if let Err(error) = host
        .submit_kernel_command(KernelCommand::SubmitMcpCallReceipt(receipt))
        .await
    {
        return result_with_warning(
            result,
            format!("MCP receipt recording failed in ai supervisor: {error}"),
        );
    }
    match transcript_warning {
        Some(warning) => result_with_warning(result, warning),
        None => result,
    }
}

async fn execute_recorded_native_ai_mcp_tool<H: McpToolHost>(
    name: &str,
    args: Value,
    host: &H,
) -> Value {
    eprintln!("[canon-ai-mcp] native tool start name={name}");
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

    let result = if name == "shell" {
        run_recorded_ai_mcp_shell(&args, host).await
    } else {
        call_ai_mcp_tool(name, &args, host).await
    };
    eprintln!(
        "[canon-ai-mcp] native tool finish name={} is_error={}",
        name,
        result
            .get("isError")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    );
    let response_bytes = serde_json::to_vec(&result).unwrap_or_default();
    let response_json =
        String::from_utf8(response_bytes.clone()).unwrap_or_else(|_| "null".to_string());
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
    let workspace = host.workspace();
    let transcript_warning = match append_mcp_transcript(
        &workspace.root,
        name,
        &args_json,
        &response_json,
        &request,
        &receipt,
    ) {
        Ok(_) => None,
        Err(error) => Some(format!(
            "MCP transcript recording failed in runtime: {error}"
        )),
    };
    if let Err(error) = host
        .submit_kernel_command(KernelCommand::SubmitMcpCallReceipt(receipt))
        .await
    {
        return result_with_warning(
            result,
            format!("MCP receipt recording failed in ai supervisor: {error}"),
        );
    }
    match transcript_warning {
        Some(warning) => result_with_warning(result, warning),
        None => result,
    }
}

async fn call_gateway_tool<H: McpToolHost>(name: &str, args: &Value, host: &H) -> Value {
    match name {
        landmarks::GATEWAY_GET_MANIFEST => landmarks::manifest_result(),
        landmarks::GATEWAY_GET_LANDMARKS => landmarks::landmarks_result(),
        landmarks::GATEWAY_INSPECT_LANDMARK => landmarks::inspect_landmark_result(args),
        landmarks::GATEWAY_CALL_ACTION => run_gateway_call_action(args, host).await,
        landmarks::GATEWAY_EXECUTE_SEQUENCE => run_gateway_sequence(args, host).await,
        _ => tool_error(format!("Unknown gateway tool: {name}")),
    }
}

async fn run_gateway_call_action<H: McpToolHost>(args: &Value, host: &H) -> Value {
    let (action_id, parameters) = match landmarks::parameters_from_call_action(args) {
        Ok(parsed) => parsed,
        Err(error) => return landmarks::error_result(error),
    };
    run_gateway_action(action_id, parameters, host).await
}

async fn run_gateway_action<H: McpToolHost>(action_id: &str, parameters: Value, host: &H) -> Value {
    eprintln!("[canon-ai-mcp] landmark action dispatch action={action_id}");
    let Some(action) = landmarks::resolve_action_id(action_id) else {
        return landmarks::error_result(format!(
            "Unknown action '{action_id}'. Use get_landmarks then inspect_landmark before calling actions."
        ));
    };
    execute_recorded_native_ai_mcp_tool(action.native_tool, parameters, host).await
}

async fn run_gateway_sequence<H: McpToolHost>(args: &Value, host: &H) -> Value {
    let steps = match landmarks::sequence_steps(args) {
        Ok(steps) => steps,
        Err(error) => return landmarks::error_result(error),
    };

    let mut aliases = Map::new();
    let mut results = Vec::new();

    for (idx, step) in steps.into_iter().enumerate() {
        let alias = step.alias.clone().unwrap_or_else(|| format!("step{idx}"));
        let resolved = match landmarks::resolve_piping(step.parameters, &aliases) {
            Ok(value) => value,
            Err(error) => {
                let result = json!({
                    "status": "error",
                    "_PROTOCOL_ERROR": "PIPING_FAILED",
                    "message": error,
                    "remedy": "Check that the alias exists and use dot paths such as $step0.content.0.text."
                });
                aliases.insert(format!("step{idx}"), result.clone());
                aliases.insert(alias.clone(), result.clone());
                results.push(json!({
                    "step": idx,
                    "action": step.action,
                    "alias": alias,
                    "result": result
                }));
                if step.on_error == "stop" {
                    break;
                }
                continue;
            }
        };

        let result = run_gateway_action(&step.action, resolved, host).await;
        aliases.insert(format!("step{idx}"), result.clone());
        aliases.insert(alias.clone(), result.clone());
        let is_error = landmarks::result_is_error(&result);
        results.push(json!({
            "step": idx,
            "action": step.action,
            "alias": alias,
            "result": result
        }));
        if is_error && step.on_error == "stop" {
            break;
        }
    }

    landmarks::json_result(Value::Array(results))
}

async fn call_ai_mcp_tool<H: McpToolHost>(name: &str, args: &Value, host: &H) -> Value {
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
        "canon_graph_plan_patch" => {
            let workspace = host.workspace();
            canon_graph_editor::run_plan_patch(args, &workspace)
        }
        "canon_graph_plan_cfg" => {
            let workspace = host.workspace();
            canon_graph_editor::run_plan_cfg(args, &workspace)
        }
        "canon_graph_apply_ops" => {
            let workspace = host.workspace();
            canon_graph_editor::run_apply_ops(args, &workspace)
        }
        "canon_graph_verify_cfg_delta" => {
            let workspace = host.workspace();
            canon_graph_editor::run_verify_cfg_delta(args, &workspace)
        }
        "canon_graph_auto_refactor_cfg" => {
            let workspace = host.workspace();
            canon_graph_editor::run_auto_refactor_cfg(args, &workspace)
        }
        "shell" => {
            let workspace = host.workspace();
            shell::run_unrecorded(args, &workspace).await
        }
        "canon_spawn_agent"
        | "canon_runtime_state"
        | "canon_supervisor_health"
        | "canon_supervisor_reload_worker"
        | "canon_supervisor_restart"
        | "canon_workspace_get"
        | "canon_workspace_set"
        | "canon_browser_list_tabs"
        | "canon_browser_close_tab"
        | "canon_browser_upload"
        | "canon_browser_group_chat" => host
            .run_host_tool(name, args)
            .await
            .unwrap_or_else(|| tool_error(format!("Unknown host tool: {name}"))),
        "canon_send_agent_message" => run_authorized_mailbox_send(args, host).await,
        "canon_read_mailbox" => {
            let root = host.workspace().root;
            canon_read_mailbox::run(args, &root)
        }
        _ => tool_error(format!("Unknown tool: {name}")),
    }
}

async fn run_authorized_mailbox_send<H: McpToolHost>(args: &Value, host: &H) -> Value {
    let parsed = match canon_send_agent_message::parse_args(args) {
        Ok(parsed) => parsed,
        Err(error) => return tool_error(error),
    };
    if let Err(error) = host
        .submit_kernel_command(KernelCommand::AuthorizeMailboxMessage(parsed.request))
        .await
    {
        return tool_error(format!(
            "mailbox message denied before append by ai worker: {error}"
        ));
    }

    let root = host.workspace().root;
    match canon_send_agent_message::append_authorized(&parsed, &root) {
        Ok((receipt, result)) => {
            if let Err(error) = host
                .submit_kernel_command(KernelCommand::SubmitMailboxMessageReceipt(receipt))
                .await
            {
                return result_with_warning(
                    result,
                    format!("mailbox receipt recording failed in ai worker: {error}"),
                );
            }
            result
        }
        Err(error) => tool_error(error),
    }
}

async fn run_recorded_ai_mcp_shell<H: McpToolHost>(args: &Value, host: &H) -> Value {
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
