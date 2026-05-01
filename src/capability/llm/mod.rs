//! LLM capability.
//!
//! The LLM layer owns prompt/response details and converts assembled context
//! into a structured judgment record. The kernel still sees only
//! `Evidence::JudgmentRecord`.

pub mod ollama;
pub mod record;

pub use self::ollama::{
    append_ollama_judgment_proof_event_ndjson, append_ollama_llm_effect_receipt_ndjson,
    decode_ollama_judgment_proof_event_ndjson, decode_ollama_llm_effect_receipt_ndjson,
    encode_ollama_judgment_proof_event_ndjson, encode_ollama_llm_effect_receipt_ndjson,
    load_ollama_judgment_proof_events_ndjson, load_ollama_llm_effect_receipts_ndjson,
    verify_ollama_judgment_proof_event_order_ndjson, verify_ollama_judgment_proof_events,
    verify_ollama_judgment_proof_events_ndjson, verify_ollama_judgment_tlog_ndjson,
    verify_ollama_llm_effect_receipts,
    OllamaChatResponse, OllamaClient, OllamaConfig, OllamaError, OllamaJudgmentProofEvent,
    OllamaLlmCall, OllamaLlmEffectReceipt, OllamaMessage, OllamaRetryBudgetDecision,
    OllamaRetryBudgetLedger, OllamaRetryBudgetPolicy, OLLAMA_JUDGMENT_PROOF_LINE,
    OLLAMA_JUDGMENT_PROOF_RECORD,
    OLLAMA_JUDGMENT_PROOF_SCHEMA_VERSION,
    OLLAMA_LLM_EFFECT_RECEIPT_RECORD, OLLAMA_LLM_EFFECT_RECEIPT_SCHEMA_VERSION,
    OLLAMA_PROVIDER,
};
pub use self::record::{
    LlmDecision, LlmPromptRecord, LlmRecord, LlmResponseRecord, LlmStructuredAdapter,
};
