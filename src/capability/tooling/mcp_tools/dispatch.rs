//! MCP dispatch, receipt recording, and MCP tool execution.

use chrono::Utc;
use serde_json::{json, Map, Value};

use super::landmarks;
use crate::api::mcp::{
    dispatch_ai_mcp_plan, mcp_ok, result_with_warning, tool_error, AiMcpDispatchPlan,
};
use crate::api::protocol::Command as KernelCommand;
use crate::capability::tooling::native;
pub use crate::capability::tooling::native::NativeToolHost as McpToolHost;
use crate::runtime::append_mcp_transcript;
use crate::{CapabilityRegistry, McpCallReceipt, McpCallRequest};

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
        native::execute_recorded_shell(&args, host).await
    } else {
        native::execute_native_tool(name, &args, host).await
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
    execute_recorded_native_ai_mcp_tool(action.native_tool.as_str(), parameters, host).await
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
