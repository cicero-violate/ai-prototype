//! Durable eval payload owned by the eval capability.

use crate::capability::{EvidenceProducer, EvidenceSubmission, PacketEffect};
use crate::kernel::{mix, Evidence, GateId};

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
        let passed = self.decision() == EvalDecision::Pass;
        EvidenceSubmission::with_effect_payload(
            GateId::Eval,
            Evidence::EvalScore,
            passed,
            if passed {
                PacketEffect::CompleteObjective
            } else {
                PacketEffect::None
            },
            eval_payload_hash(self),
        )
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

fn eval_payload_hash(record: &EvalRecord) -> u64 {
    let mut h = 0x510e_527f_ade6_82d1u64;
    h = mix(h, record.score);
    h = mix(h, record.threshold_used);
    for dimension in &record.dimensions {
        h = mix(h, dimension_id_hash(dimension.id));
        h = mix(h, dimension.score);
        h = mix(h, dimension.threshold);
    }
    h.max(1)
}

fn dimension_id_hash(id: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for byte in id.as_bytes() {
        h ^= u64::from(*byte);
        h = h.wrapping_mul(0x100000001b3);
    }
    h.max(1)
}

