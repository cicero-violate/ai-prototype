use super::{
    CapabilityRegistryProjection, Evidence, FailureClass, GateId, Phase, RecoveryAction,
    RuntimeConfig, State,
};

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
    EvidenceSubmitted = 20,
    AgentCycleEventSubmitted = 21,
    WaveDispatched = 22,
    ChildTaskCompleted = 23,
    SymbolMutationObserved = 24,
    ArchitecturalDecisionMade = 25,
    CostGateEvaluated = 26,
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
    SymbolLayerChanged = 11,
    SymbolRenamed = 12,
    CostThresholdExceeded = 13,
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
    pub capability_registry_projection: CapabilityRegistryProjection,
    pub api_command_id: u64,
    pub api_command_hash: u64,
    pub prev_hash: u64,
    pub self_hash: u64,
}
