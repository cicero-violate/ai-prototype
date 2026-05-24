//! Temporary filesystem copy helpers for apply_patch check mode.

use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn copy_dir_recursive_excluding(
    src: &Path,
    dst: &Path,
    excluded_roots: &[PathBuf],
) -> Result<(), String> {
    fs::create_dir_all(dst)
        .map_err(|error| format!("create temp cwd {}: {error}", dst.display()))?;
    for entry in
        fs::read_dir(src).map_err(|error| format!("read cwd {}: {error}", src.display()))?
    {
        let entry = entry.map_err(|error| format!("read cwd entry: {error}"))?;
        let source_path = entry.path();
        if excluded_roots
            .iter()
            .any(|excluded| source_path.starts_with(excluded))
        {
            continue;
        }
        let dest_path = dst.join(entry.file_name());
        let file_type = entry
            .file_type()
            .map_err(|error| format!("read file type {}: {error}", source_path.display()))?;
        if file_type.is_dir() {
            copy_dir_recursive_excluding(&source_path, &dest_path, excluded_roots)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tmp_dir() -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn unique_test_dir(name: &str) -> (PathBuf, tempfile::TempDir) {
        let tmp = tempfile::Builder::new()
            .prefix(&format!("canon-ai-mcp-copy-test-{name}-"))
            .tempdir_in(test_tmp_dir())
            .expect("test dir creation should succeed");
        let dir = tmp.path().to_owned();
        (dir, tmp)
    }

    #[test]
    fn copy_dir_recursive_excluding_skips_temp_root_inside_source() {
        let (root, _tmp) = unique_test_dir("exclude-temp-root");
        let src = root.join("workspace");
        let temp_root = src.join("state/tmp/canon-ai-mcp");
        let old_candidate = temp_root.join("apply-patch-old");
        let dst = temp_root.join("apply-patch-new");

        fs::create_dir_all(&old_candidate).expect("create old candidate");
        fs::write(src.join("Cargo.toml"), "[workspace]\n").expect("write source file");
        fs::write(old_candidate.join("copied-too-many-times"), "old").expect("write temp file");

        copy_dir_recursive_excluding(&src, &dst, std::slice::from_ref(&temp_root))
            .expect("copy workspace excluding temp root");

        assert!(dst.join("Cargo.toml").exists());
        assert!(!dst.join("state/tmp/canon-ai-mcp").exists());
    }
}
