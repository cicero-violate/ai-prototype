//! RouterClient — wraps OpenAiClient and threads target_url across turns.
//!
//! The router-server maintains ChatGPT conversation continuity via the browser
//! tab URL. Each turn returns a target_url; the next turn must supply it via
//! OpenAiBrowserOptions::continue_at to stay in the same thread.

use std::env;
use std::io::{self, Read, Write};
use std::net::TcpStream;
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

        let outcome = close_browser_tab_for_url(&target_url, self.inner.config().timeout_ms)?;
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
        let response = self.inner.chat_with_request(&request)?;
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
    let mut stream =
        TcpStream::connect((endpoint.host.as_str(), endpoint.port)).map_err(OpenAiError::Io)?;
    stream
        .set_read_timeout(Some(Duration::from_millis(1_000)))
        .map_err(OpenAiError::Io)?;
    stream
        .set_write_timeout(Some(Duration::from_millis(config.timeout_ms)))
        .map_err(OpenAiError::Io)?;

    let request_str = format!(
        "POST {path} HTTP/1.1\r\nHost: {}:{}\r\nContent-Type: application/json\r\nAccept: text/event-stream\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
        endpoint.host,
        endpoint.port,
        body.len(),
    );
    stream
        .write_all(request_str.as_bytes())
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

    let full_response = String::from_utf8_lossy(&full_response);

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
}
