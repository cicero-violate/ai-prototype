//! `canon_plan_read` / `canon_plan_update` MCP tools.
//!
//! The plan DAG is stored at `state/plan.json` inside the workspace root. Read
//! responses derive readiness from the kernel-visible `PlanState` projection;
//! before an imported kernel projection is available, the persisted plan is the
//! explicit fallback source used to build that projection.
//!
//! Schema:
//!   { "version": 1,
//!     "nodes": [ { "id", "title", "description", "status", "assignee",
//!                  "score_axes", "files", "evidence" } ],
//!     "edges": [ { "from", "to" } ]  }
//!
//! `status` values: "pending" | "running" | "done" | "failed" | "skipped"
//!
//! `canon_plan_read`   — returns the current plan (or empty plan if none exists).
//! `canon_plan_update` — accepts one operation:
//!     op="replace"       full plan replacement (nodes + edges arrays required)
//!     op="set_status"    { node_id, status }
//!     op="set_assignee"  { node_id, assignee }
//!     op="upsert_node"   { node: { id, title, ... } }
//!     op="append_evidence" { node_id, evidence:
//!       { path, kind, summary, gate, evidence, receipt_hash, accepted } }
//!     op="add_edge"      { edge: { from, to } }
//!     op="remove_node"   { node_id }

use crate::domain::plan::{
    load_plan, ready_nodes_from_available_plan_state, save_plan, validate_plan_patch_mutation,
    NodeStatus, PlanDag, PlanEdge, PlanEvidenceRef, PlanNode,
};
use crate::process::scheduler::plan_store::{
    append_evidence_patch, append_status_change_patch, load_plan_read_model,
};
use crate::runtime::WorkspaceView;
use serde_json::{json, Value};

pub const CANON_PLAN_READ_TOOL: &str = "canon_plan_read";
pub const CANON_PLAN_UPDATE_TOOL: &str = "canon_plan_update";

// ── MCP tool handlers ─────────────────────────────────────────────────────────

pub fn run_read(_args: &Value, workspace: &WorkspaceView) -> Value {
    let (plan, plan_state) = load_plan_read_model(&workspace.root).unwrap_or_else(|_| (load_plan(&workspace.root), None));
    let ready: Vec<&str> = ready_nodes_from_available_plan_state(&plan, plan_state.as_ref())
        .iter()
        .map(|n| n.id.as_str())
        .collect();
    ok(json!({ "plan": plan, "ready_node_ids": ready }))
}

pub fn run_update(args: &Value, workspace: &WorkspaceView) -> Value {
    let op = match args.get("op").and_then(Value::as_str) {
        Some(o) => o,
        None => return error("canon_plan_update requires 'op'"),
    };

    let mut plan = load_plan(&workspace.root);
    let mut status_patch: Option<(String, NodeStatus)> = None;
    let mut evidence_patch: Option<(String, PlanEvidenceRef)> = None;

    match op {
        "replace" => {
            match serde_json::from_value::<PlanDag>(args.get("plan").cloned().unwrap_or(args.clone())) {
                Ok(new_plan) => plan = new_plan,
                Err(e) => return error(format!("invalid plan structure: {e}")),
            }
        }
        "set_status" => {
            let node_id = match args.get("node_id").and_then(Value::as_str) {
                Some(id) => id,
                None => return error("set_status requires 'node_id'"),
            };
            let status_str = match args.get("status").and_then(Value::as_str) {
                Some(s) => s,
                None => return error("set_status requires 'status'"),
            };
            let status = match NodeStatus::parse_name(status_str) {
                Some(s) => s,
                None => return error(format!("unknown status '{status_str}'; valid: pending, running, done, failed, skipped")),
            };
            match plan.nodes.iter_mut().find(|n| n.id == node_id) {
                Some(node) => {
                    node.status = status.clone();
                    status_patch = Some((node_id.to_string(), status));
                }
                None => return error(format!("node '{node_id}' not found")),
            }
        }
        "set_assignee" => {
            let node_id = match args.get("node_id").and_then(Value::as_str) {
                Some(id) => id,
                None => return error("set_assignee requires 'node_id'"),
            };
            let assignee = args.get("assignee").and_then(Value::as_str).map(str::to_string);
            match plan.nodes.iter_mut().find(|n| n.id == node_id) {
                Some(node) => node.assignee = assignee,
                None => return error(format!("node '{node_id}' not found")),
            }
        }
        "upsert_node" => {
            let node_val = match args.get("node") {
                Some(v) => v.clone(),
                None => return error("upsert_node requires 'node'"),
            };
            match serde_json::from_value::<PlanNode>(node_val) {
                Ok(new_node) => {
                    if let Some(existing) = plan.nodes.iter_mut().find(|n| n.id == new_node.id) {
                        *existing = new_node;
                    } else {
                        plan.nodes.push(new_node);
                    }
                }
                Err(e) => return error(format!("invalid node: {e}")),
            }
        }
        "append_evidence" => {
            let node_id = match args.get("node_id").and_then(Value::as_str) {
                Some(id) => id,
                None => return error("append_evidence requires 'node_id'"),
            };
            let evidence_val = match args.get("evidence") {
                Some(v) => v.clone(),
                None => return error("append_evidence requires 'evidence'"),
            };
            let evidence = match serde_json::from_value::<PlanEvidenceRef>(evidence_val) {
                Ok(evidence) => evidence,
                Err(e) => return error(format!("invalid evidence: {e}")),
            };
            if evidence.path.trim().is_empty()
                || evidence.kind.trim().is_empty()
                || evidence.summary.trim().is_empty()
            {
                return error("append_evidence requires non-empty evidence.path, evidence.kind, and evidence.summary");
            }
            match plan.nodes.iter_mut().find(|n| n.id == node_id) {
                Some(node) => {
                    if !node.evidence.iter().any(|existing| {
                        existing.path == evidence.path && existing.kind == evidence.kind
                    }) {
                        node.evidence.push(evidence.clone());
                    }
                    evidence_patch = Some((node_id.to_string(), evidence));
                }
                None => return error(format!("node '{node_id}' not found")),
            }
        }
        "add_edge" => {
            let edge_val = match args.get("edge") {
                Some(v) => v.clone(),
                None => return error("add_edge requires 'edge'"),
            };
            match serde_json::from_value::<PlanEdge>(edge_val) {
                Ok(edge) => {
                    plan.edges.push(edge);
                }
                Err(e) => return error(format!("invalid edge: {e}")),
            }
        }
        "remove_node" => {
            let node_id = match args.get("node_id").and_then(Value::as_str) {
                Some(id) => id,
                None => return error("remove_node requires 'node_id'"),
            };
            plan.nodes.retain(|n| n.id != node_id);
            plan.edges.retain(|e| e.from != node_id && e.to != node_id);
        }
        other => return error(format!("unknown op '{other}'; valid: replace, set_status, set_assignee, upsert_node, append_evidence, add_edge, remove_node")),
    }

    if let Err(e) = validate_plan_patch_mutation(&plan) {
        return error(format!("invalid plan mutation [{}]: {}", e.code, e.message));
    }

    if let Err(e) = save_plan(&workspace.root, &plan) {
        return error(format!("failed to save plan: {e}"));
    }

    if let Some((node_id, status)) = status_patch {
        if let Err(e) = append_status_change_patch(&workspace.root, &node_id, &status) {
            return error(format!("failed to append plan status patch to TLog: {e}"));
        }
    }

    if let Some((node_id, evidence)) = evidence_patch {
        if let Err(e) = append_evidence_patch(
            &workspace.root,
            &node_id,
            &evidence.path,
            &evidence.kind,
            &evidence.summary,
        ) {
            return error(format!("failed to append plan evidence patch to TLog: {e}"));
        }
    }

    let (read_model, plan_state) = load_plan_read_model(&workspace.root).unwrap_or_else(|_| (plan.clone(), None));
    let ready: Vec<&str> = ready_nodes_from_available_plan_state(&read_model, plan_state.as_ref())
        .iter()
        .map(|n| n.id.as_str())
        .collect();
    ok(
        json!({ "ok": true, "op": op, "node_count": read_model.nodes.len(), "edge_count": read_model.edges.len(), "ready_node_ids": ready }),
    )
}

fn ok(payload: Value) -> Value {
    let text = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string());
    json!({ "content": [{ "type": "text", "text": text }], "isError": false })
}

fn error(msg: impl Into<String>) -> Value {
    json!({ "content": [{ "type": "text", "text": format!("Error: {}", msg.into()) }], "isError": true })
}
