//! Legacy MCP-named action receipt aliases.
//!
//! The implementation lives under `capability::execution::action::record`.
//! Keep this module as a compatibility shim while command tags and callers
//! migrate to action terminology.

pub use crate::capability::execution::action::record::{
    append_action_receipt_ndjson as append_mcp_call_receipt_ndjson,
    decode_action_receipt_ndjson as decode_mcp_call_receipt_ndjson,
    encode_action_receipt_ndjson as encode_mcp_call_receipt_ndjson,
    load_action_receipts_ndjson as load_mcp_call_receipts_ndjson,
    verify_action_receipts as verify_mcp_call_receipts, ActionCallRequest as McpCallRequest,
    ActionReceipt as McpCallReceipt, LiveActionExecutor as LiveMcpCallExecutor,
    ACTION_RECEIPT_RECORD as MCP_CALL_RECEIPT_RECORD,
    ACTION_RECEIPT_SCHEMA_VERSION as MCP_CALL_RECEIPT_SCHEMA_VERSION,
};
