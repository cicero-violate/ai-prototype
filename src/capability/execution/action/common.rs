//! Shared helpers for runtime/file/process/agent action tools.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

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
    cleanup_tmp_dir_periodically(&path)?;
    Ok(path)
}

fn cleanup_tmp_dir_periodically(path: &Path) -> Result<(), String> {
    let interval = Duration::from_secs(300);
    let retention = Duration::from_secs(3600);
    let marker = path.join(".last-cleanup");
    if marker_is_fresh(&marker, interval) {
        return Ok(());
    }
    cleanup_tmp_dir_contents(path, retention)?;
    fs::write(&marker, b"")
        .map_err(|error| format!("write temp cleanup marker {}: {error}", marker.display()))
}

fn marker_is_fresh(marker: &Path, interval: Duration) -> bool {
    marker
        .metadata()
        .and_then(|metadata| metadata.modified())
        .ok()
        .and_then(|modified| SystemTime::now().duration_since(modified).ok())
        .is_some_and(|age| age < interval)
}

fn cleanup_tmp_dir_contents(path: &Path, retention: Duration) -> Result<(), String> {
    let now = SystemTime::now();
    for entry in
        fs::read_dir(path).map_err(|error| format!("read temp dir {}: {error}", path.display()))?
    {
        let entry = entry.map_err(|error| format!("read temp dir entry: {error}"))?;
        let entry_path = entry.path();
        if entry.file_name() == ".last-cleanup" {
            continue;
        }
        let metadata = fs::symlink_metadata(&entry_path)
            .map_err(|error| format!("stat temp entry {}: {error}", entry_path.display()))?;
        let modified = metadata
            .modified()
            .map_err(|error| format!("read temp entry mtime {}: {error}", entry_path.display()))?;
        let age = now.duration_since(modified).unwrap_or_default();
        if age < retention {
            continue;
        }
        if metadata.is_dir() {
            fs::remove_dir_all(&entry_path)
                .map_err(|error| format!("remove temp dir {}: {error}", entry_path.display()))?;
        } else {
            fs::remove_file(&entry_path)
                .map_err(|error| format!("remove temp file {}: {error}", entry_path.display()))?;
        }
    }
    Ok(())
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

    #[test]
    fn cleanup_tmp_dir_contents_removes_expired_entries() {
        let (workspace, _tmp) = unique_test_dir("tmp-cleanup");
        let root = workspace.join("state/tmp/canon-ai-mcp");
        let old_dir = root.join("apply-patch-old");
        let fresh_dir = root.join("apply-patch-fresh");
        fs::create_dir_all(&old_dir).expect("create old temp dir");
        fs::create_dir_all(&fresh_dir).expect("create fresh temp dir");
        fs::write(old_dir.join("patch"), "old").expect("write old file");

        cleanup_tmp_dir_contents(&root, Duration::from_secs(0)).expect("cleanup expired entries");

        assert!(!old_dir.exists());
        assert!(!fresh_dir.exists());
    }

    #[test]
    fn cleanup_tmp_dir_contents_preserves_unexpired_entries() {
        let (workspace, _tmp) = unique_test_dir("tmp-cleanup-preserve");
        let root = workspace.join("state/tmp/canon-ai-mcp");
        let temp_dir = root.join("apply-patch-active");
        fs::create_dir_all(&temp_dir).expect("create active temp dir");

        cleanup_tmp_dir_contents(&root, Duration::from_secs(86_400))
            .expect("cleanup should preserve unexpired entries");

        assert!(temp_dir.exists());
    }
}
