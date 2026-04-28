//! Durable tool execution payload owned by the tooling capability.
//!
//! Tooling owns the canonical external-effect normal form:
//!
//! ```text
//! request -> authorize -> execute -> Effect { kind, digest, metadata } -> receipt -> tlog
//! ```
//!
//! The kernel still sees only evidence, gates, packet effects, and hashes.
//! File artifacts, process effects, receipt codecs, replay verification, and
//! executor authorization live in typed submodules below this boundary.

mod artifact;
mod hash;
mod process;
mod receipt;
mod request;
mod types;

pub use self::artifact::{
    DeterministicToolExecutor, LiveSandboxToolExecutor, ToolExecutionRecord, ToolReceipt,
};
pub use self::process::{
    append_sandbox_process_receipt_ndjson, decode_sandbox_process_receipt_ndjson,
    encode_sandbox_process_receipt_ndjson, load_sandbox_process_receipts_ndjson,
    verify_sandbox_process_receipts, LiveSandboxProcessExecutor, SandboxProcessReceipt,
};
pub use self::receipt::{
    append_process_effect_receipt_ndjson, append_tool_effect_receipt_ndjson,
    decode_process_effect_receipt_ndjson, decode_tool_effect_receipt_ndjson,
    encode_process_effect_receipt_ndjson, encode_tool_effect_receipt_ndjson,
    load_process_effect_receipts_ndjson, load_tool_effect_receipts_ndjson,
    verify_process_effect_receipts, verify_tool_effect_receipts, ProcessEffectReceipt,
    ToolEffectReceipt,
};
pub use self::request::{SandboxProcessRequest, ToolRequest};
pub use self::types::{
    Effect, ToolDecision, ToolEffectKind, ToolKind, ToolSandboxError,
    PROCESS_EFFECT_RECEIPT_RECORD, PROCESS_EFFECT_RECEIPT_SCHEMA_VERSION,
    SANDBOX_PROCESS_RECEIPT_RECORD, SANDBOX_PROCESS_RECEIPT_SCHEMA_VERSION,
    TOOL_EFFECT_RECEIPT_RECORD, TOOL_EFFECT_RECEIPT_SCHEMA_VERSION,
};