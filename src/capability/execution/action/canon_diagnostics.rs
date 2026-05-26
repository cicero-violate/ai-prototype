//! Bounded runtime diagnostics for planner self-review.

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::runtime::WorkspaceView;

pub const CANON_DIAGNOSTICS_READ_TOOL: &str = "canon_diagnostics_read";

const DEFAULT_MAX_LINES: usize = 200;
const MAX_LINES_LIMIT: usize = 1_000;
const DEFAULT_ERROR_LIMIT: usize = 100;
const MAX_ERROR_LIMIT: usize = 500;

pub fn run_read(args: &Value, workspace: &WorkspaceView) -> Value {
    let max_lines = bounded_usize(args, "max_lines", DEFAULT_MAX_LINES, MAX_LINES_LIMIT);
    let error_limit = bounded_usize(args, "error_limit", DEFAULT_ERROR_LIMIT, MAX_ERROR_LIMIT);

    let state_dir = workspace.allowed_boundary.join("state");
    let tlog_dir = env::var("AI_TLOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| state_dir.join("tlog"));
    let console_log = state_dir.join("console.log");
    let actions_log = state_dir.join("actions.ndjson");
    let tlog_path = tlog_dir.join("canon-agent.tlog.ndjson");

    let console_tail = tail_lines(&console_log, max_lines);
    let action_errors = matching_tail_lines(&actions_log, error_limit, diagnostic_line);
    let tlog_errors = matching_tail_lines(&tlog_path, error_limit, diagnostic_line);

    ok(json!({
        "schema": "canon.diagnostics.v1",
        "bounded": {
            "max_lines": max_lines,
            "error_limit": error_limit
        },
        "workspace": {
            "root": workspace.root.display().to_string(),
            "allowed_boundary": workspace.allowed_boundary.display().to_string(),
            "state_dir": state_dir.display().to_string()
        },
        "runtime": runtime_config(),
        "logs": {
            "console": {
                "path": console_log.display().to_string(),
                "exists": console_log.exists(),
                "tail": console_tail
            },
            "actions": {
                "path": actions_log.display().to_string(),
                "exists": actions_log.exists(),
                "recent_diagnostics": action_errors
            },
            "tlog": {
                "path": tlog_path.display().to_string(),
                "exists": tlog_path.exists(),
                "recent_diagnostics": tlog_errors
            }
        }
    }))
}

fn runtime_config() -> Value {
    let keys = [
        "PROJECT_DIR",
        "MCP_CONNECTOR_URL",
        "AI_MCP_BASE_URL",
        "AI_MCP_WORKER_URL",
        "AI_TLOG_DIR",
        "CANON_OPENAI_BASE_URL",
        "OPENAI_BASE_URL",
        "EXECUTE_TURNS",
        "AGENT_COUNT",
        "CANON_EXECUTOR_COUNT",
        "TASK_RUNNER_ENABLED",
        "TASK_RUNNER_COUNT",
        "TASK_RUNNER_LEASE_TTL_MS",
    ];
    let mut values = serde_json::Map::new();
    for key in keys {
        values.insert(
            key.to_string(),
            env::var(key)
                .map(|value| json!(redact(&value)))
                .unwrap_or_else(|_| Value::Null),
        );
    }
    Value::Object(values)
}

fn tail_lines(path: &Path, max_lines: usize) -> Vec<String> {
    read_lines(path)
        .map(|lines| {
            let start = lines.len().saturating_sub(max_lines);
            lines[start..].iter().map(|line| redact(line)).collect()
        })
        .unwrap_or_default()
}

fn matching_tail_lines(path: &Path, max_lines: usize, predicate: fn(&str) -> bool) -> Vec<String> {
    let mut matches = read_lines(path)
        .unwrap_or_default()
        .into_iter()
        .filter(|line| predicate(line))
        .collect::<Vec<_>>();
    let start = matches.len().saturating_sub(max_lines);
    matches
        .drain(start..)
        .map(|line| redact(&line))
        .collect::<Vec<_>>()
}

fn read_lines(path: &Path) -> Result<Vec<String>, String> {
    let file = File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let reader = BufReader::new(file);
    reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read {}: {error}", path.display()))
}

fn diagnostic_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("error")
        || lower.contains("failed")
        || lower.contains("failure")
        || lower.contains("status=5")
        || lower.contains("\"status\":5")
        || lower.contains("connection refused")
        || lower.contains("temporarily unavailable")
        || lower.contains("payload_tag")
}

fn bounded_usize(args: &Value, key: &str, default: usize, max: usize) -> usize {
    args.get(key)
        .and_then(Value::as_u64)
        .map(|value| (value as usize).clamp(1, max))
        .unwrap_or(default)
}

fn redact(input: &str) -> String {
    input
        .split_whitespace()
        .map(redact_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn redact_token(token: &str) -> String {
    let lower = token.to_ascii_lowercase();
    let sensitive_key = lower.contains("key")
        || lower.contains("token")
        || lower.contains("secret")
        || lower.contains("authorization")
        || lower.contains("password");
    if sensitive_key {
        if let Some((key, _)) = token.split_once('=') {
            return format!("{key}=<redacted>");
        }
        if let Some((key, _)) = token.split_once(':') {
            return format!("{key}:<redacted>");
        }
        return "<redacted>".to_string();
    }
    if token.starts_with("sk-") || token.len() > 80 && token.chars().all(|c| c.is_ascii_graphic()) {
        return "<redacted>".to_string();
    }
    token.to_string()
}

fn ok(value: Value) -> Value {
    json!({ "ok": true, "result": value })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_usize_clamps_values() {
        assert_eq!(bounded_usize(&json!({"n": 0}), "n", 5, 10), 1);
        assert_eq!(bounded_usize(&json!({"n": 99}), "n", 5, 10), 10);
        assert_eq!(bounded_usize(&json!({}), "n", 5, 10), 5);
    }

    #[test]
    fn redacts_obvious_secret_tokens() {
        assert_eq!(
            redact("OPENAI_API_KEY=sk-test value"),
            "OPENAI_API_KEY=<redacted> value"
        );
        assert_eq!(
            redact("authorization:Bearer abc"),
            "authorization:<redacted> abc"
        );
    }
}
