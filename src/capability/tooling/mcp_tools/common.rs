//! Shared helpers for runtime/file/process/agent MCP tools.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::capability::tooling::SandboxProcessReceipt;
use crate::runtime::workspace::workspace_state_dir;

pub fn tool_error(message: String) -> Value {
    json!({ "content": [{ "type": "text", "text": format!("Error: {message}") }], "isError": true })
}

pub fn tmp_dir(workspace_root: &Path) -> Result<PathBuf, String> {
    let path = workspace_state_dir(workspace_root)
        .join("tmp")
        .join("canon-ai-mcp");
    fs::create_dir_all(&path)
        .map_err(|error| format!("create temp dir {}: {error}", path.display()))?;
    Ok(path)
}

pub fn native_process_output_paths(
    root: &Path,
    receipt: &SandboxProcessReceipt,
) -> (PathBuf, PathBuf) {
    let dir = workspace_state_dir(root).join("process");
    (
        dir.join(format!("{:016x}.stdout", receipt.request_hash)),
        dir.join(format!("{:016x}.stderr", receipt.request_hash)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("canon-ai-mcp-common-{name}-{}", std::process::id()))
    }

    #[test]
    fn tmp_dir_uses_workspace_state_tmp() {
        let workspace = unique_test_dir("workspace-state");
        let _ = fs::remove_dir_all(&workspace);
        fs::create_dir_all(&workspace).expect("workspace dir");

        let result = tmp_dir(&workspace).expect("workspace tmp dir");

        assert_eq!(result, workspace.join("state/tmp/canon-ai-mcp"));
        assert!(result.is_dir());

        let _ = fs::remove_dir_all(&workspace);
    }
}
