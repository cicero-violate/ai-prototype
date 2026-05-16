//! OAuth support for the supervisor MCP API surface.
//!
//! Store, request DTOs, metadata helpers, and crypto helpers are split by
//! responsibility. Axum route wiring remains in `api::routes::supervisor`.

mod crypto;
mod metadata;
mod store;
mod types;

pub use metadata::{
    auth_error_response, html_escape, metadata_value, protected_resource_metadata_value,
};
pub use store::OAuthStore;
pub use types::{AuthorizeForm, AuthorizePageContext, AuthorizeQuery, RegisterBody, TokenForm};
