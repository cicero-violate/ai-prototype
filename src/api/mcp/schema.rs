//! Static native MCP tool schemas.

use serde_json::{json, Value};

pub fn ai_mcp_tools_list() -> Value {
    json!([
        {
            "name": "apply_patch",
            "description": "Apply an apply_patch-format patch to files under the AI MCP workspace root. No shell execution.",
            "inputSchema": {
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
            },
            "annotations": { "readOnlyHint": false, "destructiveHint": false, "idempotentHint": false, "openWorldHint": false }
        },
        {
            "name": "echo",
            "description": "Echo back the provided text.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": { "type": "string" },
                    "intent": { "type": "string" }
                },
                "required": ["text"]
            },
            "annotations": { "readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false }
        },
        {
            "name": "get_current_time",
            "description": "Returns the current UTC date and time in ISO 8601 format.",
            "inputSchema": {
                "type": "object",
                "properties": { "intent": { "type": "string" } }
            },
            "annotations": { "readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false }
        },
        {
            "name": "shell",
            "description": "Run a bounded shell command under the AI MCP workspace.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "command": { "type": "string" },
                    "cwd": { "type": "string", "default": "." },
                    "timeout_ms": { "type": "integer", "default": 180000 },
                    "max_output_bytes": { "type": "integer", "default": 65536 },
                    "intent": { "type": "string" }
                },
                "required": ["command"]
            },
            "annotations": { "readOnlyHint": false, "destructiveHint": true, "idempotentHint": false, "openWorldHint": false }
        },
        {
            "name": "canon_spawn_agent",
            "description": "Spawn a child agent via the AI supervisor. Fire-and-forget; returns a spawn receipt immediately.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "domain": { "type": "string", "description": "Domain hint for the child agent's objective." },
                    "metric": { "type": "string", "description": "Success metric the child agent must satisfy." },
                    "max_steps": { "type": "integer", "description": "Maximum cycle steps for the child agent.", "default": 20 },
                    "intent": { "type": "string" }
                },
                "required": ["domain", "metric"]
            },
            "annotations": { "readOnlyHint": false, "destructiveHint": false, "idempotentHint": false, "openWorldHint": false }
        },
        {
            "name": "canon_send_agent_message",
            "description": "Write a typed message to a named agent's mailbox.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sender": { "type": "string", "description": "Identifier for the sender." },
                    "target_agent": { "type": "string", "description": "Identifier of the receiving agent." },
                    "message_kind": { "type": "string", "enum": ["DomainSignal", "GraphEditRequest", "EvalRequest", "PolicyCandidate", "Observation", "TaskAssignment"] },
                    "payload": { "type": "string", "description": "JSON payload matching the message kind schema." },
                    "intent": { "type": "string" }
                },
                "required": ["sender", "target_agent", "message_kind", "payload"]
            },
            "annotations": { "readOnlyHint": false, "destructiveHint": false, "idempotentHint": false, "openWorldHint": false }
        },
        {
            "name": "canon_read_mailbox",
            "description": "Read messages from this agent's mailbox since a cursor.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "agent_id": { "type": "string", "description": "This agent's identifier." },
                    "since": { "type": "integer", "description": "Cursor from the previous read.", "default": 0 },
                    "intent": { "type": "string" }
                },
                "required": ["agent_id"]
            },
            "annotations": { "readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false }
        }
    ])
}
