//! Policy promotion payloads derived from TLog history.

use crate::kernel::{ControlEvent, EventKind, Evidence, Phase};

pub const POLICY_PROMOTION_SOURCE_SEQ: &str = "learning.policy_promotion.source_seq";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyPromotion {
    pub source_seq: u64,
    pub promoted_policy_version: u64,
    pub evidence: Evidence,
}

impl PolicyPromotion {
    pub fn from_tlog(tlog: &[ControlEvent], promoted_policy_version: u64) -> Option<Self> {
        tlog.iter()
            .rev()
            .find(|event| {
                event.kind == EventKind::Advanced
                    && event.to == Phase::Persist
                    && event.evidence == Evidence::EvalScore
            })
            .map(|event| Self {
                source_seq: event.seq,
                promoted_policy_version,
                evidence: Evidence::PolicyPromotion,
            })
    }

    pub fn is_valid(&self) -> bool {
        self.source_seq != 0
            && self.promoted_policy_version != 0
            && self.evidence == Evidence::PolicyPromotion
    }
}
