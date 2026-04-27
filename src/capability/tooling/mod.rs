//! Tooling capability.
//!
//! Tooling executes real work outside the kernel and submits a deterministic
//! artifact receipt into the runtime.

pub mod record;

pub use self::record::{
    append_sandbox_process_receipt_ndjson, append_tool_effect_receipt_ndjson,
    decode_sandbox_process_receipt_ndjson, decode_tool_effect_receipt_ndjson,
    encode_sandbox_process_receipt_ndjson, encode_tool_effect_receipt_ndjson,
    load_sandbox_process_receipts_ndjson, load_tool_effect_receipts_ndjson,
    verify_sandbox_process_receipts, verify_tool_effect_receipts, DeterministicToolExecutor, Effect,
    LiveSandboxProcessExecutor, LiveSandboxToolExecutor, SandboxProcessReceipt,
    SandboxProcessRequest, ToolDecision, ToolEffectKind, ToolEffectReceipt, ToolExecutionRecord,
    ToolKind, ToolReceipt, ToolRequest, ToolSandboxError, SANDBOX_PROCESS_RECEIPT_RECORD,
    SANDBOX_PROCESS_RECEIPT_SCHEMA_VERSION, TOOL_EFFECT_RECEIPT_RECORD,
    TOOL_EFFECT_RECEIPT_SCHEMA_VERSION,
};