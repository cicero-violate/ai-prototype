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
use crate::capability::execution::action::landmarks;
use crate::capability::execution::{ActionCallRequest, ActionReceipt};
use crate::capability::learning::artifact::SymbolMutationRecord;
use crate::runtime::learning_transcript::{append_symbol_mutation, resolve_def_paths_for_file};
use crate::runtime::{append_action_transcript, ActionTranscriptReceiptFacts, WorkspaceView};
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

fn append_action_transcript_for_workspace(
    workspace: &WorkspaceView,
    name: &str,
    args_json: &str,
    response_json: &str,
    request: &ActionCallRequest,
    receipt: &ActionReceipt,
) -> Option<String> {
    let mut errors = Vec::new();
    let mut recorded = false;
    let receipt_facts = ActionTranscriptReceiptFacts {
        request_hash: request.contract_hash(),
        receipt_hash: receipt.receipt_hash,
        response_hash: receipt.response_hash,
        response_bytes: receipt.response_bytes,
        exit_status: receipt.exit_status,
        timed_out: receipt.timed_out,
    };
    append_raw_action_result(workspace, name, args_json, response_json, receipt)
        .unwrap_or_else(|error| eprintln!("[canon-ai-action] raw result save failed: {error}"));

    match append_action_transcript(
        &workspace.root,
        name,
        args_json,
        response_json,
        receipt_facts,
    ) {
        Ok(_) => recorded = true,
        Err(error) => errors.push(format!("workspace {}: {error}", workspace.root.display())),
    }

    if workspace.allowed_boundary != workspace.root {
        match append_action_transcript(
            &workspace.allowed_boundary,
            name,
            args_json,
            response_json,
            receipt_facts,
        ) {
            Ok(_) => recorded = true,
            Err(error) => errors.push(format!(
                "project {}: {error}",
                workspace.allowed_boundary.display()
            )),
        }
    }

    try_record_symbol_mutation(workspace, name, args_json, receipt);

    if errors.is_empty() {
        None
    } else if recorded {
        Some(format!(
            "Action transcript mirror warning: {}",
            errors.join("; ")
        ))
    } else {
        Some(format!(
            "Action transcript recording failed in runtime: {}",
            errors.join("; ")
        ))
    }
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
        "request_json": args_json,
        "response_json": response_json,
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
    use crate::runtime::{action_transcript_path, replay_action_transcripts};
    use std::fs;
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

    #[test]
    fn action_transcript_is_mirrored_from_nested_workspace_to_project_state() {
        let (project, _tmp) = unique_project();
        let nested = project.join("browser-router");
        fs::create_dir_all(&nested).expect("create nested workspace");
        fs::create_dir_all(project.join("state")).expect("create shared state");
        let workspace = WorkspaceView::new(nested.clone(), project.clone()).expect("workspace");
        let args_json = r#"{"command":"get_landmarks"}"#;
        let response_json = r#"{"content":[{"type":"text","text":"saved"}],"isError":false}"#;
        let request = ActionCallRequest::new(
            CapabilityRegistry::canonical(),
            "ai-native:/ai/mcp",
            "get_landmarks",
            args_json,
            1000,
            65_536,
        );
        let receipt = ActionReceipt::from_response(&request, response_json.as_bytes(), 0, false);

        let warning = append_action_transcript_for_workspace(
            &workspace,
            "get_landmarks",
            args_json,
            response_json,
            &request,
            &receipt,
        );

        assert!(warning.is_none());
        let nested_records = replay_action_transcripts(&nested).expect("nested transcript");
        let project_records = replay_action_transcripts(&project).expect("project transcript");
        assert_eq!(nested_records.len(), 1);
        assert_eq!(project_records.len(), 1);
        assert_eq!(project_records[0].response_json, response_json);
        assert!(
            project.join("state").join("actions.ndjson").exists(),
            "raw result sink is written under the project state root"
        );
        assert_eq!(
            action_transcript_path(&project),
            project
                .join("state")
                .join("agent_state")
                .join("action")
                .join("action-transcript.tlog.ndjson")
        );
    }

    #[test]
    fn action_transcript_is_not_duplicated_when_workspace_is_project_root() {
        let (project, _tmp) = unique_project();
        fs::create_dir_all(&project).expect("create project");
        let workspace = WorkspaceView::new(project.clone(), project.clone()).expect("workspace");
        let args_json = r#"{"command":"get_manifest"}"#;
        let response_json = r#"{"content":[{"type":"text","text":"manifest"}],"isError":false}"#;
        let request = ActionCallRequest::new(
            CapabilityRegistry::canonical(),
            "ai-native:/ai/mcp",
            "get_manifest",
            args_json,
            1000,
            65_536,
        );
        let receipt = ActionReceipt::from_response(&request, response_json.as_bytes(), 0, false);

        let warning = append_action_transcript_for_workspace(
            &workspace,
            "get_manifest",
            args_json,
            response_json,
            &request,
            &receipt,
        );

        assert!(warning.is_none());
        let project_records = replay_action_transcripts(&project).expect("project transcript");
        assert_eq!(project_records.len(), 1);
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
    let receipt = ActionReceipt::from_response(&request, &response_bytes, exit_status, false);
    let workspace = host.workspace();
    let transcript_warning = append_action_transcript_for_workspace(
        &workspace,
        name,
        &args_json,
        &response_json,
        &request,
        &receipt,
    );
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

    let result = if name == "shell" {
        host::execute_recorded_shell(&args, host).await
    } else {
        host::execute_native_tool(name, &args, host).await
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
    let receipt = ActionReceipt::from_response(&request, &response_bytes, exit_status, false);
    let workspace = host.workspace();
    let transcript_warning = append_action_transcript_for_workspace(
        &workspace,
        name,
        &args_json,
        &response_json,
        &request,
        &receipt,
    );
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

/// Emit a `SymbolMutationRecord` when a file edit succeeds on a file that has
/// known symbols in the semantic index. Errors here are non-fatal.
///
/// `cycle` and `task_node_hash` are unknown at dispatch time; `derive_training.py`
/// correlates them from `cycle-events.ndjson` by timestamp.
fn try_record_symbol_mutation(
    workspace: &WorkspaceView,
    tool_name: &str,
    args_json: &str,
    receipt: &ActionReceipt,
) {
    if !matches!(tool_name, "apply_patch" | "structural_edit") || receipt.exit_status != 0 {
        return;
    }
    let Ok(args) = serde_json::from_str::<Value>(args_json) else {
        return;
    };
    let file_paths = match tool_name {
        "apply_patch" => apply_patch_target_files(&args),
        "structural_edit" => structural_edit_target_files(&args),
        _ => Vec::new(),
    };
    if file_paths.is_empty() {
        return;
    }
    let ts = chrono::Utc::now().timestamp_millis() as u64;
    for file_path in file_paths {
        let def_paths = resolve_def_paths_for_file(&workspace.allowed_boundary, &file_path);
        if def_paths.is_empty() {
            continue;
        }
        let record = SymbolMutationRecord::new(
            ts,
            0,
            0,
            0,
            file_path.clone(),
            def_paths,
            true,
            receipt.exit_status,
            receipt.receipt_hash,
        );
        if let Err(e) = append_symbol_mutation(&workspace.allowed_boundary, &record, 0) {
            eprintln!("[canon-ai-learning] symbol mutation record failed path={file_path}: {e}");
        }
    }
}

fn apply_patch_target_files(args: &Value) -> Vec<String> {
    if let Some(path) = args.get("path").and_then(|v| v.as_str()) {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return vec![trimmed.to_string()];
        }
    }
    let Some(patch) = args.get("patch").and_then(|v| v.as_str()) else {
        return Vec::new();
    };
    let mut files = Vec::new();
    for line in patch.lines().map(str::trim_end) {
        let path = line
            .strip_prefix("*** Add File: ")
            .or_else(|| line.strip_prefix("*** Update File: "))
            .or_else(|| line.strip_prefix("*** Delete File: "))
            .or_else(|| line.strip_prefix("*** Move to: "));
        if let Some(path) = path.map(str::trim).filter(|path| !path.is_empty()) {
            files.push(path.to_string());
        }
    }
    files.sort();
    files.dedup();
    files
}

fn structural_edit_target_files(args: &Value) -> Vec<String> {
    let mut files = Vec::new();
    if let Some(ops) = args.get("ops").and_then(Value::as_array) {
        for op in ops {
            collect_structural_paths(op, &mut files);
        }
    }
    files.sort();
    files.dedup();
    files
}

fn collect_structural_paths(value: &Value, files: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(path) = map
                .get("path")
                .or_else(|| map.get("file"))
                .or_else(|| map.get("manifest"))
                .or_else(|| map.get("receipt_path"))
                .or_else(|| map.get("rollback_path"))
                .and_then(Value::as_str)
            {
                let trimmed = path.trim();
                if !trimmed.is_empty() {
                    files.push(trimmed.to_string());
                }
            }
            for child in map.values() {
                collect_structural_paths(child, files);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_structural_paths(child, files);
            }
        }
        _ => {}
    }
}

async fn call_gateway_tool<H: ActionHost>(name: &str, args: &Value, host: &H) -> Value {
    match name {
        landmarks::GATEWAY_GET_MANIFEST => landmarks::manifest_result(),
        landmarks::GATEWAY_GET_LANDMARKS => landmarks::landmarks_result(),
        landmarks::GATEWAY_INSPECT_LANDMARK => landmarks::inspect_landmark_result(args),
        landmarks::GATEWAY_CALL_ACTION => run_gateway_call_action(args, host).await,
        landmarks::GATEWAY_EXECUTE_SEQUENCE => run_gateway_sequence(args, host).await,
        _ => tool_error(format!("Unknown gateway tool: {name}")),
    }
}

async fn run_gateway_call_action<H: ActionHost>(args: &Value, host: &H) -> Value {
    let (action_id, parameters) = match landmarks::parameters_from_call_action(args) {
        Ok(parsed) => parsed,
        Err(error) => return landmarks::error_result(error),
    };
    run_gateway_action(action_id, parameters, host).await
}

async fn run_gateway_action<H: ActionHost>(action_id: &str, parameters: Value, host: &H) -> Value {
    eprintln!("[canon-ai-mcp] landmark action dispatch action={action_id}");
    let Some(action) = landmarks::resolve_action_id(action_id) else {
        return landmarks::error_result(format!(
            "Unknown action '{action_id}'. Use get_landmarks then inspect_landmark before calling actions."
        ));
    };
    execute_recorded_native_action(action.native_tool.as_str(), parameters, host).await
}

async fn run_gateway_sequence<H: ActionHost>(args: &Value, host: &H) -> Value {
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
