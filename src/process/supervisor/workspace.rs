//! Workspace path policy for the supervisor MCP runtime.
//!
//! Re-exports the capability-owned workspace view as the supervisor runtime
//! configuration type.

pub use crate::capability::tooling::stateful_mcp::WorkspaceView as WorkspaceConfig;
