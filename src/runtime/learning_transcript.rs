//! Hash-linked learning transcript writer.
//!
//! Records which learning artifacts were emitted and links each row to the TLog
//! event and receipt that justify it. Mirrors `action_transcript.rs`.
//!
//! Artifact files written under `state/learning/`:
//!   symbol_mutation_log.ndjson
//!   architectural_decisions_proposed.ndjson
//!   architectural_decisions_outcome.ndjson
//!   task_symbol_index.ndjson
//!
//! The transcript index itself is at `state/learning/learning-transcript.tlog.ndjson`.

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::capability::learning::artifact::{
    ArchDecisionOutcomeRecord, ArchDecisionProposedRecord, LearningArtifactKind,
    SymbolMutationRecord, TaskSymbolIndexRecord,
};
use crate::kernel::mix;
use crate::runtime::workspace::workspace_state_dir;

pub const LEARNING_TRANSCRIPT_SCHEMA_VERSION: u64 = 1;
pub const LEARNING_TRANSCRIPT_RECORD_KIND: u64 = 0x4c52_4e54_5253_4301;

// ── Transcript record ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LearningTranscriptRecord {
    pub schema_version: u64,
    pub record_kind: u64,
    pub id: String,
    pub artifact_kind: u8,
    pub artifact_path: String,
    pub artifact_hash: u64,
    pub event_hash: u64,
    pub receipt_hash: u64,
    pub created_at: String,
    pub prev_hash: u64,
    pub self_hash: u64,
}

impl LearningTranscriptRecord {
    pub fn expected_self_hash(&self) -> u64 {
        let mut h = 0x4c52_4e54_4841_5301u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.record_kind);
        h = mix(h, lt_string_hash(&self.id));
        h = mix(h, self.artifact_kind as u64);
        h = mix(h, lt_string_hash(&self.artifact_path));
        h = mix(h, self.artifact_hash);
        h = mix(h, self.event_hash);
        h = mix(h, self.receipt_hash);
        h = mix(h, lt_string_hash(&self.created_at));
        h = mix(h, self.prev_hash);
        h.max(1)
    }

    pub fn is_contract_valid(&self) -> bool {
        self.schema_version == LEARNING_TRANSCRIPT_SCHEMA_VERSION
            && self.record_kind == LEARNING_TRANSCRIPT_RECORD_KIND
            && !self.id.is_empty()
            && !self.artifact_path.is_empty()
            && self.artifact_hash != 0
            && !self.created_at.is_empty()
            && self.self_hash == self.expected_self_hash()
    }
}

// ── Public append helpers ─────────────────────────────────────────────────────

pub fn append_symbol_mutation(
    workspace_root: &Path,
    record: &SymbolMutationRecord,
    event_hash: u64,
) -> Result<LearningTranscriptRecord, String> {
    if !record.is_valid() {
        return Err("invalid symbol mutation record".to_string());
    }
    let path = symbol_mutation_log_path(workspace_root);
    append_artifact_ndjson(&path, record)?;
    append_learning_transcript(
        workspace_root,
        LearningArtifactKind::SymbolMutation,
        &path.to_string_lossy(),
        record.record_hash,
        event_hash,
        record.receipt_hash,
    )
}

pub fn append_arch_decision_proposed(
    workspace_root: &Path,
    record: &ArchDecisionProposedRecord,
    event_hash: u64,
) -> Result<LearningTranscriptRecord, String> {
    if !record.is_valid() {
        return Err("invalid arch decision proposed record".to_string());
    }
    let path = arch_decisions_proposed_path(workspace_root);
    append_artifact_ndjson(&path, record)?;
    append_learning_transcript(
        workspace_root,
        LearningArtifactKind::ArchDecisionProposed,
        &path.to_string_lossy(),
        record.record_hash,
        event_hash,
        0,
    )
}

pub fn append_arch_decision_outcome(
    workspace_root: &Path,
    record: &ArchDecisionOutcomeRecord,
    event_hash: u64,
) -> Result<LearningTranscriptRecord, String> {
    if !record.is_valid() {
        return Err("invalid arch decision outcome record".to_string());
    }
    let path = arch_decisions_outcome_path(workspace_root);
    append_artifact_ndjson(&path, record)?;
    append_learning_transcript(
        workspace_root,
        LearningArtifactKind::ArchDecisionOutcome,
        &path.to_string_lossy(),
        record.record_hash,
        event_hash,
        record.outcome_receipt_hash,
    )
}

pub fn append_task_symbol_index(
    workspace_root: &Path,
    record: &TaskSymbolIndexRecord,
    event_hash: u64,
) -> Result<LearningTranscriptRecord, String> {
    if !record.is_valid() {
        return Err("invalid task symbol index record".to_string());
    }
    let path = task_symbol_index_path(workspace_root);
    append_artifact_ndjson(&path, record)?;
    append_learning_transcript(
        workspace_root,
        LearningArtifactKind::TaskSymbolIndex,
        &path.to_string_lossy(),
        record.record_hash,
        event_hash,
        0,
    )
}

// ── Semantic index join ───────────────────────────────────────────────────────

/// Look up all `def_path` values for symbols whose source file matches `file_path`.
///
/// Scans `state/rustc/*/semantic_index.jsonl`. Checks both `symbol_def` records
/// (via `span.file`) and `refactor_cost_profile` records (via top-level `file`).
/// Returns an empty vec if no match or on any I/O error — callers treat this as
/// a non-fatal miss.
pub fn resolve_def_paths_for_file(workspace_root: &Path, file_path: &str) -> Vec<String> {
    let rustc_dir = workspace_state_dir(workspace_root).join("rustc");
    let Ok(entries) = fs::read_dir(&rustc_dir) else {
        return Vec::new();
    };

    let mut def_paths = Vec::new();
    for entry in entries.flatten() {
        let idx = entry.path().join("semantic_index.jsonl");
        if !idx.exists() {
            continue;
        }
        let Ok(file) = fs::File::open(&idx) else {
            continue;
        };
        for line in BufReader::new(file).lines().flatten() {
            if line.trim().is_empty() {
                continue;
            }
            let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) else {
                continue;
            };
            let kind = value.get("kind").and_then(|v| v.as_str()).unwrap_or("");
            let dp = match kind {
                "symbol_def" => {
                    let span_file = value
                        .pointer("/span/file")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if span_file != file_path {
                        continue;
                    }
                    value.get("def_path").and_then(|v| v.as_str()).unwrap_or("")
                }
                "refactor_cost_profile" => {
                    let f = value.get("file").and_then(|v| v.as_str()).unwrap_or("");
                    if f != file_path {
                        continue;
                    }
                    value.get("def_path").and_then(|v| v.as_str()).unwrap_or("")
                }
                _ => continue,
            };
            if !dp.is_empty() && !def_paths.contains(&dp.to_string()) {
                def_paths.push(dp.to_string());
            }
        }
    }
    def_paths
}

// ── Artifact file paths ───────────────────────────────────────────────────────

pub fn learning_transcript_path(workspace_root: &Path) -> PathBuf {
    learning_dir(workspace_root).join("learning-transcript.tlog.ndjson")
}

pub fn symbol_mutation_log_path(workspace_root: &Path) -> PathBuf {
    learning_dir(workspace_root).join("symbol_mutation_log.ndjson")
}

pub fn arch_decisions_proposed_path(workspace_root: &Path) -> PathBuf {
    learning_dir(workspace_root).join("architectural_decisions_proposed.ndjson")
}

pub fn arch_decisions_outcome_path(workspace_root: &Path) -> PathBuf {
    learning_dir(workspace_root).join("architectural_decisions_outcome.ndjson")
}

pub fn task_symbol_index_path(workspace_root: &Path) -> PathBuf {
    learning_dir(workspace_root).join("task_symbol_index.ndjson")
}

fn learning_dir(workspace_root: &Path) -> PathBuf {
    workspace_state_dir(workspace_root).join("learning")
}

// ── Replay ────────────────────────────────────────────────────────────────────

pub fn replay_learning_transcripts(
    workspace_root: &Path,
) -> Result<Vec<LearningTranscriptRecord>, String> {
    let path = learning_transcript_path(workspace_root);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file =
        fs::File::open(&path).map_err(|e| format!("open learning transcript: {e}"))?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut prev_hash = 0_u64;

    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("read learning transcript line {idx}: {e}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let record = decode_learning_transcript_record(&line)
            .map_err(|e| format!("decode learning transcript line {idx}: {e}"))?;
        if record.prev_hash != prev_hash || !record.is_contract_valid() {
            return Err(format!(
                "invalid learning transcript hash chain at line {idx}"
            ));
        }
        prev_hash = record.self_hash;
        records.push(record);
    }
    Ok(records)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn append_learning_transcript(
    workspace_root: &Path,
    kind: LearningArtifactKind,
    artifact_path: &str,
    artifact_hash: u64,
    event_hash: u64,
    receipt_hash: u64,
) -> Result<LearningTranscriptRecord, String> {
    let tlog_path = learning_transcript_path(workspace_root);
    if let Some(parent) = tlog_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("create learning transcript dir: {e}"))?;
    }

    let mut record = LearningTranscriptRecord {
        schema_version: LEARNING_TRANSCRIPT_SCHEMA_VERSION,
        record_kind: LEARNING_TRANSCRIPT_RECORD_KIND,
        id: Uuid::new_v4().to_string(),
        artifact_kind: kind as u8,
        artifact_path: artifact_path.to_string(),
        artifact_hash,
        event_hash,
        receipt_hash,
        created_at: Utc::now().to_rfc3339(),
        prev_hash: learning_transcript_last_hash_or_recover(workspace_root)?,
        self_hash: 0,
    };
    record.self_hash = record.expected_self_hash();
    if !record.is_contract_valid() {
        return Err("invalid learning transcript record".to_string());
    }

    let line = serde_json::to_string(&record)
        .map_err(|e| format!("serialize learning transcript: {e}"))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&tlog_path)
        .map_err(|e| format!("open learning transcript {}: {e}", tlog_path.display()))?;
    writeln!(file, "{line}").map_err(|e| format!("write learning transcript: {e}"))?;
    file.sync_all()
        .map_err(|e| format!("sync learning transcript: {e}"))?;
    Ok(record)
}

fn append_artifact_ndjson<T: Serialize>(path: &Path, record: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create artifact dir: {e}"))?;
    }
    let line =
        serde_json::to_string(record).map_err(|e| format!("serialize artifact: {e}"))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("open artifact {}: {e}", path.display()))?;
    writeln!(file, "{line}").map_err(|e| format!("write artifact: {e}"))?;
    file.sync_all()
        .map_err(|e| format!("sync artifact: {e}"))?;
    Ok(())
}

fn learning_transcript_last_hash(workspace_root: &Path) -> Result<u64, String> {
    Ok(replay_learning_transcripts(workspace_root)?
        .last()
        .map(|r| r.self_hash)
        .unwrap_or(0))
}

fn learning_transcript_last_hash_or_recover(workspace_root: &Path) -> Result<u64, String> {
    match learning_transcript_last_hash(workspace_root) {
        Ok(hash) => Ok(hash),
        Err(error) => {
            let path = learning_transcript_path(workspace_root);
            if !path.exists() {
                return Ok(0);
            }
            let timestamp = Utc::now()
                .format("%Y%m%dT%H%M%S%.fZ")
                .to_string()
                .replace('.', "");
            let quarantine =
                path.with_extension(format!("tlog.ndjson.invalid-{timestamp}"));
            fs::rename(&path, &quarantine).map_err(|rename_error| {
                format!(
                    "{error}; failed to quarantine invalid learning transcript {} to {}: {rename_error}",
                    path.display(),
                    quarantine.display()
                )
            })?;
            Ok(0)
        }
    }
}

fn decode_learning_transcript_record(line: &str) -> Result<LearningTranscriptRecord, String> {
    let v: serde_json::Value =
        serde_json::from_str(line).map_err(|e| e.to_string())?;
    let u64_field = |name: &str| {
        v.get(name)
            .and_then(|x| x.as_u64())
            .ok_or_else(|| format!("field `{name}` must be a u64"))
    };
    let str_field = |name: &str| {
        v.get(name)
            .and_then(|x| x.as_str())
            .map(str::to_owned)
            .ok_or_else(|| format!("field `{name}` must be a string"))
    };

    Ok(LearningTranscriptRecord {
        schema_version: u64_field("schema_version")?,
        record_kind: u64_field("record_kind")?,
        id: str_field("id")?,
        artifact_kind: u64_field("artifact_kind")? as u8,
        artifact_path: str_field("artifact_path")?,
        artifact_hash: u64_field("artifact_hash")?,
        event_hash: u64_field("event_hash")?,
        receipt_hash: u64_field("receipt_hash")?,
        created_at: str_field("created_at")?,
        prev_hash: u64_field("prev_hash")?,
        self_hash: u64_field("self_hash")?,
    })
}

fn lt_string_hash(value: &str) -> u64 {
    let mut h = 0x4c52_4e48_4153_4c01u64;
    h = mix(h, value.len() as u64);
    for byte in value.as_bytes() {
        h = mix(h, *byte as u64);
    }
    h.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::learning::artifact::SymbolMutationRecord;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn unique_workspace() -> PathBuf {
        std::env::temp_dir()
            .join(format!("canon-learning-transcript-test-{}", Uuid::new_v4()))
    }

    #[test]
    fn learning_transcript_appends_and_replays_hash_chain() {
        let workspace = unique_workspace();
        let record = SymbolMutationRecord::new(
            1_000_000,
            1,
            0xDEAD_BEEF,
            0xCAFE_BABE,
            "ai/src/lib.rs".to_string(),
            vec!["ai::lib::foo".to_string()],
            true,
            0,
            0xA11C_E001,
        );
        assert!(record.is_valid());

        let transcript = append_symbol_mutation(&workspace, &record, 0x0001_0001)
            .expect("append symbol mutation");
        assert!(transcript.is_contract_valid());
        assert_eq!(transcript.artifact_hash, record.record_hash);
        assert_eq!(transcript.prev_hash, 0);

        let replayed = replay_learning_transcripts(&workspace).expect("replay");
        assert_eq!(replayed.len(), 1);
        assert_eq!(replayed[0].artifact_hash, record.record_hash);

        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn learning_transcript_hash_chain_links_successive_records() {
        let workspace = unique_workspace();

        let make_record = |ts: u64| {
            SymbolMutationRecord::new(
                ts, 1, 0xABCD, 0xEF01,
                "ai/src/lib.rs".to_string(),
                vec![],
                true, 0, ts,
            )
        };

        let r1 = make_record(1_000);
        let r2 = make_record(2_000);

        let t1 = append_symbol_mutation(&workspace, &r1, 0).expect("append 1");
        let t2 = append_symbol_mutation(&workspace, &r2, 0).expect("append 2");

        assert_eq!(t2.prev_hash, t1.self_hash);

        let replayed = replay_learning_transcripts(&workspace).expect("replay");
        assert_eq!(replayed.len(), 2);

        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn learning_transcript_quarantines_corrupt_chain() {
        let workspace = unique_workspace();
        let r = SymbolMutationRecord::new(
            1_000, 1, 0xDEAD, 0xBEEF,
            "ai/src/lib.rs".to_string(),
            vec![],
            true, 0, 0xABCD,
        );
        append_symbol_mutation(&workspace, &r, 0).expect("append");

        let path = learning_transcript_path(&workspace);
        let line = fs::read_to_string(&path).unwrap();
        let mut tampered: LearningTranscriptRecord =
            serde_json::from_str(line.trim()).expect("decode transcript for tamper");
        tampered.self_hash = tampered.self_hash.wrapping_add(1);
        fs::write(&path, format!("{}\n", serde_json::to_string(&tampered).unwrap())).unwrap();

        let r2 = SymbolMutationRecord::new(
            2_000, 1, 0xDEAD, 0xBEEF,
            "ai/src/other.rs".to_string(),
            vec![],
            true, 0, 0xBEEF,
        );
        let recovered = append_symbol_mutation(&workspace, &r2, 0)
            .expect("append after corrupt");
        assert_eq!(recovered.prev_hash, 0);

        let replayed = replay_learning_transcripts(&workspace).expect("replay recovered");
        assert_eq!(replayed.len(), 1);

        let _ = fs::remove_dir_all(workspace);
    }
}
