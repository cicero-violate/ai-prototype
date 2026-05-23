use std::path::{Path, PathBuf};
use std::time::Duration;

use ai::capability::llm::{LlmTaskContext, LlmTurnReceipt, LlmTurnRecord};
use ai::domain::plan::{NodeStatus, PlanDag, PlanEvidenceRef, PlanNode};
use ai::process::agent::{heartbeat_claim, run_with_heartbeat};
use ai::process::scheduler::plan_store::{
    append_evidence_patch, load_plan, load_plan_read_model, save_plan,
};
use ai::process::supervisor::{
    TaskClaimRequest, TaskCompleteRequest, TaskFailRequest, TaskHeartbeatRequest, WorkerProcess,
};
use ai::{
    CapabilityRegistry, Cause, Command, CommandEnvelope, Evidence, Gate, GateId, Phase,
    RuntimeConfig, State, TLog, TaskLifecycleReceipt,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn source(path: &str) -> String {
    std::fs::read_to_string(repo_root().join(path)).expect("source file should be readable")
}

fn contract_root(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "ai-architecture-boundary-{name}-{}",
        std::process::id()
    ))
}

fn plan_node(id: &str, status: NodeStatus, evidence: Vec<PlanEvidenceRef>) -> PlanNode {
    PlanNode {
        id: id.to_string(),
        title: format!("{id} title"),
        description: format!("{id} description"),
        status,
        assignee: None,
        score_axes: Vec::new(),
        files: Vec::new(),
        evidence,
    }
}

fn accepted_execution_evidence(path: &str) -> PlanEvidenceRef {
    PlanEvidenceRef {
        path: path.to_string(),
        kind: "verification".to_string(),
        summary: "node completed with accepted execution receipt".to_string(),
        gate: "Execution".to_string(),
        evidence: "ExecutionReceipt".to_string(),
        receipt_hash: 42,
        accepted: true,
    }
}

fn write_plan(root: &Path, evidence: Vec<PlanEvidenceRef>) {
    let evidence_refs = evidence.clone();
    save_plan(
        root,
        &PlanDag {
            version: 1,
            nodes: vec![plan_node("node-1", NodeStatus::Pending, Vec::new())],
            edges: Vec::new(),
            ..Default::default()
        },
    )
    .expect("plan should write");
    for evidence in evidence_refs {
        append_evidence_patch(
            root,
            "node-1",
            &evidence.path,
            &evidence.kind,
            &evidence.summary,
        )
        .expect("plan evidence patch should write");
    }
}

fn worker_process(root: &Path) -> WorkerProcess {
    WorkerProcess::new(
        root.join("kernel_tlog"),
        root.join("tlog"),
        "http://127.0.0.1:1".to_string(),
        "http://127.0.0.1:2".to_string(),
        root.to_path_buf(),
        "http://127.0.0.1:3".to_string(),
        0,
    )
}

#[test]
fn llm_task_receipt_types_are_public_and_bind_to_evidence_submission() {
    let ctx =
        LlmTaskContext::new("node-1", "worker-a", 7, 0xabc, 9).expect("context should be valid");
    let record =
        LlmTurnRecord::new(&ctx, 1, 0x111, 0x222, 33, true).expect("turn record should be valid");
    let receipt = LlmTurnReceipt::from_turn_record(&record, 5, 0x333, 7)
        .expect("turn receipt should be valid");

    let submission = receipt.submission();
    assert!(submission.is_contract_valid());
    assert_eq!(submission.payload_hash, receipt.receipt_hash);
}

#[test]
fn worker_heartbeat_helper_is_part_of_the_process_agent_api() {
    let _api: fn(&str, &ai::process::agent::ActiveClaim, &str, u64) = heartbeat_claim;
    let claim = ai::process::agent::ActiveClaim {
        node_id: "node-1".to_string(),
        claim_id: 1,
        expires_at_ms: 2,
        receipt_hash: 3,
        tlog_submitted: false,
    };

    let result = run_with_heartbeat("http://127.0.0.1:1", &claim, "worker-a", 60_000, || 42);
    assert_eq!(result, 42);
}

#[test]
fn runtime_replay_core_does_not_import_capabilities() {
    for path in [
        "src/runtime/reducer.rs",
        "src/runtime/durable.rs",
        "src/runtime/transition_table.rs",
        "src/runtime/writer.rs",
        "src/runtime/verify.rs",
        "src/api/transport/replay.rs",
    ] {
        let body = source(path);
        assert!(
            !body.contains("crate::capability") && !body.contains("super::capability"),
            "{path} is replay-critical and must not import capability modules"
        );
    }
}

#[test]
fn runtime_kernel_process_ownership_boundaries_are_single_source() {
    let kernel = source("src/kernel/mod.rs");
    assert!(
        kernel.contains("stable state model"),
        "kernel should own stable packet/state/recovery vocabulary only"
    );
    assert!(
        !kernel.contains("pub mod scheduler")
            && !kernel.contains("WorkerProcess")
            && !kernel.contains("TaskClaimRequest"),
        "kernel must not own scheduling or task lifecycle mutation"
    );

    let runtime = source("src/runtime/mod.rs");
    assert!(
        runtime.contains("pub mod event_bus")
            && runtime.contains("mod recovery_policy")
            && runtime.contains("mod reducer"),
        "runtime should own event emission projection, reducer transitions, and recovery policy"
    );
    assert!(runtime.contains("Ownership boundary:"));
    assert!(runtime.contains("event-bus"));
    assert!(runtime.contains("policy instead of duplicating"));

    let process = source("src/process/mod.rs");
    assert!(
        process.contains("owns autonomous agent loops and supervisor process lifecycle"),
        "process should own scheduling and task lifecycle orchestration"
    );
    assert!(process.contains("adapter code"));
}

#[test]
fn scheduler_adapter_uses_runtime_wakeup_taxonomy_and_process_lifecycle_boundary() {
    let body = source("src/process/scheduler/handler.rs");
    assert!(
        body.contains("pub use crate::runtime::event_bus::WakeupKind as SchedulerWakeupKind"),
        "scheduler adapter should consume runtime-owned wakeup kinds instead of defining duplicate event taxonomy"
    );
    assert!(
        !body.contains("enum SchedulerWakeupKind"),
        "scheduler adapter must not own lifecycle/recovery wakeup taxonomy"
    );
    assert!(
        body.contains("trait SchedulerLifecycleBoundary")
            && body.contains("impl SchedulerLifecycleBoundary for WorkerProcess"),
        "scheduler adapter should delegate lifecycle mutation to process-owned WorkerProcess"
    );
    assert!(
        body.contains("EvalVerdict") && body.contains("LearningCandidate"),
        "adapter should explicitly ignore runtime wakeups outside task lifecycle"
    );
}

#[test]
fn process_browser_router_files_are_compatibility_shims_only() {
    for path in ["src/process/agent/router.rs", "src/process/agent/sse.rs"] {
        let body = source(path);
        assert!(
            body.contains("Re-export shim") && body.contains("pub use crate::capability::llm::"),
            "{path} should only re-export the capability-owned implementation"
        );
        assert!(
            !body.contains("reqwest")
                && !body.contains("TcpStream")
                && !body.contains("parse_sse_body(")
                && !body.contains("decode_chunked_body("),
            "{path} must not regain browser-router or SSE implementation logic"
        );
    }
}

#[test]
fn supervisor_uses_domain_plan_types_not_tooling_plan_types() {
    for path in [
        "src/process/supervisor/process.rs",
        "src/api/routes/supervisor/control.rs",
    ] {
        let body = source(path);
        assert!(
            !body.contains("capability::tooling::mcp_tools::canon_plan"),
            "{path} must not depend on tooling-owned plan types"
        );
        assert!(
            body.contains("domain::plan") || path.ends_with("control.rs"),
            "{path} should route plan state through the domain plan module"
        );
    }
}

#[test]
fn process_code_uses_scheduler_plan_store_for_plan_file_io() {
    for path in [
        "src/process/supervisor/process.rs",
        "src/api/routes/supervisor/control.rs",
        "src/process/scheduler/dispatch.rs",
    ] {
        let body = source(path);
        assert!(
            body.contains("process::scheduler::plan_store"),
            "{path} should import plan file I/O from process::scheduler::plan_store"
        );
        assert!(
            !body.contains("domain::plan::{load_plan")
                && !body.contains("domain::plan::{save_plan")
                && !body.contains("domain::plan::{load_plan,")
                && !body.contains("domain::plan::{save_plan,"),
            "{path} must not import plan file I/O from domain::plan"
        );
    }
}

#[test]
fn supervisor_lifecycle_does_not_save_plan_json_status() {
    let body = source("src/process/supervisor/process.rs");
    assert!(
        body.contains("append_status_change_patch"),
        "supervisor lifecycle should append status changes through the plan patch TLog"
    );
    assert!(
        body.contains("attach_supervisor_execution_evidence"),
        "supervisor lifecycle should attach accepted execution evidence before completion"
    );
    assert!(
        body.contains("load_plan_read_model"),
        "supervisor lifecycle should read projected plan state instead of raw plan status"
    );
    assert!(
        body.contains("self.active.is_some() && !tlog_submitted"),
        "active supervisor completion should require accepted receipt submission before Done"
    );
}

#[test]
fn scheduler_dispatch_reads_projected_plan_state() {
    let body = source("src/process/scheduler/dispatch.rs");
    assert!(
        body.contains("load_plan_read_model"),
        "scheduler dispatch should select ready nodes from the projected read model"
    );
    assert!(
        !body.contains("supervisor writes NodeStatus::Running to plan.json")
            && !body.contains("supervisor writes plan.json"),
        "scheduler comments must not treat raw plan.json status as lifecycle truth"
    );
}

#[test]
fn task_lifecycle_http_details_live_in_dispatch_task_client() {
    let task_client = source("src/process/dispatch/task_client.rs");
    assert!(task_client.contains("/v1/task/claim"));
    assert!(task_client.contains("/v1/task/heartbeat"));
    assert!(task_client.contains("/v1/task/complete"));
    assert!(task_client.contains("/v1/task/fail"));

    for path in [
        "src/process/scheduler/dispatch.rs",
        "src/process/agent/worker.rs",
        "src/process/agent/loop_driver/mod.rs",
    ] {
        let body = source(path);
        assert!(
            body.contains("TaskClient") || body.contains("process::scheduler::run_wave"),
            "{path} should use TaskClient or delegate to the scheduler"
        );
        assert!(
            !body.contains("/v1/task/claim")
                && !body.contains("/v1/task/heartbeat")
                && !body.contains("/v1/task/complete")
                && !body.contains("/v1/task/fail"),
            "{path} must not hard-code task lifecycle endpoints"
        );
    }
}

#[test]
fn supervisor_rejects_completion_without_plan_evidence() {
    let root = contract_root("empty-evidence");
    let _ = std::fs::remove_dir_all(&root);
    write_plan(&root, Vec::new());
    let mut worker = worker_process(&root);

    let claim = worker
        .claim_task(TaskClaimRequest {
            node_id: "node-1".to_string(),
            worker_id: "worker-a".to_string(),
            idempotency_key: 99,
            lease_ttl_ms: Some(60_000),
        })
        .expect("claim should succeed");
    let err = worker
        .complete_task(TaskCompleteRequest {
            node_id: "node-1".to_string(),
            worker_id: "worker-a".to_string(),
            claim_id: claim.claim_id,
        })
        .expect_err("completion without evidence should fail");

    assert!(err.contains("projected task evidence"));
    let plan = load_plan(&root);
    assert_ne!(plan.nodes[0].status, NodeStatus::Done);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn supervisor_accepts_legacy_task_evidence_by_attaching_completion_receipt() {
    let root = contract_root("legacy-unaccepted-evidence");
    let _ = std::fs::remove_dir_all(&root);
    write_plan(
        &root,
        vec![PlanEvidenceRef {
            path: "state/agent-evidence/node-1.md".to_string(),
            kind: "verification".to_string(),
            summary: "legacy evidence without accepted execution receipt metadata".to_string(),
            gate: String::new(),
            evidence: String::new(),
            receipt_hash: 0,
            accepted: false,
        }],
    );
    let mut worker = worker_process(&root);

    let claim = worker
        .claim_task(TaskClaimRequest {
            node_id: "node-1".to_string(),
            worker_id: "worker-a".to_string(),
            idempotency_key: 97,
            lease_ttl_ms: Some(60_000),
        })
        .expect("claim should succeed");
    let complete = worker
        .complete_task(TaskCompleteRequest {
            node_id: "node-1".to_string(),
            worker_id: "worker-a".to_string(),
            claim_id: claim.claim_id,
        })
        .expect("legacy task evidence should be completed by supervisor receipt attachment");

    assert_ne!(complete.receipt_hash, 0);
    let (read_model, plan_state) = load_plan_read_model(&root).expect("read model should project");
    let node = read_model
        .nodes
        .iter()
        .find(|node| node.id == "node-1")
        .expect("node should remain projected");
    assert_eq!(node.status, NodeStatus::Done);
    let plan_state = plan_state.expect("projected state should exist");
    assert!(!plan_state
        .nodes
        .values()
        .next()
        .expect("projected node should exist")
        .evidence
        .is_empty());
    let raw_plan = load_plan(&root);
    assert_eq!(raw_plan.nodes[0].status, NodeStatus::Done);
    assert!(!raw_plan.nodes[0].evidence.is_empty());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn supervisor_accepts_completion_by_attaching_execution_receipt_to_projected_evidence() {
    let root = contract_root("projected-evidence");
    let _ = std::fs::remove_dir_all(&root);
    write_plan(
        &root,
        vec![accepted_execution_evidence(
            "state/agent-evidence/node-1.md",
        )],
    );
    let mut worker = worker_process(&root);

    let claim = worker
        .claim_task(TaskClaimRequest {
            node_id: "node-1".to_string(),
            worker_id: "worker-a".to_string(),
            idempotency_key: 98,
            lease_ttl_ms: Some(60_000),
        })
        .expect("claim should succeed");
    let complete = worker
        .complete_task(TaskCompleteRequest {
            node_id: "node-1".to_string(),
            worker_id: "worker-a".to_string(),
            claim_id: claim.claim_id,
        })
        .expect("projected evidence should be completed with supervisor execution receipt");

    assert_ne!(complete.receipt_hash, 0);
    let (read_model, plan_state) = load_plan_read_model(&root).expect("read model should project");
    let node = read_model
        .nodes
        .iter()
        .find(|node| node.id == "node-1")
        .expect("node should remain projected");
    assert_eq!(node.status, NodeStatus::Done);
    let plan_state = plan_state.expect("projected state should exist");
    assert!(!plan_state
        .nodes
        .values()
        .next()
        .expect("projected node should exist")
        .evidence
        .is_empty());
    let raw_plan = load_plan(&root);
    assert_eq!(raw_plan.nodes[0].status, NodeStatus::Done);
    assert!(!raw_plan.nodes[0].evidence.is_empty());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn supervisor_rejects_completion_after_lease_expiry() {
    let root = contract_root("expired-lease");
    let _ = std::fs::remove_dir_all(&root);
    write_plan(
        &root,
        vec![accepted_execution_evidence(
            "state/agent-evidence/node-1.md",
        )],
    );
    let mut worker = worker_process(&root);

    let claim = worker
        .claim_task(TaskClaimRequest {
            node_id: "node-1".to_string(),
            worker_id: "worker-a".to_string(),
            idempotency_key: 100,
            lease_ttl_ms: Some(1),
        })
        .expect("claim should succeed");
    std::thread::sleep(Duration::from_millis(5));
    let err = worker
        .complete_task(TaskCompleteRequest {
            node_id: "node-1".to_string(),
            worker_id: "worker-a".to_string(),
            claim_id: claim.claim_id,
        })
        .expect_err("completion after lease expiry should fail");

    assert!(err.contains("expired"));
    let plan = load_plan(&root);
    assert_ne!(plan.nodes[0].status, NodeStatus::Done);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn supervisor_task_lifecycle_responses_are_receipt_backed() {
    let root = contract_root("lifecycle-receipts");
    let _ = std::fs::remove_dir_all(&root);
    write_plan(
        &root,
        vec![accepted_execution_evidence(
            "state/agent-evidence/node-1.md",
        )],
    );
    let mut worker = worker_process(&root);

    let claim = worker
        .claim_task(TaskClaimRequest {
            node_id: "node-1".to_string(),
            worker_id: "worker-a".to_string(),
            idempotency_key: 101,
            lease_ttl_ms: Some(60_000),
        })
        .expect("claim should succeed");
    assert_ne!(claim.receipt_hash, 0);
    assert!(!claim.tlog_submitted);

    let heartbeat = worker
        .heartbeat_task(TaskHeartbeatRequest {
            node_id: "node-1".to_string(),
            worker_id: "worker-a".to_string(),
            claim_id: claim.claim_id,
            lease_ttl_ms: Some(60_000),
        })
        .expect("heartbeat should succeed");
    assert_ne!(heartbeat.receipt_hash, 0);
    assert!(!heartbeat.tlog_submitted);

    let complete = worker
        .complete_task(TaskCompleteRequest {
            node_id: "node-1".to_string(),
            worker_id: "worker-a".to_string(),
            claim_id: claim.claim_id,
        })
        .expect("completion with evidence should succeed");
    assert_ne!(complete.receipt_hash, 0);
    assert!(!complete.tlog_submitted);

    let raw_plan = load_plan(&root);
    assert_eq!(raw_plan.nodes[0].status, NodeStatus::Done);
    let (read_model, plan_state) = load_plan_read_model(&root).expect("read model should project");
    assert!(plan_state.is_some());
    assert_eq!(read_model.nodes[0].status, NodeStatus::Done);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn supervisor_failure_response_is_receipt_backed() {
    let root = contract_root("failure-receipts");
    let _ = std::fs::remove_dir_all(&root);
    write_plan(&root, Vec::new());
    let mut worker = worker_process(&root);

    let claim = worker
        .claim_task(TaskClaimRequest {
            node_id: "node-1".to_string(),
            worker_id: "worker-a".to_string(),
            idempotency_key: 102,
            lease_ttl_ms: Some(60_000),
        })
        .expect("claim should succeed");
    let failed = worker
        .fail_task(TaskFailRequest {
            node_id: "node-1".to_string(),
            worker_id: "worker-a".to_string(),
            claim_id: claim.claim_id,
            retry_after_ms: 0,
        })
        .expect("failure should succeed");

    assert_ne!(failed.receipt_hash, 0);
    assert!(!failed.tlog_submitted);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn supervisor_task_lifecycle_submits_receipts_through_command_ingress() {
    let source = source("src/process/supervisor/process.rs");

    assert!(source.contains("KernelCommand::SubmitEvidence"));
    assert!(source.contains("CommandEnvelope::new(receipt.receipt_hash"));
    assert!(source.contains("\"payload_tag\": \"SubmitEvidence\""));
    assert!(source.contains("\"source\": \"supervisor_task_lifecycle\""));
}

#[test]
fn task_lifecycle_receipt_is_tlog_auditable_via_command_ingress() {
    let receipt = TaskLifecycleReceipt::claim("node-1", "worker-a", 7, 101, 50_000, 10_000);
    let command = Command::SubmitEvidence(receipt.submission());
    let envelope = CommandEnvelope::new(receipt.receipt_hash, command);
    let mut state = State {
        phase: Phase::Execute,
        ..State::default()
    };
    state.packet.bind_ready_task();
    state.gates.invariant = Gate::pass(Evidence::InvariantProof);
    state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
    state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);
    state.gates.plan = Gate::pass(Evidence::TaskReady);
    let mut tlog: TLog = Vec::new();

    ai::api::routes::handle_envelope(&mut state, &mut tlog, RuntimeConfig::default(), envelope)
        .expect("lifecycle receipt should enter command ingress");

    let event = tlog
        .iter()
        .find(|event| event.cause == Cause::EvidenceSubmitted)
        .expect("receipt submission should be persisted");
    assert_eq!(event.evidence, Evidence::ExecutionReceipt);
    assert_eq!(event.affected_gate, Some(GateId::Execution));
    assert_eq!(event.api_command_id, receipt.receipt_hash);
    assert_eq!(
        event.capability_registry_projection,
        CapabilityRegistry::canonical().projection()
    );
}
