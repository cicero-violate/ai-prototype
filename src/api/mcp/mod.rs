//! MCP JSON-RPC helpers, static schema, dispatch planning, and worker proxying.

pub mod dispatch;
pub mod jsonrpc;
pub mod proxy;
pub mod schema;

pub use dispatch::{dispatch_ai_mcp_plan, AiMcpDispatchPlan};
pub use jsonrpc::{mcp_err, mcp_ok, result_with_warning, tool_error};
pub use proxy::{kernel_command_payload, kernel_command_payload_tag, submit_mcp_kernel_command};
pub use schema::ai_mcp_tools_list;
