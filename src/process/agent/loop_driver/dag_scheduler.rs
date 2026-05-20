//! DAG-driven task scheduler for the agent loop.
//!
//! After each planning turn the loop driver checks whether the agent wrote a
//! structured plan DAG to `state/plan.json`.  If ready nodes exist (pending
//! nodes whose dependencies are all done), one child LoopDriver thread is
//! spawned per ready node.  The scheduler joins all threads, updates node
//! statuses, and returns.  The next cycle's planning turn then reviews what
//! completed and decides what to schedule next.

use std::collections::HashSet;
use std::thread;

use serde_json::Value;

use crate::api::protocol::{Command, CommandEnvelope};
use crate::capability::orchestration::{ChildCompleteRecord, WaveRecord};
use crate::capability::tooling::mcp_tools::canon_plan::{
    load_plan, ready_nodes, save_plan, NodeStatus, PlanDag, PlanNode,
};
use crate::process::agent::{AgentLoopConfig, LoopDriver, MAX_MINI_AGENT_COUNT};
use crate::runtime::workspace::workspace_state_dir;

use super::common::stable_agent_hash;
use super::http::{agent_command_url, get_json_body_local, post_json_local};

/// Run one scheduling wave: spawn agents for all currently-ready plan nodes,
/// wait for them all to finish, then persist their final statuses.
///
/// Returns `true` if the DAG had ready nodes and the wave ran, `false` if
/// there was nothing to schedule (caller should fall back to execute turns).
pub(super) fn run_wave(config: &AgentLoopConfig, tag: &str, cycle_num: u64) -> bool {
    let plan = load_plan(&config.working_dir);

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
    let agent_cap = config.mini_agent_count.clamp(1, MAX_MINI_AGENT_COUNT) as usize;
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

    // Mark ready nodes as running before spawning so other agents don't double-pick them.
    let mut plan_mut = plan;
    for node in &ready_owned {
        if let Some(n) = plan_mut.nodes.iter_mut().find(|n| n.id == node.id) {
            n.status = NodeStatus::Running;
            n.assignee = Some(format!("cycle-{cycle_num}"));
        }
    }
    if let Err(e) = save_plan(&config.working_dir, &plan_mut) {
        eprintln!("[{tag}] DAG scheduler: save running state failed: {e}");
    }

    // Spawn one thread per ready node.
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

    // Join all threads and record outcomes.
    // Reload the plan first so agent writes (e.g. set_status via MCP) are preserved.
    let mut plan_final = load_plan(&config.working_dir);
    for (node_id, handle) in handles {
        let panicked = handle.join().is_err();
        if let Some(node) = plan_final.nodes.iter_mut().find(|n| n.id == node_id) {
            // Only update if the agent didn't already update it (e.g. via canon_plan_update).
            if node.status == NodeStatus::Running {
                node.status = if panicked {
                    NodeStatus::Failed
                } else {
                    NodeStatus::Done
                };
            }
        }
        eprintln!(
            "[{tag}] DAG scheduler: task={node_id}  {}  cycle={cycle_num}",
            if panicked { "panicked→failed" } else { "finished→done" }
        );
        // Record child completion in the kernel TLog.
        let child_record =
            ChildCompleteRecord::new(wave_id, stable_agent_hash(node_id.as_bytes()), panicked);
        if let Some(url) = &cmd_url {
            submit_child_complete(url, child_record, &node_id, tag);
        }
    }
    if let Err(e) = save_plan(&config.working_dir, &plan_final) {
        eprintln!("[{tag}] DAG scheduler: save final state failed: {e}");
    }

    let remaining = plan_final
        .nodes
        .iter()
        .filter(|n| n.status == NodeStatus::Pending || n.status == NodeStatus::Running)
        .count();
    eprintln!(
        "[{tag}] DAG scheduler: wave complete  remaining_tasks={remaining}  cycle={cycle_num}"
    );

    true
}

// ── Resource scheduler ────────────────────────────────────────────────────────

/// Query the browser-router's /tabs endpoint and return the number of live
/// tabs.  Used to cap the wave size so we never spawn more agents than there
/// are tabs to serve them.  Falls back to usize::MAX (no constraint) if the
/// router is unreachable or not configured.
fn query_available_tab_count(config: &AgentLoopConfig) -> usize {
    let Some(port) = config.worker_port else {
        return usize::MAX;
    };
    let url = format!("http://127.0.0.1:{port}/tabs");
    match get_json_body_local(&url) {
        Ok(Value::Array(tabs)) => tabs.len().max(1),
        Ok(_) => usize::MAX,
        Err(e) => {
            eprintln!("[dag-scheduler] /tabs query failed ({e}) — no tab cap applied");
            usize::MAX
        }
    }
}

// ── DAG linter ────────────────────────────────────────────────────────────────

/// Validate the plan graph before dispatch.  Returns `true` if the plan is
/// safe to run, `false` if a hard structural error was found (cycle, dangling
/// edge).  Soft warnings (empty fields) are logged but do not abort the wave.
fn lint_plan(plan: &PlanDag, tag: &str) -> bool {
    let mut ok = true;
    let node_ids: HashSet<&str> = plan.nodes.iter().map(|n| n.id.as_str()).collect();

    // Dangling edge references.
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

    // Soft: pending nodes with missing task/criterion fields.
    for node in &plan.nodes {
        if node.status == NodeStatus::Pending {
            if node.title.trim().is_empty() {
                eprintln!(
                    "[{tag}] DAG lint WARN: node '{}' has empty title — mini-agent will have no task",
                    node.id
                );
            }
            if node.description.trim().is_empty() {
                eprintln!(
                    "[{tag}] DAG lint WARN: node '{}' has empty description — mini-agent will have no success criterion",
                    node.id
                );
            }
        }
    }

    // Cycle detection (DFS with recursion stack).
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

// ── Child agent config ────────────────────────────────────────────────────────

// ── Wave identity ─────────────────────────────────────────────────────────────

fn wave_id_for(tag: &str, cycle_num: u64) -> u64 {
    let h = stable_agent_hash(tag.as_bytes());
    h.wrapping_mul(0x0000_0100_0000_01b3).wrapping_add(cycle_num).max(1)
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
    let envelope = CommandEnvelope::new(record.contract_hash(), Command::SubmitWaveDispatch(record));
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
        Ok(s) => eprintln!(
            "[{tag}] dag: ChildComplete kernel POST failed: node={node_id} status={s}"
        ),
        Err(e) => eprintln!(
            "[{tag}] dag: ChildComplete kernel POST error: node={node_id} {e}"
        ),
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
        mini_agent_count: 1,
        working_dir: parent.working_dir.clone(),
        sse_chunks_dir,
        project_dir: parent.project_dir.clone(),
        mcp_connector_url: parent.mcp_connector_url.clone(),
        router_turn_max_ms: parent.router_turn_max_ms,
        router_first_capture_ms: parent.router_first_capture_ms,
        router_idle_ms: parent.router_idle_ms,
        worker_port: parent.worker_port,
        supervisor_port: parent.supervisor_port,
        cert_max_steps: parent.cert_max_steps,
        domain: Some(node.title.clone()),
        metric: Some(node.description.clone()),
        plan_node_id: Some(node.id.clone()),
    }
}
