//! MCP JSON-RPC response helpers.

use serde_json::{json, Value};

enum McpResponsePayload {
    Result(Value),
    Error { code: i64, message: String },
}

fn mcp_response(id: Value, payload: McpResponsePayload) -> Value {
    match payload {
        McpResponsePayload::Result(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
        McpResponsePayload::Error { code, message } => {
            json!({"jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message }})
        }
    }
}

pub fn mcp_ok(id: Value, result: Value) -> Value {
    mcp_response(id, McpResponsePayload::Result(result))
}

pub fn mcp_err(id: Value, code: i64, message: &str) -> Value {
    mcp_response(
        id,
        McpResponsePayload::Error {
            code,
            message: message.to_owned(),
        },
    )
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
