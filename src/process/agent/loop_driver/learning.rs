//! Post-cycle learning and policy promotion helpers.

use std::path::Path;

use serde_json::json;

use crate::{
    load_tlog_ndjson, Command, CommandEnvelope, Evidence, EvidenceSubmission,
    EvidenceSubmissionDto, GateId, PolicyPromotion, PolicyStore, POLICY_FEEDBACK_HASH,
};

use super::http::post_json_local;

pub(super) fn run_post_cycle_learning(
    command_url: &str,
    tlog_path: &Path,
    policy_path: &Path,
    cycle_num: u64,
) {
    let tlog = match load_tlog_ndjson(tlog_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("agent: learning  tlog read failed  cycle={cycle_num}  {e:?}");
            return;
        }
    };

    let mut store = PolicyStore::load_ndjson(policy_path).unwrap_or_default();
    let next_version = store.latest_version() + 1;

    let promotion = match PolicyPromotion::from_tlog(&tlog, next_version) {
        Some(p) => p,
        None => {
            eprintln!("agent: learning  no promotable pattern  cycle={cycle_num}");
            return;
        }
    };

    let payload_hash = promotion.promoted_policy_hash;
    let passed = promotion.is_valid();

    match store.promote_durable(policy_path, promotion) {
        Ok(entry) => {
            eprintln!(
                "agent: learning  promoted  version={}  source_seq={}  cycle={cycle_num}",
                entry.version, entry.value,
            );
        }
        Err(e) => {
            eprintln!("agent: learning  promote failed  cycle={cycle_num}  {e:?}");
            return;
        }
    }

    let submission = EvidenceSubmission::with_payload(
        GateId::Learning,
        Evidence::PolicyPromotion,
        passed,
        payload_hash,
    );
    let envelope = CommandEnvelope::new(payload_hash, Command::SubmitEvidence(submission));
    let body = json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitEvidence",
        "payload": EvidenceSubmissionDto {
            gate: "Learning".to_string(),
            evidence: "PolicyPromotion".to_string(),
            passed,
            effect: Some("None".to_string()),
            payload_hash,
        },
    });
    match post_json_local(command_url, &body) {
        Ok(s) => eprintln!("agent: policy promotion  cycle={cycle_num}  status={s}"),
        Err(e) => eprintln!("agent: policy promotion failed  cycle={cycle_num}  {e}"),
    }
}

pub(super) fn load_policy_feedback(working_dir: &Path) -> Option<String> {
    let policy_path = working_dir.join("state").join("policy.ndjson");
    let store = PolicyStore::load_ndjson(&policy_path).ok()?;
    if store.entries().is_empty() {
        return None;
    }
    let version = store.latest_version();
    let feedback_hash = store.latest_value(POLICY_FEEDBACK_HASH).unwrap_or_else(|| {
        store
            .latest_value(crate::POLICY_PROMOTION_SOURCE_SEQ)
            .unwrap_or(0)
    });
    Some(format!(
        "policy_version={version}  feedback_hash={feedback_hash:#018x}"
    ))
}
