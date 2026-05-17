//! Temporary filesystem copy helpers for apply_patch check mode.

use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst)
        .map_err(|error| format!("create temp cwd {}: {error}", dst.display()))?;
    for entry in
        fs::read_dir(src).map_err(|error| format!("read cwd {}: {error}", src.display()))?
    {
        let entry = entry.map_err(|error| format!("read cwd entry: {error}"))?;
        let source_path = entry.path();
        let dest_path = dst.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|error| format!("read file type {}: {error}", source_path.display()))?;
        if file_type.is_dir() {
            copy_dir_recursive(&source_path, &dest_path)?;
        } else if file_type.is_file() {
            fs::copy(&source_path, &dest_path).map_err(|error| {
                format!(
                    "copy {} to {}: {error}",
                    source_path.display(),
                    dest_path.display()
                )
            })?;
        } else if file_type.is_symlink() {
            let target = fs::read_link(&source_path)
                .map_err(|error| format!("read symlink {}: {error}", source_path.display()))?;
            #[cfg(unix)]
            std::os::unix::fs::symlink(&target, &dest_path).map_err(|error| {
                format!(
                    "copy symlink {} to {}: {error}",
                    source_path.display(),
                    dest_path.display()
                )
            })?;
        }
    }
    Ok(())
}

pub(super) struct TempDirGuard(pub(super) PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
