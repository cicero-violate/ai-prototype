//! Static analysis capability.
//!
//! Invokes the `canon-rustc-v3-wrapper` binary via `cargo check`, parses the
//! resulting judgement-style artifacts, and emits a typed `AnalysisReceipt`.
//!
//! Ownership boundary:
//!   - This module owns: binary invocation, output parsing, receipt emission.
//!   - This module does NOT own: rustc process lifecycle, scheduling, evidence storage.
//!   - Callers submit the returned `AnalysisReceipt` through the runtime receipt boundary.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Typed receipt for a completed rustc analysis run.
///
/// Contains deterministic counts and a stable receipt hash so callers can
/// submit this through the runtime receipt boundary for TLog appending.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisReceipt {
    pub crate_name: String,
    pub node_count: usize,
    pub edge_count: usize,
    /// Number of functions with a non-empty intent classification.
    pub intent_count: usize,
    /// Stable hash over crate_name, counts, and capture timestamp.
    pub receipt_hash: u64,
    pub artifact_dir: PathBuf,
    pub captured_at_ms: u64,
}

impl AnalysisReceipt {
    pub fn is_contract_valid(&self) -> bool {
        !self.crate_name.is_empty()
            && self.receipt_hash > 0
            && self.artifact_dir.join("manifest.json").exists()
            && self.artifact_dir.join("semantic_index.jsonl").exists()
    }
}

/// Invoke the canon-rustc-v3 wrapper, parse the resulting artifacts, and
/// return a typed `AnalysisReceipt`.
///
/// `workspace_root` — directory containing the workspace Cargo.toml
/// `wrapper_bin`   — path to the canon-rustc-v3-wrapper binary
/// `artifact_root` — directory where artifact files are written (e.g. `state/rustc`)
/// `crate_name`    — name of the target crate whose artifact directory to parse
pub fn invoke_rustc_analysis(
    workspace_root: &Path,
    wrapper_bin: &Path,
    artifact_root: &Path,
    crate_name: &str,
) -> Result<AnalysisReceipt, String> {
    if !wrapper_bin.exists() {
        return Err(format!(
            "canon-rustc-v3-wrapper not found at {}",
            wrapper_bin.display()
        ));
    }

    let captured_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let status = Command::new("cargo")
        .current_dir(workspace_root)
        .args(["check", "--workspace"])
        .env("CARGO_BUILD_RUSTC_WRAPPER", wrapper_bin)
        .env("AI_ARTIFACT_ROOT", artifact_root)
        .status()
        .map_err(|e| format!("cargo check failed to launch: {e}"))?;

    if !status.success() {
        return Err(format!("cargo check exited with {status}"));
    }

    let artifact_dir = artifact_root.join(crate_name);
    parse_artifact_receipt(crate_name, &artifact_dir, captured_at_ms)
}

/// Parse an already-generated artifact directory and emit an `AnalysisReceipt`.
///
/// Use this variant when the artifacts were produced by a prior run or
/// captured independently (e.g., by the canon-rustc-v3-wrapper hook).
pub fn receipt_from_artifacts(
    crate_name: &str,
    artifact_dir: &Path,
) -> Result<AnalysisReceipt, String> {
    let captured_at_ms = manifest_captured_at_ms(&artifact_dir.join("manifest.json"))
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0)
        });
    parse_artifact_receipt(crate_name, artifact_dir, captured_at_ms)
}

fn parse_artifact_receipt(
    crate_name: &str,
    artifact_dir: &Path,
    captured_at_ms: u64,
) -> Result<AnalysisReceipt, String> {
    let manifest_path = artifact_dir.join("manifest.json");
    let semantic_path = artifact_dir.join("semantic_index.jsonl");
    if !manifest_path.exists() {
        return Err(format!(
            "manifest.json not found at {}",
            manifest_path.display()
        ));
    }
    if !semantic_path.exists() {
        return Err(format!(
            "semantic_index.jsonl not found at {}",
            semantic_path.display()
        ));
    }

    let (node_count, edge_count, intent_count) = semantic_counts(&semantic_path)?;

    let receipt_hash = compute_receipt_hash(
        crate_name,
        node_count,
        edge_count,
        intent_count,
        captured_at_ms,
    );

    Ok(AnalysisReceipt {
        crate_name: crate_name.to_string(),
        node_count,
        edge_count,
        intent_count,
        receipt_hash,
        artifact_dir: artifact_dir.to_path_buf(),
        captured_at_ms,
    })
}

fn semantic_counts(path: &Path) -> Result<(usize, usize, usize), String> {
    use std::io::BufRead;

    let file = std::fs::File::open(path).map_err(|e| format!("open semantic_index.jsonl: {e}"))?;
    let reader = std::io::BufReader::with_capacity(256 * 1024, file);
    let mut node_count = 0usize;
    let mut edge_count = 0usize;
    let mut fn_count = 0usize;

    for line in reader.lines() {
        let line = line.map_err(|e| format!("read semantic_index.jsonl: {e}"))?;
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        match value.get("kind").and_then(serde_json::Value::as_str) {
            Some("symbol_def") => {
                node_count += 1;
                if value.get("node_kind").and_then(serde_json::Value::as_str) == Some("fn") {
                    fn_count += 1;
                }
            }
            Some("semantic_edge") => edge_count += 1,
            _ => {}
        }
    }

    Ok((node_count, edge_count, fn_count))
}

fn manifest_captured_at_ms(manifest_path: &Path) -> Option<u64> {
    let metadata = std::fs::metadata(manifest_path).ok()?;
    let modified = metadata.modified().ok()?;
    modified
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64)
}

fn compute_receipt_hash(
    crate_name: &str,
    node_count: usize,
    edge_count: usize,
    intent_count: usize,
    captured_at_ms: u64,
) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in crate_name.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h = h.wrapping_add(node_count as u64);
    h = h.wrapping_mul(0x0000_0100_0000_01b3);
    h = h.wrapping_add(edge_count as u64);
    h = h.wrapping_mul(0x0000_0100_0000_01b3);
    h = h.wrapping_add(intent_count as u64);
    h ^= captured_at_ms;
    h.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_artifacts(dir: &Path, crate_name: &str, nodes: u64, edges: u64, fns: usize) {
        let crate_dir = dir.join(crate_name);
        std::fs::create_dir_all(&crate_dir).unwrap();
        let manifest = serde_json::json!({
            "schema_version": 1,
            "graph_schema_version": 17,
            "crate_target": {
                "package_crate": crate_name,
                "target_key": crate_name,
                "target_kind": "lib"
            },
            "hashes": {},
            "artifacts": {}
        });
        let mut manifest_file = std::fs::File::create(crate_dir.join("manifest.json")).unwrap();
        write!(
            manifest_file,
            "{}",
            serde_json::to_string(&manifest).unwrap()
        )
        .unwrap();

        let mut semantic = std::fs::File::create(crate_dir.join("semantic_index.jsonl")).unwrap();
        writeln!(
            semantic,
            "{{\"kind\":\"semantic_sentinel\",\"schema_version\":17}}"
        )
        .unwrap();
        for i in 0..nodes {
            let node_kind = if (i as usize) < fns { "fn" } else { "struct" };
            writeln!(
                semantic,
                "{{\"kind\":\"symbol_def\",\"def_path\":\"crate::item_{i}\",\"node_kind\":\"{node_kind}\"}}"
            )
            .unwrap();
        }
        for i in 0..edges {
            writeln!(
                semantic,
                "{{\"kind\":\"semantic_edge\",\"relation\":\"call\",\"from\":\"crate::item_0\",\"to\":\"crate::item_{i}\"}}"
            )
            .unwrap();
        }
    }

    #[test]
    fn receipt_from_artifacts_parses_counts_and_intents() {
        let tmp_guard = tempfile::Builder::new()
            .prefix("ai-rustc-analysis-test-")
            .tempdir_in({
                let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
                std::fs::create_dir_all(&d).unwrap();
                d
            })
            .unwrap();
        let tmp = tmp_guard.path();
        write_artifacts(tmp, "my_crate", 100, 500, 80);
        let receipt = receipt_from_artifacts("my_crate", &tmp.join("my_crate")).expect("receipt");
        assert_eq!(receipt.crate_name, "my_crate");
        assert_eq!(receipt.node_count, 100);
        assert_eq!(receipt.edge_count, 500);
        assert_eq!(receipt.intent_count, 80);
        assert!(receipt.receipt_hash > 0);
        assert!(receipt.is_contract_valid());
    }

    #[test]
    fn receipt_from_artifacts_fails_on_missing_file() {
        let result = receipt_from_artifacts("missing", Path::new("/nonexistent/artifacts"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("manifest.json not found"));
    }

    #[test]
    fn compute_receipt_hash_is_deterministic() {
        let h1 = compute_receipt_hash("ai", 1000, 5000, 700, 1_700_000_000_000);
        let h2 = compute_receipt_hash("ai", 1000, 5000, 700, 1_700_000_000_000);
        assert_eq!(h1, h2);
        let h3 = compute_receipt_hash("ai", 1000, 5000, 701, 1_700_000_000_000);
        assert_ne!(h1, h3);
    }

    #[test]
    fn receipt_hash_is_never_zero() {
        let h = compute_receipt_hash("", 0, 0, 0, 0);
        assert!(h > 0);
    }
}
