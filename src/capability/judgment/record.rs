//! Judgment payload owned outside the kernel.

use crate::capability::{EvidenceProducer, EvidenceSubmission};
use crate::kernel::{mix, Evidence, GateId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JudgmentRecord {
    pub decision_id: u64,
    pub policy_version: u64,
    pub rationale_hash: u64,
}

impl JudgmentRecord {
    pub fn is_valid(&self) -> bool {
        self.decision_id != 0 && self.policy_version != 0 && self.rationale_hash != 0
    }

    pub fn submission(&self) -> EvidenceSubmission {
        EvidenceSubmission::with_payload(
            GateId::Judgment,
            Evidence::JudgmentRecord,
            self.is_valid(),
            judgment_payload_hash(self),
        )
    }
}

impl EvidenceProducer for JudgmentRecord {
    type Record = JudgmentRecord;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        JudgmentRecord::submission(self)
    }
}

fn judgment_payload_hash(record: &JudgmentRecord) -> u64 {
    let mut h = 0xaf63_dc4c_8601_ec8cu64;
    h = mix(h, record.decision_id);
    h = mix(h, record.policy_version);
    h = mix(h, record.rationale_hash);
    h.max(1)
}

