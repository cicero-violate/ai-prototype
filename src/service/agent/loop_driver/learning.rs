//! Post-cycle learning and policy promotion helpers.

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use chrono::Utc;
use serde_json::json;

use crate::capability::learning::artifact::{
    artifact_string_hash, TaskSymbolIndexRecord, TouchedSymbol,
};
use crate::runtime::learning_transcript::{append_task_symbol_index, symbol_mutation_log_path};
use crate::{
    load_tlog_ndjson, Command, CommandEnvelope, Evidence, EvidenceSubmission,
    EvidenceSubmissionDto, GateId, PolicyPromotion, PolicyStore, POLICY_FEEDBACK_HASH,
};

use super::http::post_json_local;

pub(super) fn run_post_cycle_learning(
    command_url: &str,
    tlog_path: &Path,
    policy_path: &Path,
    _working_dir: &Path,
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

pub(super) fn load_mcp_feedback(working_dir: &Path) -> Option<String> {
    fs::read_to_string(working_dir.join("state").join("mcp-feedback.md")).ok()
}

// ── Task symbol index ─────────────────────────────────────────────────────────

/// Emit a `TaskSymbolIndexRecord` at the end of a spawned agent's cycle.
///
/// Reads the symbol mutation log accumulated during the session, aggregates
/// touched symbols, and writes one index row for the completed task node.
/// Non-fatal: all errors are logged and suppressed so the agent can still exit.
pub(super) fn try_emit_task_symbol_index(project_dir: &Path, plan_node_id: &str, cycle_num: u64) {
    let (task_title, task_status) = read_task_metadata(project_dir, plan_node_id);
    let task_node_hash = artifact_string_hash(plan_node_id);

    let (symbols_touched, files_touched) = collect_symbols_and_files(project_dir);
    let tool_sequence = read_tool_sequence(project_dir);

    let ts = Utc::now().timestamp_millis() as u64;
    let record = TaskSymbolIndexRecord::new(
        ts,
        cycle_num,
        task_node_hash,
        task_title,
        task_status,
        symbols_touched,
        files_touched,
        tool_sequence,
    );

    if !record.is_valid() {
        eprintln!(
            "agent: task symbol index invalid — skipping  node={plan_node_id}  cycle={cycle_num}"
        );
        return;
    }

    match append_task_symbol_index(project_dir, &record, 0) {
        Ok(_) => {
            eprintln!("agent: task symbol index written  node={plan_node_id}  cycle={cycle_num}")
        }
        Err(e) => eprintln!(
            "agent: task symbol index failed  node={plan_node_id}  cycle={cycle_num}: {e}"
        ),
    }
}

fn read_task_metadata(project_dir: &Path, plan_node_id: &str) -> (String, String) {
    let plan_path = project_dir.join("state").join("plan.json");
    let Ok(text) = fs::read_to_string(&plan_path) else {
        return (plan_node_id.to_string(), "unknown".to_string());
    };
    let Ok(plan) = serde_json::from_str::<serde_json::Value>(&text) else {
        return (plan_node_id.to_string(), "unknown".to_string());
    };
    let nodes = plan.get("nodes").and_then(|v| v.as_array());
    let Some(nodes) = nodes else {
        return (plan_node_id.to_string(), "unknown".to_string());
    };
    for node in nodes {
        if node.get("id").and_then(|v| v.as_str()) == Some(plan_node_id) {
            let title = node
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or(plan_node_id)
                .to_string();
            let status = node
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            return (title, status);
        }
    }
    (plan_node_id.to_string(), "unknown".to_string())
}

/// Derive which layer a def_path belongs to from its module segments.
fn layer_from_def_path(def_path: &str) -> String {
    const LAYERS: &[&str] = &["kernel", "codec", "runtime", "capability", "service", "api"];
    for seg in def_path.split("::") {
        if LAYERS.contains(&seg) {
            return seg.to_string();
        }
    }
    "unknown".to_string()
}

fn collect_symbols_and_files(project_dir: &Path) -> (Vec<TouchedSymbol>, Vec<String>) {
    let path = symbol_mutation_log_path(project_dir);
    let Ok(file) = fs::File::open(&path) else {
        return (Vec::new(), Vec::new());
    };

    // mutation_count per def_path, files in order
    let mut symbol_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut files: Vec<String> = Vec::new();

    for line in BufReader::new(file).lines().flatten() {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        let Ok(rec) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        // Only count records that match the symbol_mutation schema
        if rec
            .get("schema")
            .and_then(|v| v.as_str())
            .map(|s| !s.starts_with("canon.learning.symbol_mutation"))
            .unwrap_or(true)
        {
            continue;
        }
        let file_path = rec.get("file").and_then(|v| v.as_str()).unwrap_or("");
        if !file_path.is_empty() && !files.contains(&file_path.to_string()) {
            files.push(file_path.to_string());
        }
        if let Some(def_paths) = rec.get("def_paths").and_then(|v| v.as_array()) {
            for dp in def_paths.iter().filter_map(|v| v.as_str()) {
                *symbol_counts.entry(dp.to_string()).or_insert(0) += 1;
            }
        }
    }

    let symbols = symbol_counts
        .into_iter()
        .map(|(def_path, mutation_count)| {
            let layer = layer_from_def_path(&def_path);
            TouchedSymbol {
                def_path,
                layer,
                mutation_count,
            }
        })
        .collect();

    (symbols, files)
}

fn read_tool_sequence(project_dir: &Path) -> Vec<String> {
    let path = project_dir.join("state").join("actions.ndjson");
    let Ok(file) = fs::File::open(&path) else {
        return Vec::new();
    };
    BufReader::new(file)
        .lines()
        .flatten()
        .filter_map(|line| {
            let v: serde_json::Value = serde_json::from_str(line.trim()).ok()?;
            v.get("tool_name")?.as_str().map(str::to_string)
        })
        .collect()
}
