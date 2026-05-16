//! `shell` MCP tool.
//!
//! Runs bounded shell commands under the configured workspace.

mod recorded;
mod unrecorded;

pub const SHELL_TOOL: &str = "shell";

pub use recorded::{recorded_process_request, render_recorded_response, run_recorded_process};
pub use unrecorded::run_unrecorded;
