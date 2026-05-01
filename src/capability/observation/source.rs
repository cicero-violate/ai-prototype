//! Bounded file-backed observation ingress.
//!
//! This module is intentionally outside the kernel. It reads one append-only
//! line source, converts bounded unseen lines into `ObservationRecord`s, and
//! persists only the observation cursor. Backpressure is explicit: if unseen
//! frames exceed the configured backlog cap, no records are emitted and the
//! cursor is not advanced.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::capability::EvidenceSubmission;
use crate::kernel::{mix, Evidence, GateId};

use super::record::{
    ObservationCursor, ObservationFrame, ObservationFrameKind, ObservationRecord,
    MAX_OBSERVATION_PAYLOAD_BYTES,
};

pub const OBSERVATION_CURSOR_SCHEMA_VERSION: u64 = 1;
pub const OBSERVATION_CURSOR_RECORD: u64 = 0x0b5e_0001;
pub const DEFAULT_OBSERVATION_BATCH_FRAMES: usize = 8;
pub const DEFAULT_OBSERVATION_BACKLOG_FRAMES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ObservationIngressDecision {
    Accepted = 1,
    Empty = 2,
    Backpressure = 3,
    Rejected = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservationIngressConfig {
    pub source_id: u64,
    pub max_batch_frames: usize,
    pub max_backlog_frames: usize,
    pub received_at_tick: u64,
}

impl ObservationIngressConfig {
    pub const fn new(
        source_id: u64,
        max_batch_frames: usize,
        max_backlog_frames: usize,
        received_at_tick: u64,
    ) -> Self {
        Self {
            source_id,
            max_batch_frames,
            max_backlog_frames,
            received_at_tick,
        }
    }

    pub const fn default_for_source(source_id: u64, received_at_tick: u64) -> Self {
        Self::new(
            source_id,
            DEFAULT_OBSERVATION_BATCH_FRAMES,
            DEFAULT_OBSERVATION_BACKLOG_FRAMES,
            received_at_tick,
        )
    }

    pub fn is_valid(self) -> bool {
        self.source_id != 0
            && self.max_batch_frames != 0
            && self.max_backlog_frames != 0
            && self.max_batch_frames <= self.max_backlog_frames
            && self.received_at_tick != 0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationIngressBatch {
    pub decision: ObservationIngressDecision,
    pub source_id: u64,
    pub source_hash: u64,
    pub cursor: ObservationCursor,
    pub backlog_len: usize,
    pub records: Vec<ObservationRecord>,
}

impl ObservationIngressBatch {
    pub fn accepted(
        source_id: u64,
        source_hash: u64,
        cursor: ObservationCursor,
        backlog_len: usize,
        records: Vec<ObservationRecord>,
    ) -> Self {
        Self {
            decision: ObservationIngressDecision::Accepted,
            source_id,
            source_hash,
            cursor,
            backlog_len,
            records,
        }
    }

    pub fn empty(source_id: u64, source_hash: u64, cursor: ObservationCursor) -> Self {
        Self {
            decision: ObservationIngressDecision::Empty,
            source_id,
            source_hash,
            cursor,
            backlog_len: 0,
            records: Vec::new(),
        }
    }

    pub fn backpressure(
        source_id: u64,
        source_hash: u64,
        cursor: ObservationCursor,
        backlog_len: usize,
    ) -> Self {
        Self {
            decision: ObservationIngressDecision::Backpressure,
            source_id,
            source_hash,
            cursor,
            backlog_len,
            records: Vec::new(),
        }
    }

    pub fn rejected(source_id: u64, cursor: ObservationCursor) -> Self {
        Self {
            decision: ObservationIngressDecision::Rejected,
            source_id,
            source_hash: 0,
            cursor,
            backlog_len: 0,
            records: Vec::new(),
        }
    }

    pub fn is_accepted(&self) -> bool {
        self.decision == ObservationIngressDecision::Accepted
            && !self.records.is_empty()
            && self.cursor.last_sequence != 0
            && self.cursor.last_observed_hash != 0
    }

    pub fn is_contract_valid(&self) -> bool {
        if !self.is_accepted()
            || self.source_id == 0
            || self.source_hash == 0
            || self.cursor.source_id != self.source_id
        {
            return false;
        }

        let mut expected_previous_sequence = 0;
        for record in &self.records {
            if !record.is_valid()
                || record.source_id != self.source_id
                || record.sequence <= expected_previous_sequence
            {
                return false;
            }
            expected_previous_sequence = record.sequence;
        }

        self.records
            .last()
            .map(|last| {
                self.cursor.last_sequence == last.sequence
                    && self.cursor.last_observed_hash == last.observed_hash
            })
            .unwrap_or(false)
    }

    pub fn submission(&self) -> EvidenceSubmission {
        EvidenceSubmission::with_payload(
            GateId::Invariant,
            Evidence::InvariantProof,
            self.is_contract_valid(),
            self.contract_hash(),
        )
    }

    pub fn contract_hash(&self) -> u64 {
        let mut h = 0x5f37_59df_6a09_e667u64;
        h = mix(h, self.decision as u64);
        h = mix(h, self.source_id);
        h = mix(h, self.source_hash);
        h = mix(h, self.cursor.source_id);
        h = mix(h, self.cursor.last_sequence);
        h = mix(h, self.cursor.last_observed_hash);
        h = mix(h, self.backlog_len as u64);
        h = mix(h, self.records.len() as u64);
        for record in &self.records {
            h = mix(h, record.source_id);
            h = mix(h, record.sequence);
            h = mix(h, record.observed_hash);
            h = mix(h, record.received_at_tick);
        }
        h.max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundedLineObservationSource {
    pub source_path: PathBuf,
    pub cursor_path: PathBuf,
    pub config: ObservationIngressConfig,
}

impl BoundedLineObservationSource {
    pub fn new(
        source_path: impl Into<PathBuf>,
        cursor_path: impl Into<PathBuf>,
        config: ObservationIngressConfig,
    ) -> Self {
        Self {
            source_path: source_path.into(),
            cursor_path: cursor_path.into(),
            config,
        }
    }

    pub fn read_batch(&self) -> io::Result<ObservationIngressBatch> {
        if !self.config.is_valid() {
            return Ok(ObservationIngressBatch::rejected(
                self.config.source_id,
                ObservationCursor::new(self.config.source_id),
            ));
        }

        let bytes = fs::read(&self.source_path)?;
        let source_hash = observation_source_hash(&bytes);
        let mut cursor = load_observation_cursor_ndjson(&self.cursor_path)?
            .unwrap_or_else(|| ObservationCursor::new(self.config.source_id));

        if cursor.source_id != self.config.source_id {
            return Ok(ObservationIngressBatch::rejected(
                self.config.source_id,
                cursor,
            ));
        }

        let frames = parse_line_frames(&bytes, self.config);
        let unseen = frames
            .iter()
            .filter(|frame| frame.sequence > cursor.last_sequence)
            .count();

        if unseen == 0 {
            return Ok(ObservationIngressBatch::empty(
                self.config.source_id,
                source_hash,
                cursor,
            ));
        }

        if unseen > self.config.max_backlog_frames {
            return Ok(ObservationIngressBatch::backpressure(
                self.config.source_id,
                source_hash,
                cursor,
                unseen,
            ));
        }

        let start_sequence = cursor.last_sequence;
        let mut records = Vec::new();
        for frame in frames
            .iter()
            .filter(|frame| frame.sequence > start_sequence)
            .take(self.config.max_batch_frames)
        {
            let record = cursor.ingest(frame);
            if record.is_valid() {
                records.push(record);
            }
        }

        if records.is_empty() {
            return Ok(ObservationIngressBatch::rejected(
                self.config.source_id,
                cursor,
            ));
        }

        write_observation_cursor_ndjson(&self.cursor_path, cursor)?;
        Ok(ObservationIngressBatch::accepted(
            self.config.source_id,
            source_hash,
            cursor,
            unseen.saturating_sub(records.len()),
            records,
        ))
    }
}

pub fn encode_observation_cursor_ndjson(cursor: ObservationCursor) -> String {
    format!(
        "[{},{},{},{},{}]\n",
        OBSERVATION_CURSOR_SCHEMA_VERSION,
        OBSERVATION_CURSOR_RECORD,
        cursor.source_id,
        cursor.last_sequence,
        cursor.last_observed_hash
    )
}

pub fn decode_observation_cursor_ndjson(line: &str) -> Option<ObservationCursor> {
    let body = line.trim().strip_prefix('[')?.strip_suffix(']')?;
    let fields = body
        .split(',')
        .map(|raw| raw.trim().parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .ok()?;

    if fields.len() != 5
        || fields[0] != OBSERVATION_CURSOR_SCHEMA_VERSION
        || fields[1] != OBSERVATION_CURSOR_RECORD
        || fields[2] == 0
    {
        return None;
    }

    Some(ObservationCursor {
        source_id: fields[2],
        last_sequence: fields[3],
        last_observed_hash: fields[4],
    })
}

pub fn load_observation_cursor_ndjson(path: impl AsRef<Path>) -> io::Result<Option<ObservationCursor>> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content
            .lines()
            .rev()
            .find_map(decode_observation_cursor_ndjson)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn write_observation_cursor_ndjson(
    path: impl AsRef<Path>,
    cursor: ObservationCursor,
) -> io::Result<()> {
    fs::write(path, encode_observation_cursor_ndjson(cursor))
}

fn parse_line_frames(bytes: &[u8], config: ObservationIngressConfig) -> Vec<ObservationFrame> {
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .enumerate()
        .map(|(idx, line)| {
            ObservationFrame::from_payload(
                ObservationFrameKind::ExternalSignal,
                config.source_id,
                (idx as u64).saturating_add(1),
                config.received_at_tick.saturating_add(idx as u64).saturating_add(1),
                trim_carriage_return(line),
            )
        })
        .collect()
}

fn trim_carriage_return(line: &[u8]) -> &[u8] {
    if let Some((last, prefix)) = line.split_last() {
        if *last == b'\r' {
            return prefix;
        }
    }
    line
}

fn observation_source_hash(bytes: &[u8]) -> u64 {
    if bytes.is_empty() {
        return 0;
    }

    let mut h = 0x6a09_e667_f3bc_c909u64;
    h = mix(h, bytes.len() as u64);
    for byte in bytes.iter().take(MAX_OBSERVATION_PAYLOAD_BYTES * 16) {
        h = mix(h, *byte as u64);
    }
    h.max(1)
}