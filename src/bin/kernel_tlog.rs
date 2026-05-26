//! Reloadable worker binary.
//!
//! The worker owns the kernel session and serves the deterministic API router
//! on a local TCP port. State advances only through HTTP command ingress, which
//! delegates to `ApiTransportSession`.

#![forbid(unsafe_code)]

use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use ai::{
    build_router, canonical_tlog_path_from_dir, resume_durable_runtime, tick_durable,
    ApiTransportLedger, ApiTransportSession, RuntimeConfig, State, WorkerAppState,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("worker error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    if env::args().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }

    let cfg = WorkerConfig::from_env()?;

    // Bind the port before TLog replay so the process is reachable immediately.
    // Health and command routes return 503 until set_ready() is called below.
    let listener = TcpListener::bind(cfg.addr)
        .await
        .map_err(|err| format!("bind {} failed: {err}", cfg.addr))?;

    let state = WorkerAppState::loading();
    let app = build_router(state.clone());

    let tlog_path = cfg.tlog_path.clone();
    let state_clone = state.clone();
    tokio::task::spawn_blocking(move || match load_session(&tlog_path) {
        Ok(session) => state_clone.set_ready(session, &tlog_path),
        Err(err) => {
            eprintln!("worker error: {err}");
            std::process::exit(1);
        }
    });

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|err| format!("serve failed: {err}"))?;
    Ok(())
}

fn print_help() {
    println!("usage: kernel_tlog [--help]");
    println!("environment: PORT, AI_TLOG_DIR, AI_MCP_WORKER_URL, AI_WORKER_GENERATION");
    println!("canonical TLog: AI_TLOG_DIR/canon-agent.tlog.ndjson");
    println!("routes: GET /health/worker, GET /v1/state, POST /v1/command");
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkerConfig {
    addr: SocketAddr,
    tlog_path: PathBuf,
}

impl WorkerConfig {
    fn from_env() -> Result<Self, String> {
        let port = env::var("PORT")
            .map_err(|_| "PORT is required".to_string())?
            .parse::<u16>()
            .map_err(|_| "PORT must be a u16".to_string())?;
        let tlog_dir = env::var("AI_TLOG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let project = env::var("PROJECT_DIR").unwrap_or_default();
                PathBuf::from(project).join("state/tlog")
            });
        let tlog_dir = if tlog_dir.is_relative() {
            let project = env::var("PROJECT_DIR").unwrap_or_default();
            PathBuf::from(project).join(&tlog_dir)
        } else {
            tlog_dir
        };
        let tlog_path = tlog_path_from_dir(&tlog_dir);
        Ok(Self {
            addr: SocketAddr::from(([127, 0, 0, 1], port)),
            tlog_path,
        })
    }
}

fn tlog_path_from_dir(dir: &Path) -> PathBuf {
    canonical_tlog_path_from_dir(dir)
}

fn load_session(tlog_path: &Path) -> Result<ApiTransportSession, String> {
    ensure_tlog_dir(tlog_path)?;

    let cfg = RuntimeConfig::default();
    let mut runtime = resume_durable_runtime(State::default(), tlog_path)
        .map_err(|err| format!("resume durable runtime failed: {err}"))?;
    initialize_empty_runtime(&mut runtime, tlog_path, cfg)?;
    session_from_runtime(runtime, cfg)
}

fn session_from_runtime(
    runtime: ai::DurableRuntimeState,
    cfg: RuntimeConfig,
) -> Result<ApiTransportSession, String> {
    ApiTransportSession::from_parts(
        runtime.state,
        runtime.tlog,
        cfg,
        runtime.command_ledger,
        ApiTransportLedger::default(),
    )
    .map_err(|err| format!("transport session verification failed: {err}"))
}

fn ensure_tlog_dir(tlog_path: &Path) -> Result<(), String> {
    let Some(parent) = tlog_path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    std::fs::create_dir_all(parent).map_err(|err| format!("create tlog dir failed: {err}"))
}

fn initialize_empty_runtime(
    runtime: &mut ai::DurableRuntimeState,
    tlog_path: &Path,
    cfg: RuntimeConfig,
) -> Result<(), String> {
    if !runtime.tlog.is_empty() {
        return Ok(());
    }
    tick_durable(&mut runtime.state, &mut runtime.tlog, tlog_path, cfg)
        .map_err(|err| format!("initialize durable runtime failed: {err}"))?;
    runtime.command_ledger = ai::CommandLedger::reconstruct_from_tlog(&runtime.tlog)
        .map_err(|err| format!("reconstruct command ledger failed: {err}"))?;
    Ok(())
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
