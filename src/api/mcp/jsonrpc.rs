//! MCP JSON-RPC response helpers.

use serde_json::{json, Value};

fn mcp_response(id: Value, field: &str, payload: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, field: payload})
}

pub fn mcp_ok(id: Value, result: Value) -> Value {
    mcp_response(id, "result", result)
}

pub fn mcp_err(id: Value, code: i64, message: &str) -> Value {
    mcp_response(id, "error", json!({ "code": code, "message": message }))
}

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
