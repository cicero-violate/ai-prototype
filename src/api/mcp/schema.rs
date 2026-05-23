//! Static native MCP tool schemas.

use serde_json::Value;

use crate::capability::execution::action::landmarks::gateway_mcp_tools_list;

pub fn ai_mcp_tools_list() -> Value {
    gateway_mcp_tools_list()
}
