//! Durable observation payload owned outside the kernel.
//!
//! Observation is the first boundary where external bytes enter the system.
//! The kernel never sees raw bytes; this capability converts a bounded,
//! ordered external frame into a hash-addressed `ObservationRecord`.

use crate::capability::{EvidenceProducer, EvidenceSubmission};
use crate::kernel::{mix, Evidence, GateId};

pub const MAX_OBSERVATION_PAYLOAD_BYTES: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationDecision {
    Accepted,
    Rejected,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationRecord {
    pub source_id: u64,
    pub sequence: u64,
    pub observed_hash: u64,
    pub received_at_tick: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ObservationFrameKind {
    ExternalSignal = 1,
    Heartbeat = 2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationFrame {
    pub kind: ObservationFrameKind,
    pub source_id: u64,
    pub sequence: u64,
    pub received_at_tick: u64,
    pub payload_len: u64,
    pub payload_hash: u64,
}

impl ObservationFrame {
    pub fn from_payload(
        kind: ObservationFrameKind,
        source_id: u64,
        sequence: u64,
        received_at_tick: u64,
        payload: &[u8],
    ) -> Self {
        Self {
            kind,
            source_id,
            sequence,
            received_at_tick,
            payload_len: payload.len() as u64,
            payload_hash: observation_bytes_hash(payload),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.source_id != 0
            && self.sequence != 0
            && self.received_at_tick != 0
            && self.payload_len != 0
            && self.payload_len <= MAX_OBSERVATION_PAYLOAD_BYTES as u64
            && self.payload_hash != 0
    }

    pub fn record(&self) -> ObservationRecord {
        ObservationRecord::new(
            self.source_id,
            self.sequence,
            self.observed_hash(),
            self.received_at_tick,
        )
    }

    pub fn observed_hash(&self) -> u64 {
        let mut h = 0x9e37_79b9_7f4a_7c15u64;
        h = mix(h, self.kind as u64);
        h = mix(h, self.source_id);
        h = mix(h, self.sequence);
        h = mix(h, self.received_at_tick);
        h = mix(h, self.payload_len);
        h = mix(h, self.payload_hash);
        h.max(1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservationCursor {
    pub source_id: u64,
    pub last_sequence: u64,
    pub last_observed_hash: u64,
}

impl ObservationCursor {
    pub const fn new(source_id: u64) -> Self {
        Self {
            source_id,
            last_sequence: 0,
            last_observed_hash: 0,
        }
    }

    pub fn ingest(&mut self, frame: &ObservationFrame) -> ObservationRecord {
        if self.accepts(frame) {
            let record = frame.record();
            self.last_sequence = frame.sequence;
            self.last_observed_hash = record.observed_hash;
            record
        } else {
            ObservationRecord::new(frame.source_id, frame.sequence, 0, frame.received_at_tick)
        }
    }

    pub fn accepts(&self, frame: &ObservationFrame) -> bool {
        self.source_id != 0
            && frame.is_valid()
            && frame.source_id == self.source_id
            && frame.sequence > self.last_sequence
    }
}

impl ObservationRecord {
    pub const fn new(
        source_id: u64,
        sequence: u64,
        observed_hash: u64,
        received_at_tick: u64,
    ) -> Self {
        Self {
            source_id,
            sequence,
            observed_hash,
            received_at_tick,
        }
    }

    pub fn decision(&self) -> ObservationDecision {
        if self.is_valid() {
            ObservationDecision::Accepted
        } else {
            ObservationDecision::Rejected
        }
    }

    pub fn is_valid(&self) -> bool {
        self.source_id != 0
            && self.sequence != 0
            && self.observed_hash != 0
            && self.received_at_tick != 0
    }

    pub fn submission(&self) -> EvidenceSubmission {
        EvidenceSubmission::with_payload(
            GateId::Invariant,
            Evidence::InvariantProof,
            self.decision() == ObservationDecision::Accepted,
            observation_payload_hash(self),
        )
    }
}

impl EvidenceProducer for ObservationRecord {
    type Record = ObservationRecord;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        ObservationRecord::submission(self)
    }
}
fn observation_payload_hash(record: &ObservationRecord) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    h = mix(h, record.source_id);
    h = mix(h, record.sequence);
    h = mix(h, record.observed_hash);
    h = mix(h, record.received_at_tick);
    h.max(1)
}

fn observation_bytes_hash(bytes: &[u8]) -> u64 {
    if bytes.is_empty() || bytes.len() > MAX_OBSERVATION_PAYLOAD_BYTES {
        return 0;
    }

    let mut h = 0x243f_6a88_85a3_08d3u64;
    h = mix(h, bytes.len() as u64);
    for byte in bytes {
        h = mix(h, *byte as u64);
    }
    h.max(1)
}
