//! Reloadable worker binary.
//!
//! The worker owns the kernel session and serves the deterministic API router
//! on a local TCP port. State advances only through HTTP command ingress, which
//! delegates to `ApiTransportSession`.

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use ai::{
    build_router, canonical_tlog_path_from_dir, decode_control_event_ndjson,
    resume_durable_runtime, tick_durable, verify_tlog_from, write_tlog_ndjson, ApiTransportLedger,
    ApiTransportSession, CanonError, RuntimeConfig, State, TLog, WorkerAppState,
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
    let mut runtime = resume_runtime_with_repair(tlog_path)?;
    initialize_empty_runtime(&mut runtime, tlog_path, cfg)?;
    session_from_runtime(runtime, cfg)
}

fn resume_runtime_with_repair(tlog_path: &Path) -> Result<ai::DurableRuntimeState, String> {
    match resume_durable_runtime(State::default(), tlog_path) {
        Ok(runtime) => Ok(runtime),
        Err(CanonError::InvalidTlogRecord) => {
            let repaired = repair_canonical_tlog(tlog_path)
                .map_err(|err| format!("repair invalid durable runtime TLog failed: {err}"))?;
            eprintln!(
                "worker: repaired invalid TLog  path={}  preserved={}  kept_events={}",
                tlog_path.display(),
                repaired.preserved_path.display(),
                repaired.kept_events
            );
            resume_durable_runtime(State::default(), tlog_path)
                .map_err(|err| format!("resume durable runtime after repair failed: {err}"))
        }
        Err(err) => Err(format!("resume durable runtime failed: {err}")),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TlogRepairReport {
    kept_events: usize,
    preserved_path: PathBuf,
}

fn repair_canonical_tlog(tlog_path: &Path) -> Result<TlogRepairReport, String> {
    let valid_prefix = load_valid_tlog_prefix(tlog_path)?;
    verify_tlog_from(State::default(), &valid_prefix)
        .map_err(|err| format!("valid prefix replay failed: {err}"))?;

    let preserved_path = preserved_tlog_path(tlog_path);
    fs::rename(tlog_path, &preserved_path)
        .map_err(|err| format!("preserve corrupt TLog failed: {err}"))?;
    write_tlog_ndjson(tlog_path, &valid_prefix)
        .map_err(|err| format!("rewrite repaired TLog failed: {err}"))?;

    Ok(TlogRepairReport {
        kept_events: valid_prefix.len(),
        preserved_path,
    })
}

fn load_valid_tlog_prefix(tlog_path: &Path) -> Result<TLog, String> {
    if !tlog_path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(tlog_path).map_err(|err| format!("open TLog failed: {err}"))?;
    let reader = BufReader::new(file);
    let mut tlog = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|err| format!("read TLog line failed: {err}"))?;
        if line.trim().is_empty() {
            continue;
        }
        match decode_control_event_ndjson(&line) {
            Ok(event) => tlog.push(event),
            Err(_) => break,
        }
    }

    Ok(tlog)
}

fn preserved_tlog_path(tlog_path: &Path) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    tlog_path.with_extension(format!("ndjson.invalid-{timestamp}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repair_canonical_tlog_preserves_corrupt_file_and_keeps_valid_prefix() {
        let dir = tempfile::Builder::new()
            .prefix("kernel-tlog-repair-")
            .tempdir()
            .expect("tempdir should be created");
        let tlog_path = dir.path().join("canon-agent.tlog.ndjson");

        let cfg = RuntimeConfig::default();
        let mut state = State::default();
        let mut tlog = Vec::new();
        ai::tick(&mut state, &mut tlog, cfg).expect("test event should be produced");
        let valid_line = ai::encode_control_event_ndjson(&tlog[0]);
        fs::write(&tlog_path, format!("{valid_line}\n[6,1,999\n"))
            .expect("corrupt fixture should be written");

        let report = repair_canonical_tlog(&tlog_path).expect("repair should succeed");

        assert_eq!(report.kept_events, 1);
        assert!(report.preserved_path.exists());
        let repaired = ai::load_tlog_ndjson(&tlog_path).expect("repaired tlog should load");
        assert_eq!(repaired, tlog);
    }
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
