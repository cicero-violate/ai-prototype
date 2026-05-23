//! Durable per-agent cycle event log.
//!
//! Appends a newline-delimited JSON record to `sse-chunks/cycle-events.ndjson`
//! at each turn boundary and at cycle start/end.  On restart the log lets us
//! detect which cycles completed and which were interrupted mid-turn.
//!
//! Each event is also submitted to the kernel TLog via POST /v1/command so the
//! full scheduling history is replayable from the canonical event log.
//!
//! Schema:
//!   { "schema": "canon.agent.cycle_event.v1",
//!     "ts": <unix_ms>,
//!     "agent": "<tag>",
//!     "cycle": <u64>,
//!     "event": "cycle_start" | "turn_complete" | "turn_failed" | "cycle_end",
//!     "label": "<turn label, omitted for cycle events>",
//!     "content_hash": <u64, omitted for cycle events> }

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::api::protocol::{Command, CommandEnvelope};
use crate::capability::orchestration::{AgentCycleEvent, AgentCycleEventKind};

use super::common::{stable_agent_hash, timestamp_ms};
use super::http::post_json_local;

const CYCLE_LOG_FILE: &str = "cycle-events.ndjson";

pub(super) fn append_cycle_start(dir: &Path, tag: &str, cycle_num: u64, command_url: Option<&str>) {
    append(dir, tag, cycle_num, "cycle_start", None, None);
    submit_cycle_event(
        command_url,
        AgentCycleEventKind::CycleStart,
        stable_agent_hash(tag.as_bytes()),
        cycle_num,
        0,
        0,
    );
}

pub(super) fn append_cycle_end(dir: &Path, tag: &str, cycle_num: u64, command_url: Option<&str>) {
    append(dir, tag, cycle_num, "cycle_end", None, None);
    submit_cycle_event(
        command_url,
        AgentCycleEventKind::CycleEnd,
        stable_agent_hash(tag.as_bytes()),
        cycle_num,
        0,
        0,
    );
}

pub(super) fn append_turn_complete(
    dir: &Path,
    tag: &str,
    cycle_num: u64,
    label: &str,
    content: &str,
    command_url: Option<&str>,
) {
    let content_hash = stable_agent_hash(content.as_bytes());
    append(
        dir,
        tag,
        cycle_num,
        "turn_complete",
        Some(label),
        Some(content_hash),
    );
    submit_cycle_event(
        command_url,
        AgentCycleEventKind::TurnComplete,
        stable_agent_hash(tag.as_bytes()),
        cycle_num,
        stable_agent_hash(label.as_bytes()),
        content_hash,
    );
}

pub(super) fn append_turn_failed(
    dir: &Path,
    tag: &str,
    cycle_num: u64,
    label: &str,
    command_url: Option<&str>,
) {
    append(dir, tag, cycle_num, "turn_failed", Some(label), None);
    submit_cycle_event(
        command_url,
        AgentCycleEventKind::TurnFailed,
        stable_agent_hash(tag.as_bytes()),
        cycle_num,
        stable_agent_hash(label.as_bytes()),
        0,
    );
}

/// Returns the last cycle number for which a `cycle_end` event was recorded,
/// allowing the caller to detect an interrupted previous run.
pub(super) fn last_completed_cycle(dir: &Path, tag: &str) -> Option<u64> {
    let path = dir.join(CYCLE_LOG_FILE);
    let text = fs::read_to_string(&path).ok()?;
    let mut last: Option<u64> = None;
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if v.get("agent").and_then(|a| a.as_str()) != Some(tag) {
            continue;
        }
        if v.get("event").and_then(|e| e.as_str()) == Some("cycle_end") {
            if let Some(n) = v.get("cycle").and_then(|c| c.as_u64()) {
                last = Some(last.map_or(n, |prev| prev.max(n)));
            }
        }
    }
    last
}

fn submit_cycle_event(
    command_url: Option<&str>,
    kind: AgentCycleEventKind,
    agent_hash: u64,
    cycle: u64,
    label_hash: u64,
    content_hash: u64,
) {
    let Some(url) = command_url else { return };
    let event = AgentCycleEvent::new(kind, agent_hash, cycle, label_hash, content_hash);
    if !event.is_contract_valid() {
        eprintln!("[cycle_log] skipping kernel POST: invalid contract hash");
        return;
    }
    let envelope =
        CommandEnvelope::new(event.contract_hash(), Command::SubmitAgentCycleEvent(event));
    let payload = serde_json::json!({
        "command_id": envelope.command_id,
        "command_hash": envelope.command_hash,
        "payload_tag": "SubmitAgentCycleEvent",
        "payload": {
            "kind": kind.as_u64(),
            "agent_hash": event.agent_hash,
            "cycle": event.cycle,
            "label_hash": event.label_hash,
            "content_hash": event.content_hash,
        },
    });
    match post_json_local(url, &payload) {
        Ok(status) if (200..300).contains(&status) => {}
        Ok(status) => {
            eprintln!("[cycle_log] kernel POST failed: status={status}");
        }
        Err(err) => {
            eprintln!("[cycle_log] kernel POST error: {err}");
        }
    }
}

fn append(
    dir: &Path,
    tag: &str,
    cycle_num: u64,
    event: &str,
    label: Option<&str>,
    content_hash: Option<u64>,
) {
    let _ = fs::create_dir_all(dir);
    let path = dir.join(CYCLE_LOG_FILE);
    let mut record = serde_json::json!({
        "schema": "canon.agent.cycle_event.v1",
        "ts": timestamp_ms(),
        "agent": tag,
        "cycle": cycle_num,
        "event": event,
    });
    if let Some(l) = label {
        record["label"] = serde_json::Value::String(l.to_string());
    }
    if let Some(h) = content_hash {
        record["content_hash"] = serde_json::Value::Number(h.into());
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{record}");
    }
}
