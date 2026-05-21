//! SSE response parsing for browser-router LLM turns.
//!
//! Import path: `crate::capability::llm::sse`.

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Writes ndjson chunk logs for a single router turn. Mirrors agent-loop.mjs makeChunkLogger.
pub struct ChunkLogger {
    file: File,
    sequence: u64,
}

impl ChunkLogger {
    pub fn new(
        sse_chunks_dir: &Path,
        tag: &str,
        cycle_num: u64,
        label: &str,
    ) -> std::io::Result<Self> {
        fs::create_dir_all(sse_chunks_dir)?;
        let ts = timestamp_ms();
        let filename = safe_filename(&format!("{ts}-{tag}-cycle-{cycle_num}-{label}.ndjson"));
        let path = sse_chunks_dir.join(filename);
        let file = File::create(path)?;
        Ok(Self { file, sequence: 0 })
    }

    /// Append one ndjson entry. `extra_json` is the inner key-value pairs (already JSON, no outer braces).
    pub fn write_entry(&mut self, kind: &str, extra_json: &str) {
        let seq = self.sequence;
        self.sequence += 1;
        let ts = timestamp_ms();
        let line = if extra_json.is_empty() {
            format!("{{\"sequence\":{seq},\"observed_at\":{ts},\"kind\":\"{kind}\"}}\n")
        } else {
            format!(
                "{{\"sequence\":{seq},\"observed_at\":{ts},\"kind\":\"{kind}\",{extra_json}}}\n"
            )
        };
        let _ = self.file.write_all(line.as_bytes());
    }
}

fn safe_filename(value: &str) -> String {
    let s: String = value
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '.' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let s = s.trim_matches('-');
    if s.is_empty() {
        "turn.ndjson".to_string()
    } else {
        s.to_string()
    }
}

fn timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

// ── Turn result ───────────────────────────────────────────────────────────────

pub struct SseResult {
    pub content: String,
    pub target_id: Option<String>,
    pub target_url: Option<String>,
    pub message_stream_complete: bool,
    pub done: bool,
    pub finish_reason: Option<String>,
}

impl SseResult {
    pub fn is_complete(&self) -> bool {
        self.done && self.message_stream_complete && self.finish_reason.as_deref() != Some("length")
    }

    pub fn completion_reason(&self) -> &'static str {
        if self.finish_reason.as_deref() == Some("length") {
            "length_finished"
        } else if !self.done {
            "missing_done_frame"
        } else if !self.message_stream_complete {
            "missing_message_stream_complete"
        } else {
            "ok"
        }
    }

    /// Mirrors retryIsSafe() from agent-loop.mjs.
    pub fn retry_is_safe(&self, has_target_url: bool) -> bool {
        if has_target_url {
            return false;
        }
        if self.finish_reason.as_deref() == Some("length") {
            return false;
        }
        if self.target_url.is_some() {
            return false;
        }
        if self.message_stream_complete {
            return false;
        }
        true
    }
}

// ── SSE parsing ───────────────────────────────────────────────────────────────

/// Parse a complete SSE response body into an SseResult, logging each frame.
/// Handles both `chat.completion.chunk` and `x-turn` frame types.
pub fn parse_sse_body(body: &str, logger: &mut ChunkLogger) -> SseResult {
    let mut content = String::new();
    let mut target_id: Option<String> = None;
    let mut target_url: Option<String> = None;
    let mut message_stream_complete = false;
    let mut done = false;
    let mut finish_reason: Option<String> = None;

    let mut rest = body;
    loop {
        let (raw, next) = match rest.find("\n\n") {
            Some(idx) => (&rest[..idx], &rest[idx + 2..]),
            None => {
                if !rest.trim().is_empty() {
                    process_sse_frame(
                        rest,
                        &mut content,
                        &mut target_id,
                        &mut target_url,
                        &mut message_stream_complete,
                        &mut done,
                        &mut finish_reason,
                        logger,
                    );
                }
                break;
            }
        };
        rest = next;
        if raw.trim().is_empty() {
            continue;
        }
        process_sse_frame(
            raw,
            &mut content,
            &mut target_id,
            &mut target_url,
            &mut message_stream_complete,
            &mut done,
            &mut finish_reason,
            logger,
        );
    }

    SseResult {
        content,
        target_id,
        target_url,
        message_stream_complete,
        done,
        finish_reason,
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "SSE frame processing mutates several streaming accumulators together"
)]
fn process_sse_frame(
    raw: &str,
    content: &mut String,
    target_id: &mut Option<String>,
    target_url: &mut Option<String>,
    message_stream_complete: &mut bool,
    done: &mut bool,
    finish_reason: &mut Option<String>,
    logger: &mut ChunkLogger,
) {
    let data: String = raw
        .lines()
        .filter(|line| line.starts_with("data:"))
        .map(|line| line[5..].trim_start().to_string())
        .collect::<Vec<_>>()
        .join("");

    if data.is_empty() {
        return;
    }

    if data == "[DONE]" {
        *done = true;
        logger.write_entry("sse_frame", "\"done\":true");
        return;
    }

    if data.contains("\"chat.completion.chunk\"") {
        if let Some(delta) = extract_delta_content(&data) {
            content.push_str(&delta);
        }
        if let Some(fr) = extract_sse_string(&data, "finish_reason") {
            if !fr.is_empty() {
                *finish_reason = Some(fr);
            }
        }
        logger.write_entry("sse_frame", "\"parsed_type\":\"chunk\"");
    } else if data.contains("\"x-turn\"") {
        if let Some(id) = extract_sse_string(&data, "target_id") {
            *target_id = Some(id);
        }
        // browser.target_url
        if let Some(url) = extract_sse_string(&data, "target_url") {
            *target_url = Some(url);
        }
        // turn.message_stream_complete
        if data.contains("\"message_stream_complete\":true") {
            *message_stream_complete = true;
        }
        logger.write_entry(
            "sse_frame",
            &format!(
                "\"parsed_type\":\"x-turn\",\"message_stream_complete\":{message_stream_complete}"
            ),
        );
    }
}

fn extract_delta_content(json: &str) -> Option<String> {
    let delta_idx = json.find("\"delta\"")?;
    let content_idx = json[delta_idx..].find("\"content\"")? + delta_idx;
    let after = &json[content_idx + "\"content\"".len()..];
    let colon = after.find(':')?;
    let after_colon = after[colon + 1..].trim_start();
    if after_colon.starts_with("null") {
        return None;
    }
    decode_json_string(after_colon)
}

fn extract_sse_string(json: &str, key: &str) -> Option<String> {
    let search = format!("\"{key}\"");
    let idx = json.find(&search)?;
    let after = &json[idx + search.len()..];
    let colon = after.find(':')?;
    let after_colon = after[colon + 1..].trim_start();
    decode_json_string(after_colon)
}

fn decode_json_string(raw: &str) -> Option<String> {
    let mut chars = raw.chars();
    if chars.next()? != '"' {
        return None;
    }
    let mut out = String::new();
    let mut escaped = false;
    for ch in chars {
        if escaped {
            match ch {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                _ => {
                    out.push('\\');
                    out.push(ch);
                }
            }
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(out),
            _ => out.push(ch),
        }
    }
    None
}

// ── Chunked transfer encoding decoder ────────────────────────────────────────

/// Decode an HTTP/1.1 chunked body into a plain string.
pub fn decode_chunked_body(body: &str) -> Option<String> {
    let mut output = String::new();
    let mut rest = body;
    loop {
        let end_of_size = rest.find("\r\n")?;
        let size_str = rest[..end_of_size].trim().split(';').next().unwrap_or("");
        let size = usize::from_str_radix(size_str, 16).ok()?;
        rest = &rest[end_of_size + 2..];
        if size == 0 {
            break;
        }
        if rest.len() < size {
            // Partial final chunk — return what we have.
            output.push_str(rest);
            return Some(output);
        }
        output.push_str(&rest[..size]);
        rest = &rest[size..];
        if rest.starts_with("\r\n") {
            rest = &rest[2..];
        }
    }
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_fixture(body: &str) -> SseResult {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
        fs::create_dir_all(&root).expect("test setup should succeed");
        let dir = root.join(format!("canon-sse-fixture-{}", std::process::id()));
        let mut logger =
            ChunkLogger::new(&dir, "fixture", 0, "turn").expect("test setup should succeed");
        parse_sse_body(body, &mut logger)
    }

    #[test]
    fn complete_stream_requires_done_and_message_stream_complete() {
        let result = parse_fixture(
            "data: {\"object\":\"chat.completion.chunk\",\"choices\":[{\"delta\":{\"content\":\"ok\"},\"finish_reason\":null}]}\n\n\
             data: {\"object\":\"chat.completion.chunk\",\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n\
             data: {\"object\":\"x-turn\",\"message_stream_complete\":true,\"target_url\":\"http://127.0.0.1/thread\"}\n\n\
             data: [DONE]\n\n",
        );
        assert_eq!(result.content, "ok");
        assert_eq!(
            result.target_url.as_deref(),
            Some("http://127.0.0.1/thread")
        );
        assert!(result.is_complete());
        assert_eq!(result.completion_reason(), "ok");
        assert!(!result.retry_is_safe(false));
    }

    #[test]
    fn x_turn_preserves_target_id_for_close_path() {
        let result = parse_fixture(
            "data: {\"object\":\"x-turn\",\"message_stream_complete\":true,\"target_url\":\"https://chatgpt.com/c/current\",\"browser\":{\"target_id\":\"target-1\",\"target_url\":\"https://chatgpt.com/c/current\"}}\n\n\
             data: [DONE]\n\n",
        );
        assert_eq!(result.target_id.as_deref(), Some("target-1"));
        assert_eq!(
            result.target_url.as_deref(),
            Some("https://chatgpt.com/c/current")
        );
    }

    #[test]
    fn truncated_stream_missing_done_is_retry_safe_for_new_chat_without_target_url() {
        let result = parse_fixture(
            "data: {\"object\":\"chat.completion.chunk\",\"choices\":[{\"delta\":{\"content\":\"partial\"},\"finish_reason\":null}]}\n\n",
        );
        assert_eq!(result.content, "partial");
        assert!(!result.is_complete());
        assert_eq!(result.completion_reason(), "missing_done_frame");
        assert!(result.retry_is_safe(false));
        assert!(!result.retry_is_safe(true));
    }

    #[test]
    fn missing_message_stream_complete_is_retry_safe_only_without_target_url() {
        let result = parse_fixture(
            "data: {\"object\":\"chat.completion.chunk\",\"choices\":[{\"delta\":{\"content\":\"body\"},\"finish_reason\":\"stop\"}]}\n\n\
             data: [DONE]\n\n",
        );
        assert!(!result.is_complete());
        assert_eq!(
            result.completion_reason(),
            "missing_message_stream_complete"
        );
        assert!(result.retry_is_safe(false));
        assert!(!result.retry_is_safe(true));
    }

    #[test]
    fn target_url_preserves_evidence_and_blocks_retry_even_when_stream_metadata_is_missing() {
        let result = parse_fixture(
            "data: {\"object\":\"chat.completion.chunk\",\"choices\":[{\"delta\":{\"content\":\"body\"},\"finish_reason\":\"stop\"}]}\n\n\
             data: {\"object\":\"x-turn\",\"target_url\":\"http://127.0.0.1/thread\"}\n\n\
             data: [DONE]\n\n",
        );
        assert!(!result.is_complete());
        assert_eq!(
            result.completion_reason(),
            "missing_message_stream_complete"
        );
        assert!(result.target_url.is_some());
        assert!(!result.retry_is_safe(false));
    }

    #[test]
    fn length_finished_output_is_incomplete_and_never_retry_safe() {
        let result = parse_fixture(
            "data: {\"object\":\"chat.completion.chunk\",\"choices\":[{\"delta\":{\"content\":\"too long\"},\"finish_reason\":\"length\"}]}\n\n\
             data: {\"object\":\"x-turn\",\"message_stream_complete\":true}\n\n\
             data: [DONE]\n\n",
        );
        assert_eq!(result.finish_reason.as_deref(), Some("length"));
        assert!(!result.is_complete());
        assert_eq!(result.completion_reason(), "length_finished");
        assert!(!result.retry_is_safe(false));
        assert!(!result.retry_is_safe(true));
    }

    #[test]
    fn partial_chunked_body_preserves_available_payload_for_retry_evidence() {
        let body = "5\r\nhello\r\n9\r\nworld";
        assert_eq!(decode_chunked_body(body).as_deref(), Some("helloworld"));
    }
}
