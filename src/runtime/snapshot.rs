//! TLog snapshot metadata and session boundary detection.
//!
//! Adapted from canon-storage-eventlog session_scan/snapshot patterns.
//! Records the last known-good state position in the TLog so
//! resume_durable_runtime can start from a verified checkpoint instead of
//! replaying from the beginning.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::kernel::{ControlEvent, EventKind, Phase};
use crate::runtime::CanonError;

/// Lightweight sidecar metadata pinning the last clean snapshot position.
/// Written as a JSON file alongside the TLog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TlogSnapshotMeta {
    /// seq of the last snapshotted ControlEvent.
    pub tlog_seq: u64,
    /// self_hash of that event — forms integrity anchor.
    pub state_hash: u64,
    /// Number of events covered by this snapshot (events with seq <= tlog_seq).
    pub event_count: u64,
    /// Unix timestamp ms when this snapshot was written.
    pub created_at: u64,
}

/// Read snapshot metadata from a JSON sidecar file.
/// Returns `None` if the file does not exist; `Err` if it is malformed.
pub fn read_snapshot_meta(path: &Path) -> Result<Option<TlogSnapshotMeta>, CanonError> {
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(path).map_err(|_| CanonError::TlogIo)?;
    serde_json::from_str(&data)
        .map(Some)
        .map_err(|_| CanonError::InvalidTlogRecord)
}

/// Write snapshot metadata atomically (temp → rename).
pub fn write_snapshot_meta(path: &Path, meta: &TlogSnapshotMeta) -> Result<(), CanonError> {
    let data = serde_json::to_string_pretty(meta).map_err(|_| CanonError::InvalidTlogRecord)?;
    let tmp = path.with_extension("meta.tmp");
    fs::write(&tmp, data.as_bytes()).map_err(|_| CanonError::TlogIo)?;
    fs::rename(&tmp, path).map_err(|_| CanonError::TlogIo)?;
    Ok(())
}

/// Return the seq of the last successfully completed event in the TLog
/// (`kind == Completed`, `to == Phase::Done`). Returns `None` when no such
/// event exists — the lifecycle has not completed a full successful run yet.
pub fn find_last_completed_seq(tlog: &[ControlEvent]) -> Option<u64> {
    tlog.iter()
        .filter(|e| e.kind == EventKind::Completed && e.to == Phase::Done)
        .map(|e| e.seq)
        .last()
}

/// Build snapshot metadata anchored at the last completed event in the TLog.
/// Returns `None` if no completed event exists.
pub fn snapshot_meta_from_tlog(tlog: &[ControlEvent], created_at: u64) -> Option<TlogSnapshotMeta> {
    let last_seq = find_last_completed_seq(tlog)?;
    let last_event = tlog.iter().find(|e| e.seq == last_seq)?;
    let event_count = tlog.iter().filter(|e| e.seq <= last_seq).count() as u64;
    Some(TlogSnapshotMeta {
        tlog_seq: last_seq,
        state_hash: last_event.self_hash,
        event_count,
        created_at,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::ndjson::{decode_control_event_ndjson, encode_control_event_ndjson};
    use crate::kernel::{
        CapabilityRegistryProjection, Cause, Decision, Evidence, GateSet, Packet, Phase,
        RuntimeConfig, SemanticDelta, State,
    };

    fn make_event(seq: u64, kind: EventKind, to: Phase) -> ControlEvent {
        let state = State {
            phase: to,
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
            to,
            kind,
            cause: Cause::Start,
            delta: SemanticDelta::NoChange,
            evidence: Evidence::Missing,
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
            self_hash: seq * 1000 + 1,
        }
    }

    #[test]
    fn find_last_completed_seq_returns_none_on_empty_tlog() {
        assert_eq!(find_last_completed_seq(&[]), None);
    }

    #[test]
    fn find_last_completed_seq_returns_none_when_no_completed_event() {
        let tlog = vec![
            make_event(1, EventKind::Advanced, Phase::Invariant),
            make_event(2, EventKind::Failed, Phase::Recovery),
        ];
        assert_eq!(find_last_completed_seq(&tlog), None);
    }

    #[test]
    fn find_last_completed_seq_returns_seq_of_completed_event() {
        let tlog = vec![
            make_event(1, EventKind::Advanced, Phase::Invariant),
            make_event(2, EventKind::Completed, Phase::Done),
        ];
        assert_eq!(find_last_completed_seq(&tlog), Some(2));
    }

    #[test]
    fn find_last_completed_seq_returns_last_when_multiple() {
        let tlog = vec![
            make_event(1, EventKind::Completed, Phase::Done),
            make_event(5, EventKind::Advanced, Phase::Invariant),
            make_event(9, EventKind::Completed, Phase::Done),
        ];
        assert_eq!(find_last_completed_seq(&tlog), Some(9));
    }

    #[test]
    fn snapshot_meta_from_tlog_is_none_without_completed_event() {
        let tlog = vec![make_event(1, EventKind::Advanced, Phase::Invariant)];
        assert!(snapshot_meta_from_tlog(&tlog, 0).is_none());
    }

    #[test]
    fn snapshot_meta_from_tlog_anchors_at_last_completed() {
        let tlog = vec![
            make_event(1, EventKind::Advanced, Phase::Invariant),
            make_event(2, EventKind::Completed, Phase::Done),
            make_event(3, EventKind::Advanced, Phase::Invariant),
        ];
        let meta = snapshot_meta_from_tlog(&tlog, 42).unwrap();
        assert_eq!(meta.tlog_seq, 2);
        assert_eq!(meta.state_hash, 2001);
        assert_eq!(meta.event_count, 2);
        assert_eq!(meta.created_at, 42);
    }

    #[test]
    fn snapshot_meta_round_trips_through_ndjson_codec() {
        let event = make_event(7, EventKind::Completed, Phase::Done);
        let line = encode_control_event_ndjson(&event);
        let decoded = decode_control_event_ndjson(&line).unwrap();
        assert_eq!(decoded.seq, event.seq);
        assert_eq!(decoded.kind, event.kind);
        assert_eq!(decoded.to, event.to);
    }

    #[test]
    fn read_write_snapshot_meta_round_trips() {
        let dir = std::env::temp_dir().join(format!("ai-snap-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("snap.meta.json");

        let meta = TlogSnapshotMeta {
            tlog_seq: 42,
            state_hash: 0xdeadbeef,
            event_count: 10,
            created_at: 1_000_000,
        };
        write_snapshot_meta(&path, &meta).unwrap();
        let loaded = read_snapshot_meta(&path).unwrap().unwrap();
        assert_eq!(loaded, meta);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_snapshot_meta_returns_none_for_missing_file() {
        let path = std::path::Path::new("/nonexistent/path/snap.meta.json");
        assert!(read_snapshot_meta(path).unwrap().is_none());
    }
}
