#![forbid(unsafe_code)]
//! Canonical atomic state-machine runtime.
//!
//! Layering rule: kernel stays frozen; capabilities own rich records; runtime
//! turns evidence into control events; codec only serializes/deserializes; API is
//! the outer surface.

pub mod api;
pub mod capability;
pub mod codec;
pub mod error;
pub mod kernel;
pub mod runtime;

pub use crate::api::protocol::{
    Command, CommandEnvelope, ControlEventResponse, API_PROTOCOL_SCHEMA_VERSION,
};
pub use crate::capability::context::{ContextDecision, ContextRecord};
pub use crate::capability::eval::{EvalDecision, EvalDimension, EvalRecord};
pub use crate::capability::learning::{
    PolicyPromotion, POLICY_FEEDBACK_HASH, POLICY_PROMOTION_SOURCE_SEQ,
};
pub use crate::capability::llm::{
    append_ollama_judgment_proof_event_ndjson, append_ollama_llm_effect_receipt_ndjson,
    decode_ollama_judgment_proof_event_ndjson, decode_ollama_llm_effect_receipt_ndjson,
    encode_ollama_judgment_proof_event_ndjson, encode_ollama_llm_effect_receipt_ndjson,
    load_ollama_judgment_proof_events_ndjson, load_ollama_llm_effect_receipts_ndjson,
    verify_ollama_judgment_proof_event_order_ndjson, verify_ollama_judgment_proof_events,
    verify_ollama_judgment_proof_events_ndjson, verify_ollama_judgment_tlog_ndjson,
    verify_ollama_llm_effect_receipts, LlmDecision, LlmPromptRecord, LlmRecord,
    LlmResponseRecord, LlmStructuredAdapter, OllamaChatResponse, OllamaClient, OllamaConfig,
    OllamaError, OllamaJudgmentProofEvent, OllamaLlmCall,
    OllamaLlmEffectReceipt, OllamaMessage, OllamaRetryBudgetDecision,
    OllamaRetryBudgetLedger, OllamaRetryBudgetPolicy, OLLAMA_JUDGMENT_PROOF_LINE,
    OLLAMA_JUDGMENT_PROOF_RECORD, OLLAMA_JUDGMENT_PROOF_SCHEMA_VERSION,
    OLLAMA_LLM_EFFECT_RECEIPT_RECORD, OLLAMA_LLM_EFFECT_RECEIPT_SCHEMA_VERSION,
    OLLAMA_PROVIDER,
};
pub use crate::capability::memory::{MemoryFact, MemoryIndex, MemoryLookupRecord};
pub use crate::capability::observation::{
    decode_observation_cursor_ndjson, encode_observation_cursor_ndjson,
    load_observation_cursor_ndjson, write_observation_cursor_ndjson,
    BoundedLineObservationSource,
    ObservationIngressBatch, ObservationIngressConfig, ObservationIngressDecision,
    ObservationCursor, ObservationDecision, ObservationFrame, ObservationFrameKind,
    ObservationRecord, MAX_OBSERVATION_PAYLOAD_BYTES, OBSERVATION_CURSOR_RECORD,
    OBSERVATION_CURSOR_SCHEMA_VERSION,
};
pub use crate::capability::orchestration::{
    CapabilityRoute, OrchestrationDecision, OrchestrationRecord,
};
pub use crate::capability::planning::{PlanDecision, PlanRecord};
pub use crate::capability::policy::{PolicyEntry, PolicyStore, PolicyStoreError};
pub use crate::capability::tooling::{
    append_process_effect_receipt_ndjson, append_sandbox_process_receipt_ndjson,
    append_tool_effect_receipt_ndjson, decode_process_effect_receipt_ndjson,
    decode_sandbox_process_receipt_ndjson, decode_tool_effect_receipt_ndjson,
    encode_process_effect_receipt_ndjson, encode_sandbox_process_receipt_ndjson,
    encode_tool_effect_receipt_ndjson, load_process_effect_receipts_ndjson,
    load_sandbox_process_receipts_ndjson, load_tool_effect_receipts_ndjson,
    verify_process_effect_receipts, verify_sandbox_process_receipts, verify_tool_effect_receipts,
    DeterministicToolExecutor, Effect, LiveSandboxProcessExecutor, LiveSandboxToolExecutor,
    ProcessEffectReceipt, SandboxProcessReceipt, SandboxProcessRequest, ToolDecision,
    ToolEffectKind, ToolEffectReceipt, ToolExecutionRecord, ToolKind, ToolReceipt, ToolRequest,
    ToolSandboxError, PROCESS_EFFECT_RECEIPT_RECORD, PROCESS_EFFECT_RECEIPT_SCHEMA_VERSION,
    SANDBOX_PROCESS_RECEIPT_RECORD, SANDBOX_PROCESS_RECEIPT_SCHEMA_VERSION,
    TOOL_EFFECT_RECEIPT_RECORD, TOOL_EFFECT_RECEIPT_SCHEMA_VERSION,
};
pub use crate::capability::verification::{
    ArtifactSemanticProfile, DeterministicSemanticVerifier, SemanticVerificationReceipt,
    verify_verification_proof_record_bindings, verify_verification_proof_record_order_ndjson,
    verify_verification_proof_record_replay, verify_verification_proof_record_replay_ndjson,
    ProofSubjectKind, VerificationCheck, VerificationDecision, VerificationProofBinding,
    VerificationProofRecord, VerificationRecord, VerificationRequest,
};
pub use crate::capability::{
    evidence_allowed_for_gate, expected_evidence_for_gate, CapabilityEffectRoute, CapabilityId,
    CapabilityRegistry, EvidenceSubmission, PacketEffect, CAPABILITY_EFFECT_ROUTES,
};
pub use crate::codec::ndjson::{
    append_tlog_ndjson, decode_control_event_ndjson, decode_tlog_ndjson_str,
    encode_control_event_ndjson, encode_tlog_ndjson_string, load_tlog_ndjson, write_tlog_ndjson,
    TLOG_RECORD_EVENT, TLOG_SCHEMA_VERSION,
};
pub use crate::kernel::{
    CapabilityRegistryProjection, Cause, ControlEvent, Decision, EventKind, Evidence,
    FailureClass, Gate, GateId, GateSet, GateStatus, Packet, Phase, RecoveryAction,
    RuntimeConfig, SemanticDelta, State, TLog, EXECUTION_GATE_ORDER, GATE_ORDER, PHASES,
};
pub use crate::runtime::{
    CanonError, CommandLedger, CommandReceipt,
    durable_replay_report, legal_transition, replay_report_from, replay_report_ndjson,
    replay_tlog_ndjson, resume_durable_runtime, run_until_done,
    run_until_done_durable, run_until_done_durable_with_ledger, semantic_diff, tick,
    tick_durable, tick_durable_checked, touch_all_surfaces, verify_tlog, verify_tlog_from,
    DurableRuntimeState, ReplayReport,
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

    fn tamper_first_ollama_receipt_field(
        source: &std::path::Path,
        target: &std::path::Path,
        field_index: usize,
    ) {
        let input = std::fs::read_to_string(source).unwrap();
        let mut output = String::new();
        let mut changed = false;

        for line in input.lines() {
            let trimmed = line.trim();
            let maybe_body = trimmed
                .strip_prefix('[')
                .and_then(|value| value.strip_suffix(']'));
            if !changed {
                if let Some(body) = maybe_body {
                    let parsed = body
                        .split(',')
                        .map(|raw| raw.trim().parse::<u64>())
                        .collect::<Result<Vec<_>, _>>();
                    if let Ok(mut fields) = parsed {
                        if fields.len() >= 2
                            && fields[0] == OLLAMA_LLM_EFFECT_RECEIPT_SCHEMA_VERSION
                            && fields[1] == OLLAMA_LLM_EFFECT_RECEIPT_RECORD
                            && field_index < fields.len()
                        {
                            fields[field_index] ^= 1;
                            output.push('[');
                            output.push_str(
                                &fields
                                    .iter()
                                    .map(u64::to_string)
                                    .collect::<Vec<_>>()
                                    .join(","),
                            );
                            output.push_str("]\n");
                            changed = true;
                            continue;
                        }
                    }
                }
            }
            output.push_str(line);
            output.push('\n');
        }

        assert!(changed);
        std::fs::write(target, output).unwrap();
    }

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
    fn observation_cursor_accepts_ordered_external_frame() {
        let frame = ObservationFrame::from_payload(
            ObservationFrameKind::ExternalSignal,
            7,
            1,
            3,
            b"external-frame-payload",
        );
        let mut cursor = ObservationCursor::new(7);

        let record = cursor.ingest(&frame);

        assert!(frame.is_valid());
        assert_eq!(record.decision(), ObservationDecision::Accepted);
        assert_eq!(record.observed_hash, frame.observed_hash());
        assert_eq!(cursor.last_sequence, 1);
        assert_eq!(cursor.last_observed_hash, record.observed_hash);
        assert!(record.submission().is_contract_valid());
    }

    #[test]
    fn observation_cursor_rejects_replayed_sequence_without_advancing() {
        let mut cursor = ObservationCursor::new(7);
        let first = ObservationFrame::from_payload(
            ObservationFrameKind::ExternalSignal,
            7,
            1,
            3,
            b"external-frame-payload",
        );
        let replay = ObservationFrame::from_payload(
            ObservationFrameKind::ExternalSignal,
            7,
            1,
            4,
            b"external-frame-payload-replay",
        );

        let accepted = cursor.ingest(&first);
        let rejected = cursor.ingest(&replay);

        assert_eq!(accepted.decision(), ObservationDecision::Accepted);
        assert_eq!(rejected.decision(), ObservationDecision::Rejected);
        assert_eq!(rejected.observed_hash, 0);
        assert_eq!(cursor.last_sequence, 1);
        assert_eq!(cursor.last_observed_hash, accepted.observed_hash);
    }

    #[test]
    fn bounded_line_observation_source_persists_cursor_and_applies_backpressure() {
        let stem = std::env::temp_dir().join(format!(
            "ai-observation-ingress-{}",
            std::process::id()
        ));
        let source_path = stem.with_extension("log");
        let cursor_path = stem.with_extension("cursor.ndjson");
        std::fs::remove_file(&source_path).ok();
        std::fs::remove_file(&cursor_path).ok();
        std::fs::write(&source_path, b"alpha\nbeta\ngamma\n").unwrap();

        let pressured = BoundedLineObservationSource::new(
            source_path.clone(),
            cursor_path.clone(),
            ObservationIngressConfig::new(9, 1, 2, 5),
        )
        .read_batch()
        .unwrap();

        assert_eq!(pressured.decision, ObservationIngressDecision::Backpressure);
        assert_eq!(pressured.backlog_len, 3);
        assert!(pressured.records.is_empty());
        assert!(load_observation_cursor_ndjson(&cursor_path).unwrap().is_none());

        let source = BoundedLineObservationSource::new(
            source_path.clone(),
            cursor_path.clone(),
            ObservationIngressConfig::new(9, 2, 4, 5),
        );
        let first = source.read_batch().unwrap();

        assert_eq!(first.decision, ObservationIngressDecision::Accepted);
        assert_eq!(first.records.len(), 2);
        assert_eq!(first.records[0].sequence, 1);
        assert_eq!(first.records[1].sequence, 2);
        assert_eq!(first.backlog_len, 1);
        assert!(first.is_accepted());

        let persisted = load_observation_cursor_ndjson(&cursor_path)
            .unwrap()
            .unwrap();
        assert_eq!(persisted.source_id, 9);
        assert_eq!(persisted.last_sequence, 2);
        assert_eq!(persisted.last_observed_hash, first.records[1].observed_hash);

        let second = source.read_batch().unwrap();
        assert_eq!(second.decision, ObservationIngressDecision::Accepted);
        assert_eq!(second.records.len(), 1);
        assert_eq!(second.records[0].sequence, 3);
        assert_eq!(second.backlog_len, 0);

        let finished = source.read_batch().unwrap();
        assert_eq!(finished.decision, ObservationIngressDecision::Empty);
        assert!(finished.records.is_empty());

        std::fs::remove_file(&source_path).ok();
        std::fs::remove_file(&cursor_path).ok();
    }

    #[test]
    fn observation_ingress_batch_routes_through_api_to_invariant_gate() {
        let stem = std::env::temp_dir().join(format!(
            "ai-observation-api-ingress-{}",
            std::process::id()
        ));
        let source_path = stem.with_extension("log");
        let cursor_path = stem.with_extension("cursor.ndjson");
        std::fs::remove_file(&source_path).ok();
        std::fs::remove_file(&cursor_path).ok();
        std::fs::write(&source_path, b"external-alpha\nexternal-beta\n").unwrap();

        let batch = BoundedLineObservationSource::new(
            source_path.clone(),
            cursor_path.clone(),
            ObservationIngressConfig::new(31, 2, 4, 11),
        )
        .read_batch()
        .unwrap();

        assert!(batch.is_contract_valid());
        assert_eq!(batch.submission().gate, GateId::Invariant);
        assert_eq!(batch.submission().evidence, Evidence::InvariantProof);
        assert!(batch.submission().passed);

        let mut state = State::default();
        let mut tlog = Vec::new();
        let mut ledger = CommandLedger::default();
        let cfg = RuntimeConfig::default();
        tick(&mut state, &mut tlog, cfg).unwrap();
        assert_eq!(state.phase, Phase::Invariant);

        let envelope = CommandEnvelope::new(91, Command::SubmitObservationIngress(batch));
        let command_hash = envelope.command_hash;
        let response = crate::api::routes::handle_envelope_once(
            &mut state,
            &mut tlog,
            cfg,
            &mut ledger,
            envelope,
        )
        .unwrap();

        assert_eq!(response.event.from, Phase::Invariant);
        assert_eq!(response.event.to, Phase::Analysis);
        assert_eq!(response.event.api_command_id, 91);
        assert_eq!(response.event.api_command_hash, command_hash);
        assert_eq!(state.gates.invariant.status, GateStatus::Pass);
        assert_eq!(state.gates.invariant.evidence, Evidence::InvariantProof);
        assert_eq!(ledger.len(), 1);
        verify_tlog(&tlog).unwrap();

        std::fs::remove_file(&source_path).ok();
        std::fs::remove_file(&cursor_path).ok();
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
    fn ollama_config_rejects_non_local_base_url() {
        let cfg = OllamaConfig {
            base_url: "http://192.168.1.2:11434/v1".to_string(),
            model: "qwen2.5-coder:7b".to_string(),
            timeout_ms: 30_000,
        };

        assert!(OllamaClient::new(cfg).is_err());
    }

    #[test]
    fn ollama_adapter_builds_openai_compatible_qwen_request() {
        let client = OllamaClient::new(OllamaConfig::default()).unwrap();
        let body = client
            .request_json(&[
                OllamaMessage::system("system"),
                OllamaMessage::user("ping"),
            ])
            .unwrap();

        assert_eq!(
            client.config().chat_completions_path().unwrap(),
            "/v1/chat/completions"
        );
        assert!(body.contains("\"model\":\"qwen2.5-coder:7b\""));
        assert!(body.contains("\"role\":\"system\""));
        assert!(body.contains("\"role\":\"user\""));
        assert!(body.contains("\"stream\":false"));
    }

    #[test]
    fn ollama_retry_budget_ledger_rejects_duplicate_request_identity() {
        let policy = OllamaRetryBudgetPolicy::new(30_000, 2, 3).unwrap();
        let mut ledger = OllamaRetryBudgetLedger::new(policy);

        let first = ledger.record_request(1, 2, 3, 4).unwrap();
        assert!(first.allowed);
        assert_eq!(first.retry_count, 0);
        assert!(!first.budget_exhausted);
        assert!(!first.duplicate_request);
        assert_ne!(first.request_identity_hash, 0);
        assert_eq!(first.retry_budget_hash, policy.policy_hash());

        assert!(matches!(
            ledger.record_request(1, 2, 3, 4),
            Err(OllamaError::DuplicateRequest)
        ));
    }

    #[test]
    fn ollama_response_body_drives_judgment_record_through_api() {
        let mut state = State::default();
        state.phase = Phase::Judgment;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);
        let policy = PolicyStore::default();
        let client = OllamaClient::new(OllamaConfig::default()).unwrap();
        let llm = client
            .record_from_response_body(
                &context,
                &policy,
                "{\"id\":\"chatcmpl-1\",\"object\":\"chat.completion\",\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"Proceed with the plan gate.\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":30,\"completion_tokens\":8,\"total_tokens\":38}}",
            )
            .unwrap();

        let mut tlog = Vec::new();
        let response = crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            crate::api::protocol::Command::SubmitEvidence(llm.submission()),
        )
        .unwrap();

        assert_eq!(llm.decision(), LlmDecision::Structured);
        assert_eq!(llm.response.model_id, client.config().model_id());
        assert_eq!(llm.response.token_count, 38);
        assert_eq!(response.event.from, Phase::Judgment);
        assert_eq!(response.event.to, Phase::Plan);
        assert_eq!(state.gates.judgment.status, GateStatus::Pass);
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn ollama_effect_receipt_persists_in_mixed_tlog_and_replays() {
        let mut state = State::default();
        state.phase = Phase::Judgment;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);
        let policy = PolicyStore::default();
        let client = OllamaClient::new(OllamaConfig::default()).unwrap();
        let call = client
            .call_from_response_body(
                &context,
                &policy,
                "{\"id\":\"chatcmpl-1\",\"object\":\"chat.completion\",\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"Persist this external llm receipt.\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":30,\"completion_tokens\":8,\"total_tokens\":38}}",
            )
            .unwrap();
        let envelope = CommandEnvelope::new(
            call.request_hash,
            crate::api::protocol::Command::SubmitEvidence(call.submission()),
        );

        let mut tlog = Vec::new();
        crate::api::routes::handle_envelope(&mut state, &mut tlog, RuntimeConfig::default(), envelope.clone())
            .unwrap();
        let persisted_event = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.evidence == Evidence::JudgmentRecord
                    && event.api_command_hash == envelope.command_hash
            })
            .copied()
            .unwrap();
        let receipt = call
            .receipt_for_configured_event(client.config(), envelope.command_hash, &persisted_event)
            .unwrap();

        assert!(receipt.is_valid());
        assert!(receipt.replay_verified(&tlog));
        assert_eq!(receipt.provider_hash, crate::capability::llm::ollama::hash_text(OLLAMA_PROVIDER));
        assert_eq!(receipt.base_url_hash, client.config().base_url_id());
        assert_eq!(receipt.model_id, client.config().model_id());
        assert_eq!(receipt.request_hash, call.request_hash);
        assert_eq!(receipt.timeout_ms, client.config().timeout_ms);
        assert_eq!(receipt.retry_count, 0);
        assert_eq!(receipt.max_retries, 0);
        assert_eq!(receipt.attempt_budget, 1);
        assert_eq!(receipt.request_identity_hash, call.request_identity_hash);
        assert_eq!(receipt.retry_budget_hash, call.retry_budget_hash);
        assert!(receipt.budget_exhausted);
        assert!(!receipt.duplicate_request);
        assert_eq!(receipt.response_hash, call.response_hash);

        let encoded = encode_ollama_llm_effect_receipt_ndjson(receipt);
        let decoded = decode_ollama_llm_effect_receipt_ndjson(&encoded).unwrap();
        assert_eq!(decoded, receipt);

        let path = std::env::temp_dir().join(format!(
            "ai-ollama-mixed-tlog-{}-{}.ndjson",
            std::process::id(),
            receipt.receipt_hash
        ));
        std::fs::remove_file(&path).ok();
        write_tlog_ndjson(&path, &tlog).unwrap();
        append_ollama_llm_effect_receipt_ndjson(&path, &receipt).unwrap();

        let loaded_tlog = load_tlog_ndjson(&path).unwrap();
        let loaded_receipts = load_ollama_llm_effect_receipts_ndjson(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(loaded_tlog, tlog);
        assert_eq!(loaded_receipts, vec![receipt]);
        assert_eq!(
            verify_ollama_llm_effect_receipts(&loaded_tlog, &loaded_receipts).unwrap(),
            1
        );

        let mut tampered = receipt;
        tampered.response_hash ^= 1;
        assert!(!tampered.is_valid());
        assert!(!tampered.replay_verified(&loaded_tlog));
    }


    #[test]
    fn ollama_judgment_final_proof_persists_as_verification_event() {
        let mut state = State::default();
        state.phase = Phase::Judgment;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);
        let policy = PolicyStore::default();
        let client = OllamaClient::new(OllamaConfig::default()).unwrap();
        let call = client
            .call_from_response_body(
                &context,
                &policy,
                "{\"id\":\"chatcmpl-1\",\"object\":\"chat.completion\",\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"Durably persist the final proof line.\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":30,\"completion_tokens\":8,\"total_tokens\":38}}",
            )
            .unwrap();
        let envelope = CommandEnvelope::new(
            call.request_hash,
            crate::api::protocol::Command::SubmitEvidence(call.submission()),
        );

        let mut tlog = Vec::new();
        crate::api::routes::handle_envelope(&mut state, &mut tlog, RuntimeConfig::default(), envelope.clone())
            .unwrap();
        let persisted_event = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.evidence == Evidence::JudgmentRecord
                    && event.api_command_hash == envelope.command_hash
            })
            .copied()
            .unwrap();
        let base_receipt = call
            .receipt_for_configured_event(client.config(), envelope.command_hash, &persisted_event)
            .unwrap();

        let (receipt, proof_event) = OllamaJudgmentProofEvent::finalize_receipt_after_tlog(
            base_receipt,
            &tlog,
            true,
            true,
            base_receipt.base_url_provenance_verified(client.config()),
            state.phase == Phase::Plan,
        )
        .unwrap();
        assert!(receipt.has_proof_binding());
        assert_eq!(receipt.proof_hash, proof_event.proof_hash);
        assert_eq!(
            proof_event.proof_line_hash,
            crate::capability::llm::ollama::hash_text(OLLAMA_JUDGMENT_PROOF_LINE)
        );
        assert!(proof_event.matches_receipt(receipt, &tlog));

        let encoded = encode_ollama_judgment_proof_event_ndjson(proof_event);
        let decoded = decode_ollama_judgment_proof_event_ndjson(&encoded).unwrap();
        assert_eq!(decoded, proof_event);

        let path = std::env::temp_dir().join(format!(
            "ai-ollama-proof-tlog-{}-{}.ndjson",
            std::process::id(),
            proof_event.proof_hash
        ));
        std::fs::remove_file(&path).ok();
        write_tlog_ndjson(&path, &tlog).unwrap();
        append_ollama_llm_effect_receipt_ndjson(&path, &receipt).unwrap();
        append_ollama_judgment_proof_event_ndjson(&path, &proof_event).unwrap();

        let loaded_tlog = load_tlog_ndjson(&path).unwrap();
        let loaded_receipts = load_ollama_llm_effect_receipts_ndjson(&path).unwrap();
        let loaded_proofs = load_ollama_judgment_proof_events_ndjson(&path).unwrap();
        std::fs::remove_file(&path).ok();

        assert_eq!(loaded_tlog, tlog);
        assert_eq!(loaded_receipts, vec![receipt]);
        assert_eq!(loaded_proofs, vec![proof_event]);
        assert_eq!(
            verify_ollama_judgment_proof_events(&loaded_tlog, &loaded_receipts, &loaded_proofs)
                .unwrap(),
            1
        );
    }

    #[test]
    fn ollama_proof_event_projects_into_generic_verification_spine() {
        let mut state = State::default();
        state.phase = Phase::Judgment;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);
        let policy = PolicyStore::default();
        let client = OllamaClient::new(OllamaConfig::default()).unwrap();
        let call = client
            .call_from_response_body(
                &context,
                &policy,
                "{\"id\":\"chatcmpl-1\",\"object\":\"chat.completion\",\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"Project proof into generic verification spine.\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":30,\"completion_tokens\":8,\"total_tokens\":38}}",
            )
            .unwrap();
        let envelope = CommandEnvelope::new(
            call.request_hash,
            crate::api::protocol::Command::SubmitEvidence(call.submission()),
        );

        let mut tlog = Vec::new();
        crate::api::routes::handle_envelope(&mut state, &mut tlog, RuntimeConfig::default(), envelope.clone())
            .unwrap();
        let persisted_event = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.evidence == Evidence::JudgmentRecord
                    && event.api_command_hash == envelope.command_hash
            })
            .copied()
            .unwrap();
        let base_receipt = call
            .receipt_for_configured_event(client.config(), envelope.command_hash, &persisted_event)
            .unwrap();
        let (receipt, proof_event) = OllamaJudgmentProofEvent::finalize_receipt_after_tlog(
            base_receipt,
            &tlog,
            true,
            true,
            base_receipt.base_url_provenance_verified(client.config()),
            state.phase == Phase::Plan,
        )
        .unwrap();

        let proof_record = proof_event.to_verification_proof_record().unwrap();
        let binding = receipt.verification_proof_binding().unwrap();
        assert_eq!(proof_record.subject, ProofSubjectKind::LlmEffect);
        assert!(proof_record.matches_binding(binding));
        assert_eq!(
            verify_verification_proof_record_bindings(&[proof_record], &[binding]).unwrap(),
            1
        );
        assert_eq!(
            verify_ollama_judgment_proof_events(&tlog, &[receipt], &[proof_event]).unwrap(),
            1
        );

        let mut tampered = proof_record;
        tampered.provider_proof_hash ^= 1;
        tampered.record_hash = tampered.expected_record_hash();
        assert!(verify_verification_proof_record_bindings(&[tampered], &[binding]).is_err());
    }

    #[test]
    fn generic_verification_proof_replay_rejects_missing_duplicate_and_displaced_events() {
        let initial = State::default();
        let (_, tlog) = run_until_done(initial, RuntimeConfig::default()).unwrap();
        let receipt_event = tlog.first().copied().unwrap();
        let proof_event_seq = tlog.last().map(|event| event.seq + 1).unwrap();
        let binding = VerificationProofBinding::new(
            ProofSubjectKind::SemanticVerification,
            0xabc1,
            0xabc2,
            receipt_event.seq,
            receipt_event.self_hash,
            0xabc3,
        )
        .unwrap();
        let proof_record = VerificationProofRecord::from_binding(
            binding,
            0xabc4,
            proof_event_seq,
            0xabc5,
            crate::capability::verification::PROOF_FLAGS_REQUIRED,
        )
        .unwrap();

        let base = std::env::temp_dir().join(format!(
            "ai-generic-proof-replay-{}-{}",
            std::process::id(),
            proof_record.record_hash
        ));
        let valid_path = base.with_extension("valid.ndjson");
        let missing_path = base.with_extension("missing.ndjson");
        let duplicate_path = base.with_extension("duplicate.ndjson");
        let displaced_path = base.with_extension("displaced.ndjson");
        for path in [&valid_path, &missing_path, &duplicate_path, &displaced_path] {
            std::fs::remove_file(path).ok();
        }

        write_tlog_ndjson(&valid_path, &tlog).unwrap();
        crate::capability::verification::append_verification_proof_record_ndjson(
            &valid_path,
            &proof_record,
        )
        .unwrap();
        assert_eq!(
            verify_verification_proof_record_replay_ndjson(&valid_path, &[binding]).unwrap(),
            1
        );
        assert_eq!(
            verify_verification_proof_record_order_ndjson(&valid_path).unwrap(),
            1
        );

        write_tlog_ndjson(&missing_path, &tlog).unwrap();
        assert!(verify_verification_proof_record_replay_ndjson(
            &missing_path,
            &[binding]
        )
        .is_err());

        write_tlog_ndjson(&duplicate_path, &tlog).unwrap();
        crate::capability::verification::append_verification_proof_record_ndjson(
            &duplicate_path,
            &proof_record,
        )
        .unwrap();
        crate::capability::verification::append_verification_proof_record_ndjson(
            &duplicate_path,
            &proof_record,
        )
        .unwrap();
        assert!(verify_verification_proof_record_replay_ndjson(
            &duplicate_path,
            &[binding]
        )
        .is_err());

        write_tlog_ndjson(&displaced_path, &tlog).unwrap();
        {
            use std::io::Write as _;

            let mut displaced_event = receipt_event;
            displaced_event.seq = proof_event_seq;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&displaced_path)
                .unwrap();
            writeln!(
                file,
                "{}",
                crate::codec::ndjson::encode_control_event_ndjson(&displaced_event)
            )
            .unwrap();
            file.sync_all().unwrap();
        }
        crate::capability::verification::append_verification_proof_record_ndjson(
            &displaced_path,
            &proof_record,
        )
        .unwrap();
        assert!(verify_verification_proof_record_order_ndjson(&displaced_path).is_err());
        assert!(verify_verification_proof_record_replay_ndjson(
            &displaced_path,
            &[binding]
        )
        .is_err());

        for path in [&valid_path, &missing_path, &duplicate_path, &displaced_path] {
            std::fs::remove_file(path).ok();
        }
    }

    #[test]
    fn ollama_receipt_and_proof_hash_are_bidirectionally_bound() {
        let mut state = State::default();
        state.phase = Phase::Judgment;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);
        let policy = PolicyStore::default();
        let client = OllamaClient::new(OllamaConfig::default()).unwrap();
        let call = client
            .call_from_response_body(
                &context,
                &policy,
                "{\"id\":\"chatcmpl-1\",\"object\":\"chat.completion\",\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"Bind receipt and proof hashes both ways.\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":30,\"completion_tokens\":8,\"total_tokens\":38}}",
            )
            .unwrap();
        let envelope = CommandEnvelope::new(
            call.request_hash,
            crate::api::protocol::Command::SubmitEvidence(call.submission()),
        );

        let mut tlog = Vec::new();
        crate::api::routes::handle_envelope(&mut state, &mut tlog, RuntimeConfig::default(), envelope.clone())
            .unwrap();
        let persisted_event = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.evidence == Evidence::JudgmentRecord
                    && event.api_command_hash == envelope.command_hash
            })
            .copied()
            .unwrap();
        let base_receipt = call
            .receipt_for_configured_event(client.config(), envelope.command_hash, &persisted_event)
            .unwrap();

        assert!(!base_receipt.has_proof_binding());
        let (receipt, proof_event) = OllamaJudgmentProofEvent::finalize_receipt_after_tlog(
            base_receipt,
            &tlog,
            true,
            true,
            base_receipt.base_url_provenance_verified(client.config()),
            state.phase == Phase::Plan,
        )
        .unwrap();

        assert!(receipt.has_proof_binding());
        assert_eq!(receipt.proof_hash, proof_event.proof_hash);
        assert_ne!(receipt.receipt_hash, base_receipt.receipt_hash);
        assert!(proof_event.matches_receipt(receipt, &tlog));

        let mut tampered_receipt = receipt;
        tampered_receipt.proof_hash ^= 1;
        assert!(!tampered_receipt.is_valid());
        assert!(!proof_event.matches_receipt(tampered_receipt, &tlog));

        let mut tampered_proof = proof_event;
        tampered_proof.proof_hash ^= 1;
        assert!(!tampered_proof.is_valid());

        let mut mismatched_proof = proof_event;
        mismatched_proof.receipt_hash ^= 1;
        assert!(mismatched_proof.is_valid());
        assert!(!mismatched_proof.matches_receipt(receipt, &tlog));
    }


    #[test]
    fn ollama_receipt_hash_binds_proof_event_sequence_ordering() {
        let mut state = State::default();
        state.phase = Phase::Judgment;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);
        let policy = PolicyStore::default();
        let client = OllamaClient::new(OllamaConfig::default()).unwrap();
        let call = client
            .call_from_response_body(
                &context,
                &policy,
                "{\"id\":\"chatcmpl-1\",\"object\":\"chat.completion\",\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"Bind proof event sequence into the receipt hash.\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":30,\"completion_tokens\":8,\"total_tokens\":38}}",
            )
            .unwrap();
        let envelope = CommandEnvelope::new(
            call.request_hash,
            crate::api::protocol::Command::SubmitEvidence(call.submission()),
        );

        let mut tlog = Vec::new();
        crate::api::routes::handle_envelope(&mut state, &mut tlog, RuntimeConfig::default(), envelope.clone())
            .unwrap();
        let persisted_event = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.evidence == Evidence::JudgmentRecord
                    && event.api_command_hash == envelope.command_hash
            })
            .copied()
            .unwrap();
        let base_receipt = call
            .receipt_for_configured_event(client.config(), envelope.command_hash, &persisted_event)
            .unwrap();

        assert_eq!(base_receipt.proof_event_seq, 0);
        let base_hash = base_receipt.receipt_hash;
        let (receipt, proof_event) = OllamaJudgmentProofEvent::finalize_receipt_after_tlog(
            base_receipt,
            &tlog,
            true,
            true,
            base_receipt.base_url_provenance_verified(client.config()),
            state.phase == Phase::Plan,
        )
        .unwrap();

        assert_eq!(receipt.proof_event_seq, proof_event.proof_event_seq);
        assert_eq!(
            receipt.proof_event_seq,
            tlog.last().map(|event| event.seq + 1).unwrap()
        );
        assert!(receipt.proof_event_seq > receipt.event_seq);
        assert_ne!(receipt.receipt_hash, base_hash);
        assert!(proof_event.matches_receipt(receipt, &tlog));

        let mut tampered_receipt = receipt;
        tampered_receipt.proof_event_seq ^= 1;
        assert!(!tampered_receipt.is_valid());
        assert!(!proof_event.matches_receipt(tampered_receipt, &tlog));

        let mut tampered_proof = proof_event;
        tampered_proof.proof_event_seq ^= 1;
        assert!(!tampered_proof.is_valid());

        let mut mismatched_proof = proof_event;
        mismatched_proof.proof_event_seq = mismatched_proof.proof_event_seq.saturating_add(1);
        assert!(!mismatched_proof.is_valid());
        assert!(!mismatched_proof.matches_receipt(receipt, &tlog));
    }

    #[test]
    fn ollama_proof_replay_rejects_displaced_proof_event_sequence() {
        let mut state = State::default();
        state.phase = Phase::Judgment;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);
        let policy = PolicyStore::default();
        let client = OllamaClient::new(OllamaConfig::default()).unwrap();
        let call = client
            .call_from_response_body(
                &context,
                &policy,
                "{\"id\":\"chatcmpl-1\",\"object\":\"chat.completion\",\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"Reject displaced proof ordering.\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":30,\"completion_tokens\":8,\"total_tokens\":38}}",
            )
            .unwrap();
        let envelope = CommandEnvelope::new(
            call.request_hash,
            crate::api::protocol::Command::SubmitEvidence(call.submission()),
        );

        let mut tlog = Vec::new();
        crate::api::routes::handle_envelope(&mut state, &mut tlog, RuntimeConfig::default(), envelope.clone())
            .unwrap();
        let persisted_event = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.evidence == Evidence::JudgmentRecord
                    && event.api_command_hash == envelope.command_hash
            })
            .copied()
            .unwrap();
        let base_receipt = call
            .receipt_for_configured_event(client.config(), envelope.command_hash, &persisted_event)
            .unwrap();
        let (receipt, proof_event) = OllamaJudgmentProofEvent::finalize_receipt_after_tlog(
            base_receipt,
            &tlog,
            true,
            true,
            base_receipt.base_url_provenance_verified(client.config()),
            state.phase == Phase::Plan,
        )
        .unwrap();

        let valid_path = std::env::temp_dir().join(format!(
            "ai-ollama-proof-order-valid-{}-{}.ndjson",
            std::process::id(),
            proof_event.proof_hash
        ));
        let displaced_path = valid_path.with_file_name(format!(
            "ai-ollama-proof-order-displaced-{}-{}.ndjson",
            std::process::id(),
            proof_event.proof_hash
        ));
        std::fs::remove_file(&valid_path).ok();
        std::fs::remove_file(&displaced_path).ok();

        write_tlog_ndjson(&valid_path, &tlog).unwrap();
        append_ollama_llm_effect_receipt_ndjson(&valid_path, &receipt).unwrap();
        append_ollama_judgment_proof_event_ndjson(&valid_path, &proof_event).unwrap();
        assert_eq!(
            verify_ollama_judgment_proof_events_ndjson(&valid_path).unwrap(),
            1
        );

        write_tlog_ndjson(&displaced_path, &tlog).unwrap();
        append_ollama_llm_effect_receipt_ndjson(&displaced_path, &receipt).unwrap();
        {
            use std::io::Write as _;

            let mut displaced_event = persisted_event;
            displaced_event.seq = receipt.proof_event_seq;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(&displaced_path)
                .unwrap();
            writeln!(
                file,
                "{}",
                crate::codec::ndjson::encode_control_event_ndjson(&displaced_event)
            )
            .unwrap();
            file.sync_all().unwrap();
        }
        append_ollama_judgment_proof_event_ndjson(&displaced_path, &proof_event).unwrap();

        assert!(matches!(
            verify_ollama_judgment_proof_events_ndjson(&displaced_path),
            Err(OllamaError::InvalidReplay)
        ));

        std::fs::remove_file(&valid_path).ok();
        std::fs::remove_file(&displaced_path).ok();
    }

    #[test]
    fn ollama_receipt_proves_configured_local_endpoint_provenance() {
        let mut state = State::default();
        state.phase = Phase::Judgment;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);
        let policy = PolicyStore::default();
        let client = OllamaClient::new(OllamaConfig::default()).unwrap();
        let config = client.config().clone();
        let call = client
            .call_from_response_body(
                &context,
                &policy,
                "{\"id\":\"chatcmpl-1\",\"object\":\"chat.completion\",\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"Bind the endpoint provenance.\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":30,\"completion_tokens\":8,\"total_tokens\":38}}",
            )
            .unwrap();
        let envelope = CommandEnvelope::new(
            call.request_hash,
            crate::api::protocol::Command::SubmitEvidence(call.submission()),
        );

        let mut tlog = Vec::new();
        crate::api::routes::handle_envelope(&mut state, &mut tlog, RuntimeConfig::default(), envelope.clone())
            .unwrap();
        let persisted_event = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.evidence == Evidence::JudgmentRecord
                    && event.api_command_hash == envelope.command_hash
            })
            .copied()
            .unwrap();
        let receipt = call
            .receipt_for_configured_event(&config, envelope.command_hash, &persisted_event)
            .unwrap();

        assert!(config.validate().is_ok());
        assert!(call.base_url_provenance_verified(&config));
        assert_eq!(call.base_url_hash, config.base_url_id());
        assert_eq!(receipt.base_url_hash, config.base_url_id());
        assert!(receipt.base_url_provenance_verified(&config));

        let wrong_local_endpoint = OllamaConfig {
            base_url: "http://127.0.0.1:11435/v1".to_string(),
            model: config.model.clone(),
            timeout_ms: config.timeout_ms,
        };
        assert!(wrong_local_endpoint.validate().is_ok());
        assert!(!call.base_url_provenance_verified(&wrong_local_endpoint));
        assert!(call
            .receipt_for_configured_event(
                &wrong_local_endpoint,
                envelope.command_hash,
                &persisted_event
            )
            .is_none());
        assert_ne!(receipt.base_url_hash, wrong_local_endpoint.base_url_id());
        assert!(!receipt.base_url_provenance_verified(&wrong_local_endpoint));

        let wrong_model = OllamaConfig {
            base_url: config.base_url.clone(),
            model: "qwen2.5-coder:14b".to_string(),
            timeout_ms: config.timeout_ms,
        };
        assert!(wrong_model.validate().is_ok());
        assert!(!call.base_url_provenance_verified(&wrong_model));
        assert!(call
            .receipt_for_configured_event(&wrong_model, envelope.command_hash, &persisted_event)
            .is_none());
        assert_eq!(receipt.base_url_hash, wrong_model.base_url_id());
        assert!(!receipt.base_url_provenance_verified(&wrong_model));

        let non_local_endpoint = OllamaConfig {
            base_url: "https://example.com/v1".to_string(),
            model: config.model,
            timeout_ms: config.timeout_ms,
        };
        assert!(non_local_endpoint.validate().is_err());
        assert!(!call.base_url_provenance_verified(&non_local_endpoint));
        assert!(call
            .receipt_for_configured_event(
                &non_local_endpoint,
                envelope.command_hash,
                &persisted_event
            )
            .is_none());
        assert!(!receipt.base_url_provenance_verified(&non_local_endpoint));
    }

    #[test]
    fn ollama_receipt_construction_rejects_non_local_endpoint_before_receipt() {
        let mut state = State::default();
        state.phase = Phase::Judgment;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);
        let policy = PolicyStore::default();
        let client = OllamaClient::new(OllamaConfig::default()).unwrap();
        let call = client
            .call_from_response_body(
                &context,
                &policy,
                "{\"id\":\"chatcmpl-1\",\"object\":\"chat.completion\",\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"Reject non-local provenance before receipt.\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":30,\"completion_tokens\":8,\"total_tokens\":38}}",
            )
            .unwrap();
        let envelope = CommandEnvelope::new(
            call.request_hash,
            crate::api::protocol::Command::SubmitEvidence(call.submission()),
        );

        let mut tlog = Vec::new();
        crate::api::routes::handle_envelope(&mut state, &mut tlog, RuntimeConfig::default(), envelope.clone())
            .unwrap();
        let persisted_event = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.evidence == Evidence::JudgmentRecord
                    && event.api_command_hash == envelope.command_hash
            })
            .copied()
            .unwrap();

        let local_config = client.config().clone();
        assert!(call
            .receipt_for_configured_event(&local_config, envelope.command_hash, &persisted_event)
            .is_some());

        let non_local_endpoint = OllamaConfig {
            base_url: "http://192.168.1.2:11434/v1".to_string(),
            model: local_config.model,
            timeout_ms: local_config.timeout_ms,
        };
        assert!(non_local_endpoint.validate().is_err());
        assert!(!call.base_url_provenance_verified(&non_local_endpoint));
        assert!(call
            .receipt_for_configured_event(
                &non_local_endpoint,
                envelope.command_hash,
                &persisted_event
            )
            .is_none());
    }

    #[test]
    fn ollama_judgment_tlog_rejects_each_critical_receipt_field_tamper() {
        let mut state = State::default();
        state.phase = Phase::Judgment;
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);

        let mut memory = MemoryIndex::default();
        assert!(memory.insert(MemoryFact::new(state.packet.objective_id, 0xfeed, 7, 1)));
        let lookup = memory.lookup(state.packet.objective_id, 8);
        let context = ContextRecord::from_packet_memory(state.packet, 0xabc, &lookup);
        let policy = PolicyStore::default();
        let client = OllamaClient::new(OllamaConfig::default()).unwrap();
        let call = client
            .call_from_response_body(
                &context,
                &policy,
                "{\"id\":\"chatcmpl-1\",\"object\":\"chat.completion\",\"choices\":[{\"index\":0,\"message\":{\"role\":\"assistant\",\"content\":\"Persist this external llm receipt.\"},\"finish_reason\":\"stop\"}],\"usage\":{\"prompt_tokens\":30,\"completion_tokens\":8,\"total_tokens\":38}}",
            )
            .unwrap();
        let envelope = CommandEnvelope::new(
            call.request_hash,
            crate::api::protocol::Command::SubmitEvidence(call.submission()),
        );

        let mut tlog = Vec::new();
        crate::api::routes::handle_envelope(&mut state, &mut tlog, RuntimeConfig::default(), envelope.clone())
            .unwrap();
        let persisted_event = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.evidence == Evidence::JudgmentRecord
                    && event.api_command_hash == envelope.command_hash
            })
            .copied()
            .unwrap();
        let receipt = call
            .receipt_for_configured_event(client.config(), envelope.command_hash, &persisted_event)
            .unwrap();

        let path = std::env::temp_dir().join(format!(
            "ollama_judgment-{}-{}.tlog.ndjson",
            std::process::id(),
            receipt.receipt_hash
        ));
        std::fs::remove_file(&path).ok();

        write_tlog_ndjson(&path, &tlog).unwrap();
        append_ollama_llm_effect_receipt_ndjson(&path, &receipt).unwrap();
        assert_eq!(verify_ollama_judgment_tlog_ndjson(&path).unwrap(), 1);

        for (field_name, field_index) in [
            ("provider", 2usize),
            ("base_url", 3usize),
            ("model", 4usize),
            ("request_hash", 5usize),
            ("timeout_ms", 6usize),
            ("retry_count", 7usize),
            ("max_retries", 8usize),
            ("attempt_budget", 9usize),
            ("request_identity_hash", 10usize),
            ("retry_budget_hash", 11usize),
            ("budget_exhausted", 12usize),
            ("duplicate_request", 13usize),
            ("response_hash", 14usize),
            ("raw_response_hash", 15usize),
            ("proof_event_seq", 22usize),
            ("proof_hash", 23usize),
            ("receipt_hash", 24usize),
        ] {
            let tampered_path = path.with_file_name(format!(
                "ollama_judgment-{}-{}-{field_name}.tampered.tlog.ndjson",
                std::process::id(),
                receipt.receipt_hash
            ));
            std::fs::remove_file(&tampered_path).ok();
            tamper_first_ollama_receipt_field(&path, &tampered_path, field_index);
            assert!(
                matches!(
                    verify_ollama_judgment_tlog_ndjson(&tampered_path),
                    Err(OllamaError::InvalidReplay)
                ),
                "field {field_name} tamper should be rejected"
            );
            std::fs::remove_file(&tampered_path).ok();
        }

        std::fs::remove_file(&path).ok();
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
    fn api_protocol_schema_v4_binds_command_hash_to_payload() {
        assert_eq!(API_PROTOCOL_SCHEMA_VERSION, 4);

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
    fn tooling_effect_receipt_binds_request_receipt_and_tlog_event() {
        let mut state = State::default();
        state.phase = Phase::Execute;
        state.packet.bind_ready_task();
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
        state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);
        state.gates.plan = Gate::pass(Evidence::TaskReady);

        let mut tlog = Vec::new();
        let mut ledger = CommandLedger::default();
        let record = ToolExecutionRecord::from_packet(state.packet);
        let envelope = CommandEnvelope::new(101, Command::SubmitEvidence(record.submission()));
        let response = crate::api::routes::handle_envelope_once(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            &mut ledger,
            envelope.clone(),
        )
        .unwrap();

        assert_eq!(response.event.from, Phase::Execute);
        assert_eq!(response.event.to, Phase::Verify);
        assert_eq!(ledger.len(), 1);

        let persisted = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.cause == Cause::EvidenceSubmitted
                    && event.affected_gate == Some(GateId::Execution)
            })
            .unwrap();

        assert_eq!(persisted.api_command_id, envelope.command_id);
        assert_eq!(persisted.api_command_hash, envelope.command_hash);
        assert_eq!(response.event.api_command_id, envelope.command_id);
        assert_eq!(response.event.api_command_hash, envelope.command_hash);

        let effect_receipt = record.effect_receipt_for_event(persisted).unwrap();
        assert_eq!(effect_receipt.receipt_hash, record.receipt.receipt_hash);
        assert!(effect_receipt.replay_verified(&tlog));

        let proof_event_seq = effect_receipt.event_seq + 1;
        let proof_record = effect_receipt
            .to_verification_proof_record(proof_event_seq)
            .unwrap();
        let binding = effect_receipt
            .verification_proof_binding(proof_event_seq)
            .unwrap();
        assert_eq!(proof_record.subject, ProofSubjectKind::ArtifactEffect);
        assert!(proof_record.matches_binding(binding));
        assert_eq!(
            verify_verification_proof_record_bindings(&[proof_record], &[binding]).unwrap(),
            1
        );
        let mut tampered = proof_record;
        tampered.provider_proof_hash ^= 1;
        assert!(verify_verification_proof_record_bindings(&[tampered], &[binding]).is_err());
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn live_sandbox_tooling_writes_artifact_and_durable_receipt_replays() {
        let sandbox_root = std::env::temp_dir().join(format!(
            "canon-live-tool-{}-{}",
            std::process::id(),
            0x5eed_u64
        ));
        let _ = std::fs::remove_dir_all(&sandbox_root);
        let receipt_path = sandbox_root.join("tool_effect_receipts.ndjson");

        let mut state = State::default();
        state.phase = Phase::Execute;
        state.packet.bind_ready_task();
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
        state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);
        state.gates.plan = Gate::pass(Evidence::TaskReady);

        let mut tlog = Vec::new();
        let executor = LiveSandboxToolExecutor::new(&sandbox_root);
        let record = executor.execute_packet(state.packet).unwrap();
        let artifact_path = executor.artifact_path_for(record.request).unwrap();

        assert_eq!(record.request.tool_kind, ToolKind::SandboxFile);
        assert_eq!(record.decision(), ToolDecision::Succeeded);
        assert_eq!(record.receipt.effect.kind, ToolEffectKind::Artifact);
        assert_eq!(record.receipt.effect.digest, record.receipt.output_hash);
        assert!(record.receipt.is_sandbox_artifact_bound());
        assert!(artifact_path.exists());

        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            Command::SubmitEvidence(record.submission()),
        )
        .unwrap();

        let persisted = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.cause == Cause::EvidenceSubmitted
                    && event.affected_gate == Some(GateId::Execution)
            })
            .unwrap();
        let effect_receipt = record.effect_receipt_for_event(persisted).unwrap();

        assert!(effect_receipt.is_sandbox_artifact_bound());
        assert!(effect_receipt.replay_verified(&tlog));

        append_tool_effect_receipt_ndjson(&receipt_path, &effect_receipt).unwrap();
        let loaded = load_tool_effect_receipts_ndjson(&receipt_path).unwrap();

        assert_eq!(loaded, vec![effect_receipt]);
        assert_eq!(verify_tool_effect_receipts(&tlog, &loaded).unwrap(), 1);
        assert_eq!(
            decode_tool_effect_receipt_ndjson(&encode_tool_effect_receipt_ndjson(effect_receipt))
                .unwrap(),
            effect_receipt
        );
        verify_tlog(&tlog).unwrap();

        let _ = std::fs::remove_dir_all(&sandbox_root);
    }


    #[test]
    fn live_sandbox_process_runner_records_and_replays_receipt() {
        let sandbox_root = std::env::temp_dir().join(format!(
            "canon-live-process-{}-{}",
            std::process::id(),
            0x71e5_u64
        ));
        let _ = std::fs::remove_dir_all(&sandbox_root);
        let receipt_path = sandbox_root.join("process_receipts.ndjson");

        let executor = LiveSandboxProcessExecutor::new(&sandbox_root)
            .with_allowed_command("/usr/bin/printf")
            .with_locked_env("CANON_SANDBOX", "1")
            .with_timeout_ms(1000)
            .with_max_output_bytes(4096);

        let receipt = executor
            .execute_process("/usr/bin/printf", &["canon-process"], "")
            .unwrap();

        assert_eq!(receipt.exit_status, 0);
        assert!(!receipt.timed_out);
        assert_eq!(receipt.effect.kind, ToolEffectKind::Process);
        assert!(receipt.effect_is_normalized());
        assert!(executor
            .replay_receipt(&receipt, "/usr/bin/printf", &["canon-process"], "")
            .unwrap());

        append_sandbox_process_receipt_ndjson(&receipt_path, &receipt).unwrap();
        let loaded = load_sandbox_process_receipts_ndjson(&receipt_path).unwrap();

        assert_eq!(loaded, vec![receipt.clone()]);
        assert_eq!(
            verify_sandbox_process_receipts(
                &executor,
                &loaded,
                "/usr/bin/printf",
                &["canon-process"],
                ""
            )
            .unwrap(),
            1
        );
        assert_eq!(
            decode_sandbox_process_receipt_ndjson(&encode_sandbox_process_receipt_ndjson(&receipt))
                .unwrap(),
            receipt
        );

        let _ = std::fs::remove_dir_all(&sandbox_root);
    }


    #[test]
    fn process_receipt_enters_tlog_as_process_effect_without_artifact_leakage() {
        let sandbox_root = std::env::temp_dir().join(format!(
            "canon-live-process-effect-{}-{}",
            std::process::id(),
            0x9e11_u64
        ));
        let _ = std::fs::remove_dir_all(&sandbox_root);
        let receipt_path = sandbox_root.join("process_effect_receipts.ndjson");

        let executor = LiveSandboxProcessExecutor::new(&sandbox_root)
            .with_allowed_command("/usr/bin/printf")
            .with_locked_env("CANON_SANDBOX", "1")
            .with_timeout_ms(1000)
            .with_max_output_bytes(4096);

        let receipt = executor
            .execute_process("/usr/bin/printf", &["canon-process-effect"], "")
            .unwrap();

        let mut state = State::default();
        state.phase = Phase::Execute;
        state.packet.bind_ready_task();
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
        state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);
        state.gates.plan = Gate::pass(Evidence::TaskReady);

        let mut tlog = Vec::new();
        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            Command::SubmitProcessReceipt(receipt.clone()),
        )
        .unwrap();

        let persisted = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.cause == Cause::EvidenceSubmitted
                    && event.evidence == Evidence::ExecutionReceipt
                    && event.affected_gate == Some(GateId::Execution)
            })
            .unwrap();

        assert_eq!(persisted.state_before.packet, persisted.state_after.packet);
        assert_eq!(persisted.state_after.gates.execution.evidence, Evidence::ExecutionReceipt);
        assert_eq!(receipt.submission().effect, PacketEffect::None);

        let effect_receipt = ProcessEffectReceipt::from_persisted_event(&receipt, persisted).unwrap();
        assert_eq!(effect_receipt.effect.kind, ToolEffectKind::Process);
        assert!(effect_receipt.replay_verified(&tlog));

        let proof_event_seq = effect_receipt.event_seq + 1;
        let proof_record = effect_receipt
            .to_verification_proof_record(proof_event_seq)
            .unwrap();
        let binding = effect_receipt
            .verification_proof_binding(proof_event_seq)
            .unwrap();
        assert_eq!(proof_record.subject, ProofSubjectKind::ProcessEffect);
        assert!(proof_record.matches_binding(binding));
        assert_eq!(
            verify_verification_proof_record_bindings(&[proof_record], &[binding]).unwrap(),
            1
        );

        append_process_effect_receipt_ndjson(&receipt_path, &effect_receipt).unwrap();
        let loaded = load_process_effect_receipts_ndjson(&receipt_path).unwrap();

        assert_eq!(loaded, vec![effect_receipt]);
        assert_eq!(verify_process_effect_receipts(&tlog, &loaded).unwrap(), 1);
        assert_eq!(
            decode_process_effect_receipt_ndjson(&encode_process_effect_receipt_ndjson(
                effect_receipt
            ))
            .unwrap(),
            effect_receipt
        );
        verify_tlog(&tlog).unwrap();

        let _ = std::fs::remove_dir_all(&sandbox_root);
    }

    #[test]
    fn live_sandbox_process_runner_denies_unlisted_command() {
        let sandbox_root = std::env::temp_dir().join(format!(
            "canon-live-process-deny-{}-{}",
            std::process::id(),
            0xdec1_u64
        ));
        let _ = std::fs::remove_dir_all(&sandbox_root);

        let executor = LiveSandboxProcessExecutor::new(&sandbox_root)
            .with_allowed_command("printf");

        assert_eq!(
            executor.execute_process("sh", &["-c", "echo no"], ""),
            Err(ToolSandboxError::CommandDenied)
        );

        let _ = std::fs::remove_dir_all(&sandbox_root);
    }

    #[test]
    fn tooling_effect_receipt_rejects_tampered_packet_effect_replay() {
        let mut state = State::default();
        state.phase = Phase::Execute;
        state.packet.bind_ready_task();
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
        state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);
        state.gates.plan = Gate::pass(Evidence::TaskReady);

        let mut tlog = Vec::new();
        let record = ToolExecutionRecord::from_packet(state.packet);
        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            Command::SubmitEvidence(record.submission()),
        )
        .unwrap();

        let persisted = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.cause == Cause::EvidenceSubmitted
                    && event.affected_gate == Some(GateId::Execution)
            })
            .unwrap();
        let effect_receipt = record.effect_receipt_for_event(persisted).unwrap();

        let mut tampered = tlog.clone();
        tampered[0].state_after.packet.artifact_bytes =
            tampered[0].state_after.packet.artifact_bytes.saturating_add(1);

        assert!(!effect_receipt.replay_verified(&tampered));
        assert!(!record.verifies_persisted_event(&tampered[0]));
    }

    #[test]
    fn tooling_executor_denies_invalid_request_without_packet_effect() {
        let request = ToolRequest {
            capability: CapabilityId::Tooling,
            registry_policy_hash: CapabilityRegistry::canonical().policy_hash(),
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
    fn capability_registry_denies_unknown_routes_by_default() {
        let mut packet = Packet::empty();
        packet.bind_ready_task();

        let executor = DeterministicToolExecutor {
            max_input_hash: u64::MAX,
            registry: CapabilityRegistry::empty(),
        };
        let record = ToolExecutionRecord::from_packet_with_executor(packet, executor);
        let submission = record.submission();

        assert!(executor.registry.is_empty());
        assert_eq!(record.decision(), ToolDecision::Failed);
        assert!(submission.is_contract_valid());
        assert!(!submission.passed);
        assert_eq!(submission.effect, PacketEffect::None);
        assert!(!executor.registry.allows(CapabilityId::Tooling, submission));
    }

    #[test]
    fn capability_registry_allows_only_explicit_tool_effect() {
        let registry = CapabilityRegistry::canonical();
        let mut packet = Packet::empty();
        packet.bind_ready_task();

        let mut request = ToolRequest::from_packet(packet);
        assert!(registry.permits_effect(
            CapabilityId::Tooling,
            GateId::Execution,
            Evidence::ArtifactReceipt,
            PacketEffect::MaterializeArtifact
        ));
        assert!(registry.permits_effect(
            CapabilityId::Tooling,
            GateId::Execution,
            Evidence::ExecutionReceipt,
            PacketEffect::None
        ));
        assert!(!registry.permits_effect(
            CapabilityId::Tooling,
            GateId::Execution,
            Evidence::ArtifactReceipt,
            PacketEffect::RepairLineage
        ));

        request.requested_effect = PacketEffect::RepairLineage;
        let executor = DeterministicToolExecutor {
            max_input_hash: u64::MAX,
            registry,
        };
        let receipt = executor.execute(request);

        assert_eq!(receipt.exit_code, 126);
        assert!(!receipt.is_success_for(request));
    }

    #[test]
    fn capability_registry_policy_hash_is_execution_receipt_input() {
        let canonical = CapabilityRegistry::canonical();
        let drifted = CapabilityRegistry::empty();
        let mut packet = Packet::empty();
        packet.bind_ready_task();

        let request = ToolRequest::from_packet_with_registry(packet, canonical);
        let receipt = DeterministicToolExecutor::default().execute(request);

        assert_ne!(canonical.policy_hash(), drifted.policy_hash());
        assert_eq!(request.registry_policy_hash, canonical.policy_hash());
        assert_eq!(receipt.registry_policy_hash, canonical.policy_hash());
        assert!(receipt.is_success_for(request));
    }

    #[test]
    fn tooling_effect_receipt_rejects_registry_policy_drift() {
        let mut state = State::default();
        state.phase = Phase::Execute;
        state.packet.bind_ready_task();
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
        state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);
        state.gates.plan = Gate::pass(Evidence::TaskReady);

        let mut tlog = Vec::new();
        let record = ToolExecutionRecord::from_packet(state.packet);
        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            Command::SubmitEvidence(record.submission()),
        )
        .unwrap();

        let persisted = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.cause == Cause::EvidenceSubmitted
                    && event.affected_gate == Some(GateId::Execution)
            })
            .unwrap();
        let effect_receipt = record.effect_receipt_for_event(persisted).unwrap();

        assert_eq!(
            effect_receipt.registry_policy_hash,
            CapabilityRegistry::canonical().policy_hash()
        );
        assert!(effect_receipt
            .replay_verified_with_registry(&tlog, CapabilityRegistry::canonical()));
        assert!(!effect_receipt
            .replay_verified_with_registry(&tlog, CapabilityRegistry::empty()));
    }

    #[test]
    fn capability_registry_projection_is_persisted_in_execution_tlog() {
        let mut state = State::default();
        state.phase = Phase::Execute;
        state.packet.bind_ready_task();
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
        state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);
        state.gates.plan = Gate::pass(Evidence::TaskReady);

        let mut tlog = Vec::new();
        let record = ToolExecutionRecord::from_packet(state.packet);
        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            Command::SubmitEvidence(record.submission()),
        )
        .unwrap();

        let persisted = tlog
            .iter()
            .find(|event| {
                event.kind == EventKind::Persisted
                    && event.cause == Cause::EvidenceSubmitted
                    && event.affected_gate == Some(GateId::Execution)
            })
            .unwrap();

        assert_eq!(
            persisted.capability_registry_projection,
            CapabilityRegistry::canonical().projection()
        );

        let effect_receipt = record.effect_receipt_for_event(persisted).unwrap();
        assert!(effect_receipt.replay_verified(&tlog));
        verify_tlog(&tlog).unwrap();
    }

    #[test]
    fn tooling_effect_receipt_rejects_tlog_registry_projection_drift() {
        let mut state = State::default();
        state.phase = Phase::Execute;
        state.packet.bind_ready_task();
        state.gates.invariant = Gate::pass(Evidence::InvariantProof);
        state.gates.analysis = Gate::pass(Evidence::AnalysisReport);
        state.gates.judgment = Gate::pass(Evidence::JudgmentRecord);
        state.gates.plan = Gate::pass(Evidence::TaskReady);

        let mut tlog = Vec::new();
        let record = ToolExecutionRecord::from_packet(state.packet);
        crate::api::routes::handle_command(
            &mut state,
            &mut tlog,
            RuntimeConfig::default(),
            Command::SubmitEvidence(record.submission()),
        )
        .unwrap();

        let persisted_index = tlog
            .iter()
            .position(|event| {
                event.kind == EventKind::Persisted
                    && event.cause == Cause::EvidenceSubmitted
                    && event.affected_gate == Some(GateId::Execution)
            })
            .unwrap();
        let effect_receipt = record
            .effect_receipt_for_event(&tlog[persisted_index])
            .unwrap();

        let mut drifted = tlog.clone();
        drifted[persisted_index].capability_registry_projection =
            CapabilityRegistryProjection::new(1, CapabilityRegistry::empty().policy_hash());

        assert!(effect_receipt.replay_verified(&tlog));
        assert!(!effect_receipt.replay_verified(&drifted));
        assert_eq!(verify_tlog(&drifted), Err(CanonError::InvalidHashChain));
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
            capability_registry_projection: event.capability_registry_projection,
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
