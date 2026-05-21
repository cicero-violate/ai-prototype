//! Binary/segmented TLog backend (feature = "binary-tlog").
//!
//! Adapted from canon-runtime-events/src/tlog/binary.rs.
//! Stores the same u64-array records as the NDJSON codec but as length-prefixed
//! binary frames, giving ~3-5× smaller files and O(1) record scanning.
//!
//! # Record format
//!
//! Each segment file starts with a 4-byte magic header `TLOG` (0x544C4F47 LE).
//! After the header, records are packed as:
//!
//! ```text
//! [field_count: u32 LE][field_0: u64 LE]...[field_N: u64 LE]
//! ```
//!
//! The field array is identical to the NDJSON encoding: `[schema_version, record_type, …]`.
//! This means NDJSON ↔ binary round-trips without re-interpretation.
//!
//! # Segmentation
//!
//! Files are rotated when `max_bytes` is exceeded.  Segment names are
//! zero-padded base-seq numbers: `00000000000000000000.log`.
//! An optional `.idx` stride-index file is maintained for future seek support.
//! Retention pruning drops the oldest segments when `retain_segments` is set.
//!
//! NDJSON remains the canonical truth store.  Binary files may be deleted and
//! reconstructed from NDJSON without loss of information.

use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::codec::ndjson::{control_event_from_fields, control_event_to_fields};
use crate::kernel::{CanonError, ControlEvent, TLog};

const MAGIC: u32 = 0x544C4F47; // "TLOG" LE

// ---------------------------------------------------------------------------
// Segment config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SegmentConfig {
    /// Rotate to a new segment after this many bytes.
    pub max_bytes: u64,
    /// Write a sparse index entry every N records.
    pub index_stride: u32,
    /// Keep at most this many segments (oldest pruned on rotation).
    pub retain_segments: Option<usize>,
}

impl Default for SegmentConfig {
    fn default() -> Self {
        Self {
            max_bytes: 64 * 1024 * 1024,
            index_stride: 256,
            retain_segments: None,
        }
    }
}

impl SegmentConfig {
    /// Override `retain_segments` from the `CANON_TLOG_RETAIN_SEGMENTS` env var.
    pub fn with_env(mut self) -> Self {
        if let Ok(raw) = std::env::var("CANON_TLOG_RETAIN_SEGMENTS") {
            if let Ok(value) = raw.parse::<usize>() {
                if value > 0 {
                    self.retain_segments = Some(value);
                }
            }
        }
        self
    }
}

// ---------------------------------------------------------------------------
// Format detection
// ---------------------------------------------------------------------------

/// Return `true` when `path` starts with the binary TLOG magic bytes.
pub fn is_binary_tlog(path: &Path) -> bool {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut magic = [0u8; 4];
    if file.read_exact(&mut magic).is_err() {
        return false;
    }
    u32::from_le_bytes(magic) == MAGIC
}

// ---------------------------------------------------------------------------
// Internal segment state
// ---------------------------------------------------------------------------

struct SegmentFiles {
    log: BufWriter<File>,
    idx: BufWriter<File>,
    size: u64,
    records: u32,
}

// ---------------------------------------------------------------------------
// Writer
// ---------------------------------------------------------------------------

/// Append-only writer for the binary segmented TLog.
pub struct BinaryTlogWriter {
    dir: PathBuf,
    config: SegmentConfig,
    seq: AtomicU64,
    fsync: bool,
    inner: Mutex<SegmentFiles>,
}

impl BinaryTlogWriter {
    /// Open (or create) a binary TLog directory with default config.
    pub fn open(dir: &Path) -> Result<Self, CanonError> {
        Self::open_with_config(dir, SegmentConfig::default().with_env())
    }

    /// Open (or create) a binary TLog directory with explicit config.
    pub fn open_with_config(dir: &Path, config: SegmentConfig) -> Result<Self, CanonError> {
        fs::create_dir_all(dir).map_err(|_| CanonError::TlogIo)?;

        let base_seq = find_latest_segment(dir)?.unwrap_or(0);
        let (files, recovered_seq) = recover_segment(dir, base_seq, &config)?;

        let next_seq = recovered_seq.map(|s| s + 1).unwrap_or(base_seq);

        let files = if files.size >= config.max_bytes {
            open_new_segment(dir, next_seq)?
        } else {
            files
        };

        if let Some(keep) = config.retain_segments {
            prune_old_segments(dir, keep)?;
        }

        Ok(Self {
            dir: dir.to_path_buf(),
            config,
            seq: AtomicU64::new(next_seq),
            fsync: false,
            inner: Mutex::new(files),
        })
    }

    /// Enable or disable fsync after each write.
    pub fn with_fsync(mut self, enabled: bool) -> Self {
        self.fsync = enabled;
        self
    }

    /// Append a ControlEvent to the current segment, rotating if needed.
    pub fn append(&self, event: &ControlEvent) -> Result<(), CanonError> {
        let fields = control_event_to_fields(event);
        let frame = encode_frame(&fields);
        let frame_len = frame.len() as u64;
        let seq = self.seq.fetch_add(1, Ordering::Relaxed);

        let mut guard = self.inner.lock().map_err(|_| CanonError::TlogIo)?;

        // Write magic header at start of each segment
        if guard.size == 0 {
            let magic_bytes = MAGIC.to_le_bytes();
            guard
                .log
                .write_all(&magic_bytes)
                .map_err(|_| CanonError::TlogIo)?;
            guard.size += magic_bytes.len() as u64;
        }

        // Rotate segment if needed
        if guard.size + frame_len > self.config.max_bytes {
            guard.log.flush().map_err(|_| CanonError::TlogIo)?;
            guard.idx.flush().map_err(|_| CanonError::TlogIo)?;
            *guard = open_new_segment(&self.dir, seq)?;
            if let Some(keep) = self.config.retain_segments {
                prune_old_segments(&self.dir, keep)?;
            }
        }

        let record_pos = guard.size;
        guard
            .log
            .write_all(&frame)
            .map_err(|_| CanonError::TlogIo)?;
        guard.size += frame_len;
        guard.records += 1;

        // Sparse index entry
        if guard.records % self.config.index_stride == 0 {
            guard
                .idx
                .write_all(&seq.to_le_bytes())
                .map_err(|_| CanonError::TlogIo)?;
            guard
                .idx
                .write_all(&record_pos.to_le_bytes())
                .map_err(|_| CanonError::TlogIo)?;
        }

        guard.log.flush().map_err(|_| CanonError::TlogIo)?;
        guard.idx.flush().map_err(|_| CanonError::TlogIo)?;

        if self.fsync {
            guard
                .log
                .get_ref()
                .sync_data()
                .map_err(|_| CanonError::TlogIo)?;
            guard
                .idx
                .get_ref()
                .sync_data()
                .map_err(|_| CanonError::TlogIo)?;
        }
        Ok(())
    }

    /// The directory this writer is operating on.
    pub fn path(&self) -> &Path {
        &self.dir
    }
}

// ---------------------------------------------------------------------------
// Reader
// ---------------------------------------------------------------------------

/// Read all events from a binary TLog directory, across all segments in order.
pub fn read_binary_tlog(dir: &Path) -> Result<TLog, CanonError> {
    let mut segments = collect_segments(dir)?;
    segments.sort();

    let mut tlog = Vec::new();
    for seg_seq in segments {
        let log_path = segment_log_path(dir, seg_seq);
        read_segment(&log_path, &mut tlog)?;
    }
    Ok(tlog)
}

// ---------------------------------------------------------------------------
// Frame encoding/decoding
// ---------------------------------------------------------------------------

fn encode_frame(fields: &[u64]) -> Vec<u8> {
    let count = fields.len() as u32;
    let mut frame = Vec::with_capacity(4 + fields.len() * 8);
    frame.extend_from_slice(&count.to_le_bytes());
    for &f in fields {
        frame.extend_from_slice(&f.to_le_bytes());
    }
    frame
}

fn read_segment(log_path: &Path, out: &mut Vec<ControlEvent>) -> Result<(), CanonError> {
    let bytes = fs::read(log_path).map_err(|_| CanonError::TlogIo)?;
    if bytes.is_empty() {
        return Ok(());
    }

    // Check and skip magic header
    if bytes.len() < 4 {
        return Err(CanonError::InvalidTlogRecord);
    }
    let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    if magic != MAGIC {
        return Err(CanonError::InvalidTlogRecord);
    }
    let mut pos = 4usize;

    while pos < bytes.len() {
        // Read field_count (u32 LE)
        if pos + 4 > bytes.len() {
            break; // truncated frame — stop cleanly
        }
        let count = u32::from_le_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]])
            as usize;
        pos += 4;

        // Read count u64 fields
        let field_bytes = count * 8;
        if pos + field_bytes > bytes.len() {
            break; // truncated record
        }
        let mut fields = Vec::with_capacity(count);
        for i in 0..count {
            let off = pos + i * 8;
            fields.push(u64::from_le_bytes([
                bytes[off],
                bytes[off + 1],
                bytes[off + 2],
                bytes[off + 3],
                bytes[off + 4],
                bytes[off + 5],
                bytes[off + 6],
                bytes[off + 7],
            ]));
        }
        pos += field_bytes;

        let event = control_event_from_fields(&fields)?;
        out.push(event);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Segment helpers
// ---------------------------------------------------------------------------

fn segment_log_path(dir: &Path, base_seq: u64) -> PathBuf {
    dir.join(format!("{:020}.log", base_seq))
}

fn segment_idx_path(dir: &Path, base_seq: u64) -> PathBuf {
    dir.join(format!("{:020}.idx", base_seq))
}

fn open_new_segment(dir: &Path, base_seq: u64) -> Result<SegmentFiles, CanonError> {
    let log_path = segment_log_path(dir, base_seq);
    let idx_path = segment_idx_path(dir, base_seq);
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|_| CanonError::TlogIo)?;
    let idx = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&idx_path)
        .map_err(|_| CanonError::TlogIo)?;
    let size = log.metadata().map(|m| m.len()).unwrap_or(0);
    Ok(SegmentFiles {
        log: BufWriter::new(log),
        idx: BufWriter::new(idx),
        size,
        records: 0,
    })
}

/// Find the base_seq of the most recent segment (highest-numbered .log file).
fn find_latest_segment(dir: &Path) -> Result<Option<u64>, CanonError> {
    let seqs = collect_segments(dir)?;
    Ok(seqs.into_iter().max())
}

fn collect_segments(dir: &Path) -> Result<Vec<u64>, CanonError> {
    let mut out = Vec::new();
    let rd = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return Ok(out),
    };
    for entry in rd {
        let entry = entry.map_err(|_| CanonError::TlogIo)?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("log") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if let Ok(seq) = stem.parse::<u64>() {
                out.push(seq);
            }
        }
    }
    Ok(out)
}

/// Recover the latest segment: read back existing records to determine the
/// recovered_seq and the current file size.
fn recover_segment(
    dir: &Path,
    base_seq: u64,
    _config: &SegmentConfig,
) -> Result<(SegmentFiles, Option<u64>), CanonError> {
    let log_path = segment_log_path(dir, base_seq);
    if !log_path.exists() {
        return Ok((open_new_segment(dir, base_seq)?, None));
    }

    // Count valid records to determine recovered_seq
    let bytes = fs::read(&log_path).map_err(|_| CanonError::TlogIo)?;
    let mut recovered_seq: Option<u64> = None;

    if bytes.len() >= 4 {
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if magic == MAGIC {
            let mut pos = 4usize;
            let mut seq = base_seq;
            while pos < bytes.len() {
                if pos + 4 > bytes.len() {
                    break;
                }
                let count = u32::from_le_bytes([
                    bytes[pos],
                    bytes[pos + 1],
                    bytes[pos + 2],
                    bytes[pos + 3],
                ]) as usize;
                pos += 4;
                let field_bytes = count * 8;
                if pos + field_bytes > bytes.len() {
                    break;
                }
                pos += field_bytes;
                recovered_seq = Some(seq);
                seq += 1;
            }
        }
    }

    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|_| CanonError::TlogIo)?;
    let idx_path = segment_idx_path(dir, base_seq);
    let idx = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&idx_path)
        .map_err(|_| CanonError::TlogIo)?;
    let size = log.metadata().map(|m| m.len()).unwrap_or(0);

    Ok((
        SegmentFiles {
            log: BufWriter::new(log),
            idx: BufWriter::new(idx),
            size,
            records: 0,
        },
        recovered_seq,
    ))
}

fn prune_old_segments(dir: &Path, keep: usize) -> Result<(), CanonError> {
    if keep == 0 {
        return Ok(());
    }
    let mut seqs = collect_segments(dir)?;
    seqs.sort();
    if seqs.len() <= keep {
        return Ok(());
    }
    let remove_count = seqs.len() - keep;
    for seq in seqs.into_iter().take(remove_count) {
        let _ = fs::remove_file(segment_log_path(dir, seq));
        let _ = fs::remove_file(segment_idx_path(dir, seq));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::{
        CapabilityRegistryProjection, Cause, Decision, EventKind, Evidence, GateSet, Packet, Phase,
        RuntimeConfig, SemanticDelta, State,
    };
    use std::sync::atomic::{AtomicU32, Ordering as AtomicOrd};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn scratch_dir() -> PathBuf {
        let n = COUNTER.fetch_add(1, AtomicOrd::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("ai-binary-tlog-test-{}-{n}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

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
            prev_hash: seq.wrapping_sub(1),
            self_hash: seq * 0xdeadbeef,
        }
    }

    #[test]
    fn is_binary_tlog_returns_false_for_missing_file() {
        assert!(!is_binary_tlog(Path::new("/nonexistent/tlog.log")));
    }

    #[test]
    fn is_binary_tlog_detects_magic_header() {
        let dir = scratch_dir();
        let path = dir.join("test.log");
        fs::write(&path, &MAGIC.to_le_bytes()).unwrap();
        assert!(is_binary_tlog(&path));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_and_read_single_event_roundtrip() {
        let dir = scratch_dir();
        let writer = BinaryTlogWriter::open(&dir).unwrap();
        let event = make_event(0);
        writer.append(&event).unwrap();
        drop(writer);

        let tlog = read_binary_tlog(&dir).unwrap();
        assert_eq!(tlog.len(), 1);
        assert_eq!(tlog[0].seq, event.seq);
        assert_eq!(tlog[0].from, event.from);
        assert_eq!(tlog[0].to, event.to);
        assert_eq!(tlog[0].self_hash, event.self_hash);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_multiple_events_all_round_trip() {
        let dir = scratch_dir();
        let writer = BinaryTlogWriter::open(&dir).unwrap();
        let events: Vec<_> = (0u64..10).map(make_event).collect();
        for e in &events {
            writer.append(e).unwrap();
        }
        drop(writer);

        let tlog = read_binary_tlog(&dir).unwrap();
        assert_eq!(tlog.len(), events.len());
        for (original, recovered) in events.iter().zip(tlog.iter()) {
            assert_eq!(original.seq, recovered.seq);
            assert_eq!(original.self_hash, recovered.self_hash);
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn segment_rotates_at_max_bytes() {
        let dir = scratch_dir();
        let config = SegmentConfig {
            max_bytes: 200,
            index_stride: 256,
            retain_segments: None,
        };
        let writer = BinaryTlogWriter::open_with_config(&dir, config).unwrap();
        for seq in 0u64..20 {
            writer.append(&make_event(seq)).unwrap();
        }
        drop(writer);

        let segments = {
            let mut seqs = collect_segments(&dir).unwrap();
            seqs.sort();
            seqs
        };
        assert!(
            segments.len() >= 2,
            "expected segment rotation, got {} segments",
            segments.len()
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn retain_segments_prunes_oldest() {
        let dir = scratch_dir();
        let config = SegmentConfig {
            max_bytes: 100,
            index_stride: 256,
            retain_segments: Some(2),
        };
        let writer = BinaryTlogWriter::open_with_config(&dir, config).unwrap();
        for seq in 0u64..30 {
            writer.append(&make_event(seq)).unwrap();
        }
        drop(writer);

        let seqs = collect_segments(&dir).unwrap();
        assert!(
            seqs.len() <= 2,
            "expected at most 2 segments, got {}",
            seqs.len()
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn binary_ndjson_equivalence_same_events() {
        use crate::codec::ndjson::{decode_control_event_ndjson, encode_control_event_ndjson};

        let events: Vec<_> = (0u64..5).map(make_event).collect();

        // Write via binary
        let dir = scratch_dir();
        let writer = BinaryTlogWriter::open(&dir).unwrap();
        for e in &events {
            writer.append(e).unwrap();
        }
        drop(writer);
        let binary_tlog = read_binary_tlog(&dir).unwrap();

        // Encode via NDJSON round-trip
        let ndjson_tlog: Vec<ControlEvent> = events
            .iter()
            .map(|e| decode_control_event_ndjson(&encode_control_event_ndjson(e)).unwrap())
            .collect();

        assert_eq!(binary_tlog.len(), ndjson_tlog.len());
        for (b, n) in binary_tlog.iter().zip(ndjson_tlog.iter()) {
            assert_eq!(b.seq, n.seq, "seq mismatch");
            assert_eq!(b.self_hash, n.self_hash, "self_hash mismatch");
            assert_eq!(b.from, n.from, "from mismatch");
            assert_eq!(b.to, n.to, "to mismatch");
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn writer_recovers_existing_segment_on_reopen() {
        let dir = scratch_dir();

        // Write 3 events
        {
            let writer = BinaryTlogWriter::open(&dir).unwrap();
            for seq in 0u64..3 {
                writer.append(&make_event(seq)).unwrap();
            }
        }

        // Reopen and write 2 more
        {
            let writer = BinaryTlogWriter::open(&dir).unwrap();
            for seq in 3u64..5 {
                writer.append(&make_event(seq)).unwrap();
            }
        }

        let tlog = read_binary_tlog(&dir).unwrap();
        assert_eq!(
            tlog.len(),
            5,
            "expected 5 events after reopen, got {}",
            tlog.len()
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_empty_dir_returns_empty_tlog() {
        let dir = scratch_dir();
        let tlog = read_binary_tlog(&dir).unwrap();
        assert!(tlog.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }
}
