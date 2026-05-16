//! `apply_patch` MCP tool.
//!
//! Applies or checks apply_patch-format patches inside the configured workspace.

mod fs_copy;
mod parser;
mod runner;

use serde_json::{json, Value};
use uuid::Uuid;

use super::common::{tmp_dir, tool_error};
use super::WorkspaceView;
use fs_copy::{copy_dir_recursive, TempDirGuard};
use parser::{collect_apply_patch_changed_files, validate_workspace_relative_path};
use runner::{format_apply_patch_failure, run_apply_patch_binary};

pub const APPLY_PATCH_TOOL: &str = "apply_patch";

pub async fn run(args: &Value, workspace: &WorkspaceView) -> Value {
    match run_inner(args, workspace).await {
        Ok(value) => value,
        Err(error) => tool_error(error),
    }
}

async fn run_inner(args: &Value, workspace: &WorkspaceView) -> Result<Value, String> {
    let patch_text = args
        .get("patch")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing 'patch' field".to_string())?
        .trim();
    if patch_text.is_empty() {
        return Err("patch is empty".to_string());
    }
    let max_bytes = args
        .get("maxBytes")
        .and_then(Value::as_u64)
        .unwrap_or(200_000)
        .max(1) as usize;
    if patch_text.len() > max_bytes {
        return Err(format!(
            "patch is {} bytes, larger than maxBytes={max_bytes}",
            patch_text.len()
        ));
    }
    let strip = args.get("strip").and_then(Value::as_u64).unwrap_or(0);
    if strip != 0 {
        return Err("strip is not supported by the apply_patch binary".to_string());
    }
    let mode = args.get("mode").and_then(Value::as_str).unwrap_or("check");
    let do_apply = match mode {
        "apply" => true,
        "check" => false,
        _ => return Err(format!("invalid mode {mode:?}; must be 'check' or 'apply'")),
    };
    let cwd = args.get("cwd").and_then(Value::as_str).unwrap_or(".");
    let work_dir = workspace.resolve_cwd(cwd)?;
    let changed_files = collect_apply_patch_changed_files(patch_text)?;
    for path in &changed_files {
        validate_workspace_relative_path(path)?;
    }

    let (run_dir, _guard) = if do_apply {
        (work_dir.clone(), None)
    } else {
        let temp = tmp_dir()?.join(format!("apply-patch-{}", Uuid::new_v4()));
        copy_dir_recursive(&work_dir, &temp)?;
        (temp.clone(), Some(TempDirGuard(temp)))
    };

    let output = run_apply_patch_binary(patch_text, &run_dir).await?;
    if !output.status.success() {
        return Err(format_apply_patch_failure(output));
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let summary = if stdout.is_empty() {
        let verb = if do_apply { "applied" } else { "ok (check)" };
        format!("{verb} across {} file(s)", changed_files.len())
    } else if do_apply {
        stdout
    } else {
        format!("ok (check): {stdout}")
    };
    let payload = json!({
        "ok": true,
        "mode": mode,
        "changedFiles": changed_files,
        "summary": summary,
        "rejects": []
    });
    Ok(json!({
        "content": [{ "type": "text", "text": payload.to_string() }],
        "isError": false
    }))
}
