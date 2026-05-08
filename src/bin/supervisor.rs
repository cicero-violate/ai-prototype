//! Stable supervisor binary.
//!
//! The supervisor owns worker process lifecycle and serves reload/health routes.
//! It never proxies API commands; callers submit commands to the active worker.

#![forbid(unsafe_code)]

use std::env;
use std::net::{SocketAddr, TcpListener as StdTcpListener};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::State as AxumState;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use tokio::net::TcpListener;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("supervisor error: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    if env::args().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }

    let cfg = SupervisorConfig::from_env()?;
    let process = WorkerProcess::new(
        cfg.worker_bin.clone(),
        cfg.tlog_dir.clone(),
        cfg.mcp_worker_url.clone(),
    );
    let state = SupervisorState {
        inner: Arc::new(Mutex::new(process)),
    };

    {
        let mut guard = state.inner.lock().await;
        guard.reload_inner().await?;
    }

    let app = build_router(state.clone());
    let listener = TcpListener::bind(cfg.addr)
        .await
        .map_err(|err| format!("bind {} failed: {err}", cfg.addr))?;
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
    println!("environment: SUPERVISOR_PORT, AI_TLOG_DIR, AI_WORKER_BIN, AI_MCP_WORKER_URL");
    println!("routes: GET /health, POST /reload");
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SupervisorConfig {
    addr: SocketAddr,
    tlog_dir: PathBuf,
    worker_bin: PathBuf,
    mcp_worker_url: String,
}

impl SupervisorConfig {
    fn from_env() -> Result<Self, String> {
        let port = env::var("SUPERVISOR_PORT")
            .unwrap_or_else(|_| "9100".to_string())
            .parse::<u16>()
            .map_err(|_| "SUPERVISOR_PORT must be a u16".to_string())?;
        let tlog_dir =
            PathBuf::from(env::var("AI_TLOG_DIR").unwrap_or_else(|_| "tlog".to_string()));
        let worker_bin = match env::var("AI_WORKER_BIN") {
            Ok(path) => PathBuf::from(path),
            Err(_) => default_worker_bin()?,
        };
        let mcp_worker_url = env::var("AI_MCP_WORKER_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:38469/mcp_worker".to_string());
        Ok(Self {
            addr: SocketAddr::from(([0, 0, 0, 0], port)),
            tlog_dir,
            worker_bin,
            mcp_worker_url,
        })
    }
}

fn default_worker_bin() -> Result<PathBuf, String> {
    let exe = env::current_exe().map_err(|err| format!("current_exe failed: {err}"))?;
    let dir = exe
        .parent()
        .ok_or_else(|| "supervisor binary has no parent directory".to_string())?;
    let mut path = dir.join("worker");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    Ok(path)
}

#[derive(Clone)]
struct SupervisorState {
    inner: Arc<Mutex<WorkerProcess>>,
}

struct WorkerProcess {
    active: Option<WorkerInstance>,
    retired: Vec<RetiredWorker>,
    next_generation: u64,
    binary_path: PathBuf,
    tlog_dir: PathBuf,
    mcp_worker_url: String,
    drain_after: Duration,
}

impl WorkerProcess {
    fn new(binary_path: PathBuf, tlog_dir: PathBuf, mcp_worker_url: String) -> Self {
        Self {
            active: None,
            retired: Vec::new(),
            next_generation: 1,
            binary_path,
            tlog_dir,
            mcp_worker_url,
            drain_after: Duration::from_secs(30),
        }
    }

    async fn health(&mut self) -> Result<HealthDto, String> {
        self.reap_retired().await;
        let Some(active) = self.active.as_mut() else {
            return Err("no active worker".to_string());
        };
        if let Some(status) = active
            .child
            .try_wait()
            .map_err(|err| format!("worker wait failed: {err}"))?
        {
            let generation = active.generation;
            self.active = None;
            return Err(format!(
                "active worker generation {generation} exited with {status}"
            ));
        }
        Ok(HealthDto {
            ok: true,
            generation: active.generation,
            worker_port: active.port,
        })
    }

    async fn reload_inner(&mut self) -> Result<ReloadDto, String> {
        self.reap_retired().await;
        let generation = self.next_generation;
        let instance = spawn_worker(
            &self.binary_path,
            generation,
            &self.tlog_dir,
            &self.mcp_worker_url,
        )
        .await?;
        self.next_generation = self
            .next_generation
            .checked_add(1)
            .ok_or_else(|| "generation overflow".to_string())?;
        if let Some(old) = self.active.take() {
            self.retired.push(RetiredWorker {
                instance: old,
                retired_at: Instant::now(),
            });
        }
        let active = ActiveWorkerDto {
            generation: instance.generation,
            worker_port: instance.port,
        };
        self.active = Some(instance);
        Ok(ReloadDto { ok: true, active })
    }

    async fn reap_retired(&mut self) {
        let now = Instant::now();
        let mut survivors = Vec::new();
        for mut retired in self.retired.drain(..) {
            if now.duration_since(retired.retired_at) >= self.drain_after {
                let _ = retired.instance.child.kill().await;
                let _ = retired.instance.child.wait().await;
            } else {
                survivors.push(retired);
            }
        }
        self.retired = survivors;
    }

    async fn shutdown(&mut self) {
        if let Some(mut active) = self.active.take() {
            let _ = active.child.kill().await;
            let _ = active.child.wait().await;
        }
        for mut retired in self.retired.drain(..) {
            let _ = retired.instance.child.kill().await;
            let _ = retired.instance.child.wait().await;
        }
    }
}

struct WorkerInstance {
    child: Child,
    port: u16,
    generation: u64,
}

struct RetiredWorker {
    instance: WorkerInstance,
    retired_at: Instant,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct HealthDto {
    ok: bool,
    generation: u64,
    worker_port: u16,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct ReloadDto {
    ok: bool,
    active: ActiveWorkerDto,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct ActiveWorkerDto {
    generation: u64,
    worker_port: u16,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct ErrorDto {
    ok: bool,
    error: String,
}

fn build_router(state: SupervisorState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/reload", post(reload))
        .with_state(state)
}

async fn health(
    AxumState(state): AxumState<SupervisorState>,
) -> Result<Json<HealthDto>, (StatusCode, Json<ErrorDto>)> {
    let mut guard = state.inner.lock().await;
    guard.health().await.map(Json).map_err(error_response)
}

async fn reload(
    AxumState(state): AxumState<SupervisorState>,
) -> Result<Json<ReloadDto>, (StatusCode, Json<ErrorDto>)> {
    let mut guard = state.inner.lock().await;
    guard.reload_inner().await.map(Json).map_err(error_response)
}

async fn spawn_worker(
    binary_path: &PathBuf,
    generation: u64,
    tlog_dir: &PathBuf,
    mcp_worker_url: &str,
) -> Result<WorkerInstance, String> {
    let port = allocate_port()?;
    let mut child = Command::new(binary_path)
        .env("AI_WORKER_MODE", "1")
        .env("PORT", port.to_string())
        .env("AI_TLOG_DIR", tlog_dir)
        .env("AI_MCP_WORKER_URL", mcp_worker_url)
        .env("AI_WORKER_GENERATION", generation.to_string())
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| format!("spawn worker failed: {err}"))?;

    if let Err(err) = wait_for_health(port).await {
        let _ = child.kill().await;
        let _ = child.wait().await;
        return Err(err);
    }

    Ok(WorkerInstance {
        child,
        port,
        generation,
    })
}

fn allocate_port() -> Result<u16, String> {
    let listener = StdTcpListener::bind("127.0.0.1:0")
        .map_err(|err| format!("allocate worker port failed: {err}"))?;
    listener
        .local_addr()
        .map(|addr| addr.port())
        .map_err(|err| format!("read allocated worker port failed: {err}"))
}

async fn wait_for_health(port: u16) -> Result<(), String> {
    let url = format!("http://127.0.0.1:{port}/health/worker");
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        match reqwest::get(&url).await {
            Ok(response) if response.status().is_success() => return Ok(()),
            _ => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    }
    Err(format!("worker health deadline expired for port {port}"))
}

fn error_response(error: String) -> (StatusCode, Json<ErrorDto>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorDto { ok: false, error }),
    )
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
