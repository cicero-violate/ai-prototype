//! Core supervisor health, reload, spawn, and command gateway routes.

use axum::body::Bytes;
use axum::extract::State as AxumState;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::process::supervisor::{HealthDto, ReloadDto, SpawnDto, SpawnRequest};

use crate::process::supervisor::{ErrorDto, SupervisorState};

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

pub async fn spawn_agent_handler(
    AxumState(state): AxumState<SupervisorState>,
    Json(body): Json<SpawnRequest>,
) -> Result<Json<SpawnDto>, (StatusCode, Json<ErrorDto>)> {
    let mut guard = state.inner.lock().await;
    let max_steps = body.max_steps.unwrap_or(20).max(1).min(100);
    guard
        .spawn_agent(&body.domain, &body.metric, max_steps)
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
        guard.reap_retired().await;
        guard.active_worker_port().map_err(error_response)?
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

fn error_response(error: String) -> (StatusCode, Json<ErrorDto>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorDto { ok: false, error }),
    )
}
