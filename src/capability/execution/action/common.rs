//! Shared helpers for runtime/file/process/agent action tools.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::capability::execution::SandboxProcessReceipt;
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

    fn test_tmp_dir() -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
        fs::create_dir_all(&dir).unwrap();
        dir.canonicalize().unwrap()
    }

    fn unique_test_dir(name: &str) -> (PathBuf, tempfile::TempDir) {
        let tmp = tempfile::Builder::new()
            .prefix(&format!("canon-ai-mcp-common-{name}-"))
            .tempdir_in(test_tmp_dir())
            .expect("test dir creation should succeed");
        let dir = tmp.path().to_owned();
        (dir, tmp)
    }

    #[test]
    fn tmp_dir_uses_workspace_state_tmp() {
        let (workspace, _tmp) = unique_test_dir("workspace-state");

        let result = tmp_dir(&workspace).expect("workspace tmp dir");

        assert_eq!(result, workspace.join("state/tmp/canon-ai-mcp"));
        assert!(result.is_dir());
    }
}
