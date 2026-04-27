//! Policy promotion payloads derived from TLog history.

use crate::kernel::Evidence;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PolicyPromotion {
    pub source_seq: u64,
    pub promoted_policy_version: u64,
    pub evidence: Evidence,
}

// TODO: read EvalRecord history and emit durable threshold promotions.
