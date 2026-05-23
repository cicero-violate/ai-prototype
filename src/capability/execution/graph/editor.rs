//! Graph editor tooling bridge.
//!
//! This module is the operational bridge between MCP/tooling and the canonical
//! graph patch contract in `crate::capability::execution::graph`. It owns file/workspace
//! I/O and response shaping, but it does not define graph mutation receipt or
//! verification authority.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::capability::execution::graph::{
    decode_graph_mutation_ops_ndjson, decode_graph_snapshot_contract_ndjson,
    encode_graph_mutation_ops_ndjson, encode_graph_mutation_opset_receipt_ndjson,
    encode_graph_patch_receipt_ndjson, generate_graph_patch, verify_graph_mutation_ops_ndjson,
    GraphMutationOp, GraphMutationVerdict, GraphSourceFile,
};
use crate::runtime::WorkspaceView;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPlanPatchRequest {
    pub graph_contract_path: String,
    pub ops_path: String,
    pub source_root: String,
    pub patch_out: Option<String>,
    pub receipt_out: Option<String>,
}

impl GraphPlanPatchRequest {
    pub fn parse(args: &Value) -> Result<Self, String> {
        let graph_contract_path = required_string(args, "graph_contract")
            .or_else(|_| required_string(args, "graph_contract_path"))?;
        let ops_path =
            required_string(args, "ops").or_else(|_| required_string(args, "ops_path"))?;
        let source_root = args
            .get("source_root")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or(".")
            .to_string();
        let patch_out = optional_string(args, "patch_out")?;
        let receipt_out = optional_string(args, "receipt_out")?;
        Ok(Self {
            graph_contract_path,
            ops_path,
            source_root,
            patch_out,
            receipt_out,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphApplyOpsRequest {
    pub graph_path: String,
    pub ops_path: String,
    pub worktree_out: String,
    pub graph_out: Option<String>,
    pub receipt_out: Option<String>,
    pub patch_out: Option<String>,
    pub apply_patch_out: Option<String>,
    pub apply_to_source: bool,
    pub validate_command: Option<String>,
    pub recapture_command: Option<String>,
    pub artifact_root: Option<String>,
}

impl GraphApplyOpsRequest {
    pub fn parse(args: &Value) -> Result<Self, String> {
        let graph_path =
            required_string(args, "graph").or_else(|_| required_string(args, "graph_path"))?;
        let ops_path =
            required_string(args, "ops").or_else(|_| required_string(args, "ops_path"))?;
        let worktree_out = required_string(args, "worktree_out")?;
        Ok(Self {
            graph_path,
            ops_path,
            worktree_out,
            graph_out: optional_string(args, "graph_out")?,
            receipt_out: optional_string(args, "receipt_out")?,
            patch_out: optional_string(args, "patch_out")?,
            apply_patch_out: optional_string(args, "apply_patch_out")?,
            apply_to_source: optional_bool(args, "apply_to_source")?.unwrap_or(false),
            validate_command: optional_string(args, "validate_command")?,
            recapture_command: optional_string(args, "recapture_command")?,
            artifact_root: optional_string(args, "graph_artifact_root")?
                .or(optional_string(args, "artifact_root")?),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPlanCfgRequest {
    pub graph_path: String,
    pub node: String,
    pub strategy: String,
    pub ops_out: Option<String>,
    pub replacement: Option<String>,
    pub guard: Option<String>,
    pub lo: Option<usize>,
    pub hi: Option<usize>,
}

impl GraphPlanCfgRequest {
    pub fn parse(args: &Value) -> Result<Self, String> {
        let graph_path =
            required_string(args, "graph").or_else(|_| required_string(args, "graph_path"))?;
        let node = required_string(args, "node").or_else(|_| required_string(args, "path"))?;
        let strategy = required_string(args, "strategy")?;
        Ok(Self {
            graph_path,
            node,
            strategy,
            ops_out: optional_string(args, "ops_out")?,
            replacement: optional_string(args, "replacement")?,
            guard: optional_string(args, "guard")?,
            lo: optional_usize(args, "lo")?,
            hi: optional_usize(args, "hi")?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphVerifyCfgDeltaRequest {
    pub old_graph_path: String,
    pub new_graph_path: String,
    pub node: String,
    pub max_complexity_increase: Option<i64>,
    pub require_changed: bool,
}

impl GraphVerifyCfgDeltaRequest {
    pub fn parse(args: &Value) -> Result<Self, String> {
        let old_graph_path = required_string(args, "old_graph")
            .or_else(|_| required_string(args, "old_graph_path"))?;
        let new_graph_path = required_string(args, "new_graph")
            .or_else(|_| required_string(args, "new_graph_path"))?;
        let node = required_string(args, "node").or_else(|_| required_string(args, "path"))?;
        Ok(Self {
            old_graph_path,
            new_graph_path,
            node,
            max_complexity_increase: optional_i64(args, "max_complexity_increase")?,
            require_changed: optional_bool(args, "require_changed")?.unwrap_or(false),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphAutoRefactorCfgRequest {
    pub graph_path: String,
    pub node: String,
    pub strategy: String,
    pub ops_out: String,
    pub worktree_out: String,
    pub graph_out: Option<String>,
    pub receipt_out: Option<String>,
    pub patch_out: Option<String>,
    pub apply_patch_out: Option<String>,
    pub apply_to_source: bool,
    pub validate_command: Option<String>,
    pub recapture_command: Option<String>,
    pub new_graph_path: Option<String>,
    pub artifact_root: Option<String>,
    pub replacement: Option<String>,
    pub guard: Option<String>,
    pub lo: Option<usize>,
    pub hi: Option<usize>,
}

impl GraphAutoRefactorCfgRequest {
    pub fn parse(args: &Value) -> Result<Self, String> {
        let graph_path =
            required_string(args, "graph").or_else(|_| required_string(args, "graph_path"))?;
        let node = required_string(args, "node").or_else(|_| required_string(args, "path"))?;
        let strategy = required_string(args, "strategy")?;
        let ops_out = required_string(args, "ops_out")?;
        let worktree_out = required_string(args, "worktree_out")?;
        Ok(Self {
            graph_path,
            node,
            strategy,
            ops_out,
            worktree_out,
            graph_out: optional_string(args, "graph_out")?,
            receipt_out: optional_string(args, "receipt_out")?,
            patch_out: optional_string(args, "patch_out")?,
            apply_patch_out: optional_string(args, "apply_patch_out")?,
            apply_to_source: optional_bool(args, "apply_to_source")?.unwrap_or(false),
            validate_command: optional_string(args, "validate_command")?,
            recapture_command: optional_string(args, "recapture_command")?,
            artifact_root: optional_graph_artifact_root(args)?,
            new_graph_path: optional_new_graph_path(args)?,
            replacement: optional_string(args, "replacement")?,
            guard: optional_string(args, "guard")?,
            lo: optional_usize(args, "lo")?,
            hi: optional_usize(args, "hi")?,
        })
    }
}

fn optional_graph_artifact_root(args: &Value) -> Result<Option<String>, String> {
    Ok(
        optional_string_alias(args, &["graph_artifact_root", "artifact_root"])?
            .or_else(|| Some("state/rustc".to_string())),
    )
}

fn optional_new_graph_path(args: &Value) -> Result<Option<String>, String> {
    optional_string_alias(args, &["new_graph", "new_graph_path"])
}

pub fn verify_cfg_delta_tool(args: &Value, workspace: &WorkspaceView) -> Value {
    match verify_cfg_delta_tool_inner(args, workspace) {
        Ok(value) => value,
        Err(error) => crate::api::action::tool_error(error),
    }
}

fn verify_cfg_delta_tool_inner(args: &Value, workspace: &WorkspaceView) -> Result<Value, String> {
    let request = GraphVerifyCfgDeltaRequest::parse(args)?;
    let old_graph_path = resolve_workspace_file(workspace, &request.old_graph_path)?;
    let new_graph_path = resolve_workspace_file(workspace, &request.new_graph_path)?;
    let old_graph = read_json(&old_graph_path)?;
    let new_graph = read_json(&new_graph_path)?;
    let old_summary = cfg_summary(&old_graph, &request.node)?;
    let new_summary = cfg_summary(&new_graph, &request.node)?;
    let complexity_delta = new_summary.complexity as i64 - old_summary.complexity as i64;
    let changed = old_summary != new_summary;
    let max_ok = request
        .max_complexity_increase
        .map(|max| complexity_delta <= max)
        .unwrap_or(true);
    let changed_ok = !request.require_changed || changed;
    let verdict = max_ok && changed_ok;
    let payload = json!({
        "ok": verdict,
        "node": request.node,
        "oldGraph": workspace_relative_display(workspace, &old_graph_path),
        "newGraph": workspace_relative_display(workspace, &new_graph_path),
        "old": old_summary.to_json(),
        "new": new_summary.to_json(),
        "delta": {
            "complexity": complexity_delta,
            "blocks": new_summary.blocks as i64 - old_summary.blocks as i64,
            "edges": new_summary.edges as i64 - old_summary.edges as i64,
        },
        "checks": {
            "maxComplexityIncreaseOk": max_ok,
            "changedOk": changed_ok,
        }
    });
    Ok(json!({
        "content": [{ "type": "text", "text": payload.to_string() }],
        "isError": !verdict
    }))
}

pub fn auto_refactor_cfg_tool(args: &Value, workspace: &WorkspaceView) -> Value {
    match auto_refactor_cfg_tool_inner(args, workspace) {
        Ok(value) => value,
        Err(error) => crate::api::action::tool_error(error),
    }
}

fn auto_refactor_cfg_tool_inner(args: &Value, workspace: &WorkspaceView) -> Result<Value, String> {
    let request = GraphAutoRefactorCfgRequest::parse(args)?;
    let plan_args = json!({
        "graph": request.graph_path,
        "node": request.node,
        "strategy": request.strategy,
        "replacement": request.replacement,
        "guard": request.guard,
        "lo": request.lo,
        "hi": request.hi,
        "ops_out": request.ops_out,
    });
    let plan = plan_cfg_tool_inner(&plan_args, workspace)?;
    let apply_args = json!({
        "graph": request.graph_path,
        "ops": request.ops_out,
        "worktree_out": request.worktree_out,
        "graph_out": request.graph_out,
        "receipt_out": request.receipt_out,
        "patch_out": request.patch_out,
        "apply_patch_out": request.apply_patch_out,
        "apply_to_source": request.apply_to_source,
        "validate_command": request.validate_command,
        "recapture_command": request.recapture_command,
        "artifact_root": request.artifact_root,
    });
    let apply = apply_ops_tool_inner(&apply_args, workspace)?;
    let cfg_delta = if let Some(new_graph_path) = request.new_graph_path.as_deref() {
        let verify_args = json!({
            "old_graph": request.graph_path,
            "new_graph": new_graph_path,
            "node": request.node,
            "max_complexity_increase": 0,
        });
        Some(verify_cfg_delta_tool_inner(&verify_args, workspace)?)
    } else {
        None
    };
    let payload = json!({
        "ok": true,
        "strategy": request.strategy,
        "node": request.node,
        "opsOut": request.ops_out,
        "worktreeOut": request.worktree_out,
        "plan": plan,
        "apply": apply,
        "cfgDelta": cfg_delta,
    });
    Ok(json!({
        "content": [{ "type": "text", "text": payload.to_string() }],
        "isError": false
    }))
}

pub fn plan_cfg_tool(args: &Value, workspace: &WorkspaceView) -> Value {
    match plan_cfg_tool_inner(args, workspace) {
        Ok(value) => value,
        Err(error) => crate::api::action::tool_error(error),
    }
}

fn plan_cfg_tool_inner(args: &Value, workspace: &WorkspaceView) -> Result<Value, String> {
    let request = GraphPlanCfgRequest::parse(args)?;
    let graph_path = resolve_workspace_file(workspace, &request.graph_path)?;
    let graph_input = read_to_string(&graph_path)?;
    let graph_json: Value = serde_json::from_str(&graph_input)
        .map_err(|error| format!("invalid graph json {}: {error}", graph_path.display()))?;
    let node = graph_json
        .get("nodes")
        .and_then(Value::as_object)
        .and_then(|nodes| nodes.get(&request.node))
        .ok_or_else(|| format!("graph node not found: {}", request.node))?;
    let span = node
        .get("def")
        .ok_or_else(|| format!("graph node has no source span: {}", request.node))?;
    let file = span
        .get("file")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("graph node span has no file: {}", request.node))?
        .to_string();
    let lo = request
        .lo
        .or_else(|| {
            span.get("lo")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
        })
        .ok_or_else(|| format!("graph node span has no lo: {}", request.node))?;
    let hi = request
        .hi
        .or_else(|| {
            span.get("hi")
                .and_then(Value::as_u64)
                .map(|value| value as usize)
        })
        .ok_or_else(|| format!("graph node span has no hi: {}", request.node))?;
    let cfg = node.get("metrics").and_then(|metrics| metrics.get("cfg"));
    if cfg.is_none() {
        return Err(format!("graph node has no metrics.cfg: {}", request.node));
    }

    let op = plan_cfg_op(&request, file, lo, hi)?;
    let ops = vec![op];
    let ops_ndjson = encode_graph_mutation_ops_ndjson(&ops);
    if let Some(path) = request.ops_out.as_deref() {
        let out = resolve_workspace_file(workspace, path)?;
        write_string(&out, &ops_ndjson)?;
    }
    let payload = json!({
        "ok": true,
        "graph": workspace_relative_display(workspace, &graph_path),
        "node": request.node,
        "strategy": request.strategy,
        "ops": ops_ndjson,
        "opsOut": request.ops_out,
        "cfg": {
            "blocks": cfg.and_then(|cfg| cfg.get("blocks")).and_then(Value::as_array).map(|v| v.len()).unwrap_or(0),
            "edges": cfg.and_then(|cfg| cfg.get("edges")).and_then(Value::as_array).map(|v| v.len()).unwrap_or(0)
        }
    });
    Ok(json!({
        "content": [{ "type": "text", "text": payload.to_string() }],
        "isError": false
    }))
}

fn plan_cfg_op(
    request: &GraphPlanCfgRequest,
    file: String,
    lo: usize,
    hi: usize,
) -> Result<GraphMutationOp, String> {
    let path = request.node.clone();
    match request.strategy.as_str() {
        "ReplaceSpan" => Ok(GraphMutationOp::ReplaceSpan {
            path,
            file,
            lo,
            hi,
            replacement: request
                .replacement
                .clone()
                .ok_or_else(|| "replacement is required for ReplaceSpan".to_string())?,
        }),
        "InvertBranch" => Ok(GraphMutationOp::InvertBranch {
            path,
            file,
            lo,
            hi,
            replacement: request
                .replacement
                .clone()
                .ok_or_else(|| "replacement is required for InvertBranch".to_string())?,
        }),
        "GuardClauseInsert" => Ok(GraphMutationOp::GuardClauseInsert {
            path,
            file,
            lo,
            hi,
            guard: request
                .guard
                .clone()
                .or_else(|| request.replacement.clone())
                .ok_or_else(|| {
                    "guard or replacement is required for GuardClauseInsert".to_string()
                })?,
        }),
        "ExtractBlock" => Ok(GraphMutationOp::ExtractBlock {
            path,
            file,
            lo,
            hi,
            replacement: request
                .replacement
                .clone()
                .ok_or_else(|| "replacement is required for ExtractBlock".to_string())?,
        }),
        "InlineBlock" => Ok(GraphMutationOp::InlineBlock {
            path,
            file,
            lo,
            hi,
            replacement: request
                .replacement
                .clone()
                .ok_or_else(|| "replacement is required for InlineBlock".to_string())?,
        }),
        "SplitLoop" => Ok(GraphMutationOp::SplitLoop {
            path,
            file,
            lo,
            hi,
            replacement: request
                .replacement
                .clone()
                .ok_or_else(|| "replacement is required for SplitLoop".to_string())?,
        }),
        "ConvertIfToMatch" => Ok(GraphMutationOp::ConvertIfToMatch {
            path,
            file,
            lo,
            hi,
            replacement: request
                .replacement
                .clone()
                .ok_or_else(|| "replacement is required for ConvertIfToMatch".to_string())?,
        }),
        "MoveStatement" => Ok(GraphMutationOp::MoveStatement {
            path,
            file,
            lo,
            hi,
            replacement: request
                .replacement
                .clone()
                .ok_or_else(|| "replacement is required for MoveStatement".to_string())?,
        }),
        "DeleteDeadBranch" => Ok(GraphMutationOp::DeleteDeadBranch { path, file, lo, hi }),
        other => Err(format!("unsupported CFG strategy: {other}")),
    }
}

pub fn apply_ops_tool(args: &Value, workspace: &WorkspaceView) -> Value {
    match apply_ops_tool_inner(args, workspace) {
        Ok(value) => value,
        Err(error) => crate::api::action::tool_error(error),
    }
}

fn apply_ops_tool_inner(args: &Value, workspace: &WorkspaceView) -> Result<Value, String> {
    let request = GraphApplyOpsRequest::parse(args)?;
    let graph_path = resolve_workspace_file(workspace, &request.graph_path)?;
    let ops_path = resolve_workspace_file(workspace, &request.ops_path)?;
    let worktree_out = resolve_workspace_file(workspace, &request.worktree_out)?;

    let graph_input = read_to_string(&graph_path)?;
    let mut graph_json: Value = serde_json::from_str(&graph_input)
        .map_err(|error| format!("invalid graph json {}: {error}", graph_path.display()))?;
    let ops_input = read_to_string(&ops_path)?;
    let ops_receipt = verify_graph_mutation_ops_ndjson(&ops_input);
    if ops_receipt.verdict == GraphMutationVerdict::Fail {
        return Err("operation ledger failed verification".to_string());
    }
    let ops = decode_graph_mutation_ops_ndjson(&ops_input).ok_or_else(|| {
        format!(
            "failed to decode graph mutation ops ndjson: {}",
            ops_path.display()
        )
    })?;

    let graph_contract = decode_graph_snapshot_contract_ndjson(&graph_input)
        .or_else(|| graph_snapshot_contract_from_graph_json(&graph_json))
        .ok_or_else(|| {
            format!(
                "failed to derive graph mutation contract from {}",
                graph_path.display()
            )
        })?;
    let sources = read_sources_from_graph_files(&graph_json, &ops)?;
    let plan = generate_graph_patch(&graph_contract, &sources, &ops)
        .map_err(|error| format!("failed to generate graph patch: {error}"))?;

    apply_ops_to_graph_files(&mut graph_json, &ops)?;
    render_apply_ops_worktree(workspace, &request, &graph_json, &worktree_out)?;

    let apply_patch_text = apply_patch_text_from_unified_diff(&plan.diff)?;
    write_apply_ops_outputs(
        workspace,
        &request,
        &graph_json,
        &plan.diff,
        &apply_patch_text,
    )?;
    let source_apply = apply_diff_to_source_if_requested(workspace, &request, &plan.diff)?;

    let receipt_payload = apply_ops_receipt_payload(
        workspace,
        &request,
        &graph_path,
        &ops_path,
        &worktree_out,
        &apply_patch_text,
        source_apply.as_ref(),
        &plan,
        &ops_receipt,
    );
    write_optional_json(workspace, request.receipt_out.as_deref(), &receipt_payload)?;

    let validation =
        run_optional_worktree_command(request.validate_command.as_deref(), &worktree_out)?;
    let recapture =
        run_optional_worktree_command(request.recapture_command.as_deref(), &worktree_out)?;

    let payload = json!({
        "ok": true,
        "graph": workspace_relative_display(workspace, &graph_path),
        "ops": workspace_relative_display(workspace, &ops_path),
        "worktreeOut": workspace_relative_display(workspace, &worktree_out),
        "graphOut": request.graph_out,
        "receiptOut": request.receipt_out,
        "patchOut": request.patch_out,
        "applyPatchOut": request.apply_patch_out,
        "artifactRoot": request.artifact_root,
        "changedFiles": changed_files(&ops),
        "validationStatus": validation
            .as_ref()
            .and_then(|value| value.get("status"))
            .and_then(Value::as_i64),
        "patchHash": plan.receipt.patch_hash,
        "sourceApply": source_apply,
        "validation": validation,
        "recapture": recapture,
    });
    Ok(json!({
        "content": [{ "type": "text", "text": payload.to_string() }],
        "isError": false
    }))
}

fn render_apply_ops_worktree(
    workspace: &WorkspaceView,
    request: &GraphApplyOpsRequest,
    graph_json: &Value,
    worktree_out: &Path,
) -> Result<(), String> {
    let Some(artifact_root) = request.artifact_root.as_deref() else {
        return render_graph_files_json(graph_json, worktree_out);
    };

    let artifact_root = resolve_workspace_file(workspace, artifact_root)?;
    let mut render_json = merged_graph_files_json(&artifact_root)?;
    overlay_graph_files(&mut render_json, graph_json)?;
    render_graph_files_json(&render_json, worktree_out)?;
    complete_missing_workspace_members(&workspace.root, worktree_out)
}

fn write_apply_ops_outputs(
    workspace: &WorkspaceView,
    request: &GraphApplyOpsRequest,
    graph_json: &Value,
    patch_diff: &str,
    apply_patch_text: &str,
) -> Result<(), String> {
    write_optional_pretty_json(workspace, request.graph_out.as_deref(), graph_json)?;
    write_optional_text(workspace, request.patch_out.as_deref(), patch_diff)?;
    write_optional_text(
        workspace,
        request.apply_patch_out.as_deref(),
        apply_patch_text,
    )?;
    Ok(())
}

fn write_optional_text(
    workspace: &WorkspaceView,
    path: Option<&str>,
    contents: &str,
) -> Result<(), String> {
    let Some(path) = path else {
        return Ok(());
    };
    let out = resolve_workspace_file(workspace, path)?;
    write_string(&out, contents)
}

fn write_optional_pretty_json(
    workspace: &WorkspaceView,
    path: Option<&str>,
    payload: &Value,
) -> Result<(), String> {
    write_optional_text(
        workspace,
        path,
        &serde_json::to_string_pretty(payload).unwrap_or_else(|_| payload.to_string()),
    )
}

fn apply_diff_to_source_if_requested(
    workspace: &WorkspaceView,
    request: &GraphApplyOpsRequest,
    patch_diff: &str,
) -> Result<Option<Value>, String> {
    if !request.apply_to_source {
        return Ok(None);
    }
    apply_unified_diff_to_workspace(workspace, patch_diff).map(Some)
}

fn apply_unified_diff_to_workspace(
    workspace: &WorkspaceView,
    patch_diff: &str,
) -> Result<Value, String> {
    let output = Command::new("git")
        .arg("apply")
        .arg("--index")
        .arg("--whitespace=nowarn")
        .current_dir(&workspace.root)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                stdin.write_all(patch_diff.as_bytes())?;
            }
            child.wait_with_output()
        })
        .map_err(|error| format!("failed to run git apply: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let status = output.status.code().unwrap_or(-1);
    if !output.status.success() {
        return Err(format!("git apply failed with status {status}: {stderr}"));
    }
    Ok(json!({
        "command": "git apply --index --whitespace=nowarn",
        "status": status,
        "success": true,
        "stdout": stdout,
        "stderr": stderr,
    }))
}

#[expect(
    clippy::too_many_arguments,
    reason = "receipt payload assembly records all graph mutation metadata at one boundary"
)]
fn apply_ops_receipt_payload(
    workspace: &WorkspaceView,
    request: &GraphApplyOpsRequest,
    graph_path: &Path,
    ops_path: &Path,
    worktree_out: &Path,
    apply_patch_text: &str,
    source_apply: Option<&Value>,
    plan: &crate::capability::execution::graph::GraphPatchPlan,
    ops_receipt: &crate::capability::execution::graph::GraphMutationOpSetReceipt,
) -> Value {
    json!({
        "ok": true,
        "graph": workspace_relative_display(workspace, graph_path),
        "ops": workspace_relative_display(workspace, ops_path),
        "worktreeOut": workspace_relative_display(workspace, worktree_out),
        "graphOut": request.graph_out,
        "patchOut": request.patch_out,
        "applyPatchOut": request.apply_patch_out,
        "artifactRoot": request.artifact_root,
        "applyToSource": request.apply_to_source,
        "sourceApply": source_apply,
        "applyPatch": apply_patch_text,
        "patchReceipt": encode_graph_patch_receipt_ndjson(&plan.receipt),
        "opsReceipt": encode_graph_mutation_opset_receipt_ndjson(ops_receipt),
    })
}

fn write_optional_json(
    workspace: &WorkspaceView,
    path: Option<&str>,
    payload: &Value,
) -> Result<(), String> {
    let Some(path) = path else {
        return Ok(());
    };
    let out = resolve_workspace_file(workspace, path)?;
    write_string(
        &out,
        &serde_json::to_string_pretty(payload).unwrap_or_else(|_| payload.to_string()),
    )
}

fn run_optional_worktree_command(
    command: Option<&str>,
    worktree_out: &Path,
) -> Result<Option<Value>, String> {
    command
        .map(|command| run_shell_command(command, worktree_out))
        .transpose()
}

fn graph_snapshot_contract_from_graph_json(
    graph_json: &Value,
) -> Option<crate::capability::execution::graph::GraphSnapshotContract> {
    use crate::capability::execution::graph::{
        GraphEdgeContract, GraphNodeContract, GraphSnapshotContract, GraphSourceSpan,
    };
    let schema = graph_json.get("meta")?.get("schema_version")?.as_u64()? as u32;
    let graph_hash = graph_json
        .get("meta")?
        .get("graph_hash")
        .and_then(Value::as_str)
        .map(stable_text_u64)
        .unwrap_or(1);
    let mut contract = GraphSnapshotContract::new(u64::from(schema), graph_hash);
    let nodes = graph_json.get("nodes")?.as_object()?;
    for (path, node) in nodes {
        let Some(span) = node.get("def") else {
            continue;
        };
        let Some(file) = span.get("file").and_then(Value::as_str) else {
            continue;
        };
        let Some(lo) = span.get("lo").and_then(Value::as_u64) else {
            continue;
        };
        let Some(hi) = span.get("hi").and_then(Value::as_u64) else {
            continue;
        };
        let line = span.get("line").and_then(Value::as_u64).unwrap_or(1);
        let col = span.get("col").and_then(Value::as_u64).unwrap_or(1);
        let kind = node.get("kind").and_then(Value::as_str).unwrap_or("");
        let def_id = node.get("def_id").and_then(Value::as_str).unwrap_or("");
        contract.insert_node(GraphNodeContract::new(
            path.clone(),
            kind.to_string(),
            def_id.to_string(),
            GraphSourceSpan::new(file.to_string(), line, col, lo as usize, hi as usize),
        ));
    }
    if let Some(edges) = graph_json.get("edges").and_then(Value::as_array) {
        for edge in edges {
            let Some(relation) = edge.get("relation").and_then(Value::as_str) else {
                continue;
            };
            let Some(from) = edge.get("from").and_then(Value::as_str) else {
                continue;
            };
            let Some(to) = edge.get("to").and_then(Value::as_str) else {
                continue;
            };
            contract.insert_edge(GraphEdgeContract::new(
                relation.to_string(),
                from.to_string(),
                to.to_string(),
            ));
        }
    }
    Some(contract)
}

fn read_sources_from_graph_files(
    graph_json: &Value,
    ops: &[GraphMutationOp],
) -> Result<Vec<GraphSourceFile>, String> {
    let files = graph_json
        .get("files")
        .and_then(Value::as_object)
        .ok_or_else(|| "graph json has no files object".to_string())?;
    let mut wanted = changed_files(ops);
    wanted.sort();
    wanted.dedup();
    let mut sources = Vec::new();
    for path in wanted {
        let record = files
            .get(&path)
            .ok_or_else(|| format!("graph.files missing source file {path}"))?;
        let text = record
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("graph.files[{path}].text missing"))?;
        sources.push(GraphSourceFile::new(path, text.to_string()));
    }
    Ok(sources)
}

fn apply_ops_to_graph_files(graph_json: &mut Value, ops: &[GraphMutationOp]) -> Result<(), String> {
    let files = graph_json
        .get_mut("files")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| "graph json has no mutable files object".to_string())?;
    let mut by_file: BTreeMap<String, Vec<GraphMutationOp>> = BTreeMap::new();
    for op in ops {
        by_file
            .entry(op.file().to_string())
            .or_default()
            .push(op.clone());
    }
    for (path, mut file_ops) in by_file {
        file_ops.sort_by(|a, b| b.lo().cmp(&a.lo()).then(b.hi().cmp(&a.hi())));
        let record = files
            .get_mut(&path)
            .ok_or_else(|| format!("graph.files missing source file {path}"))?;
        let text_value = record
            .get_mut("text")
            .ok_or_else(|| format!("graph.files[{path}].text missing"))?;
        let mut text = text_value
            .as_str()
            .ok_or_else(|| format!("graph.files[{path}].text is not a string"))?
            .to_string();
        for op in file_ops {
            let replacement = replacement_for_op(&text, &op)?;
            text.replace_range(op.lo()..op.hi(), &replacement);
        }
        *text_value = Value::String(text.clone());
        if let Some(obj) = record.as_object_mut() {
            obj.insert("byte_len".to_string(), json!(text.len()));
            obj.insert("sha256".to_string(), json!(sha256_text(&text)));
        }
    }
    Ok(())
}

fn replacement_for_op(content: &str, op: &GraphMutationOp) -> Result<String, String> {
    if op.lo() > op.hi() || op.hi() > content.len() {
        return Err(format!("invalid op span for {}", op.file()));
    }
    let old = &content[op.lo()..op.hi()];
    op.replacement_text(old)
        .ok_or_else(|| format!("unsupported graph op replacement for {}", op.file()))
}

fn merged_graph_files_json(artifact_root: &Path) -> Result<Value, String> {
    let mut merged = json!({ "files": {} });
    let mut paths = Vec::new();
    collect_graph_json_paths(artifact_root, &mut paths)?;
    paths.sort();
    for path in paths {
        let graph = read_json(&path)?;
        overlay_graph_files(&mut merged, &graph)?;
    }
    Ok(merged)
}

fn overlay_graph_files(target: &mut Value, source: &Value) -> Result<(), String> {
    let Some(source_files) = source.get("files").and_then(Value::as_object) else {
        return Ok(());
    };
    let target_files = target
        .get_mut("files")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| "target graph json has no mutable files object".to_string())?;
    for (path, record) in source_files {
        target_files.insert(path.clone(), record.clone());
    }
    Ok(())
}

fn collect_graph_json_paths(dir: &Path, out: &mut Vec<std::path::PathBuf>) -> Result<(), String> {
    let entries = fs::read_dir(dir)
        .map_err(|error| format!("failed to read artifact dir {}: {error}", dir.display()))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("failed to read entry in {}: {error}", dir.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to inspect {}: {error}", path.display()))?;
        if file_type.is_dir() {
            collect_graph_json_paths(&path, out)?;
        } else if file_type.is_file()
            && path.file_name().and_then(|name| name.to_str()) == Some("graph.json")
        {
            out.push(path);
        }
    }
    Ok(())
}

fn render_graph_files_json(graph_json: &Value, output_root: &Path) -> Result<(), String> {
    let files = graph_json
        .get("files")
        .and_then(Value::as_object)
        .ok_or_else(|| "graph json has no files object".to_string())?;
    if output_root.exists() {
        fs::remove_dir_all(output_root)
            .map_err(|error| format!("failed to clear {}: {error}", output_root.display()))?;
    }
    fs::create_dir_all(output_root)
        .map_err(|error| format!("failed to create {}: {error}", output_root.display()))?;
    for (path, record) in files {
        validate_relative_render_path(path)?;
        let text = record
            .get("text")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("graph.files[{path}].text missing"))?;
        let dest = output_root.join(path);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
        fs::write(&dest, text)
            .map_err(|error| format!("failed to write {}: {error}", dest.display()))?;
    }
    Ok(())
}

fn complete_missing_workspace_members(
    workspace_root: &Path,
    output_root: &Path,
) -> Result<(), String> {
    let rendered_manifest = output_root.join("Cargo.toml");
    if !rendered_manifest.exists() {
        return Ok(());
    }
    let manifest = fs::read_to_string(&rendered_manifest)
        .map_err(|error| format!("failed to read {}: {error}", rendered_manifest.display()))?;
    for member in workspace_members_from_manifest(&manifest) {
        validate_relative_render_path(&member)?;
        let rendered_member_manifest = output_root.join(&member).join("Cargo.toml");
        if rendered_member_manifest.exists() {
            continue;
        }
        let source_member = workspace_root.join(&member);
        if !source_member.join("Cargo.toml").exists() {
            continue;
        }
        copy_workspace_member_tree(&source_member, &output_root.join(&member))?;
    }
    Ok(())
}

fn workspace_members_from_manifest(manifest: &str) -> Vec<String> {
    let Some(members_start) = manifest.find("members") else {
        return Vec::new();
    };
    let tail = &manifest[members_start..];
    let Some(open) = tail.find('[') else {
        return Vec::new();
    };
    let Some(close) = tail[open + 1..].find(']') else {
        return Vec::new();
    };
    let body = &tail[open + 1..open + 1 + close];
    body.split(',')
        .filter_map(|part| {
            let trimmed = part.trim().trim_matches('"').trim_matches('\'');
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .collect()
}

fn copy_workspace_member_tree(source: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest)
        .map_err(|error| format!("failed to create {}: {error}", dest.display()))?;
    for entry in fs::read_dir(source)
        .map_err(|error| format!("failed to read {}: {error}", source.display()))?
    {
        let entry =
            entry.map_err(|error| format!("failed to read workspace member entry: {error}"))?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == ".git" || name == "target" {
            continue;
        }
        let source_path = entry.path();
        let dest_path = dest.join(name.as_ref());
        let file_type = entry
            .file_type()
            .map_err(|error| format!("failed to inspect {}: {error}", source_path.display()))?;
        if file_type.is_dir() {
            copy_workspace_member_tree(&source_path, &dest_path)?;
        } else if file_type.is_file() {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
            }
            fs::copy(&source_path, &dest_path).map_err(|error| {
                format!(
                    "failed to copy {} to {}: {error}",
                    source_path.display(),
                    dest_path.display()
                )
            })?;
        }
    }
    Ok(())
}

fn run_shell_command(command: &str, cwd: &Path) -> Result<Value, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(cwd)
        .output()
        .map_err(|error| format!("failed to run command {command:?}: {error}"))?;
    Ok(json!({
        "command": command,
        "status": output.status.code(),
        "success": output.status.success(),
        "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
        "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
    }))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CfgSummary {
    complexity: u64,
    blocks: usize,
    edges: usize,
    cleanup_blocks: usize,
    terminator_hash: String,
}

impl CfgSummary {
    fn to_json(&self) -> Value {
        json!({
            "complexity": self.complexity,
            "blocks": self.blocks,
            "edges": self.edges,
            "cleanupBlocks": self.cleanup_blocks,
            "terminatorHash": self.terminator_hash,
        })
    }
}

fn cfg_summary(graph: &Value, node_path: &str) -> Result<CfgSummary, String> {
    let node = graph
        .get("nodes")
        .and_then(Value::as_object)
        .and_then(|nodes| nodes.get(node_path))
        .ok_or_else(|| format!("graph node not found: {node_path}"))?;
    let metrics = node
        .get("metrics")
        .ok_or_else(|| format!("graph node has no metrics: {node_path}"))?;
    let complexity = metrics
        .get("cyclomatic_complexity")
        .and_then(Value::as_u64)
        .unwrap_or(1);
    let cfg = metrics
        .get("cfg")
        .ok_or_else(|| format!("graph node has no metrics.cfg: {node_path}"))?;
    let blocks = cfg
        .get("blocks")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("graph node cfg has no blocks: {node_path}"))?;
    let edges = cfg
        .get("edges")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("graph node cfg has no edges: {node_path}"))?;
    let cleanup_blocks = blocks
        .iter()
        .filter(|block| {
            block
                .get("is_cleanup")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
        .count();
    let terminators = blocks
        .iter()
        .filter_map(|block| block.get("terminator").and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    Ok(CfgSummary {
        complexity,
        blocks: blocks.len(),
        edges: edges.len(),
        cleanup_blocks,
        terminator_hash: sha256_text(&terminators),
    })
}

fn read_json(path: &Path) -> Result<Value, String> {
    let input = read_to_string(path)?;
    serde_json::from_str(&input)
        .map_err(|error| format!("invalid json {}: {error}", path.display()))
}

fn changed_files(ops: &[GraphMutationOp]) -> Vec<String> {
    let mut files: Vec<String> = ops.iter().map(|op| op.file().to_string()).collect();
    files.sort();
    files.dedup();
    files
}

fn validate_relative_render_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if path.is_absolute() {
        return Err(format!("render path must be relative: {}", path.display()));
    }
    for component in path.components() {
        match component {
            std::path::Component::Normal(_) | std::path::Component::CurDir => {}
            std::path::Component::Prefix(_)
            | std::path::Component::RootDir
            | std::path::Component::ParentDir => {
                return Err(format!(
                    "render path escapes output dir: {}",
                    path.display()
                ))
            }
        }
    }
    Ok(())
}

fn apply_patch_text_from_unified_diff(diff: &str) -> Result<String, String> {
    let mut out = String::from("*** Begin Patch\n");
    let mut current_file: Option<String> = None;
    let mut saw_hunk = false;
    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("--- a/") {
            current_file = Some(path.split('\t').next().unwrap_or(path).to_string());
            continue;
        }
        if let Some(path) = line.strip_prefix("+++ b/") {
            let path = path.split('\t').next().unwrap_or(path);
            let selected = current_file.take().unwrap_or_else(|| path.to_string());
            out.push_str("*** Update File: ");
            out.push_str(&selected);
            out.push('\n');
            continue;
        }
        if line.starts_with("@@") || saw_hunk {
            saw_hunk = true;
            out.push_str(line);
            out.push('\n');
        }
    }
    if !saw_hunk {
        return Err("generated patch has no hunks".to_string());
    }
    out.push_str("*** End Patch\n");
    Ok(out)
}

fn sha256_text(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}

fn stable_text_u64(text: &str) -> u64 {
    let digest = Sha256::digest(text.as_bytes());
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    u64::from_be_bytes(bytes).max(1)
}

pub fn plan_patch_tool(args: &Value, workspace: &WorkspaceView) -> Value {
    match plan_patch_tool_inner(args, workspace) {
        Ok(value) => value,
        Err(error) => crate::api::action::tool_error(error),
    }
}

fn plan_patch_tool_inner(args: &Value, workspace: &WorkspaceView) -> Result<Value, String> {
    let request = GraphPlanPatchRequest::parse(args)?;
    let source_root = workspace.resolve_cwd(&request.source_root)?;
    let graph_path = resolve_workspace_file(workspace, &request.graph_contract_path)?;
    let ops_path = resolve_workspace_file(workspace, &request.ops_path)?;

    let graph_input = read_to_string(&graph_path)?;
    let graph = decode_graph_snapshot_contract_ndjson(&graph_input).ok_or_else(|| {
        format!(
            "failed to decode graph snapshot contract ndjson: {}",
            graph_path.display()
        )
    })?;

    let ops_input = read_to_string(&ops_path)?;
    let ops_receipt = verify_graph_mutation_ops_ndjson(&ops_input);
    if ops_receipt.verdict == GraphMutationVerdict::Fail {
        return Err("operation ledger failed verification".to_string());
    }
    let ops = decode_graph_mutation_ops_ndjson(&ops_input).ok_or_else(|| {
        format!(
            "failed to decode graph mutation ops ndjson: {}",
            ops_path.display()
        )
    })?;
    let sources = read_sources(&source_root, &ops)?;
    let plan = generate_graph_patch(&graph, &sources, &ops)
        .map_err(|error| format!("failed to generate graph patch: {error}"))?;
    let patch_receipt = encode_graph_patch_receipt_ndjson(&plan.receipt);
    let ops_receipt = encode_graph_mutation_opset_receipt_ndjson(&ops_receipt);

    if let Some(path) = request.patch_out.as_deref() {
        let out = resolve_workspace_file(workspace, path)?;
        write_string(&out, &plan.diff)?;
    }
    if let Some(path) = request.receipt_out.as_deref() {
        let out = resolve_workspace_file(workspace, path)?;
        write_string(&out, &patch_receipt)?;
    }

    let payload = json!({
        "ok": true,
        "sourceRoot": source_root.display().to_string(),
        "graphContract": workspace_relative_display(workspace, &graph_path),
        "ops": workspace_relative_display(workspace, &ops_path),
        "patch": plan.diff,
        "patchReceipt": patch_receipt,
        "opsReceipt": ops_receipt,
        "patchOut": request.patch_out,
        "receiptOut": request.receipt_out,
    });
    Ok(json!({
        "content": [{ "type": "text", "text": payload.to_string() }],
        "isError": false
    }))
}

fn read_sources(
    source_root: &Path,
    ops: &[GraphMutationOp],
) -> Result<Vec<GraphSourceFile>, String> {
    let mut files = Vec::<String>::new();
    for op in ops {
        let file = op.file().to_string();
        if !files.contains(&file) {
            files.push(file);
        }
    }
    files.sort();

    let mut sources = Vec::new();
    for file in files {
        let path = source_root.join(&file);
        let content = read_to_string(&path)?;
        sources.push(GraphSourceFile::new(file, content));
    }
    Ok(sources)
}

fn required_string(args: &Value, key: &str) -> Result<String, String> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| format!("{key} is required"))
}

fn optional_string(args: &Value, key: &str) -> Result<Option<String>, String> {
    optional_arg(args, key, |value| match value {
        Value::String(value) if !value.is_empty() => Ok(Some(value.clone())),
        Value::String(_) => Ok(None),
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::Array(_) | Value::Object(_) => {
            Err(format!("{key} must be a string"))
        }
    })
}

fn optional_usize(args: &Value, key: &str) -> Result<Option<usize>, String> {
    optional_arg(args, key, |value| match value {
        Value::Number(value) => value
            .as_u64()
            .map(|value| Some(value as usize))
            .ok_or_else(|| format!("{key} must be a non-negative integer")),
        Value::String(value) if value.is_empty() => Ok(None),
        Value::String(value) => value
            .parse::<usize>()
            .map(Some)
            .map_err(|_| format!("{key} must be a non-negative integer")),
        Value::Null | Value::Bool(_) | Value::Array(_) | Value::Object(_) => {
            Err(format!("{key} must be a non-negative integer"))
        }
    })
}

fn optional_i64(args: &Value, key: &str) -> Result<Option<i64>, String> {
    optional_arg(args, key, |value| match value {
        Value::Number(value) => value
            .as_i64()
            .map(Some)
            .ok_or_else(|| format!("{key} must be an integer")),
        Value::String(value) if value.is_empty() => Ok(None),
        Value::String(value) => value
            .parse::<i64>()
            .map(Some)
            .map_err(|_| format!("{key} must be an integer")),
        Value::Null | Value::Bool(_) | Value::Array(_) | Value::Object(_) => {
            Err(format!("{key} must be an integer"))
        }
    })
}

fn optional_bool(args: &Value, key: &str) -> Result<Option<bool>, String> {
    optional_arg(args, key, |value| match value {
        Value::Bool(value) => Ok(Some(*value)),
        Value::String(value) if value.is_empty() => Ok(None),
        Value::String(value) => value
            .parse::<bool>()
            .map(Some)
            .map_err(|_| format!("{key} must be a boolean")),
        Value::Null | Value::Number(_) | Value::Array(_) | Value::Object(_) => {
            Err(format!("{key} must be a boolean"))
        }
    })
}

fn optional_arg<T>(
    args: &Value,
    key: &str,
    parse: impl FnOnce(&Value) -> Result<Option<T>, String>,
) -> Result<Option<T>, String> {
    match args.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => parse(value),
    }
}

fn optional_string_alias(args: &Value, keys: &[&str]) -> Result<Option<String>, String> {
    keys.iter().try_fold(None, |found, key| {
        optional_string(args, key).map(|value| found.or(value))
    })
}

fn resolve_workspace_file(
    workspace: &WorkspaceView,
    path: &str,
) -> Result<std::path::PathBuf, String> {
    let path = workspace.resolve_cwd(path)?;
    if !path.starts_with(&workspace.allowed_boundary) {
        return Err("path escapes allowed workspace boundary".to_string());
    }
    Ok(path)
}

fn read_to_string(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("failed to read {}: {error}", path.display()))
}

fn write_string(path: &Path, content: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    fs::write(path, content).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn workspace_relative_display(workspace: &WorkspaceView, path: &Path) -> String {
    path.strip_prefix(&workspace.root)
        .map(|rel| rel.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn graph_plan_patch_request_accepts_aliases_and_optional_outputs() {
        let request = GraphPlanPatchRequest::parse(&json!({
            "graph_contract_path": "state/graph.ndjson",
            "ops_path": "ops.ndjson",
            "source_root": "ai",
            "patch_out": "patch.diff",
            "receipt_out": "receipt.ndjson"
        }))
        .expect("valid request");
        assert_eq!(request.graph_contract_path, "state/graph.ndjson");
        assert_eq!(request.ops_path, "ops.ndjson");
        assert_eq!(request.source_root, "ai");
        assert_eq!(request.patch_out.as_deref(), Some("patch.diff"));
        assert_eq!(request.receipt_out.as_deref(), Some("receipt.ndjson"));
    }

    #[test]
    fn graph_plan_patch_request_requires_graph_and_ops() {
        assert!(GraphPlanPatchRequest::parse(&json!({"ops": "ops.ndjson"})).is_err());
        assert!(GraphPlanPatchRequest::parse(&json!({"graph_contract": "graph.ndjson"})).is_err());
    }

    #[test]
    fn graph_apply_ops_request_accepts_required_fields() {
        let request = GraphApplyOpsRequest::parse(&json!({
            "graph": "state/rustc/ai/graph.json",
            "ops": "ops.ndjson",
            "worktree_out": "tmp/worktree",
            "graph_out": "tmp/graph.json",
            "patch_out": "tmp/patch.diff",
            "apply_patch_out": "tmp/apply.patch",
            "apply_to_source": true,
            "validate_command": "cargo check",
            "artifact_root": "state/rustc"
        }))
        .expect("valid request");
        assert_eq!(request.graph_path, "state/rustc/ai/graph.json");
        assert_eq!(request.ops_path, "ops.ndjson");
        assert_eq!(request.worktree_out, "tmp/worktree");
        assert_eq!(request.graph_out.as_deref(), Some("tmp/graph.json"));
        assert_eq!(request.patch_out.as_deref(), Some("tmp/patch.diff"));
        assert_eq!(request.apply_patch_out.as_deref(), Some("tmp/apply.patch"));
        assert!(request.apply_to_source);
        assert_eq!(request.validate_command.as_deref(), Some("cargo check"));
        assert_eq!(request.artifact_root.as_deref(), Some("state/rustc"));
    }

    #[test]
    fn validate_relative_render_path_rejects_escape() {
        assert!(validate_relative_render_path("../x.rs").is_err());
        assert!(validate_relative_render_path("/tmp/x.rs").is_err());
        assert!(validate_relative_render_path("ai/src/lib.rs").is_ok());
    }

    #[test]
    fn apply_patch_text_from_unified_diff_wraps_update_file_hunks() {
        let diff = "--- a/ai/src/lib.rs\n+++ b/ai/src/lib.rs\n@@ -1 +1 @@\n-old\n+new\n";
        let patch = apply_patch_text_from_unified_diff(diff).expect("apply patch text");
        assert!(patch.starts_with("*** Begin Patch\n"));
        assert!(patch.contains("*** Update File: ai/src/lib.rs\n"));
        assert!(patch.contains("@@ -1 +1 @@\n-old\n+new\n"));
        assert!(patch.ends_with("*** End Patch\n"));
    }

    #[test]
    fn workspace_members_from_manifest_parses_workspace_members() {
        let members = workspace_members_from_manifest(
            "[workspace]\nmembers = [\"ai\", \"browser-router\", \"score\"]\n",
        );
        assert_eq!(members, vec!["ai", "browser-router", "score"]);
    }

    #[test]
    fn graph_plan_cfg_request_accepts_strategy_and_outputs() {
        let request = GraphPlanCfgRequest::parse(&json!({
            "graph": "state/rustc/ai/graph.json",
            "node": "crate::f",
            "strategy": "GuardClauseInsert",
            "guard": "if !ok { return Err(e); }",
            "ops_out": "tmp/ops.ndjson",
            "lo": 1,
            "hi": 9
        }))
        .expect("valid plan cfg request");
        assert_eq!(request.graph_path, "state/rustc/ai/graph.json");
        assert_eq!(request.node, "crate::f");
        assert_eq!(request.strategy, "GuardClauseInsert");
        assert_eq!(request.ops_out.as_deref(), Some("tmp/ops.ndjson"));
        assert_eq!(request.lo, Some(1));
        assert_eq!(request.hi, Some(9));
    }

    #[test]
    fn graph_verify_cfg_delta_request_accepts_policy_fields() {
        let request = GraphVerifyCfgDeltaRequest::parse(&json!({
            "old_graph": "old.json",
            "new_graph": "new.json",
            "node": "crate::f",
            "max_complexity_increase": 0,
            "require_changed": true
        }))
        .expect("valid verify request");
        assert_eq!(request.old_graph_path, "old.json");
        assert_eq!(request.new_graph_path, "new.json");
        assert_eq!(request.node, "crate::f");
        assert_eq!(request.max_complexity_increase, Some(0));
        assert!(request.require_changed);
    }

    #[test]
    fn graph_auto_refactor_cfg_request_requires_outputs() {
        let request = GraphAutoRefactorCfgRequest::parse(&json!({
            "graph": "state/rustc/ai/graph.json",
            "node": "crate::f",
            "strategy": "ReplaceSpan",
            "replacement": "fn f() {}",
            "ops_out": "tmp/cfg.ops.ndjson",
            "worktree_out": "tmp/worktree",
            "apply_to_source": true
        }))
        .expect("valid auto refactor request");
        assert_eq!(request.ops_out, "tmp/cfg.ops.ndjson");
        assert_eq!(request.worktree_out, "tmp/worktree");
        assert!(request.apply_to_source);
    }
}
