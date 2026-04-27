//! Pure canonical kernel types and packet/state invariants.
//!
//! This module is intentionally free of filesystem I/O and runner policy.
//! It owns the stable state model that reducer, recovery, codec, writer, and
//! verifier code operate on.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Phase {
    Delta = 1,
    Invariant = 2,
    Analysis = 3,
    Judgment = 4,
    Plan = 5,
    Execute = 6,
    Verify = 7,
    Eval = 8,
    Recovery = 9,
    Learn = 10,
    Persist = 11,
    Done = 12,
}

pub const PHASES: [Phase; 12] = [
    Phase::Delta,
    Phase::Invariant,
    Phase::Analysis,
    Phase::Judgment,
    Phase::Plan,
    Phase::Execute,
    Phase::Verify,
    Phase::Eval,
    Phase::Recovery,
    Phase::Learn,
    Phase::Persist,
    Phase::Done,
];

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Gate {
    pub status: GateStatus,
    pub evidence: Evidence,
    pub version: u64,
}

impl Gate {
    pub const fn unknown() -> Self {
        Self {
            status: GateStatus::Unknown,
            evidence: Evidence::Missing,
            version: 0,
        }
    }

    pub const fn pass(evidence: Evidence) -> Self {
        Self {
            status: GateStatus::Pass,
            evidence,
            version: 1,
        }
    }

    pub const fn fail(evidence: Evidence) -> Self {
        Self {
            status: GateStatus::Fail,
            evidence,
            version: 1,
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
        let gate = self.get_mut(id);
        *gate = Gate {
            status: GateStatus::Pass,
            evidence,
            version: gate.version.saturating_add(1),
        };
    }

    pub fn set_fail(&mut self, id: GateId, evidence: Evidence) {
        let gate = self.get_mut(id);
        *gate = Gate {
            status: GateStatus::Fail,
            evidence,
            version: gate.version.saturating_add(1),
        };
    }

    pub fn all_passed(self) -> bool {
        GATE_ORDER
            .iter()
            .all(|id| self.get(*id).status == GateStatus::Pass)
    }

    pub fn all_execution_passed(self) -> bool {
        EXECUTION_GATE_ORDER
            .iter()
            .all(|id| self.get(*id).status == GateStatus::Pass)
    }

    pub fn first_non_pass(self) -> Option<(GateId, Gate)> {
        GATE_ORDER.iter().copied().find_map(|id| {
            let gate = self.get(id);
            (gate.status != GateStatus::Pass).then_some((id, gate))
        })
    }

    pub fn first_execution_non_pass(self) -> Option<(GateId, Gate)> {
        EXECUTION_GATE_ORDER.iter().copied().find_map(|id| {
            let gate = self.get(id);
            (gate.status != GateStatus::Pass).then_some((id, gate))
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Packet {
    pub objective_id: u64,
    pub objective_required_tasks: u8,
    pub objective_done_tasks: u8,
    pub ready_tasks: u8,
    pub active_task_id: u64,
    pub artifact_id: u64,
    pub parent_artifact_id: u64,
    pub artifact_bytes: u64,
    pub artifact_receipt_hash: u64,
    pub artifact_lineage_hash: u64,
    pub revision: u64,
}

impl Packet {
    pub const fn empty() -> Self {
        Self {
            objective_id: 1,
            objective_required_tasks: 1,
            objective_done_tasks: 0,
            ready_tasks: 0,
            active_task_id: 0,
            artifact_id: 0,
            parent_artifact_id: 0,
            artifact_bytes: 0,
            artifact_receipt_hash: 0,
            artifact_lineage_hash: 0,
            revision: 0,
        }
    }

    pub fn ready() -> Self {
        let mut packet = Self::empty();
        packet.bind_ready_task();
        packet.materialize_artifact();
        packet.repair_lineage();
        packet.complete_objective();
        packet
    }

    pub fn has_ready_task(self) -> bool {
        self.ready_tasks > 0 && self.active_task_id != 0
    }

    pub fn objective_complete(self) -> bool {
        self.objective_required_tasks > 0
            && self.objective_done_tasks >= self.objective_required_tasks
    }

    pub fn artifact_present(self) -> bool {
        self.artifact_id != 0 && self.artifact_bytes != 0 && self.artifact_receipt_hash != 0
    }

    pub fn artifact_receipt_valid(self) -> bool {
        self.artifact_id != 0
            && self.artifact_bytes != 0
            && self.artifact_receipt_hash == self.expected_receipt_hash()
    }

    pub fn lineage_valid(self) -> bool {
        self.artifact_receipt_valid() && self.artifact_lineage_hash == self.expected_lineage_hash()
    }

    pub fn bind_ready_task(&mut self) {
        self.revision = self.revision.saturating_add(1);
        self.ready_tasks = self.ready_tasks.max(1);
        if self.active_task_id == 0 {
            self.active_task_id = self.objective_id.saturating_mul(100).saturating_add(1);
        }
    }

    pub fn materialize_artifact(&mut self) {
        self.revision = self.revision.saturating_add(1);
        if self.active_task_id == 0 {
            self.bind_ready_task();
        }
        self.parent_artifact_id = self.artifact_id;
        self.artifact_id = self
            .objective_id
            .saturating_mul(10_000)
            .saturating_add(self.active_task_id)
            .saturating_add(self.revision);
        self.artifact_bytes = self.artifact_id.saturating_mul(3).saturating_add(17);
        self.artifact_receipt_hash = self.expected_receipt_hash();
        self.ready_tasks = self.ready_tasks.saturating_sub(1);
        self.repair_lineage();
    }

    pub fn repair_lineage(&mut self) {
        if self.artifact_receipt_valid() {
            self.artifact_lineage_hash = self.expected_lineage_hash();
        }
    }

    pub fn complete_objective(&mut self) {
        if self.lineage_valid() {
            self.objective_done_tasks = self.objective_required_tasks;
        }
    }

    pub fn expected_receipt_hash(self) -> u64 {
        let mut h = 0x243f6a8885a308d3u64;
        h = mix(h, self.objective_id);
        h = mix(h, self.active_task_id);
        h = mix(h, self.parent_artifact_id);
        h = mix(h, self.artifact_id);
        h = mix(h, self.artifact_bytes);
        h = mix(h, self.revision);
        h
    }

    pub fn expected_lineage_hash(self) -> u64 {
        let mut h = 0x9e3779b97f4a7c15u64;
        h = mix(h, self.objective_id);
        h = mix(h, self.active_task_id);
        h = mix(h, self.parent_artifact_id);
        h = mix(h, self.artifact_id);
        h = mix(h, self.artifact_bytes);
        h = mix(h, self.artifact_receipt_hash);
        h = mix(h, self.revision);
        h
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct State {
    pub phase: Phase,
    pub gates: GateSet,
    pub packet: Packet,
    pub failure: Option<FailureClass>,
    pub recovery_action: Option<RecoveryAction>,
    pub recovery_attempts: u8,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Delta,
            gates: GateSet::default(),
            packet: Packet::empty(),
            failure: None,
            recovery_action: None,
            recovery_attempts: 0,
        }
    }
}

impl State {
    pub fn ready() -> Self {
        Self {
            phase: Phase::Delta,
            gates: GateSet::ready(),
            packet: Packet::ready(),
            failure: None,
            recovery_action: None,
            recovery_attempts: 0,
        }
    }

    pub fn is_success(self) -> bool {
        self.failure.is_none()
            && self.phase == Phase::Done
            && self.gates.all_passed()
            && self.packet.objective_complete()
            && self.packet.lineage_valid()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum FailureClass {
    InvariantUnknown = 1,
    InvariantBlocked = 2,
    AnalysisMissing = 3,
    AnalysisFailed = 4,
    JudgmentMissing = 5,
    JudgmentFailed = 6,
    PlanMissing = 7,
    PlanFailed = 8,
    PlanReadyQueueEmpty = 9,
    ExecutionMissing = 10,
    ExecutionFailed = 11,
    TaskReceiptMissing = 12,
    VerificationUnknown = 13,
    VerificationFailed = 14,
    ArtifactLineageBroken = 15,
    EvalMissing = 16,
    EvalFailed = 17,
    RecoveryExhausted = 18,
    ConvergenceFailed = 19,
    LearningMissing = 20,
    LearningFailed = 21,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum RecoveryAction {
    RecheckInvariant = 1,
    RunAnalysis = 2,
    Rejudge = 3,
    Replan = 4,
    BindReadyTask = 5,
    Reexecute = 6,
    Reverify = 7,
    RepairArtifactLineage = 8,
    RecomputeEval = 9,
    Escalate = 10,
}

impl RecoveryAction {
    pub fn target(self) -> Phase {
        match self {
            RecoveryAction::RecheckInvariant => Phase::Invariant,
            RecoveryAction::RunAnalysis => Phase::Analysis,
            RecoveryAction::Rejudge => Phase::Judgment,
            RecoveryAction::Replan | RecoveryAction::BindReadyTask => Phase::Plan,
            RecoveryAction::Reexecute => Phase::Execute,
            RecoveryAction::Reverify | RecoveryAction::RepairArtifactLineage => Phase::Verify,
            RecoveryAction::RecomputeEval => Phase::Eval,
            RecoveryAction::Escalate => Phase::Done,
        }
    }

    pub fn repaired_gate(self) -> Option<GateId> {
        match self {
            RecoveryAction::RecheckInvariant => Some(GateId::Invariant),
            RecoveryAction::RunAnalysis => Some(GateId::Analysis),
            RecoveryAction::Rejudge => Some(GateId::Judgment),
            RecoveryAction::Replan | RecoveryAction::BindReadyTask => Some(GateId::Plan),
            RecoveryAction::Reexecute => Some(GateId::Execution),
            RecoveryAction::Reverify | RecoveryAction::RepairArtifactLineage => {
                Some(GateId::Verification)
            }
            RecoveryAction::RecomputeEval => Some(GateId::Eval),
            RecoveryAction::Escalate => None,
        }
    }

    pub fn produced_evidence(self) -> Option<Evidence> {
        match self {
            RecoveryAction::RecheckInvariant => Some(Evidence::InvariantProof),
            RecoveryAction::RunAnalysis => Some(Evidence::AnalysisReport),
            RecoveryAction::Rejudge => Some(Evidence::JudgmentRecord),
            RecoveryAction::Replan => Some(Evidence::PlanRecord),
            RecoveryAction::BindReadyTask => Some(Evidence::TaskReady),
            RecoveryAction::Reexecute => Some(Evidence::ArtifactReceipt),
            RecoveryAction::Reverify => Some(Evidence::VerificationReport),
            RecoveryAction::RepairArtifactLineage => Some(Evidence::LineageProof),
            RecoveryAction::RecomputeEval => Some(Evidence::EvalScore),
            RecoveryAction::Escalate => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EventKind {
    Advanced = 1,
    Blocked = 2,
    Failed = 3,
    Recovered = 4,
    Learned = 5,
    Completed = 6,
    Persisted = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Cause {
    Start = 1,
    GatePassed = 2,
    GateFailed = 3,
    EvidenceMissing = 4,
    JudgmentMade = 5,
    PlanReady = 6,
    ReadyQueueEmpty = 7,
    ExecutionFinished = 8,
    TaskReceiptMissing = 9,
    VerificationPassed = 10,
    ArtifactLineageBroken = 11,
    EvalPassed = 12,
    EvalFailed = 13,
    RepairSelected = 14,
    RepairApplied = 15,
    RecoveryLimit = 16,
    MaxSteps = 17,
    Persisted = 18,
    PolicyPromoted = 19,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Decision {
    Continue = 1,
    Complete = 2,
    Block = 3,
    Fail = 4,
    Repair = 5,
    Halt = 6,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SemanticDelta {
    NoChange = 1,
    PhaseAdvanced = 2,
    FailureRaised = 3,
    RepairSelected = 4,
    RepairApplied = 5,
    PayloadChanged = 6,
    Completed = 7,
    Halted = 8,
    Persisted = 9,
    LearningPromoted = 10,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RuntimeConfig {
    pub max_steps: u64,
    pub max_recovery_attempts: u8,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_steps: 96,
            max_recovery_attempts: 8,
        }
    }
}

pub type TLog = Vec<ControlEvent>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ControlEvent {
    pub seq: u64,
    pub from: Phase,
    pub to: Phase,
    pub kind: EventKind,
    pub cause: Cause,
    pub delta: SemanticDelta,
    pub evidence: Evidence,
    pub decision: Decision,
    pub failure: Option<FailureClass>,
    pub recovery_action: Option<RecoveryAction>,
    pub affected_gate: Option<GateId>,
    pub runtime_config: RuntimeConfig,
    pub state_before: State,
    pub state_after: State,
    pub prev_hash: u64,
    pub self_hash: u64,
}

pub(crate) fn mix(mut h: u64, x: u64) -> u64 {
    h ^= x;
    h = h.wrapping_mul(0x100000001b3);
    h
}
