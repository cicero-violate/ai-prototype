//! Graph artifact metadata and freshness checking.
//!
//! Adapted from canon-storage-graph artifact writer concepts.
//! Provides a lightweight JSON sidecar (`GraphArtifactMeta`) that records
//! which TLog position produced the current graph artifact, enabling callers
//! to skip a full re-capture when the artifact is already up to date.
//!
//! The graph artifact itself (graph.json) remains authoritative.
//! This module only provides metadata helpers — it never writes to graph.json.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::kernel::ControlEvent;
use crate::runtime::CanonError;

/// Sidecar metadata pinning a graph artifact to its source TLog position.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphArtifactMeta {
    /// seq of the TLog event that triggered the last graph capture.
    pub tlog_seq: u64,
    /// Hash of the graph artifact content at capture time.
    pub graph_hash: u64,
    /// Node count at capture time.
    pub node_count: usize,
    /// Edge count at capture time.
    pub edge_count: usize,
    /// Unix timestamp ms when the artifact was written.
    pub created_at: u64,
}

/// Read graph artifact metadata from a JSON sidecar file.
/// Returns `None` if the file does not exist; `Err` if malformed.
pub fn read_graph_artifact_meta(path: &Path) -> Result<Option<GraphArtifactMeta>, CanonError> {
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(path).map_err(|_| CanonError::TlogIo)?;
    serde_json::from_str(&data)
        .map(Some)
        .map_err(|_| CanonError::InvalidTlogRecord)
}

/// Write graph artifact metadata atomically (temp → rename).
pub fn write_graph_artifact_meta(path: &Path, meta: &GraphArtifactMeta) -> Result<(), CanonError> {
    let data = serde_json::to_string_pretty(meta).map_err(|_| CanonError::InvalidTlogRecord)?;
    let tmp = path.with_extension("meta.tmp");
    fs::write(&tmp, data.as_bytes()).map_err(|_| CanonError::TlogIo)?;
    fs::rename(&tmp, path).map_err(|_| CanonError::TlogIo)?;
    Ok(())
}

/// Return `true` when the graph artifact recorded in `meta` still covers the
/// current TLog — i.e. the TLog has not advanced past the snapshot position.
///
/// A `false` result means the graph artifact is stale and should be re-captured.
pub fn is_graph_artifact_fresh(meta: &GraphArtifactMeta, tlog: &[ControlEvent]) -> bool {
    let last_seq = tlog.last().map(|e| e.seq).unwrap_or(0);
    meta.tlog_seq >= last_seq
}

/// Return `true` when the graph artifact's recorded `graph_hash` matches
/// the provided expected hash (e.g. freshly computed from the on-disk artifact).
pub fn graph_artifact_hash_matches(meta: &GraphArtifactMeta, expected_hash: u64) -> bool {
    meta.graph_hash == expected_hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::{
        CapabilityRegistryProjection, Cause, Decision, EventKind, Evidence, GateSet, Packet, Phase,
        RuntimeConfig, SemanticDelta, State,
    };
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn test_tmp_dir() -> PathBuf {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn make_event(seq: u64) -> ControlEvent {
        let state = State {
            phase: Phase::Invariant,
            gates: GateSet::default(),
            packet: Packet::empty(),
            failure: None,
            recovery_action: None,
            recovery_attempts: 0,
            wave_pending: 0,
            plan_state_hash: 0,
        };
        ControlEvent {
            seq,
            from: Phase::Delta,
            to: Phase::Invariant,
            kind: EventKind::Advanced,
            cause: Cause::Start,
            delta: SemanticDelta::PhaseAdvanced,
            evidence: Evidence::DeltaComputed,
            decision: Decision::Continue,
            failure: None,
            recovery_action: None,
            affected_gate: None,
            runtime_config: RuntimeConfig {
                max_steps: 10,
                max_recovery_attempts: 3,
            },
            state_before: state,
            state_after: state,
            capability_registry_projection: CapabilityRegistryProjection::new(1, 1),
            api_command_id: 0,
            api_command_hash: 0,
            prev_hash: 0,
            self_hash: seq,
        }
    }

    fn scratch_dir() -> (PathBuf, tempfile::TempDir) {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let tmp = tempfile::Builder::new()
            .prefix(&format!("ai-graph-meta-test-{}-{n}-", std::process::id()))
            .tempdir_in(test_tmp_dir())
            .unwrap();
        let dir = tmp.path().to_owned();
        (dir, tmp)
    }

    #[test]
    fn fresh_when_tlog_empty() {
        let meta = GraphArtifactMeta {
            tlog_seq: 0,
            graph_hash: 1,
            node_count: 0,
            edge_count: 0,
            created_at: 0,
        };
        assert!(is_graph_artifact_fresh(&meta, &[]));
    }

    #[test]
    fn fresh_when_meta_seq_covers_tlog() {
        let meta = GraphArtifactMeta {
            tlog_seq: 5,
            graph_hash: 1,
            node_count: 0,
            edge_count: 0,
            created_at: 0,
        };
        let tlog = vec![make_event(3), make_event(5)];
        assert!(is_graph_artifact_fresh(&meta, &tlog));
    }

    #[test]
    fn stale_when_tlog_advanced_past_meta() {
        let meta = GraphArtifactMeta {
            tlog_seq: 3,
            graph_hash: 1,
            node_count: 0,
            edge_count: 0,
            created_at: 0,
        };
        let tlog = vec![make_event(3), make_event(7)];
        assert!(!is_graph_artifact_fresh(&meta, &tlog));
    }

    #[test]
    fn hash_matches_correct_hash() {
        let meta = GraphArtifactMeta {
            tlog_seq: 1,
            graph_hash: 0xabcd,
            node_count: 10,
            edge_count: 20,
            created_at: 0,
        };
        assert!(graph_artifact_hash_matches(&meta, 0xabcd));
        assert!(!graph_artifact_hash_matches(&meta, 0xdead));
    }

    #[test]
    fn round_trip_read_write_meta() {
        let (dir, _tmp) = scratch_dir();
        let path = dir.join("graph.meta.json");
        let meta = GraphArtifactMeta {
            tlog_seq: 42,
            graph_hash: 0xdeadbeef,
            node_count: 100,
            edge_count: 300,
            created_at: 1_700_000_000,
        };
        write_graph_artifact_meta(&path, &meta).unwrap();
        let loaded = read_graph_artifact_meta(&path).unwrap().unwrap();
        assert_eq!(loaded, meta);
    }

    #[test]
    fn read_returns_none_for_missing_file() {
        let path = std::path::Path::new("/nonexistent/graph.meta.json");
        assert!(read_graph_artifact_meta(path).unwrap().is_none());
    }
}
