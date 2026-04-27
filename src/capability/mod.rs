//! Pluggable evidence producers.
//!
//! Capabilities own rich records, policy lookup, thresholds, and external work.
//! They submit only kernel-visible evidence tokens through the runtime.

use crate::kernel::{Evidence, GateId, State};

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

impl EvidenceSubmission {
    pub fn apply_to(self, state: &mut State) {
        state.apply_evidence(self.gate, self.evidence, self.passed);
    }
}

pub trait EvidenceProducer {
    type Record;

    fn record(&self) -> &Self::Record;
    fn submission(&self) -> EvidenceSubmission;
}
