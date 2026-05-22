//! Workspace native tooling capability: patch application and shell execution.

use std::process::Stdio;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::process::Command;

use super::NativeToolHost;
use crate::api::mcp::{result_with_warning, tool_error};
use crate::api::protocol::Command as KernelCommand;
use crate::capability::tooling::mcp_tools::{apply_patch, shell};
use crate::runtime::WorkspaceView;

pub async fn execute<H: NativeToolHost>(name: &str, args: &Value, host: &H) -> Value {
    match name {
        "apply_patch" => {
            let workspace = host.workspace();
            apply_patch::run(args, &workspace).await
        }
        "shell" => {
            let workspace = host.workspace();
            shell::run_unrecorded(args, &workspace).await
        }
        "python" => {
            let workspace = host.workspace();
            run_python(args, &workspace).await
        }
        _ => tool_error(format!("Unknown workspace tool: {name}")),
    }
}

async fn run_python(args: &Value, workspace: &WorkspaceView) -> Value {
    let code = args
        .get("code")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if code.is_empty() {
        return tool_error("empty code".to_string());
    }
    let timeout_ms = args
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .unwrap_or(30_000)
        .max(1);
    let max_output_bytes = args
        .get("max_output_bytes")
        .and_then(Value::as_u64)
        .unwrap_or(65_536)
        .max(1) as usize;
    let cwd = args.get("cwd").and_then(Value::as_str).unwrap_or(".");
    let work_dir = match workspace.resolve_cwd(cwd) {
        Ok(path) => path,
        Err(error) => return tool_error(error),
    };
    let child = match Command::new("python3")
        .arg("-c")
        .arg(code)
        .current_dir(&work_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => return tool_error(error.to_string()),
    };
    let output = match tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        child.wait_with_output(),
    )
    .await
    {
        Ok(Ok(output)) => output,
        Ok(Err(error)) => return tool_error(error.to_string()),
        Err(_) => {
            return json!({
                "content": [{ "type": "text", "text": format!("Error: python timed out after {timeout_ms} ms") }],
                "isError": true,
                "exit_code": -1,
                "timed_out": true
            });
        }
    };
    let mut stdout = output.stdout;
    let mut stderr = output.stderr;
    let stdout_truncated = stdout.len() > max_output_bytes;
    let stderr_truncated = stderr.len() > max_output_bytes;
    stdout.truncate(max_output_bytes);
    stderr.truncate(max_output_bytes);
    let stdout_text = String::from_utf8_lossy(&stdout);
    let stderr_text = String::from_utf8_lossy(&stderr);
    let mut text = String::new();
    if !stdout_text.is_empty() {
        text.push_str(stdout_text.trim_end());
        if stdout_truncated {
            text.push_str("\n... (stdout truncated)");
        }
    }
    if !stderr_text.is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str("--- stderr ---\n");
        text.push_str(stderr_text.trim_end());
        if stderr_truncated {
            text.push_str("\n... (stderr truncated)");
        }
    }
    let exit_code = output.status.code().unwrap_or(-1);
    if text.is_empty() {
        text = format!("(exit {exit_code})");
    }
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": !output.status.success(),
        "exit_code": exit_code,
        "timed_out": false,
        "stdout_truncated": stdout_truncated,
        "stderr_truncated": stderr_truncated
    })
}

pub async fn execute_recorded_shell<H: NativeToolHost>(args: &Value, host: &H) -> Value {
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

    let args_owned = args.clone();
    let run_result =
        tokio::task::spawn_blocking(move || shell::run_recorded_process(&args_owned, &workspace))
            .await;
    match run_result {
        Err(join_err) => tool_error(format!("shell task panicked: {join_err}")),
        Ok(Err(error)) => tool_error(error),
        Ok(Ok((receipt, stdout, stderr))) => {
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
    }
}
