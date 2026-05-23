//! Typed native tooling capability interface used by MCP transport adapters.
//!
//! MCP dispatch owns JSON-RPC transport, gateway discovery, call receipts, and
//! sequencing. These capability-owned modules own native execution for graph
//! editing/analysis, patch application, shell execution, scoring, plan mutation,
//! agent host actions, and mailbox access.

mod agents;
mod graph;
mod project;
mod utility;
mod workspace;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::api::action::tool_error;
use crate::api::protocol::Command as KernelCommand;
use crate::runtime::WorkspaceView;

#[expect(
    async_fn_in_trait,
    reason = "native tooling host trait is internal and not exposed as a public downstream API"
)]
pub trait NativeToolHost {
    fn workspace(&self) -> WorkspaceView;

    async fn active_generation(&self) -> u64;

    fn record_session(&self, session_id: String, worker_generation: u64, created_at: DateTime<Utc>);

    async fn submit_kernel_command(&self, command: KernelCommand) -> Result<(), String>;

    async fn run_host_tool(&self, name: &str, args: &Value) -> Option<Value>;
}

pub async fn execute_native_tool<H: NativeToolHost>(name: &str, args: &Value, host: &H) -> Value {
    match name {
        "echo" | "get_current_time" => utility::execute(name, args),
        "apply_patch" | "shell" | "python" => workspace::execute(name, args, host).await,
        "canon_graph_plan_patch"
        | "canon_graph_plan_cfg"
        | "canon_graph_apply_ops"
        | "canon_graph_verify_cfg_delta"
        | "canon_graph_auto_refactor_cfg" => graph::execute(name, args, host),
        "canon_score" | "canon_plan_read" | "canon_plan_update" => {
            project::execute(name, args, host)
        }
        "canon_spawn_agent"
        | "canon_runtime_state"
        | "canon_supervisor_health"
        | "canon_supervisor_reload_worker"
        | "canon_supervisor_restart"
        | "canon_workspace_get"
        | "canon_workspace_set"
        | "canon_browser_list_tabs"
        | "canon_browser_close_tab"
        | "canon_browser_upload"
        | "canon_browser_group_chat"
        | "canon_send_agent_message"
        | "canon_read_mailbox" => agents::execute(name, args, host).await,
        _ => tool_error(format!("Unknown tool: {name}")),
    }
}

pub async fn execute_recorded_shell<H: NativeToolHost>(args: &Value, host: &H) -> Value {
    workspace::execute_recorded_shell(args, host).await
}
