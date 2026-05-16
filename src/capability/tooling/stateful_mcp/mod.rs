//! Runtime/file/process/agent MCP tools.
//!
//! Each file in this module maps to one MCP tool that touches process state,
//! filesystem state, agent lifecycle, or mailbox state. Pure MCP protocol JSON
//! helpers remain in `api::mcp`.

pub mod apply_patch;
pub mod canon_read_mailbox;
pub mod canon_send_agent_message;
pub mod canon_spawn_agent;
pub mod common;
pub mod dispatch;
pub mod shell;
pub mod workspace;

pub use apply_patch::APPLY_PATCH_TOOL;
pub use canon_read_mailbox::CANON_READ_MAILBOX_TOOL;
pub use canon_send_agent_message::CANON_SEND_AGENT_MESSAGE_TOOL;
pub use canon_spawn_agent::{SpawnAgentToolRequest, CANON_SPAWN_AGENT_TOOL};
pub use shell::SHELL_TOOL;

pub use dispatch::StatefulMcpHost;
pub use workspace::WorkspaceView;
