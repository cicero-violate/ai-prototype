//! Pure MCP method dispatch planning.

use serde_json::{json, Value};
use uuid::Uuid;

use crate::process::supervisor::WorkspaceConfig;

use super::jsonrpc::{mcp_err, mcp_ok};
use super::schema::ai_mcp_tools_list;

#[derive(Clone, Debug, PartialEq)]
pub enum AiMcpDispatchPlan {
    Initialize { response: Value, session_id: String },
    Immediate { response: Value },
    ToolCall { name: String, args: Value },
}

pub fn dispatch_ai_mcp_plan(
    method: &str,
    id: Value,
    params: Value,
    workspace: Option<&WorkspaceConfig>,
) -> AiMcpDispatchPlan {
    match method {
        "initialize" => {
            let sid = Uuid::new_v4().to_string();
            AiMcpDispatchPlan::Initialize {
                session_id: sid,
                response: mcp_ok(
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
        "tools/list" => AiMcpDispatchPlan::Immediate {
            response: mcp_ok(id, json!({ "tools": ai_mcp_tools_list() })),
        },
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            if name.is_empty() {
                return AiMcpDispatchPlan::Immediate {
                    response: mcp_err(id, -32602, "tools/call requires 'name'"),
                };
            }
            AiMcpDispatchPlan::ToolCall {
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
            AiMcpDispatchPlan::Immediate {
                response: mcp_ok(id, json!({ "resources": resources })),
            }
        }
        "prompts/list" => AiMcpDispatchPlan::Immediate {
            response: mcp_ok(id, json!({ "prompts": [] })),
        },
        "ping" => AiMcpDispatchPlan::Immediate {
            response: mcp_ok(id, json!({})),
        },
        _ => AiMcpDispatchPlan::Immediate {
            response: mcp_err(id, -32601, &format!("Method not found: {method}")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tools_list_dispatch_exposes_gateway_landmark_surface_only() {
        let plan = dispatch_ai_mcp_plan("tools/list", json!(1), Value::Null, None);
        let response = match plan {
            AiMcpDispatchPlan::Immediate { response } => response,
            other => panic!("expected immediate tools/list response, got {other:?}"),
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
        let plan = dispatch_ai_mcp_plan(
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
            AiMcpDispatchPlan::ToolCall { name, args } => {
                assert_eq!(name, "call_action");
                assert_eq!(args["action"], "workspace:shell");
                assert_eq!(args["parameters"]["command"], "pwd");
            }
            other => panic!("expected gateway tool call dispatch, got {other:?}"),
        }
    }
}
