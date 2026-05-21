//! Runtime-owned workspace boundary view.
//!
//! Tool adapters receive this view, but capability code does not own workspace
//! authority or boundary policy.

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

pub fn workspace_state_dir(workspace_root: &Path) -> PathBuf {
    let root = normalize_path(workspace_root);
    if let (Some(parent), Some(name)) = (root.parent(), root.file_name()) {
        let shared_state = parent.join("state");
        if shared_state.is_dir()
            || parent.join("ai").is_dir()
            || parent.join("browser-router").is_dir()
            || parent.join("canon-rustc-v3").is_dir()
        {
            return shared_state.join(name);
        }
    }
    root.join("state")
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
            other @ Component::Prefix(_)
            | other @ Component::RootDir
            | other @ Component::Normal(_) => {
                out.push(other);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_state_dir_uses_shared_prototype_state_for_members() {
        let root =
            std::env::temp_dir().join(format!("canon-workspace-state-test-{}", std::process::id()));
        let member = root.join("canon-rustc-v3");
        std::fs::create_dir_all(&member).expect("create member");
        std::fs::create_dir_all(root.join("state")).expect("create state");

        assert_eq!(
            workspace_state_dir(&member),
            root.join("state").join("canon-rustc-v3")
        );

        let _ = std::fs::remove_dir_all(root);
    }
}
