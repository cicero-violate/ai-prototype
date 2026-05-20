#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GateStatus {
    Unknown = 1,
    Pass = 2,
    Fail = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GateId {
    Invariant = 1,
    Analysis = 2,
    Judgment = 3,
    Plan = 4,
    Execution = 5,
    Verification = 6,
    Eval = 7,
    Learning = 8,
}

pub const EXECUTION_GATE_ORDER: [GateId; 7] = [
    GateId::Invariant,
    GateId::Analysis,
    GateId::Judgment,
    GateId::Plan,
    GateId::Execution,
    GateId::Verification,
    GateId::Eval,
];

pub const GATE_ORDER: [GateId; 8] = [
    GateId::Invariant,
    GateId::Analysis,
    GateId::Judgment,
    GateId::Plan,
    GateId::Execution,
    GateId::Verification,
    GateId::Eval,
    GateId::Learning,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Evidence {
    Missing = 1,
    DeltaComputed = 2,
    InvariantProof = 3,
    AnalysisReport = 4,
    JudgmentRecord = 5,
    PlanRecord = 6,
    TaskReady = 7,
    ExecutionReceipt = 8,
    ArtifactReceipt = 9,
    VerificationReport = 10,
    LineageProof = 11,
    EvalScore = 12,
    RecoveryPolicy = 13,
    CompletionProof = 14,
    ConvergenceLimit = 15,
    PersistedRecord = 16,
    LearningRecord = 17,
    PolicyPromotion = 18,
    AgentCycleEvent = 19,
    WaveDispatched = 20,
    ChildTaskComplete = 21,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Gate {
    pub status: GateStatus,
    pub evidence: Evidence,
    pub version: u64,
}

impl Gate {
    const fn known(status: GateStatus, evidence: Evidence) -> Self {
        Self {
            status,
            evidence,
            version: 1,
        }
    }

    pub const fn unknown() -> Self {
        Self {
            status: GateStatus::Unknown,
            evidence: Evidence::Missing,
            version: 0,
        }
    }

    pub const fn pass(evidence: Evidence) -> Self {
        Self::known(GateStatus::Pass, evidence)
    }

    pub const fn fail(evidence: Evidence) -> Self {
        Self::known(GateStatus::Fail, evidence)
    }

    pub fn is_structurally_valid(self) -> bool {
        match self.status {
            GateStatus::Unknown => self.evidence == Evidence::Missing && self.version == 0,
            GateStatus::Pass | GateStatus::Fail => {
                self.evidence != Evidence::Missing && self.version != 0
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GateSet {
    pub invariant: Gate,
    pub analysis: Gate,
    pub judgment: Gate,
    pub plan: Gate,
    pub execution: Gate,
    pub verification: Gate,
    pub eval: Gate,
    pub learning: Gate,
}

impl Default for GateSet {
    fn default() -> Self {
        Self {
            invariant: Gate::unknown(),
            analysis: Gate::unknown(),
            judgment: Gate::unknown(),
            plan: Gate::unknown(),
            execution: Gate::unknown(),
            verification: Gate::unknown(),
            eval: Gate::unknown(),
            learning: Gate::unknown(),
        }
    }
}

impl GateSet {
    pub fn ready() -> Self {
        Self {
            invariant: Gate::pass(Evidence::InvariantProof),
            analysis: Gate::pass(Evidence::AnalysisReport),
            judgment: Gate::pass(Evidence::JudgmentRecord),
            plan: Gate::pass(Evidence::TaskReady),
            execution: Gate::pass(Evidence::ArtifactReceipt),
            verification: Gate::pass(Evidence::LineageProof),
            eval: Gate::pass(Evidence::EvalScore),
            learning: Gate::pass(Evidence::PolicyPromotion),
        }
    }

    pub fn get(self, id: GateId) -> Gate {
        match id {
            GateId::Invariant => self.invariant,
            GateId::Analysis => self.analysis,
            GateId::Judgment => self.judgment,
            GateId::Plan => self.plan,
            GateId::Execution => self.execution,
            GateId::Verification => self.verification,
            GateId::Eval => self.eval,
            GateId::Learning => self.learning,
        }
    }

    pub fn get_mut(&mut self, id: GateId) -> &mut Gate {
        match id {
            GateId::Invariant => &mut self.invariant,
            GateId::Analysis => &mut self.analysis,
            GateId::Judgment => &mut self.judgment,
            GateId::Plan => &mut self.plan,
            GateId::Execution => &mut self.execution,
            GateId::Verification => &mut self.verification,
            GateId::Eval => &mut self.eval,
            GateId::Learning => &mut self.learning,
        }
    }

    pub fn set_pass(&mut self, id: GateId, evidence: Evidence) {
        self.set(id, GateStatus::Pass, evidence);
    }

    pub fn set_fail(&mut self, id: GateId, evidence: Evidence) {
        self.set(id, GateStatus::Fail, evidence);
    }

    fn set(&mut self, id: GateId, status: GateStatus, evidence: Evidence) {
        let gate = self.get_mut(id);
        *gate = Gate {
            status,
            evidence,
            version: gate.version.saturating_add(1),
        };
    }

    pub fn all_passed(self) -> bool {
        self.all_passed_in(&GATE_ORDER)
    }

    pub fn all_execution_passed(self) -> bool {
        self.all_passed_in(&EXECUTION_GATE_ORDER)
    }

    pub fn first_non_pass(self) -> Option<(GateId, Gate)> {
        self.first_non_pass_in(&GATE_ORDER)
    }

    pub fn first_execution_non_pass(self) -> Option<(GateId, Gate)> {
        self.first_non_pass_in(&EXECUTION_GATE_ORDER)
    }

    fn all_passed_in(self, order: &[GateId]) -> bool {
        order.iter().copied().all(|id| self.is_passed(id))
    }

    fn first_non_pass_in(self, order: &[GateId]) -> Option<(GateId, Gate)> {
        order.iter().copied().find_map(|id| {
            let gate = self.get(id);
            (!self.is_passed(id)).then_some((id, gate))
        })
    }

    fn is_passed(self, id: GateId) -> bool {
        self.get(id).status == GateStatus::Pass
    }

    pub fn is_structurally_valid(self) -> bool {
        GATE_ORDER
            .iter()
            .copied()
            .all(|id| self.get(id).is_structurally_valid())
    }
}
