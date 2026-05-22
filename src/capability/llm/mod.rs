//! LLM capability.
//!
//! The LLM layer owns prompt/response details and converts assembled context
//! into a structured judgment record. The kernel still sees only
//! `Evidence::JudgmentRecord`.

pub mod browser_router;
pub mod ollama;
pub mod openai;
mod provider_common;
pub mod record;
pub mod sse;
pub mod task_receipt;
pub mod transport;

pub use self::browser_router::{
    RouterClient, RouterStreamingResult, RouterTabCloseOutcome, RouterTurnResult,
};
pub use self::sse::{ChunkLogger, SseResult};

pub use self::ollama::{
    append_ollama_judgment_proof_event_ndjson, append_ollama_llm_effect_receipt_ndjson,
    decode_ollama_judgment_proof_event_ndjson, decode_ollama_llm_effect_receipt_ndjson,
    encode_ollama_judgment_proof_event_ndjson, encode_ollama_llm_effect_receipt_ndjson,
    load_ollama_judgment_proof_events_ndjson, load_ollama_llm_effect_receipts_ndjson,
    verify_ollama_judgment_proof_event_order_ndjson, verify_ollama_judgment_proof_events,
    verify_ollama_judgment_proof_events_ndjson, verify_ollama_judgment_tlog_ndjson,
    verify_ollama_llm_effect_receipts, OllamaChatResponse, OllamaClient, OllamaConfig, OllamaError,
    OllamaJudgmentProofEvent, OllamaLlmCall, OllamaLlmEffectReceipt, OllamaMessage,
    OllamaRetryBudgetDecision, OllamaRetryBudgetLedger, OllamaRetryBudgetPolicy,
    OLLAMA_JUDGMENT_PROOF_LINE, OLLAMA_JUDGMENT_PROOF_RECORD, OLLAMA_JUDGMENT_PROOF_SCHEMA_VERSION,
    OLLAMA_LLM_EFFECT_RECEIPT_RECORD, OLLAMA_LLM_EFFECT_RECEIPT_SCHEMA_VERSION, OLLAMA_PROVIDER,
};
pub use self::openai::{
    append_openai_judgment_proof_event_ndjson, append_openai_llm_effect_receipt_ndjson,
    decode_openai_judgment_proof_event_ndjson, decode_openai_llm_effect_receipt_ndjson,
    encode_openai_judgment_proof_event_ndjson, encode_openai_llm_effect_receipt_ndjson,
    load_openai_judgment_proof_events_ndjson, load_openai_llm_effect_receipts_ndjson,
    messages_from_context as openai_messages_from_context,
    verify_openai_judgment_proof_event_order_ndjson, verify_openai_judgment_proof_events,
    verify_openai_judgment_proof_events_ndjson, verify_openai_judgment_tlog_ndjson,
    verify_openai_llm_effect_receipts, OpenAiChatRequest, OpenAiChatResponse, OpenAiClient,
    OpenAiConfig, OpenAiError, OpenAiFunctionCall, OpenAiFunctionTool, OpenAiJudgmentProofEvent,
    OpenAiLlmCall, OpenAiLlmEffectReceipt, OpenAiMessage, OpenAiRetryBudgetDecision,
    OpenAiRetryBudgetLedger, OpenAiRetryBudgetPolicy, OpenAiTool, OpenAiToolCall,
    OPENAI_COMPAT_PROVIDER, OPENAI_JUDGMENT_PROOF_LINE, OPENAI_JUDGMENT_PROOF_RECORD,
    OPENAI_JUDGMENT_PROOF_SCHEMA_VERSION, OPENAI_LLM_EFFECT_RECEIPT_RECORD,
    OPENAI_LLM_EFFECT_RECEIPT_SCHEMA_VERSION,
};
pub use self::record::{
    LlmDecision, LlmPromptRecord, LlmRecord, LlmResponseRecord, LlmStructuredAdapter,
};
pub use self::task_receipt::{LlmTaskContext, LlmTurnReceipt, LlmTurnRecord};
