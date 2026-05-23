//! Action tool schema list.

use serde_json::Value;

use crate::capability::execution::action::landmarks::gateway_mcp_tools_list;

pub fn action_schema_list() -> Value {
    gateway_mcp_tools_list()
}

// Legacy MCP name alias.
pub use self::action_schema_list as ai_mcp_tools_list;
