use std::path::{Path, PathBuf};
use std::thread;

use ai::domain::plan::{load_plan, ready_nodes, PlanNode};
use ai::process::agent::{AgentLoopConfig, LoopDriver, DEFAULT_EXECUTOR_COUNT};
use ai::process::endpoints::{
    mcp_connector_url_from_env, supervisor_port_from_env, DEFAULT_SUPERVISOR_PORT,
};

fn main() {
    let workspace = workspace_root();
    let max_agents = env_u32("CANON_EXAMPLE_MAX_AGENTS", DEFAULT_EXECUTOR_COUNT).clamp(1, 5);
    let execute_turns = env_u32("CANON_EXAMPLE_EXECUTE_TURNS", 1).max(1);

    let plan = load_plan(&workspace);
    let mut ready: Vec<PlanNode> = ready_nodes(&plan).into_iter().cloned().collect();
    ready.sort_by(|a, b| a.id.cmp(&b.id));
    ready.truncate(max_agents as usize);

    println!("workspace={}", workspace.display());
    println!("plan_nodes={}", plan.nodes.len());
    println!("plan_edges={}", plan.edges.len());
    println!("selected_ready_nodes={}", ready.len());
    println!("execute_turns_per_child={execute_turns}");

    if ready.is_empty() {
        println!("spawned_agents=0");
        return;
    }

    let handles: Vec<_> = ready
        .into_iter()
        .map(|node| {
            let node_id = node.id.clone();
            let config = child_config(&workspace, &node, execute_turns);
            println!(
                "spawning_bounded_agent node_id={node_id} title={:?}",
                node.title
            );
            thread::spawn(move || {
                LoopDriver::new(config).run_all_agents();
                node_id
            })
        })
        .collect();

    let mut finished = Vec::new();
    for handle in handles {
        finished.push(
            handle
                .join()
                .expect("bounded agent thread should not panic"),
        );
    }
    finished.sort();

    println!("spawned_agents={}", finished.len());
    for node_id in finished {
        println!("finished_bounded_agent node_id={node_id}");
    }
}

fn child_config(workspace: &Path, node: &PlanNode, execute_turns: u32) -> AgentLoopConfig {
    AgentLoopConfig {
        execute_turns,
        turn_retry_limit: env_u32("TURN_RETRY_LIMIT", 0),
        loop_sleep_ms: env_u64("LOOP_SLEEP_MS", 1000),
        agent_count: 1,
        executor_count: 1,
        project_dir: workspace.to_path_buf(),
        working_dir: workspace.to_path_buf(),
        sse_chunks_dir: workspace
            .join("state")
            .join("agent_state")
            .join("sse-chunks"),
        mcp_connector_url: mcp_connector_url_from_env(supervisor_port()),
        router_turn_max_ms: env_u64("ROUTER_TURN_MAX_MS", 120_000),
        router_first_capture_ms: env_u64("ROUTER_FIRST_CAPTURE_MS", 30_000),
        router_idle_ms: env_u64("ROUTER_IDLE_MS", 2_500),
        worker_port: std::env::var("AI_WORKER_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok()),
        supervisor_port: Some(supervisor_port()),
        browser_router_url: std::env::var("CANON_OPENAI_BASE_URL").ok().map(|url| {
            let url = url.trim_end_matches('/').to_string();
            url.strip_suffix("/v1").unwrap_or(&url).to_string()
        }),
        cert_max_steps: env_u64("AI_CERT_MAX_STEPS", 30),
        domain: Some(node.title.clone()),
        metric: Some(node.description.clone()),
        plan_node_id: Some(node.id.clone()),
    }
}

fn workspace_root() -> PathBuf {
    std::env::var_os("CANON_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."))
}

fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn supervisor_port() -> u16 {
    supervisor_port_from_env().unwrap_or(DEFAULT_SUPERVISOR_PORT)
}
