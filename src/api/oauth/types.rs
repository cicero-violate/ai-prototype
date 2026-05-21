//! OAuth request/record types.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ClientRecord {
    pub(crate) redirect_uris: Vec<String>,
    pub(crate) client_name: Option<String>,
}

pub(crate) struct AuthCode {
    pub(crate) client_id: String,
    pub(crate) redirect_uri: String,
    pub(crate) code_challenge: String,
    pub(crate) expires_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct TokenRecord {
    pub(crate) client_id: String,
    pub(crate) expires_at: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct RefreshTokenRecord {
    pub(crate) client_id: String,
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct PersistedOAuthStore {
    pub(crate) clients: HashMap<String, ClientRecord>,
    pub(crate) access_tokens: HashMap<String, TokenRecord>,
    pub(crate) refresh_tokens: HashMap<String, RefreshTokenRecord>,
}

#[derive(Deserialize)]
pub struct RegisterBody {
    pub redirect_uris: Vec<String>,
    pub client_name: Option<String>,
}

#[derive(Deserialize)]
pub struct AuthorizeQuery {
    pub client_id: String,
    pub redirect_uri: String,
    pub code_challenge: String,
    #[serde(default)]
    pub code_challenge_method: String,
    #[serde(default)]
    pub state: String,
}

#[derive(Deserialize)]
pub struct AuthorizeForm {
    pub client_id: String,
    pub redirect_uri: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub state: Option<String>,
    pub decision: String,
}

#[derive(Deserialize)]
pub struct TokenForm {
    pub grant_type: String,
    pub code: Option<String>,
    pub code_verifier: Option<String>,
    pub client_id: Option<String>,
    pub redirect_uri: Option<String>,
    pub refresh_token: Option<String>,
}

pub struct AuthorizePageContext {
    pub client_name: String,
    pub method: String,
}
