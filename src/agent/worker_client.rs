//! Blocking HTTP client for the ai worker API.
//!
//! Routes served by the worker:
//!   GET  /health/worker    → "ok" (200)
//!   GET  /v1/state         → StateDto (JSON)
//!   POST /v1/command       → CommandResponseDto (JSON)
//!
//! Uses std::net::TcpStream (blocking) consistent with openai.rs.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

const DEFAULT_TIMEOUT_MS: u64 = 5_000;

pub struct WorkerClient {
    port: u16,
    timeout: Duration,
}

#[derive(Debug)]
pub enum WorkerClientError {
    Connect(String),
    Io(String),
    HttpError(u16),
    InvalidResponse(String),
}

impl std::fmt::Display for WorkerClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect(e) => write!(f, "worker connect: {e}"),
            Self::Io(e) => write!(f, "worker io: {e}"),
            Self::HttpError(s) => write!(f, "worker http {s}"),
            Self::InvalidResponse(e) => write!(f, "worker invalid response: {e}"),
        }
    }
}

pub struct WorkerResponse {
    pub status: u16,
    pub body: String,
}

impl WorkerClient {
    pub fn new(port: u16) -> Self {
        Self::new_with_timeout(port, DEFAULT_TIMEOUT_MS)
    }

    pub fn new_with_timeout(port: u16, timeout_ms: u64) -> Self {
        Self {
            port,
            timeout: Duration::from_millis(timeout_ms),
        }
    }

    pub fn from_env() -> Result<Self, WorkerClientError> {
        let raw = std::env::var("AI_WORKER_PORT")
            .map_err(|_| WorkerClientError::Connect("AI_WORKER_PORT not set".into()))?;
        let port: u16 = raw
            .parse()
            .map_err(|_| WorkerClientError::Connect(format!("invalid AI_WORKER_PORT: {raw}")))?;
        Ok(Self::new(port))
    }

    pub fn health(&self) -> Result<bool, WorkerClientError> {
        let resp = self.get("/health/worker")?;
        Ok(resp.status == 200)
    }

    pub fn state(&self) -> Result<WorkerResponse, WorkerClientError> {
        self.get("/v1/state")
    }

    pub fn submit_command(&self, body_json: &str) -> Result<WorkerResponse, WorkerClientError> {
        self.post("/v1/command", body_json)
    }

    fn get(&self, path: &str) -> Result<WorkerResponse, WorkerClientError> {
        let port = self.port;
        let request =
            format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n");
        self.send(request.as_bytes())
    }

    fn post(&self, path: &str, body: &str) -> Result<WorkerResponse, WorkerClientError> {
        let port = self.port;
        let request = format!(
            "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n\
             Content-Type: application/json\r\nContent-Length: {}\r\n\
             Connection: close\r\n\r\n{body}",
            body.len()
        );
        self.send(request.as_bytes())
    }

    fn send(&self, request: &[u8]) -> Result<WorkerResponse, WorkerClientError> {
        let addr: SocketAddr = format!("127.0.0.1:{}", self.port)
            .parse()
            .map_err(|e| WorkerClientError::Connect(format!("{e}")))?;
        let stream = TcpStream::connect_timeout(&addr, self.timeout)
            .map_err(|e| WorkerClientError::Connect(format!("{e}")))?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|e| WorkerClientError::Io(format!("{e}")))?;
        let mut stream = stream;
        stream
            .write_all(request)
            .map_err(|e| WorkerClientError::Io(format!("{e}")))?;
        let mut bytes = Vec::new();
        stream
            .read_to_end(&mut bytes)
            .map_err(|e| WorkerClientError::Io(format!("{e}")))?;
        parse_response(bytes)
    }
}

fn parse_response(bytes: Vec<u8>) -> Result<WorkerResponse, WorkerClientError> {
    let split = bytes
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| WorkerClientError::InvalidResponse("no header/body separator".into()))?;
    let head = String::from_utf8_lossy(&bytes[..split]);
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|s| s.parse::<u16>().ok())
        .ok_or_else(|| WorkerClientError::InvalidResponse("missing status line".into()))?;
    let body = String::from_utf8_lossy(&bytes[split + 4..]).into_owned();
    Ok(WorkerResponse { status, body })
}
