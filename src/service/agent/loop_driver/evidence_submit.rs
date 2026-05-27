//! Agent-loop evidence submission helpers.

use std::fs;
use std::path::Path;

use serde_json::json;

use crate::{
    Command, CommandEnvelope, Evidence, EvidenceSubmission, EvidenceSubmissionDto, GateId,
    ObservationCursor, ObservationFrame, ObservationFrameKind, ObservationIngressBatch,
    PacketEffect,
};

use super::common::stable_agent_hash;
use super::http::post_json_local;

pub(super) fn submit_observation_ingress(
    command_url: &str,
    cycle_num: u64,
    source_id: u64,
    source_bytes: &[u8],
) {
    let goal_bytes = source_bytes;
    let source_hash = stable_agent_hash(goal_bytes);

    // Build a typed Observation batch from actual source content.
    let (batch, records_json) = if goal_bytes.is_empty() {
        let cursor = ObservationCursor {
            source_id,
            last_sequence: 0,
            last_observed_hash: 0,
        };
        (
            ObservationIngressBatch::empty(source_id, source_hash, cursor),
            serde_json::Value::Array(vec![]),
        )
    } else {
        let frame = ObservationFrame::from_payload(
            ObservationFrameKind::ExternalSignal,
            source_id,
            cycle_num,
            cycle_num, // tick proxy: non-zero, strictly increases with cycle
            goal_bytes,
        );
        let record = frame.record();
        let cursor = ObservationCursor {
            source_id,
            last_sequence: record.sequence,
            last_observed_hash: record.observed_hash,
        };
        let rec_json = json!({
            "source_id": record.source_id,
            "sequence": record.sequence,
            "observed_hash": record.observed_hash,
            "received_at_tick": record.received_at_tick,
        });
        (
            ObservationIngressBatch::accepted(source_id, source_hash, cursor, 0, vec![record]),
            serde_json::Value::Array(vec![rec_json]),
        )
    };

    let contract_passed = batch.is_contract_valid();
    let payload_hash = batch.contract_hash();
    let cursor = batch.cursor;
    let envelope = CommandEnvelope::new(payload_hash, Command::SubmitObservationIngress(batch));
    let body = json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitObservationIngress",
        "payload": {
            "source_id": source_id,
            "source_hash": source_hash,
            "cursor_source_id": cursor.source_id,
            "cursor_last_sequence": cursor.last_sequence,
            "cursor_last_observed_hash": cursor.last_observed_hash,
            "backlog_len": 0u64,
            "records": records_json,
        },
    });
    match post_json_local(command_url, &body) {
        Ok(s) => {
            eprintln!("agent: observation  cycle={cycle_num}  passed={contract_passed}  status={s}")
        }
        Err(e) => eprintln!("agent: observation failed  cycle={cycle_num}  {e}"),
    }
}

pub(super) fn submit_judgment_evidence(command_url: &str, content_hash: u64) {
    let analysis_hash = stable_agent_hash(format!("canon:analysis:from:{content_hash}").as_bytes());
    let analysis = EvidenceSubmission::with_payload(
        GateId::Analysis,
        Evidence::AnalysisReport,
        true,
        analysis_hash,
    );
    let judgment = EvidenceSubmission::with_payload(
        GateId::Judgment,
        Evidence::JudgmentRecord,
        true,
        content_hash,
    );
    let envelope = CommandEnvelope::new(
        content_hash,
        Command::SubmitEvidenceBatch(vec![analysis, judgment]),
    );
    let body = json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitEvidenceBatch",
        "payload": [
            EvidenceSubmissionDto {
                gate: "Analysis".to_string(),
                evidence: "AnalysisReport".to_string(),
                passed: true,
                effect: Some("None".to_string()),
                payload_hash: analysis_hash,
            },
            EvidenceSubmissionDto {
                gate: "Judgment".to_string(),
                evidence: "JudgmentRecord".to_string(),
                passed: true,
                effect: Some("None".to_string()),
                payload_hash: content_hash,
            },
        ],
    });
    match post_json_local(command_url, &body) {
        Ok(s) => eprintln!("agent: judgment  content_hash={content_hash}  status={s}"),
        Err(e) => eprintln!("agent: judgment failed  {e}"),
    }
}

pub(super) fn submit_eval_evidence(command_url: &str, score_hash: u64, cycle_num: u64) {
    let verify_hash =
        stable_agent_hash(format!("canon:verification:cycle:{cycle_num}:{score_hash}").as_bytes());
    let verification = EvidenceSubmission::with_effect_payload(
        GateId::Verification,
        Evidence::LineageProof,
        true,
        PacketEffect::RepairLineage,
        verify_hash,
    );
    let eval = EvidenceSubmission::with_effect_payload(
        GateId::Eval,
        Evidence::EvalScore,
        true,
        PacketEffect::CompleteObjective,
        score_hash,
    );
    let command_id =
        stable_agent_hash(format!("canon:eval:cycle:{cycle_num}:{score_hash}").as_bytes());
    let envelope = CommandEnvelope::new(
        command_id,
        Command::SubmitEvidenceBatch(vec![verification, eval]),
    );
    let body = json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitEvidenceBatch",
        "payload": [
            EvidenceSubmissionDto {
                gate: "Verification".to_string(),
                evidence: "LineageProof".to_string(),
                passed: true,
                effect: Some("RepairLineage".to_string()),
                payload_hash: verify_hash,
            },
            EvidenceSubmissionDto {
                gate: "Eval".to_string(),
                evidence: "EvalScore".to_string(),
                passed: true,
                effect: Some("CompleteObjective".to_string()),
                payload_hash: score_hash,
            },
        ],
    });
    match post_json_local(command_url, &body) {
        Ok(s) => eprintln!("agent: eval  cycle={cycle_num}  score_hash={score_hash}  status={s}"),
        Err(e) => eprintln!("agent: eval failed  cycle={cycle_num}  {e}"),
    }
}

/// Submit a failed InvariantProof evidence to the kernel command URL.
///
/// This triggers `InvariantBlocked → RecheckInvariant` recovery on the next
/// tick, routing the supervisor to the Recovery phase so the LLM can plan a
/// fix for the detected signal integrity violations.
pub(super) fn submit_invariant_failure(command_url: &str, violation_count: usize, cycle_num: u64) {
    let payload_hash = stable_agent_hash(
        format!("canon:signal-integrity:violations:{violation_count}:{cycle_num}").as_bytes(),
    );
    let submission = EvidenceSubmission::with_payload(
        GateId::Invariant,
        Evidence::InvariantProof,
        false,
        payload_hash,
    );
    let envelope =
        CommandEnvelope::new(payload_hash, Command::SubmitEvidenceBatch(vec![submission]));
    let body = json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitEvidenceBatch",
        "payload": [
            EvidenceSubmissionDto {
                gate: "Invariant".to_string(),
                evidence: "InvariantProof".to_string(),
                passed: false,
                effect: Some("None".to_string()),
                payload_hash,
            },
        ],
    });
    match post_json_local(command_url, &body) {
        Ok(s) => eprintln!(
            "agent: signal-integrity failure submitted  violations={violation_count}  cycle={cycle_num}  status={s}"
        ),
        Err(e) => eprintln!(
            "agent: signal-integrity failure submit failed  cycle={cycle_num}  {e}"
        ),
    }
}

pub(super) fn read_score_hash(working_dir: &Path) -> u64 {
    let bytes = fs::read(working_dir.join("SCORE_REPORT.md")).unwrap_or_default();
    if bytes.is_empty() {
        stable_agent_hash(b"canon:eval:no-score-report")
    } else {
        stable_agent_hash(&bytes)
    }
}
