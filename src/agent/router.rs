//! RouterClient — wraps OpenAiClient and threads target_url across turns.
//!
//! The router-server maintains ChatGPT conversation continuity via the browser
//! tab URL. Each turn returns a target_url; the next turn must supply it via
//! OpenAiBrowserOptions::continue_at to stay in the same thread.

use std::env;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::agent::sse::{decode_chunked_body, parse_sse_body, ChunkLogger};
use crate::capability::llm::openai::{
    OpenAiBrowserOptions, OpenAiChatRequest, OpenAiChatResponse, OpenAiClient, OpenAiConfig,
    OpenAiError,
};
use crate::capability::llm::transport::{
    chat_completions_path, parse_local_http_endpoint, LocalEndpointError,
};

const DEFAULT_TRANSIENT_ROUTER_ATTEMPTS: u32 = 8;
const DEFAULT_TRANSIENT_ROUTER_BACKOFF_MS: u64 = 500;
const DEFAULT_TRANSIENT_ROUTER_MAX_BACKOFF_MS: u64 = 4_000;

pub struct RouterClient {
    inner: OpenAiClient,
    /// The ChatGPT tab URL returned by the last turn. None means no session yet.
    target_url: Option<String>,
}

pub struct RouterTurnResult {
    pub response: OpenAiChatResponse,
    /// The tab URL to pass to the next turn. Same value as response.target_url.
    pub target_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouterTabCloseOutcome {
    NoTarget,
    NoCdpEndpoint,
    NoMatchingTarget,
    Closed,
    CloseHttpStatus(u16),
}

impl RouterClient {
    pub fn new(config: OpenAiConfig) -> Result<Self, OpenAiError> {
        Ok(Self {
            inner: OpenAiClient::new(config)?,
            target_url: None,
        })
    }

    pub fn from_env() -> Result<Self, OpenAiError> {
        Ok(Self {
            inner: OpenAiClient::from_env()?,
            target_url: None,
        })
    }

    pub fn target_url(&self) -> Option<&str> {
        self.target_url.as_deref()
    }

    /// Drop the current tab URL; the next turn will open a new chat.
    pub fn reset_session(&mut self) {
        self.target_url = None;
    }

    /// Best-effort close of the currently pinned browser tab.
    ///
    /// This uses Chrome's local DevTools HTTP API when the returned target URL
    /// is itself a `/devtools/page/<id>` URL, or when `CANON_BROWSER_CDP_URL`,
    /// `CDP_URL`, `CDP_PORT`, or the default `127.0.0.1:9222` debugging
    /// endpoint can resolve the tab by URL.
    pub fn close_current_tab(&mut self) -> Result<RouterTabCloseOutcome, OpenAiError> {
        let Some(target_url) = self.target_url.clone() else {
            return Ok(RouterTabCloseOutcome::NoTarget);
        };

        let outcome = retry_transient_router_io(|| {
            close_browser_tab_for_url(&target_url, self.inner.config().timeout_ms)
        })?;
        if matches!(outcome, RouterTabCloseOutcome::Closed) {
            self.target_url = None;
        }
        Ok(outcome)
    }

    /// Send one turn, threading target_url automatically.
    pub fn turn(&mut self, request: OpenAiChatRequest) -> Result<RouterTurnResult, OpenAiError> {
        let browser = match &self.target_url {
            Some(url) => OpenAiBrowserOptions::continue_at(url.clone()),
            None => OpenAiBrowserOptions::new_chat(),
        };
        let request = request.with_browser(browser);
        let response = retry_transient_router_io(|| self.inner.chat_with_request(&request))?;
        if let Some(url) = response.target_url.clone() {
            self.target_url = Some(url);
        }
        let target_url = response.target_url.clone();
        Ok(RouterTurnResult {
            response,
            target_url,
        })
    }

    pub fn inner(&self) -> &OpenAiClient {
        &self.inner
    }

    /// Streaming turn: sends `stream: true` with router browser-timing hints,
    /// reads SSE frames, logs chunks, and updates the tab URL. Mirrors chatTurn()
    /// from chatgpt-agent-loop/agent-loop.mjs.
    pub fn streaming_turn(
        &mut self,
        request: OpenAiChatRequest,
        logger: &mut ChunkLogger,
        router_turn_max_ms: u64,
        router_first_capture_ms: u64,
        router_idle_ms: u64,
    ) -> Result<RouterStreamingResult, OpenAiError> {
        let browser = match &self.target_url {
            Some(url) => OpenAiBrowserOptions::continue_at(url.clone()),
            None => OpenAiBrowserOptions::new_chat(),
        };

        let body = build_streaming_request_json(
            self.inner.config(),
            &request,
            &browser,
            router_turn_max_ms,
            router_first_capture_ms,
            router_idle_ms,
        )?;

        logger.write_entry(
            "request",
            &format!(
                "\"router_url\":\"{}\",\"model\":\"{}\",\"new_chat\":{},\"prompt_length\":{}",
                self.inner.config().base_url,
                self.inner.config().model,
                browser.new_chat,
                body.len(),
            ),
        );

        let sse = send_streaming_request(
            self.inner.config(),
            &body,
            logger,
            router_turn_max_ms.saturating_add(30_000),
        )?;

        if let Some(url) = &sse.target_url {
            self.target_url = Some(url.clone());
        }

        let complete = sse.is_complete();
        let reason = sse.completion_reason().to_string();
        let finish_reason = sse.finish_reason;

        Ok(RouterStreamingResult {
            content: sse.content,
            target_url: self.target_url.clone(),
            complete,
            reason,
            finish_reason,
        })
    }
}

fn retry_transient_router_io<T>(
    mut op: impl FnMut() -> Result<T, OpenAiError>,
) -> Result<T, OpenAiError> {
    retry_transient_router_io_with_policy(router_retry_policy(), &mut op)
}

fn retry_transient_router_io_with_policy<T>(
    policy: RouterRetryPolicy,
    mut op: impl FnMut() -> Result<T, OpenAiError>,
) -> Result<T, OpenAiError> {
    let mut attempt = 0;
    loop {
        match op() {
            Ok(value) => return Ok(value),
            Err(err) if is_transient_router_error(&err) && attempt + 1 < policy.attempts => {
                attempt += 1;
                let backoff_ms = policy.backoff_ms(attempt);
                if backoff_ms != 0 {
                    thread::sleep(Duration::from_millis(backoff_ms));
                }
            }
            Err(err) => return Err(err),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct RouterRetryPolicy {
    attempts: u32,
    base_backoff_ms: u64,
    max_backoff_ms: u64,
}

impl RouterRetryPolicy {
    fn backoff_ms(self, retry_attempt: u32) -> u64 {
        if self.base_backoff_ms == 0 || retry_attempt == 0 {
            return 0;
        }
        let shift = retry_attempt.saturating_sub(1).min(10);
        let multiplier = 1_u64 << shift;
        self.base_backoff_ms
            .saturating_mul(multiplier)
            .min(self.max_backoff_ms)
    }
}

fn router_retry_policy() -> RouterRetryPolicy {
    RouterRetryPolicy {
        attempts: env_u32(
            "CANON_ROUTER_TRANSIENT_ATTEMPTS",
            DEFAULT_TRANSIENT_ROUTER_ATTEMPTS,
        )
        .max(1),
        base_backoff_ms: env_u64(
            "CANON_ROUTER_TRANSIENT_BACKOFF_MS",
            DEFAULT_TRANSIENT_ROUTER_BACKOFF_MS,
        ),
        max_backoff_ms: env_u64(
            "CANON_ROUTER_TRANSIENT_MAX_BACKOFF_MS",
            DEFAULT_TRANSIENT_ROUTER_MAX_BACKOFF_MS,
        ),
    }
}

fn env_u32(name: &str, default: u32) -> u32 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_u64(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn is_transient_router_error(err: &OpenAiError) -> bool {
    match err {
        OpenAiError::Io(io_err) => matches!(
            io_err.kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted
        ),
        OpenAiError::HttpStatus(status) => {
            matches!(status, 408 | 409 | 425 | 429 | 500 | 502 | 503 | 504)
        }
        _ => false,
    }
}

fn close_browser_tab_for_url(
    target_url: &str,
    timeout_ms: u64,
) -> Result<RouterTabCloseOutcome, OpenAiError> {
    if let Some((host, port, target_id)) = devtools_page_target(target_url) {
        return close_cdp_target(&host, port, &target_id, timeout_ms);
    }

    let Some(cdp_url) = cdp_endpoint_from_env() else {
        return Ok(RouterTabCloseOutcome::NoCdpEndpoint);
    };
    let endpoint = parse_local_http_endpoint(&cdp_url).map_err(|e| match e {
        LocalEndpointError::InvalidUrl => OpenAiError::InvalidUrl,
        LocalEndpointError::NonLocalHost => OpenAiError::InvalidConfig("cdp url must be local"),
    })?;
    let (status, body) = cdp_get(&endpoint.host, endpoint.port, "/json/list", timeout_ms)?;
    if status != 200 {
        return Ok(RouterTabCloseOutcome::CloseHttpStatus(status));
    }

    let Some(target_id) = cdp_target_id_for_url(&body, target_url) else {
        return Ok(RouterTabCloseOutcome::NoMatchingTarget);
    };
    close_cdp_target(&endpoint.host, endpoint.port, &target_id, timeout_ms)
}

fn cdp_endpoint_from_env() -> Option<String> {
    env::var("CANON_BROWSER_CDP_URL")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| env::var("CDP_URL").ok().filter(|v| !v.trim().is_empty()))
        .or_else(|| {
            env::var("CDP_PORT")
                .ok()
                .filter(|v| !v.trim().is_empty())
                .map(|port| format!("http://127.0.0.1:{port}"))
        })
        .or_else(|| Some("http://127.0.0.1:9222".to_string()))
}

fn devtools_page_target(target_url: &str) -> Option<(String, u16, String)> {
    let endpoint = parse_local_http_endpoint(target_url).ok()?;
    let target_id = endpoint
        .path_prefix
        .strip_prefix("/devtools/page/")?
        .trim_matches('/');
    if target_id.is_empty() {
        return None;
    }
    Some((endpoint.host, endpoint.port, target_id.to_string()))
}

fn cdp_target_id_for_url(list_body: &str, target_url: &str) -> Option<String> {
    let value: Value = serde_json::from_str(list_body).ok()?;
    let entries = value.as_array()?;
    entries.iter().find_map(|entry| {
        let obj = entry.as_object()?;
        let id = obj.get("id")?.as_str()?;
        let url_matches = obj
            .get("url")
            .and_then(Value::as_str)
            .is_some_and(|url| url == target_url);
        let websocket_matches = obj
            .get("webSocketDebuggerUrl")
            .and_then(Value::as_str)
            .is_some_and(|url| url == target_url);
        let frontend_matches = obj
            .get("devtoolsFrontendUrl")
            .and_then(Value::as_str)
            .is_some_and(|url| url == target_url);
        (url_matches || websocket_matches || frontend_matches).then(|| id.to_string())
    })
}

fn close_cdp_target(
    host: &str,
    port: u16,
    target_id: &str,
    timeout_ms: u64,
) -> Result<RouterTabCloseOutcome, OpenAiError> {
    let path = format!("/json/close/{target_id}");
    let (status, _) = cdp_get(host, port, &path, timeout_ms)?;
    if status == 200 {
        Ok(RouterTabCloseOutcome::Closed)
    } else {
        Ok(RouterTabCloseOutcome::CloseHttpStatus(status))
    }
}

fn cdp_get(
    host: &str,
    port: u16,
    path: &str,
    timeout_ms: u64,
) -> Result<(u16, String), OpenAiError> {
    let mut stream = TcpStream::connect((host, port)).map_err(OpenAiError::Io)?;
    let timeout = Duration::from_millis(timeout_ms);
    stream
        .set_read_timeout(Some(timeout))
        .map_err(OpenAiError::Io)?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(OpenAiError::Io)?;

    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .map_err(OpenAiError::Io)?;
    stream.flush().map_err(OpenAiError::Io)?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(OpenAiError::Io)?;
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or(OpenAiError::InvalidResponse)?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|raw| raw.parse::<u16>().ok())
        .ok_or(OpenAiError::InvalidResponse)?;
    Ok((status, body.to_string()))
}

/// Result of a streaming SSE turn. Mirrors the return shape of chatTurn().
pub struct RouterStreamingResult {
    pub content: String,
    pub target_url: Option<String>,
    /// true when `done` frame and `message_stream_complete` both received.
    pub complete: bool,
    /// Human-readable explanation when complete=false.
    pub reason: String,
    pub finish_reason: Option<String>,
}

impl RouterStreamingResult {
    /// Whether it is safe to retry this incomplete turn. Mirrors retryIsSafe().
    pub fn retry_is_safe(&self, had_target_url_before: bool) -> bool {
        if had_target_url_before {
            return false;
        }
        if self.finish_reason.as_deref() == Some("length") {
            return false;
        }
        if self.target_url.is_some() {
            return false;
        }
        !self.complete
    }
}

// ── Streaming request builder ────────────────────────────────────────────────

fn build_streaming_request_json(
    config: &OpenAiConfig,
    request: &OpenAiChatRequest,
    browser: &OpenAiBrowserOptions,
    max_ms: u64,
    first_capture_ms: u64,
    idle_ms: u64,
) -> Result<String, OpenAiError> {
    if request.messages.is_empty() {
        return Err(OpenAiError::InvalidConfig("empty message list"));
    }

    let mut json = String::new();
    json.push_str("{\"model\":\"");
    json.push_str(&sse_json_escape(&config.model));
    json.push_str("\",\"messages\":[");
    for (idx, msg) in request.messages.iter().enumerate() {
        if idx != 0 {
            json.push(',');
        }
        push_msg_json(&mut json, &msg.role, msg.content.as_deref());
    }
    json.push_str("],\"stream\":true,\"browser\":{");
    json.push_str(&format!("\"new_chat\":{}", browser.new_chat));
    json.push_str(&format!(
        ",\"max_ms\":{max_ms},\"first_capture_ms\":{first_capture_ms},\"idle_ms\":{idle_ms}"
    ));
    if let Some(url) = &browser.target_url {
        json.push_str(",\"target_url\":\"");
        json.push_str(&sse_json_escape(url));
        json.push('"');
    }
    json.push_str("}}");
    Ok(json)
}

fn push_msg_json(out: &mut String, role: &str, content: Option<&str>) {
    out.push_str("{\"role\":\"");
    out.push_str(&sse_json_escape(role));
    out.push('"');
    if let Some(c) = content {
        out.push_str(",\"content\":\"");
        out.push_str(&sse_json_escape(c));
        out.push('"');
    }
    out.push('}');
}

fn sse_json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

// ── Streaming HTTP transport ─────────────────────────────────────────────────

fn send_streaming_request(
    config: &OpenAiConfig,
    body: &str,
    logger: &mut ChunkLogger,
    stream_deadline_ms: u64,
) -> Result<crate::agent::sse::SseResult, OpenAiError> {
    let endpoint = parse_local_http_endpoint(&config.base_url).map_err(|e| match e {
        LocalEndpointError::InvalidUrl => OpenAiError::InvalidUrl,
        LocalEndpointError::NonLocalHost => OpenAiError::InvalidConfig("base url must be local"),
    })?;

    let path = chat_completions_path(&endpoint.path_prefix);
    let request_str = build_streaming_http_request(&path, &endpoint.host, endpoint.port, body);

    let full_response = collect_streaming_response_bytes(
        &endpoint.host,
        endpoint.port,
        config.timeout_ms,
        &request_str,
        logger,
        stream_deadline_ms,
    )?;

    finalize_streaming_response(&full_response, logger)
}

fn collect_streaming_response_bytes(
    endpoint_host: &str,
    endpoint_port: u16,
    write_timeout_ms: u64,
    request: &str,
    logger: &mut ChunkLogger,
    stream_deadline_ms: u64,
) -> Result<Vec<u8>, OpenAiError> {
    let mut stream = TcpStream::connect((endpoint_host, endpoint_port)).map_err(OpenAiError::Io)?;
    stream
        .set_read_timeout(Some(Duration::from_millis(1_000)))
        .map_err(OpenAiError::Io)?;
    stream
        .set_write_timeout(Some(Duration::from_millis(write_timeout_ms)))
        .map_err(OpenAiError::Io)?;

    stream
        .write_all(request.as_bytes())
        .map_err(OpenAiError::Io)?;
    stream.flush().map_err(OpenAiError::Io)?;

    let mut full_response = Vec::new();
    let mut buf = [0_u8; 8192];
    let deadline = Instant::now()
        .checked_add(Duration::from_millis(stream_deadline_ms.max(1_000)))
        .unwrap_or_else(Instant::now);
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                full_response.extend_from_slice(&buf[..n]);
                logger.write_entry(
                    "http_chunk",
                    &format!(
                        "\"byte_length\":{},\"total_byte_length\":{}",
                        n,
                        full_response.len()
                    ),
                );
                if response_bytes_have_done_frame(&full_response) {
                    break;
                }
                if Instant::now() >= deadline {
                    return Err(OpenAiError::Io(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "stream timed out before router [DONE] frame",
                    )));
                }
            }
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                ) =>
            {
                if response_bytes_have_done_frame(&full_response) {
                    break;
                }
                if Instant::now() >= deadline {
                    return Err(OpenAiError::Io(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "stream timed out before router [DONE] frame",
                    )));
                }
            }
            Err(e) => return Err(OpenAiError::Io(e)),
        }
    }

    Ok(full_response)
}

fn finalize_streaming_response(
    full_response: &[u8],
    logger: &mut ChunkLogger,
) -> Result<crate::agent::sse::SseResult, OpenAiError> {
    let full_response = String::from_utf8_lossy(full_response);

    let (head, raw_body) = full_response
        .split_once("\r\n\r\n")
        .ok_or(OpenAiError::InvalidResponse)?;

    let status: u16 = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    if status != 200 {
        return Err(OpenAiError::HttpStatus(status));
    }

    if !response_text_has_done_frame(&full_response) {
        return Err(OpenAiError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "stream ended before router [DONE] frame",
        )));
    }

    let is_chunked = head.lines().any(|l| {
        l.to_ascii_lowercase().contains("transfer-encoding")
            && l.to_ascii_lowercase().contains("chunked")
    });

    let body_text = if is_chunked {
        decode_chunked_body(raw_body).unwrap_or_else(|| raw_body.to_string())
    } else {
        raw_body.to_string()
    };

    Ok(parse_sse_body(&body_text, logger))
}

fn build_streaming_http_request(path: &str, host: &str, port: u16, body: &str) -> String {
    format!(
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nAccept: text/event-stream\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
        body.len(),
    )
}

fn response_bytes_have_done_frame(response: &[u8]) -> bool {
    response
        .windows(b"data: [DONE]".len())
        .any(|window| window == b"data: [DONE]")
}

fn response_text_has_done_frame(response: &str) -> bool {
    response.lines().any(|line| line.trim() == "data: [DONE]")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn devtools_page_target_extracts_local_target_id() {
        assert_eq!(
            devtools_page_target("http://127.0.0.1:9222/devtools/page/ABC123"),
            Some(("127.0.0.1".to_string(), 9222, "ABC123".to_string()))
        );
        assert_eq!(devtools_page_target("https://chatgpt.com/c/abc123"), None);
    }

    #[test]
    fn cdp_target_id_matches_target_url_from_json_list() {
        let body = r#"[
          {"id":"old","url":"https://example.invalid/old"},
          {"id":"target-1","url":"https://chatgpt.com/c/current"}
        ]"#;
        assert_eq!(
            cdp_target_id_for_url(body, "https://chatgpt.com/c/current"),
            Some("target-1".to_string())
        );
        assert_eq!(cdp_target_id_for_url(body, "https://missing.invalid"), None);
    }

    #[test]
    fn response_done_frame_detection_accepts_chunked_raw_body() {
        let response = concat!(
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n",
            "e\r\ndata: [DONE]\n\n\r\n0\r\n\r\n"
        );
        assert!(response_bytes_have_done_frame(response.as_bytes()));
        assert!(response_text_has_done_frame(response));
    }

    #[test]
    fn response_done_frame_detection_rejects_partial_stream() {
        let response = concat!(
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n",
            "22\r\ndata: {\"object\":\"chat.completion.chunk\"}\n\n\r\n"
        );
        assert!(!response_bytes_have_done_frame(response.as_bytes()));
        assert!(!response_text_has_done_frame(response));
    }

    #[test]
    fn streaming_http_request_builder_preserves_post_headers_and_body() {
        let body = r#"{"model":"gpt-test","stream":true}"#;

        let request = build_streaming_http_request("/v1/chat/completions", "127.0.0.1", 8080, body);

        let (head, actual_body) = request
            .split_once("\r\n\r\n")
            .expect("streaming HTTP request separates headers and body");
        let mut lines = head.lines();
        assert_eq!(lines.next(), Some("POST /v1/chat/completions HTTP/1.1"));

        let headers: Vec<&str> = lines.collect();
        assert!(headers.contains(&"Host: 127.0.0.1:8080"));
        assert!(headers.contains(&"Content-Type: application/json"));
        assert!(headers.contains(&"Accept: text/event-stream"));
        assert!(headers.contains(&"Connection: close"));

        let content_length = headers
            .iter()
            .find_map(|header| header.strip_prefix("Content-Length: "))
            .expect("streaming HTTP request includes Content-Length")
            .parse::<usize>()
            .expect("Content-Length is numeric");
        assert_eq!(content_length, body.len());
        assert_eq!(content_length, actual_body.len());
        assert_eq!(actual_body.as_bytes(), body.as_bytes());
    }

    #[test]
    fn finalize_streaming_response_accepts_in_memory_sse_done_response() {
        let temp_dir = std::env::temp_dir().join(format!(
            "canon-router-finalize-success-{}",
            std::process::id()
        ));
        let mut logger = ChunkLogger::new(&temp_dir, "test", 1, "success")
            .expect("chunk logger can be created for in-memory finalize test");
        let body = concat!(
            r#"data: {"object":"chat.completion.chunk","choices":[{"delta":{"content":"Hello "},"finish_reason":null}]}"#,
            "\n\n",
            r#"data: {"object":"chat.completion.chunk","choices":[{"delta":{"content":"world"},"finish_reason":"stop"}]}"#,
            "\n\n",
            r#"data: {"object":"x-turn","target_url":"thread-1","message_stream_complete":true}"#,
            "\n\n",
            "data: [DONE]\n\n",
        );
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );

        let result = finalize_streaming_response(response.as_bytes(), &mut logger)
            .expect("complete in-memory SSE response finalizes successfully");

        assert_eq!(result.content, "Hello world");
        assert_eq!(result.target_url.as_deref(), Some("thread-1"));
        assert!(result.message_stream_complete);
        assert!(result.done);
        assert_eq!(result.finish_reason.as_deref(), Some("stop"));
        assert!(result.is_complete());
        assert_eq!(result.completion_reason(), "ok");

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn finalize_streaming_response_rejects_non_200_status() {
        let temp_dir = std::env::temp_dir().join(format!(
            "canon-router-finalize-http-status-{}",
            std::process::id()
        ));
        let mut logger = ChunkLogger::new(&temp_dir, "test", 1, "http-status")
            .expect("chunk logger can be created for in-memory finalize test");
        let response = concat!(
            "HTTP/1.1 503 Service Unavailable\r\n",
            "Content-Type: text/plain\r\n",
            "Content-Length: 11\r\n",
            "\r\n",
            "unavailable"
        );

        let err = finalize_streaming_response(response.as_bytes(), &mut logger)
            .err()
            .expect("non-200 in-memory response should be rejected");

        assert!(matches!(err, OpenAiError::HttpStatus(503)));

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn finalize_streaming_response_rejects_missing_done_frame() {
        let temp_dir = std::env::temp_dir().join(format!(
            "canon-router-finalize-missing-done-{}",
            std::process::id()
        ));
        let mut logger = ChunkLogger::new(&temp_dir, "test", 1, "missing-done")
            .expect("chunk logger can be created for in-memory finalize test");
        let body = concat!(
            r#"data: {"object":"chat.completion.chunk","choices":[{"delta":{"content":"partial"},"finish_reason":null}]}"#,
            "\n\n"
        );
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );

        let err = finalize_streaming_response(response.as_bytes(), &mut logger)
            .err()
            .expect("200 in-memory response without [DONE] should be rejected");

        match err {
            OpenAiError::Io(io_err) => assert_eq!(io_err.kind(), io::ErrorKind::UnexpectedEof),
            other => panic!("expected missing [DONE] to return UnexpectedEof, got {other:?}"),
        }

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn transient_router_classifier_accepts_retryable_io_errors() {
        for kind in [
            io::ErrorKind::WouldBlock,
            io::ErrorKind::TimedOut,
            io::ErrorKind::Interrupted,
        ] {
            let err = OpenAiError::Io(io::Error::new(kind, "transient router io"));
            assert!(is_transient_router_error(&err));
        }
    }

    #[test]
    fn transient_router_classifier_accepts_retryable_http_errors() {
        for status in [408, 409, 425, 429, 500, 502, 503, 504] {
            assert!(is_transient_router_error(&OpenAiError::HttpStatus(status)));
        }
    }

    #[test]
    fn transient_router_classifier_rejects_non_retryable_errors() {
        let err = OpenAiError::Io(io::Error::new(
            io::ErrorKind::ConnectionRefused,
            "router is down",
        ));
        assert!(!is_transient_router_error(&err));
        assert!(!is_transient_router_error(&OpenAiError::HttpStatus(400)));
    }

    #[test]
    fn retry_transient_router_io_retries_then_succeeds() {
        let mut attempts = 0;
        let policy = RouterRetryPolicy {
            attempts: 3,
            base_backoff_ms: 0,
            max_backoff_ms: 0,
        };
        let result = retry_transient_router_io_with_policy(policy, || {
            attempts += 1;
            if attempts == 1 {
                return Err(OpenAiError::Io(io::Error::new(
                    io::ErrorKind::WouldBlock,
                    "router temporarily unavailable",
                )));
            }
            Ok("ok")
        });

        assert_eq!(result.unwrap(), "ok");
        assert_eq!(attempts, 2);
    }

    #[test]
    fn retry_transient_router_io_does_not_retry_non_transient_error() {
        let mut attempts = 0;
        let policy = RouterRetryPolicy {
            attempts: 3,
            base_backoff_ms: 0,
            max_backoff_ms: 0,
        };
        let result: Result<(), OpenAiError> = retry_transient_router_io_with_policy(policy, || {
            attempts += 1;
            Err(OpenAiError::Io(io::Error::new(
                io::ErrorKind::ConnectionRefused,
                "router is down",
            )))
        });

        assert!(result.is_err());
        assert_eq!(attempts, 1);
    }

    #[test]
    fn retry_transient_router_io_stops_at_attempt_limit() {
        let mut attempts = 0;
        let policy = RouterRetryPolicy {
            attempts: 3,
            base_backoff_ms: 0,
            max_backoff_ms: 0,
        };
        let result: Result<(), OpenAiError> = retry_transient_router_io_with_policy(policy, || {
            attempts += 1;
            Err(OpenAiError::HttpStatus(502))
        });

        assert!(result.is_err());
        assert_eq!(attempts, 3);
    }

    #[test]
    fn retry_policy_backoff_caps_exponential_growth() {
        let policy = RouterRetryPolicy {
            attempts: 8,
            base_backoff_ms: 500,
            max_backoff_ms: 2_000,
        };

        assert_eq!(policy.backoff_ms(1), 500);
        assert_eq!(policy.backoff_ms(2), 1_000);
        assert_eq!(policy.backoff_ms(3), 2_000);
        assert_eq!(policy.backoff_ms(4), 2_000);
    }
}
