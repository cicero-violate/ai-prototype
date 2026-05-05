//! Argv parsing helpers (ported from canon-rustc flags.rs).

use std::fs;
use std::path::{Path, PathBuf};

pub fn find_flag_value(args: &[String], flag: &str) -> Option<String> {
    if let Some(val) = args.windows(2).find(|w| w[0] == flag).map(|w| w[1].clone()) {
        return Some(val);
    }
    let prefix = format!("{flag}=");
    args.iter()
        .find_map(|arg| arg.strip_prefix(&prefix).map(|v| v.to_string()))
}

pub fn find_flag_values(args: &[String], flag: &str) -> Vec<String> {
    let mut values: Vec<String> = args
        .windows(2)
        .filter(|w| w[0] == flag)
        .map(|w| w[1].clone())
        .collect();
    let prefix = format!("{flag}=");
    for arg in args {
        if let Some(val) = arg.strip_prefix(&prefix) {
            values.push(val.to_string());
        }
    }
    values
}

/// Resolve the active Cargo project root without host-specific paths.
pub fn workspace_root_from_output_dir(output_dir: &Path) -> PathBuf {
    logical_pwd_project_root()
        .or_else(current_dir_project_root)
        .or_else(|| project_root_from_target_path(output_dir))
        .or_else(|| find_workspace_root(output_dir))
        .or_else(|| output_dir.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| output_dir.to_path_buf())
}

fn logical_pwd_project_root() -> Option<PathBuf> {
    let pwd = PathBuf::from(std::env::var_os("PWD")?);
    if pwd.join("Cargo.toml").is_file()
        && !is_cargo_registry_path(&pwd)
        && !pwd.components().any(|c| c.as_os_str() == "target")
    {
        Some(pwd)
    } else {
        None
    }
}

fn current_dir_project_root() -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    if current_dir.join("Cargo.toml").is_file()
        && !is_cargo_registry_path(&current_dir)
        && !current_dir.components().any(|c| c.as_os_str() == "target")
    {
        Some(current_dir)
    } else {
        None
    }
}

/// Return the parent of the first `target/` ancestor.
fn project_root_from_target_path(out_dir: &Path) -> Option<PathBuf> {
    let mut cursor = Some(out_dir);
    while let Some(path) = cursor {
        if path.file_name().and_then(|s| s.to_str()) == Some("target") {
            return path.parent().map(|p| p.to_path_buf());
        }
        cursor = path.parent();
    }
    None
}

fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    for dir in start.ancestors() {
        let manifest = dir.join("Cargo.toml");
        if let Ok(text) = fs::read_to_string(&manifest) {
            if text.contains("[workspace]") {
                return Some(dir.to_path_buf());
            }
        }
    }
    None
}

pub fn is_cargo_registry_path(path: &Path) -> bool {
    if path
        .components()
        .any(|c| c.as_os_str() == "registry" || c.as_os_str() == "git")
        && path.components().any(|c| c.as_os_str() == ".cargo")
    {
        return true;
    }
    let raw = path.to_string_lossy();
    raw.contains("/.cargo/registry/") || raw.contains("/.cargo/git/")
}

/// Analyze normal build artifacts only.
pub fn should_capture_crate(crate_name: Option<&str>, crate_types: &[String]) -> bool {
    if crate_name.is_none() {
        return false;
    }
    crate_types
        .iter()
        .any(|t| t == "bin" || t == "lib" || t == "rlib")
}

/// Default to `<workspace>/state/rustc`; ignore absolute overrides.
pub fn default_artifact_dir(workspace_root: &Path) -> PathBuf {
    let configured = std::env::var("CANON_RUSTC_V3_ARTIFACT_DIR")
        .or_else(|_| std::env::var("CANON_RUSTC_V2_ARTIFACT_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("state/rustc"));

    if configured.is_absolute() {
        workspace_root.join("state/rustc")
    } else {
        workspace_root.join(configured)
    }
}
