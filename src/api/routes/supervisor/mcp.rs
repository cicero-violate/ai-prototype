//! MCP-facing supervisor route handlers.

use axum::extract::State as AxumState;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures::stream;
use serde_json::Value;

use crate::api::action::action_err;

use super::oauth::require_ai_mcp_auth;
use crate::service::dispatch::dispatch_action_request;
use crate::service::supervisor::SupervisorState;

pub async fn ai_mcp_post(
    AxumState(state): AxumState<SupervisorState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    if let Some(response) = require_ai_mcp_auth(&state, &headers) {
        return response;
    }
    let messages: Vec<&Value> = body
        .as_array()
        .map_or_else(|| vec![&body], |items| items.iter().collect());
    if messages
        .iter()
        .all(|message| message.get("id").is_none_or(Value::is_null))
    {
        return StatusCode::ACCEPTED.into_response();
    }

    let mut responses = Vec::new();
    let mut new_session_id = None;
    for message in messages {
        let Some(id) = message.get("id").filter(|id| !id.is_null()).cloned() else {
            continue;
        };
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            responses.push(action_err(id, -32600, "missing 'method'"));
            continue;
        };
        let params = message.get("params").cloned().unwrap_or(Value::Null);
        let (response, sid) = dispatch_action_request(method, id, params, &state).await;
        if sid.is_some() {
            new_session_id = sid;
        }
        responses.push(response);
    }
    let body = if responses.len() == 1 {
        responses.remove(0)
    } else {
        Value::Array(responses)
    };
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(sid) = new_session_id {
        builder = builder.header("mcp-session-id", sid);
    }
    builder
        .body(body.to_string().into())
        .expect("building JSON-RPC response should succeed")
}

pub async fn ai_mcp_get_sse(
    AxumState(state): AxumState<SupervisorState>,
    headers: HeaderMap,
) -> Response {
    if let Some(response) = require_ai_mcp_auth(&state, &headers) {
        return response;
    }
    let stream = stream::pending::<Result<Event, std::convert::Infallible>>();
    Sse::new(stream)
        .keep_alive(KeepAlive::default())
        .into_response()
}

pub async fn ai_mcp_delete(
    AxumState(state): AxumState<SupervisorState>,
    headers: HeaderMap,
) -> StatusCode {
    if require_ai_mcp_auth(&state, &headers).is_some() {
        return StatusCode::UNAUTHORIZED;
    }
    match headers
        .get("mcp-session-id")
        .and_then(|value| value.to_str().ok())
    {
        Some(sid) => {
            state
                .mcp
                .sessions
                .lock()
                .expect("sessions mutex should not be poisoned")
                .remove(sid);
            StatusCode::OK
        }
        None => StatusCode::BAD_REQUEST,
    }
}
