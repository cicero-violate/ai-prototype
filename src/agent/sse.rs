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
    pub fn new(sse_chunks_dir: &Path, tag: &str, cycle_num: u64, label: &str) -> std::io::Result<Self> {
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
            format!("{{\"sequence\":{seq},\"observed_at\":{ts},\"kind\":\"{kind}\",{extra_json}}}\n")
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
    if s.is_empty() { "turn.ndjson".to_string() } else { s.to_string() }
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
    pub target_url: Option<String>,
    pub message_stream_complete: bool,
    pub done: bool,
    pub finish_reason: Option<String>,
}

impl SseResult {
    pub fn is_complete(&self) -> bool {
        self.done && self.message_stream_complete
    }

    pub fn completion_reason(&self) -> &'static str {
        if !self.done {
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
            &mut target_url,
            &mut message_stream_complete,
            &mut done,
            &mut finish_reason,
            logger,
        );
    }

    SseResult { content, target_url, message_stream_complete, done, finish_reason }
}

fn process_sse_frame(
    raw: &str,
    content: &mut String,
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
        let size_str = rest[..end_of_size]
            .trim()
            .split(';')
            .next()
            .unwrap_or("");
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
