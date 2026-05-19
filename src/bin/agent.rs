//! Agent binary.
//!
//! Loop mode (used when PROJECT_DIR contains GOAL.md):
//!   PROJECT_DIR, EXECUTE_TURNS, TURN_RETRY_LIMIT, LOOP_SLEEP_MS, AGENT_COUNT,
//!   MCP_CONNECTOR_URL, SSE_CHUNKS_DIR, ROUTER_TURN_MAX_MS,
//!   ROUTER_FIRST_CAPTURE_MS, ROUTER_IDLE_MS, CANON_OPENAI_BASE_URL.
//!   Planning also inspects PROJECT_DIR/state/rustc/auto-refactor when present.
//!
//! Single-cycle mode (fallback, requires a running canon worker):
//!   SUPERVISOR_PORT, AI_WORKER_PORT, CANON_OPENAI_BASE_URL, AI_AGENT_DOMAIN,
//!   AI_AGENT_METRIC, AI_AGENT_MAX_STEPS

use ai::process::agent::{
    AgentCycle, AgentLoopConfig, AgentObjective, LoopDriver, RouterClient, WorkerClient,
};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn main() {
    if std::env::args().any(|a| a == "--help" || a == "-h") {
        println!("usage: agent [--help]");
        println!("  Loop mode:        set PROJECT_DIR pointing to a directory with GOAL.md");
        println!(
            "  Single-cycle mode: SUPERVISOR_PORT, AI_WORKER_PORT, AI_AGENT_DOMAIN, AI_AGENT_METRIC"
        );
        println!("  Planning reads:   plan.md, status.md, score.md, SCORE_REPORT.md, state/rustc/auto-refactor");
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

    let worker = resolve_single_cycle_worker().unwrap_or_else(|e| {
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

fn resolve_single_cycle_worker() -> Result<WorkerClient, String> {
    let fallback_worker_port = env_port("AI_WORKER_PORT");

    if let Some(supervisor_port) = env_port("SUPERVISOR_PORT") {
        match supervisor_reload_worker_port(supervisor_port, fallback_worker_port) {
            Ok(worker_port) => {
                eprintln!(
                    "agent: resolved active worker via supervisor reload  supervisor_port={supervisor_port} worker_port={worker_port}"
                );
                return Ok(WorkerClient::new(worker_port));
            }
            Err(err) => {
                eprintln!(
                    "agent: supervisor reload failed on port {supervisor_port}: {err}; falling back to AI_WORKER_PORT"
                );
            }
        }
    }

    let worker_port = fallback_worker_port.ok_or_else(|| {
        "AI_WORKER_PORT not set and SUPERVISOR_PORT did not resolve an active worker".to_string()
    })?;
    Ok(WorkerClient::new(worker_port))
}

fn env_port(name: &str) -> Option<u16> {
    std::env::var(name).ok()?.parse::<u16>().ok()
}

fn supervisor_reload_worker_port(
    supervisor_port: u16,
    fallback_worker_port: Option<u16>,
) -> Result<u16, String> {
    let response = supervisor_reload_http_exchange(supervisor_port)?;

    let status: u16 = response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse().ok())
        .unwrap_or(0);

    match status {
        200 => parse_reload_worker_port(&response),
        204 => fallback_worker_port.ok_or_else(|| {
            "supervisor /reload returned 204 and AI_WORKER_PORT is unset".to_string()
        }),
        _ => Err(format!("supervisor /reload returned HTTP {status}")),
    }
}

fn supervisor_reload_http_exchange(supervisor_port: u16) -> Result<String, String> {
    let mut stream = TcpStream::connect(("127.0.0.1", supervisor_port))
        .map_err(|e| format!("supervisor connect: {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();

    let request = format!(
        "POST /reload HTTP/1.1\r\nHost: 127.0.0.1:{supervisor_port}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("supervisor write: {e}"))?;
    stream.flush().ok();

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| format!("supervisor read: {e}"))?;
    Ok(response)
}

fn parse_reload_worker_port(response: &str) -> Result<u16, String> {
    let body = reload_response_body(response)?;
    let json: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("supervisor /reload JSON: {e}"))?;
    let port = json
        .get("active")
        .and_then(|active| active.get("worker_port"))
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| "supervisor /reload response missing active.worker_port".to_string())?;
    u16::try_from(port).map_err(|_| format!("supervisor /reload worker_port out of range: {port}"))
}

fn reload_response_body(response: &str) -> Result<&str, String> {
    response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .or_else(|| response.split_once("\n\n").map(|(_, body)| body))
        .ok_or_else(|| "supervisor /reload response missing body".to_string())
}
