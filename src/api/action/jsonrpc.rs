//! JSON-RPC action wire helpers.

use serde_json::{json, Value};

fn action_response(id: Value, field: &'static str, payload: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, field: payload})
}

pub fn action_ok(id: Value, result: Value) -> Value {
    action_response(id, "result", result)
}

pub fn action_err(id: Value, code: i64, message: &str) -> Value {
    action_response(id, "error", json!({ "code": code, "message": message }))
}

// Legacy MCP name aliases.
pub use self::action_ok as mcp_ok;
pub use self::action_err as mcp_err;

pub fn tool_error(message: String) -> Value {
    json!({ "content": [{ "type": "text", "text": format!("Error: {message}") }], "isError": true })
}

pub fn result_with_warning(mut result: Value, warning: String) -> Value {
    match result.get_mut("content").and_then(Value::as_array_mut) {
        Some(content) => content.push(json!({ "type": "text", "text": warning })),
        None => result["content"] = json!([{ "type": "text", "text": warning }]),
    }
    result["receipt_warning"] = Value::String(warning);
    result
}
