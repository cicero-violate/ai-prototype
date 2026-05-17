//! `canon_send_agent_message` MCP tool.
//!
//! Requests a runtime-owned mailbox projection append. The dispatcher records
//! semantic authorization and receipt commands around the append.

use std::path::Path;

use serde_json::{json, Value};

use super::common::tool_error;
use crate::runtime::{append_mailbox_message, MailboxMessageReceipt, MailboxMessageRequest};

pub const CANON_SEND_AGENT_MESSAGE_TOOL: &str = "canon_send_agent_message";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SendAgentMessageArgs {
    pub sender: String,
    pub target: String,
    pub kind: String,
    pub payload: String,
    pub request: MailboxMessageRequest,
}

pub fn parse_args(args: &Value) -> Result<SendAgentMessageArgs, String> {
    let sender = args
        .get("sender")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let target = args
        .get("target_agent")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "target_agent is required".to_string())?
        .to_string();
    let kind = args
        .get("message_kind")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "message_kind is required".to_string())?
        .to_string();
    let payload = args
        .get("payload")
        .and_then(Value::as_str)
        .unwrap_or("{}")
        .to_string();
    let request = MailboxMessageRequest::new(&sender, &target, &kind, &payload)?;
    Ok(SendAgentMessageArgs {
        sender,
        target,
        kind,
        payload,
        request,
    })
}

pub fn append_authorized(
    parsed: &SendAgentMessageArgs,
    workspace_root: &Path,
) -> Result<(MailboxMessageReceipt, Value), String> {
    let record = append_mailbox_message(
        workspace_root,
        &parsed.sender,
        &parsed.target,
        &parsed.kind,
        &parsed.payload,
    )?;
    let receipt = MailboxMessageReceipt::from_record(&parsed.request, &record);
    let text = serde_json::to_string_pretty(&record).unwrap_or_default();
    Ok((
        receipt,
        json!({ "content": [{ "type": "text", "text": text }], "isError": false }),
    ))
}

pub fn run(args: &Value, workspace_root: &Path) -> Value {
    let parsed = match parse_args(args) {
        Ok(parsed) => parsed,
        Err(error) => return tool_error(error),
    };
    match append_authorized(&parsed, workspace_root) {
        Ok((_receipt, result)) => result,
        Err(error) => tool_error(error),
    }
}
