//! `canon_plan_read` / `canon_plan_update` MCP tools.
//!
//! The plan DAG scaffold is stored at `state/plan.json` inside the workspace
//! root. Read responses derive readiness and lifecycle state from the
//! kernel-visible `PlanState` projection; before an imported kernel projection
//! is available, the persisted plan is the explicit fallback source used to
//! build that projection.
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
//!     op="set_status"    { node_id, status } appends a TLog plan patch
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
    append_assignee_change_patch, append_edge_add_patch, append_edge_remove_patch,
    append_evidence_patch, append_node_remove_patch, append_node_upsert_patch,
    append_status_change_patch, load_plan_read_model,
};
use crate::runtime::WorkspaceView;
use serde_json::{json, Value};

pub const CANON_PLAN_READ_TOOL: &str = "canon_plan_read";
pub const CANON_PLAN_UPDATE_TOOL: &str = "canon_plan_update";

// ── MCP tool handlers ─────────────────────────────────────────────────────────

pub fn run_read(_args: &Value, workspace: &WorkspaceView) -> Value {
    let (plan, plan_state) = load_plan_read_model(&workspace.root)
        .unwrap_or_else(|_| (load_plan(&workspace.root), None));
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
    let old_plan = plan.clone();
    let mut plan_changed = false;
    let mut status_patch: Option<(String, NodeStatus)> = None;
    let mut evidence_patch: Option<(String, PlanEvidenceRef)> = None;
    let mut assignee_patch: Option<(String, Option<String>)> = None;
    let mut node_upserts: Vec<PlanNode> = Vec::new();
    let mut edge_adds: Vec<PlanEdge> = Vec::new();
    let mut node_removes: Vec<String> = Vec::new();
    let mut edge_removes: Vec<PlanEdge> = Vec::new();

    match op {
        "replace" => {
            match serde_json::from_value::<PlanDag>(args.get("plan").cloned().unwrap_or(args.clone())) {
                Ok(new_plan) => {
                    plan = new_plan;
                    node_upserts = plan.nodes.clone();
                    edge_adds = plan.edges.clone();
                    node_removes = old_plan
                        .nodes
                        .iter()
                        .filter(|old_node| !plan.nodes.iter().any(|node| node.id == old_node.id))
                        .map(|node| node.id.clone())
                        .collect();
                    edge_removes = old_plan
                        .edges
                        .iter()
                        .filter(|old_edge| !plan_edge_exists(&plan.edges, old_edge))
                        .cloned()
                        .collect();
                    plan_changed = true;
                }
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
            if plan.nodes.iter().any(|n| n.id == node_id) {
                status_patch = Some((node_id.to_string(), status));
            } else {
                return error(format!("node '{node_id}' not found"));
            }
        }
        "set_assignee" => {
            let node_id = match args.get("node_id").and_then(Value::as_str) {
                Some(id) => id,
                None => return error("set_assignee requires 'node_id'"),
            };
            let assignee = args.get("assignee").and_then(Value::as_str).map(str::to_string);
            match plan.nodes.iter_mut().find(|n| n.id == node_id) {
                Some(node) => {
                    node.assignee = assignee;
                    assignee_patch = Some((node_id.to_string(), node.assignee.clone()));
                    plan_changed = true;
                }
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
                        node_upserts.push(existing.clone());
                    } else {
                        node_upserts.push(new_node.clone());
                        plan.nodes.push(new_node);
                    }
                    plan_changed = true;
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
                Some(_) => {
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
                    if !plan_edge_exists(&plan.edges, &edge) {
                        edge_adds.push(edge.clone());
                        plan.edges.push(edge);
                    }
                    plan_changed = true;
                }
                Err(e) => return error(format!("invalid edge: {e}")),
            }
        }
        "remove_node" => {
            let node_id = match args.get("node_id").and_then(Value::as_str) {
                Some(id) => id,
                None => return error("remove_node requires 'node_id'"),
            };
            for edge in plan.edges.iter().filter(|edge| edge.from == node_id || edge.to == node_id)
            {
                edge_removes.push(edge.clone());
            }
            plan.nodes.retain(|n| n.id != node_id);
            plan.edges.retain(|e| e.from != node_id && e.to != node_id);
            node_removes.push(node_id.to_string());
            plan_changed = true;
        }
        other => return error(format!("unknown op '{other}'; valid: replace, set_status, set_assignee, upsert_node, append_evidence, add_edge, remove_node")),
    }

    if let Err(e) = validate_plan_patch_mutation(&plan) {
        return error(format!("invalid plan mutation [{}]: {}", e.code, e.message));
    }

    if plan_changed {
        if let Err(e) = save_plan(&workspace.root, &plan) {
            return error(format!("failed to save plan: {e}"));
        }
    }

    if let Some((node_id, status)) = status_patch {
        if let Err(e) = append_status_change_patch(&workspace.root, &node_id, &status) {
            return error(format!("failed to append plan status patch to TLog: {e}"));
        }
    }

    for node in &node_upserts {
        if let Err(e) = append_node_upsert_patch(&workspace.root, node) {
            return error(format!(
                "failed to append plan node upsert patch to TLog: {e}"
            ));
        }
    }

    for edge in &edge_removes {
        if let Err(e) = append_edge_remove_patch(&workspace.root, edge) {
            return error(format!(
                "failed to append plan edge remove patch to TLog: {e}"
            ));
        }
    }

    for node_id in &node_removes {
        if let Err(e) = append_node_remove_patch(&workspace.root, node_id) {
            return error(format!(
                "failed to append plan node remove patch to TLog: {e}"
            ));
        }
    }

    for edge in &edge_adds {
        if let Err(e) = append_edge_add_patch(&workspace.root, edge) {
            return error(format!("failed to append plan edge add patch to TLog: {e}"));
        }
    }

    if let Some((node_id, assignee)) = assignee_patch {
        if let Err(e) = append_assignee_change_patch(&workspace.root, &node_id, assignee.as_deref())
        {
            return error(format!("failed to append plan assignee patch to TLog: {e}"));
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

    let (read_model, plan_state) =
        load_plan_read_model(&workspace.root).unwrap_or_else(|_| (plan.clone(), None));
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

fn plan_edge_exists(edges: &[PlanEdge], needle: &PlanEdge) -> bool {
    edges
        .iter()
        .any(|edge| edge.from == needle.from && edge.to == needle.to)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::plan::plan_text_hash;
    use crate::process::scheduler::plan_store::load_tlog_projected_plan_state;
    use std::path::{Path, PathBuf};

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("canon-plan-tool-{name}-{}", std::process::id()))
    }

    fn node(id: &str, status: NodeStatus) -> PlanNode {
        PlanNode {
            id: id.to_string(),
            title: format!("{id} title"),
            description: format!("{id} description"),
            status,
            assignee: None,
            score_axes: Vec::new(),
            files: Vec::new(),
            evidence: Vec::new(),
        }
    }

    fn write_plan(root: &Path, status: NodeStatus) {
        save_plan(
            root,
            &PlanDag {
                version: 1,
                nodes: vec![node("node-1", status)],
                edges: Vec::new(),
                ..Default::default()
            },
        )
        .expect("plan should write");
    }

    #[test]
    fn set_status_appends_tlog_patch_without_mutating_plan_json_status() {
        let root = test_root("status-patch");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("root should exist");
        write_plan(&root, NodeStatus::Pending);
        let workspace = WorkspaceView::new(root.clone(), root.clone()).expect("workspace view");

        let result = run_update(
            &json!({"op": "set_status", "node_id": "node-1", "status": "done"}),
            &workspace,
        );
        assert_eq!(result.get("isError").and_then(Value::as_bool), Some(false));

        let raw_plan = load_plan(&root);
        assert_eq!(raw_plan.nodes[0].status, NodeStatus::Pending);

        let (read_model, plan_state) = load_plan_read_model(&root).expect("read model projects");
        assert!(plan_state.is_some());
        assert_eq!(read_model.nodes[0].status, NodeStatus::Done);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn append_evidence_appends_tlog_patch_without_mutating_plan_json_evidence() {
        let root = test_root("evidence-patch");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("root should exist");
        write_plan(&root, NodeStatus::Pending);
        let workspace = WorkspaceView::new(root.clone(), root.clone()).expect("workspace view");

        let result = run_update(
            &json!({
                "op": "append_evidence",
                "node_id": "node-1",
                "evidence": {
                    "path": "state/agent-evidence/node-1.md",
                    "kind": "validation",
                    "summary": "evidence is projected from TLog"
                }
            }),
            &workspace,
        );
        assert_eq!(result.get("isError").and_then(Value::as_bool), Some(false));

        let raw_plan = load_plan(&root);
        assert!(raw_plan.nodes[0].evidence.is_empty());

        let plan_state = load_tlog_projected_plan_state(&root)
            .expect("TLog projection should load")
            .expect("TLog evidence patch should project");
        let node = plan_state
            .nodes
            .get(&plan_text_hash("node-1"))
            .expect("node should project");
        assert_eq!(node.evidence.len(), 1);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn structural_updates_append_tlog_projection_patches() {
        let root = test_root("structural-patches");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("root should exist");
        let workspace = WorkspaceView::new(root.clone(), root.clone()).expect("workspace view");

        let upsert = run_update(
            &json!({
                "op": "upsert_node",
                "node": {
                    "id": "node-1",
                    "title": "Node one",
                    "description": "prove structural TLog projection",
                    "status": "pending",
                    "score_axes": ["Determinism"],
                    "files": ["ai/src/process/scheduler/plan_store.rs"]
                }
            }),
            &workspace,
        );
        assert_eq!(upsert.get("isError").and_then(Value::as_bool), Some(false));

        let status = run_update(
            &json!({"op": "set_status", "node_id": "node-1", "status": "done"}),
            &workspace,
        );
        assert_eq!(status.get("isError").and_then(Value::as_bool), Some(false));

        std::fs::remove_file(root.join("state/plan.json")).expect("plan scaffold should delete");
        let plan_state = load_tlog_projected_plan_state(&root)
            .expect("TLog projection should load")
            .expect("TLog structural patches should project without plan.json");
        let node = plan_state
            .nodes
            .get(&plan_text_hash("node-1"))
            .expect("node should project from TLog");
        assert_eq!(node.status, 3);
        assert_eq!(node.title_hash, plan_text_hash("Node one"));

        let _ = std::fs::remove_dir_all(root);
    }
}
