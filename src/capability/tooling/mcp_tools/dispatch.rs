//! Legacy MCP-named action dispatch compatibility facade.
//!
//! The implementation lives under `capability::execution::action::dispatch`.

pub use crate::capability::execution::action::dispatch::{
    dispatch_action_request as dispatch_ai_mcp, ActionHost as McpToolHost,
};
