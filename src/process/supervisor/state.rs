//! Shared supervisor route state.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex as StdMutex};

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use super::process::WorkerProcess;
use crate::api::oauth::OAuthStore;
use crate::api::protocol::Command as KernelCommand;
use crate::capability::tooling::mcp_tools::{McpToolHost, SpawnAgentToolRequest};
use crate::process::supervisor::SupervisorConfig;
use crate::process::supervisor::WorkspaceConfig;

#[derive(Clone)]
pub struct SupervisorState {
    pub inner: Arc<tokio::sync::Mutex<WorkerProcess>>,
    pub mcp: Arc<NativeMcpState>,
}

impl SupervisorState {
    pub fn new(process: WorkerProcess, cfg: &SupervisorConfig) -> Result<Self, String> {
        Ok(Self {
            inner: Arc::new(tokio::sync::Mutex::new(process)),
            mcp: Arc::new(NativeMcpState::from_config(cfg)?),
        })
    }
}

impl McpToolHost for SupervisorState {
    fn workspace(&self) -> WorkspaceConfig {
        self.mcp.workspace.lock().unwrap().clone()
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
        self.mcp.sessions.lock().unwrap().insert(
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
            guard.reap_retired().await;
            guard.active_worker_port()?
        };
        crate::api::mcp::submit_mcp_kernel_command(command_id, worker_port, command).await
    }

    async fn run_host_tool(&self, name: &str, args: &Value) -> Option<Value> {
        match name {
            "canon_spawn_agent" => {
                let request = match SpawnAgentToolRequest::parse(args) {
                    Ok(request) => request,
                    Err(error) => return Some(crate::api::mcp::tool_error(error)),
                };
                let mut guard = self.inner.lock().await;
                Some(
                    match guard.spawn_agent(&request.domain, &request.metric, request.max_steps) {
                        Ok(dto) => {
                            let text = serde_json::to_string_pretty(&dto).unwrap_or_default();
                            serde_json::json!({
                                "content": [{ "type": "text", "text": text }],
                                "isError": false
                            })
                        }
                        Err(error) => crate::api::mcp::tool_error(error),
                    },
                )
            }
            _ => None,
        }
    }
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
