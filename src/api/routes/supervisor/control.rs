//! Core supervisor health, reload, spawn, and command gateway routes.

use axum::body::Bytes;
use axum::extract::State as AxumState;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::Json;
use std::process::{Command as StdCommand, Stdio};

use crate::domain::plan::{ready_nodes_from_plan_state, NodeStatus};
use crate::service::scheduler::plan_store::{load_plan, load_plan_read_model};
use crate::service::supervisor::{
    AgentStatusDto, HealthDto, PlanStatusDto, ReloadDto, RestartDto, SpawnDto, SpawnRequest,
    StartLoopDto, StartLoopRequest, TaskClaimDto, TaskClaimRequest, TaskCompleteDto,
    TaskCompleteRequest, TaskFailDto, TaskFailRequest, TaskHeartbeatDto, TaskHeartbeatRequest,
    TaskNextDto,
};

use crate::service::supervisor::{ErrorDto, SupervisorState};

pub(crate) const SUPERVISOR_RESTART_DELAY_MS: u64 = 700;
pub(crate) const SUPERVISOR_EXIT_DELAY_MS: u64 = 150;

pub async fn control_page(
    AxumState(state): AxumState<SupervisorState>,
) -> Result<Html<String>, (StatusCode, Json<ErrorDto>)> {
    let health = {
        let mut guard = state.inner.lock().await;
        guard.health().await.map_err(error_response)?
    };
    let html = render_control_page_html(&health);
    Ok(Html(html))
}

fn render_control_page_html(health: &HealthDto) -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Canon AI Supervisor Control</title>
<style>
:root {
  color-scheme: light dark;
  --bg: #0b1020;
  --panel: #111827;
  --panel-2: #0f172a;
  --line: #263244;
  --text: #e5e7eb;
  --muted: #94a3b8;
  --good: #22c55e;
  --warn: #f59e0b;
  --bad: #ef4444;
  --action: #2563eb;
}
* { box-sizing: border-box; }
body {
  margin: 0;
  background: var(--bg);
  color: var(--text);
  font: 14px/1.5 ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}
main {
  width: min(1180px, calc(100vw - 32px));
  margin: 28px auto 40px;
}
header {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
  margin-bottom: 18px;
}
h1, h2, h3, p { margin: 0; }
h1 { font-size: 22px; letter-spacing: -0.02em; }
h2 { font-size: 15px; margin-bottom: 14px; }
h3 { font-size: 13px; color: var(--muted); margin-bottom: 8px; text-transform: uppercase; letter-spacing: 0.08em; }
.subtle { color: var(--muted); }
.status-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  justify-content: flex-end;
}
.pill {
  border: 1px solid var(--line);
  border-radius: 999px;
  padding: 5px 9px;
  color: var(--muted);
  background: rgba(255,255,255,0.03);
}
.pill.ok { color: var(--good); }
.pill.bad { color: var(--bad); }
.grid {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  gap: 14px;
}
.card {
  grid-column: span 6;
  border: 1px solid var(--line);
  border-radius: 14px;
  background: linear-gradient(180deg, var(--panel), var(--panel-2));
  padding: 16px;
  min-width: 0;
}
.wide { grid-column: span 12; }
.third { grid-column: span 4; }
.kv {
  display: grid;
  grid-template-columns: 150px minmax(0, 1fr);
  gap: 8px 12px;
  align-items: baseline;
}
.kv dt { color: var(--muted); }
.kv dd { margin: 0; min-width: 0; overflow-wrap: anywhere; }
code, .mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
}
.actions { display: flex; flex-wrap: wrap; gap: 10px; }
button, input, textarea {
  font: inherit;
  border-radius: 10px;
  border: 1px solid var(--line);
}
button {
  cursor: pointer;
  padding: 9px 12px;
  background: #1f2937;
  color: var(--text);
}
button.primary { background: var(--action); border-color: var(--action); color: white; }
button.warn { background: #7c2d12; border-color: #9a3412; color: white; }
button.danger { background: #7f1d1d; border-color: #991b1b; color: white; }
button:disabled { cursor: not-allowed; opacity: 0.55; }
input, textarea {
  width: 100%;
  background: #020617;
  color: var(--text);
  padding: 9px 10px;
}
textarea { min-height: 76px; resize: vertical; }
label { display: grid; gap: 5px; color: var(--muted); }
.form-grid { display: grid; gap: 10px; }
.two { display: grid; grid-template-columns: 1fr 140px; gap: 10px; }
.metrics {
  display: grid;
  grid-template-columns: repeat(7, minmax(88px, 1fr));
  gap: 8px;
}
.metric {
  border: 1px solid var(--line);
  border-radius: 12px;
  padding: 10px;
  background: rgba(255,255,255,0.03);
}
.metric strong { display: block; font-size: 20px; }
.bar {
  height: 10px;
  overflow: hidden;
  border-radius: 999px;
  background: #020617;
  border: 1px solid var(--line);
  margin-top: 12px;
}
.bar > span {
  display: block;
  height: 100%;
  width: 0%;
  background: var(--good);
}
.log {
  max-height: 230px;
  overflow: auto;
  padding: 10px;
  border-radius: 10px;
  background: #020617;
  border: 1px solid var(--line);
}
.log div { padding: 3px 0; color: var(--muted); }
.error { color: var(--bad); }
.success { color: var(--good); }
.warn-text { color: var(--warn); }
@media (max-width: 820px) {
  header { display: block; }
  .status-strip { justify-content: flex-start; margin-top: 12px; }
  .card, .third { grid-column: span 12; }
  .metrics { grid-template-columns: repeat(2, minmax(120px, 1fr)); }
  .kv { grid-template-columns: 1fr; }
  .two { grid-template-columns: 1fr; }
}
</style>
</head>
<body>
<main>
  <header>
    <div>
      <h1>Canon AI Supervisor</h1>
      <p class="subtle">Operator console for worker lifecycle, plan status, agents, and workspace.</p>
    </div>
    <div class="status-strip">
      <span id="health-pill" class="pill ok">healthy</span>
      <span class="pill">gen <span id="header-generation">__GENERATION__</span></span>
      <span class="pill">worker <span id="header-worker">__WORKER_PORT__</span></span>
      <span class="pill">updated <span id="last-updated">never</span></span>
    </div>
  </header>

  <section class="grid">
    <section class="card third">
      <h2>Runtime</h2>
      <dl class="kv">
        <dt>Status</dt><dd id="runtime-status" class="success">ok</dd>
        <dt>Worker generation</dt><dd class="mono" id="runtime-generation">__GENERATION__</dd>
        <dt>Worker port</dt><dd class="mono" id="runtime-worker"><code>__WORKER_PORT__</code></dd>
        <dt>Workspace</dt><dd class="mono" id="workspace-root">loading...</dd>
        <dt>Allowed boundary</dt><dd class="mono" id="workspace-boundary">loading...</dd>
      </dl>
    </section>

    <section class="card third">
      <h2>Agent Loop</h2>
      <dl class="kv">
        <dt>Main loop</dt><dd id="agent-running">loading...</dd>
        <dt>Recovery agents</dt><dd class="mono" id="recovery-agent-count">–</dd>
      </dl>
    </section>

    <section class="card third">
      <h2>Actions</h2>
      <div class="actions">
        <button id="refresh-button" class="primary" type="button">Refresh</button>
        <button id="start-loop-button" type="button">Start main loop</button>
        <form method="post" action="/reload">
          <button id="reload-button" class="warn" type="submit">Reload worker</button>
        </form>
        <form method="post" action="/restart" onsubmit="return confirm('Restart the supervisor process? This briefly disconnects the control API.');">
          <button id="restart-button" class="danger" type="submit">Restart supervisor</button>
        </form>
      </div>
      <p class="subtle" style="margin-top:12px"><span class="warn-text">Reload</span> replaces the worker. <span class="error">Restart</span> briefly disconnects this API.</p>
    </section>

    <section class="card wide">
      <h2>Plan Status</h2>
      <div class="metrics">
        <div class="metric"><span class="subtle">Total</span><strong id="plan-total">–</strong></div>
        <div class="metric"><span class="subtle">Ready</span><strong id="plan-ready">–</strong></div>
        <div class="metric"><span class="subtle">Pending</span><strong id="plan-pending">–</strong></div>
        <div class="metric"><span class="subtle">Running</span><strong id="plan-running">–</strong></div>
        <div class="metric"><span class="subtle">Done</span><strong id="plan-done">–</strong></div>
        <div class="metric"><span class="subtle">Failed</span><strong id="plan-failed">–</strong></div>
        <div class="metric"><span class="subtle">Skipped</span><strong id="plan-skipped">–</strong></div>
      </div>
      <div class="bar"><span id="plan-progress"></span></div>
    </section>

    <section class="card">
      <h2>Next Ready Task</h2>
      <dl class="kv">
        <dt>Node</dt><dd class="mono" id="task-node">–</dd>
        <dt>Title</dt><dd id="task-title">–</dd>
        <dt>Ready count</dt><dd id="task-ready-count">–</dd>
        <dt>Description</dt><dd id="task-description">–</dd>
      </dl>
    </section>

    <section class="card">
      <h2>Spawn Child Agent</h2>
      <form id="spawn-form" class="form-grid">
        <label>Domain
          <textarea id="spawn-domain" required placeholder="implement src/domain/records.rs"></textarea>
        </label>
        <div class="two">
          <label>Metric
            <input id="spawn-metric" required placeholder="cargo test passes">
          </label>
          <label>Max steps
            <input id="spawn-steps" type="number" min="1" max="100" value="20">
          </label>
        </div>
        <button class="primary" type="submit">Spawn agent</button>
      </form>
    </section>

    <section class="card wide">
      <h2>Event / API Log</h2>
      <div id="event-log" class="log mono"></div>
    </section>
  </section>
</main>

<script>
const $ = (id) => document.getElementById(id);
const logEl = $('event-log');

function nowStamp() {
  return new Date().toLocaleTimeString([], { hour12: false });
}

function log(message, kind = '') {
  const row = document.createElement('div');
  if (kind) row.className = kind;
  row.textContent = `${nowStamp()} ${message}`;
  logEl.prepend(row);
  while (logEl.children.length > 50) logEl.removeChild(logEl.lastChild);
}

async function jsonFetch(url, options = {}) {
  const response = await fetch(url, {
    headers: { 'content-type': 'application/json', ...(options.headers || {}) },
    ...options,
  });
  if (response.status === 204) return { ok: true, empty: true };
  const text = await response.text();
  const data = text ? JSON.parse(text) : {};
  if (!response.ok) throw new Error(data.error || `${url} returned ${response.status}`);
  return data;
}

function setHealth(data) {
  $('health-pill').textContent = data.ok ? 'healthy' : 'unhealthy';
  $('health-pill').className = `pill ${data.ok ? 'ok' : 'bad'}`;
  $('runtime-status').textContent = data.ok ? 'ok' : 'error';
  $('runtime-status').className = data.ok ? 'success' : 'error';
  $('runtime-generation').textContent = data.generation ?? '–';
  $('runtime-worker').textContent = data.worker_port ?? '–';
  $('header-generation').textContent = data.generation ?? '–';
  $('header-worker').textContent = data.worker_port ?? '–';
}

function setPlan(data) {
  for (const key of ['total', 'ready', 'pending', 'running', 'done', 'failed', 'skipped']) {
    $(`plan-${key}`).textContent = data[key] ?? '–';
  }
  const total = Number(data.total || 0);
  const done = Number(data.done || 0) + Number(data.skipped || 0);
  $('plan-progress').style.width = total > 0 ? `${Math.round((done / total) * 100)}%` : '0%';
}

function setTask(data) {
  if (data.empty) {
    $('task-node').textContent = '–';
    $('task-title').textContent = 'No ready task';
    $('task-ready-count').textContent = '0';
    $('task-description').textContent = '–';
    return;
  }
  $('task-node').textContent = data.node_id ?? '–';
  $('task-title').textContent = data.title ?? '–';
  $('task-ready-count').textContent = data.ready_count ?? '–';
  $('task-description').textContent = data.description ?? '–';
}

function setWorkspace(data) {
  $('workspace-root').textContent = data.workspaceRoot ?? '–';
  $('workspace-boundary').textContent = data.allowedWorkspaceRoot ?? '–';
}

function setAgentStatus(data) {
  $('agent-running').textContent = data.running ? 'running' : 'stopped';
  $('agent-running').className = data.running ? 'success' : 'subtle';
  $('recovery-agent-count').textContent = data.recovery_agent_count ?? 0;
}

async function refreshAll() {
  try {
    setHealth(await jsonFetch('/health'));
    log('GET /health ok', 'success');
  } catch (error) {
    log(`GET /health failed: ${error.message}`, 'error');
  }
  try {
    setWorkspace(await jsonFetch('/ai/workspace'));
    log('GET /ai/workspace ok', 'success');
  } catch (error) {
    log(`GET /ai/workspace failed: ${error.message}`, 'error');
  }
  try {
    setAgentStatus(await jsonFetch('/agent/status'));
    log('GET /agent/status ok', 'success');
  } catch (error) {
    log(`GET /agent/status failed: ${error.message}`, 'error');
  }
  try {
    setPlan(await jsonFetch('/v1/plan/status'));
    log('GET /v1/plan/status ok', 'success');
  } catch (error) {
    log(`GET /v1/plan/status failed: ${error.message}`, 'error');
  }
  try {
    setTask(await jsonFetch('/v1/task/next'));
    log('GET /v1/task/next ok', 'success');
  } catch (error) {
    setTask({ empty: true });
    log(`GET /v1/task/next: ${error.message}`, 'warn-text');
  }
  $('last-updated').textContent = nowStamp();
}

async function postAction(path, body) {
  const data = await jsonFetch(path, {
    method: 'POST',
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  log(`POST ${path} ok ${JSON.stringify(data)}`, 'success');
  await refreshAll();
  return data;
}

$('refresh-button').addEventListener('click', refreshAll);
$('reload-button').form.addEventListener('submit', async (event) => {
  event.preventDefault();
  if (!confirm('Reload the worker and start fresh TLog state?')) return;
  try { await postAction('/reload'); } catch (error) { log(`POST /reload failed: ${error.message}`, 'error'); }
});
$('restart-button').form.addEventListener('submit', async (event) => {
  event.preventDefault();
  if (!confirm('Restart the supervisor process? This briefly disconnects the control API.')) return;
  try { await postAction('/restart'); } catch (error) { log(`POST /restart failed: ${error.message}`, 'error'); }
});
$('start-loop-button').addEventListener('click', async () => {
  try { await postAction('/agent/start', {}); } catch (error) { log(`POST /agent/start failed: ${error.message}`, 'error'); }
});
$('spawn-form').addEventListener('submit', async (event) => {
  event.preventDefault();
  const body = {
    domain: $('spawn-domain').value.trim(),
    metric: $('spawn-metric').value.trim(),
    max_steps: Number($('spawn-steps').value || 20),
  };
  try { await postAction('/spawn', body); } catch (error) { log(`POST /spawn failed: ${error.message}`, 'error'); }
});

refreshAll();
setInterval(refreshAll, 10000);
</script>
</body>
</html>"#
        .replace("__GENERATION__", &health.generation.to_string())
        .replace("__WORKER_PORT__", &health.worker_port.to_string())
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

pub async fn agent_status_handler(
    AxumState(state): AxumState<SupervisorState>,
) -> Result<Json<AgentStatusDto>, (StatusCode, Json<ErrorDto>)> {
    let guard = state.inner.lock().await;
    Ok(Json(guard.agent_status()))
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
        .spawn_agent(&body.domain, &body.metric, max_steps, None)
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
        guard
            .ensure_worker_alive_or_reload()
            .await
            .map_err(error_response)?
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_page_dashboard_contains_operator_markers() {
        let html = render_control_page_html(&HealthDto {
            ok: true,
            generation: 7,
            worker_port: 43123,
        });

        for marker in [
            "Canon AI Supervisor",
            "Operator console for worker lifecycle, plan status, agents, and workspace.",
            "id=\"refresh-button\"",
            "id=\"start-loop-button\"",
            "id=\"reload-button\"",
            "id=\"restart-button\"",
            "action=\"/reload\"",
            "action=\"/restart\"",
            "id=\"agent-running\"",
            "id=\"spawn-form\"",
            "id=\"event-log\"",
            "GET /agent/status ok",
            "GET /v1/plan/status ok",
            "GET /v1/task/next ok",
            "POST /agent/start",
            "POST /spawn",
            "POST /reload",
            "POST /restart",
        ] {
            assert!(
                html.contains(marker),
                "missing control page marker: {marker}"
            );
        }

        assert!(html.contains("gen <span id=\"header-generation\">7</span>"));
        assert!(html.contains("worker <span id=\"header-worker\">43123</span>"));
        assert!(html.contains("<code>43123</code>"));
    }
}

fn error_response(error: String) -> (StatusCode, Json<ErrorDto>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorDto { ok: false, error }),
    )
}
