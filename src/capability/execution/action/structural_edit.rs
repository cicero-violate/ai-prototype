//! Workspace structural editing backed by the shared structural-editor crate.

use serde::Deserialize;
use serde_json::{json, Value};
use structural_editor::op::{
    CargoChange, EdgeKind, NodeLocator, OpBatch, StructuralOp, VerifyPredicate,
};

use crate::api::action::tool_error;
use crate::runtime::WorkspaceView;

pub const STRUCTURAL_EDIT_TOOL: &str = "structural_edit";

#[derive(Debug, Deserialize)]
struct StructuralEditRequest {
    #[serde(default = "default_mode")]
    mode: String,
    #[serde(default = "default_cwd")]
    cwd: String,
    #[serde(default)]
    label: Option<String>,
    ops: Vec<StructuralOp>,
}

pub fn run(args: &Value, workspace: &WorkspaceView) -> Value {
    match run_inner(args, workspace) {
        Ok(value) => value,
        Err(error) => tool_error(error),
    }
}

fn run_inner(args: &Value, workspace: &WorkspaceView) -> Result<Value, String> {
    let request: StructuralEditRequest = serde_json::from_value(args.clone())
        .map_err(|error| format!("invalid structural edit request: {error}"))?;
    if request.ops.is_empty() {
        return Err("structural_edit requires at least one op".to_string());
    }

    let mode = request.mode.as_str();
    if mode != "check" && mode != "apply" {
        return Err("mode must be 'check' or 'apply'".to_string());
    }

    let root = workspace.resolve_cwd(&request.cwd)?;
    let batch = OpBatch {
        label: request.label,
        ops: request.ops,
    };
    let touched_paths = touched_paths(&batch);

    if mode == "check" {
        let payload = json!({
            "ok": true,
            "mode": "check",
            "label": batch.label,
            "opCount": batch.ops.len(),
            "touchedPaths": touched_paths,
            "summary": format!("structural edit schema ok for {} op(s)", batch.ops.len()),
            "rejects": []
        });
        return Ok(success_response(payload));
    }

    let result = structural_editor::executor::apply(&batch, &root);
    let payload = json!({
        "ok": result.ok,
        "mode": "apply",
        "label": result.label,
        "opCount": batch.ops.len(),
        "touchedPaths": touched_paths,
        "deltas": result.deltas,
        "error": result.error,
        "rejects": if result.ok { json!([]) } else { json!(["structural_editor_apply_failed"]) }
    });

    Ok(json!({
        "content": [{ "type": "text", "text": payload.to_string() }],
        "isError": !payload.get("ok").and_then(Value::as_bool).unwrap_or(false)
    }))
}

fn success_response(payload: Value) -> Value {
    json!({
        "content": [{ "type": "text", "text": payload.to_string() }],
        "isError": false
    })
}

fn default_mode() -> String {
    "check".to_string()
}

fn default_cwd() -> String {
    ".".to_string()
}

fn touched_paths(batch: &OpBatch) -> Vec<String> {
    let mut paths = Vec::new();
    for op in &batch.ops {
        for path in touched_paths_for_op(op) {
            if !paths.iter().any(|existing| existing == &path) {
                paths.push(path);
            }
        }
    }
    paths
}

fn touched_paths_for_op(op: &StructuralOp) -> Vec<String> {
    match op {
        StructuralOp::CreateNode(op) => vec![locator_path(&op.at)],
        StructuralOp::DeleteNode(op) => vec![locator_path(&op.at)],
        StructuralOp::ReplaceNode(op) => vec![locator_path(&op.at)],
        StructuralOp::MoveNode(op) => vec![locator_path(&op.from), locator_path(&op.to)],
        StructuralOp::RenameSymbol(op) => {
            let mut paths = vec![locator_path(&op.at)];
            paths.extend(op.scope.iter().cloned());
            paths
        }
        StructuralOp::SetAttr(op) => vec![locator_path(&op.at)],
        StructuralOp::AddEdge(op) => vec![op.file.clone()],
        StructuralOp::RemoveEdge(op) => vec![op.file.clone()],
        StructuralOp::Verify(op) => vec![predicate_path(&op.predicate).to_string()],
        StructuralOp::Cargo(op) => vec![manifest_path(op).to_string()],
        StructuralOp::Receipt(op) => vec![op.receipt_path.clone()],
        StructuralOp::Rollback(op) => vec![op.rollback_path.clone()],
    }
}

fn locator_path(locator: &NodeLocator) -> String {
    match locator {
        NodeLocator::Anchor { path, .. } | NodeLocator::Selector { path, .. } => path.clone(),
    }
}

fn predicate_path(predicate: &VerifyPredicate) -> &str {
    match predicate {
        VerifyPredicate::FileExists { path }
        | VerifyPredicate::FileAbsent { path }
        | VerifyPredicate::ContainsText { path, .. }
        | VerifyPredicate::TextAbsent { path, .. }
        | VerifyPredicate::SymbolExists { path, .. } => path,
    }
}

fn manifest_path(change: &CargoChange) -> &str {
    match change {
        CargoChange::AddDependency { manifest, .. }
        | CargoChange::RemoveDependency { manifest, .. }
        | CargoChange::AddDevDependency { manifest, .. }
        | CargoChange::RemoveDevDependency { manifest, .. }
        | CargoChange::AddBuildDependency { manifest, .. }
        | CargoChange::AddFeature { manifest, .. }
        | CargoChange::RemoveFeature { manifest, .. }
        | CargoChange::SetPackageField { manifest, .. }
        | CargoChange::AddBinTarget { manifest, .. }
        | CargoChange::AddLibTarget { manifest, .. }
        | CargoChange::AddTestTarget { manifest, .. }
        | CargoChange::AddExampleTarget { manifest, .. }
        | CargoChange::RemoveTarget { manifest, .. }
        | CargoChange::InsertSnippet { manifest, .. } => manifest,
    }
}

#[allow(dead_code)]
fn edge_tokens(edge: &EdgeKind) -> Vec<&str> {
    match edge {
        EdgeKind::Uses { .. } => vec!["uses"],
        EdgeKind::Declares { .. } => vec!["declares"],
        EdgeKind::Implements { .. } => vec!["implements"],
        EdgeKind::Bound { .. } => vec!["bound"],
        EdgeKind::PathRef { .. } => vec!["path_ref"],
        EdgeKind::ExternCrate(_) => vec!["extern_crate"],
    }
}
