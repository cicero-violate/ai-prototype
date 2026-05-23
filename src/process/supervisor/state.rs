//! Shared supervisor route state.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};

use super::process::WorkerProcess;
use crate::api::oauth::OAuthStore;
use crate::api::protocol::Command as KernelCommand;
use crate::capability::execution::action::ActionHost;
use crate::capability::tooling::mcp_tools::SpawnAgentToolRequest;
use crate::process::scheduler::handler::TaskReadyNotifier;
use crate::process::supervisor::SupervisorConfig;
use crate::process::supervisor::WorkspaceConfig;
use std::time::Duration;

#[derive(Clone)]
pub struct SupervisorState {
    pub inner: Arc<tokio::sync::Mutex<WorkerProcess>>,
    pub mcp: Arc<NativeMcpState>,
    /// Shared wakeup signal — supervisor writes, task runners read.
    /// Notified when a node transitions to Pending (retry, reset, plan update).
    pub task_ready_notifier: TaskReadyNotifier,
}

impl SupervisorState {
    pub fn new(process: WorkerProcess, cfg: &SupervisorConfig) -> Result<Self, String> {
        Ok(Self {
            inner: Arc::new(tokio::sync::Mutex::new(process)),
            mcp: Arc::new(NativeMcpState::from_config(cfg)?),
            task_ready_notifier: TaskReadyNotifier::new(),
        })
    }

    pub async fn restart_supervisor(
        &self,
    ) -> Result<crate::process::supervisor::RestartDto, String> {
        let replacement = crate::api::routes::supervisor::control::schedule_supervisor_replacement(
            crate::api::routes::supervisor::control::SUPERVISOR_RESTART_DELAY_MS,
        )?;
        Ok(crate::process::supervisor::RestartDto {
            ok: true,
            pid: std::process::id(),
            replacement,
            delay_ms: crate::api::routes::supervisor::control::SUPERVISOR_RESTART_DELAY_MS,
        })
    }

    pub fn workspace_status_json(&self) -> Value {
        let workspace = self.workspace();
        json!({
            "ok": true,
            "workspaceRoot": workspace.root.display().to_string(),
            "allowedWorkspaceRoot": workspace.allowed_boundary.display().to_string()
        })
    }

    pub fn set_workspace_root(&self, root: std::path::PathBuf) -> Result<Value, String> {
        let allowed = self
            .mcp
            .workspace
            .lock()
            .expect("workspace mutex should not be poisoned")
            .allowed_boundary
            .clone();
        let next = WorkspaceConfig::new(root, allowed)?;
        *self
            .mcp
            .workspace
            .lock()
            .expect("workspace mutex should not be poisoned") = next;
        Ok(self.workspace_status_json())
    }
}

impl ActionHost for SupervisorState {
    fn workspace(&self) -> WorkspaceConfig {
        self.mcp
            .workspace
            .lock()
            .expect("workspace mutex should not be poisoned")
            .clone()
    }

    async fn active_generation(&self) -> u64 {
        self.inner.lock().await.active_generation().unwrap_or(0)
    }

    fn record_session(
        &self,
        session_id: String,
        worker_generation: u64,
        created_at: DateTime<Utc>,
    ) {
        self.mcp
            .sessions
            .lock()
            .expect("sessions mutex should not be poisoned")
            .insert(
                session_id,
                NativeMcpSession {
                    _worker_generation: worker_generation,
                    _created_at: created_at,
                },
            );
    }

    async fn submit_kernel_command(&self, command: KernelCommand) -> Result<(), String> {
        let command_id = self.mcp.next_command_id();
        let worker_port = {
            let mut guard = self.inner.lock().await;
            guard.ensure_worker_alive_or_reload().await?
        };
        crate::api::action::proxy::submit_action_kernel_command(command_id, worker_port, command).await
    }

    async fn run_host_tool(&self, name: &str, args: &Value) -> Option<Value> {
        eprintln!("[canon-ai-supervisor] host tool start name={name}");
        let result = match name {
            "canon_spawn_agent" => Some(self.run_spawn_agent(args).await),
            "canon_runtime_state" => Some(self.run_runtime_state().await),
            "canon_supervisor_health" => Some(self.run_supervisor_health().await),
            "canon_supervisor_reload_worker" => Some(self.run_supervisor_reload_worker().await),
            "canon_supervisor_restart" => Some(self.run_supervisor_restart().await),
            "canon_workspace_get" => Some(self.run_workspace_get()),
            "canon_workspace_set" => Some(self.run_workspace_set(args)),
            "canon_browser_list_tabs" => Some(self.run_browser_get("/tabs").await),
            "canon_browser_close_tab" => Some(self.run_browser_close_tab(args).await),
            "canon_browser_upload" => {
                Some(self.run_browser_post("/actions/upload", args.clone()).await)
            }
            "canon_browser_group_chat" => Some(
                self.run_browser_post("/actions/group-chat", args.clone())
                    .await,
            ),
            _ => None,
        };
        eprintln!(
            "[canon-ai-supervisor] host tool finish name={} handled={} is_error={}",
            name,
            result.is_some(),
            result
                .as_ref()
                .and_then(|value| value.get("isError"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
        );
        result
    }
}

impl SupervisorState {
    async fn run_spawn_agent(&self, args: &Value) -> Value {
        eprintln!("[canon-ai-supervisor] agents:spawn requested");
        let request = match SpawnAgentToolRequest::parse(args) {
            Ok(request) => request,
            Err(error) => return crate::api::action::tool_error(error),
        };
        let mut guard = self.inner.lock().await;
        match guard.spawn_agent(&request.domain, &request.metric, request.max_steps) {
            Ok(dto) => json_text_result(dto),
            Err(error) => crate::api::action::tool_error(error),
        }
    }

    async fn run_runtime_state(&self) -> Value {
        eprintln!("[canon-ai-supervisor] runtime:state requested");
        let worker_port = {
            let mut guard = self.inner.lock().await;
            match guard.ensure_worker_alive_or_reload().await {
                Ok(port) => port,
                Err(error) => return crate::api::action::tool_error(error),
            }
        };
        local_json_get(&format!("http://127.0.0.1:{worker_port}/v1/state")).await
    }

    async fn run_supervisor_health(&self) -> Value {
        eprintln!("[canon-ai-supervisor] supervisor:health requested");
        let mut guard = self.inner.lock().await;
        match guard.health().await {
            Ok(dto) => json_text_result(dto),
            Err(error) => crate::api::action::tool_error(error),
        }
    }

    async fn run_supervisor_reload_worker(&self) -> Value {
        eprintln!("[canon-ai-supervisor] supervisor:reload_worker requested");
        let mut guard = self.inner.lock().await;
        match guard.reload_inner().await {
            Ok(dto) => json_text_result(dto),
            Err(error) => crate::api::action::tool_error(error),
        }
    }

    async fn run_supervisor_restart(&self) -> Value {
        eprintln!("[canon-ai-supervisor] supervisor:restart requested");
        let replacement =
            match crate::api::routes::supervisor::control::schedule_supervisor_replacement(
                crate::api::routes::supervisor::control::SUPERVISOR_RESTART_DELAY_MS,
            ) {
                Ok(replacement) => replacement,
                Err(error) => return crate::api::action::tool_error(error),
            };
        let pid = std::process::id();
        let state = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(
                crate::api::routes::supervisor::control::SUPERVISOR_EXIT_DELAY_MS,
            ))
            .await;
            let mut guard = state.inner.lock().await;
            guard.shutdown().await;
            std::process::exit(0);
        });
        json_text_result(json!({
            "ok": true,
            "pid": pid,
            "replacement": replacement,
            "delay_ms": crate::api::routes::supervisor::control::SUPERVISOR_RESTART_DELAY_MS
        }))
    }

    fn run_workspace_get(&self) -> Value {
        eprintln!("[canon-ai-supervisor] workspace:get requested");
        let workspace = self
            .mcp
            .workspace
            .lock()
            .expect("workspace mutex should not be poisoned")
            .clone();
        json_text_result(json!({
            "ok": true,
            "workspaceRoot": workspace.root.display().to_string(),
            "allowedWorkspaceRoot": workspace.allowed_boundary.display().to_string()
        }))
    }

    fn run_workspace_set(&self, args: &Value) -> Value {
        eprintln!("[canon-ai-supervisor] workspace:set requested");
        let Some(root) = args
            .get("root")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
        else {
            return crate::api::action::tool_error(
                "workspace:set requires non-empty 'root'".to_string(),
            );
        };
        let allowed = self
            .mcp
            .workspace
            .lock()
            .expect("workspace mutex should not be poisoned")
            .allowed_boundary
            .clone();
        let next = match WorkspaceConfig::new(root.into(), allowed) {
            Ok(next) => next,
            Err(error) => return crate::api::action::tool_error(error),
        };
        *self
            .mcp
            .workspace
            .lock()
            .expect("workspace mutex should not be poisoned") = next;
        self.run_workspace_get()
    }

    async fn run_browser_get(&self, path: &str) -> Value {
        eprintln!("[canon-ai-supervisor] browser GET requested path={path}");
        local_json_get(&format!("{}{}", self.browser_router_base_url(), path)).await
    }

    async fn run_browser_close_tab(&self, args: &Value) -> Value {
        eprintln!("[canon-ai-supervisor] browser:close_tab requested");
        let Some(target_id) = args
            .get("target_id")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
        else {
            return crate::api::action::tool_error(
                "browser:close_tab requires non-empty 'target_id'".to_string(),
            );
        };
        local_json_delete(&format!(
            "{}/tabs/{}",
            self.browser_router_base_url(),
            url_path_segment(target_id)
        ))
        .await
    }

    async fn run_browser_post(&self, path: &str, body: Value) -> Value {
        eprintln!("[canon-ai-supervisor] browser POST requested path={path}");
        local_json_post(&format!("{}{}", self.browser_router_base_url(), path), body).await
    }

    fn browser_router_base_url(&self) -> String {
        self.mcp
            .router_url
            .strip_suffix("/v1")
            .unwrap_or(&self.mcp.router_url)
            .trim_end_matches('/')
            .to_string()
    }
}

fn json_text_result(value: impl Serialize) -> Value {
    let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| "null".to_string());
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": false
    })
}

async fn local_json_get(url: &str) -> Value {
    eprintln!("[canon-ai-supervisor] outbound GET {url}");
    let client = match local_http_client() {
        Ok(client) => client,
        Err(error) => return crate::api::action::tool_error(error),
    };
    match client.get(url).send().await {
        Ok(response) => response_to_tool_result(response).await,
        Err(error) => crate::api::action::tool_error(format!("GET {url} failed: {error}")),
    }
}

async fn local_json_delete(url: &str) -> Value {
    eprintln!("[canon-ai-supervisor] outbound DELETE {url}");
    let client = match local_http_client() {
        Ok(client) => client,
        Err(error) => return crate::api::action::tool_error(error),
    };
    match client.delete(url).send().await {
        Ok(response) => response_to_tool_result(response).await,
        Err(error) => crate::api::action::tool_error(format!("DELETE {url} failed: {error}")),
    }
}

async fn local_json_post(url: &str, body: Value) -> Value {
    eprintln!("[canon-ai-supervisor] outbound POST {url}");
    let client = match local_http_client() {
        Ok(client) => client,
        Err(error) => return crate::api::action::tool_error(error),
    };
    match client.post(url).json(&body).send().await {
        Ok(response) => response_to_tool_result(response).await,
        Err(error) => crate::api::action::tool_error(format!("POST {url} failed: {error}")),
    }
}

async fn response_to_tool_result(response: reqwest::Response) -> Value {
    let status = response.status();
    let text = match response.text().await {
        Ok(text) => text,
        Err(error) => {
            return crate::api::action::tool_error(format!("response body read failed: {error}"))
        }
    };
    if !status.is_success() {
        return crate::api::action::tool_error(format!("HTTP {}: {}", status.as_u16(), text));
    }
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": false
    })
}

fn local_http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| format!("HTTP client build failed: {error}"))
}

fn url_path_segment(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            other => format!("%{other:02X}").chars().collect(),
        })
        .collect()
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ErrorDto {
    pub ok: bool,
    pub error: String,
}

pub struct NativeMcpState {
    pub sessions: StdMutex<HashMap<String, NativeMcpSession>>,
    pub oauth: StdMutex<OAuthStore>,
    pub workspace: StdMutex<WorkspaceConfig>,
    pub base_url: String,
    pub router_url: String,
    next_command_id: AtomicU64,
}

#[derive(Clone)]
pub struct NativeMcpSession {
    pub _worker_generation: u64,
    pub _created_at: DateTime<Utc>,
}

impl NativeMcpState {
    pub fn from_config(cfg: &SupervisorConfig) -> Result<Self, String> {
        Ok(Self {
            sessions: StdMutex::new(HashMap::new()),
            oauth: StdMutex::new(OAuthStore::load(
                cfg.oauth_store_file.clone(),
                cfg.oauth_store_key.clone(),
            )?),
            workspace: StdMutex::new(WorkspaceConfig::new(
                cfg.project_dir.clone(),
                cfg.mcp_allowed_workspace_root.clone(),
            )?),
            base_url: cfg.mcp_base_url.trim_end_matches('/').to_string(),
            router_url: cfg.router_url.trim_end_matches('/').to_string(),
            next_command_id: AtomicU64::new(initial_native_mcp_command_id()),
        })
    }

    pub fn next_command_id(&self) -> u64 {
        self.next_command_id.fetch_add(1, Ordering::Relaxed).max(1)
    }
}

fn initial_native_mcp_command_id() -> u64 {
    Utc::now()
        .timestamp_nanos_opt()
        .unwrap_or(1)
        .unsigned_abs()
        .max(1)
}
