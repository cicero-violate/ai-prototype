//! MCP tool adapters.
//!
//! Each file in this module maps to one MCP tool adapter. Durable state and
//! process lifecycle authority live in runtime/process modules. Pure MCP
//! protocol JSON helpers remain in `api::mcp`.

pub mod apply_patch;
pub mod canon_graph_editor;
pub mod canon_read_mailbox;
pub mod canon_send_agent_message;
pub mod canon_spawn_agent;
pub mod common;
pub mod dispatch;
pub mod landmarks;
pub mod shell;

pub use apply_patch::APPLY_PATCH_TOOL;
pub use canon_graph_editor::{
    CANON_GRAPH_APPLY_OPS_TOOL, CANON_GRAPH_AUTO_REFACTOR_CFG_TOOL, CANON_GRAPH_PLAN_CFG_TOOL,
    CANON_GRAPH_PLAN_PATCH_TOOL, CANON_GRAPH_VERIFY_CFG_DELTA_TOOL,
};
pub use canon_read_mailbox::CANON_READ_MAILBOX_TOOL;
pub use canon_send_agent_message::CANON_SEND_AGENT_MESSAGE_TOOL;
pub use canon_spawn_agent::{SpawnAgentToolRequest, CANON_SPAWN_AGENT_TOOL};
pub use shell::SHELL_TOOL;

pub use dispatch::McpToolHost;
