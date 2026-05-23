//! Unrecorded shell execution for direct local tool calls.

use std::process::Stdio;
use std::time::Duration;

use serde_json::{json, Value};
use tokio::process::Command;

use crate::capability::execution::action::common::tool_error;
use crate::runtime::WorkspaceView;

pub async fn run_unrecorded(args: &Value, workspace: &WorkspaceView) -> Value {
    let command = args
        .get("command")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim();
    if command.is_empty() {
        return tool_error("empty command".to_string());
    }
    let timeout_ms = args
        .get("timeout_ms")
        .and_then(Value::as_u64)
        .unwrap_or(180_000)
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
    let child = match Command::new("/bin/sh")
        .arg("-c")
        .arg(command)
        .current_dir(work_dir)
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
                "content": [{ "type": "text", "text": format!("Error: command timed out after {timeout_ms} ms") }],
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
