//! Capability-owned workspace view for stateful MCP tools.
//!
//! This keeps file/process tools independent from supervisor-owned runtime config.

use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceView {
    pub root: PathBuf,
    pub allowed_boundary: PathBuf,
}

impl WorkspaceView {
    pub fn new(root: PathBuf, allowed_boundary: PathBuf) -> Result<Self, String> {
        let root = canonical_existing_dir(&root)?;
        let allowed_boundary = canonical_existing_dir(&allowed_boundary)?;
        if !root.starts_with(&allowed_boundary) {
            return Err(format!(
                "workspace root {} is outside allowed boundary {}",
                root.display(),
                allowed_boundary.display()
            ));
        }
        Ok(Self {
            root,
            allowed_boundary,
        })
    }

    pub fn resolve_cwd(&self, cwd: &str) -> Result<PathBuf, String> {
        let rel = if cwd.trim().is_empty() {
            "."
        } else {
            cwd.trim()
        };
        let requested = Path::new(rel);
        let joined = if requested.is_absolute() {
            normalize_path(requested)
        } else {
            normalize_path(&self.root.join(requested))
        };
        if !joined.starts_with(&self.allowed_boundary) {
            return Err("cwd escapes allowed workspace boundary".to_string());
        }
        Ok(joined)
    }
}

fn canonical_existing_dir(path: &Path) -> Result<PathBuf, String> {
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("canonicalize {}: {e}", path.display()))?;
    if !canonical.is_dir() {
        return Err(format!("{} is not a directory", canonical.display()));
    }
    Ok(canonical)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}
