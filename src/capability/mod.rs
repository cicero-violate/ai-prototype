//! Pluggable evidence producers.
//!
//! Capabilities own rich records, policy lookup, thresholds, and external work.
//! They submit only kernel-visible evidence tokens through the runtime.

use crate::kernel::{Evidence, GateId};

pub mod eval;
pub mod judgment;
pub mod learning;
pub mod policy;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvidenceSubmission {
    pub gate: GateId,
    pub evidence: Evidence,
    pub passed: bool,
}

pub trait EvidenceProducer {
    type Record;

    fn record(&self) -> &Self::Record;
    fn submission(&self) -> EvidenceSubmission;
}

// TODO: define command intake once the external API protocol is fixed.
