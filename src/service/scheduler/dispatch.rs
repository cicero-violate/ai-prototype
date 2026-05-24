//! DAG-driven task dispatch — called by handler, not on a timer.
//!
//! Given the current plan state, finds ready nodes (pending nodes whose
//! dependencies are all done), claims each via the supervisor, spawns one
//! bounded LoopDriver thread per node, joins all threads, and reports
//! complete/fail back to the supervisor.
//!
//! Entry point: `run_wave(config, tag, cycle_num)`.
//! Caller: `process/agent/loop_driver` after a planning turn identifies
//! ready nodes.  This module does not poll; it is called on demand.

use std::collections::{HashMap, HashSet};
use std::thread;

use serde_json::Value;

use crate::api::protocol::{Command, CommandEnvelope};
use crate::capability::orchestration::{ChildCompleteRecord, WaveRecord};
use crate::domain::plan::{ready_nodes, NodeStatus, PlanDag, PlanNode};
use crate::runtime::workspace::workspace_state_dir;
use crate::service::agent::loop_driver::common::stable_agent_hash;
use crate::service::agent::loop_driver::http::{
    agent_command_url, get_json_body_local, post_json_local,
};
use crate::service::agent::{AgentLoopConfig, LoopDriver, MAX_EXECUTOR_COUNT};
use crate::service::dispatch::task_client::{TaskClaim, TaskClient};
use crate::service::scheduler::plan_store::{
    append_status_change_patch, load_plan, load_plan_read_model,
};

/// Dispatch one wave: claim all currently-ready plan nodes, spawn one
/// LoopDriver thread per node, join all threads, report complete/fail.
///
/// Returns `true` if the DAG had ready nodes and the wave ran, `false` if
/// there was nothing to schedule (caller should fall back to execute turns).
pub(crate) fn run_wave(config: &AgentLoopConfig, tag: &str, cycle_num: u64) -> bool {
    let (plan, _) = load_plan_read_model(&config.working_dir)
        .unwrap_or_else(|_| (load_plan(&config.working_dir), None));

    if plan.nodes.is_empty() {
        return false;
    }

    // Lint before dispatch — bail on hard graph errors.
    if !lint_plan(&plan, tag) {
        eprintln!("[{tag}] DAG scheduler: lint failed — skipping wave  cycle={cycle_num}");
        return false;
    }

    let ready_total = ready_nodes(&plan).len();
    let mut ready_owned: Vec<PlanNode> = ready_nodes(&plan).into_iter().cloned().collect();
    ready_owned.sort_by(|a, b| a.id.cmp(&b.id));

    // Cap by configured max AND by available browser tabs (resource scheduler).
    let agent_cap = config.executor_count.clamp(1, MAX_EXECUTOR_COUNT) as usize;
    let tab_cap = query_available_tab_count(config);
    let wave_cap = agent_cap.min(tab_cap);
    if tab_cap < agent_cap {
        eprintln!(
            "[{tag}] DAG scheduler: tab_cap={tab_cap} < agent_cap={agent_cap} — throttling wave"
        );
    }
    ready_owned.truncate(wave_cap);

    if ready_owned.is_empty() {
        return false;
    }

    eprintln!(
        "[{tag}] DAG scheduler  cycle={cycle_num}  wave_size={}  ready_total={}  total_nodes={}",
        ready_owned.len(),
        ready_total,
        plan.nodes.len(),
    );

    // Record wave dispatch intent in the kernel TLog before spawning.
    let cmd_url = agent_command_url(config);
    let wave_id = wave_id_for(tag, cycle_num);
    let node_ids_hash = node_ids_hash(&ready_owned);
    let wave_record = WaveRecord::new(
        wave_id,
        stable_agent_hash(tag.as_bytes()),
        cycle_num,
        ready_owned.len() as u16,
        node_ids_hash,
    );
    if let Some(url) = &cmd_url {
        submit_wave_dispatch(url, wave_record, tag);
    }

    // ── Claim phase ───────────────────────────────────────────────────────────
    //
    // If the supervisor is reachable, claim each node via its HTTP API so the
    // supervisor becomes the authoritative source of truth for task lifecycle.
    // The supervisor appends TLog-backed plan patches; dispatch reads the
    // projected plan instead of relying on raw plan.json status.
    //
    // Without a supervisor, append accepted TLog-backed status patches directly
    // (tests, standalone mode). Do not mutate raw plan.json lifecycle state.
    let sv_url = supervisor_url_from_config(config);
    let scheduler_worker_id = format!("dag-{tag}-cycle-{cycle_num}");

    // claim_id per node_id — only populated in the supervisor path.
    let mut claims: HashMap<String, u64> = HashMap::new();

    if let Some(ref sv) = sv_url {
        let mut claimed = Vec::new();
        for node in &ready_owned {
            match claim_node_via_supervisor(sv, &node.id, &scheduler_worker_id, wave_id) {
                Ok(claim_id) => {
                    claims.insert(node.id.clone(), claim_id);
                    claimed.push(node.clone());
                }
                Err(e) => {
                    eprintln!(
                        "[{tag}] DAG scheduler: skipping node={} — claim: {e}",
                        node.id
                    );
                }
            }
        }
        ready_owned = claimed;
    } else {
        for node in &ready_owned {
            if let Err(e) =
                append_status_change_patch(&config.working_dir, &node.id, &NodeStatus::Running)
            {
                eprintln!(
                    "[{tag}] DAG scheduler: append running state failed for node={}: {e}",
                    node.id
                );
            }
        }
    }

    if ready_owned.is_empty() {
        return false;
    }

    // Spawn one thread per claimed/ready node.
    let handles: Vec<(String, thread::JoinHandle<()>)> = ready_owned
        .iter()
        .map(|node| {
            let node_id = node.id.clone();
            let child = child_config(config, node);
            eprintln!(
                "[{tag}] DAG scheduler: spawning task={node_id}  title={:?}",
                node.title
            );
            let handle = thread::spawn(move || {
                LoopDriver::new(child).run_all_agents();
            });
            (node_id, handle)
        })
        .collect();

    // ── Join phase ────────────────────────────────────────────────────────────
    //
    // Supervisor path: report complete/fail via HTTP — supervisor appends
    // TLog-backed lifecycle patches.
    for (node_id, handle) in handles {
        let panicked = handle.join().is_err();

        if let Some(ref sv) = sv_url {
            let claim_id = claims.get(&node_id).copied().unwrap_or(0);
            if panicked {
                fail_node_via_supervisor(sv, &node_id, &scheduler_worker_id, claim_id, 60_000);
            } else {
                complete_node_via_supervisor(sv, &node_id, &scheduler_worker_id, claim_id);
            }
        } else {
            let status = if panicked {
                NodeStatus::Failed
            } else {
                NodeStatus::Done
            };
            if let Err(e) = append_status_change_patch(&config.working_dir, &node_id, &status) {
                eprintln!(
                    "[{tag}] DAG scheduler: append final state failed for node={node_id}: {e}"
                );
            }
        }

        eprintln!(
            "[{tag}] DAG scheduler: task={node_id}  {}  cycle={cycle_num}",
            if panicked {
                "panicked→failed"
            } else {
                "finished→done"
            }
        );
        let child_record =
            ChildCompleteRecord::new(wave_id, stable_agent_hash(node_id.as_bytes()), panicked);
        if let Some(url) = &cmd_url {
            submit_child_complete(url, child_record, &node_id, tag);
        }
    }

    let (plan_after, _) = load_plan_read_model(&config.working_dir)
        .unwrap_or_else(|_| (load_plan(&config.working_dir), None));
    let remaining = plan_after
        .nodes
        .iter()
        .filter(|n| n.status == NodeStatus::Pending || n.status == NodeStatus::Running)
        .count();
    eprintln!(
        "[{tag}] DAG scheduler: wave complete  remaining_tasks={remaining}  cycle={cycle_num}"
    );

    true
}

// ── Supervisor claim helpers ──────────────────────────────────────────────────

fn supervisor_url_from_config(config: &AgentLoopConfig) -> Option<String> {
    config
        .supervisor_port
        .map(|port| format!("http://127.0.0.1:{port}"))
        .or_else(|| {
            std::env::var("SUPERVISOR_PORT")
                .ok()
                .and_then(|v| v.parse::<u16>().ok())
                .map(|port| format!("http://127.0.0.1:{port}"))
        })
}

fn claim_node_via_supervisor(
    sv_url: &str,
    node_id: &str,
    worker_id: &str,
    idempotency_key: u64,
) -> Result<u64, String> {
    TaskClient::new(sv_url)
        .claim(node_id, worker_id, idempotency_key, 600_000)
        .map(|claim| claim.claim_id)
}

fn complete_node_via_supervisor(sv_url: &str, node_id: &str, worker_id: &str, claim_id: u64) {
    let claim = TaskClaim {
        node_id: node_id.to_string(),
        claim_id,
        expires_at_ms: 0,
        receipt_hash: 0,
        tlog_submitted: false,
    };
    match TaskClient::new(sv_url).complete(&claim, worker_id) {
        Ok(_) => {}
        Err(e) => eprintln!("[dag-scheduler] complete failed for node={node_id}: {e}"),
    }
}

fn fail_node_via_supervisor(
    sv_url: &str,
    node_id: &str,
    worker_id: &str,
    claim_id: u64,
    retry_after_ms: u64,
) {
    let claim = TaskClaim {
        node_id: node_id.to_string(),
        claim_id,
        expires_at_ms: 0,
        receipt_hash: 0,
        tlog_submitted: false,
    };
    match TaskClient::new(sv_url).fail(&claim, worker_id, retry_after_ms) {
        Ok(_) => {}
        Err(e) => eprintln!("[dag-scheduler] fail failed for node={node_id}: {e}"),
    }
}

// ── Resource scheduler ────────────────────────────────────────────────────────

fn query_available_tab_count(config: &AgentLoopConfig) -> usize {
    let Some(ref base_url) = config.browser_router_url else {
        return usize::MAX;
    };
    let url = format!("{base_url}/tabs");
    match get_json_body_local(&url) {
        Ok(Value::Object(ref obj)) => obj
            .get("targets")
            .and_then(Value::as_array)
            .map(|arr| arr.len().max(1))
            .unwrap_or(usize::MAX),
        Ok(Value::Array(ref tabs)) => tabs.len().max(1),
        Ok(_) => usize::MAX,
        Err(e) => {
            eprintln!("[dag-scheduler] /tabs query failed ({e}) — no tab cap applied");
            usize::MAX
        }
    }
}

// ── DAG linter ────────────────────────────────────────────────────────────────

fn lint_plan(plan: &PlanDag, tag: &str) -> bool {
    let mut ok = true;
    let mut node_ids: HashSet<&str> = HashSet::new();

    for node in &plan.nodes {
        if node.id.trim().is_empty() {
            eprintln!("[{tag}] DAG lint ERROR: node has empty id");
            ok = false;
            continue;
        }
        if !node_ids.insert(node.id.as_str()) {
            eprintln!("[{tag}] DAG lint ERROR: duplicate node id '{}'", node.id);
            ok = false;
        }
    }

    for edge in &plan.edges {
        if !node_ids.contains(edge.from.as_str()) {
            eprintln!(
                "[{tag}] DAG lint ERROR: edge.from='{}' references unknown node",
                edge.from
            );
            ok = false;
        }
        if !node_ids.contains(edge.to.as_str()) {
            eprintln!(
                "[{tag}] DAG lint ERROR: edge.to='{}' references unknown node",
                edge.to
            );
            ok = false;
        }
    }

    for node in &plan.nodes {
        if node.status == NodeStatus::Pending {
            if node.title.trim().is_empty() {
                eprintln!(
                    "[{tag}] DAG lint ERROR: node '{}' has empty title — mini-agent would have no task",
                    node.id
                );
                ok = false;
            }
            if node.description.trim().is_empty() {
                eprintln!(
                    "[{tag}] DAG lint ERROR: node '{}' has empty description — mini-agent would have no success criterion",
                    node.id
                );
                ok = false;
            }
        }
    }

    if dag_has_cycle(plan) {
        eprintln!("[{tag}] DAG lint ERROR: cycle detected — no node will ever become ready");
        ok = false;
    }

    ok
}

fn dag_has_cycle(plan: &PlanDag) -> bool {
    let mut visited: HashSet<&str> = HashSet::new();
    let mut in_stack: HashSet<&str> = HashSet::new();
    for node in &plan.nodes {
        if dfs_cycle(node.id.as_str(), plan, &mut visited, &mut in_stack) {
            return true;
        }
    }
    false
}

fn dfs_cycle<'a>(
    id: &'a str,
    plan: &'a PlanDag,
    visited: &mut HashSet<&'a str>,
    in_stack: &mut HashSet<&'a str>,
) -> bool {
    if in_stack.contains(id) {
        return true;
    }
    if visited.contains(id) {
        return false;
    }
    visited.insert(id);
    in_stack.insert(id);
    for edge in &plan.edges {
        if edge.from == id && dfs_cycle(edge.to.as_str(), plan, visited, in_stack) {
            return true;
        }
    }
    in_stack.remove(id);
    false
}

// ── Wave identity ─────────────────────────────────────────────────────────────

fn wave_id_for(tag: &str, cycle_num: u64) -> u64 {
    let h = stable_agent_hash(tag.as_bytes());
    h.wrapping_mul(0x0000_0100_0000_01b3)
        .wrapping_add(cycle_num)
        .max(1)
}

fn node_ids_hash(nodes: &[PlanNode]) -> u64 {
    let mut ids: Vec<&str> = nodes.iter().map(|n| n.id.as_str()).collect();
    ids.sort_unstable();
    stable_agent_hash(ids.join(",").as_bytes())
}

// ── Kernel submissions ────────────────────────────────────────────────────────

fn submit_wave_dispatch(url: &str, record: WaveRecord, tag: &str) {
    if !record.is_contract_valid() {
        return;
    }
    let envelope =
        CommandEnvelope::new(record.contract_hash(), Command::SubmitWaveDispatch(record));
    let payload = serde_json::json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitWaveDispatch",
        "payload": {
            "wave_id": record.wave_id,
            "parent_hash": record.parent_hash,
            "cycle": record.cycle,
            "node_count": record.node_count,
            "node_ids_hash": record.node_ids_hash,
        },
    });
    match post_json_local(url, &payload) {
        Ok(s) if (200..300).contains(&s) => {}
        Ok(s) => eprintln!("[{tag}] dag: WaveDispatch kernel POST failed: status={s}"),
        Err(e) => eprintln!("[{tag}] dag: WaveDispatch kernel POST error: {e}"),
    }
}

fn submit_child_complete(url: &str, record: ChildCompleteRecord, node_id: &str, tag: &str) {
    if !record.is_contract_valid() {
        return;
    }
    let envelope =
        CommandEnvelope::new(record.contract_hash(), Command::SubmitChildComplete(record));
    let payload = serde_json::json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitChildComplete",
        "payload": {
            "wave_id": record.wave_id,
            "node_id_hash": record.node_id_hash,
            "exit_status": record.exit_status,
        },
    });
    match post_json_local(url, &payload) {
        Ok(s) if (200..300).contains(&s) => {}
        Ok(s) => {
            eprintln!("[{tag}] dag: ChildComplete kernel POST failed: node={node_id} status={s}")
        }
        Err(e) => eprintln!("[{tag}] dag: ChildComplete kernel POST error: node={node_id} {e}"),
    }
}

fn child_config(parent: &AgentLoopConfig, node: &PlanNode) -> AgentLoopConfig {
    let sse_chunks_dir = workspace_state_dir(&parent.project_dir)
        .join("agent_state")
        .join("sse-chunks");
    AgentLoopConfig {
        execute_turns: parent.execute_turns,
        turn_retry_limit: parent.turn_retry_limit,
        loop_sleep_ms: parent.loop_sleep_ms,
        agent_count: 1,
        executor_count: 1,
        working_dir: parent.working_dir.clone(),
        sse_chunks_dir,
        project_dir: parent.project_dir.clone(),
        mcp_connector_url: parent.mcp_connector_url.clone(),
        router_turn_max_ms: parent.router_turn_max_ms,
        router_first_capture_ms: parent.router_first_capture_ms,
        router_idle_ms: parent.router_idle_ms,
        worker_port: parent.worker_port,
        supervisor_port: parent.supervisor_port,
        browser_router_url: parent.browser_router_url.clone(),
        cert_max_steps: parent.cert_max_steps,
        domain: Some(node.title.clone()),
        metric: Some(node.description.clone()),
        plan_node_id: Some(node.id.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::plan::PlanEdge;

    fn test_node(id: &str) -> PlanNode {
        PlanNode {
            id: id.to_string(),
            title: format!("{id} task"),
            description: format!("{id} success criterion"),
            status: NodeStatus::Pending,
            assignee: None,
            score_axes: Vec::new(),
            files: Vec::new(),
            evidence: Vec::new(),
        }
    }

    fn test_edge(from: &str, to: &str) -> PlanEdge {
        PlanEdge {
            from: from.to_string(),
            to: to.to_string(),
        }
    }

    fn test_plan(nodes: Vec<PlanNode>, edges: Vec<PlanEdge>) -> PlanDag {
        PlanDag {
            version: 1,
            nodes,
            edges,
            ..Default::default()
        }
    }

    #[test]
    fn pre_dispatch_lint_accepts_well_formed_dag() {
        let mut root = test_node("root");
        root.status = NodeStatus::Done;
        let child = test_node("child");

        assert!(lint_plan(
            &test_plan(vec![root, child], vec![test_edge("root", "child")]),
            "test"
        ));
    }

    #[test]
    fn pre_dispatch_lint_rejects_dependency_cycle() {
        assert!(!lint_plan(
            &test_plan(
                vec![test_node("a"), test_node("b")],
                vec![test_edge("a", "b"), test_edge("b", "a")],
            ),
            "test"
        ));
    }

    #[test]
    fn pre_dispatch_lint_rejects_dangling_dependency_edges() {
        assert!(!lint_plan(
            &test_plan(vec![test_node("a")], vec![test_edge("missing", "a")]),
            "test"
        ));
        assert!(!lint_plan(
            &test_plan(vec![test_node("a")], vec![test_edge("a", "missing")]),
            "test"
        ));
    }

    #[test]
    fn pre_dispatch_lint_rejects_duplicate_node_ids() {
        assert!(!lint_plan(
            &test_plan(vec![test_node("dup"), test_node("dup")], Vec::new()),
            "test"
        ));
    }

    #[test]
    fn pre_dispatch_lint_rejects_empty_required_pending_node_fields() {
        let mut missing_id = test_node("");
        missing_id.title = "task".to_string();
        missing_id.description = "criterion".to_string();
        assert!(!lint_plan(&test_plan(vec![missing_id], Vec::new()), "test"));

        let mut missing_title = test_node("missing-title");
        missing_title.title = "   ".to_string();
        assert!(!lint_plan(
            &test_plan(vec![missing_title], Vec::new()),
            "test"
        ));

        let mut missing_description = test_node("missing-description");
        missing_description.description = "   ".to_string();
        assert!(!lint_plan(
            &test_plan(vec![missing_description], Vec::new()),
            "test"
        ));
    }
}
