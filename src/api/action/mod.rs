//! Protocol-neutral action API surface.

pub mod dispatch;
pub mod jsonrpc;
pub mod proxy;
pub mod schema;

pub use dispatch::{
    dispatch_action_plan, ActionDispatchPlan,
    dispatch_ai_mcp_plan, AiMcpDispatchPlan,
};
pub use jsonrpc::{action_err, action_ok, mcp_err, mcp_ok, result_with_warning, tool_error};
pub use proxy::{
    action_kernel_command_payload, action_kernel_command_payload_tag, submit_action_kernel_command,
    kernel_command_payload, kernel_command_payload_tag, submit_mcp_kernel_command,
};
pub use schema::{action_schema_list, ai_mcp_tools_list};
