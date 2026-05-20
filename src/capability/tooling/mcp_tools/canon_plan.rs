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
//!     op="append_evidence" { node_id, evidence: { path, kind, summary } }
//!     op="add_edge"      { edge: { from, to } }
//!     op="remove_node"   { node_id }

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::kernel::{
    PlanEdgeProjection, PlanEvidenceProjection, PlanNodeProjection, PlanState,
};
use crate::runtime::WorkspaceView;

const PROJECTED_STATUS_PENDING: u64 = 1;
const PROJECTED_STATUS_DONE: u64 = 3;

pub const CANON_PLAN_READ_TOOL: &str = "canon_plan_read";
pub const CANON_PLAN_UPDATE_TOOL: &str = "canon_plan_update";

const PLAN_FILE: &str = "state/plan.json";

// ── Schema ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlanDag {
    pub version: u32,
    pub nodes: Vec<PlanNode>,
    pub edges: Vec<PlanEdge>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanNode {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: NodeStatus,
    #[serde(default)]
    pub assignee: Option<String>,
    #[serde(default)]
    pub score_axes: Vec<String>,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub evidence: Vec<PlanEvidenceRef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanEvidenceRef {
    pub path: String,
    pub kind: String,
    pub summary: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanEdge {
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlanValidationError {
    pub code: &'static str,
    pub message: String,
}

impl PlanValidationError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    #[default]
    Pending,
    Running,
    Done,
    Failed,
    Skipped,
}

impl NodeStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "running" => Some(Self::Running),
            "done" => Some(Self::Done),
            "failed" => Some(Self::Failed),
            "skipped" => Some(Self::Skipped),
            _ => None,
        }
    }
}

// ── Persistence ───────────────────────────────────────────────────────────────

pub fn plan_path(workspace_root: &Path) -> std::path::PathBuf {
    workspace_root.join(PLAN_FILE)
}

pub fn load_plan(workspace_root: &Path) -> PlanDag {
    let path = plan_path(workspace_root);
    let Ok(bytes) = std::fs::read(&path) else {
        return PlanDag {
            version: 1,
            ..Default::default()
        };
    };
    serde_json::from_slice(&bytes).unwrap_or_else(|_| PlanDag {
        version: 1,
        ..Default::default()
    })
}

pub fn save_plan(workspace_root: &Path, plan: &PlanDag) -> Result<(), String> {
    let path = plan_path(workspace_root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(plan).map_err(|e| e.to_string())?;
    std::fs::write(&path, bytes).map_err(|e| e.to_string())
}

/// Returns nodes whose dependencies are all `Done`, and whose own status is `Pending`.
pub fn ready_nodes(plan: &PlanDag) -> Vec<&PlanNode> {
    let done_ids: std::collections::HashSet<&str> = plan
        .nodes
        .iter()
        .filter(|n| n.status == NodeStatus::Done)
        .map(|n| n.id.as_str())
        .collect();

    plan.nodes
        .iter()
        .filter(|node| {
            node.status == NodeStatus::Pending
                && plan
                    .edges
                    .iter()
                    .filter(|e| e.to == node.id)
                    .all(|e| done_ids.contains(e.from.as_str()))
        })
        .collect()
}

/// Build the kernel-visible PlanState projection for a persisted plan DAG.
pub fn plan_state_projection(plan: &PlanDag) -> PlanState {
    let mut projection = PlanState::default();
    projection.revision = u64::from(plan.version);

    for node in &plan.nodes {
        let node_id_hash = plan_text_hash(&node.id);
        let evidence = node
            .evidence
            .iter()
            .map(|evidence| PlanEvidenceProjection {
                path_hash: plan_text_hash(&evidence.path),
                kind_hash: plan_text_hash(&evidence.kind),
                summary_hash: plan_text_hash(&evidence.summary),
            })
            .collect();

        projection.nodes.insert(
            node_id_hash,
            PlanNodeProjection {
                node_id_hash,
                title_hash: plan_text_hash(&node.title),
                description_hash: plan_text_hash(&node.description),
                status: node_status_projection(&node.status),
                assignee_hash: node
                    .assignee
                    .as_deref()
                    .map(plan_text_hash)
                    .unwrap_or_default(),
                score_axes_hash: plan_text_list_hash(&node.score_axes),
                files_hash: plan_text_list_hash(&node.files),
                evidence,
            },
        );
    }

    for edge in &plan.edges {
        projection.edges.insert(PlanEdgeProjection {
            from_node_hash: plan_text_hash(&edge.from),
            to_node_hash: plan_text_hash(&edge.to),
        });
    }

    projection.imported_nodes_hash = plan_projected_nodes_hash(&projection);
    projection.imported_edges_hash = plan_projected_edges_hash(&projection);
    projection
}

/// Select ready nodes from the kernel PlanState projection in plan-node order.
pub fn ready_nodes_from_plan_state(plan: &PlanDag) -> Vec<&PlanNode> {
    let projection = plan_state_projection(plan);

    plan.nodes
        .iter()
        .filter(|node| {
            let node_id_hash = plan_text_hash(&node.id);
            let Some(projected_node) = projection.nodes.get(&node_id_hash) else {
                return false;
            };

            projected_node.status == PROJECTED_STATUS_PENDING
                && projection
                    .edges
                    .iter()
                    .filter(|edge| edge.to_node_hash == node_id_hash)
                    .all(|edge| {
                        projection
                            .nodes
                            .get(&edge.from_node_hash)
                            .is_some_and(|dependency| dependency.status == PROJECTED_STATUS_DONE)
                    })
        })
        .collect()
}

fn node_status_projection(status: &NodeStatus) -> u64 {
    match status {
        NodeStatus::Pending => PROJECTED_STATUS_PENDING,
        NodeStatus::Running => 2,
        NodeStatus::Done => PROJECTED_STATUS_DONE,
        NodeStatus::Failed => 4,
        NodeStatus::Skipped => 5,
    }
}

fn plan_text_hash(value: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash.max(1)
}

fn plan_text_list_hash(values: &[String]) -> u64 {
    values.iter().fold(0x504c_414e_4c49_5354u64, |hash, value| {
        hash.wrapping_mul(0x0000_0100_0000_01b3) ^ plan_text_hash(value)
    })
}

fn plan_projected_nodes_hash(projection: &PlanState) -> u64 {
    projection
        .nodes
        .values()
        .fold(0x504c_414e_4e4f_4445u64, |hash, node| {
            hash.wrapping_mul(0x0000_0100_0000_01b3)
                ^ node.node_id_hash
                ^ node.title_hash
                ^ node.description_hash
                ^ node.status
                ^ node.assignee_hash
                ^ node.score_axes_hash
                ^ node.files_hash
        })
        .max(1)
}

fn plan_projected_edges_hash(projection: &PlanState) -> u64 {
    projection
        .edges
        .iter()
        .fold(0x504c_414e_4544_4745u64, |hash, edge| {
            hash.wrapping_mul(0x0000_0100_0000_01b3) ^ edge.from_node_hash ^ edge.to_node_hash
        })
        .max(1)
}

/// Pure, deterministic validation for plan mutations before kernel acceptance.
///
/// This function only inspects the proposed `PlanDag`. It performs no I/O and
/// uses ordered iteration over the stored vectors so rejection reasons are
/// replay-safe.
pub fn validate_plan_patch_mutation(plan: &PlanDag) -> Result<(), PlanValidationError> {
    let mut node_ids = HashSet::new();
    let mut node_index = HashMap::new();

    for (idx, node) in plan.nodes.iter().enumerate() {
        validate_non_empty_field("node.id", &node.id)?;
        validate_non_empty_field("node.title", &node.title)?;
        validate_known_status(&node.status)?;

        if !node_ids.insert(node.id.as_str()) {
            return Err(PlanValidationError::new(
                "duplicate_node",
                format!("duplicate node id '{}'", node.id),
            ));
        }
        node_index.insert(node.id.as_str(), idx);

        for path in &node.files {
            validate_relative_path("node.files", path)?;
        }

        for evidence in &node.evidence {
            validate_non_empty_field("evidence.path", &evidence.path)?;
            validate_non_empty_field("evidence.kind", &evidence.kind)?;
            validate_non_empty_field("evidence.summary", &evidence.summary)?;
            validate_relative_path("evidence.path", &evidence.path)?;
        }
    }

    let mut edges = HashSet::new();
    for edge in &plan.edges {
        validate_non_empty_field("edge.from", &edge.from)?;
        validate_non_empty_field("edge.to", &edge.to)?;

        if !node_ids.contains(edge.from.as_str()) {
            return Err(PlanValidationError::new(
                "unknown_dependency",
                format!(
                    "edge.from '{}' does not reference an existing node",
                    edge.from
                ),
            ));
        }
        if !node_ids.contains(edge.to.as_str()) {
            return Err(PlanValidationError::new(
                "unknown_dependency",
                format!("edge.to '{}' does not reference an existing node", edge.to),
            ));
        }
        if !edges.insert((edge.from.as_str(), edge.to.as_str())) {
            return Err(PlanValidationError::new(
                "duplicate_edge",
                format!("duplicate edge '{} -> {}'", edge.from, edge.to),
            ));
        }
    }

    validate_acyclic(plan, &node_index)
}

fn validate_non_empty_field(field: &'static str, value: &str) -> Result<(), PlanValidationError> {
    if value.trim().is_empty() {
        Err(PlanValidationError::new(
            "missing_required_field",
            format!("{field} must be present and non-empty"),
        ))
    } else {
        Ok(())
    }
}

fn validate_known_status(status: &NodeStatus) -> Result<(), PlanValidationError> {
    match status {
        NodeStatus::Pending
        | NodeStatus::Running
        | NodeStatus::Done
        | NodeStatus::Failed
        | NodeStatus::Skipped => Ok(()),
    }
}

fn validate_relative_path(
    field: &'static str,
    path_value: &str,
) -> Result<(), PlanValidationError> {
    validate_non_empty_field(field, path_value)?;

    let path = Path::new(path_value);
    if path.is_absolute() {
        return Err(PlanValidationError::new(
            "invalid_path",
            format!("{field} must be workspace-relative: '{path_value}'"),
        ));
    }

    if path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(PlanValidationError::new(
            "invalid_path",
            format!("{field} must not escape the workspace: '{path_value}'"),
        ));
    }

    Ok(())
}

fn validate_acyclic(
    plan: &PlanDag,
    node_index: &HashMap<&str, usize>,
) -> Result<(), PlanValidationError> {
    let mut outgoing = vec![Vec::new(); plan.nodes.len()];
    for edge in &plan.edges {
        let from = node_index[edge.from.as_str()];
        let to = node_index[edge.to.as_str()];
        outgoing[from].push(to);
    }

    let mut state = vec![0u8; plan.nodes.len()];
    for idx in 0..plan.nodes.len() {
        if state[idx] == 0 {
            visit_acyclic(idx, &outgoing, &mut state, plan)?;
        }
    }
    Ok(())
}

fn visit_acyclic(
    idx: usize,
    outgoing: &[Vec<usize>],
    state: &mut [u8],
    plan: &PlanDag,
) -> Result<(), PlanValidationError> {
    state[idx] = 1;
    for &next in &outgoing[idx] {
        match state[next] {
            0 => visit_acyclic(next, outgoing, state, plan)?,
            1 => {
                return Err(PlanValidationError::new(
                    "dependency_cycle",
                    format!(
                        "dependency cycle includes '{}' and '{}'",
                        plan.nodes[idx].id, plan.nodes[next].id
                    ),
                ));
            }
            _ => {}
        }
    }
    state[idx] = 2;
    Ok(())
}

// ── MCP tool handlers ─────────────────────────────────────────────────────────

pub fn run_read(_args: &Value, workspace: &WorkspaceView) -> Value {
    let plan = load_plan(&workspace.root);
    let ready: Vec<&str> = ready_nodes_from_plan_state(&plan).iter().map(|n| n.id.as_str()).collect();
    ok(json!({ "plan": plan, "ready_node_ids": ready }))
}

pub fn run_update(args: &Value, workspace: &WorkspaceView) -> Value {
    let op = match args.get("op").and_then(Value::as_str) {
        Some(o) => o,
        None => return error("canon_plan_update requires 'op'"),
    };

    let mut plan = load_plan(&workspace.root);

    match op {
        "replace" => {
            match serde_json::from_value::<PlanDag>(args.get("plan").cloned().unwrap_or(args.clone())) {
                Ok(new_plan) => plan = PlanDag { version: 1, ..new_plan },
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
            let status = match NodeStatus::from_str(status_str) {
                Some(s) => s,
                None => return error(format!("unknown status '{status_str}'; valid: pending, running, done, failed, skipped")),
            };
            match plan.nodes.iter_mut().find(|n| n.id == node_id) {
                Some(node) => node.status = status,
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
                        node.evidence.push(evidence);
                    }
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

    let ready: Vec<&str> = ready_nodes_from_plan_state(&plan).iter().map(|n| n.id.as_str()).collect();
    ok(
        json!({ "ok": true, "op": op, "node_count": plan.nodes.len(), "edge_count": plan.edges.len(), "ready_node_ids": ready }),
    )
}

fn ok(payload: Value) -> Value {
    let text = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string());
    json!({ "content": [{ "type": "text", "text": text }], "isError": false })
}

fn error(msg: impl Into<String>) -> Value {
    json!({ "content": [{ "type": "text", "text": format!("Error: {}", msg.into()) }], "isError": true })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str) -> PlanNode {
        PlanNode {
            id: id.to_string(),
            title: format!("node {id}"),
            description: String::new(),
            status: NodeStatus::Pending,
            assignee: None,
            score_axes: Vec::new(),
            files: vec![format!("ai/src/{id}.rs")],
            evidence: vec![PlanEvidenceRef {
                path: format!("state/agent-evidence/{id}.md"),
                kind: "validation".to_string(),
                summary: "evidence summary".to_string(),
            }],
        }
    }

    fn dag(edges: Vec<PlanEdge>) -> PlanDag {
        PlanDag {
            version: 1,
            nodes: vec![node("a"), node("b"), node("c")],
            edges,
        }
    }

    fn edge(from: &str, to: &str) -> PlanEdge {
        PlanEdge {
            from: from.to_string(),
            to: to.to_string(),
        }
    }

    #[test]
    fn validates_well_formed_patch_plan() {
        let plan = dag(vec![edge("a", "b"), edge("b", "c")]);

        assert_eq!(validate_plan_patch_mutation(&plan), Ok(()));
    }

    #[test]
    fn rejects_duplicate_edges() {
        let plan = dag(vec![edge("a", "b"), edge("a", "b")]);

        let err = validate_plan_patch_mutation(&plan).unwrap_err();

        assert_eq!(err.code, "duplicate_edge");
    }

    #[test]
    fn rejects_unknown_dependency_endpoint() {
        let plan = dag(vec![edge("missing", "b")]);

        let err = validate_plan_patch_mutation(&plan).unwrap_err();

        assert_eq!(err.code, "unknown_dependency");
    }

    #[test]
    fn rejects_dependency_cycles() {
        let plan = dag(vec![edge("a", "b"), edge("b", "c"), edge("c", "a")]);

        let err = validate_plan_patch_mutation(&plan).unwrap_err();

        assert_eq!(err.code, "dependency_cycle");
    }

    #[test]
    fn rejects_invalid_file_and_evidence_paths() {
        let mut plan = dag(Vec::new());
        plan.nodes[0].files = vec!["../outside.rs".to_string()];

        let err = validate_plan_patch_mutation(&plan).unwrap_err();

        assert_eq!(err.code, "invalid_path");

        let mut plan = dag(Vec::new());
        plan.nodes[0].evidence[0].path.clear();

        let err = validate_plan_patch_mutation(&plan).unwrap_err();

        assert_eq!(err.code, "missing_required_field");
    }

    #[test]
    fn plan_state_projection_returns_pending_nodes_with_done_dependencies() {
        let mut plan = dag(vec![edge("a", "b"), edge("b", "c")]);
        plan.nodes[0].status = NodeStatus::Done;
        plan.nodes[1].status = NodeStatus::Pending;
        plan.nodes[2].status = NodeStatus::Pending;

        let ready = ready_nodes_from_plan_state(&plan);

        assert_eq!(
            ready.iter().map(|node| node.id.as_str()).collect::<Vec<_>>(),
            vec!["b"]
        );
    }

    #[test]
    fn plan_state_projection_does_not_return_blocked_pending_nodes() {
        let plan = dag(vec![edge("a", "b"), edge("b", "c")]);

        let ready = ready_nodes_from_plan_state(&plan);

        assert_eq!(
            ready.iter().map(|node| node.id.as_str()).collect::<Vec<_>>(),
            vec!["a"]
        );
        assert!(!ready.iter().any(|node| node.id == "b" || node.id == "c"));
    }

    #[test]
    fn plan_state_projection_preserves_no_ready_when_all_pending_nodes_are_blocked() {
        let mut plan = dag(vec![edge("a", "b"), edge("b", "c")]);
        plan.nodes[0].status = NodeStatus::Running;

        let ready = ready_nodes_from_plan_state(&plan);

        assert!(ready.is_empty());
    }
}
