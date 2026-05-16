//! Supervisor worker-process lifecycle and agent spawning.
//!
//! This belongs to the outer API layer because it owns OS process lifecycle,
//! HTTP health probing, and bridge calls into the agent loop driver.

use std::env;
use std::net::TcpListener as StdTcpListener;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command};

use crate::process::agent::{AgentLoopConfig, LoopDriver};

pub struct WorkerProcess {
    active: Option<WorkerInstance>,
    retired: Vec<RetiredWorker>,
    next_generation: u64,
    binary_path: PathBuf,
    tlog_dir: PathBuf,
    mcp_worker_url: String,
    drain_after: Duration,
    project_dir: PathBuf,
    mcp_connector_url: String,
    supervisor_port: u16,
    next_spawn: u64,
}

impl WorkerProcess {
    pub fn new(
        binary_path: PathBuf,
        tlog_dir: PathBuf,
        mcp_worker_url: String,
        _router_url: String,
        project_dir: PathBuf,
        mcp_connector_url: String,
        supervisor_port: u16,
    ) -> Self {
        Self {
            active: None,
            retired: Vec::new(),
            next_generation: 1,
            binary_path,
            tlog_dir,
            mcp_worker_url,
            drain_after: Duration::from_secs(30),
            project_dir,
            mcp_connector_url,
            supervisor_port,
            next_spawn: 0,
        }
    }

    pub async fn health(&mut self) -> Result<HealthDto, String> {
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

    pub async fn reload_inner(&mut self) -> Result<ReloadDto, String> {
        self.reap_retired().await;
        let generation = self.next_generation;
        eprintln!("supervisor: starting worker generation {generation}");
        let instance = spawn_worker(
            &self.binary_path,
            generation,
            &self.tlog_dir,
            &self.mcp_worker_url,
        )
        .await?;
        eprintln!(
            "supervisor: worker generation {generation} ready on port {}",
            instance.port
        );
        self.next_generation = self
            .next_generation
            .checked_add(1)
            .ok_or_else(|| "generation overflow".to_string())?;
        if let Some(old) = self.active.take() {
            eprintln!("supervisor: retiring worker generation {}", old.generation);
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

    pub async fn reap_retired(&mut self) {
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

    pub async fn shutdown(&mut self) {
        if let Some(mut active) = self.active.take() {
            let _ = active.child.kill().await;
            let _ = active.child.wait().await;
        }
        for mut retired in self.retired.drain(..) {
            let _ = retired.instance.child.kill().await;
            let _ = retired.instance.child.wait().await;
        }
    }

    pub fn spawn_agent(
        &mut self,
        domain: &str,
        metric: &str,
        max_steps: u64,
    ) -> Result<SpawnDto, String> {
        let worker_port = self
            .active
            .as_ref()
            .map(|w| w.port)
            .ok_or_else(|| "no active worker — call /reload first".to_string())?;

        let n = self.next_spawn;
        self.next_spawn += 1;
        let spawn_id = format!("agent-{n}");

        let project_dir = self.project_dir.clone();
        let mcp_connector_url = self.mcp_connector_url.clone();
        let execute_turns = max_steps.min(100).max(1) as u32;
        let config = AgentLoopConfig {
            execute_turns,
            turn_retry_limit: 2,
            loop_sleep_ms: 5000,
            agent_count: 1,
            working_dir: project_dir.clone(),
            sse_chunks_dir: project_dir.join("agent_state").join("sse-chunks"),
            project_dir,
            mcp_connector_url,
            router_turn_max_ms: 600_000,
            router_first_capture_ms: 60_000,
            router_idle_ms: 2_500,
            worker_port: Some(worker_port),
            supervisor_port: Some(self.supervisor_port),
            cert_max_steps: 30,
            domain: Some(domain.to_string()),
            metric: Some(metric.to_string()),
        };

        eprintln!(
            "supervisor: spawning agent {spawn_id}  domain={domain:?}  metric={metric:?}  worker_port={worker_port}"
        );
        let sid = spawn_id.clone();
        std::thread::spawn(move || {
            LoopDriver::new(config).run_all_agents();
            eprintln!("supervisor: agent {sid} finished");
        });

        Ok(SpawnDto {
            ok: true,
            spawn_id,
            pid: std::process::id(),
            domain: domain.to_string(),
            metric: metric.to_string(),
            worker_port,
        })
    }

    pub fn active_worker_port(&self) -> Result<u16, String> {
        self.active
            .as_ref()
            .map(|worker| worker.port)
            .ok_or_else(|| "no active worker — call /reload first".to_string())
    }

    pub fn active_generation(&self) -> Option<u64> {
        self.active.as_ref().map(|worker| worker.generation)
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
pub struct HealthDto {
    pub ok: bool,
    pub generation: u64,
    pub worker_port: u16,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ReloadDto {
    pub ok: bool,
    pub active: ActiveWorkerDto,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ActiveWorkerDto {
    pub generation: u64,
    pub worker_port: u16,
}

#[derive(Debug, Deserialize)]
pub struct SpawnRequest {
    pub domain: String,
    pub metric: String,
    pub max_steps: Option<u64>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct SpawnDto {
    pub ok: bool,
    pub spawn_id: String,
    pub pid: u32,
    pub domain: String,
    pub metric: String,
    pub worker_port: u16,
}

async fn spawn_worker(
    binary_path: &PathBuf,
    generation: u64,
    tlog_dir: &PathBuf,
    mcp_worker_url: &str,
) -> Result<WorkerInstance, String> {
    ensure_worker_binary(binary_path).await?;
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

async fn ensure_worker_binary(binary_path: &PathBuf) -> Result<(), String> {
    if binary_path.exists() {
        return Ok(());
    }

    eprintln!(
        "supervisor: worker binary missing at {}; building it",
        binary_path.display()
    );
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_manifest = manifest_dir
        .parent()
        .ok_or_else(|| "CARGO_MANIFEST_DIR has no workspace parent".to_string())?
        .join("Cargo.toml");

    let mut command = Command::new("cargo");
    command
        .arg("build")
        .arg("--manifest-path")
        .arg(&workspace_manifest)
        .arg("--bin")
        .arg("kernel_tlog")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    if binary_path
        .components()
        .any(|component| component.as_os_str() == "release")
    {
        command.arg("--release");
    }
    let status = command
        .status()
        .await
        .map_err(|err| format!("build worker binary failed to spawn: {err}"))?;
    if !status.success() {
        return Err(format!("build worker binary exited with {status}"));
    }
    binary_path
        .exists()
        .then_some(())
        .ok_or_else(|| format!("built worker binary not found at {}", binary_path.display()))
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
