//! Workspace supervisor route handlers.

use axum::extract::State as AxumState;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::process::supervisor::WorkspaceConfig;

use crate::process::supervisor::SupervisorState;

#[derive(Deserialize)]
pub struct WorkspaceUpdateBody {
    root: std::path::PathBuf,
}

pub async fn ai_workspace_get(AxumState(state): AxumState<SupervisorState>) -> Json<Value> {
    Json(state.workspace_status_json())
}

pub async fn ai_workspace_update(
    AxumState(state): AxumState<SupervisorState>,
    Json(body): Json<WorkspaceUpdateBody>,
) -> Response {
    let allowed = state
        .mcp
        .workspace
        .lock()
        .expect("workspace mutex should not be poisoned")
        .allowed_boundary
        .clone();
    let next = match WorkspaceConfig::new(body.root, allowed) {
        Ok(next) => next,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"ok": false, "error": error})),
            )
                .into_response();
        }
    };
    *state
        .mcp
        .workspace
        .lock()
        .expect("workspace mutex should not be poisoned") = next;
    Json(workspace_json(&state)).into_response()
}

fn workspace_json(state: &SupervisorState) -> Value {
    let workspace = state
        .mcp
        .workspace
        .lock()
        .expect("workspace mutex should not be poisoned")
        .clone();
    json!({
        "ok": true,
        "workspaceRoot": workspace.root.display().to_string(),
        "allowedWorkspaceRoot": workspace.allowed_boundary.display().to_string()
    })
}
