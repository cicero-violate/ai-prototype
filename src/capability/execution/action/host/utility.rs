//! Utility native tooling capability.

use chrono::Utc;
use serde_json::{json, Value};

use super::ActionToolOutcome;

pub fn execute(name: &str, args: &Value) -> ActionToolOutcome {
    ActionToolOutcome::Ok(match name {
        "echo" => json!({
            "content": [{ "type": "text", "text": args.get("text").and_then(Value::as_str).unwrap_or("") }],
            "isError": false
        }),
        "get_current_time" => json!({
            "content": [{ "type": "text", "text": Utc::now().to_rfc3339() }],
            "isError": false
        }),
        _ => {
            return ActionToolOutcome::Error(crate::api::action::tool_error(format!(
                "Unknown utility tool: {name}"
            )))
        }
    })
}
