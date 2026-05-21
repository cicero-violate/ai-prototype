//! Re-export shim — SSE types live in `capability::llm::sse`.
pub use crate::capability::llm::sse::{
    decode_chunked_body, parse_sse_body, ChunkLogger, SseResult,
};
