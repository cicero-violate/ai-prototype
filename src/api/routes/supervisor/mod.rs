//! Axum-facing supervisor routes.

pub mod control;
pub mod mcp;
pub mod oauth;
pub mod router;
pub mod workspace;

pub use control::{
    command_gateway, control_page, get_task_next, health, reload, restart, spawn_agent_handler,
    start_agent_loop_handler,
};
pub use mcp::{ai_mcp_delete, ai_mcp_get_sse, ai_mcp_post};
pub use oauth::{
    ai_oauth_authorize_get, ai_oauth_authorize_post, ai_oauth_metadata,
    ai_oauth_protected_resource_metadata, ai_oauth_register, ai_oauth_token, require_ai_mcp_auth,
};
pub use router::build_supervisor_router;
pub use workspace::{ai_workspace_get, ai_workspace_update};
