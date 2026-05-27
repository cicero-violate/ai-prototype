//! OAuth store, client registration, authorization, and token issuance.

use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

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
            let metadata = fs::metadata(&store.path)
                .map_err(|e| format!("stat oauth store {}: {e}", store.path.display()))?;
            if metadata.len() == 0 {
                return Ok(store);
            }
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

    #[expect(
        clippy::result_large_err,
        reason = "OAuth route helper returns concrete axum Response errors for direct into_response use"
    )]
    pub fn authorize_page_context(
        &mut self,
        q: &AuthorizeQuery,
    ) -> Result<AuthorizePageContext, Response> {
        let client = match self.clients.get(&q.client_id).cloned() {
            Some(client) => client,
            None if self.recover_chatgpt_client(
                &q.client_id,
                &q.redirect_uri,
                &q.code_challenge,
            ) =>
            {
                self.clients
                    .get(&q.client_id)
                    .cloned()
                    .expect("recovered client should be present")
            }
            None => return Err((StatusCode::BAD_REQUEST, "Unknown client_id").into_response()),
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
        let client = match self.clients.get(&form.client_id).cloned() {
            Some(client) => client,
            None if self.recover_chatgpt_client(
                &form.client_id,
                &form.redirect_uri,
                &form.code_challenge,
            ) =>
            {
                self.clients
                    .get(&form.client_id)
                    .cloned()
                    .expect("recovered client should be present")
            }
            None => return (StatusCode::BAD_REQUEST, "Unknown client_id").into_response(),
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
        write_oauth_store_atomic(&self.path, blob.as_bytes())
    }

    fn recover_chatgpt_client(
        &mut self,
        client_id: &str,
        redirect_uri: &str,
        code_challenge: &str,
    ) -> bool {
        if Uuid::parse_str(client_id).is_err()
            || code_challenge.is_empty()
            || !redirect_uri.starts_with("https://chatgpt.com/connector/oauth/")
        {
            return false;
        }

        self.clients.insert(
            client_id.to_string(),
            ClientRecord {
                redirect_uris: vec![redirect_uri.to_string()],
                client_name: Some("ChatGPT".to_string()),
            },
        );
        if let Err(error) = self.save() {
            self.clients.remove(client_id);
            eprintln!("supervisor: ai mcp oauth store save failed after client recovery: {error}");
            return false;
        }
        eprintln!("supervisor: recovered ChatGPT OAuth client_id after missing local store");
        true
    }

    #[cfg(test)]
    fn debug_counts(&self) -> OAuthStoreDebugCounts {
        OAuthStoreDebugCounts {
            clients: self.clients.len(),
            access_tokens: self.access_tokens.len(),
            refresh_tokens: self.refresh_tokens.len(),
        }
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

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OAuthStoreDebugCounts {
    clients: usize,
    access_tokens: usize,
    refresh_tokens: usize,
}

fn write_oauth_store_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let tmp_path = oauth_store_tmp_path(path);
    {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&tmp_path)
            .map_err(|e| format!("open oauth store temp {}: {e}", tmp_path.display()))?;
        file.write_all(bytes)
            .map_err(|e| format!("write oauth store temp {}: {e}", tmp_path.display()))?;
        file.sync_all()
            .map_err(|e| format!("sync oauth store temp {}: {e}", tmp_path.display()))?;
    }

    fs::rename(&tmp_path, path).map_err(|e| {
        format!(
            "rename oauth store temp {} -> {}: {e}",
            tmp_path.display(),
            path.display()
        )
    })?;
    sync_oauth_store_parent(path)
}

fn oauth_store_tmp_path(path: &Path) -> PathBuf {
    let mut tmp = path.to_path_buf();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .map_or_else(|| "tmp".to_string(), |value| format!("{value}.tmp"));
    tmp.set_extension(extension);
    tmp
}

fn sync_oauth_store_parent(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    let dir = fs::File::open(parent)
        .map_err(|e| format!("open oauth store dir {}: {e}", parent.display()))?;
    dir.sync_all()
        .map_err(|e| format!("sync oauth store dir {}: {e}", parent.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_empty_oauth_store_is_empty_without_error() {
        let tmp = tempfile::Builder::new()
            .prefix("oauth-store-empty-")
            .tempfile()
            .expect("temp oauth store should be created");

        let store = OAuthStore::load(tmp.path().to_path_buf(), "test-key".to_string())
            .expect("empty oauth store should load");

        assert_eq!(
            store.debug_counts(),
            OAuthStoreDebugCounts {
                clients: 0,
                access_tokens: 0,
                refresh_tokens: 0,
            }
        );
    }

    #[test]
    fn atomic_oauth_store_write_replaces_existing_file_with_nonempty_blob() {
        let dir = tempfile::Builder::new()
            .prefix("oauth-store-atomic-")
            .tempdir()
            .expect("tempdir should be created");
        let path = dir.path().join("oauth-store.enc");
        fs::write(&path, "old-store").expect("fixture store should be written");

        write_oauth_store_atomic(&path, b"new-store").expect("atomic write should succeed");

        assert_eq!(fs::read_to_string(&path).unwrap(), "new-store");
        assert!(!oauth_store_tmp_path(&path).exists());
    }

    #[test]
    fn authorize_recovers_chatgpt_client_after_missing_store() {
        let dir = tempfile::Builder::new()
            .prefix("oauth-store-recover-chatgpt-")
            .tempdir()
            .expect("tempdir should be created");
        let path = dir.path().join("oauth-store.enc");
        let key = "test-key".to_string();
        let client_id = Uuid::new_v4().to_string();
        let redirect_uri = "https://chatgpt.com/connector/oauth/test-connector".to_string();
        let query = AuthorizeQuery {
            client_id: client_id.clone(),
            redirect_uri: redirect_uri.clone(),
            code_challenge: "challenge".to_string(),
            code_challenge_method: "S256".to_string(),
            state: "state".to_string(),
        };

        let mut store = OAuthStore::load(path.clone(), key.clone()).unwrap();
        let page = store
            .authorize_page_context(&query)
            .expect("ChatGPT client should recover");

        assert_eq!(page.client_name, "ChatGPT");
        assert!(path.exists());

        let reloaded = OAuthStore::load(path, key).unwrap();
        assert!(reloaded.clients.contains_key(&client_id));
    }
}
