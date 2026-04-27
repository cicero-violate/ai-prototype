//! Durable observation payload owned outside the kernel.

use crate::capability::{EvidenceProducer, EvidenceSubmission};
use crate::kernel::{mix, Evidence, GateId};

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
