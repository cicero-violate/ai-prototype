//! OAuth store, client registration, authorization, and token issuance.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use super::crypto::{decrypt_blob, encrypt_blob, key_material, verify_pkce};
use super::types::{
    AuthCode, AuthorizeForm, AuthorizePageContext, AuthorizeQuery, ClientRecord,
    PersistedOAuthStore, RefreshTokenRecord, RegisterBody, TokenForm, TokenRecord,
};

pub struct OAuthStore {
    path: PathBuf,
    key_material: Vec<u8>,
    clients: HashMap<String, ClientRecord>,
    auth_codes: HashMap<String, AuthCode>,
    access_tokens: HashMap<String, TokenRecord>,
    refresh_tokens: HashMap<String, RefreshTokenRecord>,
}

impl OAuthStore {
    pub fn load(path: PathBuf, key: String) -> Result<Self, String> {
        let key_material = key_material(&key);
        let mut store = Self {
            path,
            key_material,
            clients: HashMap::new(),
            auth_codes: HashMap::new(),
            access_tokens: HashMap::new(),
            refresh_tokens: HashMap::new(),
        };
        if store.path.exists() {
            let blob = fs::read_to_string(&store.path)
                .map_err(|e| format!("read oauth store {}: {e}", store.path.display()))?;
            let plaintext = decrypt_blob(blob.trim(), &store.key_material)?;
            let persisted: PersistedOAuthStore = serde_json::from_slice(&plaintext)
                .map_err(|e| format!("parse oauth store {}: {e}", store.path.display()))?;
            store.clients = persisted.clients;
            store.access_tokens = persisted.access_tokens;
            store.refresh_tokens = persisted.refresh_tokens;
        }
        Ok(store)
    }

    pub fn access_token_valid(&self, token: &str) -> bool {
        self.access_tokens
            .get(token)
            .is_some_and(|record| record.expires_at > Utc::now())
    }

    pub fn register_client_response(&mut self, body: RegisterBody) -> Response {
        let client_id = Uuid::new_v4().to_string();
        let redirect_uris = body.redirect_uris.clone();
        let client_name = body.client_name.clone();
        self.clients.insert(
            client_id.clone(),
            ClientRecord {
                redirect_uris: body.redirect_uris,
                client_name: body.client_name,
            },
        );
        if let Err(error) = self.save() {
            eprintln!("supervisor: ai mcp oauth store save failed after register: {error}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "server_error"})),
            )
                .into_response();
        }
        Json(json!({
            "client_id": client_id,
            "client_id_issued_at": Utc::now().timestamp(),
            "redirect_uris": redirect_uris,
            "client_name": client_name,
            "grant_types": ["authorization_code", "refresh_token"],
            "response_types": ["code"],
            "token_endpoint_auth_method": "none"
        }))
        .into_response()
    }

    pub fn authorize_page_context(
        &self,
        q: &AuthorizeQuery,
    ) -> Result<AuthorizePageContext, Response> {
        let Some(client) = self.clients.get(&q.client_id).cloned() else {
            return Err((StatusCode::BAD_REQUEST, "Unknown client_id").into_response());
        };
        if !client.redirect_uris.contains(&q.redirect_uri) {
            return Err((StatusCode::BAD_REQUEST, "Invalid redirect_uri").into_response());
        }
        let method = if q.code_challenge_method.is_empty() {
            "S256".to_string()
        } else {
            q.code_challenge_method.clone()
        };
        Ok(AuthorizePageContext {
            client_name: client
                .client_name
                .unwrap_or_else(|| "An application".to_string()),
            method,
        })
    }

    pub fn authorize_decision_response(&mut self, form: AuthorizeForm) -> Response {
        let state_suffix = form
            .state
            .as_deref()
            .filter(|value| !value.is_empty())
            .map_or(String::new(), |value| format!("&state={value}"));
        let Some(client) = self.clients.get(&form.client_id).cloned() else {
            return (StatusCode::BAD_REQUEST, "Unknown client_id").into_response();
        };
        if !client.redirect_uris.contains(&form.redirect_uri) {
            return (StatusCode::BAD_REQUEST, "Invalid redirect_uri").into_response();
        }
        if form.decision != "approve" {
            return Redirect::to(&format!(
                "{}?error=access_denied{}",
                form.redirect_uri, state_suffix
            ))
            .into_response();
        }
        let code = Uuid::new_v4().to_string();
        self.auth_codes.insert(
            code.clone(),
            AuthCode {
                client_id: form.client_id,
                redirect_uri: form.redirect_uri.clone(),
                code_challenge: form.code_challenge,
                expires_at: Utc::now() + chrono::Duration::minutes(5),
            },
        );
        Redirect::to(&format!(
            "{}?code={}{}",
            form.redirect_uri, code, state_suffix
        ))
        .into_response()
    }

    pub fn token_response(&mut self, form: TokenForm) -> Response {
        match form.grant_type.as_str() {
            "authorization_code" => self.issue_authorization_code_token(form),
            "refresh_token" => self.issue_refresh_token(form),
            _ => (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "unsupported_grant_type"})),
            )
                .into_response(),
        }
    }

    fn save(&self) -> Result<(), String> {
        let persisted = PersistedOAuthStore {
            clients: self.clients.clone(),
            access_tokens: self.access_tokens.clone(),
            refresh_tokens: self.refresh_tokens.clone(),
        };
        let plaintext =
            serde_json::to_vec(&persisted).map_err(|e| format!("serialize oauth store: {e}"))?;
        let blob = encrypt_blob(&plaintext, &self.key_material);
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("create oauth store dir {}: {e}", parent.display()))?;
        }
        fs::write(&self.path, blob)
            .map_err(|e| format!("write oauth store {}: {e}", self.path.display()))
    }

    fn prune_expired(&mut self) {
        let now = Utc::now();
        self.auth_codes.retain(|_, code| code.expires_at > now);
        self.access_tokens.retain(|_, token| token.expires_at > now);
    }

    fn issue_authorization_code_token(&mut self, form: TokenForm) -> Response {
        let Some(code) = form.code else {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "missing code"})),
            )
                .into_response();
        };
        let Some(verifier) = form.code_verifier else {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "missing code_verifier"})),
            )
                .into_response();
        };
        self.prune_expired();
        let Some(auth_code) = self.auth_codes.remove(&code) else {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid_grant"})),
            )
                .into_response();
        };
        if form
            .client_id
            .as_deref()
            .is_some_and(|id| id != auth_code.client_id)
            || form
                .redirect_uri
                .as_deref()
                .is_some_and(|uri| uri != auth_code.redirect_uri)
            || !verify_pkce(&verifier, &auth_code.code_challenge)
        {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid_grant"})),
            )
                .into_response();
        }
        self.issue_token_pair(&auth_code.client_id)
    }

    fn issue_refresh_token(&mut self, form: TokenForm) -> Response {
        let Some(supplied_refresh) = form.refresh_token else {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "missing refresh_token"})),
            )
                .into_response();
        };
        self.prune_expired();
        let Some(record) = self.refresh_tokens.remove(&supplied_refresh) else {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid_grant"})),
            )
                .into_response();
        };
        if form
            .client_id
            .as_deref()
            .is_some_and(|id| id != record.client_id)
        {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({"error": "invalid_client"})),
            )
                .into_response();
        }
        self.issue_token_pair(&record.client_id)
    }

    fn issue_token_pair(&mut self, client_id: &str) -> Response {
        let access_token = Uuid::new_v4().to_string();
        let refresh_token = Uuid::new_v4().to_string();
        self.access_tokens.insert(
            access_token.clone(),
            TokenRecord {
                client_id: client_id.to_string(),
                expires_at: Utc::now() + chrono::Duration::minutes(15),
            },
        );
        self.refresh_tokens.insert(
            refresh_token.clone(),
            RefreshTokenRecord {
                client_id: client_id.to_string(),
            },
        );
        if let Err(error) = self.save() {
            eprintln!("supervisor: ai mcp oauth store save failed after token issue: {error}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "server_error"})),
            )
                .into_response();
        }
        Json(json!({
            "access_token": access_token,
            "refresh_token": refresh_token,
            "token_type": "bearer",
            "expires_in": 900
        }))
        .into_response()
    }
}
