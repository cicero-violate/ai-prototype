//! RouterClient — wraps OpenAiClient and threads target_url across turns.
//!
//! The router-server maintains ChatGPT conversation continuity via the browser
//! tab URL. Each turn returns a target_url; the next turn must supply it via
//! OpenAiBrowserOptions::continue_at to stay in the same thread.

use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::agent::sse::{ChunkLogger, decode_chunked_body, parse_sse_body};
use crate::capability::llm::openai::{
    OpenAiBrowserOptions, OpenAiChatRequest, OpenAiChatResponse, OpenAiClient, OpenAiConfig,
    OpenAiError,
};
use crate::capability::llm::transport::{
    LocalEndpointError, chat_completions_path, parse_local_http_endpoint,
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
        Ok(RouterTurnResult { response, target_url })
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

        let sse = send_streaming_request(self.inner.config(), &body, logger)?;

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
        self.reason != "missing_message_stream_complete"
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
) -> Result<crate::agent::sse::SseResult, OpenAiError> {
    let endpoint = parse_local_http_endpoint(&config.base_url).map_err(|e| match e {
        LocalEndpointError::InvalidUrl => OpenAiError::InvalidUrl,
        LocalEndpointError::NonLocalHost => OpenAiError::InvalidConfig("base url must be local"),
    })?;

    let path = chat_completions_path(&endpoint.path_prefix);
    let mut stream = TcpStream::connect((endpoint.host.as_str(), endpoint.port))
        .map_err(OpenAiError::Io)?;
    stream
        .set_read_timeout(Some(Duration::from_millis(config.timeout_ms)))
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

    let mut full_response = String::new();
    match stream.read_to_string(&mut full_response) {
        Ok(_) => {}
        Err(e)
            if matches!(e.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) => {}
        Err(e) => return Err(OpenAiError::Io(e)),
    }

    logger.write_entry(
        "http_chunk",
        &format!("\"byte_length\":{}", full_response.len()),
    );

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
