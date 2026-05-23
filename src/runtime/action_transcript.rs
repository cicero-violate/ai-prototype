//! Runtime-owned action transcript log.
//!
//! The kernel TLog stores authorization and receipt evidence. This module stores
//! raw action request/result JSON outside the kernel as a hash-linked runtime log.

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::kernel::mix;
use crate::runtime::workspace::workspace_state_dir;

pub const ACTION_TRANSCRIPT_SCHEMA_VERSION: u64 = 1;
pub const ACTION_TRANSCRIPT_RECORD_CALL_RESULT: u64 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ActionTranscriptRecord {
    pub schema_version: u64,
    pub record_kind: u64,
    pub id: String,
    pub tool_name: String,
    pub request_json: String,
    pub response_json: String,
    pub request_hash: u64,
    pub receipt_hash: u64,
    pub response_hash: u64,
    pub response_bytes: u64,
    pub exit_status: u64,
    pub timed_out: bool,
    pub created_at: String,
    pub prev_hash: u64,
    pub self_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionTranscriptReceiptFacts {
    pub request_hash: u64,
    pub receipt_hash: u64,
    pub response_hash: u64,
    pub response_bytes: u64,
    pub exit_status: u64,
    pub timed_out: bool,
}

impl ActionTranscriptReceiptFacts {
    pub fn is_valid_for_response(self, response_json: &str) -> bool {
        self.request_hash != 0
            && self.receipt_hash != 0
            && self.response_hash != 0
            && self.response_bytes as usize == response_json.len()
    }
}

impl ActionTranscriptRecord {
    pub fn expected_self_hash(&self) -> u64 {
        let mut h = 0x4d43_5054_5241_4e01u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.record_kind);
        h = mix(h, string_hash(&self.id));
        h = mix(h, string_hash(&self.tool_name));
        h = mix(h, string_hash(&self.request_json));
        h = mix(h, string_hash(&self.response_json));
        h = mix(h, self.request_hash);
        h = mix(h, self.receipt_hash);
        h = mix(h, self.response_hash);
        h = mix(h, self.response_bytes);
        h = mix(h, self.exit_status);
        h = mix(h, self.timed_out as u64);
        h = mix(h, string_hash(&self.created_at));
        h = mix(h, self.prev_hash);
        h.max(1)
    }

    pub fn is_contract_valid(&self) -> bool {
        self.schema_version == ACTION_TRANSCRIPT_SCHEMA_VERSION
            && self.record_kind == ACTION_TRANSCRIPT_RECORD_CALL_RESULT
            && !self.id.is_empty()
            && !self.tool_name.is_empty()
            && !self.request_json.is_empty()
            && !self.response_json.is_empty()
            && self.request_hash != 0
            && self.receipt_hash != 0
            && self.response_hash != 0
            && self.response_bytes as usize == self.response_json.len()
            && !self.created_at.is_empty()
            && self.self_hash == self.expected_self_hash()
    }
}

pub fn append_action_transcript(
    workspace_root: &Path,
    tool_name: &str,
    request_json: &str,
    response_json: &str,
    receipt_facts: ActionTranscriptReceiptFacts,
) -> Result<ActionTranscriptRecord, String> {
    if tool_name.trim().is_empty() {
        return Err("tool_name is required".to_string());
    }
    if request_json.trim().is_empty() {
        return Err("request_json is required".to_string());
    }
    if response_json.trim().is_empty() {
        return Err("response_json is required".to_string());
    }
    if !receipt_facts.is_valid_for_response(response_json) {
        return Err("action transcript receipt facts do not match response".to_string());
    }

    let path = action_transcript_path(workspace_root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create transcript dir: {error}"))?;
    }

    let mut record = ActionTranscriptRecord {
        schema_version: ACTION_TRANSCRIPT_SCHEMA_VERSION,
        record_kind: ACTION_TRANSCRIPT_RECORD_CALL_RESULT,
        id: Uuid::new_v4().to_string(),
        tool_name: tool_name.to_string(),
        request_json: request_json.to_string(),
        response_json: response_json.to_string(),
        request_hash: receipt_facts.request_hash,
        receipt_hash: receipt_facts.receipt_hash,
        response_hash: receipt_facts.response_hash,
        response_bytes: receipt_facts.response_bytes,
        exit_status: receipt_facts.exit_status,
        timed_out: receipt_facts.timed_out,
        created_at: Utc::now().to_rfc3339(),
        prev_hash: action_transcript_last_hash_or_recover(workspace_root)?,
        self_hash: 0,
    };
    record.self_hash = record.expected_self_hash();
    if !record.is_contract_valid() {
        return Err("invalid action transcript record".to_string());
    }

    let line = serde_json::to_string(&record)
        .map_err(|error| format!("serialize action transcript: {error}"))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| format!("open action transcript {}: {error}", path.display()))?;
    writeln!(file, "{line}").map_err(|error| format!("write action transcript: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("sync action transcript: {error}"))?;
    Ok(record)
}

pub fn replay_action_transcripts(
    workspace_root: &Path,
) -> Result<Vec<ActionTranscriptRecord>, String> {
    let path = action_transcript_path(workspace_root);
    if !path.exists() {
        let legacy_path = legacy_mcp_transcript_path(workspace_root);
        if !legacy_path.exists() {
            return Ok(Vec::new());
        }
        return replay_action_transcripts_from_path(&legacy_path);
    }

    replay_action_transcripts_from_path(&path)
}

fn replay_action_transcripts_from_path(path: &Path) -> Result<Vec<ActionTranscriptRecord>, String> {
    let file = fs::File::open(&path).map_err(|error| format!("open action transcript: {error}"))?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut prev_hash = 0_u64;

    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|error| format!("read action transcript line {idx}: {error}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let record = decode_action_transcript_record(&line)
            .map_err(|error| format!("decode action transcript line {idx}: {error}"))?;
        if record.prev_hash != prev_hash || !record.is_contract_valid() {
            return Err(format!(
                "invalid action transcript hash chain at line {idx}"
            ));
        }
        prev_hash = record.self_hash;
        records.push(record);
    }

    Ok(records)
}

fn decode_action_transcript_record(line: &str) -> Result<ActionTranscriptRecord, String> {
    let value: serde_json::Value = serde_json::from_str(line).map_err(|error| error.to_string())?;
    let field = |name: &str| {
        value
            .get(name)
            .ok_or_else(|| format!("missing field `{name}`"))
    };
    let string = |name: &str| {
        field(name)?
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| format!("field `{name}` must be a string"))
    };
    let u64_field = |name: &str| {
        field(name)?
            .as_u64()
            .ok_or_else(|| format!("field `{name}` must be an unsigned integer"))
    };
    let bool_field = |name: &str| {
        field(name)?
            .as_bool()
            .ok_or_else(|| format!("field `{name}` must be a boolean"))
    };

    Ok(ActionTranscriptRecord {
        schema_version: u64_field("schema_version")?,
        record_kind: u64_field("record_kind")?,
        id: string("id")?,
        tool_name: string("tool_name")?,
        request_json: string("request_json")?,
        response_json: string("response_json")?,
        request_hash: u64_field("request_hash")?,
        receipt_hash: u64_field("receipt_hash")?,
        response_hash: u64_field("response_hash")?,
        response_bytes: u64_field("response_bytes")?,
        exit_status: u64_field("exit_status")?,
        timed_out: bool_field("timed_out")?,
        created_at: string("created_at")?,
        prev_hash: u64_field("prev_hash")?,
        self_hash: u64_field("self_hash")?,
    })
}

pub fn action_transcript_path(workspace_root: &Path) -> PathBuf {
    workspace_state_dir(workspace_root)
        .join("agent_state")
        .join("action")
        .join("action-transcript.tlog.ndjson")
}

pub(crate) fn legacy_mcp_transcript_path(workspace_root: &Path) -> PathBuf {
    workspace_state_dir(workspace_root)
        .join("agent_state")
        .join("mcp")
        .join("mcp-transcript.tlog.ndjson")
}

fn action_transcript_last_hash(workspace_root: &Path) -> Result<u64, String> {
    Ok(replay_action_transcripts(workspace_root)?
        .last()
        .map(|record| record.self_hash)
        .unwrap_or(0))
}

fn action_transcript_last_hash_or_recover(workspace_root: &Path) -> Result<u64, String> {
    match action_transcript_last_hash(workspace_root) {
        Ok(hash) => Ok(hash),
        Err(error) => {
            let path = action_transcript_path(workspace_root);
            if !path.exists() {
                return Ok(0);
            }
            let timestamp = Utc::now()
                .format("%Y%m%dT%H%M%S%.fZ")
                .to_string()
                .replace('.', "");
            let quarantine_path = path.with_extension(format!("tlog.ndjson.invalid-{timestamp}"));
            fs::rename(&path, &quarantine_path).map_err(|rename_error| {
                format!(
                    "{error}; failed to quarantine invalid action transcript {} to {}: {rename_error}",
                    path.display(),
                    quarantine_path.display()
                )
            })?;
            Ok(0)
        }
    }
}

fn string_hash(value: &str) -> u64 {
    let mut h = 0x4d43_5054_4841_5301u64;
    h = mix(h, value.len() as u64);
    for byte in value.as_bytes() {
        h = mix(h, *byte as u64);
    }
    h.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn unique_workspace() -> PathBuf {
        std::env::temp_dir().join(format!("canon-action-transcript-test-{}", Uuid::new_v4()))
    }

    fn receipt_facts(response: &str) -> ActionTranscriptReceiptFacts {
        ActionTranscriptReceiptFacts {
            request_hash: 0xA11C_E001,
            receipt_hash: 0xA11C_E002,
            response_hash: 0xA11C_E003,
            response_bytes: response.len() as u64,
            exit_status: 0,
            timed_out: false,
        }
    }

    #[test]
    fn action_transcript_appends_and_replays_hash_chain() {
        let workspace = unique_workspace();
        let response = r#"{"content":[{"type":"text","text":"hello"}],"isError":false}"#;

        let record = append_action_transcript(
            &workspace,
            "echo",
            r#"{"text":"hello"}"#,
            response,
            receipt_facts(response),
        )
        .expect("append transcript");
        assert!(record.is_contract_valid());

        let records = replay_action_transcripts(&workspace).expect("replay transcript");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].receipt_hash, receipt_facts(response).receipt_hash);
        assert_eq!(records[0].response_json, response);

        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn mcp_transcript_rejects_tampered_hash_chain() {
        let workspace = unique_workspace();
        let response = r#"{"content":[{"type":"text","text":"hello"}],"isError":false}"#;
        append_action_transcript(
            &workspace,
            "echo",
            r#"{"text":"hello"}"#,
            response,
            receipt_facts(response),
        )
        .expect("append transcript");

        let path = action_transcript_path(&workspace);
        let tampered = fs::read_to_string(&path)
            .expect("read transcript")
            .replace("hello", "tampered");
        fs::write(&path, tampered).expect("write transcript");

        assert!(replay_action_transcripts(&workspace).is_err());
        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn mcp_transcript_append_quarantines_invalid_existing_chain() {
        let workspace = unique_workspace();
        let response =
            r#"{\"content\":[{\"type\":\"text\",\"text\":\"hello\"}],\"isError\":false}"#;
        append_action_transcript(
            &workspace,
            "echo",
            r#"{\"text\":\"hello\"}"#,
            response,
            receipt_facts(response),
        )
        .expect("append transcript");

        let path = action_transcript_path(&workspace);
        let tampered = fs::read_to_string(&path)
            .expect("read transcript")
            .replace("hello", "tampered");
        fs::write(&path, tampered).expect("write transcript");

        let recovered = append_action_transcript(
            &workspace,
            "echo",
            r#"{\"text\":\"hello\"}"#,
            response,
            receipt_facts(response),
        )
        .expect("append after invalid existing transcript");
        assert_eq!(recovered.prev_hash, 0);
        let records = replay_action_transcripts(&workspace).expect("replay recovered transcript");
        assert_eq!(records.len(), 1);

        let quarantine_count = fs::read_dir(path.parent().expect("transcript parent"))
            .expect("read transcript dir")
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().contains("invalid-"))
            .count();
        assert_eq!(quarantine_count, 1);
        let _ = fs::remove_dir_all(workspace);
    }
}
