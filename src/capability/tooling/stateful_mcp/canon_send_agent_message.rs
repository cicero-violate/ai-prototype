//! `canon_send_agent_message` MCP tool.
//!
//! Appends a typed message to an agent mailbox in the workspace.

use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde_json::{json, Value};
use uuid::Uuid;

use super::common::tool_error;

pub const CANON_SEND_AGENT_MESSAGE_TOOL: &str = "canon_send_agent_message";

pub fn run(args: &Value, workspace_root: &Path) -> Value {
    let sender = args
        .get("sender")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let target = match args
        .get("target_agent")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    {
        Some(target) => target,
        None => return tool_error("target_agent is required".to_string()),
    };
    let kind = match args
        .get("message_kind")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    {
        Some(kind) => kind,
        None => return tool_error("message_kind is required".to_string()),
    };
    let payload = args.get("payload").and_then(Value::as_str).unwrap_or("{}");
    match send_agent_message(workspace_root, sender, target, kind, payload) {
        Ok(record) => {
            let text = serde_json::to_string_pretty(&record).unwrap_or_default();
            json!({ "content": [{ "type": "text", "text": text }], "isError": false })
        }
        Err(error) => tool_error(error),
    }
}

pub(crate) fn mailbox_path(workspace_root: &Path, agent_id: &str) -> Result<PathBuf, String> {
    validate_agent_id(agent_id)?;
    Ok(workspace_root
        .join("agent_state")
        .join("mailbox")
        .join(format!("{agent_id}.ndjson")))
}

pub(crate) fn validate_agent_id(agent_id: &str) -> Result<(), String> {
    if agent_id.is_empty()
        || agent_id.contains('/')
        || agent_id.contains('\\')
        || agent_id.contains("..")
    {
        return Err(format!("invalid agent id: {agent_id:?}"));
    }
    Ok(())
}

fn send_agent_message(
    workspace_root: &Path,
    sender: &str,
    target_agent: &str,
    message_kind: &str,
    payload: &str,
) -> Result<Value, String> {
    validate_agent_id(sender)?;
    let path = mailbox_path(workspace_root, target_agent)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create mailbox dir: {error}"))?;
    }
    let record = json!({
        "id": Uuid::new_v4().to_string(),
        "sender": sender,
        "target": target_agent,
        "kind": message_kind,
        "payload": payload,
        "sent_at": Utc::now().to_rfc3339()
    });
    let line =
        serde_json::to_string(&record).map_err(|error| format!("serialize message: {error}"))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| format!("open mailbox {}: {error}", path.display()))?;
    writeln!(file, "{line}").map_err(|error| format!("write mailbox: {error}"))?;
    Ok(record)
}
