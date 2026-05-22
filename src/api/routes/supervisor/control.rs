//! Core supervisor health, reload, spawn, and command gateway routes.

use axum::body::Bytes;
use axum::extract::State as AxumState;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::Json;
use std::process::{Command as StdCommand, Stdio};

use crate::domain::plan::{ready_nodes_from_plan_state, NodeStatus};
use crate::process::scheduler::plan_store::{load_plan, load_plan_read_model};
use crate::process::supervisor::{
    HealthDto, PlanStatusDto, ReloadDto, RestartDto, SpawnDto, SpawnRequest, StartLoopDto,
    StartLoopRequest, TaskClaimDto, TaskClaimRequest, TaskCompleteDto, TaskCompleteRequest,
    TaskFailDto, TaskFailRequest, TaskHeartbeatDto, TaskHeartbeatRequest, TaskNextDto,
};

use crate::process::supervisor::{ErrorDto, SupervisorState};

pub(crate) const SUPERVISOR_RESTART_DELAY_MS: u64 = 700;
pub(crate) const SUPERVISOR_EXIT_DELAY_MS: u64 = 150;

pub async fn control_page(
    AxumState(state): AxumState<SupervisorState>,
) -> Result<Html<String>, (StatusCode, Json<ErrorDto>)> {
    let health = {
        let mut guard = state.inner.lock().await;
        guard.health().await.map_err(error_response)?
    };
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Canon AI Supervisor Control</title>
<style>
body{{font-family:system-ui,sans-serif;max-width:720px;margin:64px auto;padding:0 24px;color:#111}}
.card{{border:1px solid #e5e7eb;border-radius:10px;padding:24px;margin-top:20px}}
button{{padding:10px 18px;font-size:15px;border:0;border-radius:6px;cursor:pointer;margin-right:10px}}
.reload{{background:#1d4ed8;color:#fff}}
.restart{{background:#991b1b;color:#fff}}
code{{background:#f3f4f6;padding:2px 5px;border-radius:4px}}
</style>
</head>
<body>
<h1>Canon AI Supervisor</h1>
<div class="card">
<p>Status: <strong>ok</strong></p>
<p>Worker generation: <code>{generation}</code></p>
<p>Worker port: <code>{worker_port}</code></p>
<form method="post" action="/reload" style="display:inline">
  <button class="reload" type="submit">Reload worker</button>
</form>
<form method="post" action="/restart" style="display:inline" onsubmit="return confirm('Restart the supervisor process? This briefly disconnects the control API.');">
  <button class="restart" type="submit">Restart supervisor</button>
</form>
</div>
<p><small><code>/reload</code> replaces the worker. <code>/restart</code> starts a replacement supervisor process and exits this one.</small></p>
</body>
</html>"#,
        generation = health.generation,
        worker_port = health.worker_port,
    );
    Ok(Html(html))
}

pub async fn health(
    AxumState(state): AxumState<SupervisorState>,
) -> Result<Json<HealthDto>, (StatusCode, Json<ErrorDto>)> {
    let mut guard = state.inner.lock().await;
    guard.health().await.map(Json).map_err(error_response)
}

pub async fn reload(
    AxumState(state): AxumState<SupervisorState>,
) -> Result<Json<ReloadDto>, (StatusCode, Json<ErrorDto>)> {
    let mut guard = state.inner.lock().await;
    guard.reload_inner().await.map(Json).map_err(error_response)
}

pub async fn restart(
    AxumState(state): AxumState<SupervisorState>,
) -> Result<Json<RestartDto>, (StatusCode, Json<ErrorDto>)> {
    state
        .restart_supervisor()
        .await
        .map(Json)
        .map_err(error_response)
}

pub async fn start_agent_loop_handler(
    AxumState(state): AxumState<SupervisorState>,
    body: Option<Json<StartLoopRequest>>,
) -> Result<Json<StartLoopDto>, (StatusCode, Json<ErrorDto>)> {
    let req = body.map(|b| b.0).unwrap_or_default();
    let mut guard = state.inner.lock().await;
    guard.start_main_loop(req).map(Json).map_err(error_response)
}

pub async fn spawn_agent_handler(
    AxumState(state): AxumState<SupervisorState>,
    Json(body): Json<SpawnRequest>,
) -> Result<Json<SpawnDto>, (StatusCode, Json<ErrorDto>)> {
    let mut guard = state.inner.lock().await;
    let max_steps = body.max_steps.unwrap_or(20).clamp(1, 100);
    guard
        .spawn_agent(&body.domain, &body.metric, max_steps)
        .map(Json)
        .map_err(error_response)
}

pub async fn command_gateway(
    AxumState(state): AxumState<SupervisorState>,
    body: Bytes,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorDto>)> {
    let body_len = body.len();
    let (command_id, payload_tag, source) = command_log_fields(&body);
    let worker_port = {
        let mut guard = state.inner.lock().await;
        guard.ensure_worker_alive_or_reload().await.map_err(error_response)?
    };

    eprintln!(
        "supervisor: received kernel command via /v1/command  source={} command_id={} payload_tag={} bytes={} -> worker_port={}",
        source, command_id, payload_tag, body_len, worker_port
    );

    let url = format!("http://127.0.0.1:{worker_port}/v1/command");
    let response = reqwest::Client::new()
        .post(url)
        .header("content-type", "application/json")
        .body(body.to_vec())
        .send()
        .await
        .map_err(|err| error_response(format!("worker command proxy failed: {err}")))?;

    let status = StatusCode::from_u16(response.status().as_u16())
        .map_err(|err| error_response(format!("invalid worker status: {err}")))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|err| error_response(format!("worker command body read failed: {err}")))?;

    eprintln!(
        "supervisor: completed kernel command via /v1/command  source={} command_id={} payload_tag={} status={} response_bytes={}",
        source,
        command_id,
        payload_tag,
        status.as_u16(),
        bytes.len()
    );

    Ok((status, bytes))
}

fn command_log_fields(body: &[u8]) -> (String, String, String) {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(body) else {
        return (
            "<invalid-json>".to_string(),
            "<invalid-json>".to_string(),
            "unknown".to_string(),
        );
    };
    let command_id = value
        .get("command_id")
        .and_then(|v| v.as_u64())
        .map(|v| v.to_string())
        .unwrap_or_else(|| "<missing>".to_string());
    let payload_tag = value
        .get("payload_tag")
        .and_then(|v| v.as_str())
        .unwrap_or("<missing>")
        .to_string();
    let source = value
        .get("source")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .or_else(|| {
            value
                .get("agent_turn")
                .is_some()
                .then(|| "agent".to_string())
        })
        .or_else(|| {
            value
                .get("browser_turn")
                .is_some()
                .then(|| "browser-router".to_string())
        })
        .unwrap_or_else(|| {
            if payload_tag.contains("Mcp") {
                "mcp".to_string()
            } else {
                "kernel".to_string()
            }
        });
    (command_id, payload_tag, source)
}

/// Return the first ready, unclaimed plan node for a worker to pull.
///
/// Filters out nodes with active (non-expired) leases so two concurrent
/// workers cannot both see the same node.  Workers then race to POST
/// /v1/task/claim; the second caller gets a 409.  Returns 204 when no
/// unclaimed ready tasks exist.
pub async fn get_task_next(
    AxumState(state): AxumState<SupervisorState>,
) -> Result<Json<TaskNextDto>, (StatusCode, Json<ErrorDto>)> {
    let (project_dir, claimed) = {
        let guard = state.inner.lock().await;
        let claimed: std::collections::HashSet<String> =
            guard.active_claimed_node_ids().into_iter().collect();
        (guard.project_dir().to_path_buf(), claimed)
    };

    let plan = load_plan_read_model(&project_dir)
        .map(|(p, _)| p)
        .unwrap_or_else(|_| load_plan(&project_dir));
    let ready: Vec<_> = ready_nodes_from_plan_state(&plan)
        .into_iter()
        .filter(|n| !claimed.contains(n.id.as_str()))
        .collect();

    let Some(node) = ready.first() else {
        return Err((
            StatusCode::NO_CONTENT,
            Json(ErrorDto {
                ok: false,
                error: "no ready tasks".to_string(),
            }),
        ));
    };

    Ok(Json(TaskNextDto {
        ok: true,
        node_id: node.id.clone(),
        title: node.title.clone(),
        description: node.description.clone(),
        ready_count: ready.len(),
    }))
}

pub async fn get_plan_status(
    AxumState(state): AxumState<SupervisorState>,
) -> Result<Json<PlanStatusDto>, (StatusCode, Json<ErrorDto>)> {
    let project_dir = {
        let guard = state.inner.lock().await;
        guard.project_dir().to_path_buf()
    };

    let plan = load_plan_read_model(&project_dir)
        .map(|(p, _)| p)
        .map_err(|err| error_response(format!("load plan read model failed: {err}")))?;
    let ready = ready_nodes_from_plan_state(&plan).len();

    let mut dto = PlanStatusDto {
        ok: true,
        pending: 0,
        running: 0,
        done: 0,
        failed: 0,
        skipped: 0,
        total: plan.nodes.len(),
        ready,
    };

    for node in &plan.nodes {
        match node.status {
            NodeStatus::Pending => dto.pending += 1,
            NodeStatus::Running => dto.running += 1,
            NodeStatus::Done => dto.done += 1,
            NodeStatus::Failed => dto.failed += 1,
            NodeStatus::Skipped => dto.skipped += 1,
        }
    }

    Ok(Json(dto))
}

/// Atomically claim a ready plan node and obtain a timed lease.
///
/// Returns 409 if the node is already claimed by a different worker.
/// Idempotent: same (node_id, worker_id, idempotency_key) returns the
/// existing lease without error.
pub async fn post_task_claim(
    AxumState(state): AxumState<SupervisorState>,
    Json(body): Json<TaskClaimRequest>,
) -> Result<Json<TaskClaimDto>, (StatusCode, Json<ErrorDto>)> {
    let mut guard = state.inner.lock().await;
    guard.claim_task(body).map(Json).map_err(|e| {
        (
            StatusCode::CONFLICT,
            Json(ErrorDto {
                ok: false,
                error: e,
            }),
        )
    })
}

/// Renew the lease for an active claim.  Workers should heartbeat before
/// their lease_ttl_ms elapses (recommended: every lease_ttl_ms / 3).
pub async fn post_task_heartbeat(
    AxumState(state): AxumState<SupervisorState>,
    Json(body): Json<TaskHeartbeatRequest>,
) -> Result<Json<TaskHeartbeatDto>, (StatusCode, Json<ErrorDto>)> {
    let mut guard = state.inner.lock().await;
    guard.heartbeat_task(body).map(Json).map_err(error_response)
}

/// Mark a claimed task as successfully completed. Appends completion evidence
/// and a TLog-backed done patch, then releases the lease.
pub async fn post_task_complete(
    AxumState(state): AxumState<SupervisorState>,
    Json(body): Json<TaskCompleteRequest>,
) -> Result<Json<TaskCompleteDto>, (StatusCode, Json<ErrorDto>)> {
    let mut guard = state.inner.lock().await;
    guard.complete_task(body).map(Json).map_err(error_response)
}

/// Mark a claimed task as failed.  Pass retry_after_ms == 0 to set status
/// to "failed" permanently; pass >0 to reset to "pending" for retry.
pub async fn post_task_fail(
    AxumState(state): AxumState<SupervisorState>,
    Json(body): Json<TaskFailRequest>,
) -> Result<Json<TaskFailDto>, (StatusCode, Json<ErrorDto>)> {
    let retry_after_ms = body.retry_after_ms;
    let result = {
        let mut guard = state.inner.lock().await;
        guard.fail_task(body).map(Json).map_err(error_response)?
    };
    // Notify task runners: a retried task is now Pending and ready to claim.
    if retry_after_ms > 0 {
        state.task_ready_notifier.notify();
    }
    Ok(result)
}

pub(crate) fn schedule_supervisor_replacement(delay_ms: u64) -> Result<String, String> {
    let command = supervisor_replacement_command()?;
    #[cfg(unix)]
    {
        let delay_seconds = format!("{}", delay_ms as f64 / 1000.0);
        let script = "sleep \"$1\"; shift; exec \"$@\"";
        StdCommand::new("sh")
            .arg("-c")
            .arg(script)
            .arg("supervisor-restart")
            .arg(delay_seconds)
            .arg(&command.program)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| format!("spawn delayed supervisor restart failed: {error}"))?;
    }
    #[cfg(not(unix))]
    {
        StdCommand::new(&command.program)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| format!("spawn supervisor restart failed: {error}"))?;
    }
    Ok(command.display())
}

struct SupervisorRestartCommand {
    program: std::path::PathBuf,
}

impl SupervisorRestartCommand {
    fn display(&self) -> String {
        self.program.display().to_string()
    }
}

fn supervisor_replacement_command() -> Result<SupervisorRestartCommand, String> {
    if let Ok(path) = std::env::var("AI_SUPERVISOR_RESTART_BIN") {
        let path = std::path::PathBuf::from(path);
        if path.exists() {
            return Ok(SupervisorRestartCommand { program: path });
        }
        return Err(format!(
            "AI_SUPERVISOR_RESTART_BIN points to missing path: {}",
            path.display()
        ));
    }

    let exe = std::env::current_exe().map_err(|error| format!("current_exe failed: {error}"))?;
    if exe.exists() {
        return Ok(SupervisorRestartCommand { program: exe });
    }

    let project_dir = std::env::var("PROJECT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());
    let release_supervisor = project_dir
        .join("target")
        .join("release")
        .join(if cfg!(windows) {
            "supervisor.exe"
        } else {
            "supervisor"
        });
    if release_supervisor.exists() {
        return Ok(SupervisorRestartCommand {
            program: release_supervisor,
        });
    }

    Err(format!(
        "no restart command found; tried current_exe {} and {}",
        exe.display(),
        release_supervisor.display(),
    ))
}

fn error_response(error: String) -> (StatusCode, Json<ErrorDto>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorDto { ok: false, error }),
    )
}
