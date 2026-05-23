//! Action-facing supervisor route compatibility facade.
//!
//! The external route may continue serving MCP-compatible clients during the
//! migration, but internal call sites should move to action terminology.

pub use super::mcp::{
    ai_mcp_delete as ai_action_delete, ai_mcp_get_sse as ai_action_get_sse,
    ai_mcp_post as ai_action_post,
};
