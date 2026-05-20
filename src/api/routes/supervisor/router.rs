//! Supervisor Axum router composition.

use axum::routing::{get, post};
use axum::Router;

use super::control::{
    command_gateway, control_page, get_task_next, health, reload, restart, spawn_agent_handler,
    start_agent_loop_handler,
};
use super::mcp::{ai_mcp_delete, ai_mcp_get_sse, ai_mcp_post};
use super::oauth::{
    ai_oauth_authorize_get, ai_oauth_authorize_post, ai_oauth_metadata,
    ai_oauth_protected_resource_metadata, ai_oauth_register, ai_oauth_token,
};
use super::workspace::{ai_workspace_get, ai_workspace_update};
use crate::process::supervisor::SupervisorState;

pub fn build_supervisor_router(state: SupervisorState) -> Router {
    Router::new()
        .route("/", get(control_page))
        .route("/control", get(control_page))
        .route("/health", get(health))
        .route("/reload", post(reload))
        .route("/restart", post(restart))
        .route("/spawn", post(spawn_agent_handler))
        .route("/agent/start", post(start_agent_loop_handler))
        .route("/v1/command", post(command_gateway))
        .route("/v1/task/next", get(get_task_next))
        .route(
            "/ai/.well-known/oauth-authorization-server",
            get(ai_oauth_metadata),
        )
        .route(
            "/.well-known/oauth-authorization-server/ai",
            get(ai_oauth_metadata),
        )
        .route(
            "/.well-known/oauth-authorization-server/ai/mcp",
            get(ai_oauth_metadata),
        )
        .route(
            "/ai/.well-known/oauth-protected-resource",
            get(ai_oauth_protected_resource_metadata),
        )
        .route(
            "/ai/.well-known/oauth-protected-resource/mcp",
            get(ai_oauth_protected_resource_metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource/ai/mcp",
            get(ai_oauth_protected_resource_metadata),
        )
        .route("/ai/register", post(ai_oauth_register))
        .route(
            "/ai/authorize",
            get(ai_oauth_authorize_get).post(ai_oauth_authorize_post),
        )
        .route("/ai/token", post(ai_oauth_token))
        .route(
            "/ai/mcp",
            post(ai_mcp_post).get(ai_mcp_get_sse).delete(ai_mcp_delete),
        )
        .route(
            "/ai/workspace",
            get(ai_workspace_get).post(ai_workspace_update),
        )
        .with_state(state)
}
