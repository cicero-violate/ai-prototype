//! Capability execution surface.
//!
//! This module owns bounded effect execution that produces runtime receipts.
//! Implementations live in the submodules below.

pub mod action;
pub mod graph;
pub mod patch;
pub mod process;
pub mod record;
pub mod shell;

pub use crate::capability::analysis;
pub use crate::capability::exploration;

// Action tool constants.
pub use action::{
    McpToolHost, SpawnAgentToolRequest, APPLY_PATCH_TOOL, CANON_GRAPH_APPLY_OPS_TOOL,
    CANON_GRAPH_AUTO_REFACTOR_CFG_TOOL, CANON_GRAPH_PLAN_CFG_TOOL, CANON_GRAPH_PLAN_PATCH_TOOL,
    CANON_GRAPH_VERIFY_CFG_DELTA_TOOL, CANON_PLAN_READ_TOOL, CANON_PLAN_UPDATE_TOOL,
    CANON_READ_MAILBOX_TOOL, CANON_SCORE_TOOL, CANON_SEND_AGENT_MESSAGE_TOOL,
    CANON_SPAWN_AGENT_TOOL, SHELL_TOOL, STRUCTURAL_EDIT_TOOL,
};
// Action host.
pub use action::host::ActionHost as NativeToolHost;
pub use action::host::{execute_native_tool, execute_recorded_shell, ActionHost};
// Action receipts.
pub use action::record::{
    append_action_receipt_ndjson, decode_action_receipt_ndjson, encode_action_receipt_ndjson,
    load_action_receipts_ndjson, verify_action_receipts, ActionCallRequest, ActionReceipt,
    ActionReceiptStatus, LiveActionExecutor, ACTION_RECEIPT_RECORD, ACTION_RECEIPT_SCHEMA_VERSION,
};
// Execution records.
pub use record::{
    append_process_effect_receipt_ndjson, append_sandbox_process_receipt_ndjson,
    append_tool_effect_receipt_ndjson, decode_process_effect_receipt_ndjson,
    decode_sandbox_process_receipt_ndjson, decode_tool_effect_receipt_ndjson,
    encode_process_effect_receipt_ndjson, encode_sandbox_process_receipt_ndjson,
    encode_tool_effect_receipt_ndjson, load_process_effect_receipts_ndjson,
    load_sandbox_process_receipts_ndjson, load_tool_effect_receipts_ndjson,
    verify_process_effect_receipts, verify_sandbox_process_receipts, verify_tool_effect_receipts,
    DeterministicToolExecutor, Effect, LiveSandboxProcessExecutor, LiveSandboxToolExecutor,
    ProcessEffectReceipt, SandboxProcessReceipt, SandboxProcessRequest, ToolDecision,
    ToolEffectKind, ToolEffectReceipt, ToolExecutionRecord, ToolKind, ToolReceipt, ToolRequest,
    ToolSandboxError, PROCESS_EFFECT_RECEIPT_RECORD, PROCESS_EFFECT_RECEIPT_SCHEMA_VERSION,
    SANDBOX_PROCESS_RECEIPT_RECORD, SANDBOX_PROCESS_RECEIPT_SCHEMA_VERSION,
    TOOL_EFFECT_RECEIPT_RECORD, TOOL_EFFECT_RECEIPT_SCHEMA_VERSION,
};
// Legacy MCP record compatibility aliases.
pub use action::record::{
    append_action_receipt_ndjson as append_mcp_call_receipt_ndjson,
    decode_action_receipt_ndjson as decode_mcp_call_receipt_ndjson,
    encode_action_receipt_ndjson as encode_mcp_call_receipt_ndjson,
    load_action_receipts_ndjson as load_mcp_call_receipts_ndjson,
    verify_action_receipts as verify_mcp_call_receipts, ActionCallRequest as McpCallRequest,
    ActionReceipt as McpCallReceipt, LiveActionExecutor as LiveMcpCallExecutor,
    ACTION_RECEIPT_RECORD as MCP_CALL_RECEIPT_RECORD,
    ACTION_RECEIPT_SCHEMA_VERSION as MCP_CALL_RECEIPT_SCHEMA_VERSION,
};
