//! Patch execution capability — path categorization and apply_patch runner.

mod fs_copy;
mod parser;
mod runner;

use serde_json::{json, Value};
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

use crate::capability::execution::action::common::{tmp_dir, tool_error};
use crate::runtime::WorkspaceView;
use fs_copy::{copy_dir_recursive_excluding, TempDirGuard};
use parser::{collect_apply_patch_changed_files, validate_workspace_relative_path};
use runner::{format_apply_patch_failure, run_apply_patch_binary};

// ── AffectedPaths ─────────────────────────────────────────────────────────────

/// Categorized paths affected by a patch application.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AffectedPaths {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
}

impl AffectedPaths {
    /// Total number of affected paths across all categories.
    pub fn total(&self) -> usize {
        self.added.len() + self.modified.len() + self.deleted.len()
    }

    /// True when no paths were recorded (empty patch or parse failure).
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.modified.is_empty() && self.deleted.is_empty()
    }
}

/// Reject paths that escape the workspace root: absolute paths, `..`, or root prefixes.
pub fn is_workspace_safe_path(path: &str) -> bool {
    let p = Path::new(path);
    !p.is_absolute()
        && !p.components().any(|c| {
            matches!(
                c,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
}

/// Parse a patch string and categorize affected paths as added, modified, or deleted.
pub fn categorize_patch_paths(patch_text: &str) -> Result<AffectedPaths, String> {
    let mut added: Vec<String> = Vec::new();
    let mut modified: Vec<String> = Vec::new();
    let mut deleted: Vec<String> = Vec::new();
    let mut saw_begin = false;
    let mut saw_end = false;

    for line in patch_text.lines() {
        let line = line.trim_end();
        if line == "*** Begin Patch" {
            saw_begin = true;
        } else if line == "*** End Patch" {
            saw_end = true;
        } else if let Some(path) = line.strip_prefix("*** Add File: ") {
            let path = path.trim().to_string();
            if !is_workspace_safe_path(&path) {
                return Err(format!("{path} escapes workspace root"));
            }
            added.push(path);
        } else if let Some(path) = line.strip_prefix("*** Delete File: ") {
            let path = path.trim().to_string();
            if !is_workspace_safe_path(&path) {
                return Err(format!("{path} escapes workspace root"));
            }
            deleted.push(path);
        } else if let Some(path) = line.strip_prefix("*** Update File: ") {
            let path = path.trim().to_string();
            if !is_workspace_safe_path(&path) {
                return Err(format!("{path} escapes workspace root"));
            }
            modified.push(path);
        } else if let Some(path) = line.strip_prefix("*** Move to: ") {
            let path = path.trim().to_string();
            if !is_workspace_safe_path(&path) {
                return Err(format!("{path} escapes workspace root"));
            }
            modified.push(path);
        }
    }

    if !saw_begin || !saw_end {
        return Err("patch must use apply_patch format with begin and end markers".to_string());
    }
    if added.is_empty() && modified.is_empty() && deleted.is_empty() {
        return Err("no file patches found".to_string());
    }

    added.sort();
    added.dedup();
    modified.sort();
    modified.dedup();
    deleted.sort();
    deleted.dedup();

    Ok(AffectedPaths {
        added,
        modified,
        deleted,
    })
}

// ── apply_patch runner ────────────────────────────────────────────────────────

pub const APPLY_PATCH_TOOL: &str = "apply_patch";

pub async fn run(args: &Value, workspace: &WorkspaceView) -> Value {
    match run_inner(args, workspace).await {
        Ok(value) => value,
        Err(error) => tool_error(error),
    }
}

struct ApplyPatchRequest<'a> {
    patch_text: &'a str,
    mode: &'a str,
    do_apply: bool,
    cwd: &'a str,
}

async fn run_inner(args: &Value, workspace: &WorkspaceView) -> Result<Value, String> {
    let request = parse_apply_patch_request(args)?;
    let work_dir = workspace.resolve_cwd(request.cwd)?;
    let changed_files = collect_valid_changed_files(request.patch_text)?;
    let affected =
        categorize_patch_paths(request.patch_text).unwrap_or_else(|_| AffectedPaths::default());

    let (run_dir, _guard) =
        prepare_apply_patch_run_dir(request.do_apply, &workspace.root, &work_dir)?;
    let stdout = run_validated_apply_patch(request.patch_text, &run_dir).await?;

    Ok(success_response(
        request.mode,
        request.do_apply,
        changed_files,
        affected,
        stdout,
    ))
}

async fn run_validated_apply_patch(patch_text: &str, run_dir: &Path) -> Result<String, String> {
    let output = run_apply_patch_binary(patch_text, run_dir).await?;
    if !output.status.success() {
        return Err(format_apply_patch_failure(output));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn parse_apply_patch_request(args: &Value) -> Result<ApplyPatchRequest<'_>, String> {
    let patch_text = args
        .get("patch")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing 'patch' field".to_string())?
        .trim();
    if patch_text.is_empty() {
        return Err("patch is empty".to_string());
    }
    let max_bytes = args
        .get("maxBytes")
        .and_then(Value::as_u64)
        .unwrap_or(200_000)
        .max(1) as usize;
    if patch_text.len() > max_bytes {
        return Err(format!(
            "patch is {} bytes, larger than maxBytes={max_bytes}",
            patch_text.len()
        ));
    }
    let strip = args.get("strip").and_then(Value::as_u64).unwrap_or(0);
    if strip != 0 {
        return Err("strip is not supported by the apply_patch binary".to_string());
    }
    let mode = args.get("mode").and_then(Value::as_str).unwrap_or("check");
    let do_apply = match mode {
        "apply" => true,
        "check" => false,
        _ => return Err(format!("invalid mode {mode:?}; must be 'check' or 'apply'")),
    };
    let cwd = args.get("cwd").and_then(Value::as_str).unwrap_or(".");

    Ok(ApplyPatchRequest {
        patch_text,
        mode,
        do_apply,
        cwd,
    })
}

fn collect_valid_changed_files(patch_text: &str) -> Result<Vec<String>, String> {
    let changed_files = collect_apply_patch_changed_files(patch_text)?;
    for path in &changed_files {
        validate_workspace_relative_path(path)?;
    }
    Ok(changed_files)
}

fn prepare_apply_patch_run_dir(
    do_apply: bool,
    workspace_root: &Path,
    work_dir: &Path,
) -> Result<(PathBuf, Option<TempDirGuard>), String> {
    if do_apply {
        Ok((work_dir.to_path_buf(), None))
    } else {
        let temp_root = tmp_dir(workspace_root)?;
        let temp = temp_root.join(format!("apply-patch-{}", Uuid::new_v4()));
        copy_dir_recursive_excluding(work_dir, &temp, &[temp_root])?;
        Ok((temp.clone(), Some(TempDirGuard(temp))))
    }
}

fn success_response(
    mode: &str,
    do_apply: bool,
    changed_files: Vec<String>,
    affected: AffectedPaths,
    stdout: String,
) -> Value {
    let summary = if stdout.is_empty() {
        let verb = if do_apply { "applied" } else { "ok (check)" };
        format!("{verb} across {} file(s)", changed_files.len())
    } else if do_apply {
        stdout
    } else {
        format!("ok (check): {stdout}")
    };
    let payload = json!({
        "ok": true,
        "mode": mode,
        "changedFiles": changed_files,
        "affectedPaths": {
            "added": affected.added,
            "modified": affected.modified,
            "deleted": affected.deleted,
        },
        "summary": summary,
        "rejects": []
    });
    json!({
        "content": [{ "type": "text", "text": payload.to_string() }],
        "isError": false
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categorize_add_update_delete() {
        let patch = "\
*** Begin Patch
*** Add File: src/new.rs
+fn new() {}
*** Update File: src/existing.rs
@@ fn existing
-old
+new
*** Delete File: src/old.rs
*** End Patch";

        let affected = categorize_patch_paths(patch).expect("valid patch");
        assert_eq!(affected.added, vec!["src/new.rs"]);
        assert_eq!(affected.modified, vec!["src/existing.rs"]);
        assert_eq!(affected.deleted, vec!["src/old.rs"]);
        assert_eq!(affected.total(), 3);
    }

    #[test]
    fn move_to_records_destination_as_modified() {
        let patch = "\
*** Begin Patch
*** Update File: src/old_name.rs
*** Move to: src/new_name.rs
@@ rename
-old
+new
*** End Patch";

        let affected = categorize_patch_paths(patch).expect("valid patch");
        assert!(affected.modified.contains(&"src/new_name.rs".to_string()));
    }

    #[test]
    fn rejects_missing_begin_end_markers() {
        let err = categorize_patch_paths("*** Add File: src/foo.rs\n+fn foo() {}\n")
            .expect_err("must fail without markers");
        assert!(err.contains("begin and end markers"), "{err}");
    }

    #[test]
    fn rejects_absolute_path() {
        let patch = "\
*** Begin Patch
*** Add File: /etc/passwd
+evil
*** End Patch";
        let err = categorize_patch_paths(patch).expect_err("must reject absolute path");
        assert!(err.contains("escapes workspace root"), "{err}");
    }

    #[test]
    fn rejects_parent_dir_traversal() {
        let patch = "\
*** Begin Patch
*** Update File: ../../outside.rs
@@ ctx
-old
+new
*** End Patch";
        let err = categorize_patch_paths(patch).expect_err("must reject ../ path");
        assert!(err.contains("escapes workspace root"), "{err}");
    }

    #[test]
    fn workspace_safe_path_accepts_relative_paths() {
        assert!(is_workspace_safe_path("src/foo.rs"));
        assert!(is_workspace_safe_path("a/b/c.toml"));
        assert!(!is_workspace_safe_path("/absolute"));
        assert!(!is_workspace_safe_path("../escape"));
    }
}
