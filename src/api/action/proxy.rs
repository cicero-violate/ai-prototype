//! Action command proxy compatibility facade.

pub use crate::api::mcp::proxy::*;
pub use crate::api::mcp::proxy::{
    kernel_command_payload as action_kernel_command_payload,
    kernel_command_payload_tag as action_kernel_command_payload_tag,
    submit_mcp_kernel_command as submit_action_kernel_command,
};
