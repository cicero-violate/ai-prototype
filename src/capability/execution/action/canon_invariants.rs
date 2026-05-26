//! `canon_invariants_*` project action tools.

use serde_json::{json, Value};

use crate::api::action::tool_error;
use crate::runtime::WorkspaceView;
use crate::service::invariants::{
    load_registry, mine_and_write_workspace, promote_workspace, read_workspace_summary,
    validate_workspace,
};

pub const CANON_INVARIANTS_MINE_TOOL: &str = "canon_invariants_mine";
pub const CANON_INVARIANTS_VALIDATE_TOOL: &str = "canon_invariants_validate";
pub const CANON_INVARIANTS_PROMOTE_TOOL: &str = "canon_invariants_promote";
pub const CANON_INVARIANTS_READ_TOOL: &str = "canon_invariants_read";

pub fn run_mine(_args: &Value, workspace: &WorkspaceView) -> Value {
    match mine_and_write_workspace(&workspace.root) {
        Ok(candidates) => ok(json!({
            "candidate_count": candidates.len(),
            "candidates": candidates,
        })),
        Err(e) => tool_error(e),
    }
}

pub fn run_validate(args: &Value, workspace: &WorkspaceView) -> Value {
    let min_support = arg_u64(args, "min_support", 1);
    match validate_workspace(&workspace.root, min_support) {
        Ok(validations) => ok(json!({
            "validation_count": validations.len(),
            "validations": validations,
        })),
        Err(e) => tool_error(e),
    }
}

pub fn run_promote(args: &Value, workspace: &WorkspaceView) -> Value {
    let min_support = arg_u64(args, "min_support", 1);
    match promote_workspace(&workspace.root, min_support) {
        Ok(promoted) => ok(json!({
            "promoted_count": promoted.len(),
            "promoted": promoted,
        })),
        Err(e) => tool_error(e),
    }
}

pub fn run_read(args: &Value, workspace: &WorkspaceView) -> Value {
    let limit = arg_u64(args, "limit", 20).min(100) as usize;
    match read_workspace_summary(&workspace.root, limit) {
        Ok(summary) => ok(json!({
            "summary": summary,
            "registry": load_registry(&workspace.root).ok(),
        })),
        Err(e) => tool_error(e),
    }
}

fn arg_u64(args: &Value, key: &str, default: u64) -> u64 {
    args.get(key).and_then(Value::as_u64).unwrap_or(default)
}

fn ok(payload: Value) -> Value {
    let text = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string());
    json!({ "content": [{ "type": "text", "text": text }], "isError": false })
}
