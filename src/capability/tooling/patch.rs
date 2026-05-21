//! Patch apply utility — `AffectedPaths` and workspace-safe path categorization.
//!
//! Ported from canon-tools-patch; adapted to ai receipt contract and workspace
//! path policy. Returns categorized paths (added/modified/deleted) to include
//! in effect receipts.

use std::path::{Component, Path};

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
///
/// Paths are validated against the workspace policy; any path that escapes
/// the workspace root causes an error rather than being silently skipped.
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
            // Move destination is the post-apply location; record as modified.
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
