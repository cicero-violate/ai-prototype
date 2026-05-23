//! Static analysis capability.
//!
//! Invokes the `canon-rustc-v3-wrapper` binary via `cargo check`, parses the
//! resulting `graph.json` artifact, and emits a typed `AnalysisReceipt`.
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
    pub graph_path: PathBuf,
    pub captured_at_ms: u64,
}

impl AnalysisReceipt {
    pub fn is_contract_valid(&self) -> bool {
        !self.crate_name.is_empty() && self.receipt_hash > 0 && self.graph_path.exists()
    }
}

/// Invoke the canon-rustc-v3 wrapper, parse the resulting graph.json, and
/// return a typed `AnalysisReceipt`.
///
/// `workspace_root` — directory containing the workspace Cargo.toml
/// `wrapper_bin`   — path to the canon-rustc-v3-wrapper binary
/// `artifact_root` — directory where graph.json files are written (e.g. `state/rustc`)
/// `crate_name`    — name of the target crate whose graph.json to parse
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

    let graph_path = artifact_root.join(crate_name).join("graph.json");
    parse_graph_receipt(crate_name, &graph_path, captured_at_ms)
}

/// Parse an already-generated graph.json and emit an `AnalysisReceipt`.
///
/// Use this variant when the graph.json was produced by a prior run or
/// captured independently (e.g., by the canon-rustc-v3-wrapper hook).
pub fn receipt_from_graph_json(
    crate_name: &str,
    graph_path: &Path,
) -> Result<AnalysisReceipt, String> {
    let captured_at_ms = graph_captured_at_ms(graph_path).unwrap_or_else(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    });
    parse_graph_receipt(crate_name, graph_path, captured_at_ms)
}

fn parse_graph_receipt(
    crate_name: &str,
    graph_path: &Path,
    captured_at_ms: u64,
) -> Result<AnalysisReceipt, String> {
    if !graph_path.exists() {
        return Err(format!("graph.json not found at {}", graph_path.display()));
    }

    let content =
        std::fs::read_to_string(graph_path).map_err(|e| format!("read graph.json failed: {e}"))?;
    let graph: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("parse graph.json failed: {e}"))?;

    let node_count = graph["meta"]["node_count"].as_u64().unwrap_or(0) as usize;
    let edge_count = graph["meta"]["edge_count"].as_u64().unwrap_or(0) as usize;
    let intent_count = graph["intents"].as_object().map(|m| m.len()).unwrap_or(0);

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
        graph_path: graph_path.to_path_buf(),
        captured_at_ms,
    })
}

fn graph_captured_at_ms(graph_path: &Path) -> Option<u64> {
    let content = std::fs::read_to_string(graph_path).ok()?;
    let graph: serde_json::Value = serde_json::from_str(&content).ok()?;
    graph["meta"]["captured_at_ms"].as_u64()
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

    fn write_graph_json(dir: &Path, crate_name: &str, nodes: u64, edges: u64, intents: usize) {
        let crate_dir = dir.join(crate_name);
        std::fs::create_dir_all(&crate_dir).unwrap();
        let intents_map: serde_json::Value = (0..intents)
            .map(|i| {
                (
                    format!("fn_{i}"),
                    serde_json::Value::String("pure".to_string()),
                )
            })
            .collect::<serde_json::Map<_, _>>()
            .into();
        let graph = serde_json::json!({
            "meta": {
                "crate_name": crate_name,
                "node_count": nodes,
                "edge_count": edges,
                "captured_at_ms": 1_700_000_000_000u64,
            },
            "intents": intents_map
        });
        let mut f = std::fs::File::create(crate_dir.join("graph.json")).unwrap();
        write!(f, "{}", serde_json::to_string(&graph).unwrap()).unwrap();
    }

    #[test]
    fn receipt_from_graph_json_parses_counts_and_intents() {
        let tmp = std::env::temp_dir().join("rustc_analysis_test");
        std::fs::create_dir_all(&tmp).unwrap();
        write_graph_json(&tmp, "my_crate", 100, 500, 80);
        let receipt = receipt_from_graph_json("my_crate", &tmp.join("my_crate").join("graph.json"))
            .expect("receipt");
        assert_eq!(receipt.crate_name, "my_crate");
        assert_eq!(receipt.node_count, 100);
        assert_eq!(receipt.edge_count, 500);
        assert_eq!(receipt.intent_count, 80);
        assert!(receipt.receipt_hash > 0);
        assert!(receipt.is_contract_valid());
    }

    #[test]
    fn receipt_from_graph_json_fails_on_missing_file() {
        let result = receipt_from_graph_json("missing", Path::new("/nonexistent/graph.json"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("graph.json not found"));
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
