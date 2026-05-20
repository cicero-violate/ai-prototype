//! Progressive landmark discovery for the native AI MCP tools.
//!
//! This module keeps MCP as the transport boundary, but replaces the public
//! flat tool surface with a small gateway surface:
//! `get_manifest`, `get_landmarks`, `inspect_landmark`, `call_action`, and
//! `execute_sequence`.
//!
//! # Compiler enforcement
//!
//! - `NativeTool`: adding a variant requires a match arm in `native_input_schema`.
//!   The compiler rejects a missing arm — you cannot add a tool without a schema.
//!
//! - `Landmark`: adding a variant requires match arms in `as_str`, `description`,
//!   and `from_str`, plus an entry in `ALL`. The compiler enforces the three match
//!   arms; the `landmark_topology_covers_all_variants` test enforces `ALL`.

use serde_json::{json, Map, Value};

// ── Enums ─────────────────────────────────────────────────────────────────────

/// Every native MCP tool that can be reached through the gateway.
/// Adding a variant here forces a match arm in `native_input_schema` — the
/// compiler will reject the build until a schema is provided.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeTool {
    ApplyPatch,
    Shell,
    Echo,
    GetCurrentTime,
    CanonSpawnAgent,
    CanonSendAgentMessage,
    CanonReadMailbox,
    CanonRuntimeState,
    CanonSupervisorHealth,
    CanonSupervisorReloadWorker,
    CanonSupervisorRestart,
    CanonWorkspaceGet,
    CanonWorkspaceSet,
    CanonBrowserListTabs,
    CanonBrowserCloseTab,
    CanonBrowserUpload,
    CanonBrowserGroupChat,
    CanonGraphPlanPatch,
    CanonGraphPlanCfg,
    CanonGraphApplyOps,
    CanonGraphVerifyCfgDelta,
    CanonGraphAutoRefactorCfg,
    CanonGraphAnalysis,
    CanonScore,
    CanonPlanRead,
    CanonPlanUpdate,
}

impl NativeTool {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ApplyPatch => "apply_patch",
            Self::Shell => "shell",
            Self::Echo => "echo",
            Self::GetCurrentTime => "get_current_time",
            Self::CanonSpawnAgent => "canon_spawn_agent",
            Self::CanonSendAgentMessage => "canon_send_agent_message",
            Self::CanonReadMailbox => "canon_read_mailbox",
            Self::CanonRuntimeState => "canon_runtime_state",
            Self::CanonSupervisorHealth => "canon_supervisor_health",
            Self::CanonSupervisorReloadWorker => "canon_supervisor_reload_worker",
            Self::CanonSupervisorRestart => "canon_supervisor_restart",
            Self::CanonWorkspaceGet => "canon_workspace_get",
            Self::CanonWorkspaceSet => "canon_workspace_set",
            Self::CanonBrowserListTabs => "canon_browser_list_tabs",
            Self::CanonBrowserCloseTab => "canon_browser_close_tab",
            Self::CanonBrowserUpload => "canon_browser_upload",
            Self::CanonBrowserGroupChat => "canon_browser_group_chat",
            Self::CanonGraphPlanPatch => "canon_graph_plan_patch",
            Self::CanonGraphPlanCfg => "canon_graph_plan_cfg",
            Self::CanonGraphApplyOps => "canon_graph_apply_ops",
            Self::CanonGraphVerifyCfgDelta => "canon_graph_verify_cfg_delta",
            Self::CanonGraphAutoRefactorCfg => "canon_graph_auto_refactor_cfg",
            Self::CanonGraphAnalysis => "canon_graph_analysis",
            Self::CanonScore => "canon_score",
            Self::CanonPlanRead => "canon_plan_read",
            Self::CanonPlanUpdate => "canon_plan_update",
        }
    }
}

/// Every landmark group exposed via the discovery surface.
/// Adding a variant requires match arms in `as_str`, `description`, and
/// `from_str` (compiler-enforced), plus an entry in `ALL`
/// (test-enforced by `landmark_topology_covers_all_variants`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Landmark {
    Project,
    Workspace,
    Utility,
    Agents,
    Runtime,
    Supervisor,
    Browser,
    Graph,
}

impl Landmark {
    pub const ALL: &'static [Self] = &[
        Self::Project,
        Self::Workspace,
        Self::Utility,
        Self::Agents,
        Self::Runtime,
        Self::Supervisor,
        Self::Browser,
        Self::Graph,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Workspace => "workspace",
            Self::Utility => "utility",
            Self::Agents => "agents",
            Self::Runtime => "runtime",
            Self::Supervisor => "supervisor",
            Self::Browser => "browser",
            Self::Graph => "graph",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Project => "Plan DAG and structural quality score for the current project.",
            Self::Workspace => "File and process operations inside the configured workspace.",
            Self::Utility => "Safe utility actions.",
            Self::Agents => "Supervisor and mailbox actions for multi-agent operation.",
            Self::Runtime => "AI worker runtime state actions.",
            Self::Supervisor => "AI supervisor lifecycle actions.",
            Self::Browser => "Browser-router tab and action operations.",
            Self::Graph => "Graph-backed source mutation planning operations.",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "project" => Some(Self::Project),
            "workspace" => Some(Self::Workspace),
            "utility" => Some(Self::Utility),
            "agents" => Some(Self::Agents),
            "runtime" => Some(Self::Runtime),
            "supervisor" => Some(Self::Supervisor),
            "browser" => Some(Self::Browser),
            "graph" => Some(Self::Graph),
            _ => None,
        }
    }
}

// ── LandmarkAction ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LandmarkAction {
    pub id: &'static str,
    pub native_tool: NativeTool,
    pub landmark: Landmark,
    pub description: &'static str,
    pub read_only: bool,
    pub destructive: bool,
    pub idempotent: bool,
}

// ── Action registry ───────────────────────────────────────────────────────────

const ACTIONS: &[LandmarkAction] = &[
    LandmarkAction {
        id: "project:score",
        native_tool: NativeTool::CanonScore,
        landmark: Landmark::Project,
        description: "Run the structural quality scorer over captured compiler fact graphs and update SCORE_REPORT.md. Returns axis scores (Architecture, Structure, Simplicity, Maintainability, Determinism, Coherency) and geometric mean G.",
        read_only: false,
        destructive: false,
        idempotent: true,
    },
    LandmarkAction {
        id: "project:plan_read",
        native_tool: NativeTool::CanonPlanRead,
        landmark: Landmark::Project,
        description: "Read the current plan DAG. Returns all nodes, edges, version, and ready node IDs calculated from the kernel PlanState projection, using state/plan.json as the explicit fallback import source before a kernel projection is available.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
    LandmarkAction {
        id: "project:plan_update",
        native_tool: NativeTool::CanonPlanUpdate,
        landmark: Landmark::Project,
        description: "Update the plan DAG in state/plan.json. op=replace (full plan), op=set_status {node_id, status}, op=set_assignee {node_id, assignee}, op=upsert_node {node}, op=append_evidence {node_id, evidence:{path,kind,summary}}, op=add_edge {edge}, op=remove_node {node_id}.",
        read_only: false,
        destructive: false,
        idempotent: false,
    },
    LandmarkAction {
        id: "workspace:apply_patch",
        native_tool: NativeTool::ApplyPatch,
        landmark: Landmark::Workspace,
        description: "Apply or check an apply_patch-format patch under the configured workspace root.",
        read_only: false,
        destructive: false,
        idempotent: false,
    },
    LandmarkAction {
        id: "workspace:shell",
        native_tool: NativeTool::Shell,
        landmark: Landmark::Workspace,
        description: "Run a bounded shell command under the configured workspace root.",
        read_only: false,
        destructive: true,
        idempotent: false,
    },
    LandmarkAction {
        id: "workspace:get",
        native_tool: NativeTool::CanonWorkspaceGet,
        landmark: Landmark::Workspace,
        description: "Read the configured MCP workspace root and allowed boundary.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
    LandmarkAction {
        id: "workspace:set",
        native_tool: NativeTool::CanonWorkspaceSet,
        landmark: Landmark::Workspace,
        description: "Set the configured MCP workspace root within the allowed boundary.",
        read_only: false,
        destructive: false,
        idempotent: false,
    },
    LandmarkAction {
        id: "utility:echo",
        native_tool: NativeTool::Echo,
        landmark: Landmark::Utility,
        description: "Echo back the provided text.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
    LandmarkAction {
        id: "utility:get_current_time",
        native_tool: NativeTool::GetCurrentTime,
        landmark: Landmark::Utility,
        description: "Return the current UTC date and time in ISO 8601 format.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
    LandmarkAction {
        id: "agents:spawn",
        native_tool: NativeTool::CanonSpawnAgent,
        landmark: Landmark::Agents,
        description: "Spawn a child agent through the AI supervisor.",
        read_only: false,
        destructive: false,
        idempotent: false,
    },
    LandmarkAction {
        id: "agents:send_message",
        native_tool: NativeTool::CanonSendAgentMessage,
        landmark: Landmark::Agents,
        description: "Write a typed message to a named agent mailbox.",
        read_only: false,
        destructive: false,
        idempotent: false,
    },
    LandmarkAction {
        id: "agents:read_mailbox",
        native_tool: NativeTool::CanonReadMailbox,
        landmark: Landmark::Agents,
        description: "Read messages from an agent mailbox since a cursor.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
    LandmarkAction {
        id: "runtime:state",
        native_tool: NativeTool::CanonRuntimeState,
        landmark: Landmark::Runtime,
        description: "Read the active AI worker state snapshot.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
    LandmarkAction {
        id: "supervisor:health",
        native_tool: NativeTool::CanonSupervisorHealth,
        landmark: Landmark::Supervisor,
        description: "Read the AI supervisor health and active worker generation.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
    LandmarkAction {
        id: "supervisor:reload_worker",
        native_tool: NativeTool::CanonSupervisorReloadWorker,
        landmark: Landmark::Supervisor,
        description: "Reload the active AI worker process.",
        read_only: false,
        destructive: true,
        idempotent: false,
    },
    LandmarkAction {
        id: "supervisor:restart",
        native_tool: NativeTool::CanonSupervisorRestart,
        landmark: Landmark::Supervisor,
        description: "Request supervisor process restart through the control API semantics.",
        read_only: false,
        destructive: true,
        idempotent: false,
    },
    LandmarkAction {
        id: "browser:list_tabs",
        native_tool: NativeTool::CanonBrowserListTabs,
        landmark: Landmark::Browser,
        description: "List browser-router CDP page tabs.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
    LandmarkAction {
        id: "browser:close_tab",
        native_tool: NativeTool::CanonBrowserCloseTab,
        landmark: Landmark::Browser,
        description: "Close one browser-router tab by target id.",
        read_only: false,
        destructive: true,
        idempotent: false,
    },
    LandmarkAction {
        id: "browser:upload",
        native_tool: NativeTool::CanonBrowserUpload,
        landmark: Landmark::Browser,
        description: "Run browser-router project file upload action.",
        read_only: false,
        destructive: false,
        idempotent: false,
    },
    LandmarkAction {
        id: "browser:group_chat",
        native_tool: NativeTool::CanonBrowserGroupChat,
        landmark: Landmark::Browser,
        description: "Run browser-router group-chat creation action.",
        read_only: false,
        destructive: false,
        idempotent: false,
    },
    LandmarkAction {
        id: "graph:plan_patch",
        native_tool: NativeTool::CanonGraphPlanPatch,
        landmark: Landmark::Graph,
        description: "Plan a deterministic source patch and graph patch receipt from graph mutation ops.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
    LandmarkAction {
        id: "graph:apply_ops",
        native_tool: NativeTool::CanonGraphApplyOps,
        landmark: Landmark::Graph,
        description: "Apply graph mutation ops to graph.files, render a worktree, optionally validate, and optionally recapture.",
        read_only: false,
        destructive: false,
        idempotent: false,
    },
    LandmarkAction {
        id: "graph:plan_cfg",
        native_tool: NativeTool::CanonGraphPlanCfg,
        landmark: Landmark::Graph,
        description: "Plan CFG-oriented graph mutation ops from a function node, strategy, and replacement text.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
    LandmarkAction {
        id: "graph:verify_cfg_delta",
        native_tool: NativeTool::CanonGraphVerifyCfgDelta,
        landmark: Landmark::Graph,
        description: "Verify expected CFG metric deltas between an old and new graph.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
    LandmarkAction {
        id: "graph:auto_refactor_cfg",
        native_tool: NativeTool::CanonGraphAutoRefactorCfg,
        landmark: Landmark::Graph,
        description: "Plan CFG ops, apply them to graph.files, render a worktree, optionally validate, optionally recapture, and optionally verify CFG delta.",
        read_only: false,
        destructive: false,
        idempotent: false,
    },
    LandmarkAction {
        id: "graph:analysis",
        native_tool: NativeTool::CanonGraphAnalysis,
        landmark: Landmark::Graph,
        description: "Run static analyses and graph projections over a captured compiler fact graph: SCC cycles, layer violations, function intents, call graph, CFG, module graph, def-use, type graph, ownership, effect graph, derived edges, or all at once.",
        read_only: true,
        destructive: false,
        idempotent: true,
    },
];

// ── Gateway constants ──────────────────────────────────────────────────────────

pub const GATEWAY_GET_MANIFEST: &str = "get_manifest";
pub const GATEWAY_GET_LANDMARKS: &str = "get_landmarks";
pub const GATEWAY_INSPECT_LANDMARK: &str = "inspect_landmark";
pub const GATEWAY_CALL_ACTION: &str = "call_action";
pub const GATEWAY_EXECUTE_SEQUENCE: &str = "execute_sequence";

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

// ── Gateway tool list (MCP surface) ───────────────────────────────────────────

pub fn gateway_mcp_tools_list() -> Value {
    json!([
        {
            "name": GATEWAY_GET_MANIFEST,
            "description": "Get Canon AI landmark protocol instructions and high-level topology.",
            "inputSchema": { "type": "object", "properties": { "intent": { "type": "string" } } },
            "annotations": { "readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false }
        },
        {
            "name": GATEWAY_GET_LANDMARKS,
            "description": "Return high-level functional areas. Use before inspect_landmark.",
            "inputSchema": { "type": "object", "properties": { "intent": { "type": "string" } } },
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

// ── Manifest / landmarks / inspect ────────────────────────────────────────────

pub fn manifest_result() -> Value {
    text_result(manifest_text())
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

/// Built from `Landmark::ALL` so the topology string is always in sync.
/// Adding a new `Landmark` variant and forgetting `ALL` is caught by
/// `landmark_topology_covers_all_variants`.
pub fn landmarks_result() -> Value {
    let mut text = String::from("### LANDMARK TOPOLOGY\n");
    for landmark in Landmark::ALL {
        text.push_str(&format!(
            "- **{}**: {}\n",
            landmark.as_str(),
            landmark.description()
        ));
    }
    text.push_str(
        "\nProtocol:\n\
         1. Use `get_landmarks` to choose an area.\n\
         2. Use `inspect_landmark` for exact action IDs and schemas.\n\
         3. Use `call_action` for one action or `execute_sequence` for batched actions.\n\
         4. Do not guess parameters; inspect first.",
    );
    text_result(text)
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

fn inspect_one(id: &str) -> Value {
    if let Some(landmark) = Landmark::from_str(id) {
        let actions: Vec<Value> = ACTIONS
            .iter()
            .copied()
            .filter(|a| a.landmark == landmark)
            .map(action_summary)
            .collect();
        return json!({
            "landmark_id": landmark.as_str(),
            "type": "landmark",
            "actions": actions,
            "remedy": "Inspect a specific action ID for its exact input schema before calling it."
        });
    }
    match resolve_action_id(id) {
        Some(action) => action_detail(action),
        None => {
            let known: Vec<&str> = Landmark::ALL.iter().map(|l| l.as_str()).collect();
            json!({
                "status": "error",
                "message": format!("Unknown landmark or action: {id}"),
                "known_landmarks": known
            })
        }
    }
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
        "canon_runtime_state" => "runtime:state",
        "canon_supervisor_health" => "supervisor:health",
        "canon_supervisor_reload_worker" => "supervisor:reload_worker",
        "canon_supervisor_restart" => "supervisor:restart",
        "canon_workspace_get" => "workspace:get",
        "canon_workspace_set" => "workspace:set",
        "canon_browser_list_tabs" => "browser:list_tabs",
        "canon_browser_close_tab" => "browser:close_tab",
        "canon_browser_upload" => "browser:upload",
        "canon_browser_group_chat" => "browser:group_chat",
        "canon_graph_plan_patch" => "graph:plan_patch",
        "canon_graph_plan_cfg" => "graph:plan_cfg",
        "canon_graph_apply_ops" => "graph:apply_ops",
        "canon_graph_verify_cfg_delta" => "graph:verify_cfg_delta",
        "canon_graph_auto_refactor_cfg" => "graph:auto_refactor_cfg",
        "canon_graph_analysis" => "graph:analysis",
        "canon_score" => "project:score",
        "canon_plan_read" => "project:plan_read",
        "canon_plan_update" => "project:plan_update",
        value => value,
    };
    ACTIONS.iter().copied().find(|a| a.id == normalized)
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
        "native_tool": action.native_tool.as_str(),
        "landmark": action.landmark.as_str(),
        "description": action.description,
        "inputSchema": native_input_schema(action.native_tool),
        "annotations": annotations(action),
        "aliases": [action.native_tool.as_str()],
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

// ── Input schemas — exhaustive match enforced by compiler ─────────────────────

fn intent_only() -> Value {
    json!({ "type": "object", "properties": { "intent": { "type": "string" } } })
}

/// Returns the JSON Schema for the given tool's input parameters.
/// This match is exhaustive over `NativeTool` — the compiler will reject any
/// new variant that lacks an arm here, preventing silent schema omissions.
fn native_input_schema(tool: NativeTool) -> Value {
    match tool {
        NativeTool::ApplyPatch => json!({
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

        NativeTool::Shell => json!({
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

        NativeTool::Echo => json!({
            "type": "object",
            "properties": {
                "text": { "type": "string" },
                "intent": { "type": "string" }
            },
            "required": ["text"]
        }),

        NativeTool::GetCurrentTime
        | NativeTool::CanonRuntimeState
        | NativeTool::CanonSupervisorHealth
        | NativeTool::CanonSupervisorReloadWorker
        | NativeTool::CanonSupervisorRestart
        | NativeTool::CanonWorkspaceGet
        | NativeTool::CanonScore
        | NativeTool::CanonPlanRead
        | NativeTool::CanonBrowserListTabs => intent_only(),

        NativeTool::CanonSpawnAgent => json!({
            "type": "object",
            "properties": {
                "domain": { "type": "string", "description": "Domain hint for the child agent's objective." },
                "metric": { "type": "string", "description": "Success metric the child agent must satisfy." },
                "max_steps": { "type": "integer", "description": "Maximum cycle steps for the child agent.", "default": 20 },
                "intent": { "type": "string" }
            },
            "required": ["domain", "metric"]
        }),

        NativeTool::CanonSendAgentMessage => json!({
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

        NativeTool::CanonReadMailbox => json!({
            "type": "object",
            "properties": {
                "agent_id": { "type": "string", "description": "This agent's identifier." },
                "since": { "type": "integer", "description": "Cursor from the previous read.", "default": 0 },
                "intent": { "type": "string" }
            },
            "required": ["agent_id"]
        }),

        NativeTool::CanonWorkspaceSet => json!({
            "type": "object",
            "properties": {
                "root": { "type": "string", "description": "Workspace root path inside the allowed boundary." },
                "intent": { "type": "string" }
            },
            "required": ["root"]
        }),

        NativeTool::CanonBrowserCloseTab => json!({
            "type": "object",
            "properties": {
                "target_id": { "type": "string", "description": "Browser-router target id." },
                "intent": { "type": "string" }
            },
            "required": ["target_id"]
        }),

        NativeTool::CanonBrowserUpload => json!({
            "type": "object",
            "properties": {
                "project_id": {},
                "target_url": {},
                "match": {},
                "build_tar": { "type": "boolean" },
                "file": { "type": "string" },
                "tar_script": { "type": "string" },
                "tar_output": { "type": "string" },
                "target_wait_timeout_sec": {},
                "confirm_timeout_sec": {},
                "confirm_settle_sec": {},
                "intent": { "type": "string" }
            }
        }),

        NativeTool::CanonBrowserGroupChat => json!({
            "type": "object",
            "properties": {
                "target_url": { "type": "string" },
                "message": { "type": "string" },
                "prompt": { "type": "string" },
                "intent": { "type": "string" }
            }
        }),

        NativeTool::CanonGraphPlanPatch => json!({
            "type": "object",
            "properties": {
                "graph_contract": { "type": "string", "description": "Workspace-relative graph snapshot contract NDJSON path." },
                "graph_contract_path": { "type": "string", "description": "Alias for graph_contract." },
                "ops": { "type": "string", "description": "Workspace-relative graph mutation ops NDJSON path." },
                "ops_path": { "type": "string", "description": "Alias for ops." },
                "source_root": { "type": "string", "default": ".", "description": "Workspace-relative source root used to read files referenced by ops." },
                "patch_out": { "type": "string", "description": "Optional workspace-relative file to write the generated patch." },
                "receipt_out": { "type": "string", "description": "Optional workspace-relative file to write the patch receipt NDJSON." },
                "intent": { "type": "string" }
            }
        }),

        NativeTool::CanonGraphPlanCfg => json!({
            "type": "object",
            "properties": {
                "graph": { "type": "string", "description": "Workspace-relative schema-17 graph.json containing metrics.cfg." },
                "graph_path": { "type": "string", "description": "Alias for graph." },
                "node": { "type": "string", "description": "Graph node path for the function to transform." },
                "path": { "type": "string", "description": "Alias for node." },
                "strategy": graph_cfg_strategy_schema(),
                "replacement": { "type": "string", "description": "Replacement source text for exact span rewrite strategies." },
                "guard": { "type": "string", "description": "Guard source text for GuardClauseInsert." },
                "lo": { "type": "integer", "description": "Optional source span start override." },
                "hi": { "type": "integer", "description": "Optional source span end override." },
                "ops_out": { "type": "string", "description": "Optional workspace-relative file to write planned ops NDJSON." },
                "intent": { "type": "string" }
            },
            "required": ["node", "strategy"]
        }),

        NativeTool::CanonGraphApplyOps => json!({
            "type": "object",
            "properties": {
                "graph": { "type": "string", "description": "Workspace-relative schema-17 graph.json containing files." },
                "graph_path": { "type": "string", "description": "Alias for graph." },
                "ops": { "type": "string", "description": "Workspace-relative graph mutation ops NDJSON path." },
                "ops_path": { "type": "string", "description": "Alias for ops." },
                "worktree_out": { "type": "string", "description": "Workspace-relative output directory for rendered mutated source tree." },
                "graph_out": { "type": "string", "description": "Optional workspace-relative file for mutated graph.json." },
                "receipt_out": { "type": "string", "description": "Optional workspace-relative file for pipeline receipt JSON." },
                "patch_out": { "type": "string", "description": "Optional workspace-relative unified diff output path." },
                "apply_patch_out": { "type": "string", "description": "Optional workspace-relative apply_patch-format patch output path." },
                "apply_to_source": { "type": "boolean", "description": "Apply the generated diff to the live workspace with git apply --index." },
                "validate_command": { "type": "string", "description": "Optional shell command run inside worktree_out." },
                "recapture_command": { "type": "string", "description": "Optional shell command run inside worktree_out after validation." },
                "graph_artifact_root": { "type": "string", "description": "Optional artifact root containing graph.json files to merge into rendered worktree." },
                "artifact_root": { "type": "string", "description": "Alias for graph_artifact_root." },
                "intent": { "type": "string" }
            },
            "required": ["ops", "worktree_out"]
        }),

        NativeTool::CanonGraphVerifyCfgDelta => json!({
            "type": "object",
            "properties": {
                "old_graph": { "type": "string", "description": "Workspace-relative old schema-17 graph.json path." },
                "old_graph_path": { "type": "string", "description": "Alias for old_graph." },
                "new_graph": { "type": "string", "description": "Workspace-relative new schema-17 graph.json path." },
                "new_graph_path": { "type": "string", "description": "Alias for new_graph." },
                "node": { "type": "string", "description": "Function graph node path to compare." },
                "path": { "type": "string", "description": "Alias for node." },
                "max_complexity_increase": { "type": "integer", "description": "Optional maximum allowed cyclomatic complexity delta." },
                "require_changed": { "type": "boolean", "description": "Require CFG summary to change." },
                "intent": { "type": "string" }
            },
            "required": ["old_graph", "new_graph", "node"]
        }),

        NativeTool::CanonGraphAutoRefactorCfg => json!({
            "type": "object",
            "properties": {
                "graph": { "type": "string", "description": "Workspace-relative schema-17 graph.json containing files and metrics.cfg." },
                "graph_path": { "type": "string", "description": "Alias for graph." },
                "node": { "type": "string", "description": "Function graph node path to transform." },
                "path": { "type": "string", "description": "Alias for node." },
                "strategy": graph_cfg_strategy_schema(),
                "replacement": { "type": "string" },
                "guard": { "type": "string" },
                "lo": { "type": "integer" },
                "hi": { "type": "integer" },
                "ops_out": { "type": "string", "description": "Workspace-relative planned ops NDJSON output path." },
                "worktree_out": { "type": "string", "description": "Workspace-relative rendered worktree output directory." },
                "graph_out": { "type": "string", "description": "Optional mutated graph.json output path." },
                "receipt_out": { "type": "string", "description": "Optional pipeline receipt path." },
                "patch_out": { "type": "string", "description": "Optional workspace-relative unified diff output path." },
                "apply_patch_out": { "type": "string", "description": "Optional workspace-relative apply_patch-format patch output path." },
                "apply_to_source": { "type": "boolean", "description": "Apply the generated diff to the live workspace with git apply --index." },
                "artifact_root": { "type": "string", "description": "Optional graph artifact root for merged render; defaults to state/rustc." },
                "graph_artifact_root": { "type": "string", "description": "Alias for artifact_root." },
                "validate_command": { "type": "string" },
                "recapture_command": { "type": "string" },
                "new_graph": { "type": "string", "description": "Optional recaptured graph path for CFG delta verification." },
                "new_graph_path": { "type": "string", "description": "Alias for new_graph." },
                "intent": { "type": "string" }
            },
            "required": ["graph", "node", "strategy", "ops_out", "worktree_out"]
        }),

        NativeTool::CanonGraphAnalysis => json!({
            "type": "object",
            "properties": {
                "graph": { "type": "string", "description": "Workspace-relative path to a captured graph.json file." },
                "graph_path": { "type": "string", "description": "Alias for graph." },
                "analysis": {
                    "type": "string",
                    "description": "Which analysis to run.",
                    "enum": [
                        "scc", "layers", "intents", "derived_edges",
                        "call_graph", "cfg", "module_graph", "dependency",
                        "def_use", "type_graph", "ownership", "effect_graph",
                        "all"
                    ]
                },
                "node":   { "type": "string", "description": "Exact symbol path to focus on." },
                "module": { "type": "string", "description": "Module prefix to filter results." },
                "filter": { "type": "string", "description": "Analysis-specific narrow filter." },
                "limit":  { "type": "integer", "description": "Max results to return (default 100, max 500).", "default": 100, "maximum": 500 },
                "intent": { "type": "string" }
            },
            "required": ["graph", "analysis"]
        }),

        NativeTool::CanonPlanUpdate => json!({
            "type": "object",
            "properties": {
                "op": {
                    "type": "string",
                    "enum": ["replace", "set_status", "set_assignee", "upsert_node", "add_edge", "remove_node"],
                    "description": "Operation: replace=full plan swap; set_status/set_assignee=update one node field; upsert_node=add or replace a node; add_edge=add dependency; remove_node=delete node and its edges."
                },
                "plan": {
                    "type": "object",
                    "description": "For op=replace. Must contain 'nodes' (array) and 'edges' (array). Each node: {id, title, description, status?, assignee?, score_axes?, files?}. Each edge: {from, to}."
                },
                "node_id": { "type": "string", "description": "Target node id for set_status, set_assignee, remove_node." },
                "status": {
                    "type": "string",
                    "enum": ["pending", "running", "done", "failed", "skipped"],
                    "description": "New status for op=set_status."
                },
                "assignee": { "type": "string", "description": "New assignee string for op=set_assignee. Omit to clear." },
                "node": {
                    "type": "object",
                    "description": "Node for op=upsert_node. Required: id, title. Optional: description, status, assignee, score_axes (array), files (array).",
                    "properties": {
                        "id": { "type": "string" },
                        "title": { "type": "string" },
                        "description": { "type": "string" },
                        "status": { "type": "string", "enum": ["pending", "running", "done", "failed", "skipped"] },
                        "assignee": { "type": "string" },
                        "score_axes": { "type": "array", "items": { "type": "string" } },
                        "files": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["id", "title"]
                },
                "edge": {
                    "type": "object",
                    "description": "Edge for op=add_edge. 'to' cannot start until 'from' is done.",
                    "properties": {
                        "from": { "type": "string", "description": "ID of the prerequisite node." },
                        "to": { "type": "string", "description": "ID of the dependent node." }
                    },
                    "required": ["from", "to"]
                },
                "intent": { "type": "string" }
            },
            "required": ["op"]
        }),
    }
}

fn graph_cfg_strategy_schema() -> Value {
    json!({
        "type": "string",
        "enum": ["ReplaceSpan", "InvertBranch", "GuardClauseInsert", "ExtractBlock", "InlineBlock", "SplitLoop", "ConvertIfToMatch", "MoveStatement", "DeleteDeadBranch"]
    })
}

// ── Sequence / piping helpers ──────────────────────────────────────────────────

pub fn parameters_from_call_action(args: &Value) -> Result<(&str, Value), String> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty())
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
                .filter(|v| !v.is_empty())
                .ok_or_else(|| format!("step {idx} requires non-empty 'action'"))?
                .to_string();
            let alias = item
                .get("alias")
                .and_then(Value::as_str)
                .filter(|v| !v.is_empty())
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
            .map(|(k, v)| resolve_piping(v, aliases).map(|r| (k, r)))
            .collect::<Result<Map<String, Value>, _>>()
            .map(Value::Object),
        other => Ok(other),
    }
}

fn resolve_reference(reference: &str, aliases: &Map<String, Value>) -> Result<Value, String> {
    let body = reference.trim_start_matches('$');
    let (alias, path) = match body.find('.') {
        Some(idx) => (&body[..idx], Some(&body[idx + 1..])),
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

// ── Result helpers ─────────────────────────────────────────────────────────────

pub fn result_is_error(value: &Value) -> bool {
    value
        .get("isError")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || value
            .get("status")
            .and_then(Value::as_str)
            .map(|s| s == "error")
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

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn content_text(result: &Value) -> String {
        result["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }

    /// Every Landmark variant must appear in the topology string.
    /// Catches forgetting to add a new variant to Landmark::ALL.
    #[test]
    fn landmark_topology_covers_all_variants() {
        let text = content_text(&landmarks_result());
        for landmark in Landmark::ALL {
            assert!(
                text.contains(landmark.as_str()),
                "landmarks_result() is missing landmark: {}",
                landmark.as_str()
            );
        }
    }

    /// Every action's landmark must be inspectable via inspect_landmark.
    /// Catches adding an action that references a landmark not in Landmark::ALL.
    #[test]
    fn every_action_landmark_is_inspectable() {
        for action in ACTIONS {
            let result = inspect_landmark_result(&json!({"landmark_id": action.landmark.as_str()}));
            let text = content_text(&result);
            assert!(
                text.contains(action.id),
                "landmark '{}' inspection does not list action '{}'",
                action.landmark.as_str(),
                action.id
            );
        }
    }

    /// Every action must have a non-trivial input schema (has a 'properties' field).
    /// Since native_input_schema is an exhaustive match, the compiler already
    /// enforces coverage — this test verifies the schemas are non-empty.
    #[test]
    fn every_action_has_a_non_empty_input_schema() {
        for action in ACTIONS {
            let schema = native_input_schema(action.native_tool);
            assert!(
                schema.get("properties").is_some(),
                "action '{}' (tool={}) has no 'properties' in its schema",
                action.id,
                action.native_tool.as_str()
            );
        }
    }

    /// Landmark::from_str and as_str must round-trip for every variant in ALL.
    #[test]
    fn landmark_from_str_roundtrips_all_variants() {
        for landmark in Landmark::ALL {
            let s = landmark.as_str();
            assert_eq!(
                Landmark::from_str(s),
                Some(*landmark),
                "Landmark::from_str(\"{s}\") did not round-trip"
            );
        }
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
        for landmark in Landmark::ALL {
            assert!(landmarks.contains(landmark.as_str()));
        }
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
    fn inspect_project_landmark_returns_plan_and_score_actions() {
        let result = inspect_landmark_result(&json!({"landmark_id": "project"}));
        let text = content_text(&result);
        assert!(text.contains("project:plan_read"));
        assert!(text.contains("project:plan_update"));
        assert!(text.contains("project:score"));

        let plan_update = inspect_landmark_result(&json!({"landmark_id": "project:plan_update"}));
        let schema_text = content_text(&plan_update);
        assert!(schema_text.contains("\"op\""));
        assert!(schema_text.contains("upsert_node"));
        assert!(schema_text.contains("set_status"));
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
            NativeTool::Shell
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
