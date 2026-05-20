//! `apply_patch` MCP tool.
//!
//! Applies or checks apply_patch-format patches inside the configured workspace.

mod fs_copy;
mod parser;
mod runner;

use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::common::{tmp_dir, tool_error};
use crate::runtime::WorkspaceView;
use fs_copy::{copy_dir_recursive_excluding, TempDirGuard};
use parser::{collect_apply_patch_changed_files, validate_workspace_relative_path};
use runner::{format_apply_patch_failure, run_apply_patch_binary};

pub const APPLY_PATCH_TOOL: &str = "apply_patch";

pub async fn run(args: &Value, workspace: &WorkspaceView) -> Value {
    match run_inner(args, workspace).await {
        Ok(value) => value,
        Err(error) => tool_error(error),
    }
}

struct ApplyPatchRequest<'a> {
    patch_text: &'a str,
    mode: &'a str,
    do_apply: bool,
    cwd: &'a str,
}

async fn run_inner(args: &Value, workspace: &WorkspaceView) -> Result<Value, String> {
    let request = parse_apply_patch_request(args)?;
    let work_dir = workspace.resolve_cwd(request.cwd)?;
    let changed_files = collect_valid_changed_files(request.patch_text)?;

    let (run_dir, _guard) = prepare_apply_patch_run_dir(request.do_apply, &work_dir)?;
    let stdout = run_validated_apply_patch(request.patch_text, &run_dir).await?;

    Ok(success_response(
        request.mode,
        request.do_apply,
        changed_files,
        stdout,
    ))
}

async fn run_validated_apply_patch(patch_text: &str, run_dir: &Path) -> Result<String, String> {
    let output = run_apply_patch_binary(patch_text, run_dir).await?;
    if !output.status.success() {
        return Err(format_apply_patch_failure(output));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn parse_apply_patch_request(args: &Value) -> Result<ApplyPatchRequest<'_>, String> {
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

    Ok(ApplyPatchRequest {
        patch_text,
        mode,
        do_apply,
        cwd,
    })
}

fn collect_valid_changed_files(patch_text: &str) -> Result<Vec<String>, String> {
    let changed_files = collect_apply_patch_changed_files(patch_text)?;
    for path in &changed_files {
        validate_workspace_relative_path(path)?;
    }
    Ok(changed_files)
}

fn prepare_apply_patch_run_dir(
    do_apply: bool,
    work_dir: &Path,
) -> Result<(PathBuf, Option<TempDirGuard>), String> {
    if do_apply {
        Ok((work_dir.to_path_buf(), None))
    } else {
        let temp_root = tmp_dir()?;
        let temp = temp_root.join(format!("apply-patch-{}", Uuid::new_v4()));
        copy_dir_recursive_excluding(work_dir, &temp, &[temp_root])?;
        Ok((temp.clone(), Some(TempDirGuard(temp))))
    }
}

fn success_response(
    mode: &str,
    do_apply: bool,
    changed_files: Vec<String>,
    stdout: String,
) -> Value {
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
    json!({
        "content": [{ "type": "text", "text": payload.to_string() }],
        "isError": false
    })
}
