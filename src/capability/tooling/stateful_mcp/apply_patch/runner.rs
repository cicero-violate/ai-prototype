//! apply_patch subprocess runner.

use std::env;
use std::path::Path;
use std::process::Stdio;

use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub(super) async fn run_apply_patch_binary(
    patch_text: &str,
    cwd: &Path,
) -> Result<std::process::Output, String> {
    let bin = env::var("APPLY_PATCH_BIN").unwrap_or_else(|_| "apply_patch".to_string());
    let mut child = Command::new(bin)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("spawn apply_patch: {error}"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "failed to open apply_patch stdin".to_string())?;
    stdin
        .write_all(patch_text.as_bytes())
        .await
        .map_err(|error| format!("write apply_patch stdin: {error}"))?;
    drop(stdin);
    child
        .wait_with_output()
        .await
        .map_err(|error| format!("wait for apply_patch: {error}"))
}

pub(super) fn format_apply_patch_failure(output: std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut text = format!("apply_patch exited with {}", output.status);
    if !stdout.trim().is_empty() {
        text.push_str("\nstdout:\n");
        text.push_str(stdout.trim());
    }
    if !stderr.trim().is_empty() {
        text.push_str("\nstderr:\n");
        text.push_str(stderr.trim());
    }
    text
}
