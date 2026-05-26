//! Supervisor process runtime.
//!
//! Owns startup, initial worker reload, Axum serving, graceful shutdown, and
//! final worker cleanup. The binary entrypoint only calls `run`.

use std::env;

use tokio::net::TcpListener;

use crate::api::routes::build_supervisor_router;
use crate::runtime::introspection::canonical_tlog_path_from_dir;
use crate::service::agent::config::AgentLoopConfig;
use crate::service::dispatch::task_runner::TaskRunner;
use crate::service::invariants::event_loop as invariant_event_loop;
use crate::service::recovery::event_loop as recovery_event_loop;
use crate::service::scheduler::handler::TaskReadyNotifier;
use crate::service::supervisor::process::WorkerProcess;
use crate::service::supervisor::SupervisorConfig;
use crate::service::supervisor::SupervisorState;

pub async fn run() -> Result<(), String> {
    if env::args().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }

    let cfg = SupervisorConfig::from_env()?;
    eprintln!(
        "supervisor: port={}  project_dir={}  mcp={}  tlog={}",
        cfg.addr.port(),
        cfg.project_dir.display(),
        cfg.mcp_connector_url,
        cfg.tlog_dir.display(),
    );
    let process = WorkerProcess::new(
        cfg.worker_bin.clone(),
        cfg.tlog_dir.clone(),
        cfg.mcp_worker_url.clone(),
        cfg.router_url.clone(),
        cfg.project_dir.clone(),
        cfg.mcp_connector_url.clone(),
        cfg.addr.port(),
    );
    let state = SupervisorState::new(process, &cfg)?;

    {
        let mut guard = state.inner.lock().await;
        guard.reload_inner().await?;
        guard.reset_stale_running_nodes();
        guard.reconcile_terminal_evidence();
    }
    // Notify task runners of any tasks that were already pending at startup.
    state.task_ready_notifier.notify();

    let tlog_path = canonical_tlog_path_from_dir(&cfg.tlog_dir);
    recovery_event_loop::start(tlog_path, state.clone());
    invariant_event_loop::start(cfg.project_dir.clone());
    maybe_start_task_runner(&cfg, state.task_ready_notifier.clone());

    let app = build_supervisor_router(state.clone());
    let listener = TcpListener::bind(cfg.addr)
        .await
        .map_err(|err| format!("bind {} failed: {err}", cfg.addr))?;
    eprintln!("supervisor: ready on http://0.0.0.0:{}", cfg.addr.port());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|err| format!("serve failed: {err}"))?;

    let mut guard = state.inner.lock().await;
    guard.shutdown().await;
    Ok(())
}

/// Spawn event-driven task runner daemons as background threads.
///
/// Each runner blocks on `notifier` instead of polling on a fixed timer.
/// The supervisor signals `notifier` when tasks become Pending (retry, reset,
/// or plan update). A 30-second fallback scan handles missed signals.
/// Set `TASK_RUNNER_ENABLED=0` to disable.
fn maybe_start_task_runner(cfg: &SupervisorConfig, notifier: TaskReadyNotifier) {
    if env::var("TASK_RUNNER_ENABLED").as_deref() == Ok("0") {
        return;
    }

    let runner_count = env::var("TASK_RUNNER_COUNT")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|count| *count > 0)
        .unwrap_or(1);
    let hostname = env::var("HOSTNAME").unwrap_or_else(|_| "local".to_string());

    for runner_index in 0..runner_count {
        let supervisor_url = cfg.mcp_connector_url.clone();
        let worker_id = if runner_count == 1 {
            format!("task-runner-{hostname}")
        } else {
            format!("task-runner-{hostname}-{}", runner_index + 1)
        };
        let base_config = AgentLoopConfig::from_env();
        let runner_notifier = notifier.clone();
        eprintln!(
            "supervisor: task_runner starting  worker_id={worker_id}  supervisor={supervisor_url}  runner={}/{}",
            runner_index + 1,
            runner_count
        );
        std::thread::spawn(move || {
            TaskRunner::new(supervisor_url, worker_id, base_config, runner_notifier).run();
        });
    }
}

fn print_help() {
    println!("usage: supervisor [--help]");
    println!("environment: SUPERVISOR_PORT, AI_TLOG_DIR, AI_KERNEL_TLOG_BIN, AI_MCP_WORKER_URL");
    println!("             AI_AGENT_BIN, CANON_OPENAI_BASE_URL, PROJECT_DIR, MCP_CONNECTOR_URL");
    println!("             AI_MCP_BASE_URL, AI_MCP_OAUTH_STORE_FILE, AI_MCP_OAUTH_STORE_KEY");
    println!("             TASK_RUNNER_ENABLED (set to 0 to disable; default on)");
    println!("             TASK_RUNNER_COUNT (parallel task runners; default 1)");
    println!("             INVARIANT_MINER_ENABLED (set to 0 to disable; default on)");
    println!("             INVARIANT_AUTO_PROMOTE (set to 1 to promote validated invariants)");
    println!("             INVARIANT_MINER_INTERVAL_SECS, INVARIANT_MINER_STARTUP_DELAY_SECS, INVARIANT_MIN_SUPPORT");
    println!("routes: GET /, GET /control, GET /health, GET /v1/task/next, POST /reload, POST /restart, POST /agent/start, POST /spawn, POST /v1/command, POST /v1/task/claim, POST /v1/task/heartbeat, POST /v1/task/complete, POST /v1/task/fail, POST /ai/mcp");
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut terminate) => {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {},
                    _ = terminate.recv() => {},
                }
            }
            Err(err) => {
                eprintln!("install SIGTERM handler failed: {err}");
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
