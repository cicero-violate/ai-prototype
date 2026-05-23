//! MCP workspace synchronization helpers.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::time::Duration;

/// POST {mcp_url}/ai/workspace with the project root. Retries while the supervisor is starting.
pub(super) fn sync_mcp_workspace(mcp_url: &str, project_dir: &Path) -> Result<(), String> {
    const MAX_ATTEMPTS: u32 = 12;
    const RETRY_MS: u64 = 2_500;

    let (host, port) = parse_mcp_workspace_endpoint(mcp_url)?;
    let request = build_mcp_workspace_request(&host, port, project_dir);

    let mut last_err = String::new();
    for attempt in 1..=MAX_ATTEMPTS {
        match send_mcp_workspace_request(&host, port, &request) {
            Ok(response) => {
                let status = parse_mcp_workspace_status(&response);
                if status == 200 || status == 201 {
                    return Ok(());
                }
                return Err(format!("MCP workspace sync returned HTTP {status}"));
            }
            Err(e) => {
                last_err = e;
                if attempt < MAX_ATTEMPTS {
                    eprintln!(
                        "agent: MCP workspace sync attempt {attempt}/{MAX_ATTEMPTS} failed: {last_err} — retrying in {RETRY_MS}ms"
                    );
                    std::thread::sleep(Duration::from_millis(RETRY_MS));
                }
            }
        }
    }
    Err(format!(
        "MCP workspace sync failed after {MAX_ATTEMPTS} attempts: {last_err}"
    ))
}

pub(super) fn send_mcp_workspace_request(
    host: &str,
    port: u16,
    request: &str,
) -> Result<String, String> {
    let mut stream = TcpStream::connect((host, port))
        .map_err(|e| format!("MCP connect failed ({host}:{port}): {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();

    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("MCP request write: {e}"))?;
    stream.flush().ok();

    let mut response = String::new();
    stream.read_to_string(&mut response).ok();
    Ok(response)
}

pub(super) fn parse_mcp_workspace_endpoint(mcp_url: &str) -> Result<(String, u16), String> {
    let url = mcp_url.trim_end_matches('/');
    let host_port = url
        .strip_prefix("http://")
        .ok_or_else(|| format!("MCP_CONNECTOR_URL must start with http://: {url}"))?;

    if let Some(colon_pos) = host_port.rfind(':') {
        let h = &host_port[..colon_pos];
        let p: u16 = host_port[colon_pos + 1..]
            .parse()
            .map_err(|_| format!("invalid port in MCP_CONNECTOR_URL: {url}"))?;
        Ok((h.to_string(), p))
    } else {
        Ok((host_port.to_string(), 80u16))
    }
}

pub(super) fn parse_mcp_workspace_status(response: &str) -> u16 {
    response
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse().ok())
        .unwrap_or(0)
}

pub(super) fn build_mcp_workspace_request(host: &str, port: u16, project_dir: &Path) -> String {
    let root = project_dir.to_string_lossy();
    let body = format!("{{\"root\":\"{root}\"}}");

    format!(
        "POST /ai/workspace HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {len}\r\n\r\n{body}",
        len = body.len(),
    )
}

// ── Learning loop helpers ─────────────────────────────────────────────────────
