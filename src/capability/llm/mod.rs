//! LLM capability.
//!
//! The LLM layer owns prompt/response details and converts assembled context
//! into a structured judgment record. The kernel still sees only
//! `Evidence::JudgmentRecord`.

pub mod record;

pub use self::record::{
    LlmDecision, LlmPromptRecord, LlmRecord, LlmResponseRecord, LlmStructuredAdapter,
};
