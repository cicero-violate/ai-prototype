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
const AI_WORKER_PORT_ENV: &str = "AI_WORKER_PORT";

#[derive(Debug)]
pub struct WorkerClient {
    port: u16,
    timeout: Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerClientConfig {
    port: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkerClientConfigError {
    MissingWorkerPort,
    InvalidWorkerPort { raw: String },
}

impl WorkerClientConfig {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn from_env() -> Result<Self, WorkerClientConfigError> {
        let raw = std::env::var(AI_WORKER_PORT_ENV)
            .map_err(|_| WorkerClientConfigError::MissingWorkerPort)?;
        let port = raw
            .parse::<u16>()
            .map_err(|_| WorkerClientConfigError::InvalidWorkerPort { raw })?;
        Ok(Self::new(port))
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

impl std::fmt::Display for WorkerClientConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingWorkerPort => write!(f, "{AI_WORKER_PORT_ENV} is required"),
            Self::InvalidWorkerPort { raw } => {
                write!(f, "{AI_WORKER_PORT_ENV} must be a u16, got {raw:?}")
            }
        }
    }
}

impl std::error::Error for WorkerClientConfigError {}

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
        Self::from_timeout_ms(port, DEFAULT_TIMEOUT_MS)
    }

    pub fn new_with_timeout(port: u16, timeout_ms: u64) -> Self {
        Self::from_timeout_ms(port, timeout_ms)
    }

    fn from_timeout_ms(port: u16, timeout_ms: u64) -> Self {
        Self::from_timeout(port, Duration::from_millis(timeout_ms))
    }

    fn from_timeout(port: u16, timeout: Duration) -> Self {
        Self { port, timeout }
    }

    pub fn from_config(config: WorkerClientConfig) -> Self {
        Self::new(config.port())
    }

    pub fn from_env() -> Result<Self, WorkerClientConfigError> {
        let config = WorkerClientConfig::from_env()?;
        Ok(Self::from_config(config))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_test_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_worker_port_env<T>(value: Option<&str>, test: impl FnOnce() -> T) -> T {
        let _guard = env_test_lock()
            .lock()
            .expect("env test lock should not be poisoned");
        let snapshot = std::env::var(AI_WORKER_PORT_ENV).ok();
        match value {
            Some(value) => std::env::set_var(AI_WORKER_PORT_ENV, value),
            None => std::env::remove_var(AI_WORKER_PORT_ENV),
        }

        let result = test();

        match snapshot {
            Some(value) => std::env::set_var(AI_WORKER_PORT_ENV, value),
            None => std::env::remove_var(AI_WORKER_PORT_ENV),
        }

        result
    }

    #[test]
    fn worker_client_constructors_preserve_default_and_custom_timeouts() {
        let default_client = WorkerClient::new(8123);
        assert_eq!(default_client.port, 8123);
        assert_eq!(
            default_client.timeout,
            Duration::from_millis(DEFAULT_TIMEOUT_MS)
        );

        let custom_client = WorkerClient::new_with_timeout(8124, 123);
        assert_eq!(custom_client.port, 8124);
        assert_eq!(custom_client.timeout, Duration::from_millis(123));

        assert_ne!(default_client.port, custom_client.port);
        assert_ne!(default_client.timeout, custom_client.timeout);
    }

    #[test]
    fn worker_client_constructor_helper_preserves_timeout_boundaries() {
        let default_client = WorkerClient::new(8123);
        let default_helper_client = WorkerClient::from_timeout_ms(8123, DEFAULT_TIMEOUT_MS);
        assert_eq!(default_client.port, 8123);
        assert_eq!(
            default_client.timeout,
            Duration::from_millis(DEFAULT_TIMEOUT_MS)
        );
        assert_eq!(default_client.port, default_helper_client.port);
        assert_eq!(default_client.timeout, default_helper_client.timeout);

        let custom_client = WorkerClient::new_with_timeout(8124, 123);
        let custom_helper_client = WorkerClient::from_timeout_ms(8124, 123);
        assert_eq!(custom_client.port, 8124);
        assert_eq!(custom_client.timeout, Duration::from_millis(123));
        assert_eq!(custom_client.port, custom_helper_client.port);
        assert_eq!(custom_client.timeout, custom_helper_client.timeout);

        let zero_timeout_client = WorkerClient::new_with_timeout(8125, 0);
        assert_eq!(zero_timeout_client.port, 8125);
        assert_eq!(zero_timeout_client.timeout, Duration::from_millis(0));

        assert_ne!(default_client.port, custom_client.port);
        assert_ne!(default_client.timeout, custom_client.timeout);
        assert_ne!(custom_client.port, zero_timeout_client.port);
        assert_ne!(custom_client.timeout, zero_timeout_client.timeout);
    }

    #[test]
    fn worker_client_from_config_constructs_client_from_typed_config() {
        let config = WorkerClientConfig::new(8126);
        let client = WorkerClient::from_config(config);
        assert_eq!(client.port, 8126);
        assert_eq!(client.timeout, Duration::from_millis(DEFAULT_TIMEOUT_MS));
    }

    #[test]
    fn worker_client_from_env_constructs_client_from_valid_config() {
        with_worker_port_env(Some("8127"), || {
            let client = WorkerClient::from_env().expect("valid worker port should configure client");
            assert_eq!(client.port, 8127);
            assert_eq!(client.timeout, Duration::from_millis(DEFAULT_TIMEOUT_MS));
        });
    }

    #[test]
    fn worker_client_from_env_returns_typed_missing_config_error() {
        with_worker_port_env(None, || {
            let err = WorkerClient::from_env().expect_err("missing port should be typed config error");
            assert_eq!(err, WorkerClientConfigError::MissingWorkerPort);
        });
    }

    #[test]
    fn worker_client_from_env_returns_typed_invalid_config_error() {
        with_worker_port_env(Some("not-a-port"), || {
            let err = WorkerClient::from_env().expect_err("invalid port should be typed config error");
            assert_eq!(
                err,
                WorkerClientConfigError::InvalidWorkerPort {
                    raw: "not-a-port".to_string(),
                }
            );
        });
    }
}
