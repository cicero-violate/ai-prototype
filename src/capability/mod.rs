//! Pluggable evidence producers.
//!
//! Capabilities own rich records, policy lookup, thresholds, and external work.
//! They submit only kernel-visible evidence tokens through the runtime.

use crate::kernel::{Evidence, GateId, State};

pub mod context;
pub mod eval;
pub mod judgment;
pub mod learning;
pub mod llm;
pub mod memory;
pub mod observation;
pub mod orchestration;
pub mod planning;
pub mod policy;
pub mod tooling;
pub mod verification;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PacketEffect {
    #[default]
    None,
    BindReadyTask,
    MaterializeArtifact,
    RepairLineage,
    CompleteObjective,
}

impl PacketEffect {
    pub fn apply_to(self, state: &mut State) {
        match self {
            Self::None => {}
            Self::BindReadyTask => state.packet.bind_ready_task(),
            Self::MaterializeArtifact => state.packet.materialize_artifact(),
            Self::RepairLineage => state.packet.repair_lineage(),
            Self::CompleteObjective => state.packet.complete_objective(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvidenceSubmission {
    pub gate: GateId,
    pub evidence: Evidence,
    pub passed: bool,
    pub effect: PacketEffect,
}

impl EvidenceSubmission {
    pub const fn new(gate: GateId, evidence: Evidence, passed: bool) -> Self {
        Self {
            gate,
            evidence,
            passed,
            effect: PacketEffect::None,
        }
    }

    pub const fn with_effect(
        gate: GateId,
        evidence: Evidence,
        passed: bool,
        effect: PacketEffect,
    ) -> Self {
        Self {
            gate,
            evidence,
            passed,
            effect,
        }
    }

    pub fn apply_to(self, state: &mut State) {
        if self.passed {
            self.effect.apply_to(state);
        }

        state.apply_evidence(self.gate, self.evidence, self.passed);
    }
}

pub trait EvidenceProducer {
    type Record;

    fn record(&self) -> &Self::Record;
    fn submission(&self) -> EvidenceSubmission;
}
