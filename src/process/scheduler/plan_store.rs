//! Plan graph repository-state storage.
//!
//! `state/plan.json` is the import/export/read-model document. Durable plan
//! truth is projected from accepted plan patch records in the canonical TLog
//! when those records are available.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::capability::planning::{
    PlanEvidenceAppendPatch, PlanNodeStatus, PlanPatchPayload, PlanPatchRecord,
    PlanStatusChangePatch,
};
use crate::codec::{
    append_plan_patch_record_ndjson, load_plan_patch_records_ndjson, PlanPatchTlogRecord,
};
use crate::domain::plan::{plan_state_projection, plan_text_hash, NodeStatus, PlanDag};
use crate::kernel::PlanState;

pub const PLAN_FILE: &str = "state/plan.json";
pub const CANONICAL_TLOG_FILE: &str = "state/tlog/canon-agent.tlog.ndjson";

pub fn plan_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(PLAN_FILE)
}

pub fn canonical_tlog_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(CANONICAL_TLOG_FILE)
}

pub fn append_status_change_patch(
    workspace_root: &Path,
    node_id: &str,
    status: &NodeStatus,
) -> Result<(), String> {
    append_accepted_plan_patch(
        workspace_root,
        PlanPatchPayload::StatusChange(PlanStatusChangePatch {
            node_id_hash: plan_text_hash(node_id),
            status: plan_node_status(status),
        }),
    )
}

pub fn append_evidence_patch(
    workspace_root: &Path,
    node_id: &str,
    path: &str,
    kind: &str,
    summary: &str,
) -> Result<(), String> {
    append_accepted_plan_patch(
        workspace_root,
        PlanPatchPayload::EvidenceAppend(PlanEvidenceAppendPatch {
            node_id_hash: plan_text_hash(node_id),
            path_hash: plan_text_hash(path),
            kind_hash: plan_text_hash(kind),
            summary_hash: plan_text_hash(summary),
        }),
    )
}

fn append_accepted_plan_patch(workspace_root: &Path, payload: PlanPatchPayload) -> Result<(), String> {
    let tlog_path = canonical_tlog_path(workspace_root);
    if let Some(parent) = tlog_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let records = if tlog_path.exists() {
        load_plan_patch_records_ndjson(&tlog_path).map_err(|e| e.to_string())?
    } else {
        Vec::new()
    };
    let next_seq = records
        .iter()
        .map(plan_patch_record_seq)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
        .max(1);
    let next_revision = records
        .iter()
        .filter_map(plan_patch_applied_revision)
        .max()
        .unwrap_or(0)
        .saturating_add(1)
        .max(1);
    let patch = PlanPatchRecord::new(
        plan_text_hash("project:plan_update"),
        plan_text_hash("canon-plan-update"),
        next_seq,
        payload,
    );

    append_plan_patch_record_ndjson(&tlog_path, PlanPatchTlogRecord::Patch(patch))
        .map_err(|e| e.to_string())?;
    append_plan_patch_record_ndjson(
        &tlog_path,
        PlanPatchTlogRecord::Accepted(patch.accepted(next_revision)),
    )
    .map_err(|e| e.to_string())
}

fn plan_patch_record_seq(record: &PlanPatchTlogRecord) -> u64 {
    match record {
        PlanPatchTlogRecord::Patch(record) => record.patch_seq,
        PlanPatchTlogRecord::Accepted(record) => record.patch_seq,
        PlanPatchTlogRecord::Rejected(record) => record.patch_seq,
    }
}

fn plan_patch_applied_revision(record: &PlanPatchTlogRecord) -> Option<u64> {
    match record {
        PlanPatchTlogRecord::Accepted(record) => Some(record.applied_revision),
        PlanPatchTlogRecord::Patch(_) | PlanPatchTlogRecord::Rejected(_) => None,
    }
}

fn plan_node_status(status: &NodeStatus) -> PlanNodeStatus {
    match status {
        NodeStatus::Pending => PlanNodeStatus::Pending,
        NodeStatus::Running => PlanNodeStatus::Running,
        NodeStatus::Done => PlanNodeStatus::Done,
        NodeStatus::Failed => PlanNodeStatus::Failed,
        NodeStatus::Skipped => PlanNodeStatus::Skipped,
    }
}

fn structural_plan_import_projection(plan: &PlanDag) -> PlanState {
    let mut projection = plan.clone();
    for node in &mut projection.nodes {
        node.status = NodeStatus::Pending;
        node.assignee = None;
        node.evidence.clear();
    }
    plan_state_projection(&projection)
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

/// Load the kernel `PlanState` projection reconstructed from durable accepted
/// plan patch records in the canonical TLog. The persisted plan document is
/// used only as the import/read-model base before replaying accepted TLog
/// mutations.
pub fn load_tlog_projected_plan_state(workspace_root: &Path) -> Result<Option<PlanState>, String> {
    load_tlog_projected_plan_state_from_path(workspace_root, &canonical_tlog_path(workspace_root))
}

pub fn load_plan_read_model(workspace_root: &Path) -> Result<(PlanDag, Option<PlanState>), String> {
    let plan = load_plan(workspace_root);
    let Some(plan_state) = load_tlog_projected_plan_state(workspace_root)? else {
        return Ok((plan, None));
    };
    let read_model = project_plan_read_model_from_state(&plan, &plan_state);
    Ok((read_model, Some(plan_state)))
}

pub fn project_plan_read_model_from_state(plan: &PlanDag, plan_state: &PlanState) -> PlanDag {
    let mut read_model = plan.clone();

    read_model
        .nodes
        .retain(|node| plan_state.nodes.contains_key(&plan_text_hash(&node.id)));

    for node in &mut read_model.nodes {
        let Some(projected) = plan_state.nodes.get(&plan_text_hash(&node.id)) else {
            continue;
        };
        if let Some(status) = node_status_from_projection(projected.status) {
            node.status = status;
        }
        node.evidence.retain(|evidence| {
            projected.evidence.iter().any(|projected_evidence| {
                projected_evidence.path_hash == plan_text_hash(&evidence.path)
                    && projected_evidence.kind_hash == plan_text_hash(&evidence.kind)
                    && projected_evidence.summary_hash == plan_text_hash(&evidence.summary)
            })
        });
    }

    read_model.edges.retain(|edge| {
        plan_state.edges.iter().any(|projected_edge| {
            projected_edge.from_node_hash == plan_text_hash(&edge.from)
                && projected_edge.to_node_hash == plan_text_hash(&edge.to)
        })
    });

    read_model
}

fn node_status_from_projection(status: u64) -> Option<NodeStatus> {
    match status {
        1 => Some(NodeStatus::Pending),
        2 => Some(NodeStatus::Running),
        3 => Some(NodeStatus::Done),
        4 => Some(NodeStatus::Failed),
        5 => Some(NodeStatus::Skipped),
        _ => None,
    }
}

pub fn load_tlog_projected_plan_state_from_path(
    workspace_root: &Path,
    tlog_path: &Path,
) -> Result<Option<PlanState>, String> {
    if !tlog_path.exists() {
        return Ok(None);
    }

    let records = load_plan_patch_records_ndjson(tlog_path).map_err(|e| e.to_string())?;
    if records.is_empty() {
        return Ok(None);
    }

    let plan = load_plan(workspace_root);
    let mut projection = structural_plan_import_projection(&plan);
    let mut pending: BTreeMap<u64, PlanPatchRecord> = BTreeMap::new();
    let mut applied_patch_hashes = BTreeSet::new();
    let mut applied = 0usize;

    for record in records {
        match record {
            PlanPatchTlogRecord::Patch(patch) => {
                pending.insert(patch.patch_hash, patch);
            }
            PlanPatchTlogRecord::Accepted(accepted) => {
                if !applied_patch_hashes.insert(accepted.patch_hash) {
                    continue;
                }
                let Some(patch) = pending.get(&accepted.patch_hash).copied() else {
                    continue;
                };
                projection
                    .apply_patch(patch.payload.plan_state_patch())
                    .map_err(|rejection| format!("plan TLog replay rejected: {rejection:?}"))?;
                applied += 1;
            }
            PlanPatchTlogRecord::Rejected(_) => {}
        }
    }

    if applied == 0 {
        Ok(None)
    } else {
        Ok(Some(projection))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::plan::{PlanEdge, PlanEvidenceRef, PlanNode};

    fn test_root(name: &str) -> PathBuf {
        let mut root = std::env::temp_dir();
        root.push(format!(
            "canon-plan-store-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        root
    }

    fn node(id: &str, status: NodeStatus) -> PlanNode {
        PlanNode {
            id: id.to_string(),
            title: format!("node {id}"),
            description: String::new(),
            status,
            assignee: None,
            score_axes: Vec::new(),
            files: Vec::new(),
            evidence: Vec::new(),
        }
    }

    #[test]
    fn tlog_replay_overrides_stale_plan_json_status_and_ready_nodes() {
        let root = test_root("status");
        let mut plan = PlanDag {
            version: 1,
            nodes: vec![node("a", NodeStatus::Pending), node("b", NodeStatus::Pending)],
            edges: vec![PlanEdge {
                from: "a".to_string(),
                to: "b".to_string(),
            }],
            ..Default::default()
        };

        save_plan(&root, &plan).expect("save initial read model");
        append_status_change_patch(&root, "a", &NodeStatus::Done).expect("append status patch");

        plan.nodes[0].status = NodeStatus::Pending;
        save_plan(&root, &plan).expect("save stale read model");

        let (read_model, plan_state) = load_plan_read_model(&root).expect("load projected read model");

        assert_eq!(read_model.nodes[0].status, NodeStatus::Done);
        assert_eq!(
            crate::domain::plan::ready_nodes_from_available_plan_state(
                &read_model,
                plan_state.as_ref()
            )
            .into_iter()
            .map(|node| node.id.as_str())
            .collect::<Vec<_>>(),
            vec!["b"]
        );

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn tlog_replay_filters_evidence_to_accepted_tlog_membership() {
        let root = test_root("evidence");
        let accepted = PlanEvidenceRef {
            path: "state/agent-evidence/a.md".to_string(),
            kind: "validation".to_string(),
            summary: "accepted evidence".to_string(),
            gate: String::new(),
            evidence: String::new(),
            receipt_hash: 0,
            accepted: false,
        };
        let stale = PlanEvidenceRef {
            path: "state/agent-evidence/stale.md".to_string(),
            kind: "validation".to_string(),
            summary: "stale evidence".to_string(),
            gate: String::new(),
            evidence: String::new(),
            receipt_hash: 0,
            accepted: false,
        };
        let mut plan = PlanDag {
            version: 1,
            nodes: vec![node("a", NodeStatus::Pending)],
            edges: Vec::new(),
            ..Default::default()
        };
        plan.nodes[0].evidence = vec![accepted.clone(), stale];

        save_plan(&root, &plan).expect("save stale evidence read model");
        append_evidence_patch(
            &root,
            "a",
            &accepted.path,
            &accepted.kind,
            &accepted.summary,
        )
        .expect("append evidence patch");

        let (read_model, plan_state) = load_plan_read_model(&root).expect("load projected read model");

        assert!(plan_state.is_some());
        assert_eq!(read_model.nodes[0].evidence.len(), 1);
        assert_eq!(read_model.nodes[0].evidence[0].path, accepted.path);

        let _ = std::fs::remove_dir_all(root);
    }
}
