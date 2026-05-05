#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateId { Invariant, Analysis, Judgment, Plan, Execution, Verification, Eval, Learning }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Evidence { None, InvariantProof, AnalysisReport, JudgmentRecord, TaskReady, PlanRecord, ArtifactReceipt, ExecutionReceipt, LineageProof, VerificationReport, EvalScore, PolicyPromotion }

impl Evidence {
    pub const fn is_state_evidence_for_gate(self, gate: GateId) -> bool {
        match gate {
            GateId::Invariant => matches!(self, Self::InvariantProof),
            GateId::Analysis => matches!(self, Self::AnalysisReport),
            GateId::Judgment => matches!(self, Self::JudgmentRecord),
            GateId::Plan => matches!(self, Self::TaskReady | Self::PlanRecord),
            GateId::Execution => matches!(self, Self::ArtifactReceipt | Self::ExecutionReceipt),
            GateId::Verification => matches!(self, Self::LineageProof | Self::VerificationReport),
            GateId::Eval => matches!(self, Self::EvalScore),
            GateId::Learning => matches!(self, Self::PolicyPromotion),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GateStatus { Unknown, Pass, Fail }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Gate { pub status: GateStatus, pub evidence: Evidence }

impl Gate { pub const fn pass(evidence: Evidence) -> Self { Self { status: GateStatus::Pass, evidence } } }

impl Gate {
    pub const fn is_structurally_valid_for(self, gate: GateId) -> bool {
        match self.status {
            GateStatus::Unknown => matches!(self.evidence, Evidence::None),
            GateStatus::Fail => !matches!(self.evidence, Evidence::None),
            GateStatus::Pass => self.evidence.is_state_evidence_for_gate(gate),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GateSet { pub invariant: Gate, pub analysis: Gate, pub judgment: Gate, pub plan: Gate, pub execution: Gate, pub verification: Gate, pub eval: Gate, pub learning: Gate }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilityRegistryProjection { pub route_count: u64, pub policy_hash: u64 }

impl CapabilityRegistryProjection {
    pub const fn is_empty(self) -> bool { self.route_count == 0 && self.policy_hash == 0 }

    pub const fn is_valid(self) -> bool {
        self.is_empty() || (self.route_count != 0 && self.policy_hash != 0)
    }
}
