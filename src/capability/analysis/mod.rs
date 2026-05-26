//! Static analysis capability.
//!
//! Invokes the `canon-rustc-v3-wrapper` binary via `cargo check`, parses the
//! resulting judgement-style artifacts, and emits a typed `AnalysisReceipt`.
//!
//! Ownership boundary:
//!   - This module owns: binary invocation, output parsing, receipt emission.
//!   - This module does NOT own: rustc process lifecycle, scheduling, evidence storage.
//!   - Callers submit the returned `AnalysisReceipt` through the runtime receipt boundary.

use std::fmt;
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

/// Artifact files that make up a rustc analysis capture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisArtifact {
    Manifest,
    SemanticIndex,
}

impl fmt::Display for AnalysisArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest => f.write_str("manifest.json"),
            Self::SemanticIndex => f.write_str("semantic_index.jsonl"),
        }
    }
}

/// Compiler facts required to turn artifacts into a typed receipt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerFact {
    SemanticRecordKind,
    NodeKind,
}

impl fmt::Display for CompilerFact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SemanticRecordKind => f.write_str("kind"),
            Self::NodeKind => f.write_str("node_kind"),
        }
    }
}

/// Typed result of crossing the rustc analysis boundary.
///
/// Callers match this enum instead of re-extracting success, compiler failure,
/// missing-artifact, or missing-fact states from formatted strings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RustcAnalysisOutcome {
    Complete(AnalysisReceipt),
    CompilerUnavailable {
        wrapper_bin: PathBuf,
    },
    CompilerLaunchFailed {
        message: String,
    },
    CompilerFailed {
        code: Option<i32>,
    },
    ArtifactMissing {
        artifact: AnalysisArtifact,
        path: PathBuf,
    },
    ArtifactReadFailed {
        artifact: AnalysisArtifact,
        path: PathBuf,
        message: String,
    },
    ArtifactMalformed {
        artifact: AnalysisArtifact,
        path: PathBuf,
        line: usize,
        message: String,
    },
    CompilerFactMissing {
        artifact: AnalysisArtifact,
        path: PathBuf,
        line: usize,
        fact: CompilerFact,
    },
}

impl RustcAnalysisOutcome {
    pub fn receipt(&self) -> Option<&AnalysisReceipt> {
        match self {
            Self::Complete(receipt) => Some(receipt),
            _ => None,
        }
    }

    pub fn into_receipt(self) -> Result<AnalysisReceipt, Self> {
        match self {
            Self::Complete(receipt) => Ok(receipt),
            other => Err(other),
        }
    }
}

impl fmt::Display for RustcAnalysisOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Complete(receipt) => write!(
                f,
                "analysis complete for {} with {} nodes and {} edges",
                receipt.crate_name, receipt.node_count, receipt.edge_count
            ),
            Self::CompilerUnavailable { wrapper_bin } => {
                write!(
                    f,
                    "canon-rustc-v3-wrapper not found at {}",
                    wrapper_bin.display()
                )
            }
            Self::CompilerLaunchFailed { message } => {
                write!(f, "cargo check failed to launch: {message}")
            }
            Self::CompilerFailed { code } => {
                write!(f, "cargo check exited unsuccessfully with code {code:?}")
            }
            Self::ArtifactMissing { artifact, path } => {
                write!(f, "{artifact} not found at {}", path.display())
            }
            Self::ArtifactReadFailed {
                artifact,
                path,
                message,
            } => {
                write!(
                    f,
                    "failed to read {artifact} at {}: {message}",
                    path.display()
                )
            }
            Self::ArtifactMalformed {
                artifact,
                path,
                line,
                message,
            } => write!(
                f,
                "malformed {artifact} at {} line {line}: {message}",
                path.display()
            ),
            Self::CompilerFactMissing {
                artifact,
                path,
                line,
                fact,
            } => write!(
                f,
                "missing compiler fact {fact} in {artifact} at {} line {line}",
                path.display()
            ),
        }
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
) -> RustcAnalysisOutcome {
    if !wrapper_bin.exists() {
        return RustcAnalysisOutcome::CompilerUnavailable {
            wrapper_bin: wrapper_bin.to_path_buf(),
        };
    }

    let captured_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let status = match Command::new("cargo")
        .current_dir(workspace_root)
        .args(["check", "--workspace"])
        .env("CARGO_BUILD_RUSTC_WRAPPER", wrapper_bin)
        .env("AI_ARTIFACT_ROOT", artifact_root)
        .status()
    {
        Ok(status) => status,
        Err(error) => {
            return RustcAnalysisOutcome::CompilerLaunchFailed {
                message: error.to_string(),
            }
        }
    };

    if !status.success() {
        return RustcAnalysisOutcome::CompilerFailed {
            code: status.code(),
        };
    }

    let artifact_dir = artifact_root.join(crate_name);
    parse_artifact_receipt(crate_name, &artifact_dir, captured_at_ms)
}

/// Parse an already-generated artifact directory and emit an `AnalysisReceipt`.
///
/// Use this variant when the artifacts were produced by a prior run or
/// captured independently (e.g., by the canon-rustc-v3-wrapper hook).
pub fn receipt_from_artifacts(crate_name: &str, artifact_dir: &Path) -> RustcAnalysisOutcome {
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
) -> RustcAnalysisOutcome {
    let manifest_path = artifact_dir.join("manifest.json");
    let semantic_path = artifact_dir.join("semantic_index.jsonl");
    if !manifest_path.exists() {
        return RustcAnalysisOutcome::ArtifactMissing {
            artifact: AnalysisArtifact::Manifest,
            path: manifest_path,
        };
    }
    if !semantic_path.exists() {
        return RustcAnalysisOutcome::ArtifactMissing {
            artifact: AnalysisArtifact::SemanticIndex,
            path: semantic_path,
        };
    }

    let counts = match semantic_counts(&semantic_path) {
        Ok(counts) => counts,
        Err(outcome) => return outcome,
    };

    let receipt_hash = compute_receipt_hash(
        crate_name,
        counts.node_count,
        counts.edge_count,
        counts.intent_count,
        captured_at_ms,
    );

    RustcAnalysisOutcome::Complete(AnalysisReceipt {
        crate_name: crate_name.to_string(),
        node_count: counts.node_count,
        edge_count: counts.edge_count,
        intent_count: counts.intent_count,
        receipt_hash,
        artifact_dir: artifact_dir.to_path_buf(),
        captured_at_ms,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AnalysisCounts {
    node_count: usize,
    edge_count: usize,
    intent_count: usize,
}

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "kind")]
enum SemanticIndexRecord {
    #[serde(rename = "symbol_def")]
    SymbolDef { node_kind: Option<SemanticNodeKind> },
    #[serde(rename = "semantic_edge")]
    SemanticEdge,
    #[serde(other)]
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SemanticNodeKind {
    Function,
    Other(String),
}

impl SemanticNodeKind {
    fn carries_intent_classification(&self) -> bool {
        matches!(self, Self::Function)
    }
}

impl<'de> serde::Deserialize<'de> for SemanticNodeKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(match raw.as_str() {
            "fn" => Self::Function,
            _ => Self::Other(raw),
        })
    }
}

fn semantic_counts(path: &Path) -> Result<AnalysisCounts, RustcAnalysisOutcome> {
    use std::io::BufRead;

    let file =
        std::fs::File::open(path).map_err(|error| RustcAnalysisOutcome::ArtifactReadFailed {
            artifact: AnalysisArtifact::SemanticIndex,
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
    let reader = std::io::BufReader::with_capacity(256 * 1024, file);
    let mut node_count = 0usize;
    let mut edge_count = 0usize;
    let mut fn_count = 0usize;

    for (idx, line) in reader.lines().enumerate() {
        let line_number = idx + 1;
        let line = line.map_err(|error| RustcAnalysisOutcome::ArtifactReadFailed {
            artifact: AnalysisArtifact::SemanticIndex,
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        if line.is_empty() {
            continue;
        }
        let record: SemanticIndexRecord = serde_json::from_str(&line).map_err(|error| {
            if error.to_string().contains("missing field `kind`") {
                RustcAnalysisOutcome::CompilerFactMissing {
                    artifact: AnalysisArtifact::SemanticIndex,
                    path: path.to_path_buf(),
                    line: line_number,
                    fact: CompilerFact::SemanticRecordKind,
                }
            } else {
                RustcAnalysisOutcome::ArtifactMalformed {
                    artifact: AnalysisArtifact::SemanticIndex,
                    path: path.to_path_buf(),
                    line: line_number,
                    message: error.to_string(),
                }
            }
        })?;
        match record {
            SemanticIndexRecord::SymbolDef { node_kind } => {
                node_count += 1;
                let node_kind =
                    node_kind.ok_or_else(|| RustcAnalysisOutcome::CompilerFactMissing {
                        artifact: AnalysisArtifact::SemanticIndex,
                        path: path.to_path_buf(),
                        line: line_number,
                        fact: CompilerFact::NodeKind,
                    })?;
                if node_kind.carries_intent_classification() {
                    fn_count += 1;
                }
            }
            SemanticIndexRecord::SemanticEdge => edge_count += 1,
            SemanticIndexRecord::Other => {}
        }
    }

    Ok(AnalysisCounts {
        node_count,
        edge_count,
        intent_count: fn_count,
    })
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
        let receipt = match receipt_from_artifacts("my_crate", &tmp.join("my_crate")) {
            RustcAnalysisOutcome::Complete(receipt) => receipt,
            other => panic!("unexpected analysis outcome: {other:?}"),
        };
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
        match result {
            RustcAnalysisOutcome::ArtifactMissing {
                artifact: AnalysisArtifact::Manifest,
                path,
            } => assert!(path.ends_with("manifest.json")),
            other => panic!("unexpected analysis outcome: {other:?}"),
        }
    }

    #[test]
    fn receipt_from_artifacts_fails_on_missing_semantic_fact() {
        let tmp_guard = tempfile::Builder::new()
            .prefix("ai-rustc-analysis-test-")
            .tempdir_in({
                let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
                std::fs::create_dir_all(&d).unwrap();
                d
            })
            .unwrap();
        let tmp = tmp_guard.path();
        let crate_dir = tmp.join("my_crate");
        std::fs::create_dir_all(&crate_dir).unwrap();
        std::fs::write(crate_dir.join("manifest.json"), "{}").unwrap();
        std::fs::write(
            crate_dir.join("semantic_index.jsonl"),
            "{\"kind\":\"symbol_def\",\"def_path\":\"crate::missing_node_kind\"}\n",
        )
        .unwrap();

        let result = receipt_from_artifacts("my_crate", &crate_dir);
        match result {
            RustcAnalysisOutcome::CompilerFactMissing {
                artifact: AnalysisArtifact::SemanticIndex,
                fact: CompilerFact::NodeKind,
                line: 1,
                ..
            } => {}
            other => panic!("unexpected analysis outcome: {other:?}"),
        }
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
