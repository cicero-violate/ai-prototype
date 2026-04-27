#![forbid(unsafe_code)]
//! Canonical atomic state-machine runtime.
//!
//! Layering rule: kernel stays frozen; capabilities own rich records; runtime
//! turns evidence into control events; codec only serializes/deserializes; API is
//! the outer surface.

pub mod api;
pub mod capability;
pub mod codec;
pub mod kernel;
pub mod runtime;

pub use crate::capability::eval::{EvalDecision, EvalDimension, EvalRecord};
pub use crate::capability::learning::PolicyPromotion;
pub use crate::capability::policy::{PolicyEntry, PolicyStore, PolicyStoreError};
pub use crate::codec::ndjson::{append_tlog_ndjson, load_tlog_ndjson, write_tlog_ndjson};
pub use crate::kernel::{
    Cause, ControlEvent, Decision, EventKind, Evidence, FailureClass, Gate, GateId, GateSet,
    GateStatus, Packet, Phase, RecoveryAction, RuntimeConfig, SemanticDelta, State, TLog,
    GATE_ORDER, PHASES,
};
pub use crate::runtime::{
    legal_transition, replay_tlog_ndjson, run_until_done, run_until_done_durable, semantic_diff,
    tick, tick_durable, touch_all_surfaces, verify_tlog, verify_tlog_from, CanonError,
};

#[cfg(test)]
pub(crate) use crate::runtime::verify::{hash_event, validate_event, EventHashInput, EventView};

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
        assert_eq!(tlog.last().unwrap().kind, EventKind::Learned);
        assert_eq!(tlog.last().unwrap().affected_gate, Some(GateId::Learning));
        assert!(tlog.iter().any(|e| e.kind == EventKind::Persisted));
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn recovery_policy_covers_every_failure_class() {
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
            FailureClass::LearningMissing,
            FailureClass::LearningFailed,
        ];

        assert_eq!(
            crate::runtime::recovery_policy::recovery_policy_coverage_count(),
            failures.len()
        );

        for failure in failures {
            let action = crate::runtime::recovery_policy::recovery_action_for(failure);
            assert_eq!(
                action == RecoveryAction::Escalate,
                matches!(
                    failure,
                    FailureClass::RecoveryExhausted | FailureClass::ConvergenceFailed
                )
            );
        }
    }

    #[test]
    fn learning_materializes_policy_entry_from_tlog_history() {
        let (_state, tlog) = run_until_done(State::ready(), RuntimeConfig::default()).unwrap();

        let promotion = PolicyPromotion::from_tlog(&tlog, 1).unwrap();
        assert!(promotion.is_valid());

        let mut store = PolicyStore::default();
        let entry = store.promote(promotion.clone()).unwrap().clone();

        assert_eq!(entry.version, 1);
        assert_eq!(entry.value, promotion.source_seq);
        assert_eq!(store.latest_version(), 1);
        assert_eq!(store.latest(entry.key), Some(&entry));
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
        assert!(tlog
            .iter()
            .filter(|e| e.kind == EventKind::Learned)
            .all(|e| e.recovery_action.is_none() && e.affected_gate == Some(GateId::Learning)));
        assert!(tlog.iter().any(|e| {
            e.kind == EventKind::Persisted
                && e.recovery_action == Some(RecoveryAction::BindReadyTask)
                && e.affected_gate == Some(GateId::Plan)
                && e.evidence == Evidence::TaskReady
        }));
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
        assert!(legal_transition(
            Phase::Recovery,
            Phase::Persist,
            EventKind::Recovered,
            Cause::RepairSelected
        ));
        assert!(legal_transition(
            Phase::Learn,
            Phase::Done,
            EventKind::Learned,
            Cause::PolicyPromoted
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
    fn hash_consistent_but_non_reducer_event_fails_replay() {
        let initial = State::ready();
        let cfg = RuntimeConfig::default();
        let mut state = initial;
        let mut tlog = Vec::new();
        tick(&mut state, &mut tlog, cfg).unwrap();

        let mut event = tlog[0];
        event.state_after.packet.revision = event.state_after.packet.revision.saturating_add(1);
        event.delta = semantic_diff(event.state_before, event.state_after);
        event.self_hash = hash_event(EventHashInput {
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
            runtime_config: event.runtime_config,
            state_before: event.state_before,
            state_after: event.state_after,
        });

        assert_eq!(verify_tlog_from(initial, &[event]), Err(CanonError::InvalidReplay));
    }

    #[test]
    fn durable_tick_does_not_mutate_memory_when_disk_append_fails() {
        let cfg = RuntimeConfig::default();
        let mut state = State::ready();
        let before = state;
        let mut tlog = Vec::new();
        let bad_path = std::env::temp_dir();

        assert_eq!(
            tick_durable(&mut state, &mut tlog, bad_path, cfg),
            Err(CanonError::TlogIo)
        );
        assert_eq!(state, before);
        assert!(tlog.is_empty());
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
