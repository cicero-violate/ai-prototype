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
#[repr(u8)]
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

impl PacketEffect {
    pub const fn expected_for_gate(gate: GateId) -> Self {
        match gate {
            GateId::Plan => Self::BindReadyTask,
            GateId::Execution => Self::MaterializeArtifact,
            GateId::Verification => Self::RepairLineage,
            GateId::Eval => Self::CompleteObjective,
            GateId::Invariant | GateId::Analysis | GateId::Judgment | GateId::Learning => {
                Self::None
            }
        }
    }
}

pub const fn expected_evidence_for_gate(gate: GateId) -> Evidence {
    match gate {
        GateId::Invariant => Evidence::InvariantProof,
        GateId::Analysis => Evidence::AnalysisReport,
        GateId::Judgment => Evidence::JudgmentRecord,
        GateId::Plan => Evidence::TaskReady,
        GateId::Execution => Evidence::ArtifactReceipt,
        GateId::Verification => Evidence::LineageProof,
        GateId::Eval => Evidence::EvalScore,
        GateId::Learning => Evidence::PolicyPromotion,
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvidenceSubmission {
    pub gate: GateId,
    pub evidence: Evidence,
    pub passed: bool,
    pub effect: PacketEffect,
    pub payload_hash: u64,
}

impl EvidenceSubmission {
    pub const fn new(gate: GateId, evidence: Evidence, passed: bool) -> Self {
        Self::with_effect_payload(
            gate,
            evidence,
            passed,
            PacketEffect::None,
            structural_payload_hash(gate, evidence, passed, PacketEffect::None),
        )
    }

    pub const fn with_payload(
        gate: GateId,
        evidence: Evidence,
        passed: bool,
        payload_hash: u64,
    ) -> Self {
        Self::with_effect_payload(gate, evidence, passed, PacketEffect::None, payload_hash)
    }

    pub const fn with_effect(
        gate: GateId,
        evidence: Evidence,
        passed: bool,
        effect: PacketEffect,
    ) -> Self {
        Self::with_effect_payload(
            gate,
            evidence,
            passed,
            effect,
            structural_payload_hash(gate, evidence, passed, effect),
        )
    }

    pub const fn with_effect_payload(
        gate: GateId,
        evidence: Evidence,
        passed: bool,
        effect: PacketEffect,
        payload_hash: u64,
    ) -> Self {
        Self {
            gate,
            evidence,
            passed,
            effect,
            payload_hash,
        }
    }

    pub fn is_contract_valid(self) -> bool {
        self.payload_hash != 0
            && self.evidence == expected_evidence_for_gate(self.gate)
            && if self.passed {
                self.effect == PacketEffect::expected_for_gate(self.gate)
            } else {
                self.effect == PacketEffect::None
            }
    }

    pub fn contract_hash(self) -> u64 {
        let mut h = 0xcbf29ce484222325u64;
        h ^= self.gate as u64;
        h = h.wrapping_mul(0x100000001b3);
        h ^= self.evidence as u64;
        h = h.wrapping_mul(0x100000001b3);
        h ^= self.passed as u64;
        h = h.wrapping_mul(0x100000001b3);
        h ^= self.effect as u64;
        h = h.wrapping_mul(0x100000001b3);
        h ^= self.payload_hash;
        h = h.wrapping_mul(0x100000001b3);
        h.max(1)
    }

    pub fn apply_to(self, state: &mut State) {
        if self.passed {
            self.effect.apply_to(state);
        }

        state.apply_evidence(self.gate, self.evidence, self.passed);
    }
}

const fn structural_payload_hash(
    gate: GateId,
    evidence: Evidence,
    passed: bool,
    effect: PacketEffect,
) -> u64 {
    let mut h = 0x8422_2325_cbf2_9ce4u64;
    h ^= gate as u64;
    h = h.wrapping_mul(0x100000001b3);
    h ^= evidence as u64;
    h = h.wrapping_mul(0x100000001b3);
    h ^= passed as u64;
    h = h.wrapping_mul(0x100000001b3);
    h ^= effect as u64;
    h = h.wrapping_mul(0x100000001b3);
    if h == 0 { 1 } else { h }
}

pub trait EvidenceProducer {
    type Record;

    fn record(&self) -> &Self::Record;
    fn submission(&self) -> EvidenceSubmission;
}
