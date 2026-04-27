//! Deterministic runtime reducer.

use crate::kernel::{
    Cause, Decision, EventKind, Evidence, FailureClass, Gate, GateId, GateStatus, Phase,
    RecoveryAction, RuntimeConfig, State,
};

use super::recovery_policy::{
    decision_for_failure, event_kind_for_failure, failure_for_gate, recovery_action_for,
};
use super::Outcome;

pub(crate) fn reduce(input: State, cfg: RuntimeConfig) -> Outcome {
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
        Phase::Persist => persist(input),
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
        GateStatus::Pass if s.packet.artifact_receipt_valid() => {
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

    if let Some((gate_id, gate)) = s.gates.first_execution_non_pass() {
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

    advance(&mut s, Phase::Persist, Cause::EvalPassed, Evidence::EvalScore)
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

    s.phase = Phase::Persist;
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

fn persist(input: State) -> Outcome {
    let mut s = input;

    let Some(action) = input.recovery_action else {
        s.phase = Phase::Learn;

        return Outcome {
            state: s,
            kind: EventKind::Persisted,
            cause: Cause::Persisted,
            evidence: Evidence::PersistedRecord,
            decision: Decision::Continue,
            failure: None,
            recovery_action: None,
            affected_gate: None,
        };
    };

    if action == RecoveryAction::Escalate {
        s.phase = Phase::Done;
        s.failure = Some(input.failure.unwrap_or(FailureClass::RecoveryExhausted));

        return Outcome {
            state: s,
            kind: EventKind::Persisted,
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
        kind: EventKind::Persisted,
        cause: Cause::RepairApplied,
        evidence,
        decision: Decision::Continue,
        failure: input.failure,
        recovery_action: Some(action),
        affected_gate: Some(gate),
    }
}

fn learn(input: State) -> Outcome {
    let mut s = input;

    s.gates.set_pass(GateId::Learning, Evidence::PolicyPromotion);
    s.phase = Phase::Done;
    s.failure = None;
    s.recovery_action = None;

    Outcome {
        state: s,
        kind: EventKind::Learned,
        cause: Cause::PolicyPromoted,
        evidence: Evidence::PolicyPromotion,
        decision: Decision::Complete,
        failure: None,
        recovery_action: None,
        affected_gate: Some(GateId::Learning),
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
