//! LLM capability.
//!
//! The LLM layer owns prompt/response details and converts assembled context
//! into a structured judgment record. The kernel still sees only
//! `Evidence::JudgmentRecord`.

pub mod ollama;
pub mod record;

pub use self::ollama::{
    append_ollama_llm_effect_receipt_ndjson, decode_ollama_llm_effect_receipt_ndjson,
    encode_ollama_llm_effect_receipt_ndjson, load_ollama_llm_effect_receipts_ndjson,
    verify_ollama_llm_effect_receipts, OllamaChatResponse, OllamaClient, OllamaConfig,
    OllamaError, OllamaLlmCall, OllamaLlmEffectReceipt, OllamaMessage,
    OLLAMA_LLM_EFFECT_RECEIPT_RECORD, OLLAMA_LLM_EFFECT_RECEIPT_SCHEMA_VERSION,
    OLLAMA_PROVIDER,
};
pub use self::record::{
    LlmDecision, LlmPromptRecord, LlmRecord, LlmResponseRecord, LlmStructuredAdapter,
};
