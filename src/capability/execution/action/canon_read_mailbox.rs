//! `canon_read_mailbox` action tool.
//!
//! Reads the runtime-owned mailbox projection.

use std::path::Path;

use serde_json::{json, Value};

use crate::capability::execution::action::common::tool_error;
use crate::runtime::read_mailbox_projection;

pub const CANON_READ_MAILBOX_TOOL: &str = "canon_read_mailbox";

pub fn run(args: &Value, workspace_root: &Path) -> Value {
    let agent_id = match args
        .get("agent_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    {
        Some(agent_id) => agent_id,
        None => return tool_error("agent_id is required".to_string()),
    };
    let since = args.get("since").and_then(Value::as_u64).unwrap_or(0) as usize;
    match read_mailbox_projection(workspace_root, agent_id, since) {
        Ok(result) => {
            let text = serde_json::to_string_pretty(&result).unwrap_or_default();
            json!({ "content": [{ "type": "text", "text": text }], "isError": false })
        }
        Err(error) => tool_error(error),
    }
}
