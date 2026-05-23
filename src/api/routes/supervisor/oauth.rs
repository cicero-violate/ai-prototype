//! OAuth-facing supervisor route handlers.

use axum::extract::{Form, Query, State as AxumState};
use axum::http::{header, HeaderMap};
use axum::response::{Html, IntoResponse, Response};
use axum::Json;
use serde_json::Value;

use crate::api::oauth::{
    auth_error_response as oauth_auth_error_response, html_escape as oauth_html_escape,
    metadata_value as oauth_metadata_value,
    protected_resource_metadata_value as oauth_protected_resource_metadata_value, AuthorizeForm,
    AuthorizeQuery, RegisterBody, TokenForm,
};

use crate::service::supervisor::SupervisorState;

pub async fn ai_oauth_metadata(AxumState(state): AxumState<SupervisorState>) -> Json<Value> {
    Json(ai_oauth_metadata_value(&state))
}

fn ai_oauth_metadata_value(state: &SupervisorState) -> Value {
    oauth_metadata_value(&state.mcp.base_url)
}

pub async fn ai_oauth_protected_resource_metadata(
    AxumState(state): AxumState<SupervisorState>,
) -> Json<Value> {
    Json(ai_oauth_protected_resource_metadata_value(&state))
}

fn ai_oauth_protected_resource_metadata_value(state: &SupervisorState) -> Value {
    oauth_protected_resource_metadata_value(&state.mcp.base_url)
}

pub async fn ai_oauth_register(
    AxumState(state): AxumState<SupervisorState>,
    Json(body): Json<RegisterBody>,
) -> Response {
    let mut oauth = state
        .mcp
        .oauth
        .lock()
        .expect("oauth mutex should not be poisoned");
    oauth.register_client_response(body)
}

pub async fn ai_oauth_authorize_get(
    AxumState(state): AxumState<SupervisorState>,
    Query(q): Query<AuthorizeQuery>,
) -> Response {
    let oauth = state
        .mcp
        .oauth
        .lock()
        .expect("oauth mutex should not be poisoned");
    let page = match oauth.authorize_page_context(&q) {
        Ok(page) => page,
        Err(response) => return response,
    };
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head><meta charset="utf-8"><title>Authorize - Canon AI MCP</title>
<style>body{{font-family:system-ui,sans-serif;max-width:440px;margin:80px auto;padding:0 24px;color:#111}}.card{{border:1px solid #e5e7eb;border-radius:8px;padding:28px;margin-top:24px}}button{{padding:10px 24px;font-size:15px;border:0;border-radius:6px;cursor:pointer}}.approve{{background:#166534;color:#fff}}.deny{{background:#f3f4f6;color:#374151}}</style>
</head>
<body><h2>Authorize access</h2><div class="card">
<p><strong>{name}</strong> is requesting access to Canon AI MCP.</p>
<form method="POST" action="/ai/authorize">
<input type="hidden" name="client_id" value="{client_id}">
<input type="hidden" name="redirect_uri" value="{redirect_uri}">
<input type="hidden" name="code_challenge" value="{code_challenge}">
<input type="hidden" name="code_challenge_method" value="{method}">
<input type="hidden" name="state" value="{state_value}">
<button class="approve" type="submit" name="decision" value="approve">Approve</button>
<button class="deny" type="submit" name="decision" value="deny">Deny</button>
</form></div></body></html>"#,
        name = oauth_html_escape(&page.client_name),
        client_id = oauth_html_escape(&q.client_id),
        redirect_uri = oauth_html_escape(&q.redirect_uri),
        code_challenge = oauth_html_escape(&q.code_challenge),
        method = oauth_html_escape(&page.method),
        state_value = oauth_html_escape(&q.state),
    ))
    .into_response()
}

pub async fn ai_oauth_authorize_post(
    AxumState(state): AxumState<SupervisorState>,
    Form(form): Form<AuthorizeForm>,
) -> Response {
    state
        .mcp
        .oauth
        .lock()
        .expect("oauth mutex should not be poisoned")
        .authorize_decision_response(form)
}

pub async fn ai_oauth_token(
    AxumState(state): AxumState<SupervisorState>,
    Form(form): Form<TokenForm>,
) -> Response {
    state
        .mcp
        .oauth
        .lock()
        .expect("oauth mutex should not be poisoned")
        .token_response(form)
}

pub fn require_ai_mcp_auth(state: &SupervisorState, headers: &HeaderMap) -> Option<Response> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    match token {
        Some(token)
            if state
                .mcp
                .oauth
                .lock()
                .expect("oauth mutex should not be poisoned")
                .access_token_valid(token) =>
        {
            None
        }
        Some(_) => Some(ai_mcp_auth_error_response(state, "invalid_token")),
        None => Some(ai_mcp_auth_error_response(state, "unauthorized")),
    }
}

fn ai_mcp_auth_error_response(state: &SupervisorState, error: &str) -> Response {
    oauth_auth_error_response(&state.mcp.base_url, error)
}
