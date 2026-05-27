//! Action dispatch, receipt recording, and live action execution.

use chrono::Utc;
use serde_json::{json, Map, Value};
use std::fs::{self, OpenOptions};
use std::io::Write;

use crate::api::action::{
    action_ok, dispatch_action_plan, result_with_warning, tool_error, ActionDispatchPlan,
};
use crate::api::protocol::Command as KernelCommand;
use crate::capability::execution::action::host;
pub use crate::capability::execution::action::host::ActionHost;
use crate::capability::execution::action::host::ActionToolOutcome;
use crate::capability::execution::action::landmarks;
use crate::capability::execution::{ActionCallRequest, ActionReceipt};
use crate::runtime::WorkspaceView;
use crate::CapabilityRegistry;

pub async fn dispatch_action_request<H: ActionHost>(
    method: &str,
    id: Value,
    params: Value,
    host: &H,
) -> (Value, Option<String>) {
    let workspace = host.workspace();
    match dispatch_action_plan(method, id.clone(), params, Some(&workspace)) {
        ActionDispatchPlan::Initialize {
            response,
            session_id,
        } => {
            let worker_generation = host.active_generation().await;
            host.record_session(session_id.clone(), worker_generation, Utc::now());
            (response, Some(session_id))
        }
        ActionDispatchPlan::Immediate { response } => (response, None),
        ActionDispatchPlan::ToolCall { name, args } => {
            let result = execute_recorded_action(&name, args, host).await;
            (action_ok(id, result), None)
        }
    }
}

async fn execute_recorded_action<H: ActionHost>(name: &str, args: Value, host: &H) -> Value {
    if landmarks::is_gateway_tool(name) {
        return execute_recorded_gateway_tool(name, args, host).await;
    }
    execute_recorded_native_action(name, args, host).await
}

fn record_action_result_for_workspace(
    workspace: &WorkspaceView,
    name: &str,
    args_json: &str,
    response_json: &str,
    receipt: &ActionReceipt,
) -> Option<String> {
    append_raw_action_result(workspace, name, args_json, response_json, receipt)
        .err()
        .map(|error| format!("Action raw result recording failed: {error}"))
}

const RAW_FIELD_BYTE_LIMIT: usize = 64 * 1024;

fn truncate_raw_field(s: &str) -> &str {
    if s.len() <= RAW_FIELD_BYTE_LIMIT {
        return s;
    }
    // Find a valid UTF-8 boundary at the limit.
    let mut end = RAW_FIELD_BYTE_LIMIT;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

fn gateway_max_output_bytes(args: &Value, default_limit: u64) -> u64 {
    fn nested_limit(value: &Value) -> Option<u64> {
        value
            .get("max_output_bytes")
            .and_then(Value::as_u64)
            .filter(|limit| *limit > 0)
    }

    let mut limit = default_limit.max(1);
    if let Some(nested) = args.get("parameters").and_then(nested_limit) {
        limit = limit.max(nested);
    }
    for key in ["actions", "steps"] {
        if let Some(items) = args.get(key).and_then(Value::as_array) {
            for item in items {
                if let Some(nested) = item.get("parameters").and_then(nested_limit) {
                    limit = limit.max(nested);
                }
            }
        }
    }
    limit
}

fn append_raw_action_result(
    workspace: &WorkspaceView,
    name: &str,
    args_json: &str,
    response_json: &str,
    receipt: &ActionReceipt,
) -> Result<(), String> {
    let path = workspace
        .allowed_boundary
        .join("state")
        .join("actions.ndjson");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create raw result dir: {error}"))?;
    }
    let line = json!({
        "created_at": Utc::now().to_rfc3339(),
        "tool_name": name,
        "request_json": truncate_raw_field(args_json),
        "response_json": truncate_raw_field(response_json),
        "receipt_hash": receipt.receipt_hash,
        "response_hash": receipt.response_hash,
        "response_bytes": receipt.response_bytes,
        "exit_status": receipt.exit_status,
        "timed_out": receipt.timed_out
    });
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| format!("open raw result {}: {error}", path.display()))?;
    writeln!(file, "{line}").map_err(|error| format!("write raw result: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("sync raw result: {error}"))?;
    eprintln!("[canon-ai-action] raw result saved path={}", path.display());
    Ok(())
}

#[cfg(test)]
mod transcript_tests {
    use super::*;
    use std::fs;
    use std::io::{BufRead, BufReader};
    use std::path::PathBuf;

    fn unique_project() -> (PathBuf, tempfile::TempDir) {
        let tmp = tempfile::Builder::new()
            .prefix("canon-mcp-dispatch-test-")
            .tempdir_in({
                let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
                std::fs::create_dir_all(&d).unwrap();
                d.canonicalize().unwrap()
            })
            .unwrap();
        let path = tmp.path().to_path_buf();
        (path, tmp)
    }

    fn apply_patch_target_files(args: &Value) -> Vec<String> {
        let mut files = std::collections::BTreeSet::new();
        if let Some(path) = args.get("path").and_then(Value::as_str) {
            files.insert(path.to_string());
        }
        if let Some(patch) = args.get("patch").and_then(Value::as_str) {
            for line in patch.lines() {
                for prefix in ["*** Update File: ", "*** Add File: ", "*** Delete File: "] {
                    if let Some(path) = line.strip_prefix(prefix) {
                        files.insert(path.trim().to_string());
                    }
                }
            }
        }
        files.into_iter().collect()
    }

    fn structural_edit_target_files(args: &Value) -> Vec<String> {
        fn collect(value: &Value, files: &mut std::collections::BTreeSet<String>) {
            match value {
                Value::Object(map) => {
                    if let Some(path) = map.get("path").and_then(Value::as_str) {
                        files.insert(path.to_string());
                    }
                    for value in map.values() {
                        collect(value, files);
                    }
                }
                Value::Array(values) => {
                    for value in values {
                        collect(value, files);
                    }
                }
                _ => {}
            }
        }

        let mut files = std::collections::BTreeSet::new();
        collect(args, &mut files);
        files.into_iter().collect()
    }

    #[test]
    fn truncate_raw_field_is_noop_for_short_input() {
        let s = "hello";
        assert_eq!(truncate_raw_field(s), s);
    }

    #[test]
    fn truncate_raw_field_caps_at_limit() {
        let s = "x".repeat(RAW_FIELD_BYTE_LIMIT + 1000);
        let t = truncate_raw_field(&s);
        assert_eq!(t.len(), RAW_FIELD_BYTE_LIMIT);
    }

    #[test]
    fn truncate_raw_field_respects_utf8_boundary() {
        // Build a string where the limit falls in the middle of a multi-byte char.
        // '€' is 3 bytes (U+20AC). Place it so it straddles the boundary.
        let prefix = "a".repeat(RAW_FIELD_BYTE_LIMIT - 1);
        let s = format!("{prefix}€extra");
        assert!(s.len() > RAW_FIELD_BYTE_LIMIT);
        let t = truncate_raw_field(&s);
        assert!(t.len() < RAW_FIELD_BYTE_LIMIT);
        assert!(std::str::from_utf8(t.as_bytes()).is_ok());
    }

    #[test]
    fn raw_action_result_truncates_oversized_fields_on_disk() {
        // Regression: response_json was stored verbatim; canon_diagnostics_read would read
        // raw lines back and embed them in its response, causing repeated JSON re-encoding
        // to double all backslashes each cycle — growing to hundreds of MB per record.
        let (project, _tmp) = unique_project();
        fs::create_dir_all(project.join("state")).unwrap();
        let workspace = WorkspaceView::new(project.clone(), project.clone()).unwrap();

        let large_response = "r".repeat(RAW_FIELD_BYTE_LIMIT * 4);
        let large_args = "a".repeat(RAW_FIELD_BYTE_LIMIT * 2);
        let request = ActionCallRequest::new(
            CapabilityRegistry::canonical(),
            "ai-native:/ai/mcp",
            "canon_diagnostics_read",
            &large_args,
            1000,
            65_536,
        );
        let receipt = ActionReceipt::from_response(&request, large_response.as_bytes(), 0, false);

        append_raw_action_result(
            &workspace,
            "canon_diagnostics_read",
            &large_args,
            &large_response,
            &receipt,
        )
        .expect("write should succeed");

        let path = project.join("state").join("actions.ndjson");
        let file = fs::File::open(&path).unwrap();
        let line = BufReader::new(file).lines().next().unwrap().unwrap();
        assert!(
            line.len() <= RAW_FIELD_BYTE_LIMIT * 3,
            "record line must be bounded, got {} bytes",
            line.len()
        );
        let record: serde_json::Value = serde_json::from_str(&line).expect("must be valid JSON");
        let stored_response = record["response_json"].as_str().unwrap_or("");
        let stored_args = record["request_json"].as_str().unwrap_or("");
        assert!(
            stored_response.len() <= RAW_FIELD_BYTE_LIMIT,
            "response_json must be capped"
        );
        assert!(
            stored_args.len() <= RAW_FIELD_BYTE_LIMIT,
            "request_json must be capped"
        );
    }

    #[test]
    fn apply_patch_target_files_extracts_apply_patch_hunks() {
        let args = json!({
            "patch": "*** Begin Patch
*** Update File: ai/src/lib.rs
@@
*** Add File: ai/src/new.rs
+fn new() {}
*** Delete File: ai/src/old.rs
*** End Patch
"
        });
        assert_eq!(
            apply_patch_target_files(&args),
            vec![
                "ai/src/lib.rs".to_string(),
                "ai/src/new.rs".to_string(),
                "ai/src/old.rs".to_string(),
            ]
        );
    }

    #[test]
    fn apply_patch_target_files_keeps_legacy_path_field() {
        let args = json!({ "path": "ai/src/lib.rs" });
        assert_eq!(
            apply_patch_target_files(&args),
            vec!["ai/src/lib.rs".to_string()]
        );
    }

    #[test]
    fn structural_edit_target_files_extracts_op_paths() {
        let args = json!({
            "mode": "apply",
            "ops": [
                {
                    "op": "set_attr",
                    "kind": "function",
                    "at": { "loc": "selector", "path": "ai/src/lib.rs", "selector": "fn run" },
                    "key": "doc",
                    "value": "Run."
                },
                {
                    "op": "move_node",
                    "kind": "function",
                    "from": { "loc": "selector", "path": "ai/src/old.rs", "selector": "fn run" },
                    "to": { "loc": "selector", "path": "ai/src/new.rs", "selector": "mod target" }
                }
            ]
        });
        assert_eq!(
            structural_edit_target_files(&args),
            vec![
                "ai/src/lib.rs".to_string(),
                "ai/src/new.rs".to_string(),
                "ai/src/old.rs".to_string(),
            ]
        );
    }
}

async fn execute_recorded_gateway_tool<H: ActionHost>(name: &str, args: Value, host: &H) -> Value {
    eprintln!("[canon-ai-mcp] gateway tool start name={name}");
    let args_json = serde_json::to_string(&args).unwrap_or_else(|_| "null".to_string());
    let timeout_ms = args
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .unwrap_or(180_000)
        .max(1);
    let max_output_bytes = gateway_max_output_bytes(
        &args,
        args.get("max_output_bytes")
            .and_then(Value::as_u64)
            .unwrap_or(65_536),
    );
    let request = ActionCallRequest::new(
        CapabilityRegistry::canonical(),
        "ai-native:/ai/mcp",
        name,
        &args_json,
        timeout_ms,
        max_output_bytes,
    );
    if let Err(error) = host
        .submit_kernel_command(KernelCommand::AuthorizeActionCall(request))
        .await
    {
        return tool_error(format!(
            "MCP tool call denied before execution by ai worker: {error}"
        ));
    }

    let outcome = call_gateway_tool(name, &args, host).await;
    let exit_status = outcome.exit_status();
    let tool_timed_out = outcome.timed_out();
    let result = outcome.into_value();
    eprintln!(
        "[canon-ai-mcp] gateway tool finish name={} is_error={} timed_out={tool_timed_out}",
        name,
        exit_status != 0,
    );
    let response_bytes = serde_json::to_vec(&result).unwrap_or_default();
    let response_json =
        String::from_utf8(response_bytes.clone()).unwrap_or_else(|_| "null".to_string());
    let receipt =
        ActionReceipt::from_response(&request, &response_bytes, exit_status, tool_timed_out);
    let workspace = host.workspace();
    let transcript_warning =
        record_action_result_for_workspace(&workspace, name, &args_json, &response_json, &receipt);
    if let Err(error) = host
        .submit_kernel_command(KernelCommand::SubmitActionReceipt(receipt))
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

async fn execute_recorded_native_action<H: ActionHost>(name: &str, args: Value, host: &H) -> Value {
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
    let request = ActionCallRequest::new(
        CapabilityRegistry::canonical(),
        "ai-native:/ai/mcp",
        name,
        &args_json,
        timeout_ms,
        max_output_bytes,
    );
    if let Err(error) = host
        .submit_kernel_command(KernelCommand::AuthorizeActionCall(request))
        .await
    {
        return tool_error(format!(
            "MCP tool call denied before execution by ai worker: {error}"
        ));
    }

    let outcome = if name == "shell" {
        host::execute_recorded_shell(&args, host).await
    } else {
        host::execute_native_tool(name, &args, host).await
    };
    let exit_status = outcome.exit_status();
    let timed_out = outcome.timed_out();
    let result = outcome.into_value();
    eprintln!(
        "[canon-ai-mcp] native tool finish name={} is_error={} timed_out={timed_out}",
        name,
        exit_status != 0,
    );
    let response_bytes = serde_json::to_vec(&result).unwrap_or_default();
    let response_json =
        String::from_utf8(response_bytes.clone()).unwrap_or_else(|_| "null".to_string());
    let receipt = ActionReceipt::from_response(&request, &response_bytes, exit_status, timed_out);
    let workspace = host.workspace();
    let transcript_warning =
        record_action_result_for_workspace(&workspace, name, &args_json, &response_json, &receipt);
    if let Err(error) = host
        .submit_kernel_command(KernelCommand::SubmitActionReceipt(receipt))
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

async fn call_gateway_tool<H: ActionHost>(name: &str, args: &Value, host: &H) -> ActionToolOutcome {
    match name {
        landmarks::GATEWAY_GET_MANIFEST => ActionToolOutcome::Ok(landmarks::manifest_result()),
        landmarks::GATEWAY_GET_LANDMARKS => ActionToolOutcome::Ok(landmarks::landmarks_result()),
        landmarks::GATEWAY_INSPECT_LANDMARK => {
            ActionToolOutcome::from_value(landmarks::inspect_landmark_result(args))
        }
        landmarks::GATEWAY_CALL_ACTION => run_gateway_call_action(args, host).await,
        landmarks::GATEWAY_EXECUTE_SEQUENCE => run_gateway_sequence(args, host).await,
        _ => ActionToolOutcome::Error(tool_error(format!("Unknown gateway tool: {name}"))),
    }
}

async fn run_gateway_call_action<H: ActionHost>(args: &Value, host: &H) -> ActionToolOutcome {
    let (action_id, parameters) = match landmarks::parameters_from_call_action(args) {
        Ok(parsed) => parsed,
        Err(error) => return ActionToolOutcome::Error(landmarks::error_result(error)),
    };
    run_gateway_action(action_id, parameters, host).await
}

async fn run_gateway_action<H: ActionHost>(
    action_id: &str,
    parameters: Value,
    host: &H,
) -> ActionToolOutcome {
    eprintln!("[canon-ai-mcp] landmark action dispatch action={action_id}");
    let Some(action) = landmarks::resolve_action_id(action_id) else {
        return ActionToolOutcome::Error(landmarks::error_result(format!(
            "Unknown action '{action_id}'. Use get_landmarks then inspect_landmark before calling actions."
        )));
    };
    ActionToolOutcome::from_value(
        execute_recorded_native_action(action.native_tool.as_str(), parameters, host).await,
    )
}

async fn run_gateway_sequence<H: ActionHost>(args: &Value, host: &H) -> ActionToolOutcome {
    let steps = match landmarks::sequence_steps(args) {
        Ok(steps) => steps,
        Err(error) => return ActionToolOutcome::Error(landmarks::error_result(error)),
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
                if step.on_error.should_stop() {
                    break;
                }
                continue;
            }
        };

        let outcome = run_gateway_action(&step.action, resolved, host).await;
        let is_error = outcome.exit_status() != 0;
        let result = outcome.into_value();
        aliases.insert(format!("step{idx}"), result.clone());
        aliases.insert(alias.clone(), result.clone());
        results.push(json!({
            "step": idx,
            "action": step.action,
            "alias": alias,
            "result": result
        }));
        if is_error && step.on_error.should_stop() {
            break;
        }
    }

    ActionToolOutcome::Ok(landmarks::json_result(Value::Array(results)))
}
