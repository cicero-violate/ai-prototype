//! Progressive landmark discovery for the native AI MCP tools.
//!
//! This module keeps MCP as the transport boundary, but replaces the public
//! flat tool surface with a small gateway surface:
//! `get_manifest`, `get_landmarks`, `inspect_landmark`, `call_action`, and
//! `execute_sequence`.

use serde_json::{json, Map, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LandmarkAction {
    pub id: &'static str,
    pub native_tool: &'static str,
    pub landmark: &'static str,
    pub description: &'static str,
    pub read_only: bool,
    pub destructive: bool,
    pub idempotent: bool,
}

pub const GATEWAY_GET_MANIFEST: &str = "get_manifest";
pub const GATEWAY_GET_LANDMARKS: &str = "get_landmarks";
pub const GATEWAY_INSPECT_LANDMARK: &str = "inspect_landmark";
pub const GATEWAY_CALL_ACTION: &str = "call_action";
pub const GATEWAY_EXECUTE_SEQUENCE: &str = "execute_sequence";

const ACTIONS: &[LandmarkAction] = &[
    LandmarkAction {
        id: "workspace:apply_patch",
        native_tool: "apply_patch",
        landmark: "workspace",
        description:
            "Apply or check an apply_patch-format patch under the configured workspace root.",
        read_only: false,
        destructive: false,
        idempotent: false,
    },
    LandmarkAction {
        id: "workspace:shell",
        native_tool: "shell",
        landmark: "workspace",
        description: "Run a bounded shell command under the configured workspace root.",
        read_only: false,
        destructive: true,
        idempotent: false,
    },
    LandmarkAction {
        id: "utility:echo",
        native_tool: "echo",
        landmark: "utility",
        description: "Echo back the provided text.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
    LandmarkAction {
        id: "utility:get_current_time",
        native_tool: "get_current_time",
        landmark: "utility",
        description: "Return the current UTC date and time in ISO 8601 format.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
    LandmarkAction {
        id: "agents:spawn",
        native_tool: "canon_spawn_agent",
        landmark: "agents",
        description: "Spawn a child agent through the AI supervisor.",
        read_only: false,
        destructive: false,
        idempotent: false,
    },
    LandmarkAction {
        id: "agents:send_message",
        native_tool: "canon_send_agent_message",
        landmark: "agents",
        description: "Write a typed message to a named agent mailbox.",
        read_only: false,
        destructive: false,
        idempotent: false,
    },
    LandmarkAction {
        id: "agents:read_mailbox",
        native_tool: "canon_read_mailbox",
        landmark: "agents",
        description: "Read messages from an agent mailbox since a cursor.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
];

pub fn is_gateway_tool(name: &str) -> bool {
    matches!(
        name,
        GATEWAY_GET_MANIFEST
            | GATEWAY_GET_LANDMARKS
            | GATEWAY_INSPECT_LANDMARK
            | GATEWAY_CALL_ACTION
            | GATEWAY_EXECUTE_SEQUENCE
    )
}

pub fn gateway_mcp_tools_list() -> Value {
    json!([
        {
            "name": GATEWAY_GET_MANIFEST,
            "description": "Get Canon AI landmark protocol instructions and high-level topology.",
            "inputSchema": {
                "type": "object",
                "properties": { "intent": { "type": "string" } }
            },
            "annotations": { "readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false }
        },
        {
            "name": GATEWAY_GET_LANDMARKS,
            "description": "Return high-level functional areas. Use before inspect_landmark.",
            "inputSchema": {
                "type": "object",
                "properties": { "intent": { "type": "string" } }
            },
            "annotations": { "readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false }
        },
        {
            "name": GATEWAY_INSPECT_LANDMARK,
            "description": "Return exact action IDs and schemas for a landmark or action. Use before call_action.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "landmark_id": {
                        "oneOf": [
                            { "type": "string" },
                            { "type": "array", "items": { "type": "string" } }
                        ]
                    },
                    "intent": { "type": "string" }
                },
                "required": ["landmark_id"]
            },
            "annotations": { "readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false }
        },
        {
            "name": GATEWAY_CALL_ACTION,
            "description": "Execute one inspected action by action ID. Parameters go under the 'parameters' object.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": { "type": "string", "description": "Action ID, e.g. 'workspace:shell'." },
                    "parameters": { "type": "object", "description": "Arguments matching inspect_landmark output.", "default": {} },
                    "intent": { "type": "string" }
                },
                "required": ["action"]
            },
            "annotations": { "readOnlyHint": false, "destructiveHint": true, "idempotentHint": false, "openWorldHint": false }
        },
        {
            "name": GATEWAY_EXECUTE_SEQUENCE,
            "description": "Execute multiple inspected actions with local $stepN or custom alias piping.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "actions": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "action": { "type": "string" },
                                "alias": { "type": "string" },
                                "parameters": { "type": "object", "default": {} },
                                "on_error": { "type": "string", "enum": ["stop", "continue"], "default": "stop" }
                            },
                            "required": ["action"]
                        }
                    },
                    "steps": {
                        "type": "array",
                        "description": "Alias for actions.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "action": { "type": "string" },
                                "alias": { "type": "string" },
                                "parameters": { "type": "object", "default": {} },
                                "on_error": { "type": "string", "enum": ["stop", "continue"], "default": "stop" }
                            },
                            "required": ["action"]
                        }
                    },
                    "intent": { "type": "string" }
                }
            },
            "annotations": { "readOnlyHint": false, "destructiveHint": true, "idempotentHint": false, "openWorldHint": false }
        }
    ])
}

pub fn manifest_result() -> Value {
    text_result(manifest_text())
}

pub fn landmarks_result() -> Value {
    let text = r#"### LANDMARK TOPOLOGY
- **workspace**: File and process operations inside the configured workspace.
- **utility**: Safe utility actions.
- **agents**: Supervisor and mailbox actions for multi-agent operation.

Protocol:
1. Use `get_landmarks` to choose an area.
2. Use `inspect_landmark` for exact action IDs and schemas.
3. Use `call_action` for one action or `execute_sequence` for batched actions.
4. Do not guess parameters; inspect first.
"#;
    text_result(text.trim().to_string())
}

pub fn inspect_landmark_result(args: &Value) -> Value {
    let Some(target) = args.get("landmark_id") else {
        return error_result("inspect_landmark requires 'landmark_id'".to_string());
    };
    let inspected = if let Some(id) = target.as_str() {
        inspect_one(id)
    } else if let Some(ids) = target.as_array() {
        let items: Vec<Value> = ids
            .iter()
            .map(|item| {
                item.as_str()
                    .map(inspect_one)
                    .unwrap_or_else(|| json!({"status": "error", "message": "landmark_id array items must be strings"}))
            })
            .collect();
        Value::Array(items)
    } else {
        json!({"status": "error", "message": "landmark_id must be a string or array of strings"})
    };
    json_result(inspected)
}

pub fn resolve_action_id(action_id: &str) -> Option<LandmarkAction> {
    let normalized = match action_id {
        "apply_patch" => "workspace:apply_patch",
        "shell" => "workspace:shell",
        "echo" => "utility:echo",
        "get_current_time" => "utility:get_current_time",
        "canon_spawn_agent" => "agents:spawn",
        "canon_send_agent_message" => "agents:send_message",
        "canon_read_mailbox" => "agents:read_mailbox",
        value => value,
    };
    ACTIONS
        .iter()
        .copied()
        .find(|action| action.id == normalized)
}

pub fn manifest_text() -> String {
    r#"# CANON AI LANDMARK TOOL PROTOCOL

This MCP server uses progressive landmark discovery.

Available gateway tools:
- `get_manifest`: protocol rules.
- `get_landmarks`: high-level areas only.
- `inspect_landmark`: exact action IDs and schemas.
- `call_action`: one inspected action.
- `execute_sequence`: multiple inspected actions with `$stepN.path` piping.

Rules:
1. Discover before execution.
2. Inspect the relevant landmark before calling an action.
3. Call actions by stable landmark IDs, not by guessed native tool names.
4. For multi-step work, prefer `execute_sequence` and pipe values via `$step0`, `$step1`, or custom aliases.
5. Effects remain authorized and receipted by the AI kernel.
"#
    .trim()
    .to_string()
}

fn inspect_one(id: &str) -> Value {
    match id {
        "workspace" | "utility" | "agents" => {
            let actions: Vec<Value> = ACTIONS
                .iter()
                .copied()
                .filter(|action| action.landmark == id)
                .map(action_summary)
                .collect();
            json!({
                "landmark_id": id,
                "type": "landmark",
                "actions": actions,
                "remedy": "Inspect a specific action ID for its exact input schema before calling it."
            })
        }
        action_id => match resolve_action_id(action_id) {
            Some(action) => action_detail(action),
            None => json!({
                "status": "error",
                "message": format!("Unknown landmark or action: {action_id}"),
                "known_landmarks": ["workspace", "utility", "agents"]
            }),
        },
    }
}

fn action_summary(action: LandmarkAction) -> Value {
    json!({
        "id": action.id,
        "description": action.description,
        "read_only": action.read_only,
        "destructive": action.destructive
    })
}

fn action_detail(action: LandmarkAction) -> Value {
    json!({
        "id": action.id,
        "native_tool": action.native_tool,
        "landmark": action.landmark,
        "description": action.description,
        "inputSchema": native_input_schema(action.native_tool),
        "annotations": annotations(action),
        "aliases": [action.native_tool],
        "call": {
            "tool": GATEWAY_CALL_ACTION,
            "arguments": {
                "action": action.id,
                "parameters": "<object matching inputSchema>"
            }
        }
    })
}

fn annotations(action: LandmarkAction) -> Value {
    json!({
        "readOnlyHint": action.read_only,
        "destructiveHint": action.destructive,
        "idempotentHint": action.idempotent,
        "openWorldHint": false
    })
}

fn native_input_schema(native_tool: &str) -> Value {
    match native_tool {
        "apply_patch" => json!({
            "type": "object",
            "properties": {
                "patch": { "type": "string", "description": "apply_patch-style patch text." },
                "cwd": { "type": "string", "description": "Workspace-root-relative working directory.", "default": "." },
                "mode": { "type": "string", "enum": ["check", "apply"], "default": "check" },
                "strip": { "type": "integer", "minimum": 0, "maximum": 0, "default": 0 },
                "maxBytes": { "type": "integer", "default": 200000 },
                "intent": { "type": "string" }
            },
            "required": ["patch"]
        }),
        "shell" => json!({
            "type": "object",
            "properties": {
                "command": { "type": "string" },
                "cwd": { "type": "string", "default": "." },
                "timeout_ms": { "type": "integer", "default": 180000 },
                "max_output_bytes": { "type": "integer", "default": 65536 },
                "intent": { "type": "string" }
            },
            "required": ["command"]
        }),
        "echo" => json!({
            "type": "object",
            "properties": {
                "text": { "type": "string" },
                "intent": { "type": "string" }
            },
            "required": ["text"]
        }),
        "get_current_time" => json!({
            "type": "object",
            "properties": { "intent": { "type": "string" } }
        }),
        "canon_spawn_agent" => json!({
            "type": "object",
            "properties": {
                "domain": { "type": "string", "description": "Domain hint for the child agent's objective." },
                "metric": { "type": "string", "description": "Success metric the child agent must satisfy." },
                "max_steps": { "type": "integer", "description": "Maximum cycle steps for the child agent.", "default": 20 },
                "intent": { "type": "string" }
            },
            "required": ["domain", "metric"]
        }),
        "canon_send_agent_message" => json!({
            "type": "object",
            "properties": {
                "sender": { "type": "string", "description": "Identifier for the sender." },
                "target_agent": { "type": "string", "description": "Identifier of the receiving agent." },
                "message_kind": { "type": "string", "enum": ["DomainSignal", "GraphEditRequest", "EvalRequest", "PolicyCandidate", "Observation", "TaskAssignment"] },
                "payload": { "type": "string", "description": "JSON payload matching the message kind schema." },
                "intent": { "type": "string" }
            },
            "required": ["sender", "target_agent", "message_kind", "payload"]
        }),
        "canon_read_mailbox" => json!({
            "type": "object",
            "properties": {
                "agent_id": { "type": "string", "description": "This agent's identifier." },
                "since": { "type": "integer", "description": "Cursor from the previous read.", "default": 0 },
                "intent": { "type": "string" }
            },
            "required": ["agent_id"]
        }),
        _ => json!({ "type": "object", "properties": {} }),
    }
}

pub fn parameters_from_call_action(args: &Value) -> Result<(&str, Value), String> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "call_action requires non-empty 'action'".to_string())?;
    let parameters = args.get("parameters").cloned().unwrap_or_else(|| json!({}));
    if !parameters.is_object() {
        return Err("call_action 'parameters' must be an object".to_string());
    }
    Ok((action, parameters))
}

#[derive(Clone, Debug)]
pub struct SequenceStep {
    pub action: String,
    pub alias: Option<String>,
    pub parameters: Value,
    pub on_error: String,
}

pub fn sequence_steps(args: &Value) -> Result<Vec<SequenceStep>, String> {
    let steps_value = args
        .get("actions")
        .or_else(|| args.get("steps"))
        .ok_or_else(|| "execute_sequence requires 'actions' or 'steps'".to_string())?;
    let Some(items) = steps_value.as_array() else {
        return Err("execute_sequence actions/steps must be an array".to_string());
    };
    if items.is_empty() {
        return Err("execute_sequence requires at least one step".to_string());
    }
    items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let action = item
                .get("action")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| format!("step {idx} requires non-empty 'action'"))?
                .to_string();
            let alias = item
                .get("alias")
                .and_then(Value::as_str)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string);
            let parameters = item.get("parameters").cloned().unwrap_or_else(|| json!({}));
            if !parameters.is_object() {
                return Err(format!("step {idx} parameters must be an object"));
            }
            let on_error = item
                .get("on_error")
                .and_then(Value::as_str)
                .unwrap_or("stop");
            if on_error != "stop" && on_error != "continue" {
                return Err(format!("step {idx} on_error must be 'stop' or 'continue'"));
            }
            Ok(SequenceStep {
                action,
                alias,
                parameters,
                on_error: on_error.to_string(),
            })
        })
        .collect()
}

pub fn resolve_piping(value: Value, aliases: &Map<String, Value>) -> Result<Value, String> {
    match value {
        Value::String(text) if text.starts_with('$') => resolve_reference(&text, aliases),
        Value::Array(items) => items
            .into_iter()
            .map(|item| resolve_piping(item, aliases))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Value::Object(map) => map
            .into_iter()
            .map(|(key, value)| resolve_piping(value, aliases).map(|resolved| (key, resolved)))
            .collect::<Result<Map<String, Value>, _>>()
            .map(Value::Object),
        other => Ok(other),
    }
}

fn resolve_reference(reference: &str, aliases: &Map<String, Value>) -> Result<Value, String> {
    let body = reference.trim_start_matches('$');
    let (alias, path) = match body.find('.') {
        Some(index) => (&body[..index], Some(&body[index + 1..])),
        None => (body, None),
    };
    if alias.is_empty() {
        return Err("empty alias reference".to_string());
    }
    let mut value = aliases
        .get(alias)
        .cloned()
        .ok_or_else(|| format!("unknown alias '${alias}'"))?;
    if let Some(path) = path {
        for part in path.split('.') {
            if part.is_empty() {
                return Err(format!("invalid empty path segment in '{reference}'"));
            }
            value = descend(value, part)
                .ok_or_else(|| format!("path '{part}' not found while resolving '{reference}'"))?;
        }
    }
    Ok(value)
}

fn descend(value: Value, part: &str) -> Option<Value> {
    if let Ok(index) = part.parse::<usize>() {
        return value.as_array().and_then(|items| items.get(index)).cloned();
    }
    value.as_object().and_then(|map| map.get(part)).cloned()
}

pub fn result_is_error(value: &Value) -> bool {
    value
        .get("isError")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || value
            .get("status")
            .and_then(Value::as_str)
            .map(|status| status == "error")
            .unwrap_or(false)
}

pub fn text_result(text: String) -> Value {
    json!({ "content": [{ "type": "text", "text": text }], "isError": false })
}

pub fn json_result(value: Value) -> Value {
    let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string());
    text_result(text)
}

pub fn error_result(message: String) -> Value {
    json!({ "content": [{ "type": "text", "text": format!("Error: {message}") }], "isError": true })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content_text(result: &Value) -> String {
        result["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn gateway_tools_list_exposes_only_progressive_discovery_surface() {
        let tools = gateway_mcp_tools_list();
        let names: Vec<&str> = tools
            .as_array()
            .expect("tools list must be an array")
            .iter()
            .map(|tool| tool["name"].as_str().expect("tool name"))
            .collect();

        assert_eq!(
            names,
            vec![
                GATEWAY_GET_MANIFEST,
                GATEWAY_GET_LANDMARKS,
                GATEWAY_INSPECT_LANDMARK,
                GATEWAY_CALL_ACTION,
                GATEWAY_EXECUTE_SEQUENCE,
            ]
        );
        assert!(!names.contains(&"shell"));
        assert!(!names.contains(&"apply_patch"));
    }

    #[test]
    fn manifest_and_landmarks_describe_discovery_before_execution() {
        let manifest = content_text(&manifest_result());
        assert!(manifest.contains("progressive landmark discovery"));
        assert!(manifest.contains("inspect_landmark"));
        assert!(manifest.contains("call_action"));

        let landmarks = content_text(&landmarks_result());
        assert!(landmarks.contains("workspace"));
        assert!(landmarks.contains("utility"));
        assert!(landmarks.contains("agents"));
    }

    #[test]
    fn inspect_landmark_returns_action_ids_and_exact_schema() {
        let result = inspect_landmark_result(&json!({"landmark_id": "workspace"}));
        let text = content_text(&result);
        assert!(text.contains("workspace:apply_patch"));
        assert!(text.contains("workspace:shell"));

        let action = inspect_landmark_result(&json!({"landmark_id": "workspace:shell"}));
        let action_text = content_text(&action);
        assert!(action_text.contains("\"id\": \"workspace:shell\""));
        assert!(action_text.contains("\"command\""));
        assert!(action_text.contains("\"required\": ["));
    }

    #[test]
    fn call_action_accepts_landmark_ids_and_legacy_native_aliases() {
        let call = json!({
            "action": "workspace:shell",
            "parameters": {"command": "pwd"}
        });
        let (action, params) = parameters_from_call_action(&call).expect("valid call_action");
        assert_eq!(action, "workspace:shell");
        assert_eq!(params["command"], "pwd");

        assert_eq!(
            resolve_action_id("workspace:shell").unwrap().native_tool,
            "shell"
        );
        assert_eq!(resolve_action_id("shell").unwrap().id, "workspace:shell");
        assert!(parameters_from_call_action(
            &json!({"action": "workspace:shell", "parameters": []})
        )
        .is_err());
    }

    #[test]
    fn execute_sequence_parser_and_piping_are_deterministic() {
        let steps = sequence_steps(&json!({
            "actions": [
                {"action": "utility:echo", "alias": "first", "parameters": {"text": "hello"}},
                {"action": "utility:echo", "parameters": {"text": "$first.content.0.text"}}
            ]
        }))
        .expect("valid sequence");
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].alias.as_deref(), Some("first"));
        assert_eq!(steps[1].on_error, "stop");

        let mut aliases = Map::new();
        aliases.insert(
            "first".to_string(),
            json!({"content": [{"type": "text", "text": "hello"}], "isError": false}),
        );
        let resolved = resolve_piping(json!({"text": "$first.content.0.text"}), &aliases)
            .expect("piping resolves");
        assert_eq!(resolved["text"], "hello");
        assert!(resolve_piping(json!("$missing.value"), &aliases).is_err());
    }
}
