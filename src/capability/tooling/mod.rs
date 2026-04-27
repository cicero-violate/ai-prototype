//! Tooling capability.
//!
//! Tooling executes real work outside the kernel and submits a deterministic
//! artifact receipt into the runtime.

pub mod record;

pub use self::record::{
    DeterministicToolExecutor, ToolDecision, ToolExecutionRecord, ToolKind, ToolReceipt,
    ToolRequest,
};