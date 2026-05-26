//! JSONL trace for the task reasoning chain.
//!
//! The supervisor already owns durable task truth through `state/plan.json` and
//! the plan patch TLog. This file adds a human-readable JSONL readout around the
//! dispatch loop so each claimed task has an inspectable chain from question to
//! learning.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use serde_json::{json, Value};

use crate::domain::plan::PlanNode;
use crate::service::agent::worker::ActiveClaim;

pub const REASONING_TRACE_FILE: &str = "state/reasoning-loop.jsonl";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReasoningStage {
    Truth,
    Judgment,
    BetterQuestion,
    Goal,
    Plan,
    Task,
    Action,
    Evidence,
    Verification,
    Learning,
}

impl ReasoningStage {
    fn as_str(self) -> &'static str {
        match self {
            Self::Truth => "Truth",
            Self::Judgment => "Judgment",
            Self::BetterQuestion => "BetterQuestion",
            Self::Goal => "Goal",
            Self::Plan => "Plan",
            Self::Task => "Task",
            Self::Action => "Action",
            Self::Evidence => "Evidence",
            Self::Verification => "Verification",
            Self::Learning => "Learning",
        }
    }
}

pub fn append_task_start_trace(
    project_dir: &Path,
    assignment: &PlanNode,
    claim: &ActiveClaim,
    worker_id: &str,
) {
    let trace_id = trace_id(claim);
    append_stage(
        project_dir,
        trace_id,
        ReasoningStage::Truth,
        assignment,
        claim,
        worker_id,
        &[],
        json!({
            "source": "plan_read_model",
            "node_id": assignment.id,
            "title": assignment.title,
            "description": assignment.description,
            "status": format!("{:?}", assignment.status),
            "files": assignment.files,
            "score_axes": assignment.score_axes,
            "evidence_refs": assignment.evidence.len(),
        }),
        &["plan/TLog replay selected this node as ready"],
        &["plan read model may lag active filesystem changes until replay"],
        0.90,
    );
    append_stage(
        project_dir,
        trace_id,
        ReasoningStage::Judgment,
        assignment,
        claim,
        worker_id,
        &[ReasoningStage::Truth],
        json!({
            "decision": "execute_claimed_task",
            "lease_ttl_ms": claim.expires_at_ms.saturating_sub(now_ms()),
            "claim_receipt_hash": claim.receipt_hash,
            "tlog_submitted": claim.tlog_submitted,
        }),
        &["a non-zero claim_id is the execution authority"],
        &["worker can still fail to produce task evidence"],
        0.86,
    );
    append_stage(
        project_dir,
        trace_id,
        ReasoningStage::BetterQuestion,
        assignment,
        claim,
        worker_id,
        &[ReasoningStage::Judgment],
        json!({
            "question": format!(
                "What evidence proves plan node `{}` is complete?",
                assignment.id
            ),
            "original_task_title": assignment.title,
        }),
        &["the useful question is evidence-oriented, not just action-oriented"],
        &["success criteria depend on the plan node description"],
        0.84,
    );
    append_stage(
        project_dir,
        trace_id,
        ReasoningStage::Goal,
        assignment,
        claim,
        worker_id,
        &[ReasoningStage::BetterQuestion],
        json!({
            "goal": assignment.description,
            "completion_contract": "worker succeeds and verification/evidence gates pass",
        }),
        &["plan node description is the local goal contract"],
        &["the description may be broad and require judgment by the spawned agent"],
        0.82,
    );
    append_stage(
        project_dir,
        trace_id,
        ReasoningStage::Plan,
        assignment,
        claim,
        worker_id,
        &[ReasoningStage::Goal],
        json!({
            "plan": [
                "run LoopDriver for the claimed node",
                "keep the lease alive with heartbeat",
                "submit/observe evidence",
                "complete the claim only after success"
            ],
        }),
        &["dispatch loop owns task lifecycle, LoopDriver owns work execution"],
        &["heartbeat success does not imply task success"],
        0.88,
    );
    append_stage(
        project_dir,
        trace_id,
        ReasoningStage::Task,
        assignment,
        claim,
        worker_id,
        &[ReasoningStage::Plan],
        json!({
            "node_id": assignment.id,
            "claim_id": claim.claim_id,
            "worker_id": worker_id,
            "domain": assignment.title,
            "metric": assignment.description,
        }),
        &["one claimed plan node maps to one bounded runner task"],
        &["task may be retried if action or verification fails"],
        0.91,
    );
}

pub fn append_task_finish_trace(
    project_dir: &Path,
    assignment: &PlanNode,
    claim: &ActiveClaim,
    worker_id: &str,
    succeeded: bool,
) {
    let trace_id = trace_id(claim);
    append_stage(
        project_dir,
        trace_id,
        ReasoningStage::Action,
        assignment,
        claim,
        worker_id,
        &[ReasoningStage::Task],
        json!({
            "action": "LoopDriver::run_all_agents",
            "succeeded": succeeded,
        }),
        &["action result is the direct return value of the heartbeat-wrapped driver"],
        &["action success still needs durable completion/evidence receipts"],
        if succeeded { 0.86 } else { 0.65 },
    );
    append_stage(
        project_dir,
        trace_id,
        ReasoningStage::Evidence,
        assignment,
        claim,
        worker_id,
        &[ReasoningStage::Action],
        json!({
            "evidence_observed": succeeded,
            "expected_evidence_family": "task completion artifacts and supervisor receipts",
        }),
        &["dispatch treats successful driver completion as observed progress input"],
        &["this trace is observational; kernel/plan receipts remain the durable contract"],
        if succeeded { 0.80 } else { 0.55 },
    );
    append_stage(
        project_dir,
        trace_id,
        ReasoningStage::Verification,
        assignment,
        claim,
        worker_id,
        &[ReasoningStage::Evidence],
        json!({
            "passed": succeeded,
            "next_supervisor_transition": if succeeded { "complete_claim" } else { "fail_claim" },
        }),
        &["claim completion is gated after action/evidence observation"],
        &["failed verification routes to retry rather than silent completion"],
        if succeeded { 0.84 } else { 0.72 },
    );
    append_stage(
        project_dir,
        trace_id,
        ReasoningStage::Learning,
        assignment,
        claim,
        worker_id,
        &[ReasoningStage::Verification],
        json!({
            "lesson": if succeeded {
                "claimed task produced enough progress to advance"
            } else {
                "task needs retry or better evidence before completion"
            },
            "retry_recommended": !succeeded,
        }),
        &["learning is appended even for failures so later planning can inspect outcomes"],
        &["this is local trace learning, not policy promotion"],
        if succeeded { 0.78 } else { 0.70 },
    );
}

#[allow(clippy::too_many_arguments)]
fn append_stage(
    project_dir: &Path,
    trace_id: u64,
    stage: ReasoningStage,
    assignment: &PlanNode,
    claim: &ActiveClaim,
    worker_id: &str,
    input_stages: &[ReasoningStage],
    output: Value,
    assumptions: &[&str],
    risks: &[&str],
    confidence: f64,
) {
    let path = project_dir.join(REASONING_TRACE_FILE);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let input_refs: Vec<_> = input_stages
        .iter()
        .map(|stage| format!("{}:{trace_id}", stage.as_str()))
        .collect();
    let record = json!({
        "schema": "canon.reasoning_loop.v1",
        "ts": now_ms(),
        "trace_id": trace_id,
        "stage": stage.as_str(),
        "record_id": format!("{}:{trace_id}", stage.as_str()),
        "input_refs": input_refs,
        "node_id": assignment.id,
        "claim_id": claim.claim_id,
        "worker_id": worker_id,
        "output": output,
        "assumptions": assumptions,
        "risks": risks,
        "confidence": confidence,
    });
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{record}");
    }
}

fn trace_id(claim: &ActiveClaim) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in claim
        .node_id
        .bytes()
        .chain([0xff])
        .chain(claim.claim_id.to_le_bytes())
    {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h.max(1)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::plan::{NodeStatus, PlanNode};

    fn claim() -> ActiveClaim {
        ActiveClaim {
            node_id: "node-a".to_string(),
            claim_id: 42,
            expires_at_ms: now_ms() + 60_000,
            receipt_hash: 777,
            tlog_submitted: true,
        }
    }

    fn node() -> PlanNode {
        PlanNode {
            id: "node-a".to_string(),
            title: "Implement trace".to_string(),
            description: "Add JSONL reasoning loop trace".to_string(),
            status: NodeStatus::Pending,
            assignee: None,
            score_axes: vec!["Determinism".to_string()],
            files: vec!["ai/src/service/dispatch/task_runner.rs".to_string()],
            evidence: Vec::new(),
        }
    }

    #[test]
    fn reasoning_trace_appends_all_task_stages_as_jsonl() {
        let temp = tempfile::tempdir().expect("tempdir");
        let node = node();
        let claim = claim();

        append_task_start_trace(temp.path(), &node, &claim, "worker-1");
        append_task_finish_trace(temp.path(), &node, &claim, "worker-1", true);

        let text = std::fs::read_to_string(temp.path().join(REASONING_TRACE_FILE))
            .expect("trace file exists");
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines.len(), 10);
        assert!(lines.iter().all(|line| {
            serde_json::from_str::<serde_json::Value>(line)
                .map(|value| value["schema"] == "canon.reasoning_loop.v1")
                .unwrap_or(false)
        }));
        assert!(lines[0].contains("\"stage\":\"Truth\""));
        assert!(lines[9].contains("\"stage\":\"Learning\""));
    }
}
