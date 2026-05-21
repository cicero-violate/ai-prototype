//! Plan DAG types, invariants, validation, and readiness logic.
//!
//! Neutral module: used by supervisor (S), scheduler, and capability tooling.
//! Import path: `crate::domain::plan`.

use std::collections::{HashMap, HashSet};
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

use crate::kernel::{PlanEdgeProjection, PlanEvidenceProjection, PlanNodeProjection, PlanState};

const PROJECTED_STATUS_PENDING: u64 = 1;
const PROJECTED_STATUS_DONE: u64 = 3;

// ── Evidence kind constants ───────────────────────────────────────────────────

/// Evidence written by a worker when it cannot make progress (no success criterion,
/// missing dependency, etc.). Causes the reconciler to promote the node to Failed.
pub const EVIDENCE_KIND_BLOCKER: &str = "blocker";

/// Evidence written by a worker after it completes its task successfully.
/// Used by the reconciler as a fallback when the completion callback was missed.
pub const EVIDENCE_KIND_EXECUTION_RECEIPT: &str = "execution_receipt";

/// Evidence written during plan validation to record structural plan checks.
pub const EVIDENCE_KIND_VALIDATION: &str = "validation";

/// Gate name used in accepted execution-receipt evidence entries.
pub const EVIDENCE_GATE_EXECUTION: &str = "Execution";

/// Evidence type name used in accepted execution-receipt evidence entries.
pub const EVIDENCE_TYPE_EXECUTION_RECEIPT: &str = "ExecutionReceipt";

// ── Schema ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlanDag {
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<PlanSummary>,
    pub nodes: Vec<PlanNode>,
    pub edges: Vec<PlanEdge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub archived_completed_node_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlanSummary {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub target_goal: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emphasis: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub scope: String,
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
    #[serde(default)]
    pub gate: String,
    #[serde(default)]
    pub evidence: String,
    #[serde(default)]
    pub receipt_hash: u64,
    #[serde(default)]
    pub accepted: bool,
}

impl PlanEvidenceRef {
    pub fn is_accepted_execution_receipt(&self) -> bool {
        self.accepted
            && self.receipt_hash != 0
            && self.gate == EVIDENCE_GATE_EXECUTION
            && self.evidence == EVIDENCE_TYPE_EXECUTION_RECEIPT
    }

    pub fn is_blocker(&self) -> bool {
        self.kind == EVIDENCE_KIND_BLOCKER
    }
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
    pub fn parse_name(s: &str) -> Option<Self> {
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
    crate::process::scheduler::plan_store::plan_path(workspace_root)
}

pub fn load_plan(workspace_root: &Path) -> PlanDag {
    crate::process::scheduler::plan_store::load_plan(workspace_root)
}

pub fn save_plan(workspace_root: &Path, plan: &PlanDag) -> Result<(), String> {
    crate::process::scheduler::plan_store::save_plan(workspace_root, plan)
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
    let mut projection = PlanState {
        revision: u64::from(plan.version),
        ..Default::default()
    };

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

    ready_nodes_from_plan_state_projection(plan, &projection)
}

pub fn ready_nodes_from_plan_state_projection<'a>(
    plan: &'a PlanDag,
    projection: &PlanState,
) -> Vec<&'a PlanNode> {
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

pub fn ready_nodes_from_available_plan_state<'a>(
    plan: &'a PlanDag,
    projection: Option<&PlanState>,
) -> Vec<&'a PlanNode> {
    match projection {
        Some(projection) => ready_nodes_from_plan_state_projection(plan, projection),
        None => ready_nodes_from_plan_state(plan),
    }
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

pub(crate) fn plan_text_hash(value: &str) -> u64 {
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
            if evidence.accepted {
                if evidence.receipt_hash == 0 {
                    return Err(PlanValidationError::new(
                        "invalid_evidence_receipt",
                        "accepted evidence requires non-zero receipt_hash",
                    ));
                }
                validate_known_evidence_gate(&evidence.gate)?;
                validate_known_evidence_type(&evidence.evidence)?;
            }
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

fn validate_known_evidence_gate(value: &str) -> Result<(), PlanValidationError> {
    match value {
        "Invariant" | "Analysis" | "Judgment" | "Plan" | "Execution" | "Verification" | "Eval"
        | "Learning" => Ok(()),
        _ => Err(PlanValidationError::new(
            "unknown_evidence_gate",
            format!("unknown evidence gate '{value}'"),
        )),
    }
}

fn validate_known_evidence_type(value: &str) -> Result<(), PlanValidationError> {
    match value {
        "InvariantProof" | "AnalysisReport" | "JudgmentRecord" | "PlanRecord" | "TaskReady"
        | "ExecutionReceipt" | "ArtifactReceipt" | "VerificationReport" | "LineageProof"
        | "EvalScore" | "PersistedRecord" => Ok(()),
        _ => Err(PlanValidationError::new(
            "unknown_evidence_type",
            format!("unknown evidence type '{value}'"),
        )),
    }
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
                kind: EVIDENCE_KIND_VALIDATION.to_string(),
                summary: "evidence summary".to_string(),
                gate: EVIDENCE_GATE_EXECUTION.to_string(),
                evidence: EVIDENCE_TYPE_EXECUTION_RECEIPT.to_string(),
                receipt_hash: 1,
                accepted: true,
            }],
        }
    }

    fn dag(edges: Vec<PlanEdge>) -> PlanDag {
        PlanDag {
            version: 1,
            nodes: vec![node("a"), node("b"), node("c")],
            edges,
            ..Default::default()
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

        let err = validate_plan_patch_mutation(&plan).expect_err("test should fail");

        assert_eq!(err.code, "duplicate_edge");
    }

    #[test]
    fn rejects_unknown_dependency_endpoint() {
        let plan = dag(vec![edge("missing", "b")]);

        let err = validate_plan_patch_mutation(&plan).expect_err("test should fail");

        assert_eq!(err.code, "unknown_dependency");
    }

    #[test]
    fn rejects_dependency_cycles() {
        let plan = dag(vec![edge("a", "b"), edge("b", "c"), edge("c", "a")]);

        let err = validate_plan_patch_mutation(&plan).expect_err("test should fail");

        assert_eq!(err.code, "dependency_cycle");
    }

    #[test]
    fn rejects_invalid_file_and_evidence_paths() {
        let mut plan = dag(Vec::new());
        plan.nodes[0].files = vec!["../outside.rs".to_string()];

        let err = validate_plan_patch_mutation(&plan).expect_err("test should fail");

        assert_eq!(err.code, "invalid_path");

        let mut plan = dag(Vec::new());
        plan.nodes[0].evidence[0].path.clear();

        let err = validate_plan_patch_mutation(&plan).expect_err("test should fail");

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
            ready
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            vec!["b"]
        );
    }

    #[test]
    fn plan_state_projection_task_next_readiness_contract() {
        let empty = PlanDag::default();
        assert!(ready_nodes_from_plan_state(&empty).is_empty());

        let mut plan = PlanDag {
            version: 1,
            nodes: vec![
                node("done-dependency"),
                node("running-node"),
                node("done-node"),
                node("failed-node"),
                node("skipped-node"),
                node("blocked-pending"),
                node("first-ready"),
                node("second-ready"),
            ],
            edges: vec![
                edge("failed-node", "blocked-pending"),
                edge("done-dependency", "first-ready"),
                edge("done-dependency", "second-ready"),
            ],
            ..Default::default()
        };
        plan.nodes[0].status = NodeStatus::Done;
        plan.nodes[1].status = NodeStatus::Running;
        plan.nodes[2].status = NodeStatus::Done;
        plan.nodes[3].status = NodeStatus::Failed;
        plan.nodes[4].status = NodeStatus::Skipped;

        let ready = ready_nodes_from_plan_state(&plan);

        assert_eq!(
            ready.first().map(|node| node.id.as_str()),
            Some("first-ready")
        );
        assert_eq!(
            ready
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            vec!["first-ready", "second-ready"]
        );
        assert!(!ready.iter().any(|node| matches!(
            node.status,
            NodeStatus::Running | NodeStatus::Done | NodeStatus::Failed | NodeStatus::Skipped
        )));
    }

    #[test]
    fn available_plan_state_projection_overrides_persisted_plan_status() {
        let plan = dag(vec![edge("a", "b")]);
        let mut projection = plan_state_projection(&plan);
        let a_hash = plan_text_hash("a");
        let b_hash = plan_text_hash("b");
        projection
            .nodes
            .get_mut(&a_hash)
            .expect("test value should be present")
            .status = PROJECTED_STATUS_DONE;
        projection
            .nodes
            .get_mut(&b_hash)
            .expect("test value should be present")
            .status = PROJECTED_STATUS_PENDING;

        let ready = ready_nodes_from_available_plan_state(&plan, Some(&projection));

        // "b" depends on "a" (Done in projection) — must be ready.
        // "a" is Done in projection — must not be ready.
        // "c" is a root node (no incoming edges) and is Pending — also ready.
        assert!(ready.iter().any(|n| n.id == "b"), "'b' should be ready");
        assert!(
            !ready.iter().any(|n| n.id == "a"),
            "'a' should not be ready (Done)"
        );
    }

    #[test]
    fn unavailable_plan_state_projection_falls_back_to_persisted_plan_projection() {
        let mut plan = dag(vec![edge("a", "b")]);
        plan.nodes[0].status = NodeStatus::Done;

        let ready = ready_nodes_from_available_plan_state(&plan, None);

        // With no kernel projection, falls back to the persisted plan.
        // "b" depends on "a" (Done in plan) — must be ready.
        // "a" is Done — must not be ready.
        // "c" is a root Pending node — also ready.
        assert!(ready.iter().any(|n| n.id == "b"), "'b' should be ready");
        assert!(
            !ready.iter().any(|n| n.id == "a"),
            "'a' should not be ready (Done)"
        );
    }

    #[test]
    fn plan_state_projection_does_not_return_blocked_pending_nodes() {
        let plan = dag(vec![edge("a", "b"), edge("b", "c")]);

        let ready = ready_nodes_from_plan_state(&plan);

        assert_eq!(
            ready
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
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
