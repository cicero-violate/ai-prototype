use ai::capability::planning::{
    PlanEdgePatch, PlanEvidenceAppendPatch, PlanNodeStatus, PlanNodeUpsertPatch, PlanPatchPayload,
    PlanPatchRecord, PlanStatusChangePatch,
};
use ai::domain::plan::{
    plan_state_projection, ready_nodes_from_plan_state_projection, validate_plan_patch_mutation,
    NodeStatus, PlanDag, PlanEdge, PlanEvidenceRef, PlanNode,
};
use ai::kernel::{
    PlanEdgeProjection, PlanEvidenceProjection, PlanState, PlanStatePatch, PlanStateRejection,
};
use ai::process::scheduler::plan_store::load_plan;
use ai::process::scheduler::plan_store::{
    append_assignee_change_patch, append_evidence_patch, append_node_remove_patch,
    append_status_change_patch, load_plan_read_model, save_plan,
};
use ai::{Packet, PlanDecision, PlanRecord};
use std::path::PathBuf;

const PROJECTED_STATUS_PENDING: u64 = 1;
const PROJECTED_STATUS_DONE: u64 = 3;

#[test]
fn planning_record_decomposes_objective_with_lineage() {
    let mut packet = Packet::empty();
    packet.objective_id = 42;
    packet.objective_required_tasks = 4;
    packet.objective_done_tasks = 1;
    packet.revision = 9;

    let record = PlanRecord::from_packet(packet);

    assert_eq!(record.task_count, 4);
    assert_eq!(record.completed_tasks, 1);
    assert_eq!(record.task_id, 4202);
    assert_eq!(record.ready_tasks, 1);
    assert_eq!(record.tasks.iter().filter(|task| task.ready).count(), 1);
    assert_eq!(record.tasks[0].depends_on, 0);
    assert_eq!(record.tasks[1].depends_on, 4201);
    assert_eq!(record.tasks[2].depends_on, 4202);
    assert!(record.is_valid());

    let mut tampered = record.clone();
    tampered.tasks[1].depends_on = 0;
    assert!(!tampered.is_valid());
}

#[test]
fn planning_record_blocks_when_all_tasks_complete() {
    let mut packet = Packet::empty();
    packet.objective_id = 7;
    packet.objective_required_tasks = 2;
    packet.objective_done_tasks = 2;

    let record = PlanRecord::from_packet(packet);

    assert_eq!(record.ready_tasks, 0);
    assert_eq!(record.decision(), PlanDecision::Blocked);
    assert!(!record.is_valid());
}

fn replay_contract_root(name: &str) -> PathBuf {
    PathBuf::from("target/test-tmp/plan-replay-contract")
        .join(format!("{name}-{}", std::process::id()))
}

fn contract_node(id: &str, status: NodeStatus) -> PlanNode {
    PlanNode {
        id: id.to_string(),
        title: format!("{id} title"),
        description: format!("{id} description"),
        status,
        assignee: None,
        score_axes: vec!["Determinism".to_string()],
        files: vec![format!("ai/src/{id}.rs")],
        evidence: Vec::new(),
    }
}

fn contract_evidence(path: &str, summary: &str) -> PlanEvidenceRef {
    PlanEvidenceRef {
        path: path.to_string(),
        kind: "validation".to_string(),
        summary: summary.to_string(),
        gate: String::new(),
        evidence: String::new(),
        receipt_hash: 0,
        accepted: false,
    }
}

fn contract_edge(from: &str, to: &str) -> PlanEdge {
    PlanEdge {
        from: from.to_string(),
        to: to.to_string(),
    }
}

fn node_hash(projection: &PlanState, title_hash: u64) -> u64 {
    projection
        .nodes
        .values()
        .find(|node| node.title_hash == title_hash)
        .expect("projected node by title hash")
        .node_id_hash
}

fn accepted_payload(seq: u64, payload: PlanPatchPayload) -> PlanPatchPayload {
    let patch = PlanPatchRecord::new(0xadd9, 0x9, seq, payload);
    assert!(patch.is_self_consistent());
    let accepted = patch.accepted(seq);
    assert!(accepted.is_self_consistent());
    patch.payload
}

#[test]
fn plan_state_replay_contract_imports_accepts_rejects_and_reconstructs_ready_nodes() {
    let root = replay_contract_root("import-accept-reject-ready");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("state")).expect("test state dir");

    let imported_plan = PlanDag {
        version: 1,
        nodes: vec![
            contract_node("root", NodeStatus::Done),
            contract_node("blocked", NodeStatus::Pending),
        ],
        edges: vec![contract_edge("root", "blocked")],
        ..Default::default()
    };
    std::fs::write(
        root.join("state/plan.json"),
        serde_json::to_vec_pretty(&imported_plan).expect("encode plan json"),
    )
    .expect("write plan json");

    let loaded_plan = load_plan(&root);
    let imported_projection = plan_state_projection(&loaded_plan);
    let root_hash = node_hash(
        &imported_projection,
        imported_projection
            .nodes
            .values()
            .find(|node| node.status == PROJECTED_STATUS_DONE)
            .expect("root node")
            .title_hash,
    );
    let blocked_hash = node_hash(
        &imported_projection,
        imported_projection
            .nodes
            .values()
            .find(|node| node.status == PROJECTED_STATUS_PENDING)
            .expect("blocked node")
            .title_hash,
    );

    let import_patch = PlanStatePatch::FullImport {
        node_count: imported_projection.nodes.len() as u64,
        edge_count: imported_projection.edges.len() as u64,
        nodes_hash: imported_projection.nodes_hash(),
        edges_hash: imported_projection.edges_hash(),
    };

    let read_model = PlanDag {
        version: loaded_plan.version,
        nodes: vec![
            contract_node("root", NodeStatus::Done),
            contract_node("blocked", NodeStatus::Done),
            PlanNode {
                id: "new".to_string(),
                title: "new title".to_string(),
                description: "new description".to_string(),
                status: NodeStatus::Pending,
                assignee: None,
                score_axes: Vec::new(),
                files: Vec::new(),
                evidence: vec![PlanEvidenceRef {
                    path: "state/agent-evidence/new.md".to_string(),
                    kind: "validation".to_string(),
                    summary: "new evidence".to_string(),
                    gate: "Execution".to_string(),
                    evidence: "ExecutionReceipt".to_string(),
                    receipt_hash: 1,
                    accepted: true,
                }],
            },
        ],
        edges: vec![
            contract_edge("root", "blocked"),
            contract_edge("blocked", "new"),
        ],
        ..Default::default()
    };
    let read_projection = plan_state_projection(&read_model);
    let new_hash = read_projection
        .nodes
        .values()
        .find(|node| node.status == PROJECTED_STATUS_PENDING)
        .expect("new pending node")
        .node_id_hash;
    let new_title_hash = read_projection.nodes[&new_hash].title_hash;
    let evidence = PlanEvidenceProjection {
        path_hash: 0xabc3,
        kind_hash: 0xabc4,
        summary_hash: 0xabc5,
    };
    let mut accepted_mutations: Vec<PlanStatePatch> = imported_projection
        .nodes
        .values()
        .map(|node| PlanStatePatch::NodeUpsert {
            node_id_hash: node.node_id_hash,
            title_hash: node.title_hash,
            description_hash: node.description_hash,
            status: node.status,
            assignee_hash: node.assignee_hash,
            score_axes_hash: node.score_axes_hash,
            files_hash: node.files_hash,
        })
        .collect();
    accepted_mutations.extend(
        imported_projection
            .edges
            .iter()
            .copied()
            .map(PlanStatePatch::EdgeAdd),
    );
    accepted_mutations.extend(vec![
        import_patch,
        accepted_payload(
            1,
            PlanPatchPayload::NodeUpsert(PlanNodeUpsertPatch {
                node_id_hash: new_hash,
                title_hash: new_title_hash,
                description_hash: 0xabc6,
                status: PlanNodeStatus::Pending,
                assignee_hash: 0,
                score_axes_hash: 0xabc7,
                files_hash: 0xabc8,
            }),
        )
        .plan_state_patch(),
        accepted_payload(
            2,
            PlanPatchPayload::EdgeAdd(PlanEdgePatch {
                from_node_hash: blocked_hash,
                to_node_hash: new_hash,
            }),
        )
        .plan_state_patch(),
        accepted_payload(
            3,
            PlanPatchPayload::EvidenceAppend(PlanEvidenceAppendPatch {
                node_id_hash: new_hash,
                path_hash: evidence.path_hash,
                kind_hash: evidence.kind_hash,
                summary_hash: evidence.summary_hash,
            }),
        )
        .plan_state_patch(),
        accepted_payload(
            4,
            PlanPatchPayload::StatusChange(PlanStatusChangePatch {
                node_id_hash: blocked_hash,
                status: PlanNodeStatus::Done,
            }),
        )
        .plan_state_patch(),
    ]);

    let replayed =
        PlanState::replay(accepted_mutations.clone()).expect("replay accepted mutations");
    let mut applied = PlanState::default();
    for mutation in accepted_mutations.iter().copied() {
        applied
            .apply_patch(mutation)
            .expect("apply accepted mutation");
    }

    assert_eq!(replayed, applied);
    assert_eq!(replayed.nodes[&new_hash].evidence, vec![evidence]);
    assert_eq!(replayed.nodes[&blocked_hash].status, PROJECTED_STATUS_DONE);
    assert!(replayed.edges.contains(&PlanEdgeProjection {
        from_node_hash: blocked_hash,
        to_node_hash: new_hash,
    }));

    let ready_node_ids: Vec<&str> = ready_nodes_from_plan_state_projection(&read_model, &replayed)
        .iter()
        .map(|node| node.id.as_str())
        .collect();
    assert_eq!(ready_node_ids, vec!["new"]);

    let cycle_plan = PlanDag {
        edges: vec![
            contract_edge("root", "blocked"),
            contract_edge("blocked", "root"),
        ],
        ..loaded_plan.clone()
    };
    let cycle_error = validate_plan_patch_mutation(&cycle_plan)
        .expect_err("plan patch mutation should be rejected");
    assert_eq!(cycle_error.code, "dependency_cycle");
    let cycle_patch = PlanPatchRecord::new(
        0xadd9,
        0x9,
        5,
        PlanPatchPayload::EdgeAdd(PlanEdgePatch {
            from_node_hash: blocked_hash,
            to_node_hash: root_hash,
        }),
    );
    assert!(cycle_patch
        .rejected(PlanStateRejection::InvalidEdge.reason_hash())
        .is_self_consistent());

    let missing_dependency_plan = PlanDag {
        edges: vec![contract_edge("missing", "blocked")],
        ..loaded_plan
    };
    let missing_error = validate_plan_patch_mutation(&missing_dependency_plan)
        .expect_err("plan patch mutation should be rejected");
    assert_eq!(missing_error.code, "unknown_dependency");
    let missing_patch = PlanPatchRecord::new(
        0xadd9,
        0x9,
        6,
        PlanPatchPayload::EdgeAdd(PlanEdgePatch {
            from_node_hash: 0xfeed,
            to_node_hash: blocked_hash,
        }),
    );
    assert!(missing_patch
        .rejected(PlanStateRejection::MissingNode.reason_hash())
        .is_self_consistent());

    assert_eq!(
        PlanState::replay(vec![PlanStatePatch::EdgeAdd(PlanEdgeProjection {
            from_node_hash: 0xfeed,
            to_node_hash: blocked_hash,
        })])
        .expect_err("plan patch mutation should be rejected"),
        PlanStateRejection::MissingNode
    );

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn load_plan_read_model_projects_lifecycle_authority_from_accepted_plan_patches() {
    let root = replay_contract_root("read-model-authority");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("state")).expect("test state dir");

    let accepted_evidence = contract_evidence(
        "state/agent-evidence/accepted.md",
        "accepted validation evidence",
    );
    let stale_evidence =
        contract_evidence("state/agent-evidence/stale.md", "stale scaffold evidence");
    let mut scaffold_authoritative = contract_node("authoritative", NodeStatus::Failed);
    scaffold_authoritative.assignee = Some("accepted-worker".to_string());
    scaffold_authoritative.evidence = vec![accepted_evidence.clone(), stale_evidence];
    let mut scaffold_removed = contract_node("removed", NodeStatus::Done);
    scaffold_removed.assignee = Some("removed-worker".to_string());

    save_plan(
        &root,
        &PlanDag {
            version: 1,
            nodes: vec![scaffold_authoritative, scaffold_removed],
            edges: Vec::new(),
            ..Default::default()
        },
    )
    .expect("save stale raw plan scaffold");

    append_status_change_patch(&root, "authoritative", &NodeStatus::Running)
        .expect("append accepted status patch");
    append_assignee_change_patch(&root, "authoritative", Some("accepted-worker"))
        .expect("append accepted assignee patch");
    append_evidence_patch(
        &root,
        "authoritative",
        &accepted_evidence.path,
        &accepted_evidence.kind,
        &accepted_evidence.summary,
    )
    .expect("append accepted evidence patch");
    append_node_remove_patch(&root, "removed").expect("append accepted removal patch");

    let (read_model, plan_state) = load_plan_read_model(&root).expect("load projected read model");

    assert!(
        plan_state.is_some(),
        "accepted patches must produce a projection"
    );
    assert_eq!(
        read_model.nodes.len(),
        1,
        "accepted removal hides scaffold node"
    );
    let node = read_model
        .nodes
        .iter()
        .find(|node| node.id == "authoritative")
        .expect("authoritative node remains projected");
    assert_eq!(
        node.status,
        NodeStatus::Running,
        "accepted status patch must override stale scaffold lifecycle status"
    );
    assert_eq!(
        node.assignee.as_deref(),
        Some("accepted-worker"),
        "assignee is retained only when accepted projection authorizes it"
    );
    assert_eq!(
        node.evidence.len(),
        1,
        "stale scaffold evidence is filtered"
    );
    assert_eq!(node.evidence[0].path, accepted_evidence.path);

    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn load_plan_read_model_clears_stale_scaffold_assignee_without_accepted_patch() {
    let root = replay_contract_root("read-model-clears-assignee");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("state")).expect("test state dir");

    let mut scaffold = contract_node("node", NodeStatus::Failed);
    scaffold.assignee = Some("raw-scaffold-worker".to_string());
    save_plan(
        &root,
        &PlanDag {
            version: 1,
            nodes: vec![scaffold],
            edges: Vec::new(),
            ..Default::default()
        },
    )
    .expect("save stale raw plan scaffold");

    append_status_change_patch(&root, "node", &NodeStatus::Done)
        .expect("append accepted status patch");

    let (read_model, plan_state) = load_plan_read_model(&root).expect("load projected read model");

    assert!(
        plan_state.is_some(),
        "accepted status patch must produce projection"
    );
    assert_eq!(read_model.nodes[0].status, NodeStatus::Done);
    assert_eq!(
        read_model.nodes[0].assignee, None,
        "raw scaffold assignee is not lifecycle truth without accepted patch authority"
    );

    let _ = std::fs::remove_dir_all(root);
}
