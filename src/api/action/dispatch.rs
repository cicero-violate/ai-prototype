//! Action dispatch planning.

use serde_json::{json, Value};
use uuid::Uuid;

use crate::service::supervisor::WorkspaceConfig;

use super::jsonrpc::{action_err, action_ok};
use super::schema::action_schema_list;

#[derive(Clone, Debug, PartialEq)]
pub enum ActionDispatchPlan {
    Initialize { response: Value, session_id: String },
    Immediate { response: Value },
    ToolCall { name: String, args: Value },
}

// Legacy MCP name alias.
pub use self::ActionDispatchPlan as AiMcpDispatchPlan;

pub fn dispatch_action_plan(
    method: &str,
    id: Value,
    params: Value,
    workspace: Option<&WorkspaceConfig>,
) -> ActionDispatchPlan {
    match method {
        "initialize" => {
            let sid = Uuid::new_v4().to_string();
            ActionDispatchPlan::Initialize {
                session_id: sid,
                response: action_ok(
                    id,
                    json!({
                        "protocolVersion": "2025-03-26",
                        "capabilities": {
                            "tools": { "listChanged": false },
                            "resources": { "listChanged": false },
                            "prompts": {}
                        },
                        "serverInfo": { "name": "canon-ai-mcp", "version": env!("CARGO_PKG_VERSION") },
                        "instructions": "Canon AI native MCP. Tools execute through the AI supervisor and submit event-sourced tooling receipts."
                    }),
                ),
            }
        }
        "tools/list" => ActionDispatchPlan::Immediate {
            response: action_ok(id, json!({ "tools": action_schema_list() })),
        },
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            if name.is_empty() {
                return ActionDispatchPlan::Immediate {
                    response: action_err(id, -32602, "tools/call requires 'name'"),
                };
            }
            ActionDispatchPlan::ToolCall {
                name: name.to_string(),
                args: params.get("arguments").cloned().unwrap_or(Value::Null),
            }
        }
        "resources/list" => {
            let resources = workspace.map_or_else(
                || json!([]),
                |workspace| {
                    json!([{
                        "uri": format!("file://{}", workspace.root.display()),
                        "name": "Workspace root",
                        "description": "AI supervisor configured MCP workspace root"
                    }])
                },
            );
            ActionDispatchPlan::Immediate {
                response: action_ok(id, json!({ "resources": resources })),
            }
        }
        "prompts/list" => ActionDispatchPlan::Immediate {
            response: action_ok(id, json!({ "prompts": [] })),
        },
        "ping" => ActionDispatchPlan::Immediate {
            response: action_ok(id, json!({})),
        },
        _ => ActionDispatchPlan::Immediate {
            response: action_err(id, -32601, &format!("Method not found: {method}")),
        },
    }
}

// Legacy MCP name alias.
pub use self::dispatch_action_plan as dispatch_ai_mcp_plan;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_list_dispatch_exposes_gateway_landmark_surface_only() {
        let plan = dispatch_action_plan("tools/list", json!(1), Value::Null, None);
        let response = match plan {
            ActionDispatchPlan::Immediate { response } => response,
            other @ ActionDispatchPlan::Initialize { .. }
            | other @ ActionDispatchPlan::ToolCall { .. } => {
                panic!("expected immediate tools/list response, got {other:?}")
            }
        };
        let names: Vec<&str> = response["result"]["tools"]
            .as_array()
            .expect("tools list")
            .iter()
            .map(|tool| tool["name"].as_str().expect("tool name"))
            .collect();

        assert_eq!(
            names,
            vec![
                "get_manifest",
                "get_landmarks",
                "inspect_landmark",
                "call_action",
                "execute_sequence",
            ]
        );
        assert!(!names.contains(&"shell"));
        assert!(!names.contains(&"apply_patch"));
    }

    #[test]
    fn tools_call_dispatch_preserves_gateway_call_arguments() {
        let plan = dispatch_action_plan(
            "tools/call",
            json!(1),
            json!({
                "name": "call_action",
                "arguments": {
                    "action": "workspace:shell",
                    "parameters": { "command": "pwd" }
                }
            }),
            None,
        );

        match plan {
            ActionDispatchPlan::ToolCall { name, args } => {
                assert_eq!(name, "call_action");
                assert_eq!(args["action"], "workspace:shell");
                assert_eq!(args["parameters"]["command"], "pwd");
            }
            other @ ActionDispatchPlan::Initialize { .. }
            | other @ ActionDispatchPlan::Immediate { .. } => {
                panic!("expected gateway tool call dispatch, got {other:?}")
            }
        }
    }
}
