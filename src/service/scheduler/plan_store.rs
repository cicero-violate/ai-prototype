//! Plan graph repository-state storage.
//!
//! `state/plan.json` is the import/export/read-model document. Durable plan
//! truth is projected from accepted plan patch records in the canonical TLog
//! when those records are available.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::capability::planning::{
    PlanAssigneeChangePatch, PlanEdgePatch, PlanEvidenceAppendPatch, PlanNodeRemovePatch,
    PlanNodeStatus, PlanNodeUpsertPatch, PlanPatchPayload, PlanPatchRecord, PlanStatusChangePatch,
};
use crate::codec::{
    append_plan_patch_record_ndjson, load_plan_patch_records_ndjson, PlanPatchTlogRecord,
};
use crate::domain::plan::{
    plan_state_projection, plan_text_hash, NodeStatus, PlanDag, PlanEdge, PlanEvidenceRef, PlanNode,
};
use crate::kernel::{PlanState, PlanStatePatch, PlanStateRejection};

pub const PLAN_FILE: &str = "state/plan.json";
pub const CANONICAL_TLOG_FILE: &str = "state/tlog/canon-agent.tlog.ndjson";
pub const PLAN_PATCH_TLOG_FILE: &str = "state/tlog/plan-patches.tlog.ndjson";

pub fn plan_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(PLAN_FILE)
}

pub fn canonical_tlog_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(CANONICAL_TLOG_FILE)
}

pub fn plan_patch_tlog_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(PLAN_PATCH_TLOG_FILE)
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
    )?;
    mirror_status_to_plan_json(workspace_root, node_id, status)
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
    )?;
    mirror_evidence_to_plan_json(workspace_root, node_id, path, kind, summary)
}

fn mirror_status_to_plan_json(
    workspace_root: &Path,
    node_id: &str,
    status: &NodeStatus,
) -> Result<(), String> {
    let mut plan = load_plan(workspace_root);
    let Some(node) = plan.nodes.iter_mut().find(|node| node.id == node_id) else {
        return Ok(());
    };
    node.status = status.clone();
    if !matches!(status, NodeStatus::Running) {
        node.assignee = None;
    }
    save_plan(workspace_root, &plan)
}

fn mirror_evidence_to_plan_json(
    workspace_root: &Path,
    node_id: &str,
    path: &str,
    kind: &str,
    summary: &str,
) -> Result<(), String> {
    let mut plan = load_plan(workspace_root);
    let Some(node) = plan.nodes.iter_mut().find(|node| node.id == node_id) else {
        return Ok(());
    };
    let evidence = PlanEvidenceRef {
        path: path.to_string(),
        kind: kind.to_string(),
        summary: summary.to_string(),
        gate: String::new(),
        evidence: String::new(),
        receipt_hash: 0,
        accepted: false,
    };
    if !node.evidence.iter().any(|existing| {
        existing.path == evidence.path
            && existing.kind == evidence.kind
            && existing.summary == evidence.summary
    }) {
        node.evidence.push(evidence);
    }
    save_plan(workspace_root, &plan)
}

pub fn append_node_upsert_patch(workspace_root: &Path, node: &PlanNode) -> Result<(), String> {
    append_accepted_plan_patch(
        workspace_root,
        PlanPatchPayload::NodeUpsert(PlanNodeUpsertPatch {
            node_id_hash: plan_text_hash(&node.id),
            title_hash: plan_text_hash(&node.title),
            description_hash: plan_text_hash(&node.description),
            status: plan_node_status(&node.status),
            assignee_hash: node
                .assignee
                .as_deref()
                .map(plan_text_hash)
                .unwrap_or_default(),
            score_axes_hash: plan_text_list_hash(&node.score_axes),
            files_hash: plan_text_list_hash(&node.files),
        }),
    )
}

pub fn append_edge_add_patch(workspace_root: &Path, edge: &PlanEdge) -> Result<(), String> {
    append_accepted_plan_patch(
        workspace_root,
        PlanPatchPayload::EdgeAdd(PlanEdgePatch {
            from_node_hash: plan_text_hash(&edge.from),
            to_node_hash: plan_text_hash(&edge.to),
        }),
    )
}

pub fn append_edge_remove_patch(workspace_root: &Path, edge: &PlanEdge) -> Result<(), String> {
    append_accepted_plan_patch(
        workspace_root,
        PlanPatchPayload::EdgeRemove(PlanEdgePatch {
            from_node_hash: plan_text_hash(&edge.from),
            to_node_hash: plan_text_hash(&edge.to),
        }),
    )
}

pub fn append_node_remove_patch(workspace_root: &Path, node_id: &str) -> Result<(), String> {
    append_accepted_plan_patch(
        workspace_root,
        PlanPatchPayload::NodeRemove(PlanNodeRemovePatch {
            node_id_hash: plan_text_hash(node_id),
        }),
    )
}

pub fn append_assignee_change_patch(
    workspace_root: &Path,
    node_id: &str,
    assignee: Option<&str>,
) -> Result<(), String> {
    append_accepted_plan_patch(
        workspace_root,
        PlanPatchPayload::AssigneeChange(PlanAssigneeChangePatch {
            node_id_hash: plan_text_hash(node_id),
            assignee_hash: assignee.map(plan_text_hash).unwrap_or_default(),
        }),
    )
}

fn append_accepted_plan_patch(
    workspace_root: &Path,
    payload: PlanPatchPayload,
) -> Result<(), String> {
    let tlog_path = plan_patch_tlog_path(workspace_root);
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

fn plan_text_list_hash(values: &[String]) -> u64 {
    values.iter().fold(0x504c_414e_4c49_5354u64, |hash, value| {
        hash.wrapping_mul(0x0000_0100_0000_01b3) ^ plan_text_hash(value)
    })
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
    load_tlog_projected_plan_state_from_path(workspace_root, &plan_patch_tlog_path(workspace_root))
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
        node.assignee = node.assignee.take().filter(|assignee| {
            projected.assignee_hash != 0 && projected.assignee_hash == plan_text_hash(assignee)
        });
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
                apply_plan_tlog_patch(&mut projection, patch.payload)
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

fn apply_plan_tlog_patch(
    projection: &mut PlanState,
    payload: PlanPatchPayload,
) -> Result<(), PlanStateRejection> {
    let mutation = payload.plan_state_patch();
    match projection.apply_patch(mutation) {
        Ok(_) => Ok(()),
        // NodeRemove on an already-removed node is idempotent.
        Err(PlanStateRejection::MissingNode)
            if matches!(mutation, PlanStatePatch::NodeRemove { .. }) =>
        {
            Ok(())
        }
        // StatusChange on an already-removed node is idempotent — the reconciler may
        // write StatusChange+NodeRemove pairs on each run; later runs' StatusChange
        // patches arrive after earlier runs' NodeRemove patches.
        Err(PlanStateRejection::MissingNode)
            if matches!(mutation, PlanStatePatch::StatusChange { .. }) =>
        {
            Ok(())
        }
        Err(PlanStateRejection::MissingEdge)
            if matches!(mutation, PlanStatePatch::EdgeRemove(_)) =>
        {
            Ok(())
        }
        Err(rejection) => Err(rejection),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::plan::{PlanEdge, PlanEvidenceRef, PlanNode, EVIDENCE_KIND_VALIDATION};

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
            nodes: vec![
                node("a", NodeStatus::Pending),
                node("b", NodeStatus::Pending),
            ],
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

        let (read_model, plan_state) =
            load_plan_read_model(&root).expect("load projected read model");

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
            kind: EVIDENCE_KIND_VALIDATION.to_string(),
            summary: "accepted evidence".to_string(),
            gate: String::new(),
            evidence: String::new(),
            receipt_hash: 0,
            accepted: false,
        };
        let stale = PlanEvidenceRef {
            path: "state/agent-evidence/stale.md".to_string(),
            kind: EVIDENCE_KIND_VALIDATION.to_string(),
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

        let (read_model, plan_state) =
            load_plan_read_model(&root).expect("load projected read model");

        assert!(plan_state.is_some());
        assert_eq!(read_model.nodes[0].evidence.len(), 1);
        assert_eq!(read_model.nodes[0].evidence[0].path, accepted.path);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn tlog_replay_archives_removed_nodes_from_accepted_patch() {
        let root = test_root("node-remove");
        let plan = PlanDag {
            version: 1,
            nodes: vec![
                node("terminal", NodeStatus::Done),
                node("remaining", NodeStatus::Pending),
            ],
            edges: vec![PlanEdge {
                from: "terminal".to_string(),
                to: "remaining".to_string(),
            }],
            ..Default::default()
        };

        save_plan(&root, &plan).expect("save initial read model");
        append_node_remove_patch(&root, "terminal").expect("append remove patch");

        let persisted_plan = load_plan(&root);
        assert_eq!(persisted_plan.nodes.len(), 2);
        assert!(persisted_plan
            .nodes
            .iter()
            .any(|node| node.id == "terminal"));

        let (read_model, plan_state) =
            load_plan_read_model(&root).expect("load projected read model");

        assert!(plan_state.is_some());
        assert_eq!(
            read_model
                .nodes
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            vec!["remaining"]
        );
        assert!(read_model.edges.is_empty());

        let _ = std::fs::remove_dir_all(root);
    }
}
