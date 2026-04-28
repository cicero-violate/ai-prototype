//! Tooling capability.
//!
//! Tooling executes real work outside the kernel and submits deterministic
//! effect receipts into the runtime.

pub mod record;

pub use self::record::{
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