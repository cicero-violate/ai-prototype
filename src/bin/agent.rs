//! Agent binary.
//!
//! Loop mode (used when PROJECT_DIR contains GOAL.md):
//!   PROJECT_DIR, EXECUTE_TURNS, TURN_RETRY_LIMIT, LOOP_SLEEP_MS, AGENT_COUNT,
//!   MCP_CONNECTOR_URL, SSE_CHUNKS_DIR, ROUTER_TURN_MAX_MS,
//!   ROUTER_FIRST_CAPTURE_MS, ROUTER_IDLE_MS, CANON_OPENAI_BASE_URL
//!
//! Single-cycle mode (fallback, requires a running canon worker):
//!   AI_WORKER_PORT, CANON_OPENAI_BASE_URL, AI_AGENT_DOMAIN,
//!   AI_AGENT_METRIC, AI_AGENT_MAX_STEPS

use ai::agent::{
    AgentCycle, AgentLoopConfig, AgentObjective, LoopDriver, RouterClient, WorkerClient,
};

fn main() {
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        println!("usage: agent [--help]");
        println!("  Loop mode:        set PROJECT_DIR pointing to a directory with GOAL.md");
        println!("  Single-cycle mode: AI_WORKER_PORT, AI_AGENT_DOMAIN, AI_AGENT_METRIC");
        return;
    }

    let config = AgentLoopConfig::from_env();
    if config.project_dir.join("GOAL.md").exists() {
        LoopDriver::new(config).run_all_agents();
    } else {
        run_single_cycle();
    }
}

fn run_single_cycle() {
    let router = RouterClient::from_env().unwrap_or_else(|e| {
        eprintln!("agent: router init failed: {e}");
        std::process::exit(1);
    });

    let worker = WorkerClient::from_env().unwrap_or_else(|e| {
        eprintln!("agent: worker client init failed: {e}");
        std::process::exit(1);
    });

    let domain = std::env::var("AI_AGENT_DOMAIN").unwrap_or_else(|_| "general".into());
    let metric = std::env::var("AI_AGENT_METRIC").unwrap_or_else(|_| "complete objective".into());
    let max_steps: u64 = std::env::var("AI_AGENT_MAX_STEPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);

    let objective = AgentObjective::new(domain, metric);
    let mut cycle = AgentCycle::new(router, worker, objective).with_max_steps(max_steps);

    match cycle.run() {
        Ok(summary) => {
            println!(
                "agent: done  steps={}  success={}  stop={}  phase={}  tlog_len={}",
                summary.step_count,
                summary.success,
                summary.stop_reason,
                summary.final_phase,
                summary.final_tlog_len,
            );
        }
        Err(e) => {
            eprintln!("agent: cycle error: {e}");
            std::process::exit(1);
        }
    }
}
