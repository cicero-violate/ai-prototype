//! Browser-router LLM turn adapter.
//!
//! Wraps the OpenAI-compatible browser-router HTTP endpoint as a typed LLM
//! capability activity.  `RouterClient` is the stateful turn executor;
//! `RouterTurnResult` is the typed output of one agent turn.
//!
//! Import path: `crate::capability::llm::browser_router`.

use std::env;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::capability::llm::openai::{
    OpenAiBrowserOptions, OpenAiChatRequest, OpenAiChatResponse, OpenAiClient, OpenAiConfig,
    OpenAiError,
};
use crate::capability::llm::sse::{decode_chunked_body, parse_sse_body, ChunkLogger};
use crate::capability::llm::transport::{
    chat_completions_path, parse_local_http_endpoint, LocalEndpointError,
};

const DEFAULT_TRANSIENT_ROUTER_ATTEMPTS: u32 = 8;
const DEFAULT_TRANSIENT_ROUTER_BACKOFF_MS: u64 = 500;
const DEFAULT_TRANSIENT_ROUTER_MAX_BACKOFF_MS: u64 = 4_000;

pub struct RouterClient {
    inner: OpenAiClient,
    /// The browser-router/CDP target id returned by the last turn.
    target_id: Option<String>,
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
            target_id: None,
            target_url: None,
        })
    }

    pub fn from_env() -> Result<Self, OpenAiError> {
        Ok(Self {
            inner: OpenAiClient::from_env()?,
            target_id: None,
            target_url: None,
        })
    }

    pub fn target_url(&self) -> Option<&str> {
        self.target_url.as_deref()
    }

    pub fn target_id(&self) -> Option<&str> {
        self.target_id.as_deref()
    }

    /// Drop the current tab URL; the next turn will open a new chat.
    pub fn reset_session(&mut self) {
        self.target_url = None;
        self.target_id = None;
    }

    /// Best-effort close of the currently pinned browser tab.
    ///
    /// This uses browser-router's public tab API (`GET /tabs` and
    /// `DELETE /tabs/{target_id}`) on the configured OpenAI-compatible router
    /// base URL.
    pub fn close_current_tab(&mut self) -> Result<RouterTabCloseOutcome, OpenAiError> {
        if self.target_id.is_none() && self.target_url.is_none() {
            return Ok(RouterTabCloseOutcome::NoTarget);
        }

        let router_base_url = self.inner.config().base_url.clone();
        let timeout_ms = self.inner.config().timeout_ms;
        let outcome = if let Some(target_id) = self.target_id.as_deref() {
            retry_transient_router_io(|| {
                close_browser_tab_for_target_id(target_id, &router_base_url, timeout_ms)
            })?
        } else if let Some(target_url) = self.target_url.as_deref() {
            retry_transient_router_io(|| {
                close_browser_tab_for_url(target_url, &router_base_url, timeout_ms)
            })?
        } else {
            RouterTabCloseOutcome::NoTarget
        };
        if matches!(outcome, RouterTabCloseOutcome::Closed) {
            self.target_url = None;
            self.target_id = None;
        }
        Ok(outcome)
    }

    fn adopt_target(&mut self, next_id: Option<String>, next_url: Option<String>) {
        if let Some(next_id) = next_id {
            self.target_id = Some(next_id);
        }
        let Some(next_url) = next_url else {
            return;
        };

        if self
            .target_url
            .as_deref()
            .is_some_and(|previous_url| target_url_changed(previous_url, &next_url))
        {
            eprintln!(
                "agent: browser target changed; deferring stale-tab cleanup to explicit close path"
            );
        }

        self.target_url = Some(next_url);
    }

    /// Send one turn, threading target_url automatically.
    pub fn turn(&mut self, request: OpenAiChatRequest) -> Result<RouterTurnResult, OpenAiError> {
        let browser = match &self.target_url {
            Some(url) => OpenAiBrowserOptions::continue_at(url.clone()),
            None => OpenAiBrowserOptions::new_chat(),
        };
        let request = request.with_browser(browser);
        let response = retry_transient_router_io(|| self.inner.chat_with_request(&request))?;
        let target_url = response.target_url.clone();
        self.adopt_target(None, target_url.clone());
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

        eprintln!(
            "agent: router stream finalized complete={} reason={} target_url_present={}",
            sse.is_complete(),
            sse.completion_reason(),
            sse.target_url.is_some()
        );
        self.adopt_target(sse.target_id.clone(), sse.target_url.clone());
        eprintln!("agent: router target adopted");

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

fn target_url_changed(previous_url: &str, next_url: &str) -> bool {
    previous_url != next_url
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
        attempts: env_parsed::<u32>(
            "CANON_ROUTER_TRANSIENT_ATTEMPTS",
            DEFAULT_TRANSIENT_ROUTER_ATTEMPTS,
        )
        .max(1),
        base_backoff_ms: env_parsed::<u64>(
            "CANON_ROUTER_TRANSIENT_BACKOFF_MS",
            DEFAULT_TRANSIENT_ROUTER_BACKOFF_MS,
        ),
        max_backoff_ms: env_parsed::<u64>(
            "CANON_ROUTER_TRANSIENT_MAX_BACKOFF_MS",
            DEFAULT_TRANSIENT_ROUTER_MAX_BACKOFF_MS,
        ),
    }
}

fn env_parsed<T>(name: &str, default: T) -> T
where
    T: std::str::FromStr,
{
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
        OpenAiError::InvalidConfig(_)
        | OpenAiError::InvalidUrl
        | OpenAiError::InvalidResponse
        | OpenAiError::InvalidReceipt
        | OpenAiError::InvalidReceiptRecord
        | OpenAiError::InvalidReplay
        | OpenAiError::BudgetExhausted
        | OpenAiError::DuplicateRequest => false,
    }
}

/// Called from a background thread in loop_driver. No retry, hard 8s cap.
pub fn close_tab_for_url_with_timeout(
    target_url: &str,
    timeout_ms: u64,
) -> Result<RouterTabCloseOutcome, OpenAiError> {
    let config = OpenAiConfig::from_env()?;
    close_browser_tab_for_url(target_url, &config.base_url, timeout_ms)
}

pub fn close_tab_for_target_id_with_timeout(
    target_id: &str,
    timeout_ms: u64,
) -> Result<RouterTabCloseOutcome, OpenAiError> {
    let config = OpenAiConfig::from_env()?;
    close_browser_tab_for_target_id(target_id, &config.base_url, timeout_ms)
}

fn close_browser_tab_for_target_id(
    target_id: &str,
    router_base_url: &str,
    timeout_ms: u64,
) -> Result<RouterTabCloseOutcome, OpenAiError> {
    let endpoint = parse_local_http_endpoint(router_base_url).map_err(|e| match e {
        LocalEndpointError::InvalidUrl => OpenAiError::InvalidUrl,
        LocalEndpointError::NonLocalHost => {
            OpenAiError::InvalidConfig("browser-router url must be local")
        }
    })?;
    close_browser_router_tab(
        &endpoint.host,
        endpoint.port,
        &endpoint.path_prefix,
        target_id,
        timeout_ms,
    )
}

fn close_browser_tab_for_url(
    target_url: &str,
    router_base_url: &str,
    timeout_ms: u64,
) -> Result<RouterTabCloseOutcome, OpenAiError> {
    let endpoint = parse_local_http_endpoint(router_base_url).map_err(|e| match e {
        LocalEndpointError::InvalidUrl => OpenAiError::InvalidUrl,
        LocalEndpointError::NonLocalHost => {
            OpenAiError::InvalidConfig("browser-router url must be local")
        }
    })?;
    if let Some((_, _, target_id)) = devtools_page_target(target_url) {
        return close_browser_router_tab(
            &endpoint.host,
            endpoint.port,
            &endpoint.path_prefix,
            &target_id,
            timeout_ms,
        );
    }

    let tabs_path = browser_router_path(&endpoint.path_prefix, "/tabs");
    let (status, body) = router_get(&endpoint.host, endpoint.port, &tabs_path, timeout_ms)?;
    if status != 200 {
        return Ok(RouterTabCloseOutcome::CloseHttpStatus(status));
    }

    let Some(target_id) = browser_router_target_id_for_url(&body, target_url) else {
        return Ok(RouterTabCloseOutcome::NoMatchingTarget);
    };
    close_browser_router_tab(
        &endpoint.host,
        endpoint.port,
        &endpoint.path_prefix,
        &target_id,
        timeout_ms,
    )
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

fn browser_router_target_id_for_url(tabs_body: &str, target_url: &str) -> Option<String> {
    let value: Value = serde_json::from_str(tabs_body).ok()?;
    let entries = value.get("targets")?.as_array()?;
    entries.iter().find_map(|entry| {
        let obj = entry.as_object()?;
        let id = obj.get("id")?.as_str()?;
        let url_matches = obj
            .get("url")
            .and_then(Value::as_str)
            .is_some_and(|url| url == target_url);
        url_matches.then(|| id.to_string())
    })
}

fn close_browser_router_tab(
    host: &str,
    port: u16,
    path_prefix: &str,
    target_id: &str,
    timeout_ms: u64,
) -> Result<RouterTabCloseOutcome, OpenAiError> {
    let path = browser_router_path(path_prefix, &format!("/tabs/{target_id}"));
    let (status, _) = router_delete(host, port, &path, timeout_ms)?;
    if status == 200 {
        Ok(RouterTabCloseOutcome::Closed)
    } else {
        Ok(RouterTabCloseOutcome::CloseHttpStatus(status))
    }
}

fn browser_router_path(path_prefix: &str, path: &str) -> String {
    let prefix = path_prefix.trim_end_matches('/');
    let router_prefix = prefix.strip_suffix("/v1").unwrap_or(prefix);
    let path = path.trim_start_matches('/');
    if router_prefix.is_empty() {
        format!("/{path}")
    } else {
        format!("{router_prefix}/{path}")
    }
}

fn router_get(
    host: &str,
    port: u16,
    path: &str,
    timeout_ms: u64,
) -> Result<(u16, String), OpenAiError> {
    let response = router_request_transport_response("GET", host, port, path, timeout_ms)?;
    parse_router_response(&response)
}

fn router_delete(
    host: &str,
    port: u16,
    path: &str,
    timeout_ms: u64,
) -> Result<(u16, String), OpenAiError> {
    let response = router_request_transport_response("DELETE", host, port, path, timeout_ms)?;
    parse_router_response(&response)
}

fn router_request_transport_response(
    method: &str,
    host: &str,
    port: u16,
    path: &str,
    timeout_ms: u64,
) -> Result<String, OpenAiError> {
    use std::net::ToSocketAddrs;
    let addr = (host, port)
        .to_socket_addrs()
        .map_err(OpenAiError::Io)?
        .next()
        .ok_or_else(|| {
            OpenAiError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cdp resolve failed",
            ))
        })?;
    let timeout = Duration::from_millis(timeout_ms);
    let mut stream = TcpStream::connect_timeout(&addr, timeout).map_err(OpenAiError::Io)?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(OpenAiError::Io)?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(OpenAiError::Io)?;

    let request = build_router_request(method, host, port, path);
    stream
        .write_all(request.as_bytes())
        .map_err(OpenAiError::Io)?;
    stream.flush().map_err(OpenAiError::Io)?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(OpenAiError::Io)?;
    Ok(response)
}

fn build_router_request(method: &str, host: &str, port: u16, path: &str) -> String {
    format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
    )
}

fn parse_router_response(response: &str) -> Result<(u16, String), OpenAiError> {
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
) -> Result<crate::capability::llm::sse::SseResult, OpenAiError> {
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
    let mut stream =
        open_streaming_http_stream(endpoint_host, endpoint_port, write_timeout_ms, request)?;

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
                    logger.write_entry(
                        "stream_done_detected",
                        &format!("\"total_byte_length\":{}", full_response.len()),
                    );
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

fn open_streaming_http_stream(
    endpoint_host: &str,
    endpoint_port: u16,
    write_timeout_ms: u64,
    request: &str,
) -> Result<TcpStream, OpenAiError> {
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
    Ok(stream)
}

fn finalize_streaming_response(
    full_response: &[u8],
    logger: &mut ChunkLogger,
) -> Result<crate::capability::llm::sse::SseResult, OpenAiError> {
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
    use std::net::TcpListener;
    use std::sync::{mpsc, Mutex, OnceLock};

    const ROUTER_RETRY_ENV_KEYS: [&str; 3] = [
        "CANON_ROUTER_TRANSIENT_ATTEMPTS",
        "CANON_ROUTER_TRANSIENT_BACKOFF_MS",
        "CANON_ROUTER_TRANSIENT_MAX_BACKOFF_MS",
    ];

    fn env_test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn restore_env(snapshot: &[(&str, Option<String>)]) {
        for (key, value) in snapshot {
            match value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
        }
    }

    #[test]
    fn devtools_page_target_extracts_local_target_id() {
        assert_eq!(
            devtools_page_target("http://127.0.0.1:9222/devtools/page/ABC123"),
            Some(("127.0.0.1".to_string(), 9222, "ABC123".to_string()))
        );
        assert_eq!(devtools_page_target("https://chatgpt.com/c/abc123"), None);
    }

    #[test]
    fn browser_router_target_id_matches_target_url_from_tabs_response() {
        let body = r#"{
          "targets": [
            {"id":"old","url":"https://example.invalid/old"},
            {"id":"target-1","url":"https://chatgpt.com/c/current"}
          ]
        }"#;
        assert_eq!(
            browser_router_target_id_for_url(body, "https://chatgpt.com/c/current"),
            Some("target-1".to_string())
        );
        assert_eq!(
            browser_router_target_id_for_url(body, "https://missing.invalid"),
            None
        );
    }

    #[test]
    fn browser_router_helpers_preserve_http_request_and_response_parsing() {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .expect("loopback listener binds for browser-router helper test");
        let port = listener
            .local_addr()
            .expect("loopback listener exposes local address")
            .port();
        let (tx, rx) = mpsc::channel();
        let expected_request = build_router_request("GET", "127.0.0.1", port, "/tabs");

        assert_eq!(
            expected_request,
            format!(
                "GET /tabs HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAccept: application/json\r\nConnection: close\r\n\r\n"
            )
        );

        let server = thread::spawn(move || {
            let (mut stream, _) = listener
                .accept()
                .expect("loopback listener accepts browser-router connection");
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("loopback server read timeout is configured");

            let mut request = String::new();
            let mut buffer = [0_u8; 1024];
            loop {
                let read = stream
                    .read(&mut buffer)
                    .expect("loopback server reads browser-router request bytes");
                if read == 0 {
                    break;
                }
                request.push_str(
                    std::str::from_utf8(&buffer[..read])
                        .expect("browser-router request bytes are utf8"),
                );
                if request.contains("\r\n\r\n") {
                    break;
                }
            }
            tx.send(request)
                .expect("loopback server reports browser-router request");
            stream
                .write_all(b"HTTP/1.1 201 Created\r\nContent-Length: 16\r\nConnection: close\r\n\r\n{\"targets\":[]}")
                .expect("loopback server writes browser-router response");
        });

        let raw_response =
            router_request_transport_response("GET", "127.0.0.1", port, "/tabs", 1_000)
                .expect("browser-router transport helper reads deterministic loopback response");
        let (status, body) = parse_router_response(&raw_response)
            .expect("browser-router response parser reads deterministic loopback response");

        let observed = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("loopback server reports observed browser-router request");
        assert_eq!(observed, expected_request);
        let mut lines = observed.lines();
        assert_eq!(lines.next(), Some("GET /tabs HTTP/1.1"));
        let headers: Vec<&str> = lines.collect();
        assert!(headers.contains(&format!("Host: 127.0.0.1:{port}").as_str()));
        assert!(headers.contains(&"Accept: application/json"));
        assert!(headers.contains(&"Connection: close"));
        assert!(raw_response.starts_with("HTTP/1.1 201 Created\r\n"));
        assert_eq!(status, 201);
        assert_eq!(body, "{\"targets\":[]}");
        server
            .join()
            .expect("loopback browser-router server thread finishes without panic");
    }

    #[test]
    fn close_tab_uses_browser_router_tabs_api() {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .expect("loopback listener binds for browser-router close test");
        let port = listener
            .local_addr()
            .expect("loopback listener exposes local address")
            .port();
        let (tx, rx) = mpsc::channel();

        let server = thread::spawn(move || {
            for response_body in [
                r#"{"targets":[{"id":"target-1","url":"https://chatgpt.com/c/current"}]}"#,
                r#"{"ok":true,"object":"tab.closed","id":"target-1","message":"Target is closing"}"#,
            ] {
                let (mut stream, _) = listener
                    .accept()
                    .expect("loopback listener accepts browser-router request");
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .expect("loopback server read timeout is configured");

                let mut request = String::new();
                let mut buffer = [0_u8; 1024];
                loop {
                    let read = stream
                        .read(&mut buffer)
                        .expect("loopback server reads browser-router request bytes");
                    if read == 0 {
                        break;
                    }
                    request.push_str(
                        std::str::from_utf8(&buffer[..read])
                            .expect("browser-router request bytes are utf8"),
                    );
                    if request.contains("\r\n\r\n") {
                        break;
                    }
                }
                tx.send(request)
                    .expect("loopback server reports browser-router request");
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response_body}",
                    response_body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("loopback server writes browser-router response");
            }
        });

        let outcome = close_browser_tab_for_url(
            "https://chatgpt.com/c/current",
            &format!("http://127.0.0.1:{port}/v1"),
            1_000,
        )
        .expect("browser-router tab close request succeeds");
        assert_eq!(outcome, RouterTabCloseOutcome::Closed);

        let list_request = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("loopback server reports list request");
        let close_request = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("loopback server reports close request");
        assert!(list_request.starts_with("GET /tabs HTTP/1.1\r\n"));
        assert!(close_request.starts_with("DELETE /tabs/target-1 HTTP/1.1\r\n"));
        server
            .join()
            .expect("loopback browser-router server thread finishes without panic");
    }

    #[test]
    fn target_url_changed_only_for_distinct_urls() {
        assert!(!target_url_changed(
            "https://chatgpt.com/c/current",
            "https://chatgpt.com/c/current"
        ));
        assert!(target_url_changed(
            "https://chatgpt.com/c/old",
            "https://chatgpt.com/c/new"
        ));
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
    fn open_streaming_http_stream_sends_exact_request_to_loopback_listener() {
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .expect("loopback listener binds for router stream helper test");
        let port = listener
            .local_addr()
            .expect("loopback listener exposes local address")
            .port();
        let request = build_streaming_http_request(
            "/v1/chat/completions",
            "127.0.0.1",
            port,
            r#"{\"model\":\"gpt-test\",\"stream\":true}"#,
        );
        let expected_request = request.as_bytes().to_vec();
        let expected_len = expected_request.len();
        let (tx, rx) = mpsc::channel();

        let server = thread::spawn(move || {
            let (mut stream, _) = listener
                .accept()
                .expect("loopback listener accepts helper connection");
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .expect("loopback server read timeout is configured");

            let mut observed = vec![0_u8; expected_len];
            stream
                .read_exact(&mut observed)
                .expect("loopback server reads exact helper request bytes");
            tx.send(observed)
                .expect("loopback server sends observed request bytes");
        });

        let stream = open_streaming_http_stream("127.0.0.1", port, 1_000, &request)
            .expect("stream helper connects to loopback listener and flushes request");
        drop(stream);

        let observed = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("loopback server reports observed request bytes");
        assert_eq!(observed, expected_request);
        server
            .join()
            .expect("loopback server thread finishes without panic");
    }

    #[test]
    fn finalize_streaming_response_accepts_in_memory_sse_done_response() {
        let _temp_dir_guard = tempfile::Builder::new()
            .prefix("canon-router-finalize-success-")
            .tempdir_in({
                let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
                std::fs::create_dir_all(&d).unwrap();
                d
            })
            .unwrap();
        let temp_dir = _temp_dir_guard.path().to_path_buf();
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

    }

    #[test]
    fn finalize_streaming_response_rejects_non_200_status() {
        let _temp_dir_guard = tempfile::Builder::new()
            .prefix("canon-router-finalize-http-status-")
            .tempdir_in({
                let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
                std::fs::create_dir_all(&d).unwrap();
                d
            })
            .unwrap();
        let temp_dir = _temp_dir_guard.path().to_path_buf();
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

    }

    #[test]
    fn finalize_streaming_response_rejects_missing_done_frame() {
        let _temp_dir_guard = tempfile::Builder::new()
            .prefix("canon-router-finalize-missing-done-")
            .tempdir_in({
                let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
                std::fs::create_dir_all(&d).unwrap();
                d
            })
            .unwrap();
        let temp_dir = _temp_dir_guard.path().to_path_buf();
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
            other @ OpenAiError::InvalidConfig(_)
            | other @ OpenAiError::InvalidUrl
            | other @ OpenAiError::HttpStatus(_)
            | other @ OpenAiError::InvalidResponse
            | other @ OpenAiError::InvalidReceipt
            | other @ OpenAiError::InvalidReceiptRecord
            | other @ OpenAiError::InvalidReplay
            | other @ OpenAiError::BudgetExhausted
            | other @ OpenAiError::DuplicateRequest => {
                panic!("expected missing [DONE] to return UnexpectedEof, got {other:?}")
            }
        }

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

        assert_eq!(result.expect("test value should be present"), "ok");
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

    #[test]
    fn router_retry_policy_from_env_preserves_typed_numeric_defaults() {
        let _guard = env_test_lock()
            .lock()
            .expect("env test lock should not be poisoned");
        let snapshot: Vec<_> = ROUTER_RETRY_ENV_KEYS
            .iter()
            .map(|key| (*key, env::var(key).ok()))
            .collect();

        for key in ROUTER_RETRY_ENV_KEYS {
            env::remove_var(key);
        }

        env::set_var("CANON_ROUTER_TRANSIENT_ATTEMPTS", "11");
        env::set_var("CANON_ROUTER_TRANSIENT_BACKOFF_MS", "1200");
        env::set_var("CANON_ROUTER_TRANSIENT_MAX_BACKOFF_MS", "3400");

        let parsed = router_retry_policy();
        assert_eq!(parsed.attempts, 11);
        assert_eq!(parsed.base_backoff_ms, 1200);
        assert_eq!(parsed.max_backoff_ms, 3400);

        env::set_var("CANON_ROUTER_TRANSIENT_ATTEMPTS", "not-a-u32");
        env::set_var("CANON_ROUTER_TRANSIENT_BACKOFF_MS", "not-a-u64");
        env::set_var("CANON_ROUTER_TRANSIENT_MAX_BACKOFF_MS", "not-a-u64");

        let defaults = router_retry_policy();
        assert_eq!(defaults.attempts, DEFAULT_TRANSIENT_ROUTER_ATTEMPTS);
        assert_eq!(
            defaults.base_backoff_ms,
            DEFAULT_TRANSIENT_ROUTER_BACKOFF_MS
        );
        assert_eq!(
            defaults.max_backoff_ms,
            DEFAULT_TRANSIENT_ROUTER_MAX_BACKOFF_MS
        );

        env::set_var("CANON_ROUTER_TRANSIENT_ATTEMPTS", "0");
        let lower_bound = router_retry_policy();
        assert_eq!(lower_bound.attempts, 1);

        restore_env(&snapshot);
    }
}
