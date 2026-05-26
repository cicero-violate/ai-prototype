//! Utility native tooling capability.

use chrono::Utc;
use serde_json::{json, Value};

use super::ToolOutcome;

pub fn execute(name: &str, args: &Value) -> ToolOutcome {
    ToolOutcome::Ok(match name {
        "echo" => json!({
            "content": [{ "type": "text", "text": args.get("text").and_then(Value::as_str).unwrap_or("") }],
            "isError": false
        }),
        "get_current_time" => json!({
            "content": [{ "type": "text", "text": Utc::now().to_rfc3339() }],
            "isError": false
        }),
        _ => {
            return ToolOutcome::Error(crate::api::action::tool_error(format!(
                "Unknown utility tool: {name}"
            )))
        }
    })
}
