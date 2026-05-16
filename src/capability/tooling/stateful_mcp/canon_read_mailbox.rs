//! `canon_read_mailbox` MCP tool.
//!
//! Reads messages from an agent mailbox in the workspace.

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde_json::{json, Value};

use super::canon_send_agent_message::mailbox_path;
use super::common::tool_error;

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
    match read_agent_mailbox(workspace_root, agent_id, since) {
        Ok(result) => {
            let text = serde_json::to_string_pretty(&result).unwrap_or_default();
            json!({ "content": [{ "type": "text", "text": text }], "isError": false })
        }
        Err(error) => tool_error(error),
    }
}

fn read_agent_mailbox(
    workspace_root: &Path,
    agent_id: &str,
    since: usize,
) -> Result<Value, String> {
    let path = mailbox_path(workspace_root, agent_id)?;
    if !path.exists() {
        return Ok(json!({ "messages": [], "next_cursor": 0 }));
    }
    let file = fs::File::open(&path).map_err(|error| format!("open mailbox: {error}"))?;
    let reader = BufReader::new(file);
    let mut messages = Vec::new();
    let mut total_lines = 0_usize;
    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|error| format!("read mailbox line {idx}: {error}"))?;
        total_lines = idx + 1;
        if idx < since || line.trim().is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(&line) {
            messages.push(value);
        }
    }
    Ok(json!({ "messages": messages, "next_cursor": total_lines }))
}
