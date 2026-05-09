//! Bounded file-backed observation ingress.
//!
//! This module is intentionally outside the kernel. It reads one append-only
//! line source, converts bounded unseen lines into `ObservationRecord`s, and
//! persists only the observation cursor. Backpressure is explicit: if unseen
//! frames exceed the configured backlog cap, no records are emitted and the
//! cursor is not advanced.

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::capability::EvidenceSubmission;
use crate::kernel::{mix, Evidence, GateId};

use super::record::{
    ObservationCursor, ObservationFrame, ObservationFrameKind, ObservationRecord,
    MAX_OBSERVATION_PAYLOAD_BYTES,
};

pub const OBSERVATION_CURSOR_SCHEMA_VERSION: u64 = 1;
pub const OBSERVATION_CURSOR_RECORD: u64 = 0x0b5e_0001;
pub const OBSERVATION_INGRESS_RECEIPT_SCHEMA_VERSION: u64 = 1;
pub const OBSERVATION_INGRESS_RECEIPT_RECORD: u64 = 0x0b5e_1001;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservationIngressReceipt {
    pub schema_version: u64,
    pub record_type: u64,
    pub decision: ObservationIngressDecision,
    pub source_id: u64,
    pub source_hash: u64,
    pub cursor_source_id: u64,
    pub cursor_last_sequence: u64,
    pub cursor_last_observed_hash: u64,
    pub backlog_len: u64,
    pub record_count: u64,
    pub first_sequence: u64,
    pub last_sequence: u64,
    pub records_hash: u64,
    pub contract_hash: u64,
    pub contract_valid: bool,
    pub receipt_hash: u64,
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

    pub fn receipt(&self) -> ObservationIngressReceipt {
        ObservationIngressReceipt::from_batch(self)
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

impl ObservationIngressReceipt {
    pub fn from_batch(batch: &ObservationIngressBatch) -> Self {
        let first_sequence = batch
            .records
            .first()
            .map(|record| record.sequence)
            .unwrap_or(0);
        let last_sequence = batch
            .records
            .last()
            .map(|record| record.sequence)
            .unwrap_or(0);
        let records_hash = observation_records_hash(&batch.records);
        let contract_hash = batch.contract_hash();
        let contract_valid = batch.is_contract_valid();
        let mut receipt = Self {
            schema_version: OBSERVATION_INGRESS_RECEIPT_SCHEMA_VERSION,
            record_type: OBSERVATION_INGRESS_RECEIPT_RECORD,
            decision: batch.decision,
            source_id: batch.source_id,
            source_hash: batch.source_hash,
            cursor_source_id: batch.cursor.source_id,
            cursor_last_sequence: batch.cursor.last_sequence,
            cursor_last_observed_hash: batch.cursor.last_observed_hash,
            backlog_len: batch.backlog_len as u64,
            record_count: batch.records.len() as u64,
            first_sequence,
            last_sequence,
            records_hash,
            contract_hash,
            contract_valid,
            receipt_hash: 0,
        };
        receipt.receipt_hash = receipt.expected_receipt_hash();
        receipt
    }

    pub fn is_valid_for(self, batch: &ObservationIngressBatch) -> bool {
        self == ObservationIngressReceipt::from_batch(batch) && self.is_self_consistent()
    }

    pub fn is_self_consistent(self) -> bool {
        self.schema_version == OBSERVATION_INGRESS_RECEIPT_SCHEMA_VERSION
            && self.record_type == OBSERVATION_INGRESS_RECEIPT_RECORD
            && self.source_id != 0
            && self.cursor_source_id == self.source_id
            && self.contract_hash != 0
            && self.records_hash != 0
            && self.receipt_hash == self.expected_receipt_hash()
            && match self.decision {
                ObservationIngressDecision::Accepted => {
                    self.source_hash != 0
                        && self.contract_valid
                        && self.record_count != 0
                        && self.first_sequence != 0
                        && self.last_sequence >= self.first_sequence
                        && self.cursor_last_sequence == self.last_sequence
                        && self.cursor_last_observed_hash != 0
                }
                ObservationIngressDecision::Empty => {
                    !self.contract_valid
                        && self.source_hash != 0
                        && self.record_count == 0
                        && self.backlog_len == 0
                        && self.first_sequence == 0
                        && self.last_sequence == 0
                }
                ObservationIngressDecision::Backpressure => {
                    !self.contract_valid
                        && self.source_hash != 0
                        && self.record_count == 0
                        && self.backlog_len != 0
                        && self.first_sequence == 0
                        && self.last_sequence == 0
                }
                ObservationIngressDecision::Rejected => {
                    !self.contract_valid
                        && self.source_hash == 0
                        && self.record_count == 0
                        && self.first_sequence == 0
                        && self.last_sequence == 0
                }
            }
    }

    pub fn submission(self) -> EvidenceSubmission {
        EvidenceSubmission::with_payload(
            GateId::Invariant,
            Evidence::InvariantProof,
            self.is_self_consistent()
                && self.contract_valid
                && self.decision == ObservationIngressDecision::Accepted,
            self.receipt_hash,
        )
    }

    pub fn expected_receipt_hash(self) -> u64 {
        let mut h = 0x0b5e_1001_fee1_0001u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.record_type);
        h = mix(h, self.decision as u64);
        h = mix(h, self.source_id);
        h = mix(h, self.source_hash);
        h = mix(h, self.cursor_source_id);
        h = mix(h, self.cursor_last_sequence);
        h = mix(h, self.cursor_last_observed_hash);
        h = mix(h, self.backlog_len);
        h = mix(h, self.record_count);
        h = mix(h, self.first_sequence);
        h = mix(h, self.last_sequence);
        h = mix(h, self.records_hash);
        h = mix(h, self.contract_hash);
        h = mix(h, u64::from(self.contract_valid));
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

        let starting_cursor = cursor;
        let mut unseen = 0usize;
        let mut attempted = 0usize;
        let mut records = Vec::new();

        for frame in line_frames(&bytes, self.config) {
            if frame.sequence <= starting_cursor.last_sequence {
                continue;
            }

            unseen = unseen.saturating_add(1);
            if unseen > self.config.max_backlog_frames {
                return Ok(ObservationIngressBatch::backpressure(
                    self.config.source_id,
                    source_hash,
                    starting_cursor,
                    unseen,
                ));
            }

            if attempted < self.config.max_batch_frames {
                attempted = attempted.saturating_add(1);
                let record = cursor.ingest(&frame);
                if record.is_valid() {
                    records.push(record);
                }
            }
        }

        if unseen == 0 {
            return Ok(ObservationIngressBatch::empty(
                self.config.source_id,
                source_hash,
                starting_cursor,
            ));
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
    {
        return None;
    }

    let cursor = ObservationCursor {
        source_id: fields[2],
        last_sequence: fields[3],
        last_observed_hash: fields[4],
    };

    cursor.is_valid().then_some(cursor)
}

pub fn load_observation_cursor_ndjson(
    path: impl AsRef<Path>,
) -> io::Result<Option<ObservationCursor>> {
    match fs::read_to_string(path) {
        Ok(content) => match content.lines().rev().find(|line| !line.trim().is_empty()) {
            Some(line) => decode_observation_cursor_ndjson(line)
                .map(Some)
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "invalid observation cursor")
                }),
            None => Ok(None),
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn write_observation_cursor_ndjson(
    path: impl AsRef<Path>,
    cursor: ObservationCursor,
) -> io::Result<()> {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("observation.cursor.ndjson");
    let tmp_path = parent.join(format!(".{}.{}.tmp", file_name, std::process::id()));

    let mut tmp = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp_path)?;
    tmp.write_all(encode_observation_cursor_ndjson(cursor).as_bytes())?;
    tmp.sync_all()?;
    drop(tmp);

    fs::rename(&tmp_path, path).inspect_err(|_error| {
        let _ = fs::remove_file(&tmp_path);
    })
}

fn line_frames(
    bytes: &[u8],
    config: ObservationIngressConfig,
) -> impl Iterator<Item = ObservationFrame> + '_ {
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .enumerate()
        .map(move |(idx, line)| {
            ObservationFrame::from_payload(
                ObservationFrameKind::ExternalSignal,
                config.source_id,
                (idx as u64).saturating_add(1),
                config
                    .received_at_tick
                    .saturating_add(idx as u64)
                    .saturating_add(1),
                trim_carriage_return(line),
            )
        })
}

fn trim_carriage_return(line: &[u8]) -> &[u8] {
    if let Some((last, prefix)) = line.split_last() {
        if *last == b'\r' {
            return prefix;
        }
    }
    line
}

fn observation_records_hash(records: &[ObservationRecord]) -> u64 {
    let mut h = 0x0b5e_0000_5eca_0001u64;
    h = mix(h, records.len() as u64);
    for record in records {
        h = mix(h, record.source_id);
        h = mix(h, record.sequence);
        h = mix(h, record.observed_hash);
        h = mix(h, record.received_at_tick);
    }
    h.max(1)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn accepted_batch() -> ObservationIngressBatch {
        let records = vec![
            ObservationRecord::new(9, 1, 101, 5),
            ObservationRecord::new(9, 2, 202, 6),
        ];
        ObservationIngressBatch::accepted(
            9,
            303,
            ObservationCursor {
                source_id: 9,
                last_sequence: 2,
                last_observed_hash: 202,
            },
            1,
            records,
        )
    }

    #[test]
    fn observation_ingress_receipt_binds_accepted_batch() {
        let batch = accepted_batch();
        let receipt = batch.receipt();

        assert!(receipt.is_self_consistent());
        assert!(receipt.is_valid_for(&batch));
        assert_eq!(receipt.decision, ObservationIngressDecision::Accepted);
        assert_eq!(receipt.record_count, 2);
        assert_eq!(receipt.first_sequence, 1);
        assert_eq!(receipt.last_sequence, 2);
        assert_eq!(receipt.contract_hash, batch.contract_hash());
        assert_eq!(
            receipt.records_hash,
            observation_records_hash(&batch.records)
        );
        assert!(receipt.submission().passed);
    }

    #[test]
    fn observation_ingress_receipt_binds_backpressure_without_gate_pass() {
        let batch = ObservationIngressBatch::backpressure(
            9,
            303,
            ObservationCursor {
                source_id: 9,
                last_sequence: 0,
                last_observed_hash: 0,
            },
            7,
        );
        let receipt = batch.receipt();

        assert!(receipt.is_self_consistent());
        assert!(receipt.is_valid_for(&batch));
        assert_eq!(receipt.decision, ObservationIngressDecision::Backpressure);
        assert_eq!(receipt.record_count, 0);
        assert_eq!(receipt.backlog_len, 7);
        assert!(!receipt.contract_valid);
        assert!(!receipt.submission().passed);
    }

    #[test]
    fn observation_ingress_receipt_rejects_tampered_contract_hash() {
        let batch = accepted_batch();
        let mut receipt = batch.receipt();
        receipt.contract_hash ^= 1;

        assert!(!receipt.is_self_consistent());
        assert!(!receipt.is_valid_for(&batch));
        assert!(!receipt.submission().passed);
    }
}
