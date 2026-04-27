//! Judgment payload owned outside the kernel.

use crate::capability::{EvidenceProducer, EvidenceSubmission};
use crate::kernel::{Evidence, GateId};

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
        EvidenceSubmission::new(GateId::Judgment, Evidence::JudgmentRecord, self.is_valid())
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
