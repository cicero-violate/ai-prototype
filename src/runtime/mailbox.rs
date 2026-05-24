//! Runtime-owned agent mailbox projection.
//!
//! Mailbox files are materialized runtime projections, not capability-owned
//! authoritative state. Tooling may request mailbox operations, but this
//! module owns the durable record shape, append path, replay/read path, and
//! semantic receipt contract.

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::kernel::mix;
use crate::runtime::workspace::workspace_state_dir;

pub const MAILBOX_TLOG_SCHEMA_VERSION: u64 = 1;
pub const MAILBOX_TLOG_RECORD_MESSAGE: u64 = 1;
pub const MAILBOX_MESSAGE_CAPABILITY_ID: u64 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MailboxMessageRequest {
    pub capability_id: u64,
    pub registry_policy_hash: u64,
    pub sender_hash: u64,
    pub target_hash: u64,
    pub kind_hash: u64,
    pub payload_hash: u64,
}

impl MailboxMessageRequest {
    pub fn new(
        registry_policy_hash: u64,
        sender: &str,
        target: &str,
        kind: &str,
        payload: &str,
    ) -> Result<Self, String> {
        validate_agent_id(sender)?;
        validate_agent_id(target)?;
        validate_message_kind(kind)?;
        Ok(Self {
            capability_id: MAILBOX_MESSAGE_CAPABILITY_ID,
            registry_policy_hash,
            sender_hash: string_hash(sender),
            target_hash: string_hash(target),
            kind_hash: string_hash(kind),
            payload_hash: string_hash(payload),
        })
    }

    pub fn is_admissible(self) -> bool {
        self.capability_id == MAILBOX_MESSAGE_CAPABILITY_ID
            && self.registry_policy_hash != 0
            && self.sender_hash != 0
            && self.target_hash != 0
            && self.kind_hash != 0
            && self.payload_hash != 0
    }

    pub fn contract_hash(self) -> u64 {
        let mut h = 0x4d42_584d_5347_0001u64;
        h = mix(h, self.capability_id);
        h = mix(h, self.registry_policy_hash);
        h = mix(h, self.sender_hash);
        h = mix(h, self.target_hash);
        h = mix(h, self.kind_hash);
        h = mix(h, self.payload_hash);
        h.max(1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MailboxMessageReceipt {
    pub request_hash: u64,
    pub registry_policy_hash: u64,
    pub sender_hash: u64,
    pub target_hash: u64,
    pub kind_hash: u64,
    pub payload_hash: u64,
    pub message_id_hash: u64,
    pub sent_at_hash: u64,
    pub record_hash: u64,
    pub receipt_hash: u64,
}

impl MailboxMessageReceipt {
    pub fn from_record(request: &MailboxMessageRequest, record: &MailboxMessageRecord) -> Self {
        let record_hash = record.contract_hash();
        let mut receipt = Self {
            request_hash: request.contract_hash(),
            registry_policy_hash: request.registry_policy_hash,
            sender_hash: request.sender_hash,
            target_hash: request.target_hash,
            kind_hash: request.kind_hash,
            payload_hash: request.payload_hash,
            message_id_hash: string_hash(&record.id),
            sent_at_hash: string_hash(&record.sent_at),
            record_hash,
            receipt_hash: 1,
        };
        receipt.receipt_hash = receipt.compute_receipt_hash();
        receipt
    }

    pub fn is_valid_for(self, request: &MailboxMessageRequest) -> bool {
        self.request_hash == request.contract_hash()
            && self.registry_policy_hash == request.registry_policy_hash
            && self.sender_hash == request.sender_hash
            && self.target_hash == request.target_hash
            && self.kind_hash == request.kind_hash
            && self.payload_hash == request.payload_hash
    }

    pub fn is_contract_valid(self) -> bool {
        self.registry_policy_hash != 0
            && self.request_hash != 0
            && self.sender_hash != 0
            && self.target_hash != 0
            && self.kind_hash != 0
            && self.payload_hash != 0
            && self.message_id_hash != 0
            && self.sent_at_hash != 0
            && self.record_hash != 0
            && self.receipt_hash == self.compute_receipt_hash()
    }

    pub fn contract_hash(self) -> u64 {
        self.compute_receipt_hash()
    }

    fn compute_receipt_hash(self) -> u64 {
        let mut h = 0x4d42_5852_4350_0001u64;
        h = mix(h, self.request_hash);
        h = mix(h, self.registry_policy_hash);
        h = mix(h, self.sender_hash);
        h = mix(h, self.target_hash);
        h = mix(h, self.kind_hash);
        h = mix(h, self.payload_hash);
        h = mix(h, self.message_id_hash);
        h = mix(h, self.sent_at_hash);
        h = mix(h, self.record_hash);
        h.max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailboxMessageRecord {
    #[serde(default)]
    pub schema_version: u64,
    #[serde(default)]
    pub record_kind: u64,
    pub id: String,
    pub sender: String,
    pub target: String,
    pub kind: String,
    pub payload: String,
    pub sent_at: String,
    #[serde(default)]
    pub prev_hash: u64,
    #[serde(default)]
    pub self_hash: u64,
}

impl MailboxMessageRecord {
    pub fn expected_self_hash(&self) -> u64 {
        let mut h = 0x4d42_5845_564e_5401u64;
        h = mix(h, self.schema_version);
        h = mix(h, self.record_kind);
        h = mix(h, string_hash(&self.id));
        h = mix(h, string_hash(&self.sender));
        h = mix(h, string_hash(&self.target));
        h = mix(h, string_hash(&self.kind));
        h = mix(h, string_hash(&self.payload));
        h = mix(h, string_hash(&self.sent_at));
        h = mix(h, self.prev_hash);
        h.max(1)
    }

    pub fn is_tlog_record_valid(&self) -> bool {
        self.schema_version == MAILBOX_TLOG_SCHEMA_VERSION
            && self.record_kind == MAILBOX_TLOG_RECORD_MESSAGE
            && validate_agent_id(&self.sender).is_ok()
            && validate_agent_id(&self.target).is_ok()
            && validate_message_kind(&self.kind).is_ok()
            && !self.id.is_empty()
            && !self.sent_at.is_empty()
            && self.self_hash == self.expected_self_hash()
    }

    pub fn contract_hash(&self) -> u64 {
        let mut h = 0x4d42_5852_4543_0001u64;
        h = mix(h, string_hash(&self.id));
        h = mix(h, string_hash(&self.sender));
        h = mix(h, string_hash(&self.target));
        h = mix(h, string_hash(&self.kind));
        h = mix(h, string_hash(&self.payload));
        h = mix(h, string_hash(&self.sent_at));
        h.max(1)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailboxReadProjection {
    pub messages: Vec<MailboxMessageRecord>,
    pub next_cursor: usize,
}

pub fn append_mailbox_message(
    workspace_root: &Path,
    sender: &str,
    target_agent: &str,
    message_kind: &str,
    payload: &str,
) -> Result<MailboxMessageRecord, String> {
    validate_agent_id(sender)?;
    validate_agent_id(target_agent)?;
    validate_message_kind(message_kind)?;

    let path = mailbox_tlog_path(workspace_root)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create mailbox dir: {error}"))?;
    }

    let mut record = MailboxMessageRecord {
        schema_version: MAILBOX_TLOG_SCHEMA_VERSION,
        record_kind: MAILBOX_TLOG_RECORD_MESSAGE,
        id: Uuid::new_v4().to_string(),
        sender: sender.to_string(),
        target: target_agent.to_string(),
        kind: message_kind.to_string(),
        payload: payload.to_string(),
        sent_at: Utc::now().to_rfc3339(),
        prev_hash: mailbox_last_hash(workspace_root)?,
        self_hash: 0,
    };
    record.self_hash = record.expected_self_hash();
    if !record.is_tlog_record_valid() {
        return Err("invalid mailbox record".to_string());
    }

    let line =
        serde_json::to_string(&record).map_err(|error| format!("serialize message: {error}"))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| format!("open mailbox {}: {error}", path.display()))?;
    writeln!(file, "{line}").map_err(|error| format!("write mailbox: {error}"))?;
    file.sync_all()
        .map_err(|error| format!("sync mailbox: {error}"))?;
    Ok(record)
}

pub fn read_mailbox_projection(
    workspace_root: &Path,
    agent_id: &str,
    since: usize,
) -> Result<MailboxReadProjection, String> {
    validate_agent_id(agent_id)?;
    let records = replay_mailbox_tlog(workspace_root)?;
    let next_cursor = records.len();
    let messages = records
        .into_iter()
        .enumerate()
        .filter_map(|(idx, record)| {
            if idx >= since && record.target == agent_id {
                Some(record)
            } else {
                None
            }
        })
        .collect();

    Ok(MailboxReadProjection {
        messages,
        next_cursor,
    })
}

pub fn replay_mailbox_tlog(workspace_root: &Path) -> Result<Vec<MailboxMessageRecord>, String> {
    let path = mailbox_tlog_path(workspace_root)?;
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(&path).map_err(|error| format!("open mailbox: {error}"))?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut prev_hash = 0_u64;

    for (idx, line) in reader.lines().enumerate() {
        let line = line.map_err(|error| format!("read mailbox line {idx}: {error}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let record: MailboxMessageRecord = serde_json::from_str(&line)
            .map_err(|error| format!("decode mailbox line {idx}: {error}"))?;
        if record.prev_hash != prev_hash || !record.is_tlog_record_valid() {
            return Err(format!("invalid mailbox hash chain at line {idx}"));
        }
        prev_hash = record.self_hash;
        records.push(record);
    }

    Ok(records)
}

pub fn mailbox_path(workspace_root: &Path, agent_id: &str) -> Result<PathBuf, String> {
    validate_agent_id(agent_id)?;
    mailbox_tlog_path(workspace_root)
}

pub fn mailbox_tlog_path(workspace_root: &Path) -> Result<PathBuf, String> {
    Ok(workspace_state_dir(workspace_root)
        .join("agent_state")
        .join("mailbox")
        .join("mailbox.tlog.ndjson"))
}

fn mailbox_last_hash(workspace_root: &Path) -> Result<u64, String> {
    Ok(replay_mailbox_tlog(workspace_root)?
        .last()
        .map(|record| record.self_hash)
        .unwrap_or(0))
}

pub fn validate_agent_id(agent_id: &str) -> Result<(), String> {
    if agent_id.is_empty()
        || agent_id.contains('/')
        || agent_id.contains('\\')
        || agent_id.contains("..")
    {
        return Err(format!("invalid agent id: {agent_id:?}"));
    }
    Ok(())
}

fn validate_message_kind(message_kind: &str) -> Result<(), String> {
    if message_kind.trim().is_empty() || message_kind.contains('\0') {
        Err("message_kind is required".to_string())
    } else {
        Ok(())
    }
}

fn string_hash(value: &str) -> u64 {
    bytes_hash(value.as_bytes())
}

fn bytes_hash(bytes: &[u8]) -> u64 {
    let mut h = 0x4d42_5848_4153_4801u64;
    h = mix(h, bytes.len() as u64);
    for byte in bytes {
        h = mix(h, *byte as u64);
    }
    h.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_workspace() -> PathBuf {
        std::env::temp_dir().join(format!("canon-mailbox-test-{}", Uuid::new_v4()))
    }

    #[test]
    fn mailbox_request_and_receipt_are_contract_valid() {
        let workspace = unique_workspace();
        let request =
            MailboxMessageRequest::new(1, "agent-a", "agent-b", "Observation", "{\"ok\":true}")
                .expect("request should parse");
        assert!(request.is_admissible());

        let record = append_mailbox_message(
            &workspace,
            "agent-a",
            "agent-b",
            "Observation",
            "{\"ok\":true}",
        )
        .expect("message should append");
        let receipt = MailboxMessageReceipt::from_record(&request, &record);
        assert!(receipt.is_contract_valid());
        assert!(receipt.is_valid_for(&request));
        assert_eq!(receipt.request_hash, request.contract_hash());

        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn mailbox_projection_reads_cursor_order() {
        let workspace = unique_workspace();
        append_mailbox_message(&workspace, "agent-a", "agent-b", "Observation", "one")
            .expect("first append");
        append_mailbox_message(&workspace, "agent-a", "agent-b", "Observation", "two")
            .expect("second append");

        let all = read_mailbox_projection(&workspace, "agent-b", 0).expect("read all");
        assert_eq!(all.messages.len(), 2);
        assert_eq!(all.next_cursor, 2);
        assert_eq!(all.messages[0].payload, "one");
        assert_eq!(all.messages[1].payload, "two");

        let tail = read_mailbox_projection(&workspace, "agent-b", 1).expect("read tail");
        assert_eq!(tail.messages.len(), 1);
        assert_eq!(tail.next_cursor, 2);
        assert_eq!(tail.messages[0].payload, "two");

        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn invalid_agent_ids_are_rejected() {
        assert!(validate_agent_id("../agent").is_err());
        assert!(MailboxMessageRequest::new(1, "agent/a", "agent-b", "Observation", "{}").is_err());
        assert!(MailboxMessageRequest::new(1, "agent-a", "agent-b", "", "{}").is_err());
    }
}
