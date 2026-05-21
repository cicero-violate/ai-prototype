//! Disposable runtime event-bus projection.
//!
//! The bus is deliberately not authority. It is a wakeup cache derived only from
//! canonical TLog appends. Dropping it loses no truth: callers can replay the
//! TLog to reconstruct all wakeups that should have been observed.

use std::collections::HashSet;

use crate::kernel::{Cause, ControlEvent, Decision, EventKind, Evidence, FailureClass, GateId, TLog};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WakeupKind {
    TaskReady,
    LeaseExpired,
    ReceiptAccepted,
    GateFailed,
    EvalVerdict,
    LearningCandidate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RuntimeWakeup {
    pub event_seq: u64,
    pub kind: WakeupKind,
    pub idempotency_key: RuntimeWakeupKey,
    pub gate: Option<GateId>,
    pub evidence: Evidence,
    pub passed: Option<bool>,
    pub failure: Option<FailureClass>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RuntimeWakeupKey {
    kind: WakeupKind,
    event_seq: u64,
    gate: Option<GateId>,
    evidence: Evidence,
}

impl RuntimeWakeupKey {
    pub fn new(kind: WakeupKind, event: &ControlEvent) -> Self {
        Self {
            kind,
            event_seq: event.seq,
            gate: event.affected_gate,
            evidence: event.evidence,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeEventBus {
    wakeups: Vec<RuntimeWakeup>,
    seen: HashSet<RuntimeWakeupKey>,
}

impl RuntimeEventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replay(tlog: &TLog) -> Self {
        let mut bus = Self::new();
        for event in ordered_events(tlog) {
            bus.project_append(event);
        }
        bus
    }

    pub fn project_append(&mut self, event: &ControlEvent) -> Option<RuntimeWakeup> {
        let wakeup = wakeup_from_event(event)?;
        if !self.seen.insert(wakeup.idempotency_key) {
            return None;
        }

        let insert_at = self
            .wakeups
            .binary_search_by_key(&wakeup.event_seq, |existing| existing.event_seq)
            .unwrap_or_else(|idx| idx);
        self.wakeups.insert(insert_at, wakeup);
        Some(wakeup)
    }

    pub fn wakeups(&self) -> &[RuntimeWakeup] {
        &self.wakeups
    }

    pub fn drain(&mut self) -> Vec<RuntimeWakeup> {
        std::mem::take(&mut self.wakeups)
    }

    pub fn clear_disposable_projection(&mut self) {
        self.wakeups.clear();
        self.seen.clear();
    }
}

pub fn replay_event_bus(tlog: &TLog) -> RuntimeEventBus {
    RuntimeEventBus::replay(tlog)
}

fn ordered_events(tlog: &TLog) -> Vec<&ControlEvent> {
    let mut events = tlog.iter().collect::<Vec<_>>();
    events.sort_by_key(|event| event.seq);
    events
}

fn wakeup_from_event(event: &ControlEvent) -> Option<RuntimeWakeup> {
    let kind = wakeup_kind(event)?;
    let passed = match kind {
        WakeupKind::EvalVerdict => Some(event.decision != Decision::Fail),
        WakeupKind::GateFailed => Some(false),
        WakeupKind::ReceiptAccepted => Some(true),
        WakeupKind::TaskReady | WakeupKind::LeaseExpired | WakeupKind::LearningCandidate => None,
    };
    Some(RuntimeWakeup {
        event_seq: event.seq,
        kind,
        idempotency_key: RuntimeWakeupKey::new(kind, event),
        gate: event.affected_gate,
        evidence: event.evidence,
        passed,
        failure: event.failure,
    })
}

fn wakeup_kind(event: &ControlEvent) -> Option<WakeupKind> {
    if is_task_ready(event) {
        return Some(WakeupKind::TaskReady);
    }
    if is_lease_expired(event) {
        return Some(WakeupKind::LeaseExpired);
    }
    if is_receipt_accepted(event) {
        return Some(WakeupKind::ReceiptAccepted);
    }
    if is_gate_failed(event) {
        return Some(WakeupKind::GateFailed);
    }
    if is_eval_verdict(event) {
        return Some(WakeupKind::EvalVerdict);
    }
    if is_learning_candidate(event) {
        return Some(WakeupKind::LearningCandidate);
    }
    None
}

fn is_task_ready(event: &ControlEvent) -> bool {
    event.evidence == Evidence::TaskReady || event.cause == Cause::PlanReady
}

fn is_lease_expired(event: &ControlEvent) -> bool {
    event.cause == Cause::TaskReceiptMissing
        || event.failure == Some(FailureClass::TaskReceiptMissing)
}

fn is_receipt_accepted(event: &ControlEvent) -> bool {
    matches!(
        event.evidence,
        Evidence::ExecutionReceipt
            | Evidence::ArtifactReceipt
            | Evidence::VerificationReport
            | Evidence::LineageProof
            | Evidence::PersistedRecord
    ) && matches!(event.kind, EventKind::Advanced | EventKind::Completed | EventKind::Persisted)
        && matches!(event.decision, Decision::Continue | Decision::Complete)
}

fn is_gate_failed(event: &ControlEvent) -> bool {
    event.cause == Cause::GateFailed
        || matches!(event.kind, EventKind::Blocked | EventKind::Failed)
        || matches!(event.decision, Decision::Block | Decision::Fail)
}

fn is_eval_verdict(event: &ControlEvent) -> bool {
    event.evidence == Evidence::EvalScore
        || matches!(event.cause, Cause::EvalPassed | Cause::EvalFailed)
}

fn is_learning_candidate(event: &ControlEvent) -> bool {
    matches!(event.kind, EventKind::Learned)
        || matches!(event.cause, Cause::PolicyPromoted)
        || matches!(event.evidence, Evidence::LearningRecord | Evidence::PolicyPromotion)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::{
        CapabilityRegistryProjection, GateSet, Packet, Phase, RecoveryAction, RuntimeConfig,
        SemanticDelta, State,
    };

    #[test]
    fn replay_reconstructs_typed_wakeups_from_tlog() {
        let tlog = vec![
            event(1, EventKind::Advanced, Cause::PlanReady, Evidence::TaskReady),
            event(
                2,
                EventKind::Failed,
                Cause::TaskReceiptMissing,
                Evidence::Missing,
            ),
            event(
                3,
                EventKind::Advanced,
                Cause::ExecutionFinished,
                Evidence::ExecutionReceipt,
            ),
            event(4, EventKind::Blocked, Cause::GateFailed, Evidence::Missing),
            event(5, EventKind::Advanced, Cause::EvalPassed, Evidence::EvalScore),
            event(
                6,
                EventKind::Learned,
                Cause::PolicyPromoted,
                Evidence::PolicyPromotion,
            ),
        ];

        let bus = RuntimeEventBus::replay(&tlog);
        let kinds = bus
            .wakeups()
            .iter()
            .map(|wakeup| wakeup.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            kinds,
            vec![
                WakeupKind::TaskReady,
                WakeupKind::LeaseExpired,
                WakeupKind::ReceiptAccepted,
                WakeupKind::GateFailed,
                WakeupKind::EvalVerdict,
                WakeupKind::LearningCandidate,
            ]
        );
        assert_eq!(bus.wakeups()[4].passed, Some(true));
    }

    #[test]
    fn project_append_is_idempotent_for_same_tlog_event() {
        let mut bus = RuntimeEventBus::new();
        let ready = event(7, EventKind::Advanced, Cause::PlanReady, Evidence::TaskReady);

        assert_eq!(bus.project_append(&ready).map(|w| w.kind), Some(WakeupKind::TaskReady));
        assert_eq!(bus.project_append(&ready), None);
        assert_eq!(bus.project_append(&ready), None);

        assert_eq!(bus.wakeups().len(), 1);
        assert_eq!(bus.wakeups()[0].event_seq, 7);
    }

    #[test]
    fn replay_reconstructs_missed_wakeups_after_projection_is_discarded() {
        let tlog = vec![
            event(11, EventKind::Advanced, Cause::PlanReady, Evidence::TaskReady),
            event(12, EventKind::Blocked, Cause::GateFailed, Evidence::Missing),
        ];
        let mut disposable = RuntimeEventBus::replay(&tlog);
        let before_drop = disposable.wakeups().to_vec();

        disposable.clear_disposable_projection();
        assert!(disposable.wakeups().is_empty());

        let replayed = RuntimeEventBus::replay(&tlog);
        assert_eq!(replayed.wakeups(), before_drop.as_slice());
        assert_eq!(
            replayed
                .wakeups()
                .iter()
                .map(|wakeup| wakeup.kind)
                .collect::<Vec<_>>(),
            vec![WakeupKind::TaskReady, WakeupKind::GateFailed]
        );
    }

    #[test]
    fn wakeups_are_ordered_by_event_seq_even_when_appends_arrive_out_of_order() {
        let mut bus = RuntimeEventBus::new();
        let eval = event(30, EventKind::Advanced, Cause::EvalPassed, Evidence::EvalScore);
        let ready = event(10, EventKind::Advanced, Cause::PlanReady, Evidence::TaskReady);
        let receipt = event(
            20,
            EventKind::Advanced,
            Cause::ExecutionFinished,
            Evidence::ExecutionReceipt,
        );

        bus.project_append(&eval);
        bus.project_append(&ready);
        bus.project_append(&receipt);

        let seqs = bus.wakeups().iter().map(|wakeup| wakeup.event_seq).collect::<Vec<_>>();
        assert_eq!(seqs, vec![10, 20, 30]);
    }

    fn event(seq: u64, kind: EventKind, cause: Cause, evidence: Evidence) -> ControlEvent {
        let decision = match kind {
            EventKind::Blocked => Decision::Block,
            EventKind::Failed => Decision::Fail,
            EventKind::Completed => Decision::Complete,
            _ => Decision::Continue,
        };
        let failure = match cause {
            Cause::TaskReceiptMissing => Some(FailureClass::TaskReceiptMissing),
            Cause::EvalFailed => Some(FailureClass::EvalFailed),
            Cause::GateFailed => Some(FailureClass::ExecutionFailed),
            _ => None,
        };
        let recovery_action = failure.map(|failure| match failure {
            FailureClass::TaskReceiptMissing => RecoveryAction::Reexecute,
            FailureClass::EvalFailed => RecoveryAction::RecomputeEval,
            _ => RecoveryAction::Escalate,
        });
        let state = State {
            phase: Phase::Execute,
            gates: GateSet::default(),
            packet: Packet::empty(),
            failure,
            recovery_action,
            recovery_attempts: 0,
            wave_pending: 0,
            plan_state_hash: 0,
        };
        ControlEvent {
            seq,
            from: Phase::Plan,
            to: Phase::Execute,
            kind,
            cause,
            delta: SemanticDelta::NoChange,
            evidence,
            decision,
            failure,
            recovery_action,
            affected_gate: Some(GateId::Execution),
            runtime_config: RuntimeConfig::default(),
            state_before: state,
            state_after: state,
            capability_registry_projection: CapabilityRegistryProjection::default(),
            api_command_id: 0,
            api_command_hash: 0,
            prev_hash: seq.saturating_sub(1),
            self_hash: seq.max(1),
        }
    }
}