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

pub const EVIDENCE_GATE_VERIFICATION: &str = "Verification";

pub const EVIDENCE_TYPE_LINEAGE_PROOF: &str = "LineageProof";

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

    pub fn is_accepted_verification_evidence(&self) -> bool {
        AcceptedEvidenceReceipt::try_from(self).is_ok()
    }

    pub fn is_blocker(&self) -> bool {
        self.kind == EVIDENCE_KIND_BLOCKER
    }
}

/// Typed proof that a generic plan evidence reference is accepted verification
/// evidence. This is the only value supervisor completion may consume as a
/// task-completion prerequisite.
#[derive(Clone, Copy, Debug)]
pub struct AcceptedEvidenceReceipt<'a> {
    evidence: &'a PlanEvidenceRef,
}

impl<'a> AcceptedEvidenceReceipt<'a> {
    pub fn evidence(self) -> &'a PlanEvidenceRef {
        self.evidence
    }

    pub fn receipt_hash(self) -> u64 {
        self.evidence.receipt_hash
    }
}

impl<'a> TryFrom<&'a PlanEvidenceRef> for AcceptedEvidenceReceipt<'a> {
    type Error = PlanValidationError;

    fn try_from(evidence: &'a PlanEvidenceRef) -> Result<Self, Self::Error> {
        if !evidence.accepted {
            return Err(PlanValidationError::new(
                "unaccepted_evidence",
                "verification evidence must be accepted before task completion",
            ));
        }
        if evidence.path.is_empty() || evidence.kind.is_empty() || evidence.summary.is_empty() {
            return Err(PlanValidationError::new(
                "empty_evidence_payload",
                "accepted verification evidence requires nonempty payload fields",
            ));
        }
        if evidence.receipt_hash == 0 {
            return Err(PlanValidationError::new(
                "invalid_evidence_receipt",
                "accepted evidence requires non-zero receipt_hash",
            ));
        }
        if evidence.gate != EVIDENCE_GATE_VERIFICATION {
            return Err(PlanValidationError::new(
                "wrong_evidence_gate",
                format!(
                    "task completion requires gate '{}' but found '{}'",
                    EVIDENCE_GATE_VERIFICATION, evidence.gate
                ),
            ));
        }
        if evidence.evidence != EVIDENCE_TYPE_LINEAGE_PROOF {
            return Err(PlanValidationError::new(
                "wrong_evidence_type",
                format!(
                    "task completion requires evidence '{}' but found '{}'",
                    EVIDENCE_TYPE_LINEAGE_PROOF, evidence.evidence
                ),
            ));
        }

        Ok(Self { evidence })
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
    crate::service::scheduler::plan_store::plan_path(workspace_root)
}

pub fn load_plan(workspace_root: &Path) -> PlanDag {
    crate::service::scheduler::plan_store::load_plan(workspace_root)
}

pub fn save_plan(workspace_root: &Path, plan: &PlanDag) -> Result<(), String> {
    crate::service::scheduler::plan_store::save_plan(workspace_root, plan)
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

/// Returns `Pending` nodes that are permanently blocked: every upstream dependency
/// is terminal (Done/Failed/Skipped) and at least one is non-Done (Failed/Skipped).
///
/// These nodes can never become ready under normal scheduling. The supervisor uses
/// this to cascade `Skipped` status and to detect plan exhaustion when pending nodes
/// exist but none can ever run.
pub fn blocked_pending_nodes(plan: &PlanDag) -> Vec<&PlanNode> {
    let terminal_ids: HashSet<&str> = plan
        .nodes
        .iter()
        .filter(|n| {
            matches!(
                n.status,
                NodeStatus::Done | NodeStatus::Failed | NodeStatus::Skipped
            )
        })
        .map(|n| n.id.as_str())
        .collect();

    let non_done_terminal_ids: HashSet<&str> = plan
        .nodes
        .iter()
        .filter(|n| matches!(n.status, NodeStatus::Failed | NodeStatus::Skipped))
        .map(|n| n.id.as_str())
        .collect();

    plan.nodes
        .iter()
        .filter(|node| {
            if node.status != NodeStatus::Pending {
                return false;
            }
            let upstreams: Vec<&str> = plan
                .edges
                .iter()
                .filter(|e| e.to == node.id)
                .map(|e| e.from.as_str())
                .collect();
            !upstreams.is_empty()
                && upstreams.iter().all(|u| terminal_ids.contains(u))
                && upstreams.iter().any(|u| non_done_terminal_ids.contains(*u))
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
                gate: EVIDENCE_GATE_VERIFICATION.to_string(),
                evidence: EVIDENCE_TYPE_LINEAGE_PROOF.to_string(),
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
    fn accepted_evidence_receipt_types_completion_prerequisite_for_inv_2d8266d0f547() {
        let accepted = node("accepted").evidence.remove(0);
        let typed = AcceptedEvidenceReceipt::try_from(&accepted)
            .expect("accepted verification evidence should type-check");
        assert_eq!(typed.receipt_hash(), 1);
        assert_eq!(typed.evidence().path, "state/agent-evidence/accepted.md");

        let mut missing_acceptance = accepted.clone();
        missing_acceptance.accepted = false;
        let err = AcceptedEvidenceReceipt::try_from(&missing_acceptance)
            .expect_err("inv-2d8266d0f547: unaccepted evidence must not type-check");
        assert_eq!(err.code, "unaccepted_evidence");

        let mut missing_receipt = accepted.clone();
        missing_receipt.receipt_hash = 0;
        let err = AcceptedEvidenceReceipt::try_from(&missing_receipt)
            .expect_err("accepted evidence without receipt hash must not type-check");
        assert_eq!(err.code, "invalid_evidence_receipt");

        let mut empty_payload = accepted.clone();
        empty_payload.summary.clear();
        let err = AcceptedEvidenceReceipt::try_from(&empty_payload)
            .expect_err("accepted evidence without payload must not type-check");
        assert_eq!(err.code, "empty_evidence_payload");

        let mut wrong_gate = accepted.clone();
        wrong_gate.gate = EVIDENCE_GATE_EXECUTION.to_string();
        wrong_gate.evidence = EVIDENCE_TYPE_EXECUTION_RECEIPT.to_string();
        let err = AcceptedEvidenceReceipt::try_from(&wrong_gate)
            .expect_err("wrong gate evidence must not type-check");
        assert_eq!(err.code, "wrong_evidence_gate");

        let mut wrong_type = accepted;
        wrong_type.evidence = "VerificationReport".to_string();
        let err = AcceptedEvidenceReceipt::try_from(&wrong_type)
            .expect_err("wrong evidence type must not type-check");
        assert_eq!(err.code, "wrong_evidence_type");
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

    #[test]
    fn blocked_pending_nodes_detects_failed_upstream() {
        let mut plan = PlanDag {
            version: 1,
            nodes: vec![node("a"), node("b"), node("c")],
            edges: vec![edge("a", "b"), edge("b", "c")],
            ..Default::default()
        };
        plan.nodes[0].status = NodeStatus::Failed;
        // b: all upstreams terminal (a=Failed), at least one non-Done → blocked
        // c: upstream b is still Pending → not blocked yet

        let blocked = blocked_pending_nodes(&plan);

        assert_eq!(
            blocked.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(),
            vec!["b"]
        );
    }

    #[test]
    fn blocked_pending_nodes_skipped_upstream_also_blocks() {
        let mut plan = PlanDag {
            version: 1,
            nodes: vec![node("a"), node("b")],
            edges: vec![edge("a", "b")],
            ..Default::default()
        };
        plan.nodes[0].status = NodeStatus::Skipped;

        let blocked = blocked_pending_nodes(&plan);

        assert_eq!(
            blocked.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(),
            vec!["b"]
        );
    }

    #[test]
    fn blocked_pending_nodes_done_upstream_is_not_blocked() {
        let mut plan = PlanDag {
            version: 1,
            nodes: vec![node("a"), node("b")],
            edges: vec![edge("a", "b")],
            ..Default::default()
        };
        plan.nodes[0].status = NodeStatus::Done;

        let blocked = blocked_pending_nodes(&plan);

        assert!(blocked.is_empty());
    }

    #[test]
    fn blocked_pending_nodes_root_node_is_never_blocked() {
        // A Pending root node (no upstreams) must never appear as blocked.
        let plan = PlanDag {
            version: 1,
            nodes: vec![node("root")],
            edges: vec![],
            ..Default::default()
        };

        let blocked = blocked_pending_nodes(&plan);

        assert!(blocked.is_empty());
    }

    #[test]
    fn blocked_pending_nodes_partial_terminal_upstream_not_blocked() {
        // b depends on a (Failed) and x (Running). x is not terminal → b is not yet blocked.
        let mut plan = PlanDag {
            version: 1,
            nodes: vec![node("a"), node("x"), node("b")],
            edges: vec![edge("a", "b"), edge("x", "b")],
            ..Default::default()
        };
        plan.nodes[0].status = NodeStatus::Failed;
        plan.nodes[1].status = NodeStatus::Running;

        let blocked = blocked_pending_nodes(&plan);

        assert!(
            blocked.is_empty(),
            "b has a Running upstream — not yet permanently blocked"
        );
    }
}
