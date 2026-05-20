//! Supervisor process runtime.
//!
//! Owns startup, initial worker reload, Axum serving, graceful shutdown, and
//! final worker cleanup. The binary entrypoint only calls `run`.

use std::env;

use tokio::net::TcpListener;

use crate::api::routes::build_supervisor_router;
use crate::process::supervisor::process::WorkerProcess;
use crate::process::supervisor::SupervisorConfig;
use crate::process::supervisor::SupervisorState;

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
    }

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

fn print_help() {
    println!("usage: supervisor [--help]");
    println!("environment: SUPERVISOR_PORT, AI_TLOG_DIR, AI_KERNEL_TLOG_BIN, AI_MCP_WORKER_URL");
    println!("             AI_AGENT_BIN, CANON_OPENAI_BASE_URL, PROJECT_DIR, MCP_CONNECTOR_URL");
    println!("             AI_MCP_BASE_URL, AI_MCP_OAUTH_STORE_FILE, AI_MCP_OAUTH_STORE_KEY");
    println!("routes: GET /, GET /control, GET /health, GET /v1/task/next, POST /reload, POST /restart, POST /agent/start, POST /spawn, POST /v1/command, POST /ai/mcp");
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {},
            _ = terminate.recv() => {},
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
