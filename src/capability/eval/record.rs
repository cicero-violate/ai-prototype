//! Durable eval payload owned by the eval capability.

use crate::capability::{EvidenceProducer, EvidenceSubmission};
use crate::kernel::{Evidence, GateId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvalDimension {
    pub id: &'static str,
    pub score: u64,
    pub threshold: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalRecord {
    pub score: u64,
    pub dimensions: Vec<EvalDimension>,
    pub threshold_used: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalDecision {
    Pass,
    Fail,
}

impl EvalRecord {
    pub fn decision(&self) -> EvalDecision {
        if self.score >= self.threshold_used
            && !self.dimensions.is_empty()
            && self
                .dimensions
                .iter()
                .all(|dimension| dimension.score >= dimension.threshold)
        {
            EvalDecision::Pass
        } else {
            EvalDecision::Fail
        }
    }

    pub fn submission(&self) -> EvidenceSubmission {
        EvidenceSubmission {
            gate: GateId::Eval,
            evidence: Evidence::EvalScore,
            passed: self.decision() == EvalDecision::Pass,
        }
    }
}

impl EvidenceProducer for EvalRecord {
    type Record = EvalRecord;

    fn record(&self) -> &Self::Record {
        self
    }

    fn submission(&self) -> EvidenceSubmission {
        EvalRecord::submission(self)
    }
}
