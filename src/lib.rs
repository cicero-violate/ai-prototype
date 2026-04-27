#![forbid(unsafe_code)]
//! Canonical atomic state-machine runtime.
//!
//! The crate exposes a deterministic transition system, hash-chained tlog
//! verification, recovery selection, and a small demo runner. `src/main.rs`
//! is intentionally thin; reusable behavior lives here.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

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
    Done = 11,
}

pub const PHASES: [Phase; 11] = [
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
}

pub const GATE_ORDER: [GateId; 7] = [
    GateId::Invariant,
    GateId::Analysis,
    GateId::Judgment,
    GateId::Plan,
    GateId::Execution,
    GateId::Verification,
    GateId::Eval,
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

    pub fn first_non_pass(self) -> Option<(GateId, Gate)> {
        GATE_ORDER
            .iter()
            .copied()
            .find_map(|id| {
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
        self.artifact_id != 0 && self.artifact_bytes != 0
    }

    pub fn lineage_valid(self) -> bool {
        self.artifact_present() && self.artifact_lineage_hash == self.expected_lineage_hash()
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
        self.ready_tasks = self.ready_tasks.saturating_sub(1);
        self.repair_lineage();
    }

    pub fn repair_lineage(&mut self) {
        self.artifact_lineage_hash = self.expected_lineage_hash();
    }

    pub fn complete_objective(&mut self) {
        if self.lineage_valid() {
            self.objective_done_tasks = self.objective_required_tasks;
        }
    }

    pub fn expected_lineage_hash(self) -> u64 {
        let mut h = 0x9e3779b97f4a7c15u64;
        h = mix(h, self.objective_id);
        h = mix(h, self.active_task_id);
        h = mix(h, self.parent_artifact_id);
        h = mix(h, self.artifact_id);
        h = mix(h, self.artifact_bytes);
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    pub state_before: State,
    pub state_after: State,
    pub prev_hash: u64,
    pub self_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Outcome {
    state: State,
    kind: EventKind,
    cause: Cause,
    evidence: Evidence,
    decision: Decision,
    failure: Option<FailureClass>,
    recovery_action: Option<RecoveryAction>,
    affected_gate: Option<GateId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonError {
    IllegalEvent {
        from: Phase,
        to: Phase,
        kind: EventKind,
    },
    MissingFailureClass,
    UnexpectedFailureClass,
    MissingRecoveryAction,
    UnexpectedRecoveryAction,
    InvalidLearnTarget,
    InvalidRepairTarget,
    InvalidCompletion,
    InvalidStateContinuity,
    InvalidPacketContinuity,
    InvalidSemanticDelta,
    InvalidHashChain,
    InvalidReplay,
    TlogIo,
    InvalidTlogRecord,
    MissingAffectedGate,
    UnexpectedAffectedGate,
}

impl core::fmt::Display for CanonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for CanonError {}

#[derive(Clone, Copy)]
struct Transition {
    from: Phase,
    to: Phase,
    kind: EventKind,
    cause: Cause,
}

const TRANSITIONS: [Transition; 36] = [
    Transition { from: Phase::Delta, to: Phase::Invariant, kind: EventKind::Advanced, cause: Cause::Start },
    Transition { from: Phase::Invariant, to: Phase::Analysis, kind: EventKind::Advanced, cause: Cause::GatePassed },
    Transition { from: Phase::Invariant, to: Phase::Recovery, kind: EventKind::Blocked, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Invariant, to: Phase::Recovery, kind: EventKind::Blocked, cause: Cause::GateFailed },
    Transition { from: Phase::Analysis, to: Phase::Judgment, kind: EventKind::Advanced, cause: Cause::GatePassed },
    Transition { from: Phase::Analysis, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Analysis, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Judgment, to: Phase::Plan, kind: EventKind::Advanced, cause: Cause::JudgmentMade },
    Transition { from: Phase::Judgment, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Judgment, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Plan, to: Phase::Execute, kind: EventKind::Advanced, cause: Cause::PlanReady },
    Transition { from: Phase::Plan, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Plan, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Plan, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::ReadyQueueEmpty },
    Transition { from: Phase::Execute, to: Phase::Verify, kind: EventKind::Advanced, cause: Cause::ExecutionFinished },
    Transition { from: Phase::Execute, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Execute, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Execute, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::TaskReceiptMissing },
    Transition { from: Phase::Verify, to: Phase::Eval, kind: EventKind::Advanced, cause: Cause::VerificationPassed },
    Transition { from: Phase::Verify, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Verify, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Verify, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::ArtifactLineageBroken },
    Transition { from: Phase::Eval, to: Phase::Done, kind: EventKind::Completed, cause: Cause::EvalPassed },
    Transition { from: Phase::Eval, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvidenceMissing },
    Transition { from: Phase::Eval, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::GateFailed },
    Transition { from: Phase::Eval, to: Phase::Recovery, kind: EventKind::Failed, cause: Cause::EvalFailed },
    Transition { from: Phase::Recovery, to: Phase::Learn, kind: EventKind::Recovered, cause: Cause::RepairSelected },
    Transition { from: Phase::Recovery, to: Phase::Done, kind: EventKind::Failed, cause: Cause::RecoveryLimit },
    Transition { from: Phase::Learn, to: Phase::Invariant, kind: EventKind::Learned, cause: Cause::RepairApplied },
    Transition { from: Phase::Learn, to: Phase::Analysis, kind: EventKind::Learned, cause: Cause::RepairApplied },
    Transition { from: Phase::Learn, to: Phase::Judgment, kind: EventKind::Learned, cause: Cause::RepairApplied },
    Transition { from: Phase::Learn, to: Phase::Plan, kind: EventKind::Learned, cause: Cause::RepairApplied },
    Transition { from: Phase::Learn, to: Phase::Execute, kind: EventKind::Learned, cause: Cause::RepairApplied },
    Transition { from: Phase::Learn, to: Phase::Verify, kind: EventKind::Learned, cause: Cause::RepairApplied },
    Transition { from: Phase::Learn, to: Phase::Eval, kind: EventKind::Learned, cause: Cause::RepairApplied },
    Transition { from: Phase::Done, to: Phase::Done, kind: EventKind::Completed, cause: Cause::EvalPassed },
];

struct CanonicalWriter;

impl CanonicalWriter {
    fn append(tlog: &mut TLog, before: State, outcome: Outcome) -> Result<(), CanonError> {
        let after = outcome.state;
        let delta = semantic_diff(before, after);

        validate_event(EventView {
            from: before.phase,
            to: after.phase,
            kind: outcome.kind,
            cause: outcome.cause,
            decision: outcome.decision,
            failure: outcome.failure,
            recovery_action: outcome.recovery_action,
            affected_gate: outcome.affected_gate,
        })?;

        if outcome.kind == EventKind::Completed && !after.is_success() {
            return Err(CanonError::InvalidCompletion);
        }

        let seq = tlog.len() as u64 + 1;
        let prev_hash = tlog.last().map(|e| e.self_hash).unwrap_or(0);
        let self_hash = hash_event(EventHashInput {
            seq,
            prev_hash,
            from: before.phase,
            to: after.phase,
            kind: outcome.kind,
            cause: outcome.cause,
            delta,
            evidence: outcome.evidence,
            decision: outcome.decision,
            failure: outcome.failure,
            recovery_action: outcome.recovery_action,
            affected_gate: outcome.affected_gate,
            state_before: before,
            state_after: after,
        });

        tlog.push(ControlEvent {
            seq,
            from: before.phase,
            to: after.phase,
            kind: outcome.kind,
            cause: outcome.cause,
            delta,
            evidence: outcome.evidence,
            decision: outcome.decision,
            failure: outcome.failure,
            recovery_action: outcome.recovery_action,
            affected_gate: outcome.affected_gate,
            state_before: before,
            state_after: after,
            prev_hash,
            self_hash,
        });

        Ok(())
    }
}

fn reduce(input: State, cfg: RuntimeConfig) -> Outcome {
    match input.phase {
        Phase::Delta => {
            let mut s = input;
            advance(
                &mut s,
                Phase::Invariant,
                Cause::Start,
                Evidence::DeltaComputed,
            )
        }
        Phase::Invariant => gate_step(input, GateId::Invariant, Phase::Analysis, Cause::GatePassed),
        Phase::Analysis => gate_step(input, GateId::Analysis, Phase::Judgment, Cause::GatePassed),
        Phase::Judgment => gate_step(input, GateId::Judgment, Phase::Plan, Cause::JudgmentMade),
        Phase::Plan => plan_step(input),
        Phase::Execute => execute_step(input),
        Phase::Verify => verify_step(input),
        Phase::Eval => eval_step(input),
        Phase::Recovery => recover(input, cfg),
        Phase::Learn => learn(input),
        Phase::Done => {
            let mut s = input;
            complete(&mut s)
        }
    }
}

fn gate_step(input: State, gate_id: GateId, next: Phase, pass_cause: Cause) -> Outcome {
    let mut s = input;
    let gate = s.gates.get(gate_id);

    match gate.status {
        GateStatus::Pass => advance(&mut s, next, pass_cause, gate.evidence),
        GateStatus::Unknown | GateStatus::Fail => raise_gate_failure(&mut s, gate_id, gate),
    }
}

fn plan_step(input: State) -> Outcome {
    let mut s = input;
    let gate = s.gates.plan;

    match gate.status {
        GateStatus::Pass if s.packet.has_ready_task() || s.packet.objective_complete() => {
            advance(&mut s, Phase::Execute, Cause::PlanReady, gate.evidence)
        }
        GateStatus::Pass => raise_domain_failure(
            &mut s,
            FailureClass::PlanReadyQueueEmpty,
            Cause::ReadyQueueEmpty,
            Evidence::Missing,
            GateId::Plan,
        ),
        GateStatus::Unknown | GateStatus::Fail => raise_gate_failure(&mut s, GateId::Plan, gate),
    }
}

fn execute_step(input: State) -> Outcome {
    let mut s = input;
    let gate = s.gates.execution;

    match gate.status {
        GateStatus::Pass if s.packet.artifact_present() => {
            advance(&mut s, Phase::Verify, Cause::ExecutionFinished, gate.evidence)
        }
        GateStatus::Pass => raise_domain_failure(
            &mut s,
            FailureClass::TaskReceiptMissing,
            Cause::TaskReceiptMissing,
            Evidence::Missing,
            GateId::Execution,
        ),
        GateStatus::Unknown | GateStatus::Fail => {
            raise_gate_failure(&mut s, GateId::Execution, gate)
        }
    }
}

fn verify_step(input: State) -> Outcome {
    let mut s = input;
    let gate = s.gates.verification;

    match gate.status {
        GateStatus::Pass if s.packet.lineage_valid() => {
            advance(&mut s, Phase::Eval, Cause::VerificationPassed, gate.evidence)
        }
        GateStatus::Pass => raise_domain_failure(
            &mut s,
            FailureClass::ArtifactLineageBroken,
            Cause::ArtifactLineageBroken,
            Evidence::Missing,
            GateId::Verification,
        ),
        GateStatus::Unknown | GateStatus::Fail => {
            raise_gate_failure(&mut s, GateId::Verification, gate)
        }
    }
}

fn eval_step(input: State) -> Outcome {
    let mut s = input;

    if let Some((gate_id, gate)) = s.gates.first_non_pass() {
        return raise_gate_failure(&mut s, gate_id, gate);
    }

    if !s.packet.objective_complete() {
        return raise_domain_failure(
            &mut s,
            FailureClass::EvalFailed,
            Cause::EvalFailed,
            Evidence::Missing,
            GateId::Eval,
        );
    }

    complete(&mut s)
}

fn advance(s: &mut State, to: Phase, cause: Cause, evidence: Evidence) -> Outcome {
    s.phase = to;
    s.failure = None;
    s.recovery_action = None;

    Outcome {
        state: *s,
        kind: EventKind::Advanced,
        cause,
        evidence,
        decision: Decision::Continue,
        failure: None,
        recovery_action: None,
        affected_gate: None,
    }
}

fn complete(s: &mut State) -> Outcome {
    s.phase = Phase::Done;
    s.failure = None;
    s.recovery_action = None;

    Outcome {
        state: *s,
        kind: EventKind::Completed,
        cause: Cause::EvalPassed,
        evidence: Evidence::CompletionProof,
        decision: Decision::Complete,
        failure: None,
        recovery_action: None,
        affected_gate: None,
    }
}

fn raise_gate_failure(s: &mut State, gate_id: GateId, gate: Gate) -> Outcome {
    let class = failure_for_gate(gate_id, gate.status);
    let kind = event_kind_for_failure(class);
    let decision = decision_for_failure(class);
    let cause = match gate.status {
        GateStatus::Unknown => Cause::EvidenceMissing,
        GateStatus::Fail => Cause::GateFailed,
        GateStatus::Pass => unreachable!("passing gate cannot raise failure"),
    };
    let evidence = match gate.status {
        GateStatus::Unknown => Evidence::Missing,
        GateStatus::Fail => gate.evidence,
        GateStatus::Pass => unreachable!("passing gate cannot raise failure"),
    };

    s.phase = Phase::Recovery;
    s.failure = Some(class);
    s.recovery_action = None;

    Outcome {
        state: *s,
        kind,
        cause,
        evidence,
        decision,
        failure: Some(class),
        recovery_action: None,
        affected_gate: Some(gate_id),
    }
}

fn raise_domain_failure(
    s: &mut State,
    class: FailureClass,
    cause: Cause,
    evidence: Evidence,
    gate_id: GateId,
) -> Outcome {
    s.phase = Phase::Recovery;
    s.failure = Some(class);
    s.recovery_action = None;

    Outcome {
        state: *s,
        kind: EventKind::Failed,
        cause,
        evidence,
        decision: Decision::Fail,
        failure: Some(class),
        recovery_action: None,
        affected_gate: Some(gate_id),
    }
}

fn recover(input: State, cfg: RuntimeConfig) -> Outcome {
    let mut s = input;

    if input.recovery_attempts >= cfg.max_recovery_attempts {
        return halt_recovery(
            &mut s,
            FailureClass::RecoveryExhausted,
            Cause::RecoveryLimit,
        );
    }

    let failure = input.failure.unwrap_or(FailureClass::RecoveryExhausted);
    let action = recovery_action_for(failure);

    if action == RecoveryAction::Escalate {
        return halt_recovery(&mut s, failure, Cause::RecoveryLimit);
    }

    s.phase = Phase::Learn;
    s.recovery_attempts = s.recovery_attempts.saturating_add(1);
    s.recovery_action = Some(action);

    Outcome {
        state: s,
        kind: EventKind::Recovered,
        cause: Cause::RepairSelected,
        evidence: Evidence::RecoveryPolicy,
        decision: Decision::Repair,
        failure: Some(failure),
        recovery_action: Some(action),
        affected_gate: None,
    }
}

fn halt_recovery(s: &mut State, class: FailureClass, cause: Cause) -> Outcome {
    s.phase = Phase::Done;
    s.failure = Some(class);
    s.recovery_action = Some(RecoveryAction::Escalate);

    Outcome {
        state: *s,
        kind: EventKind::Failed,
        cause,
        evidence: Evidence::ConvergenceLimit,
        decision: Decision::Halt,
        failure: Some(class),
        recovery_action: Some(RecoveryAction::Escalate),
        affected_gate: None,
    }
}

fn learn(input: State) -> Outcome {
    let mut s = input;

    let Some(action) = input.recovery_action else {
        s.phase = Phase::Recovery;
        s.failure = Some(FailureClass::RecoveryExhausted);

        return Outcome {
            state: s,
            kind: EventKind::Failed,
            cause: Cause::EvidenceMissing,
            evidence: Evidence::Missing,
            decision: Decision::Fail,
            failure: Some(FailureClass::RecoveryExhausted),
            recovery_action: None,
            affected_gate: None,
        };
    };

    if action == RecoveryAction::Escalate {
        s.phase = Phase::Done;
        s.failure = Some(input.failure.unwrap_or(FailureClass::RecoveryExhausted));

        return Outcome {
            state: s,
            kind: EventKind::Learned,
            cause: Cause::RepairApplied,
            evidence: Evidence::ConvergenceLimit,
            decision: Decision::Halt,
            failure: s.failure,
            recovery_action: Some(action),
            affected_gate: None,
        };
    }

    apply_repair(&mut s, action);

    let gate = action
        .repaired_gate()
        .expect("non-escalation repair action must target a gate");
    let evidence = action
        .produced_evidence()
        .expect("non-escalation repair action must produce evidence");

    s.gates.set_pass(gate, evidence);
    s.phase = action.target();
    s.failure = None;
    s.recovery_action = None;

    Outcome {
        state: s,
        kind: EventKind::Learned,
        cause: Cause::RepairApplied,
        evidence,
        decision: Decision::Continue,
        failure: input.failure,
        recovery_action: Some(action),
        affected_gate: Some(gate),
    }
}

fn apply_repair(s: &mut State, action: RecoveryAction) {
    match action {
        RecoveryAction::RecheckInvariant => {
            s.packet.objective_id = s.packet.objective_id.max(1);
            s.packet.objective_required_tasks = s.packet.objective_required_tasks.max(1);
        }
        RecoveryAction::RunAnalysis | RecoveryAction::Rejudge => {
            s.packet.revision = s.packet.revision.saturating_add(1);
        }
        RecoveryAction::Replan | RecoveryAction::BindReadyTask => {
            s.packet.bind_ready_task();
        }
        RecoveryAction::Reexecute => {
            s.packet.materialize_artifact();
        }
        RecoveryAction::Reverify | RecoveryAction::RepairArtifactLineage => {
            s.packet.repair_lineage();
        }
        RecoveryAction::RecomputeEval => {
            s.packet.complete_objective();
        }
        RecoveryAction::Escalate => {}
    }
}

fn recovery_action_for(class: FailureClass) -> RecoveryAction {
    match class {
        FailureClass::InvariantUnknown | FailureClass::InvariantBlocked => {
            RecoveryAction::RecheckInvariant
        }
        FailureClass::AnalysisMissing | FailureClass::AnalysisFailed => RecoveryAction::RunAnalysis,
        FailureClass::JudgmentMissing | FailureClass::JudgmentFailed => RecoveryAction::Rejudge,
        FailureClass::PlanMissing => RecoveryAction::BindReadyTask,
        FailureClass::PlanFailed => RecoveryAction::Replan,
        FailureClass::PlanReadyQueueEmpty => RecoveryAction::BindReadyTask,
        FailureClass::ExecutionMissing
        | FailureClass::ExecutionFailed
        | FailureClass::TaskReceiptMissing => RecoveryAction::Reexecute,
        FailureClass::VerificationUnknown | FailureClass::VerificationFailed => {
            RecoveryAction::Reverify
        }
        FailureClass::ArtifactLineageBroken => RecoveryAction::RepairArtifactLineage,
        FailureClass::EvalMissing | FailureClass::EvalFailed => RecoveryAction::RecomputeEval,
        FailureClass::RecoveryExhausted | FailureClass::ConvergenceFailed => {
            RecoveryAction::Escalate
        }
    }
}

fn failure_for_gate(id: GateId, status: GateStatus) -> FailureClass {
    match (id, status) {
        (GateId::Invariant, GateStatus::Unknown) => FailureClass::InvariantUnknown,
        (GateId::Invariant, GateStatus::Fail) => FailureClass::InvariantBlocked,

        (GateId::Analysis, GateStatus::Unknown) => FailureClass::AnalysisMissing,
        (GateId::Analysis, GateStatus::Fail) => FailureClass::AnalysisFailed,

        (GateId::Judgment, GateStatus::Unknown) => FailureClass::JudgmentMissing,
        (GateId::Judgment, GateStatus::Fail) => FailureClass::JudgmentFailed,

        (GateId::Plan, GateStatus::Unknown) => FailureClass::PlanMissing,
        (GateId::Plan, GateStatus::Fail) => FailureClass::PlanFailed,

        (GateId::Execution, GateStatus::Unknown) => FailureClass::ExecutionMissing,
        (GateId::Execution, GateStatus::Fail) => FailureClass::ExecutionFailed,

        (GateId::Verification, GateStatus::Unknown) => FailureClass::VerificationUnknown,
        (GateId::Verification, GateStatus::Fail) => FailureClass::VerificationFailed,

        (GateId::Eval, GateStatus::Unknown) => FailureClass::EvalMissing,
        (GateId::Eval, GateStatus::Fail) => FailureClass::EvalFailed,

        (_, GateStatus::Pass) => unreachable!("passing gate cannot produce failure"),
    }
}

fn event_kind_for_failure(class: FailureClass) -> EventKind {
    match class {
        FailureClass::InvariantUnknown | FailureClass::InvariantBlocked => EventKind::Blocked,
        _ => EventKind::Failed,
    }
}

fn decision_for_failure(class: FailureClass) -> Decision {
    match class {
        FailureClass::InvariantUnknown | FailureClass::InvariantBlocked => Decision::Block,
        _ => Decision::Fail,
    }
}

fn evidence_for_gate(id: GateId) -> Evidence {
    match id {
        GateId::Invariant => Evidence::InvariantProof,
        GateId::Analysis => Evidence::AnalysisReport,
        GateId::Judgment => Evidence::JudgmentRecord,
        GateId::Plan => Evidence::TaskReady,
        GateId::Execution => Evidence::ArtifactReceipt,
        GateId::Verification => Evidence::LineageProof,
        GateId::Eval => Evidence::EvalScore,
    }
}

pub fn tick(state: &mut State, tlog: &mut TLog, cfg: RuntimeConfig) -> Result<(), CanonError> {
    let before = *state;
    let outcome = reduce(before, cfg);
    CanonicalWriter::append(tlog, before, outcome)?;
    *state = outcome.state;
    Ok(())
}

pub fn tick_durable(
    state: &mut State,
    tlog: &mut TLog,
    tlog_path: impl AsRef<Path>,
    cfg: RuntimeConfig,
) -> Result<(), CanonError> {
    let before = *state;
    let outcome = reduce(before, cfg);
    CanonicalWriter::append(tlog, before, outcome)?;
    let event = tlog.last().copied().ok_or(CanonError::InvalidTlogRecord)?;
    append_tlog_ndjson(tlog_path, &event)?;
    *state = outcome.state;
    Ok(())
}

pub fn run_until_done(mut state: State, cfg: RuntimeConfig) -> Result<(State, TLog), CanonError> {
    let mut tlog = Vec::new();

    for _ in 0..cfg.max_steps {
        if state.phase == Phase::Done {
            return Ok((state, tlog));
        }

        tick(&mut state, &mut tlog, cfg)?;
    }

    append_convergence_failure(&mut state, &mut tlog)?;
    Ok((state, tlog))
}

pub fn run_until_done_durable(
    initial: State,
    cfg: RuntimeConfig,
    tlog_path: impl AsRef<Path>,
) -> Result<(State, TLog), CanonError> {
    let path = tlog_path.as_ref();
    let mut tlog = load_tlog_ndjson(path)?;
    let mut state = if tlog.is_empty() {
        initial
    } else {
        verify_tlog_from(initial, &tlog)?
    };

    for _ in 0..cfg.max_steps {
        if state.phase == Phase::Done {
            return Ok((state, tlog));
        }

        tick_durable(&mut state, &mut tlog, path, cfg)?;
    }

    append_convergence_failure(&mut state, &mut tlog)?;
    let event = tlog.last().copied().ok_or(CanonError::InvalidTlogRecord)?;
    append_tlog_ndjson(path, &event)?;
    Ok((state, tlog))
}

fn append_convergence_failure(state: &mut State, tlog: &mut TLog) -> Result<(), CanonError> {
    let before = *state;
    state.phase = Phase::Done;
    state.failure = Some(FailureClass::ConvergenceFailed);
    state.recovery_action = Some(RecoveryAction::Escalate);

    let outcome = Outcome {
        state: *state,
        kind: EventKind::Failed,
        cause: Cause::MaxSteps,
        evidence: Evidence::ConvergenceLimit,
        decision: Decision::Halt,
        failure: Some(FailureClass::ConvergenceFailed),
        recovery_action: Some(RecoveryAction::Escalate),
        affected_gate: None,
    };

    CanonicalWriter::append(tlog, before, outcome)
}

pub fn semantic_diff(a: State, b: State) -> SemanticDelta {
    if a == b {
        return SemanticDelta::NoChange;
    }
    if b.phase == Phase::Done && b.failure.is_none() {
        return SemanticDelta::Completed;
    }
    if b.phase == Phase::Done && b.failure.is_some() {
        return SemanticDelta::Halted;
    }
    if a.phase == Phase::Recovery && b.phase == Phase::Learn {
        return SemanticDelta::RepairSelected;
    }
    if a.phase == Phase::Learn {
        return SemanticDelta::RepairApplied;
    }
    if b.failure.is_some() && b.phase == Phase::Recovery {
        return SemanticDelta::FailureRaised;
    }
    if a.packet != b.packet {
        return SemanticDelta::PayloadChanged;
    }
    if a.phase != b.phase {
        return SemanticDelta::PhaseAdvanced;
    }
    SemanticDelta::NoChange
}

#[derive(Clone, Copy)]
struct EventView {
    from: Phase,
    to: Phase,
    kind: EventKind,
    cause: Cause,
    decision: Decision,
    failure: Option<FailureClass>,
    recovery_action: Option<RecoveryAction>,
    affected_gate: Option<GateId>,
}

fn validate_event(event: EventView) -> Result<(), CanonError> {
    if !legal_transition(event.from, event.to, event.kind, event.cause) {
        return Err(CanonError::IllegalEvent {
            from: event.from,
            to: event.to,
            kind: event.kind,
        });
    }

    if matches!(
        event.kind,
        EventKind::Blocked | EventKind::Failed | EventKind::Recovered | EventKind::Learned
    ) && event.failure.is_none()
    {
        return Err(CanonError::MissingFailureClass);
    }

    if matches!(event.kind, EventKind::Advanced | EventKind::Completed) && event.failure.is_some() {
        return Err(CanonError::UnexpectedFailureClass);
    }

    if matches!(event.kind, EventKind::Recovered | EventKind::Learned)
        && event.recovery_action.is_none()
    {
        return Err(CanonError::MissingRecoveryAction);
    }

    if matches!(event.kind, EventKind::Advanced | EventKind::Completed)
        && event.recovery_action.is_some()
    {
        return Err(CanonError::UnexpectedRecoveryAction);
    }

    if event.kind == EventKind::Recovered && event.affected_gate.is_some() {
        return Err(CanonError::InvalidRepairTarget);
    }

    if matches!(event.kind, EventKind::Blocked | EventKind::Failed) {
        let Some(class) = event.failure else {
            return Err(CanonError::MissingFailureClass);
        };

        if failure_requires_gate(class) && event.affected_gate.is_none() {
            return Err(CanonError::MissingAffectedGate);
        }

        if !failure_requires_gate(class) && event.affected_gate.is_some() {
            return Err(CanonError::UnexpectedAffectedGate);
        }
    }

    if matches!(event.kind, EventKind::Advanced | EventKind::Completed | EventKind::Recovered)
        && event.affected_gate.is_some()
    {
        return Err(CanonError::UnexpectedAffectedGate);
    }

    if event.kind == EventKind::Learned {
        let Some(action) = event.recovery_action else {
            return Err(CanonError::MissingRecoveryAction);
        };

        if action.target() != event.to {
            return Err(CanonError::InvalidLearnTarget);
        }

        if action.repaired_gate() != event.affected_gate {
            return Err(CanonError::InvalidRepairTarget);
        }
    }

    if event.decision == Decision::Halt && event.recovery_action != Some(RecoveryAction::Escalate) {
        return Err(CanonError::MissingRecoveryAction);
    }

    Ok(())
}

fn failure_requires_gate(class: FailureClass) -> bool {
    !matches!(
        class,
        FailureClass::RecoveryExhausted | FailureClass::ConvergenceFailed
    )
}

pub fn legal_transition(from: Phase, to: Phase, kind: EventKind, cause: Cause) -> bool {
    if to == Phase::Done && kind == EventKind::Failed && cause == Cause::MaxSteps {
        return true;
    }

    TRANSITIONS.iter().any(|transition| {
        transition.from == from
            && transition.to == to
            && transition.kind == kind
            && transition.cause == cause
    })
}

pub fn append_tlog_ndjson(
    path: impl AsRef<Path>,
    event: &ControlEvent,
) -> Result<(), CanonError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|_| CanonError::TlogIo)?;
    writeln!(file, "{}", encode_event_ndjson(event)).map_err(|_| CanonError::TlogIo)
}

pub fn write_tlog_ndjson(path: impl AsRef<Path>, tlog: &[ControlEvent]) -> Result<(), CanonError> {
    let mut file = File::create(path).map_err(|_| CanonError::TlogIo)?;
    for event in tlog {
        writeln!(file, "{}", encode_event_ndjson(event)).map_err(|_| CanonError::TlogIo)?;
    }
    Ok(())
}

pub fn load_tlog_ndjson(path: impl AsRef<Path>) -> Result<TLog, CanonError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(path).map_err(|_| CanonError::TlogIo)?;
    let reader = BufReader::new(file);
    let mut tlog = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(|_| CanonError::TlogIo)?;
        if line.trim().is_empty() {
            continue;
        }
        tlog.push(decode_event_ndjson(&line)?);
    }

    verify_tlog(&tlog)?;
    Ok(tlog)
}

pub fn replay_tlog_ndjson(
    initial: State,
    path: impl AsRef<Path>,
) -> Result<State, CanonError> {
    let tlog = load_tlog_ndjson(path)?;
    verify_tlog_from(initial, &tlog)
}

pub fn verify_tlog(tlog: &[ControlEvent]) -> Result<(), CanonError> {
    let Some(first) = tlog.first() else {
        return Ok(());
    };

    verify_tlog_from(first.state_before, tlog).map(|_| ())
}

pub fn verify_tlog_from(initial: State, tlog: &[ControlEvent]) -> Result<State, CanonError> {
    let mut state = initial;
    let mut prev_hash = 0;

    for (i, event) in tlog.iter().enumerate() {
        if event.seq != i as u64 + 1 || event.prev_hash != prev_hash {
            return Err(CanonError::InvalidHashChain);
        }

        if event.from != state.phase || event.state_before.phase != state.phase {
            return Err(CanonError::InvalidStateContinuity);
        }

        if event.state_before.packet != state.packet {
            return Err(CanonError::InvalidPacketContinuity);
        }

        if event.state_after.phase != event.to {
            return Err(CanonError::InvalidStateContinuity);
        }

        if event.state_before != state {
            return Err(CanonError::InvalidReplay);
        }

        if event.delta != semantic_diff(event.state_before, event.state_after) {
            return Err(CanonError::InvalidSemanticDelta);
        }

        validate_event(EventView {
            from: event.from,
            to: event.to,
            kind: event.kind,
            cause: event.cause,
            decision: event.decision,
            failure: event.failure,
            recovery_action: event.recovery_action,
            affected_gate: event.affected_gate,
        })?;

        let expected = hash_event(EventHashInput {
            seq: event.seq,
            prev_hash: event.prev_hash,
            from: event.from,
            to: event.to,
            kind: event.kind,
            cause: event.cause,
            delta: event.delta,
            evidence: event.evidence,
            decision: event.decision,
            failure: event.failure,
            recovery_action: event.recovery_action,
            affected_gate: event.affected_gate,
            state_before: event.state_before,
            state_after: event.state_after,
        });

        if expected != event.self_hash {
            return Err(CanonError::InvalidHashChain);
        }

        state = event.state_after;
        prev_hash = event.self_hash;
    }

    Ok(state)
}

#[derive(Clone, Copy)]
struct EventHashInput {
    seq: u64,
    prev_hash: u64,
    from: Phase,
    to: Phase,
    kind: EventKind,
    cause: Cause,
    delta: SemanticDelta,
    evidence: Evidence,
    decision: Decision,
    failure: Option<FailureClass>,
    recovery_action: Option<RecoveryAction>,
    affected_gate: Option<GateId>,
    state_before: State,
    state_after: State,
}

fn hash_event(input: EventHashInput) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    h = mix(h, input.seq);
    h = mix(h, input.prev_hash);
    h = mix(h, input.from as u64);
    h = mix(h, input.to as u64);
    h = mix(h, input.kind as u64);
    h = mix(h, input.cause as u64);
    h = mix(h, input.delta as u64);
    h = mix(h, input.evidence as u64);
    h = mix(h, input.decision as u64);
    h = mix_option_failure(h, input.failure);
    h = mix_option_recovery(h, input.recovery_action);
    h = mix_option_gate(h, input.affected_gate);
    h = mix(h, state_hash(input.state_before));
    h = mix(h, state_hash(input.state_after));
    h
}

fn state_hash(state: State) -> u64 {
    let mut h = 0x84222325cbf29ce4u64;
    h = mix(h, state.phase as u64);
    h = mix(h, gates_hash(state.gates));
    h = mix(h, packet_hash(state.packet));
    h = mix_option_failure(h, state.failure);
    h = mix_option_recovery(h, state.recovery_action);
    h = mix(h, state.recovery_attempts as u64);
    h
}

fn gates_hash(gates: GateSet) -> u64 {
    let mut h = 0x517cc1b727220a95u64;
    for id in GATE_ORDER {
        let gate = gates.get(id);
        h = mix(h, id as u64);
        h = mix(h, gate.status as u64);
        h = mix(h, gate.evidence as u64);
        h = mix(h, gate.version);
    }
    h
}

fn packet_hash(packet: Packet) -> u64 {
    let mut h = 0x94d049bb133111ebu64;
    h = mix(h, packet.objective_id);
    h = mix(h, packet.objective_required_tasks as u64);
    h = mix(h, packet.objective_done_tasks as u64);
    h = mix(h, packet.ready_tasks as u64);
    h = mix(h, packet.active_task_id);
    h = mix(h, packet.artifact_id);
    h = mix(h, packet.parent_artifact_id);
    h = mix(h, packet.artifact_bytes);
    h = mix(h, packet.artifact_lineage_hash);
    h = mix(h, packet.revision);
    h
}

fn mix(mut h: u64, x: u64) -> u64 {
    h ^= x;
    h = h.wrapping_mul(0x100000001b3);
    h
}

fn mix_option_failure(h: u64, value: Option<FailureClass>) -> u64 {
    match value {
        Some(v) => mix(mix(h, 1), v as u64),
        None => mix(h, 0),
    }
}

fn mix_option_recovery(h: u64, value: Option<RecoveryAction>) -> u64 {
    match value {
        Some(v) => mix(mix(h, 1), v as u64),
        None => mix(h, 0),
    }
}

fn mix_option_gate(h: u64, value: Option<GateId>) -> u64 {
    match value {
        Some(v) => mix(mix(h, 1), v as u64),
        None => mix(h, 0),
    }
}

fn encode_event_ndjson(event: &ControlEvent) -> String {
    let mut fields = Vec::with_capacity(84);
    push_event(&mut fields, *event);
    let body = fields
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{body}]")
}

fn decode_event_ndjson(line: &str) -> Result<ControlEvent, CanonError> {
    let trimmed = line.trim();
    let body = trimmed
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .ok_or(CanonError::InvalidTlogRecord)?;
    let mut fields = Vec::new();
    if !body.trim().is_empty() {
        for raw in body.split(',') {
            fields.push(raw.trim().parse::<u64>().map_err(|_| CanonError::InvalidTlogRecord)?);
        }
    }
    let mut cursor = Cursor { fields: &fields, pos: 0 };
    let event = pop_event(&mut cursor)?;
    if cursor.pos != fields.len() {
        return Err(CanonError::InvalidTlogRecord);
    }
    Ok(event)
}

struct Cursor<'a> {
    fields: &'a [u64],
    pos: usize,
}

impl Cursor<'_> {
    fn take(&mut self) -> Result<u64, CanonError> {
        let value = *self.fields.get(self.pos).ok_or(CanonError::InvalidTlogRecord)?;
        self.pos += 1;
        Ok(value)
    }
}

fn push_event(out: &mut Vec<u64>, event: ControlEvent) {
    out.extend([
        event.seq,
        event.from as u64,
        event.to as u64,
        event.kind as u64,
        event.cause as u64,
        event.delta as u64,
        event.evidence as u64,
        event.decision as u64,
        opt_failure_to_u64(event.failure),
        opt_recovery_to_u64(event.recovery_action),
        opt_gate_to_u64(event.affected_gate),
    ]);
    push_state(out, event.state_before);
    push_state(out, event.state_after);
    out.extend([event.prev_hash, event.self_hash]);
}

fn pop_event(cursor: &mut Cursor<'_>) -> Result<ControlEvent, CanonError> {
    let seq = cursor.take()?;
    let from = phase_from_u64(cursor.take()?)?;
    let to = phase_from_u64(cursor.take()?)?;
    let kind = event_kind_from_u64(cursor.take()?)?;
    let cause = cause_from_u64(cursor.take()?)?;
    let delta = semantic_delta_from_u64(cursor.take()?)?;
    let evidence = evidence_from_u64(cursor.take()?)?;
    let decision = decision_from_u64(cursor.take()?)?;
    let failure = opt_failure_from_u64(cursor.take()?)?;
    let recovery_action = opt_recovery_from_u64(cursor.take()?)?;
    let affected_gate = opt_gate_from_u64(cursor.take()?)?;
    let state_before = pop_state(cursor)?;
    let state_after = pop_state(cursor)?;
    let prev_hash = cursor.take()?;
    let self_hash = cursor.take()?;

    Ok(ControlEvent {
        seq,
        from,
        to,
        kind,
        cause,
        delta,
        evidence,
        decision,
        failure,
        recovery_action,
        affected_gate,
        state_before,
        state_after,
        prev_hash,
        self_hash,
    })
}

fn push_state(out: &mut Vec<u64>, state: State) {
    out.push(state.phase as u64);
    push_gates(out, state.gates);
    push_packet(out, state.packet);
    out.push(opt_failure_to_u64(state.failure));
    out.push(opt_recovery_to_u64(state.recovery_action));
    out.push(state.recovery_attempts as u64);
}

fn pop_state(cursor: &mut Cursor<'_>) -> Result<State, CanonError> {
    Ok(State {
        phase: phase_from_u64(cursor.take()?)?,
        gates: pop_gates(cursor)?,
        packet: pop_packet(cursor)?,
        failure: opt_failure_from_u64(cursor.take()?)?,
        recovery_action: opt_recovery_from_u64(cursor.take()?)?,
        recovery_attempts: u8_from_u64(cursor.take()?)?,
    })
}

fn push_gates(out: &mut Vec<u64>, gates: GateSet) {
    for id in GATE_ORDER {
        let gate = gates.get(id);
        out.extend([gate.status as u64, gate.evidence as u64, gate.version]);
    }
}

fn pop_gates(cursor: &mut Cursor<'_>) -> Result<GateSet, CanonError> {
    let mut gates = GateSet::default();
    for id in GATE_ORDER {
        *gates.get_mut(id) = Gate {
            status: gate_status_from_u64(cursor.take()?)?,
            evidence: evidence_from_u64(cursor.take()?)?,
            version: cursor.take()?,
        };
    }
    Ok(gates)
}

fn push_packet(out: &mut Vec<u64>, packet: Packet) {
    out.extend([
        packet.objective_id,
        packet.objective_required_tasks as u64,
        packet.objective_done_tasks as u64,
        packet.ready_tasks as u64,
        packet.active_task_id,
        packet.artifact_id,
        packet.parent_artifact_id,
        packet.artifact_bytes,
        packet.artifact_lineage_hash,
        packet.revision,
    ]);
}

fn pop_packet(cursor: &mut Cursor<'_>) -> Result<Packet, CanonError> {
    Ok(Packet {
        objective_id: cursor.take()?,
        objective_required_tasks: u8_from_u64(cursor.take()?)?,
        objective_done_tasks: u8_from_u64(cursor.take()?)?,
        ready_tasks: u8_from_u64(cursor.take()?)?,
        active_task_id: cursor.take()?,
        artifact_id: cursor.take()?,
        parent_artifact_id: cursor.take()?,
        artifact_bytes: cursor.take()?,
        artifact_lineage_hash: cursor.take()?,
        revision: cursor.take()?,
    })
}

fn u8_from_u64(value: u64) -> Result<u8, CanonError> {
    u8::try_from(value).map_err(|_| CanonError::InvalidTlogRecord)
}

fn opt_failure_to_u64(value: Option<FailureClass>) -> u64 {
    value.map(|v| v as u64).unwrap_or(0)
}

fn opt_recovery_to_u64(value: Option<RecoveryAction>) -> u64 {
    value.map(|v| v as u64).unwrap_or(0)
}

fn opt_gate_to_u64(value: Option<GateId>) -> u64 {
    value.map(|v| v as u64).unwrap_or(0)
}

fn opt_failure_from_u64(value: u64) -> Result<Option<FailureClass>, CanonError> {
    if value == 0 { Ok(None) } else { failure_from_u64(value).map(Some) }
}

fn opt_recovery_from_u64(value: u64) -> Result<Option<RecoveryAction>, CanonError> {
    if value == 0 { Ok(None) } else { recovery_from_u64(value).map(Some) }
}

fn opt_gate_from_u64(value: u64) -> Result<Option<GateId>, CanonError> {
    if value == 0 { Ok(None) } else { gate_id_from_u64(value).map(Some) }
}

fn phase_from_u64(value: u64) -> Result<Phase, CanonError> {
    match value {
        1 => Ok(Phase::Delta), 2 => Ok(Phase::Invariant), 3 => Ok(Phase::Analysis),
        4 => Ok(Phase::Judgment), 5 => Ok(Phase::Plan), 6 => Ok(Phase::Execute),
        7 => Ok(Phase::Verify), 8 => Ok(Phase::Eval), 9 => Ok(Phase::Recovery),
        10 => Ok(Phase::Learn), 11 => Ok(Phase::Done), _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn gate_status_from_u64(value: u64) -> Result<GateStatus, CanonError> {
    match value { 1 => Ok(GateStatus::Unknown), 2 => Ok(GateStatus::Pass), 3 => Ok(GateStatus::Fail), _ => Err(CanonError::InvalidTlogRecord) }
}

fn gate_id_from_u64(value: u64) -> Result<GateId, CanonError> {
    match value {
        1 => Ok(GateId::Invariant), 2 => Ok(GateId::Analysis), 3 => Ok(GateId::Judgment),
        4 => Ok(GateId::Plan), 5 => Ok(GateId::Execution), 6 => Ok(GateId::Verification),
        7 => Ok(GateId::Eval), _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn evidence_from_u64(value: u64) -> Result<Evidence, CanonError> {
    match value {
        1 => Ok(Evidence::Missing), 2 => Ok(Evidence::DeltaComputed), 3 => Ok(Evidence::InvariantProof),
        4 => Ok(Evidence::AnalysisReport), 5 => Ok(Evidence::JudgmentRecord), 6 => Ok(Evidence::PlanRecord),
        7 => Ok(Evidence::TaskReady), 8 => Ok(Evidence::ExecutionReceipt), 9 => Ok(Evidence::ArtifactReceipt),
        10 => Ok(Evidence::VerificationReport), 11 => Ok(Evidence::LineageProof), 12 => Ok(Evidence::EvalScore),
        13 => Ok(Evidence::RecoveryPolicy), 14 => Ok(Evidence::CompletionProof), 15 => Ok(Evidence::ConvergenceLimit),
        _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn failure_from_u64(value: u64) -> Result<FailureClass, CanonError> {
    match value {
        1 => Ok(FailureClass::InvariantUnknown), 2 => Ok(FailureClass::InvariantBlocked),
        3 => Ok(FailureClass::AnalysisMissing), 4 => Ok(FailureClass::AnalysisFailed),
        5 => Ok(FailureClass::JudgmentMissing), 6 => Ok(FailureClass::JudgmentFailed),
        7 => Ok(FailureClass::PlanMissing), 8 => Ok(FailureClass::PlanFailed),
        9 => Ok(FailureClass::PlanReadyQueueEmpty), 10 => Ok(FailureClass::ExecutionMissing),
        11 => Ok(FailureClass::ExecutionFailed), 12 => Ok(FailureClass::TaskReceiptMissing),
        13 => Ok(FailureClass::VerificationUnknown), 14 => Ok(FailureClass::VerificationFailed),
        15 => Ok(FailureClass::ArtifactLineageBroken), 16 => Ok(FailureClass::EvalMissing),
        17 => Ok(FailureClass::EvalFailed), 18 => Ok(FailureClass::RecoveryExhausted),
        19 => Ok(FailureClass::ConvergenceFailed), _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn recovery_from_u64(value: u64) -> Result<RecoveryAction, CanonError> {
    match value {
        1 => Ok(RecoveryAction::RecheckInvariant), 2 => Ok(RecoveryAction::RunAnalysis),
        3 => Ok(RecoveryAction::Rejudge), 4 => Ok(RecoveryAction::Replan),
        5 => Ok(RecoveryAction::BindReadyTask), 6 => Ok(RecoveryAction::Reexecute),
        7 => Ok(RecoveryAction::Reverify), 8 => Ok(RecoveryAction::RepairArtifactLineage),
        9 => Ok(RecoveryAction::RecomputeEval), 10 => Ok(RecoveryAction::Escalate),
        _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn event_kind_from_u64(value: u64) -> Result<EventKind, CanonError> {
    match value {
        1 => Ok(EventKind::Advanced), 2 => Ok(EventKind::Blocked), 3 => Ok(EventKind::Failed),
        4 => Ok(EventKind::Recovered), 5 => Ok(EventKind::Learned), 6 => Ok(EventKind::Completed),
        _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn cause_from_u64(value: u64) -> Result<Cause, CanonError> {
    match value {
        1 => Ok(Cause::Start), 2 => Ok(Cause::GatePassed), 3 => Ok(Cause::GateFailed),
        4 => Ok(Cause::EvidenceMissing), 5 => Ok(Cause::JudgmentMade), 6 => Ok(Cause::PlanReady),
        7 => Ok(Cause::ReadyQueueEmpty), 8 => Ok(Cause::ExecutionFinished), 9 => Ok(Cause::TaskReceiptMissing),
        10 => Ok(Cause::VerificationPassed), 11 => Ok(Cause::ArtifactLineageBroken),
        12 => Ok(Cause::EvalPassed), 13 => Ok(Cause::EvalFailed), 14 => Ok(Cause::RepairSelected),
        15 => Ok(Cause::RepairApplied), 16 => Ok(Cause::RecoveryLimit), 17 => Ok(Cause::MaxSteps),
        _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn decision_from_u64(value: u64) -> Result<Decision, CanonError> {
    match value {
        1 => Ok(Decision::Continue), 2 => Ok(Decision::Complete), 3 => Ok(Decision::Block),
        4 => Ok(Decision::Fail), 5 => Ok(Decision::Repair), 6 => Ok(Decision::Halt),
        _ => Err(CanonError::InvalidTlogRecord),
    }
}

fn semantic_delta_from_u64(value: u64) -> Result<SemanticDelta, CanonError> {
    match value {
        1 => Ok(SemanticDelta::NoChange), 2 => Ok(SemanticDelta::PhaseAdvanced),
        3 => Ok(SemanticDelta::FailureRaised), 4 => Ok(SemanticDelta::RepairSelected),
        5 => Ok(SemanticDelta::RepairApplied), 6 => Ok(SemanticDelta::PayloadChanged),
        7 => Ok(SemanticDelta::Completed), 8 => Ok(SemanticDelta::Halted),
        _ => Err(CanonError::InvalidTlogRecord),
    }
}

pub fn touch_all_surfaces() -> usize {
    let statuses = [GateStatus::Unknown, GateStatus::Pass, GateStatus::Fail];
    let evidences = [
        Evidence::Missing,
        Evidence::DeltaComputed,
        Evidence::InvariantProof,
        Evidence::AnalysisReport,
        Evidence::JudgmentRecord,
        Evidence::PlanRecord,
        Evidence::TaskReady,
        Evidence::ExecutionReceipt,
        Evidence::ArtifactReceipt,
        Evidence::VerificationReport,
        Evidence::LineageProof,
        Evidence::EvalScore,
        Evidence::RecoveryPolicy,
        Evidence::CompletionProof,
        Evidence::ConvergenceLimit,
    ];
    let failures = [
        FailureClass::InvariantUnknown,
        FailureClass::InvariantBlocked,
        FailureClass::AnalysisMissing,
        FailureClass::AnalysisFailed,
        FailureClass::JudgmentMissing,
        FailureClass::JudgmentFailed,
        FailureClass::PlanMissing,
        FailureClass::PlanFailed,
        FailureClass::PlanReadyQueueEmpty,
        FailureClass::ExecutionMissing,
        FailureClass::ExecutionFailed,
        FailureClass::TaskReceiptMissing,
        FailureClass::VerificationUnknown,
        FailureClass::VerificationFailed,
        FailureClass::ArtifactLineageBroken,
        FailureClass::EvalMissing,
        FailureClass::EvalFailed,
        FailureClass::RecoveryExhausted,
        FailureClass::ConvergenceFailed,
    ];
    let actions = [
        RecoveryAction::RecheckInvariant,
        RecoveryAction::RunAnalysis,
        RecoveryAction::Rejudge,
        RecoveryAction::Replan,
        RecoveryAction::BindReadyTask,
        RecoveryAction::Reexecute,
        RecoveryAction::Reverify,
        RecoveryAction::RepairArtifactLineage,
        RecoveryAction::RecomputeEval,
        RecoveryAction::Escalate,
    ];
    let kinds = [
        EventKind::Advanced,
        EventKind::Blocked,
        EventKind::Failed,
        EventKind::Recovered,
        EventKind::Learned,
        EventKind::Completed,
    ];
    let causes = [
        Cause::Start,
        Cause::GatePassed,
        Cause::GateFailed,
        Cause::EvidenceMissing,
        Cause::JudgmentMade,
        Cause::PlanReady,
        Cause::ReadyQueueEmpty,
        Cause::ExecutionFinished,
        Cause::TaskReceiptMissing,
        Cause::VerificationPassed,
        Cause::ArtifactLineageBroken,
        Cause::EvalPassed,
        Cause::EvalFailed,
        Cause::RepairSelected,
        Cause::RepairApplied,
        Cause::RecoveryLimit,
        Cause::MaxSteps,
    ];
    let decisions = [
        Decision::Continue,
        Decision::Complete,
        Decision::Block,
        Decision::Fail,
        Decision::Repair,
        Decision::Halt,
    ];
    let deltas = [
        SemanticDelta::NoChange,
        SemanticDelta::PhaseAdvanced,
        SemanticDelta::FailureRaised,
        SemanticDelta::RepairSelected,
        SemanticDelta::RepairApplied,
        SemanticDelta::PayloadChanged,
        SemanticDelta::Completed,
        SemanticDelta::Halted,
    ];
    let errors = [
        CanonError::IllegalEvent {
            from: Phase::Delta,
            to: Phase::Done,
            kind: EventKind::Failed,
        },
        CanonError::MissingFailureClass,
        CanonError::UnexpectedFailureClass,
        CanonError::MissingRecoveryAction,
        CanonError::UnexpectedRecoveryAction,
        CanonError::InvalidLearnTarget,
        CanonError::InvalidRepairTarget,
        CanonError::InvalidCompletion,
        CanonError::InvalidStateContinuity,
        CanonError::InvalidPacketContinuity,
        CanonError::InvalidSemanticDelta,
        CanonError::InvalidHashChain,
        CanonError::InvalidReplay,
        CanonError::MissingAffectedGate,
        CanonError::UnexpectedAffectedGate,
    ];

    let mut gates = GateSet::default();
    gates.set_fail(GateId::Eval, Evidence::EvalScore);
    gates.set_pass(GateId::Eval, evidence_for_gate(GateId::Eval));

    let mut packet = Packet::empty();
    packet.bind_ready_task();
    packet.materialize_artifact();
    packet.repair_lineage();
    packet.complete_objective();

    let error_score = errors
        .iter()
        .map(|e| match e {
            CanonError::IllegalEvent { from, to, kind } => {
                *from as usize + *to as usize + *kind as usize
            }
            CanonError::MissingFailureClass => 1,
            CanonError::UnexpectedFailureClass => 2,
            CanonError::MissingRecoveryAction => 3,
            CanonError::UnexpectedRecoveryAction => 4,
            CanonError::InvalidLearnTarget => 5,
            CanonError::InvalidRepairTarget => 6,
            CanonError::InvalidCompletion => 7,
            CanonError::InvalidStateContinuity => 8,
            CanonError::InvalidPacketContinuity => 9,
            CanonError::InvalidSemanticDelta => 10,
            CanonError::InvalidHashChain => 11,
            CanonError::InvalidReplay => 12,
            CanonError::MissingAffectedGate => 13,
            CanonError::UnexpectedAffectedGate => 14,
            CanonError::TlogIo => 15,
            CanonError::InvalidTlogRecord => 16,
        })
        .sum::<usize>();

    PHASES.len()
        + statuses.len()
        + GATE_ORDER.len()
        + TRANSITIONS.len()
        + evidences.len()
        + failures.len()
        + actions.len()
        + kinds.len()
        + causes.len()
        + decisions.len()
        + deltas.len()
        + gates.all_passed() as usize
        + packet.lineage_valid() as usize
        + error_score
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunReport {
    pub ready_state: State,
    pub ready_tlog: TLog,
    pub repaired_state: State,
    pub repaired_tlog: TLog,
}

impl RunReport {
    pub fn both_succeeded(&self) -> bool {
        self.ready_state.is_success() && self.repaired_state.is_success()
    }
}

pub fn run_demo() -> Result<RunReport, CanonError> {
    let _surface_count = touch_all_surfaces();
    let cfg = RuntimeConfig::default();

    let (ready_state, ready_tlog) = run_until_done(State::ready(), cfg)?;
    if !ready_state.is_success() {
        return Err(CanonError::InvalidCompletion);
    }
    verify_tlog(&ready_tlog)?;

    let (repaired_state, repaired_tlog) = run_until_done(State::default(), cfg)?;
    if !repaired_state.is_success() {
        return Err(CanonError::InvalidCompletion);
    }
    verify_tlog(&repaired_tlog)?;

    Ok(RunReport {
        ready_state,
        ready_tlog,
        repaired_state,
        repaired_tlog,
    })
}

pub fn run() {
    run_demo().expect("canonical demo failed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_state_converges_to_done() {
        let (state, tlog) = run_until_done(State::ready(), RuntimeConfig::default()).unwrap();

        assert!(state.is_success());
        assert_eq!(tlog.last().unwrap().kind, EventKind::Completed);
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn unknown_gates_repair_with_payload_lineage() {
        let cfg = RuntimeConfig {
            max_steps: 96,
            max_recovery_attempts: 8,
        };
        let (state, tlog) = run_until_done(State::default(), cfg).unwrap();

        assert!(state.is_success());
        assert_eq!(state.recovery_attempts, 7);
        assert!(state.packet.lineage_valid());

        assert!(tlog.iter().any(|e| e.kind == EventKind::Recovered));
        assert!(tlog.iter().any(|e| e.kind == EventKind::Learned));
        assert!(tlog.iter().any(|e| {
            e.recovery_action == Some(RecoveryAction::BindReadyTask)
                && e.affected_gate == Some(GateId::Plan)
                && e.evidence == Evidence::TaskReady
        }));
        assert!(tlog.iter().any(|e| {
            e.recovery_action == Some(RecoveryAction::Reexecute)
                && e.affected_gate == Some(GateId::Execution)
                && e.state_after.packet.artifact_present()
        }));

        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn plan_pass_without_ready_task_repairs_ready_queue() {
        let cfg = RuntimeConfig {
            max_steps: 32,
            max_recovery_attempts: 4,
        };

        let mut state = State::ready();
        state.phase = Phase::Plan;
        state.packet.ready_tasks = 0;
        state.packet.active_task_id = 0;
        state.packet.objective_done_tasks = 0;

        let (state, tlog) = run_until_done(state, cfg).unwrap();

        assert!(state.is_success());
        assert!(tlog.iter().any(|e| {
            e.failure == Some(FailureClass::PlanReadyQueueEmpty)
                && e.cause == Cause::ReadyQueueEmpty
                && e.affected_gate == Some(GateId::Plan)
        }));
        assert!(tlog.iter().any(|e| {
            e.recovery_action == Some(RecoveryAction::BindReadyTask)
                && e.evidence == Evidence::TaskReady
        }));

        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn artifact_lineage_failure_repairs_lineage() {
        let cfg = RuntimeConfig {
            max_steps: 32,
            max_recovery_attempts: 4,
        };

        let mut state = State::ready();
        state.phase = Phase::Verify;
        state.packet.artifact_lineage_hash = 123;

        let (state, tlog) = run_until_done(state, cfg).unwrap();

        assert!(state.is_success());
        assert!(tlog.iter().any(|e| {
            e.failure == Some(FailureClass::ArtifactLineageBroken)
                && e.cause == Cause::ArtifactLineageBroken
        }));
        assert!(tlog.iter().any(|e| {
            e.recovery_action == Some(RecoveryAction::RepairArtifactLineage)
                && e.evidence == Evidence::LineageProof
                && e.state_after.packet.lineage_valid()
        }));

        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn replay_reconstructs_final_state() {
        let initial = State::default();
        let (state, tlog) = run_until_done(initial, RuntimeConfig::default()).unwrap();
        let replayed = verify_tlog_from(initial, &tlog).unwrap();

        assert_eq!(replayed, state);
    }

    #[test]
    fn disk_tlog_roundtrip_replays_final_state() {
        let initial = State::default();
        let (state, tlog) = run_until_done(initial, RuntimeConfig::default()).unwrap();
        let path = std::env::temp_dir().join(format!(
            "ai-tlog-roundtrip-{}-{}.ndjson",
            std::process::id(),
            tlog.len()
        ));

        write_tlog_ndjson(&path, &tlog).unwrap();
        let loaded = load_tlog_ndjson(&path).unwrap();
        let replayed = replay_tlog_ndjson(initial, &path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(loaded, tlog);
        assert_eq!(replayed, state);
    }

    #[test]
    fn durable_runner_resumes_from_disk_tlog() {
        let path = std::env::temp_dir().join(format!(
            "ai-tlog-resume-{}.ndjson",
            std::process::id()
        ));
        std::fs::remove_file(&path).ok();

        let mut partial_state = State::default();
        let mut partial_tlog = Vec::new();
        for _ in 0..3 {
            tick_durable(
                &mut partial_state,
                &mut partial_tlog,
                &path,
                RuntimeConfig::default(),
            )
            .unwrap();
        }

        let (resumed_state, resumed_tlog) =
            run_until_done_durable(State::default(), RuntimeConfig::default(), &path).unwrap();
        let replayed = replay_tlog_ndjson(State::default(), &path).unwrap();
        std::fs::remove_file(&path).ok();

        assert!(resumed_state.is_success());
        assert!(resumed_tlog.len() > partial_tlog.len());
        assert_eq!(replayed, resumed_state);
    }

    #[test]
    fn eval_cannot_complete_when_prior_gate_is_bad() {
        let cfg = RuntimeConfig {
            max_steps: 32,
            max_recovery_attempts: 4,
        };

        let mut state = State::ready();
        state.phase = Phase::Eval;
        state.gates.plan = Gate::fail(Evidence::PlanRecord);

        let (state, tlog) = run_until_done(state, cfg).unwrap();

        assert!(state.is_success());
        assert_eq!(state.gates.plan.status, GateStatus::Pass);

        assert!(tlog.iter().any(|e| {
            e.failure == Some(FailureClass::PlanFailed) && e.affected_gate == Some(GateId::Plan)
        }));

        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn low_recovery_budget_halts() {
        let cfg = RuntimeConfig {
            max_steps: 96,
            max_recovery_attempts: 1,
        };

        let (state, tlog) = run_until_done(State::default(), cfg).unwrap();

        assert_eq!(state.phase, Phase::Done);
        assert_eq!(state.failure, Some(FailureClass::RecoveryExhausted));
        assert_eq!(tlog.last().unwrap().decision, Decision::Halt);
        assert_eq!(
            tlog.last().unwrap().recovery_action,
            Some(RecoveryAction::Escalate)
        );

        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn transition_table_rejects_illegal_pair() {
        assert!(!legal_transition(
            Phase::Plan,
            Phase::Done,
            EventKind::Completed,
            Cause::EvalPassed
        ));
        assert!(legal_transition(
            Phase::Plan,
            Phase::Recovery,
            EventKind::Failed,
            Cause::ReadyQueueEmpty
        ));
    }

    #[test]
    fn tampered_tlog_fails_verification() {
        let (_, mut tlog) = run_until_done(State::ready(), RuntimeConfig::default()).unwrap();
        tlog[0].self_hash = 123;

        assert_eq!(verify_tlog(&tlog), Err(CanonError::InvalidHashChain));
    }

    #[test]
    fn broken_state_continuity_fails_verification() {
        let (_, mut tlog) = run_until_done(State::ready(), RuntimeConfig::default()).unwrap();
        tlog[1].state_before.phase = Phase::Done;

        assert_eq!(verify_tlog(&tlog), Err(CanonError::InvalidStateContinuity));
    }

    #[test]
    fn broken_packet_continuity_fails_verification() {
        let (_, mut tlog) = run_until_done(State::ready(), RuntimeConfig::default()).unwrap();
        tlog[1].state_before.packet.revision =
            tlog[1].state_before.packet.revision.saturating_add(1);

        assert_eq!(verify_tlog(&tlog), Err(CanonError::InvalidPacketContinuity));
    }

    #[test]
    fn tampered_semantic_delta_fails_verification() {
        let (_, mut tlog) = run_until_done(State::ready(), RuntimeConfig::default()).unwrap();
        tlog[0].delta = SemanticDelta::NoChange;

        assert_eq!(verify_tlog(&tlog), Err(CanonError::InvalidSemanticDelta));
    }

    #[test]
    fn broken_state_after_phase_fails_verification() {
        let (_, mut tlog) = run_until_done(State::ready(), RuntimeConfig::default()).unwrap();
        tlog[0].state_after.phase = Phase::Done;

        assert_eq!(verify_tlog(&tlog), Err(CanonError::InvalidStateContinuity));
    }

    #[test]
    fn failed_event_requires_affected_gate_when_failure_is_gate_scoped() {
        let event = EventView {
            from: Phase::Plan,
            to: Phase::Recovery,
            kind: EventKind::Failed,
            cause: Cause::ReadyQueueEmpty,
            decision: Decision::Fail,
            failure: Some(FailureClass::PlanReadyQueueEmpty),
            recovery_action: None,
            affected_gate: None,
        };

        assert_eq!(validate_event(event), Err(CanonError::MissingAffectedGate));
    }

    #[test]
    fn terminal_failure_rejects_affected_gate() {
        let event = EventView {
            from: Phase::Recovery,
            to: Phase::Done,
            kind: EventKind::Failed,
            cause: Cause::RecoveryLimit,
            decision: Decision::Halt,
            failure: Some(FailureClass::RecoveryExhausted),
            recovery_action: Some(RecoveryAction::Escalate),
            affected_gate: Some(GateId::Eval),
        };

        assert_eq!(validate_event(event), Err(CanonError::UnexpectedAffectedGate));
    }
}
