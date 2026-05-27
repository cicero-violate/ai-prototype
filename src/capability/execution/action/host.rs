//! Action host trait and bounded native tool execution.

pub mod agents;
pub mod graph;
pub mod project;
pub mod utility;
pub mod workspace;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::api::protocol::Command as KernelCommand;
use crate::runtime::WorkspaceView;

#[expect(
    async_fn_in_trait,
    reason = "action host trait is internal and not exposed as a public downstream API"
)]
pub trait ActionHost {
    fn workspace(&self) -> WorkspaceView;

    async fn active_generation(&self) -> u64;

    fn record_session(&self, session_id: String, worker_generation: u64, created_at: DateTime<Utc>);

    async fn submit_kernel_command(&self, command: KernelCommand) -> Result<(), String>;

    async fn run_host_tool(&self, name: &str, args: &Value) -> Option<HostToolResult>;
}

/// Typed boundary for supervisor-owned host tools.
///
/// Supervisor host tools still serialize to MCP's JSON envelope at the transport
/// edge, but success/error state must flow through this enum until that point.
pub enum HostToolResult {
    Success { text: String },
    Error { message: String },
}

impl HostToolResult {
    pub fn success_text(text: String) -> Self {
        Self::Success { text }
    }

    pub fn error_message(message: String) -> Self {
        Self::Error { message }
    }

    pub fn is_error(&self) -> bool {
        self.is_error_variant()
    }

    fn is_error_variant(&self) -> bool {
        match self {
            Self::Error { .. } => true,
            Self::Success { .. } => false,
        }
    }

    pub fn into_outcome(self) -> ActionToolOutcome {
        match self {
            Self::Success { text } => ActionToolOutcome::Ok(serde_json::json!({
                "content": [{ "type": "text", "text": text }],
                "isError": false
            })),
            Self::Error { message } => {
                ActionToolOutcome::Error(crate::api::action::tool_error(message))
            }
        }
    }
}

/// Single source of truth for a tool's execution outcome.
///
/// Callers derive `exit_status` and `timed_out` from the variant — never by
/// re-parsing fields in the payload JSON.
pub enum ActionToolOutcome {
    Ok(Value),
    Error(Value),
    TimedOut(Value),
}

impl ActionToolOutcome {
    /// Construct from a `Value` produced by a tool that has no timeout
    /// capability. Maps `isError: true` → `Error`, everything else → `Ok`.
    pub fn from_value(v: Value) -> Self {
        if v.get("isError").and_then(Value::as_bool).unwrap_or(false) {
            Self::Error(v)
        } else {
            Self::Ok(v)
        }
    }

    pub fn exit_status(&self) -> u64 {
        match self {
            Self::Ok(_) => 0,
            Self::Error(_) | Self::TimedOut(_) => 1,
        }
    }

    pub fn timed_out(&self) -> bool {
        matches!(self, Self::TimedOut(_))
    }

    pub fn into_value(self) -> Value {
        match self {
            Self::Ok(v) | Self::Error(v) | Self::TimedOut(v) => v,
        }
    }
}

pub async fn execute_native_tool<H: ActionHost>(
    name: &str,
    args: &Value,
    host: &H,
) -> ActionToolOutcome {
    match name {
        "echo" | "get_current_time" => utility::execute(name, args),
        "apply_patch" | "shell" | "python" | "structural_edit" => {
            workspace::execute(name, args, host).await
        }
        "canon_graph_plan_patch"
        | "canon_graph_plan_cfg"
        | "canon_graph_apply_ops"
        | "canon_graph_verify_cfg_delta"
        | "canon_graph_auto_refactor_cfg" => graph::execute(name, args, host),
        "canon_score"
        | "canon_diagnostics_read"
        | "canon_invariants_mine"
        | "canon_invariants_validate"
        | "canon_invariants_promote"
        | "canon_invariants_read"
        | "canon_plan_read"
        | "canon_plan_update" => project::execute(name, args, host),
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
        _ => ActionToolOutcome::Error(crate::api::action::tool_error(format!(
            "Unknown tool: {name}"
        ))),
    }
}

pub async fn execute_recorded_shell<H: ActionHost>(args: &Value, host: &H) -> ActionToolOutcome {
    workspace::execute_recorded_shell(args, host).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::WorkspaceView;
    use std::path::PathBuf;

    struct NullHost;

    impl ActionHost for NullHost {
        fn workspace(&self) -> WorkspaceView {
            WorkspaceView {
                root: PathBuf::from("."),
                allowed_boundary: PathBuf::from("."),
            }
        }

        async fn active_generation(&self) -> u64 {
            0
        }

        fn record_session(
            &self,
            _session_id: String,
            _worker_generation: u64,
            _created_at: DateTime<Utc>,
        ) {
        }

        async fn submit_kernel_command(&self, _command: KernelCommand) -> Result<(), String> {
            Ok(())
        }

        async fn run_host_tool(&self, _name: &str, _args: &Value) -> Option<HostToolResult> {
            None
        }
    }

    #[tokio::test]
    async fn unknown_native_tool_is_typed_error_variant() {
        let outcome = execute_native_tool("canon_missing_tool", &Value::Null, &NullHost).await;
        assert_eq!(outcome.exit_status(), 1);
        assert!(matches!(outcome, ActionToolOutcome::Error(_)));
    }

    #[tokio::test]
    async fn unknown_supervisor_host_tool_is_typed_error_variant() {
        let outcome = execute_native_tool("canon_supervisor_health", &Value::Null, &NullHost).await;
        assert_eq!(outcome.exit_status(), 1);
        assert!(matches!(outcome, ActionToolOutcome::Error(_)));
    }
}
