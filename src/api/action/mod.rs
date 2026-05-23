//! Protocol-neutral action API surface.
//!
//! This module is the migration target for the current MCP-named API edge.
//! During the migration window it re-exports the existing implementation so new
//! code can depend on `api::action` while the external compatibility route keeps
//! serving existing clients.

pub mod dispatch;
pub mod jsonrpc;
pub mod proxy;
pub mod schema;

pub use crate::api::mcp::*;
pub use crate::api::mcp::{
    ai_mcp_tools_list as action_schema_list, dispatch_ai_mcp_plan as dispatch_action_plan,
    kernel_command_payload as action_kernel_command_payload,
    kernel_command_payload_tag as action_kernel_command_payload_tag, mcp_err as action_err,
    mcp_ok as action_ok, AiMcpDispatchPlan as ActionDispatchPlan,
};
