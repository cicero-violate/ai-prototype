//! Runtime-owned action transcript log.
//!
//! The kernel TLog stores authorization and receipt evidence. This module stores
//! raw action request/result JSON outside the kernel as a hash-linked runtime log.

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::num::NonZeroU64;
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
    exit_status: u64,
    timed_out: bool,
    pub created_at: String,
    pub prev_hash: u64,
    pub self_hash: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionTranscriptOutcome {
    Success,
    NonZeroExit(NonZeroU64),
    TimedOut(NonZeroU64),
}

impl ActionTranscriptOutcome {
    pub const fn success() -> Self {
        Self::Success
    }

    pub const fn nonzero_exit(exit_status: NonZeroU64) -> Self {
        Self::NonZeroExit(exit_status)
    }

    pub const fn timed_out(exit_status: NonZeroU64) -> Self {
        Self::TimedOut(exit_status)
    }

    pub fn from_receipt(exit_status: u64, timed_out: bool) -> Result<Self, String> {
        if timed_out {
            return NonZeroU64::new(exit_status)
                .map(Self::TimedOut)
                .ok_or_else(|| {
                    "timed-out transcript outcomes require a nonzero exit status".to_string()
                });
        }

        match NonZeroU64::new(exit_status) {
            Some(status) => Ok(Self::NonZeroExit(status)),
            None => Ok(Self::Success),
        }
    }

    pub const fn exit_status(self) -> u64 {
        match self {
            Self::Success => 0,
            Self::NonZeroExit(status) | Self::TimedOut(status) => status.get(),
        }
    }

    pub const fn timed_out_flag(self) -> bool {
        matches!(self, Self::TimedOut(_))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActionTranscriptReceiptFacts {
    request_hash: u64,
    receipt_hash: u64,
    response_hash: u64,
    response_bytes: u64,
    outcome: ActionTranscriptOutcome,
}

impl ActionTranscriptReceiptFacts {
    pub fn new(
        request_hash: u64,
        receipt_hash: u64,
        response_hash: u64,
        response_bytes: u64,
        outcome: ActionTranscriptOutcome,
    ) -> Result<Self, String> {
        if request_hash == 0 {
            return Err("action transcript request_hash must be nonzero".to_string());
        }
        if receipt_hash == 0 {
            return Err("action transcript receipt_hash must be nonzero".to_string());
        }
        if response_hash == 0 {
            return Err("action transcript response_hash must be nonzero".to_string());
        }
        if response_bytes == 0 {
            return Err("action transcript response_bytes must be nonzero".to_string());
        }
        Ok(Self {
            request_hash,
            receipt_hash,
            response_hash,
            response_bytes,
            outcome,
        })
    }

    pub fn from_receipt(
        request_hash: u64,
        receipt_hash: u64,
        response_hash: u64,
        response_bytes: u64,
        exit_status: u64,
        timed_out: bool,
    ) -> Result<Self, String> {
        Self::new(
            request_hash,
            receipt_hash,
            response_hash,
            response_bytes,
            ActionTranscriptOutcome::from_receipt(exit_status, timed_out)?,
        )
    }

    pub fn is_valid_for_response(self, response_json: &str) -> bool {
        self.request_hash != 0
            && self.receipt_hash != 0
            && self.response_hash != 0
            && self.response_bytes as usize == response_json.len()
    }

    pub const fn exit_status(self) -> u64 {
        self.outcome.exit_status()
    }

    pub const fn timed_out(self) -> bool {
        self.outcome.timed_out_flag()
    }
}

impl ActionTranscriptRecord {
    pub fn outcome(&self) -> ActionTranscriptOutcome {
        ActionTranscriptOutcome::from_receipt(self.exit_status, self.timed_out)
            .expect("ActionTranscriptRecord stores a constructor-validated outcome")
    }

    pub const fn exit_status(&self) -> u64 {
        self.exit_status
    }

    pub const fn timed_out(&self) -> bool {
        self.timed_out
    }

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
            && ActionTranscriptOutcome::from_receipt(self.exit_status, self.timed_out).is_ok()
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
        exit_status: receipt_facts.exit_status(),
        timed_out: receipt_facts.timed_out(),
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

    let exit_status = u64_field("exit_status")?;
    let timed_out = bool_field("timed_out")?;
    let outcome = ActionTranscriptOutcome::from_receipt(exit_status, timed_out)?;

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
        exit_status: outcome.exit_status(),
        timed_out: outcome.timed_out_flag(),
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
    fn unique_workspace() -> (PathBuf, tempfile::TempDir) {
        let tmp = tempfile::Builder::new()
            .prefix("canon-action-transcript-test-")
            .tempdir_in({
                let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
                std::fs::create_dir_all(&d).unwrap();
                d
            })
            .unwrap();
        let path = tmp.path().to_path_buf();
        (path, tmp)
    }

    fn receipt_facts(response: &str) -> ActionTranscriptReceiptFacts {
        ActionTranscriptReceiptFacts::new(
            0xA11C_E001,
            0xA11C_E002,
            0xA11C_E003,
            response.len() as u64,
            ActionTranscriptOutcome::success(),
        )
        .expect("valid receipt facts")
    }

    fn nonzero(value: u64) -> NonZeroU64 {
        NonZeroU64::new(value).expect("nonzero test value")
    }

    fn append_with_outcome(
        workspace: &Path,
        response: &str,
        outcome: ActionTranscriptOutcome,
    ) -> ActionTranscriptRecord {
        let facts = ActionTranscriptReceiptFacts::new(
            0xA11C_E101,
            0xA11C_E102,
            0xA11C_E103,
            response.len() as u64,
            outcome,
        )
        .expect("valid receipt facts");
        append_action_transcript(
            workspace,
            "echo",
            r#"{\"text\":\"hello\"}"#,
            response,
            facts,
        )
        .expect("append transcript")
    }

    fn replay_error_for_mutated_first_record(
        workspace: &Path,
        mutate: impl FnOnce(&mut serde_json::Value),
    ) -> String {
        let path = action_transcript_path(workspace);
        let line = fs::read_to_string(&path).expect("read transcript");
        let mut value: serde_json::Value = serde_json::from_str(line.trim()).expect("json line");
        mutate(&mut value);
        fs::write(
            &path,
            serde_json::to_string(&value).expect("serialize mutated record"),
        )
        .expect("write mutated transcript");
        replay_action_transcripts(workspace).expect_err("mutated transcript rejected")
    }

    #[test]
    fn action_transcript_appends_and_replays_hash_chain() {
        let (workspace, _tmp) = unique_workspace();
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
        assert_eq!(
            records[0].receipt_hash,
            receipt_facts(response).receipt_hash
        );
        assert_eq!(records[0].response_json, response);
        assert_eq!(records[0].outcome(), ActionTranscriptOutcome::Success);
        assert_eq!(records[0].exit_status(), 0);
        assert!(!records[0].timed_out());
    }

    #[test]
    fn action_transcript_types_timeout_and_nonzero_exit_outcomes() {
        let (timeout_workspace, _timeout_tmp) = unique_workspace();
        let response = r#"{"content":[{"type":"text","text":"timeout"}],"isError":true}"#;
        let timeout = append_with_outcome(
            &timeout_workspace,
            response,
            ActionTranscriptOutcome::timed_out(nonzero(124)),
        );
        assert_eq!(timeout.exit_status(), 124);
        assert!(timeout.timed_out());
        assert_eq!(
            replay_action_transcripts(&timeout_workspace)
                .expect("replay timeout")
                .remove(0)
                .outcome(),
            ActionTranscriptOutcome::TimedOut(nonzero(124))
        );

        let (exit_workspace, _exit_tmp) = unique_workspace();
        let nonzero_exit = append_with_outcome(
            &exit_workspace,
            response,
            ActionTranscriptOutcome::nonzero_exit(nonzero(2)),
        );
        assert_eq!(nonzero_exit.exit_status(), 2);
        assert!(!nonzero_exit.timed_out());
        assert_eq!(
            replay_action_transcripts(&exit_workspace)
                .expect("replay nonzero")
                .remove(0)
                .outcome(),
            ActionTranscriptOutcome::NonZeroExit(nonzero(2))
        );
    }

    #[test]
    fn action_transcript_receipt_facts_reject_zero_hashes_and_mismatched_bytes() {
        let response = r#"{"ok":true}"#;
        assert!(ActionTranscriptReceiptFacts::new(
            0,
            0xA11C_E202,
            0xA11C_E203,
            response.len() as u64,
            ActionTranscriptOutcome::success(),
        )
        .is_err());
        assert!(ActionTranscriptReceiptFacts::new(
            0xA11C_E201,
            0,
            0xA11C_E203,
            response.len() as u64,
            ActionTranscriptOutcome::success(),
        )
        .is_err());
        assert!(ActionTranscriptReceiptFacts::new(
            0xA11C_E201,
            0xA11C_E202,
            0,
            response.len() as u64,
            ActionTranscriptOutcome::success(),
        )
        .is_err());
        assert!(ActionTranscriptReceiptFacts::new(
            0xA11C_E201,
            0xA11C_E202,
            0xA11C_E203,
            0,
            ActionTranscriptOutcome::success(),
        )
        .is_err());

        let (workspace, _tmp) = unique_workspace();
        let wrong_bytes = ActionTranscriptReceiptFacts::new(
            0xA11C_E201,
            0xA11C_E202,
            0xA11C_E203,
            response.len() as u64 + 1,
            ActionTranscriptOutcome::success(),
        )
        .expect("constructor accepts externally recovered byte fact");
        let err = append_action_transcript(
            &workspace,
            "echo",
            r#"{\"text\":\"hello\"}"#,
            response,
            wrong_bytes,
        )
        .expect_err("byte mismatch rejected at transcript boundary");
        assert!(err.contains("receipt facts do not match response"));
    }

    #[test]
    fn action_transcript_decode_rejects_inconsistent_timeout_status() {
        let (workspace, _tmp) = unique_workspace();
        let response = r#"{"content":[{"type":"text","text":"timeout"}],"isError":true}"#;
        append_with_outcome(
            &workspace,
            response,
            ActionTranscriptOutcome::nonzero_exit(nonzero(1)),
        );

        let err = replay_error_for_mutated_first_record(&workspace, |value| {
            value["timed_out"] = serde_json::Value::Bool(true);
            value["exit_status"] = serde_json::Value::Number(0_u64.into());
        });
        assert!(err.contains("timed-out transcript outcomes require a nonzero exit status"));
    }

    #[test]
    fn action_transcript_decode_rejects_malformed_fields() {
        let (workspace, _tmp) = unique_workspace();
        let response = r#"{"content":[{"type":"text","text":"hello"}],"isError":false}"#;
        append_with_outcome(&workspace, response, ActionTranscriptOutcome::success());

        let err = replay_error_for_mutated_first_record(&workspace, |value| {
            value["timed_out"] = serde_json::Value::String("false".to_string());
        });
        assert!(err.contains("field `timed_out` must be a boolean"));
    }

    #[test]
    fn action_transcript_replay_rejects_zero_receipt_hash_and_byte_count_mismatch() {
        let (zero_workspace, _zero_tmp) = unique_workspace();
        let response = r#"{"content":[{"type":"text","text":"hello"}],"isError":false}"#;
        append_with_outcome(
            &zero_workspace,
            response,
            ActionTranscriptOutcome::success(),
        );

        let zero_err = replay_error_for_mutated_first_record(&zero_workspace, |value| {
            value["receipt_hash"] = serde_json::Value::Number(0_u64.into());
            let record = decode_action_transcript_record(
                &serde_json::to_string(value).expect("serialize mutated value"),
            )
            .expect("decode mutated record");
            value["self_hash"] = serde_json::Value::Number(record.expected_self_hash().into());
        });
        assert!(zero_err.contains("invalid action transcript hash chain"));

        let (bytes_workspace, _bytes_tmp) = unique_workspace();
        append_with_outcome(
            &bytes_workspace,
            response,
            ActionTranscriptOutcome::success(),
        );
        let byte_err = replay_error_for_mutated_first_record(&bytes_workspace, |value| {
            value["response_bytes"] = serde_json::Value::Number((response.len() as u64 + 1).into());
            let record = decode_action_transcript_record(
                &serde_json::to_string(value).expect("serialize mutated value"),
            )
            .expect("decode mutated record");
            value["self_hash"] = serde_json::Value::Number(record.expected_self_hash().into());
        });
        assert!(byte_err.contains("invalid action transcript hash chain"));
    }

    #[test]
    fn action_transcript_rejects_tampered_hash_chain() {
        let (workspace, _tmp) = unique_workspace();
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
    }

    #[test]
    fn action_transcript_append_quarantines_invalid_existing_chain() {
        let (workspace, _tmp) = unique_workspace();
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
    }
}
