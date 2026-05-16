//! Recorded shell execution through native process receipts.

use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use crate::capability::tooling::{
    LiveSandboxProcessExecutor, SandboxProcessReceipt, SandboxProcessRequest,
};
use crate::capability::CapabilityRegistry;

use super::super::common::{native_process_output_paths, tmp_dir};
use super::super::WorkspaceView;

pub fn recorded_process_request(
    args: &Value,
    workspace: &WorkspaceView,
) -> Result<SandboxProcessRequest, String> {
    let (executor, command, cwd) = recorded_process_executor(args, workspace)?;
    executor
        .process_request("/bin/sh", &["-c", command.as_str()], cwd.as_str())
        .map_err(|error| format!("native process capability rejected shell command: {error:?}"))
}

pub fn run_recorded_process(
    args: &Value,
    workspace: &WorkspaceView,
) -> Result<(SandboxProcessReceipt, Vec<u8>, Vec<u8>), String> {
    let (executor, command, cwd) = recorded_process_executor(args, workspace)?;
    let receipt = executor
        .execute_process("/bin/sh", &["-c", command.as_str()], cwd.as_str())
        .map_err(|error| format!("native process capability rejected shell command: {error:?}"))?;
    let (stdout_path, stderr_path) = native_process_output_paths(&workspace.root, &receipt);
    let stdout = fs::read(&stdout_path)
        .map_err(|error| format!("read process stdout {}: {error}", stdout_path.display()))?;
    let stderr = fs::read(&stderr_path)
        .map_err(|error| format!("read process stderr {}: {error}", stderr_path.display()))?;
    Ok((receipt, stdout, stderr))
}

fn recorded_process_executor(
    args: &Value,
    workspace: &WorkspaceView,
) -> Result<(LiveSandboxProcessExecutor, String, String), String> {
    let command = args
        .get("command")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if command.is_empty() {
        return Err("empty command".to_string());
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
        .max(1);
    let cwd = args
        .get("cwd")
        .and_then(Value::as_str)
        .unwrap_or(".")
        .to_string();
    if Path::new(&cwd).is_absolute() {
        return Err("cwd must be workspace-relative for native process capability".to_string());
    }
    let tmp_dir = tmp_dir()?;
    let executor = LiveSandboxProcessExecutor::new(workspace.root.clone())
        .with_allowed_command("/bin/sh")
        .with_locked_env(
            "PATH",
            "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        )
        .with_locked_env("TMPDIR", tmp_dir.to_string_lossy())
        .with_timeout_ms(timeout_ms)
        .with_max_output_bytes(max_output_bytes)
        .with_registry(CapabilityRegistry::canonical());
    Ok((executor, command, cwd))
}

pub fn render_recorded_response(
    receipt: &SandboxProcessReceipt,
    stdout: &[u8],
    stderr: &[u8],
) -> Value {
    let stdout_text = String::from_utf8_lossy(stdout);
    let stderr_text = String::from_utf8_lossy(stderr);
    let mut text = String::new();
    if receipt.timed_out {
        text.push_str(&format!(
            "Error: command timed out after {} ms",
            receipt.timeout_ms
        ));
    }
    if !stdout_text.is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str(stdout_text.trim_end());
    }
    if !stderr_text.is_empty() {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str("--- stderr ---\n");
        text.push_str(stderr_text.trim_end());
    }
    if text.is_empty() {
        text = format!("(exit {})", receipt.exit_status);
    }
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": !receipt.is_success(),
        "exit_code": receipt.exit_status,
        "timed_out": receipt.timed_out,
        "stdout_hash": receipt.stdout_hash,
        "stderr_hash": receipt.stderr_hash,
        "receipt_hash": receipt.receipt_hash
    })
}
