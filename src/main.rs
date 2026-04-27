#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
enum Phase {
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
enum GateStatus {
    Unknown = 1,
    Pass = 2,
    Fail = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
enum GateId {
    Invariant = 1,
    Analysis = 2,
    Judgment = 3,
    Plan = 4,
    Execution = 5,
    Verification = 6,
    Eval = 7,
}

const GATE_ORDER: [GateId; 7] = [
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
enum Evidence {
    Missing = 1,
    DeltaComputed = 2,
    InvariantProof = 3,
    AnalysisReport = 4,
    JudgmentRecord = 5,
    PlanRecord = 6,
    ExecutionReceipt = 7,
    VerificationReport = 8,
    EvalScore = 9,
    RecoveryPolicy = 10,
    CompletionProof = 11,
    ConvergenceLimit = 12,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Gate {
    status: GateStatus,
    evidence: Evidence,
    version: u64,
}

impl Gate {
    const fn unknown() -> Self {
        Self {
            status: GateStatus::Unknown,
            evidence: Evidence::Missing,
            version: 0,
        }
    }

    const fn pass(evidence: Evidence) -> Self {
        Self {
            status: GateStatus::Pass,
            evidence,
            version: 1,
        }
    }

    const fn fail(evidence: Evidence) -> Self {
        Self {
            status: GateStatus::Fail,
            evidence,
            version: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct GateSet {
    invariant: Gate,
    analysis: Gate,
    judgment: Gate,
    plan: Gate,
    execution: Gate,
    verification: Gate,
    eval: Gate,
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
    fn ready() -> Self {
        Self {
            invariant: Gate::pass(Evidence::InvariantProof),
            analysis: Gate::pass(Evidence::AnalysisReport),
            judgment: Gate::pass(Evidence::JudgmentRecord),
            plan: Gate::pass(Evidence::PlanRecord),
            execution: Gate::pass(Evidence::ExecutionReceipt),
            verification: Gate::pass(Evidence::VerificationReport),
            eval: Gate::pass(Evidence::EvalScore),
        }
    }

    fn get(self, id: GateId) -> Gate {
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

    fn get_mut(&mut self, id: GateId) -> &mut Gate {
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

    fn set_pass(&mut self, id: GateId, evidence: Evidence) {
        let gate = self.get_mut(id);
        *gate = Gate {
            status: GateStatus::Pass,
            evidence,
            version: gate.version.saturating_add(1),
        };
    }

    fn set_fail(&mut self, id: GateId, evidence: Evidence) {
        let gate = self.get_mut(id);
        let next_version = gate.version.saturating_add(1);
        let mut failed = Gate::fail(evidence);
        failed.version = next_version;
        *gate = failed;
    }

    fn first_non_pass(self) -> Option<(GateId, Gate)> {
        for id in GATE_ORDER {
            let gate = self.get(id);
            if gate.status != GateStatus::Pass {
                return Some((id, gate));
            }
        }
        None
    }

    fn all_passed(self) -> bool {
        self.first_non_pass().is_none()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
enum FailureClass {
    InvariantUnknown = 1,
    InvariantBlocked = 2,
    AnalysisMissing = 3,
    AnalysisFailed = 4,
    JudgmentMissing = 5,
    JudgmentFailed = 6,
    PlanMissing = 7,
    PlanFailed = 8,
    ExecutionMissing = 9,
    ExecutionFailed = 10,
    VerificationUnknown = 11,
    VerificationFailed = 12,
    EvalMissing = 13,
    EvalFailed = 14,
    RecoveryExhausted = 15,
    ConvergenceFailed = 16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
enum RecoveryAction {
    RecheckInvariant = 1,
    RunAnalysis = 2,
    Rejudge = 3,
    Replan = 4,
    Reexecute = 5,
    Reverify = 6,
    RecomputeEval = 7,
    Escalate = 8,
}

impl RecoveryAction {
    fn target(self) -> Phase {
        match self {
            RecoveryAction::RecheckInvariant => Phase::Invariant,
            RecoveryAction::RunAnalysis => Phase::Analysis,
            RecoveryAction::Rejudge => Phase::Judgment,
            RecoveryAction::Replan => Phase::Plan,
            RecoveryAction::Reexecute => Phase::Execute,
            RecoveryAction::Reverify => Phase::Verify,
            RecoveryAction::RecomputeEval => Phase::Eval,
            RecoveryAction::Escalate => Phase::Done,
        }
    }

    fn repaired_gate(self) -> Option<GateId> {
        match self {
            RecoveryAction::RecheckInvariant => Some(GateId::Invariant),
            RecoveryAction::RunAnalysis => Some(GateId::Analysis),
            RecoveryAction::Rejudge => Some(GateId::Judgment),
            RecoveryAction::Replan => Some(GateId::Plan),
            RecoveryAction::Reexecute => Some(GateId::Execution),
            RecoveryAction::Reverify => Some(GateId::Verification),
            RecoveryAction::RecomputeEval => Some(GateId::Eval),
            RecoveryAction::Escalate => None,
        }
    }

    fn produced_evidence(self) -> Option<Evidence> {
        self.repaired_gate().map(evidence_for_gate)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
enum EventKind {
    Advanced = 1,
    Blocked = 2,
    Failed = 3,
    Recovered = 4,
    Learned = 5,
    Completed = 6,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
enum Cause {
    Start = 1,
    GatePassed = 2,
    GateFailed = 3,
    EvidenceMissing = 4,
    JudgmentMade = 5,
    PlanReady = 6,
    ExecutionFinished = 7,
    VerificationPassed = 8,
    EvalPassed = 9,
    EvalFailed = 10,
    RepairSelected = 11,
    RepairApplied = 12,
    RecoveryLimit = 13,
    MaxSteps = 14,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
enum Decision {
    Continue = 1,
    Complete = 2,
    Block = 3,
    Fail = 4,
    Repair = 5,
    Halt = 6,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
enum SemanticDelta {
    NoChange = 1,
    PhaseAdvanced = 2,
    FailureRaised = 3,
    RepairSelected = 4,
    RepairApplied = 5,
    Completed = 6,
    Halted = 7,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct State {
    phase: Phase,
    gates: GateSet,
    failure: Option<FailureClass>,
    recovery_action: Option<RecoveryAction>,
    recovery_attempts: u8,
}

impl Default for State {
    fn default() -> Self {
        Self {
            phase: Phase::Delta,
            gates: GateSet::default(),
            failure: None,
            recovery_action: None,
            recovery_attempts: 0,
        }
    }
}

impl State {
    fn ready() -> Self {
        Self {
            phase: Phase::Delta,
            gates: GateSet::ready(),
            failure: None,
            recovery_action: None,
            recovery_attempts: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimeConfig {
    max_steps: u64,
    max_recovery_attempts: u8,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_steps: 96,
            max_recovery_attempts: 8,
        }
    }
}

type TLog = Vec<ControlEvent>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ControlEvent {
    seq: u64,
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
    prev_hash: u64,
    self_hash: u64,
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
enum CanonError {
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
    InvalidHashChain,
}

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

        if outcome.kind == EventKind::Completed && !after.gates.all_passed() {
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
        Phase::Plan => gate_step(input, GateId::Plan, Phase::Execute, Cause::PlanReady),
        Phase::Execute => gate_step(
            input,
            GateId::Execution,
            Phase::Verify,
            Cause::ExecutionFinished,
        ),
        Phase::Verify => gate_step(
            input,
            GateId::Verification,
            Phase::Eval,
            Cause::VerificationPassed,
        ),
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

fn eval_step(input: State) -> Outcome {
    let mut s = input;

    if let Some((gate_id, gate)) = s.gates.first_non_pass() {
        return raise_gate_failure(&mut s, gate_id, gate);
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

    let gate = action
        .repaired_gate()
        .expect("non-escalation repair action must target a gate");
    let evidence = action
        .produced_evidence()
        .expect("non-escalation repair action must produce gate evidence");

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

fn recovery_action_for(class: FailureClass) -> RecoveryAction {
    match class {
        FailureClass::InvariantUnknown | FailureClass::InvariantBlocked => {
            RecoveryAction::RecheckInvariant
        }
        FailureClass::AnalysisMissing | FailureClass::AnalysisFailed => RecoveryAction::RunAnalysis,
        FailureClass::JudgmentMissing | FailureClass::JudgmentFailed => RecoveryAction::Rejudge,
        FailureClass::PlanMissing | FailureClass::PlanFailed => RecoveryAction::Replan,
        FailureClass::ExecutionMissing | FailureClass::ExecutionFailed => RecoveryAction::Reexecute,
        FailureClass::VerificationUnknown | FailureClass::VerificationFailed => {
            RecoveryAction::Reverify
        }
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
        GateId::Plan => Evidence::PlanRecord,
        GateId::Execution => Evidence::ExecutionReceipt,
        GateId::Verification => Evidence::VerificationReport,
        GateId::Eval => Evidence::EvalScore,
    }
}

fn tick(state: &mut State, tlog: &mut TLog, cfg: RuntimeConfig) -> Result<(), CanonError> {
    let before = *state;
    let outcome = reduce(before, cfg);
    CanonicalWriter::append(tlog, before, outcome)?;
    *state = outcome.state;
    Ok(())
}

fn run_until_done(mut state: State, cfg: RuntimeConfig) -> Result<(State, TLog), CanonError> {
    let mut tlog = Vec::new();

    for _ in 0..cfg.max_steps {
        if state.phase == Phase::Done {
            return Ok((state, tlog));
        }

        tick(&mut state, &mut tlog, cfg)?;
    }

    let before = state;
    state.phase = Phase::Done;
    state.failure = Some(FailureClass::ConvergenceFailed);
    state.recovery_action = Some(RecoveryAction::Escalate);

    let outcome = Outcome {
        state,
        kind: EventKind::Failed,
        cause: Cause::MaxSteps,
        evidence: Evidence::ConvergenceLimit,
        decision: Decision::Halt,
        failure: Some(FailureClass::ConvergenceFailed),
        recovery_action: Some(RecoveryAction::Escalate),
        affected_gate: None,
    };

    CanonicalWriter::append(&mut tlog, before, outcome)?;
    Ok((state, tlog))
}

fn semantic_diff(a: State, b: State) -> SemanticDelta {
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
        EventKind::Blocked | EventKind::Failed | EventKind::Recovered
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

fn legal_transition(from: Phase, to: Phase, kind: EventKind, cause: Cause) -> bool {
    if to == Phase::Done && kind == EventKind::Failed && cause == Cause::MaxSteps {
        return true;
    }

    matches!(
        (from, to, kind, cause),
        (
            Phase::Delta,
            Phase::Invariant,
            EventKind::Advanced,
            Cause::Start
        ) | (
            Phase::Invariant,
            Phase::Analysis,
            EventKind::Advanced,
            Cause::GatePassed
        ) | (
            Phase::Invariant,
            Phase::Recovery,
            EventKind::Blocked,
            Cause::EvidenceMissing
        ) | (
            Phase::Invariant,
            Phase::Recovery,
            EventKind::Blocked,
            Cause::GateFailed
        ) | (
            Phase::Analysis,
            Phase::Judgment,
            EventKind::Advanced,
            Cause::GatePassed
        ) | (
            Phase::Analysis,
            Phase::Recovery,
            EventKind::Failed,
            Cause::EvidenceMissing
        ) | (
            Phase::Analysis,
            Phase::Recovery,
            EventKind::Failed,
            Cause::GateFailed
        ) | (
            Phase::Judgment,
            Phase::Plan,
            EventKind::Advanced,
            Cause::JudgmentMade
        ) | (
            Phase::Judgment,
            Phase::Recovery,
            EventKind::Failed,
            Cause::EvidenceMissing
        ) | (
            Phase::Judgment,
            Phase::Recovery,
            EventKind::Failed,
            Cause::GateFailed
        ) | (
            Phase::Plan,
            Phase::Execute,
            EventKind::Advanced,
            Cause::PlanReady
        ) | (
            Phase::Plan,
            Phase::Recovery,
            EventKind::Failed,
            Cause::EvidenceMissing
        ) | (
            Phase::Plan,
            Phase::Recovery,
            EventKind::Failed,
            Cause::GateFailed
        ) | (
            Phase::Execute,
            Phase::Verify,
            EventKind::Advanced,
            Cause::ExecutionFinished
        ) | (
            Phase::Execute,
            Phase::Recovery,
            EventKind::Failed,
            Cause::EvidenceMissing
        ) | (
            Phase::Execute,
            Phase::Recovery,
            EventKind::Failed,
            Cause::GateFailed
        ) | (
            Phase::Verify,
            Phase::Eval,
            EventKind::Advanced,
            Cause::VerificationPassed
        ) | (
            Phase::Verify,
            Phase::Recovery,
            EventKind::Failed,
            Cause::EvidenceMissing
        ) | (
            Phase::Verify,
            Phase::Recovery,
            EventKind::Failed,
            Cause::GateFailed
        ) | (
            Phase::Eval,
            Phase::Done,
            EventKind::Completed,
            Cause::EvalPassed
        ) | (
            Phase::Eval,
            Phase::Recovery,
            EventKind::Failed,
            Cause::EvidenceMissing
        ) | (
            Phase::Eval,
            Phase::Recovery,
            EventKind::Failed,
            Cause::GateFailed
        ) | (
            Phase::Recovery,
            Phase::Learn,
            EventKind::Recovered,
            Cause::RepairSelected
        ) | (
            Phase::Recovery,
            Phase::Done,
            EventKind::Failed,
            Cause::RecoveryLimit
        ) | (
            Phase::Learn,
            Phase::Invariant,
            EventKind::Learned,
            Cause::RepairApplied
        ) | (
            Phase::Learn,
            Phase::Analysis,
            EventKind::Learned,
            Cause::RepairApplied
        ) | (
            Phase::Learn,
            Phase::Judgment,
            EventKind::Learned,
            Cause::RepairApplied
        ) | (
            Phase::Learn,
            Phase::Plan,
            EventKind::Learned,
            Cause::RepairApplied
        ) | (
            Phase::Learn,
            Phase::Execute,
            EventKind::Learned,
            Cause::RepairApplied
        ) | (
            Phase::Learn,
            Phase::Verify,
            EventKind::Learned,
            Cause::RepairApplied
        ) | (
            Phase::Learn,
            Phase::Eval,
            EventKind::Learned,
            Cause::RepairApplied
        ) | (
            Phase::Learn,
            Phase::Done,
            EventKind::Learned,
            Cause::RepairApplied
        ) | (
            Phase::Done,
            Phase::Done,
            EventKind::Completed,
            Cause::EvalPassed
        )
    )
}

fn verify_tlog(tlog: &[ControlEvent]) -> Result<(), CanonError> {
    let mut prev_hash = 0;
    let mut prev_to: Option<Phase> = None;

    for (i, event) in tlog.iter().enumerate() {
        if event.seq != i as u64 + 1 || event.prev_hash != prev_hash {
            return Err(CanonError::InvalidHashChain);
        }

        if let Some(expected_from) = prev_to {
            if event.from != expected_from {
                return Err(CanonError::InvalidStateContinuity);
            }
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
        });

        if expected != event.self_hash {
            return Err(CanonError::InvalidHashChain);
        }

        prev_hash = event.self_hash;
        prev_to = Some(event.to);
    }

    Ok(())
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

fn touch_all_surfaces() -> usize {
    let phases = [
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
    let statuses = [GateStatus::Unknown, GateStatus::Pass, GateStatus::Fail];
    let evidences = [
        Evidence::Missing,
        Evidence::DeltaComputed,
        Evidence::InvariantProof,
        Evidence::AnalysisReport,
        Evidence::JudgmentRecord,
        Evidence::PlanRecord,
        Evidence::ExecutionReceipt,
        Evidence::VerificationReport,
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
        FailureClass::ExecutionMissing,
        FailureClass::ExecutionFailed,
        FailureClass::VerificationUnknown,
        FailureClass::VerificationFailed,
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
        RecoveryAction::Reexecute,
        RecoveryAction::Reverify,
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
        Cause::ExecutionFinished,
        Cause::VerificationPassed,
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
        SemanticDelta::Completed,
        SemanticDelta::Halted,
    ];

    let mut gates = GateSet::default();
    gates.set_fail(GateId::Eval, Evidence::EvalScore);
    let fail_gate = gates.eval;
    gates.set_pass(GateId::Eval, Evidence::EvalScore);

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
        CanonError::InvalidHashChain,
    ];

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
            CanonError::InvalidHashChain => 9,
        })
        .sum::<usize>();

    phases.len()
        + statuses.len()
        + GATE_ORDER.len()
        + evidences.len()
        + failures.len()
        + actions.len()
        + kinds.len()
        + causes.len()
        + decisions.len()
        + deltas.len()
        + fail_gate.version as usize
        + gates.all_passed() as usize
        + error_score
}

fn main() {
    assert!(touch_all_surfaces() > 0);

    let cfg = RuntimeConfig::default();

    let (ready_state, ready_tlog) =
        run_until_done(State::ready(), cfg).expect("ready canonical run failed");
    assert_eq!(ready_state.phase, Phase::Done);
    assert_eq!(ready_state.failure, None);
    verify_tlog(&ready_tlog).expect("ready tlog invalid");

    let (repaired_state, repaired_tlog) =
        run_until_done(State::default(), cfg).expect("repair canonical run failed");
    assert_eq!(repaired_state.phase, Phase::Done);
    assert_eq!(repaired_state.failure, None);
    verify_tlog(&repaired_tlog).expect("repair tlog invalid");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_state_converges_to_done() {
        let (state, tlog) = run_until_done(State::ready(), RuntimeConfig::default()).unwrap();

        assert_eq!(state.phase, Phase::Done);
        assert_eq!(state.failure, None);
        assert_eq!(tlog.last().unwrap().kind, EventKind::Completed);
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn unknown_gates_repair_with_gate_specific_evidence() {
        let cfg = RuntimeConfig {
            max_steps: 96,
            max_recovery_attempts: 8,
        };
        let (state, tlog) = run_until_done(State::default(), cfg).unwrap();

        assert_eq!(state.phase, Phase::Done);
        assert_eq!(state.failure, None);
        assert_eq!(state.recovery_attempts, 7);
        assert!(state.gates.all_passed());

        assert!(tlog.iter().any(|e| e.kind == EventKind::Recovered));
        assert!(tlog.iter().any(|e| e.kind == EventKind::Learned));
        assert!(tlog
            .iter()
            .any(|e| e.affected_gate == Some(GateId::Plan) && e.evidence == Evidence::PlanRecord));

        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn failed_eval_repairs_eval_with_eval_score() {
        let cfg = RuntimeConfig {
            max_steps: 24,
            max_recovery_attempts: 2,
        };

        let mut state = State::ready();
        state.gates.eval = Gate::fail(Evidence::EvalScore);

        let (state, tlog) = run_until_done(state, cfg).unwrap();

        assert_eq!(state.phase, Phase::Done);
        assert_eq!(state.failure, None);
        assert_eq!(state.gates.eval.status, GateStatus::Pass);

        assert!(tlog.iter().any(|e| {
            e.recovery_action == Some(RecoveryAction::RecomputeEval)
                && e.affected_gate == Some(GateId::Eval)
                && e.evidence == Evidence::EvalScore
        }));

        verify_tlog(&tlog).unwrap();
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

        assert_eq!(state.phase, Phase::Done);
        assert_eq!(state.failure, None);
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
    fn tampered_tlog_fails_verification() {
        let (_, mut tlog) = run_until_done(State::ready(), RuntimeConfig::default()).unwrap();
        tlog[0].self_hash = 123;

        assert_eq!(verify_tlog(&tlog), Err(CanonError::InvalidHashChain));
    }

    #[test]
    fn broken_state_continuity_fails_verification() {
        let (_, mut tlog) = run_until_done(State::ready(), RuntimeConfig::default()).unwrap();
        tlog[1].from = Phase::Done;

        assert_eq!(verify_tlog(&tlog), Err(CanonError::InvalidStateContinuity));
    }
}
