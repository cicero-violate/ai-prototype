//! OAuth metadata and HTML/auth response helpers.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

pub fn metadata_value(base_url: &str) -> Value {
    let base = base_url.trim_end_matches('/');
    json!({
        "issuer": base,
        "authorization_endpoint": format!("{base}/authorize"),
        "token_endpoint": format!("{base}/token"),
        "registration_endpoint": format!("{base}/register"),
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["none"]
    })
}

pub fn protected_resource_metadata_value(base_url: &str) -> Value {
    let base = base_url.trim_end_matches('/');
    json!({
        "resource": format!("{base}/mcp"),
        "authorization_servers": [base],
        "bearer_methods_supported": ["header"],
        "scopes_supported": []
    })
}

pub fn auth_error_response(base_url: &str, error: &str) -> Response {
    let metadata_url = format!(
        "{}/.well-known/oauth-protected-resource/mcp",
        base_url.trim_end_matches('/')
    );
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .header(
            axum::http::header::WWW_AUTHENTICATE,
            format!(r#"Bearer resource_metadata="{metadata_url}", error="{error}""#),
        )
        .body(json!({ "error": error }).to_string().into())
        .unwrap_or_else(|_| {
            (StatusCode::UNAUTHORIZED, Json(json!({ "error": error }))).into_response()
        })
}

pub fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use axum::http::header::WWW_AUTHENTICATE;

    use super::*;

    #[test]
    fn auth_error_response_uses_parseable_bearer_challenge() {
        let response = auth_error_response("https://example.test/ai", "unauthorized");
        let challenge = response
            .headers()
            .get(WWW_AUTHENTICATE)
            .expect("www-authenticate header should be present")
            .to_str()
            .expect("www-authenticate header should be ascii");

        assert_eq!(
            challenge,
            r#"Bearer resource_metadata="https://example.test/ai/.well-known/oauth-protected-resource/mcp", error="unauthorized""#
        );
        assert!(!challenge.contains("\\\""));
    }
}
