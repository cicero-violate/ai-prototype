//! API route layers.
//!
//! `command_routes` contains deterministic kernel command handlers.
//! `supervisor_routes` contains Axum-facing supervisor route handlers.

pub mod command_routes;
pub mod supervisor;

pub use command_routes::{handle_command, handle_envelope, handle_envelope_once};
pub use supervisor::{
    ai_mcp_delete, ai_mcp_get_sse, ai_mcp_post, ai_oauth_authorize_get, ai_oauth_authorize_post,
    ai_oauth_metadata, ai_oauth_protected_resource_metadata, ai_oauth_register, ai_oauth_token,
    ai_workspace_get, ai_workspace_update, build_supervisor_router, command_gateway, get_task_next,
    health, reload, require_ai_mcp_auth, spawn_agent_handler, start_agent_loop_handler,
};
