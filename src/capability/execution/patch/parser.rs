//! apply_patch patch metadata parsing and path validation.

use std::path::{Component, Path};

pub(super) fn collect_apply_patch_changed_files(patch_text: &str) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    let mut saw_begin = false;
    let mut saw_end = false;
    for line in patch_text.lines() {
        let line = line.trim_end();
        if line == "*** Begin Patch" {
            saw_begin = true;
        } else if line == "*** End Patch" {
            saw_end = true;
        } else if let Some(path) = line.strip_prefix("*** Add File: ") {
            files.push(path.trim().to_string());
        } else if let Some(path) = line.strip_prefix("*** Delete File: ") {
            files.push(path.trim().to_string());
        } else if let Some(path) = line.strip_prefix("*** Update File: ") {
            files.push(path.trim().to_string());
        } else if let Some(path) = line.strip_prefix("*** Move to: ") {
            files.push(path.trim().to_string());
        }
    }
    if !saw_begin || !saw_end {
        return Err("patch must use apply_patch format with begin and end markers".to_string());
    }
    if files.is_empty() {
        return Err("no file patches found".to_string());
    }
    files.sort();
    files.dedup();
    Ok(files)
}

pub(super) fn validate_workspace_relative_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!("{} escapes workspace root", path.display()));
    }
    Ok(())
}
