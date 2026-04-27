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

pub use crate::api::protocol::{
    Command, CommandEnvelope, CommandLedger, CommandReceipt, ControlEventResponse,
    API_PROTOCOL_SCHEMA_VERSION,
};
pub use crate::capability::context::{ContextDecision, ContextRecord};
pub use crate::capability::eval::{EvalDecision, EvalDimension, EvalRecord};
pub use crate::capability::learning::{
    PolicyPromotion, POLICY_FEEDBACK_HASH, POLICY_PROMOTION_SOURCE_SEQ,
};
pub use crate::capability::llm::{
    LlmDecision, LlmPromptRecord, LlmRecord, LlmResponseRecord, LlmStructuredAdapter,
};
pub use crate::capability::memory::{MemoryFact, MemoryIndex, MemoryLookupRecord};
pub use crate::capability::observation::{ObservationDecision, ObservationRecord};
pub use crate::capability::orchestration::{
    CapabilityRoute, OrchestrationDecision, OrchestrationRecord,
};
pub use crate::capability::planning::{PlanDecision, PlanRecord};
pub use crate::capability::policy::{PolicyEntry, PolicyStore, PolicyStoreError};
pub use crate::capability::tooling::{
    DeterministicToolExecutor, ToolDecision, ToolExecutionRecord, ToolKind, ToolReceipt,
    ToolRequest,
};
pub use crate::capability::verification::{
    ArtifactSemanticProfile, DeterministicSemanticVerifier, SemanticVerificationReceipt,
    VerificationCheck, VerificationDecision, VerificationRecord, VerificationRequest,
};
pub use crate::capability::{
    expected_evidence_for_gate, EvidenceSubmission, PacketEffect,
};
pub use crate::codec::ndjson::{
    append_tlog_ndjson, decode_control_event_ndjson, decode_tlog_ndjson_str,
    encode_control_event_ndjson, encode_tlog_ndjson_string, load_tlog_ndjson, write_tlog_ndjson,
    TLOG_RECORD_EVENT, TLOG_SCHEMA_VERSION,
};
pub use crate::kernel::{
    Cause, ControlEvent, Decision, EventKind, Evidence, FailureClass, Gate, GateId, GateSet,
    GateStatus, Packet, Phase, RecoveryAction, RuntimeConfig, SemanticDelta, State, TLog,
    EXECUTION_GATE_ORDER, GATE_ORDER, PHASES,
};
pub use crate::runtime::{
    durable_replay_report, legal_transition, replay_report_from, replay_report_ndjson,
    replay_tlog_ndjson, resume_durable_runtime, run_until_done,
    run_until_done_durable, run_until_done_durable_with_ledger, semantic_diff, tick,
    tick_durable, tick_durable_checked, touch_all_surfaces, verify_tlog, verify_tlog_from,
    CanonError, DurableRuntimeState, ReplayReport,
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
        assert_eq!(promotion.submission().gate, GateId::Learning);
        assert!(promotion.submission().passed);

        let mut store = PolicyStore::default();
        let entry = store.promote(promotion.clone()).unwrap().clone();

        assert_eq!(entry.version, 1);
        assert_eq!(entry.value, promotion.source_seq);
        assert_eq!(store.latest_version(), 1);
        assert_eq!(store.latest(entry.key), Some(&entry));
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn durable_policy_store_roundtrips_promoted_policy() {
        let (_state, tlog) = run_until_done(State::ready(), RuntimeConfig::default()).unwrap();
        let promotion = PolicyPromotion::from_tlog(&tlog, 1).unwrap();
        let path = std::env::temp_dir().join(format!(
            "ai-policy-store-{}-{}.ndjson",
            std::process::id(),
            promotion.source_seq
        ));
        std::fs::remove_file(&path).ok();

        let mut store = PolicyStore::default();
        let entry = store.promote_durable(&path, promotion).unwrap().clone();
        let loaded = PolicyStore::load_ndjson(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(loaded.entries(), &[entry]);
        assert_eq!(loaded.latest_version(), 1);
    }

    #[test]
    fn durable_policy_append_is_disk_first() {
        let mut store = PolicyStore::default();
        let bad_path = std::env::temp_dir();

        assert_eq!(
            store.append_durable(
                bad_path,
                PolicyEntry {
                    version: 1,
                    key: crate::capability::learning::POLICY_PROMOTION_SOURCE_SEQ,
                    value: 7,
                }
            ),
            Err(PolicyStoreError::PolicyIo)
        );
        assert!(store.entries().is_empty());
    }

    #[test]
    fn eval_record_submission_drives_eval_gate_through_api() {
        let mut state = State::ready();
        state.phase = Phase::Eval;
        state.gates.eval = Gate::unknown();

        let mut tlog = Vec::new();
        let record = EvalRecord {
            score: 91,
            threshold_used: 80,
            dimensions: vec![EvalDimension {
                id: "lineage",
                score: 91,
                threshold: 80,
            }],
        };

        let response = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            crate::api::protocol::Command::SubmitEvidence(record.submission()),
        )
        .unwrap();

        assert_eq!(response.event.from, Phase::Eval);
        assert_eq!(response.event.to, Phase::Persist);
        assert_eq!(response.event.evidence, Evidence::EvalScore);
        assert_eq!(state.phase, Phase::Persist);
        assert_eq!(state.gates.eval.status, GateStatus::Pass);
        assert_eq!(verify_tlog_from(tlog[0].state_before, &tlog).unwrap(), state);
    }

    #[test]
    fn empty_eval_record_fails_submission() {
        let record = EvalRecord {
            score: 100,
            threshold_used: 80,
            dimensions: Vec::new(),
        };

        let submission = record.submission();

        assert_eq!(record.decision(), EvalDecision::Fail);
        assert_eq!(submission.gate, GateId::Eval);
        assert!(!submission.passed);
    }

    #[test]
    fn judgment_record_submission_applies_to_state() {
        let mut state = State::default();
        let judgment = crate::capability::judgment::JudgmentRecord {
            decision_id: 1,
            policy_version: 1,
            rationale_hash: 42,
        };

        judgment.submission().apply_to(&mut state);

        assert_eq!(state.gates.judgment.status, GateStatus::Pass);
        assert_eq!(state.gates.judgment.evidence, Evidence::JudgmentRecord);
    }

    #[test]
    fn observation_record_submission_drives_invariant_gate_through_api() {
        let mut state = State::default();
        state.phase = Phase::Invariant;

        let mut tlog = Vec::new();
        let record = ObservationRecord::new(7, 1, 0xabc, 1);
        let response = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            crate::api::protocol::Command::SubmitEvidence(record.submission()),
        )
        .unwrap();

        assert_eq!(tlog[0].from, Phase::Invariant);
        assert_eq!(tlog[0].to, Phase::Invariant);
        assert_eq!(tlog[0].cause, Cause::EvidenceSubmitted);
        assert_eq!(tlog[0].evidence, Evidence::InvariantProof);
        assert_eq!(response.event.from, Phase::Invariant);
        assert_eq!(response.event.to, Phase::Analysis);
        assert_eq!(state.gates.invariant.status, GateStatus::Pass);
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn invalid_observation_fails_submission_without_advancing_gate() {
        let mut state = State::default();
        state.phase = Phase::Invariant;

        let mut tlog = Vec::new();
        let record = ObservationRecord::new(7, 1, 0, 1);
        let response = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            crate::api::protocol::Command::SubmitEvidence(record.submission()),
        )
        .unwrap();

        assert_eq!(record.decision(), ObservationDecision::Rejected);
        assert_eq!(tlog[0].cause, Cause::EvidenceSubmitted);
        assert_eq!(tlog[0].decision, Decision::Block);
        assert_eq!(state.phase, Phase::Recovery);
        assert_eq!(state.gates.invariant.status, GateStatus::Fail);
        assert_eq!(response.event.failure, Some(FailureClass::InvariantBlocked));
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn memory_index_lookup_is_deterministic_and_weight_ordered() {
        let mut memory = MemoryIndex::default();

        assert!(memory.insert(MemoryFact::new(42, 900, 4, 3)));
        assert!(memory.insert(MemoryFact::new(42, 700, 9, 2)));
        assert!(memory.insert(MemoryFact::new(42, 800, 9, 1)));
        assert!(memory.insert(MemoryFact::new(7, 100, 5, 1)));
        assert!(!memory.insert(MemoryFact::new(42, 0, 9, 4)));

        let lookup = memory.lookup(42, 2);
        let lookup_again = memory.lookup(42, 2);

        assert!(lookup.is_valid());
        assert_eq!(lookup, lookup_again);
        assert_eq!(lookup.match_count(), 2);
        assert_eq!(lookup.matches[0], MemoryFact::new(42, 800, 9, 1));
        assert_eq!(lookup.matches[1], MemoryFact::new(42, 700, 9, 2));
    }

    #[test]
    fn context_record_submission_drives_analysis_gate_through_api() {
        let mut state = State::default();
        state.phase = Phase::Analysis;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let record = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);

        let mut tlog = Vec::new();
        let response = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            crate::api::protocol::Command::SubmitEvidence(record.submission()),
        )
        .unwrap();

        assert_eq!(record.decision(), ContextDecision::Assembled);
        assert_eq!(response.event.from, Phase::Analysis);
        assert_eq!(response.event.to, Phase::Judgment);
        assert_eq!(response.event.evidence, Evidence::AnalysisReport);
        assert_eq!(state.gates.analysis.status, GateStatus::Pass);
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn observation_memory_context_drive_start_to_judgment() {
        let mut state = State::default();
        let cfg = RuntimeConfig::default();
        let mut tlog = Vec::new();

        tick(&mut state, &mut tlog, cfg).unwrap();
        assert_eq!(state.phase, Phase::Invariant);

        let observation = ObservationRecord::new(7, 1, 0xabc, 1);
        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            cfg,
            crate::api::protocol::Command::SubmitEvidence(observation.submission()),
        )
        .unwrap();
        assert_eq!(state.phase, Phase::Analysis);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, observation.observed_hash, &lookup);

        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            cfg,
            crate::api::protocol::Command::SubmitEvidence(context.submission()),
        )
        .unwrap();

        assert_eq!(state.phase, Phase::Judgment);
        assert_eq!(state.gates.invariant.status, GateStatus::Pass);
        assert_eq!(state.gates.analysis.status, GateStatus::Pass);
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn llm_adapter_turns_context_into_judgment_record() {
        let packet = Packet::empty();
        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(packet, 0xabc, &lookup);
        let policy = PolicyStore::default();

        let llm = LlmStructuredAdapter::record_from_context(&context, &policy, 11);
        let judgment = llm.judgment_record();
        let submission = llm.submission();

        assert_eq!(llm.decision(), LlmDecision::Structured);
        assert!(llm.prompt.is_valid());
        assert!(llm.response.is_valid());
        assert!(judgment.is_valid());
        assert_eq!(submission.gate, GateId::Judgment);
        assert_eq!(submission.evidence, Evidence::JudgmentRecord);
        assert!(submission.passed);
    }

    #[test]
    fn policy_feedback_entry_roundtrips_and_fingerprints() {
        let (_state, tlog) = run_until_done(State::ready(), RuntimeConfig::default()).unwrap();
        let promotion = PolicyPromotion::from_tlog(&tlog, 1).unwrap();
        assert!(promotion.is_valid());
        assert_eq!(promotion.eval_seq, promotion.source_seq);
        assert!(promotion.judgment_seq < promotion.eval_seq);
        assert!(promotion.completion_seq >= promotion.eval_seq);
        assert_ne!(promotion.promoted_policy_hash, 0);

        let path = std::env::temp_dir().join(format!(
            "ai-policy-feedback-{}-{}.ndjson",
            std::process::id(),
            promotion.promoted_policy_hash
        ));
        std::fs::remove_file(&path).ok();

        let mut store = PolicyStore::default();
        let empty_fingerprint = store.fingerprint();
        let entry = store
            .promote_feedback_durable(&path, promotion.clone())
            .unwrap()
            .clone();
        let loaded = PolicyStore::load_ndjson(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(entry.key, POLICY_FEEDBACK_HASH);
        assert_eq!(entry.value, promotion.promoted_policy_hash);
        assert_eq!(store.feedback_hash(), promotion.promoted_policy_hash);
        assert_ne!(store.fingerprint(), empty_fingerprint);
        assert_eq!(loaded.entries(), &[entry]);
        assert_eq!(loaded.feedback_hash(), promotion.promoted_policy_hash);
    }

    #[test]
    fn policy_feedback_changes_llm_prompt_and_judgment() {
        let packet = Packet::empty();
        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(packet, 0xabc, &lookup);

        let base_policy = PolicyStore::default();
        let base_llm = LlmStructuredAdapter::record_from_context(&context, &base_policy, 11);

        let (_state, tlog) = run_until_done(State::ready(), RuntimeConfig::default()).unwrap();
        let promotion = PolicyPromotion::from_tlog(&tlog, 1).unwrap();
        let mut feedback_policy = PolicyStore::default();
        feedback_policy.promote_feedback(promotion).unwrap();
        let feedback_llm = LlmStructuredAdapter::record_from_context(&context, &feedback_policy, 11);

        assert_eq!(base_llm.prompt.policy_version, feedback_llm.prompt.policy_version);
        assert_ne!(base_llm.prompt.policy_hash, feedback_llm.prompt.policy_hash);
        assert_ne!(base_llm.prompt.prompt_hash, feedback_llm.prompt.prompt_hash);
        assert_ne!(base_llm.response.response_hash, feedback_llm.response.response_hash);
        assert_ne!(base_llm.judgment_record(), feedback_llm.judgment_record());
    }

    #[test]
    fn learning_policy_llm_feedback_loop_drives_judgment() {
        let (_learned_state, learned_tlog) =
            run_until_done(State::ready(), RuntimeConfig::default()).unwrap();
        let promotion = PolicyPromotion::from_tlog(&learned_tlog, 1).unwrap();
        let mut policy = PolicyStore::default();
        policy.promote_feedback(promotion).unwrap();

        let mut state = State::default();
        state.phase = Phase::Judgment;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);
        let llm = LlmStructuredAdapter::record_from_context(&context, &policy, 11);

        assert_eq!(llm.prompt.policy_hash, policy.fingerprint());
        assert_eq!(llm.prompt.policy_version, policy.latest_version());

        let mut tlog = Vec::new();
        let response = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            crate::api::protocol::Command::SubmitEvidence(llm.submission()),
        )
        .unwrap();

        assert_eq!(response.event.from, Phase::Judgment);
        assert_eq!(response.event.to, Phase::Plan);
        assert_eq!(response.event.evidence, Evidence::JudgmentRecord);
        assert_eq!(state.gates.judgment.status, GateStatus::Pass);
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn llm_record_submission_drives_judgment_gate_through_api() {
        let mut state = State::default();
        state.phase = Phase::Judgment;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);
        let policy = PolicyStore::default();
        let llm = LlmStructuredAdapter::record_from_context(&context, &policy, 11);

        let mut tlog = Vec::new();
        let response = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            crate::api::protocol::Command::SubmitEvidence(llm.submission()),
        )
        .unwrap();

        assert_eq!(response.event.from, Phase::Judgment);
        assert_eq!(response.event.to, Phase::Plan);
        assert_eq!(response.event.evidence, Evidence::JudgmentRecord);
        assert_eq!(state.gates.judgment.status, GateStatus::Pass);
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn invalid_llm_record_fails_without_advancing_judgment() {
        let mut state = State::default();
        state.phase = Phase::Judgment;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);

        let lookup = MemoryIndex::default().lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);
        let policy = PolicyStore::default();
        let llm = LlmStructuredAdapter::record_from_context(&context, &policy, 0);

        let mut tlog = Vec::new();
        let response = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            crate::api::protocol::Command::SubmitEvidence(llm.submission()),
        )
        .unwrap();

        assert_eq!(llm.decision(), LlmDecision::Refused);
        assert_eq!(tlog[0].decision, Decision::Block);
        assert_eq!(state.phase, Phase::Recovery);
        assert_eq!(state.gates.judgment.status, GateStatus::Fail);
        assert_eq!(response.event.failure, Some(FailureClass::JudgmentFailed));
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn observation_context_llm_drive_start_to_plan() {
        let mut state = State::default();
        let cfg = RuntimeConfig::default();
        let mut tlog = Vec::new();

        tick(&mut state, &mut tlog, cfg).unwrap();
        assert_eq!(state.phase, Phase::Invariant);

        let observation = ObservationRecord::new(7, 1, 0xabc, 1);
        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            cfg,
            crate::api::protocol::Command::SubmitEvidence(observation.submission()),
        )
        .unwrap();
        assert_eq!(state.phase, Phase::Analysis);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, observation.observed_hash, &lookup);
        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            cfg,
            crate::api::protocol::Command::SubmitEvidence(context.submission()),
        )
        .unwrap();
        assert_eq!(state.phase, Phase::Judgment);

        let policy = PolicyStore::default();
        let llm = LlmStructuredAdapter::record_from_context(&context, &policy, 11);
        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            cfg,
            crate::api::protocol::Command::SubmitEvidence(llm.submission()),
        )
        .unwrap();

        assert_eq!(state.phase, Phase::Plan);
        assert_eq!(state.gates.invariant.status, GateStatus::Pass);
        assert_eq!(state.gates.analysis.status, GateStatus::Pass);
        assert_eq!(state.gates.judgment.status, GateStatus::Pass);
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn orchestration_record_orders_ready_submissions() {
        let mut state = State::default();
        state.phase = Phase::Invariant;

        let record = OrchestrationRecord::from_state(state, 9);
        assert_eq!(record.decision(), OrchestrationDecision::Routed);
        assert!(record.is_valid());
        assert_eq!(record.routes.len(), EXECUTION_GATE_ORDER.len());

        let submissions = record.ordered_submissions();
        assert_eq!(submissions.len(), EXECUTION_GATE_ORDER.len());
        assert_eq!(submissions[0].gate, GateId::Invariant);
        assert_eq!(submissions[1].gate, GateId::Analysis);
        assert_eq!(submissions[2].gate, GateId::Judgment);
        assert_eq!(submissions[3].gate, GateId::Plan);
        assert_eq!(submissions[4].gate, GateId::Execution);
        assert_eq!(submissions[5].gate, GateId::Verification);
        assert_eq!(submissions[6].gate, GateId::Eval);
    }

    #[test]
    fn orchestration_batch_drives_execution_path_to_persist() {
        let mut state = State::default();
        let mut tlog = Vec::new();
        let cfg = RuntimeConfig::default();
        tick(&mut state, &mut tlog, cfg).unwrap();
        assert_eq!(state.phase, Phase::Invariant);

        let record = OrchestrationRecord::from_state(state, 7);
        let response = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            cfg,
            crate::api::protocol::Command::SubmitEvidenceBatch(record.ordered_submissions()),
        )
        .unwrap();

        assert_eq!(response.event.from, Phase::Eval);
        assert_eq!(response.event.to, Phase::Persist);
        assert_eq!(response.event.evidence, Evidence::EvalScore);
        assert_eq!(state.phase, Phase::Persist);
        assert!(state.gates.all_execution_passed());
        assert!(state.packet.objective_complete());
        assert!(state.packet.lineage_valid());
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn orchestration_skips_already_passed_gates() {
        let mut state = State::default();
        state.phase = Phase::Plan;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
        state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);

        let record = OrchestrationRecord::from_state(state, 3);
        let submissions = record.ordered_submissions();

        assert_eq!(submissions.len(), 4);
        assert_eq!(submissions[0].gate, GateId::Plan);
        assert_eq!(submissions[1].gate, GateId::Execution);
        assert_eq!(submissions[2].gate, GateId::Verification);
        assert_eq!(submissions[3].gate, GateId::Eval);
    }

    #[test]
    fn empty_orchestration_batch_is_rejected() {
        let mut state = State::default();
        state.phase = Phase::Invariant;
        let mut tlog = Vec::new();

        let result = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            crate::api::protocol::Command::SubmitEvidenceBatch(Vec::new()),
        );

        assert_eq!(result, Err(CanonError::InvalidApiCommand));
    }

    #[test]
    fn api_rejects_mismatched_evidence_without_mutation() {
        let mut state = State::default();
        let mut tlog = Vec::new();
        let cfg = RuntimeConfig::default();
        tick(&mut state, &mut tlog, cfg).unwrap();
        let before = state;
        let before_len = tlog.len();

        let result = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            cfg,
            Command::SubmitEvidence(EvidenceSubmission::new(
                GateId::Invariant,
                Evidence::EvalScore,
                true,
            )),
        );

        assert_eq!(result, Err(CanonError::InvalidApiCommand));
        assert_eq!(state, before);
        assert_eq!(tlog.len(), before_len);
    }

    #[test]
    fn api_rejects_invalid_batch_atomically() {
        let mut state = State::default();
        state.phase = Phase::Plan;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
        state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);

        let mut tlog = Vec::new();
        let before = state;
        let valid = PlanRecord::from_packet(state.packet).submission();
        let invalid = EvidenceSubmission::with_effect(
            GateId::Execution,
            Evidence::ArtifactReceipt,
            true,
            PacketEffect::None,
        );

        let result = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            Command::SubmitEvidenceBatch(vec![valid, invalid]),
        );

        assert_eq!(result, Err(CanonError::InvalidApiCommand));
        assert_eq!(state, before);
        assert!(tlog.is_empty());
    }

    #[test]
    fn command_envelope_rejects_tampered_hash() {
        let mut state = State::default();
        let mut tlog = Vec::new();
        let observation = ObservationRecord::new(1, 1, 0xabc, 1);
        let mut envelope = CommandEnvelope::new(7, Command::SubmitEvidence(observation.submission()));
        envelope.command_hash = envelope.command_hash.saturating_add(1);

        let result = crate::api::routes::handle_envelope(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            envelope,
        );

        assert_eq!(result, Err(CanonError::InvalidApiCommand));
        assert!(tlog.is_empty());
    }

    #[test]
    fn command_envelope_accepts_valid_command() {
        let mut state = State::default();
        let mut tlog = Vec::new();
        let cfg = RuntimeConfig::default();
        tick(&mut state, &mut tlog, cfg).unwrap();
        assert_eq!(state.phase, Phase::Invariant);

        let observation = ObservationRecord::new(1, 1, 0xabc, 1);
        let envelope = CommandEnvelope::new(1, Command::SubmitEvidence(observation.submission()));
        let response = crate::api::routes::handle_envelope(&mut state, &mut tlog, cfg, envelope)
            .unwrap();

        assert_eq!(response.event.cause, Cause::GatePassed);
        assert_eq!(state.phase, Phase::Analysis);
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn api_protocol_schema_v2_binds_command_hash_to_payload() {
        assert_eq!(API_PROTOCOL_SCHEMA_VERSION, 2);

        let first_submission = ObservationRecord::new(1, 1, 0xabc, 1).submission();
        let second_submission = ObservationRecord::new(2, 1, 0xabc, 1).submission();

        assert_eq!(first_submission.gate, second_submission.gate);
        assert_eq!(first_submission.evidence, second_submission.evidence);
        assert_eq!(first_submission.passed, second_submission.passed);
        assert_eq!(first_submission.effect, second_submission.effect);
        assert_ne!(first_submission.payload_hash, second_submission.payload_hash);

        let first_envelope = CommandEnvelope::new(11, Command::SubmitEvidence(first_submission));
        let second_envelope = CommandEnvelope::new(11, Command::SubmitEvidence(second_submission));

        assert_eq!(first_envelope.schema_version, API_PROTOCOL_SCHEMA_VERSION);
        assert_ne!(first_envelope.command_hash, second_envelope.command_hash);

        let mut forged_envelope = second_envelope.clone();
        forged_envelope.command_hash = first_envelope.command_hash;
        assert!(!forged_envelope.is_contract_valid());

        let mut legacy_envelope = first_envelope;
        legacy_envelope.schema_version = API_PROTOCOL_SCHEMA_VERSION - 1;
        assert!(!legacy_envelope.is_contract_valid());
    }

    #[test]
    fn command_ledger_replays_duplicate_envelope_without_new_event() {
        let mut state = State::default();
        let mut tlog = Vec::new();
        let mut ledger = CommandLedger::default();
        let cfg = RuntimeConfig::default();
        tick(&mut state, &mut tlog, cfg).unwrap();

        let observation = ObservationRecord::new(1, 1, 0xabc, 1);
        let envelope = CommandEnvelope::new(1, Command::SubmitEvidence(observation.submission()));
        let first = crate::api::routes::handle_envelope_once(
            &mut state,
            &mut tlog,
            cfg,
            &mut ledger,
            envelope.clone(),
        )
        .unwrap();
        let state_after_first = state;
        let tlog_len_after_first = tlog.len();

        let second = crate::api::routes::handle_envelope_once(
            &mut state,
            &mut tlog,
            cfg,
            &mut ledger,
            envelope,
        )
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(state, state_after_first);
        assert_eq!(tlog.len(), tlog_len_after_first);
        assert_eq!(ledger.len(), 1);
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn command_ledger_reconstructs_from_tlog_after_restart() {
        let mut state = State::default();
        let mut tlog = Vec::new();
        let mut ledger = CommandLedger::default();
        let cfg = RuntimeConfig::default();
        tick(&mut state, &mut tlog, cfg).unwrap();

        let observation = ObservationRecord::new(1, 1, 0xabc, 1);
        let envelope = CommandEnvelope::new(17, Command::SubmitEvidence(observation.submission()));
        let first = crate::api::routes::handle_envelope_once(
            &mut state,
            &mut tlog,
            cfg,
            &mut ledger,
            envelope.clone(),
        )
        .unwrap();

        assert_eq!(first.event.api_command_id, envelope.command_id);
        assert_eq!(first.event.api_command_hash, envelope.command_hash);

        let encoded = encode_tlog_ndjson_string(&tlog);
        let decoded = decode_tlog_ndjson_str(&encoded).unwrap();
        assert_eq!(decoded, tlog);

        let mut rebuilt_ledger = CommandLedger::reconstruct_from_tlog(&decoded).unwrap();
        assert_eq!(rebuilt_ledger.receipts(), ledger.receipts());

        let state_after_first = state;
        let tlog_len_after_first = tlog.len();
        let second = crate::api::routes::handle_envelope_once(
            &mut state,
            &mut tlog,
            cfg,
            &mut rebuilt_ledger,
            envelope,
        )
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(state, state_after_first);
        assert_eq!(tlog.len(), tlog_len_after_first);
        assert_eq!(rebuilt_ledger.len(), 1);
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn command_ledger_rejects_command_id_reuse_with_different_hash() {
        let mut state = State::default();
        let mut tlog = Vec::new();
        let mut ledger = CommandLedger::default();
        let cfg = RuntimeConfig::default();
        tick(&mut state, &mut tlog, cfg).unwrap();

        let first_observation = ObservationRecord::new(1, 1, 0xabc, 1);
        let first_envelope =
            CommandEnvelope::new(1, Command::SubmitEvidence(first_observation.submission()));
        crate::api::routes::handle_envelope_once(
            &mut state,
            &mut tlog,
            cfg,
            &mut ledger,
            first_envelope,
        )
        .unwrap();

        let state_after_first = state;
        let tlog_len_after_first = tlog.len();
        let conflicting_observation = ObservationRecord::new(2, 1, 0xdef, 1);
        let conflicting_envelope = CommandEnvelope::new(
            1,
            Command::SubmitEvidence(conflicting_observation.submission()),
        );

        let result = crate::api::routes::handle_envelope_once(
            &mut state,
            &mut tlog,
            cfg,
            &mut ledger,
            conflicting_envelope,
        );

        assert_eq!(result, Err(CanonError::InvalidApiCommand));
        assert_eq!(state, state_after_first);
        assert_eq!(tlog.len(), tlog_len_after_first);
        assert_eq!(ledger.len(), 1);
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn codec_public_event_roundtrip_preserves_event() {
        let (_, tlog) = run_until_done(State::ready(), RuntimeConfig::default()).unwrap();
        let encoded = encode_control_event_ndjson(&tlog[0]);
        let decoded = decode_control_event_ndjson(&encoded).unwrap();

        assert_eq!(decoded, tlog[0]);
    }

    #[test]
    fn codec_public_tlog_string_roundtrip_replays() {
        let initial = State::default();
        let (state, tlog) = run_until_done(initial, RuntimeConfig::default()).unwrap();
        let encoded = encode_tlog_ndjson_string(&tlog);
        let decoded = decode_tlog_ndjson_str(&encoded).unwrap();
        let replayed = verify_tlog_from(initial, &decoded).unwrap();

        assert_eq!(decoded, tlog);
        assert_eq!(replayed, state);
    }

    #[test]
    fn planning_record_binds_ready_task_through_api() {
        let mut state = State::default();
        state.phase = Phase::Plan;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
        state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);

        let mut tlog = Vec::new();
        let record = PlanRecord::from_packet(state.packet);
        let response = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            crate::api::protocol::Command::SubmitEvidence(record.submission()),
        )
        .unwrap();

        assert_eq!(response.event.from, Phase::Plan);
        assert_eq!(response.event.to, Phase::Execute);
        assert_eq!(response.event.evidence, Evidence::TaskReady);
        assert_eq!(state.gates.plan.status, GateStatus::Pass);
        assert!(state.packet.has_ready_task());
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn tooling_record_materializes_artifact_through_api() {
        let mut state = State::default();
        state.phase = Phase::Execute;
        state.packet.bind_ready_task();
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
        state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);
        state.gates.plan = Gate::pass(Evidence::TaskReady);

        let mut tlog = Vec::new();
        let record = ToolExecutionRecord::from_packet(state.packet);
        let response = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            crate::api::protocol::Command::SubmitEvidence(record.submission()),
        )
        .unwrap();

        assert_eq!(response.event.from, Phase::Execute);
        assert_eq!(response.event.to, Phase::Verify);
        assert_eq!(response.event.evidence, Evidence::ArtifactReceipt);
        assert_eq!(state.gates.execution.status, GateStatus::Pass);
        assert!(state.packet.artifact_receipt_valid());
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn tooling_executor_denies_invalid_request_without_packet_effect() {
        let request = ToolRequest {
            tool_kind: ToolKind::Denied,
            objective_id: 1,
            task_id: 0,
            command_hash: 1,
            input_hash: 1,
            requested_effect: PacketEffect::MaterializeArtifact,
        };
        let record = ToolExecutionRecord::from_request(request);
        let submission = record.submission();

        assert_eq!(record.decision(), ToolDecision::Failed);
        assert!(submission.is_contract_valid());
        assert!(!submission.passed);
        assert_eq!(submission.effect, PacketEffect::None);
    }

    #[test]
    fn tooling_executor_receipt_is_deterministic_for_same_request() {
        let mut packet = Packet::empty();
        packet.bind_ready_task();
        let request = ToolRequest::from_packet(packet);
        let executor = DeterministicToolExecutor::default();

        let first = executor.execute(request);
        let second = executor.execute(request);

        assert_eq!(first, second);
        assert!(first.is_success_for(request));
    }

    #[test]
    fn verification_record_repairs_lineage_through_api() {
        let mut state = State::default();
        state.phase = Phase::Verify;
        state.packet.bind_ready_task();
        state.packet.materialize_artifact();
        state.packet.artifact_lineage_hash = 0;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
        state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);
        state.gates.plan = Gate::pass(Evidence::TaskReady);
        state.gates.execution = Gate::pass(Evidence::ArtifactReceipt);

        let mut tlog = Vec::new();
        let record = VerificationRecord::from_packet(state.packet);
        let response = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            crate::api::protocol::Command::SubmitEvidence(record.submission()),
        )
        .unwrap();

        assert_eq!(response.event.from, Phase::Verify);
        assert_eq!(response.event.to, Phase::Eval);
        assert_eq!(response.event.evidence, Evidence::LineageProof);
        assert_eq!(state.gates.verification.status, GateStatus::Pass);
        assert!(state.packet.lineage_valid());
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn verification_profile_hashes_packet_semantics() {
        let mut packet = Packet::empty();
        packet.bind_ready_task();
        packet.materialize_artifact();

        let profile = ArtifactSemanticProfile::from_packet(packet);
        let record = VerificationRecord::from_packet(packet);

        assert!(profile.receipt_valid());
        assert!(profile.lineage_valid());
        assert!(record.is_valid());
        assert!(record.lineage_already_valid());
        assert_eq!(record.semantic_profile, profile);
        assert_eq!(record.expected_receipt_hash, packet.expected_receipt_hash());
        assert_eq!(record.expected_lineage_hash, packet.expected_lineage_hash());
    }

    #[test]
    fn semantic_verifier_receipt_is_deterministic_for_same_request() {
        let mut packet = Packet::empty();
        packet.bind_ready_task();
        packet.materialize_artifact();

        let request = VerificationRequest::from_packet(packet);
        let verifier = DeterministicSemanticVerifier;

        let first = verifier.verify(request);
        let second = verifier.verify(request);

        assert_eq!(request.check, VerificationCheck::ArtifactSemantics);
        assert_eq!(first, second);
        assert!(first.is_accepted_for(request));
    }

    #[test]
    fn semantic_verifier_rejects_denied_request_without_repair_effect() {
        let request = VerificationRequest {
            check: VerificationCheck::Denied,
            profile: ArtifactSemanticProfile::from_packet(Packet::empty()),
            requested_effect: PacketEffect::RepairLineage,
        };

        let record = VerificationRecord::from_request(request);
        let submission = record.submission();

        assert_eq!(record.decision(), VerificationDecision::Rejected);
        assert!(submission.is_contract_valid());
        assert!(!submission.passed);
        assert_eq!(submission.effect, PacketEffect::None);
    }

    #[test]
    fn semantic_verification_rejects_receipt_mismatch() {
        let mut packet = Packet::empty();
        packet.bind_ready_task();
        packet.materialize_artifact();
        packet.artifact_bytes = packet.artifact_bytes.saturating_add(1);

        let record = VerificationRecord::from_packet(packet);

        assert_eq!(record.decision(), VerificationDecision::Rejected);
        assert!(!record.is_valid());
        assert!(!record.submission().passed);
    }

    #[test]
    fn semantic_verification_rejects_tampered_profile_hash() {
        let mut packet = Packet::empty();
        packet.bind_ready_task();
        packet.materialize_artifact();

        let mut record = VerificationRecord::from_packet(packet);
        record.semantic_profile_hash = record.semantic_profile_hash.saturating_add(1);

        assert_eq!(record.decision(), VerificationDecision::Rejected);
        assert!(!record.is_valid());
        assert!(!record.submission().passed);
    }

    #[test]
    fn invalid_semantic_verification_fails_without_repairing_lineage() {
        let mut state = State::default();
        state.phase = Phase::Verify;
        state.packet.bind_ready_task();
        state.packet.materialize_artifact();
        state.packet.artifact_lineage_hash = 0;
        state.packet.artifact_bytes = state.packet.artifact_bytes.saturating_add(1);
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
        state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);
        state.gates.plan = Gate::pass(Evidence::TaskReady);
        state.gates.execution = Gate::pass(Evidence::ArtifactReceipt);

        let mut tlog = Vec::new();
        let record = VerificationRecord::from_packet(state.packet);
        let response = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            crate::api::protocol::Command::SubmitEvidence(record.submission()),
        )
        .unwrap();

        assert_eq!(response.event.from, Phase::Verify);
        assert_eq!(response.event.to, Phase::Recovery);
        assert_eq!(response.event.failure, Some(FailureClass::VerificationFailed));
        assert_eq!(state.gates.verification.status, GateStatus::Fail);
        assert!(!state.packet.lineage_valid());
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn capability_records_drive_objective_from_plan_to_persist() {
        let mut state = State::default();
        state.phase = Phase::Plan;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
        state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);

        let mut tlog = Vec::new();
        let cfg = RuntimeConfig::default();

        let plan = PlanRecord::from_packet(state.packet);
        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            cfg,
            crate::api::protocol::Command::SubmitEvidence(plan.submission()),
        )
        .unwrap();

        let tool = ToolExecutionRecord::from_packet(state.packet);
        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            cfg,
            crate::api::protocol::Command::SubmitEvidence(tool.submission()),
        )
        .unwrap();

        let verification = VerificationRecord::from_packet(state.packet);
        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            cfg,
            crate::api::protocol::Command::SubmitEvidence(verification.submission()),
        )
        .unwrap();

        let eval = EvalRecord {
            score: 95,
            threshold_used: 80,
            dimensions: vec![EvalDimension {
                id: "artifact_semantics",
                score: 95,
                threshold: 80,
            }],
        };
        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            cfg,
            crate::api::protocol::Command::SubmitEvidence(eval.submission()),
        )
        .unwrap();

        assert_eq!(state.phase, Phase::Persist);
        assert_eq!(state.gates.plan.status, GateStatus::Pass);
        assert_eq!(state.gates.execution.status, GateStatus::Pass);
        assert_eq!(state.gates.verification.status, GateStatus::Pass);
        assert_eq!(state.gates.eval.status, GateStatus::Pass);
        assert!(state.packet.objective_complete());
        assert!(state.packet.lineage_valid());
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
    fn durable_resume_reconstructs_command_ledger_from_tlog() {
        let path = std::env::temp_dir().join(format!(
            "ai-tlog-ledger-resume-{}.ndjson",
            std::process::id()
        ));
        std::fs::remove_file(&path).ok();

        let cfg = RuntimeConfig::default();
        let mut state = State::default();
        let mut tlog = Vec::new();
        let mut ledger = CommandLedger::default();
        tick(&mut state, &mut tlog, cfg).unwrap();

        let observation = ObservationRecord::new(1, 1, 0xabc, 1);
        let envelope = CommandEnvelope::new(71, Command::SubmitEvidence(observation.submission()));
        let first = crate::api::routes::handle_envelope_once(
            &mut state,
            &mut tlog,
            cfg,
            &mut ledger,
            envelope.clone(),
        )
        .unwrap();

        write_tlog_ndjson(&path, &tlog).unwrap();
        let resumed = resume_durable_runtime(State::default(), &path).unwrap();

        assert_eq!(resumed.state, state);
        assert_eq!(resumed.tlog, tlog);
        assert_eq!(resumed.command_ledger.receipts(), ledger.receipts());

        let completed =
            run_until_done_durable_with_ledger(State::default(), cfg, &path).unwrap();
        assert!(completed.state.is_success());
        assert_eq!(completed.command_ledger.receipts(), ledger.receipts());

        let mut resumed_state = resumed.state;
        let mut resumed_tlog = resumed.tlog;
        let mut resumed_ledger = resumed.command_ledger;
        let resumed_tlog_len = resumed_tlog.len();
        let second = crate::api::routes::handle_envelope_once(
            &mut resumed_state,
            &mut resumed_tlog,
            cfg,
            &mut resumed_ledger,
            envelope,
        )
        .unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(first, second);
        assert_eq!(resumed_tlog.len(), resumed_tlog_len);
        assert_eq!(resumed_ledger.receipts(), ledger.receipts());
    }

    #[test]
    fn replay_report_exposes_final_hash_and_seq_bounds() {
        let initial = State::ready();
        let (state, tlog) = run_until_done(initial, RuntimeConfig::default()).unwrap();
        let report = replay_report_from(initial, &tlog).unwrap();

        assert_eq!(report.initial_state, initial);
        assert_eq!(report.final_state, state);
        assert_eq!(report.event_count, tlog.len());
        assert_eq!(report.first_seq, Some(1));
        assert_eq!(report.last_seq, Some(tlog.len() as u64));
        assert_eq!(report.final_hash, tlog.last().unwrap().self_hash);
    }

    #[test]
    fn durable_replay_report_matches_disk_tlog() {
        let initial = State::default();
        let (state, tlog) = run_until_done(initial, RuntimeConfig::default()).unwrap();
        let path = std::env::temp_dir().join(format!(
            "ai-durable-report-{}-{}.ndjson",
            std::process::id(),
            tlog.len()
        ));

        write_tlog_ndjson(&path, &tlog).unwrap();
        let report = durable_replay_report(initial, &path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(report.final_state, state);
        assert_eq!(report.event_count, tlog.len());
        assert_eq!(report.final_hash, tlog.last().unwrap().self_hash);
    }

    #[test]
    fn durable_tick_checked_rejects_memory_disk_drift() {
        let cfg = RuntimeConfig::default();
        let path = std::env::temp_dir().join(format!(
            "ai-durable-drift-{}.ndjson",
            std::process::id()
        ));
        std::fs::remove_file(&path).ok();

        let mut state = State::default();
        let mut tlog = Vec::new();
        tick_durable(&mut state, &mut tlog, &path, cfg).unwrap();

        let mut drifted_tlog = tlog.clone();
        drifted_tlog.clear();
        let before = state;
        let result = tick_durable_checked(
            &mut state,
            &mut drifted_tlog,
            &path,
            State::default(),
            cfg,
        );

        std::fs::remove_file(&path).ok();
        assert_eq!(result, Err(CanonError::InvalidReplay));
        assert_eq!(state, before);
        assert!(drifted_tlog.is_empty());
    }

    #[test]
    fn durable_tick_checked_rejects_state_disk_drift() {
        let cfg = RuntimeConfig::default();
        let path = std::env::temp_dir().join(format!(
            "ai-durable-state-drift-{}.ndjson",
            std::process::id()
        ));
        std::fs::remove_file(&path).ok();

        let mut disk_state = State::default();
        let mut tlog = Vec::new();
        tick_durable(&mut disk_state, &mut tlog, &path, cfg).unwrap();

        let mut drifted_state = State::default();
        let result = tick_durable_checked(
            &mut drifted_state,
            &mut tlog,
            &path,
            State::default(),
            cfg,
        );

        std::fs::remove_file(&path).ok();
        assert_eq!(result, Err(CanonError::InvalidStateContinuity));
        assert_eq!(drifted_state, State::default());
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
            api_command_id: event.api_command_id,
            api_command_hash: event.api_command_hash,
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

    #[test]
    fn structurally_invalid_public_state_is_rejected() {
        let cfg = RuntimeConfig::default();
        let mut state = State::ready();
        state.gates.invariant = Gate {
            status: GateStatus::Pass,
            evidence: Evidence::Missing,
            version: 1,
        };
        let mut tlog = Vec::new();

        assert_eq!(tick(&mut state, &mut tlog, cfg), Err(CanonError::InvalidStateInvariant));
        assert!(tlog.is_empty());
    }

    #[test]
    fn zero_step_runtime_config_is_rejected() {
        let cfg = RuntimeConfig {
            max_steps: 0,
            max_recovery_attempts: 1,
        };
        let mut state = State::ready();
        let mut tlog = Vec::new();

        assert_eq!(tick(&mut state, &mut tlog, cfg), Err(CanonError::InvalidRuntimeConfig));
        assert!(tlog.is_empty());
    }

    #[test]
    fn kernel_mix_is_single_hash_primitive_source() {
        fn collect_mix_definitions(path: &std::path::Path, out: &mut Vec<String>) {
            let entries = std::fs::read_dir(path).expect("source directory must be readable");
            for entry in entries {
                let entry = entry.expect("source entry must be readable");
                let path = entry.path();

                if path.is_dir() {
                    collect_mix_definitions(&path, out);
                    continue;
                }

                if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                    continue;
                }

                let source = std::fs::read_to_string(&path).expect("rust source must be readable");
                let relative = path
                    .strip_prefix(env!("CARGO_MANIFEST_DIR"))
                    .unwrap_or(&path)
                    .display()
                    .to_string()
                    .replace('\\', "/");

                for (line_index, line) in source.lines().enumerate() {
                    let trimmed = line.trim_start();
                    let is_mix_definition = trimmed.starts_with("fn mix(")
                        || trimmed.starts_with("pub fn mix(")
                        || trimmed.starts_with("pub(crate) fn mix(")
                        || trimmed.starts_with("pub(super) fn mix(")
                        || trimmed.starts_with("const fn mix(")
                        || trimmed.starts_with("pub const fn mix(")
                        || trimmed.starts_with("pub(crate) const fn mix(")
                        || trimmed.starts_with("pub(super) const fn mix(");

                    if is_mix_definition {
                        out.push(format!("{}:{}", relative, line_index + 1));
                    }
                }
            }
        }

        let mut definitions = Vec::new();
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        collect_mix_definitions(&src, &mut definitions);

        assert_eq!(
            definitions.len(),
            1,
            "mix must have exactly one implementation source: {:?}",
            definitions
        );
        assert!(
            definitions[0].starts_with("src/kernel/mod.rs:"),
            "mix implementation must live in the kernel: {:?}",
            definitions
        );
    }

}
