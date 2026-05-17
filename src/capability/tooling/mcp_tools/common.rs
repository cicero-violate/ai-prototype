//! Shared helpers for runtime/file/process/agent MCP tools.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::capability::tooling::SandboxProcessReceipt;

pub fn tool_error(message: String) -> Value {
    json!({ "content": [{ "type": "text", "text": format!("Error: {message}") }], "isError": true })
}

pub fn tmp_dir() -> Result<PathBuf, String> {
    let path = env::temp_dir().join("canon-ai-mcp");
    fs::create_dir_all(&path)
        .map_err(|error| format!("create temp dir {}: {error}", path.display()))?;
    Ok(path)
}

pub fn native_process_output_paths(
    root: &Path,
    receipt: &SandboxProcessReceipt,
) -> (PathBuf, PathBuf) {
    let dir = root.join("process");
    (
        dir.join(format!("{:016x}.stdout", receipt.request_hash)),
        dir.join(format!("{:016x}.stderr", receipt.request_hash)),
    )
}
