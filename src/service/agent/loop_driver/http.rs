//! Local HTTP helpers for agent loop receipts.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::service::agent::config::AgentLoopConfig;

use super::mcp_workspace::parse_mcp_workspace_status;
use super::receipt::{agent_turn_kernel_command, AgentTurnStatus};

pub(crate) fn agent_command_url(config: &AgentLoopConfig) -> Option<String> {
    config
        .supervisor_port
        .map(|port| format!("http://127.0.0.1:{port}/v1/command"))
        .or_else(|| {
            std::env::var("SUPERVISOR_PORT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .map(|port| format!("http://127.0.0.1:{port}/v1/command"))
        })
        .or_else(|| {
            config
                .worker_port
                .map(|port| format!("http://127.0.0.1:{port}/v1/command"))
        })
        .or_else(|| {
            std::env::var("AI_WORKER_PORT")
                .ok()
                .and_then(|value| value.parse::<u16>().ok())
                .map(|port| format!("http://127.0.0.1:{port}/v1/command"))
        })
}

pub(super) fn submit_agent_turn_receipt(
    command_url: &str,
    receipt: &serde_json::Value,
    status: AgentTurnStatus,
) {
    let command = match serde_json::to_value(agent_turn_kernel_command(receipt, status)) {
        Ok(command) => command,
        Err(err) => {
            eprintln!(
                "agent: failed to serialize turn receipt kernel command url={command_url} error={err}"
            );
            return;
        }
    };
    match post_json_local(command_url, &command) {
        Ok(status) if (200..300).contains(&status) => {
            eprintln!(
                "agent: submitted turn receipt to kernel url={} status={}",
                command_url, status
            );
        }
        Ok(status) => {
            eprintln!(
                "agent: failed to submit turn receipt to kernel url={} status={}",
                command_url, status
            );
        }
        Err(err) => {
            eprintln!(
                "agent: failed to submit turn receipt to kernel url={command_url} error={err}"
            );
        }
    }
}

pub(crate) fn get_json_body_local(url: &str) -> Result<serde_json::Value, String> {
    let (host, port, path) = parse_local_http_url(url)?;
    let mut stream =
        TcpStream::connect((host.as_str(), port)).map_err(|e| format!("connect: {e}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| format!("set timeout: {e}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(3)))
        .map_err(|e| format!("set timeout: {e}"))?;
    let request =
        format!("GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    stream.flush().map_err(|e| format!("flush: {e}"))?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|e| format!("read: {e}"))?;
    let body = response.split("\r\n\r\n").nth(1).unwrap_or(&response);
    let body = body.trim();
    if body.is_empty() {
        return Ok(serde_json::Value::Null);
    }
    serde_json::from_str(body).map_err(|e| format!("json: {e}"))
}

pub(crate) fn post_json_local(url: &str, value: &serde_json::Value) -> Result<u16, String> {
    post_json_body_local(url, value).map(|(status, _)| status)
}

/// POST JSON and return both the HTTP status code and the parsed response body.
pub(crate) fn post_json_body_local(
    url: &str,
    value: &serde_json::Value,
) -> Result<(u16, serde_json::Value), String> {
    let (host, port, path) = parse_local_http_url(url)?;
    let body = value.to_string();
    let mut stream =
        TcpStream::connect((host.as_str(), port)).map_err(|err| format!("connect: {err}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|err| format!("set read timeout: {err}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|err| format!("set write timeout: {err}"))?;
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {len}\r\n\r\n{body}",
        len = body.len(),
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|err| format!("write: {err}"))?;
    stream.flush().map_err(|err| format!("flush: {err}"))?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(|err| format!("read: {err}"))?;
    let status = parse_mcp_workspace_status(&response);
    let body_str = response.split("\r\n\r\n").nth(1).unwrap_or("");
    let body_value = serde_json::from_str(body_str.trim()).unwrap_or(serde_json::Value::Null);
    Ok((status, body_value))
}

pub(super) fn parse_local_http_url(url: &str) -> Result<(String, u16, String), String> {
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| "url must start with http://".to_string())?;
    let (authority, path) = rest
        .split_once('/')
        .map(|(authority, path)| (authority, format!("/{path}")))
        .unwrap_or((rest, "/".to_string()));
    let (host, port) = authority
        .rsplit_once(':')
        .ok_or_else(|| "url must include host:port".to_string())?;
    if !matches!(host, "127.0.0.1" | "localhost") {
        return Err(format!("url host must be local, got {host}"));
    }
    let port = port
        .parse::<u16>()
        .map_err(|_| format!("invalid port in url: {url}"))?;
    Ok((host.to_string(), port, path))
}
