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
    working_dir: &Path,
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

    write_mcp_error_feedback(working_dir, cycle_num);
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

// ── MCP error feedback ────────────────────────────────────────────────────────

fn write_mcp_error_feedback(working_dir: &Path, cycle_num: u64) {
    let transcript_path = working_dir
        .join("state")
        .join("agent_state")
        .join("mcp")
        .join("mcp-transcript.tlog.ndjson");

    if !transcript_path.exists() {
        return;
    }

    let stats = match collect_mcp_stats(&transcript_path) {
        Ok(s) if !s.is_empty() => s,
        Ok(_) => return,
        Err(e) => {
            eprintln!("agent: mcp feedback scan failed  cycle={cycle_num}  {e}");
            return;
        }
    };

    let text = format_mcp_feedback(&stats, cycle_num);
    let out_path = working_dir.join("state").join("mcp-feedback.md");
    if let Some(parent) = out_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    match fs::write(&out_path, &text) {
        Ok(_) => eprintln!(
            "agent: mcp feedback written  tools={}  cycle={cycle_num}",
            stats.len()
        ),
        Err(e) => eprintln!("agent: mcp feedback write failed  cycle={cycle_num}  {e}"),
    }
}

struct ToolStat {
    total: u64,
    errors: u64,
    last_error_hint: Option<String>,
}

fn collect_mcp_stats(path: &Path) -> Result<BTreeMap<String, ToolStat>, String> {
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut stats: BTreeMap<String, ToolStat> = BTreeMap::new();

    for line in reader.lines() {
        let line = line.map_err(|e| e.to_string())?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(record) = decode_mcp_transcript_record(line) else {
            continue;
        };

        let stat = stats
            .entry(record.tool_name.as_str().to_string())
            .or_insert(ToolStat {
                total: 0,
                errors: 0,
                last_error_hint: None,
            });
        stat.total += 1;
        if record.outcome.is_error() {
            stat.errors += 1;
            if stat.last_error_hint.is_none() {
                stat.last_error_hint = record.error_hint().map(ErrorHint::into_string);
            }
        }
    }

    Ok(stats)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct McpTranscriptRecord {
    tool_name: McpToolName,
    outcome: ToolOutcome,
    response: McpResponseBody,
}

impl McpTranscriptRecord {
    fn decode(line: &str) -> Result<Self, McpTranscriptDecodeError> {
        let wire: McpTranscriptRecordWire =
            serde_json::from_str(line).map_err(|_| McpTranscriptDecodeError::MalformedRecord)?;
        let tool_name = wire
            .tool_name
            .ok_or(McpTranscriptDecodeError::MissingToolName)
            .and_then(McpToolName::new)?;
        let outcome = ToolOutcome::from_wire(wire.exit_status, wire.timed_out);
        let response = McpResponseBody::decode(wire.response_json);
        Ok(Self {
            tool_name,
            outcome,
            response,
        })
    }

    fn error_hint(&self) -> Option<ErrorHint> {
        extract_error_hint(self)
    }
}

#[derive(serde::Deserialize)]
struct McpTranscriptRecordWire {
    tool_name: Option<String>,
    #[serde(default)]
    exit_status: u64,
    #[serde(default)]
    timed_out: bool,
    response_json: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct McpToolName(String);

impl McpToolName {
    fn new(value: String) -> Result<Self, McpTranscriptDecodeError> {
        let value = value.trim();
        if value.is_empty() {
            Err(McpTranscriptDecodeError::EmptyToolName)
        } else {
            Ok(Self(value.to_string()))
        }
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ToolOutcome {
    Success,
    ExitStatus(std::num::NonZeroU64),
    TimedOut,
}

impl ToolOutcome {
    fn from_wire(exit_status: u64, timed_out: bool) -> Self {
        if timed_out {
            return Self::TimedOut;
        }
        match std::num::NonZeroU64::new(exit_status) {
            Some(status) => Self::ExitStatus(status),
            None => Self::Success,
        }
    }

    fn is_error(self) -> bool {
        !matches!(self, Self::Success)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum McpResponseBody {
    Text(McpResponseText),
    Opaque,
}

impl McpResponseBody {
    fn decode(response_json: Option<String>) -> Self {
        let Some(response_json) = response_json else {
            return Self::Opaque;
        };
        let Ok(response) = serde_json::from_str::<McpToolResponseWire>(&response_json) else {
            return Self::Opaque;
        };
        match response
            .content
            .into_iter()
            .find_map(|content| content.text)
        {
            Some(text) => Self::Text(McpResponseText(text)),
            None => Self::Opaque,
        }
    }

    fn error_hint(&self) -> Option<ErrorHint> {
        match self {
            Self::Text(text) => Some(text.error_hint()),
            Self::Opaque => None,
        }
    }
}

#[derive(serde::Deserialize)]
struct McpToolResponseWire {
    #[serde(default)]
    content: Vec<McpToolResponseContentWire>,
}

#[derive(serde::Deserialize)]
struct McpToolResponseContentWire {
    text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct McpResponseText(String);

impl McpResponseText {
    fn error_hint(&self) -> ErrorHint {
        let hint = self.0.trim_start_matches("Error: ").trim();
        ErrorHint::new(truncate_error_hint(hint))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ErrorHint(String);

impl ErrorHint {
    fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    fn into_string(self) -> String {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum McpTranscriptDecodeError {
    MalformedRecord,
    MissingToolName,
    EmptyToolName,
}

fn decode_mcp_transcript_record(
    line: &str,
) -> Result<McpTranscriptRecord, McpTranscriptDecodeError> {
    McpTranscriptRecord::decode(line)
}

fn extract_error_hint(record: &McpTranscriptRecord) -> Option<ErrorHint> {
    match record.outcome {
        ToolOutcome::Success => None,
        ToolOutcome::TimedOut => Some(ErrorHint::new("timed out")),
        ToolOutcome::ExitStatus(_) => record.response.error_hint(),
    }
}

fn truncate_error_hint(hint: &str) -> String {
    let mut chars = hint.chars();
    let head: String = chars.by_ref().take(80).collect();
    if chars.next().is_some() {
        format!("{head}…")
    } else {
        head
    }
}

fn format_mcp_feedback(stats: &BTreeMap<String, ToolStat>, cycle_num: u64) -> String {
    let mut out = format!("# MCP Tool Feedback (after cycle {cycle_num})\n\n");

    let mut errored: Vec<_> = stats.iter().filter(|(_, s)| s.errors > 0).collect();
    errored.sort_by_key(|(_, s)| std::cmp::Reverse(s.errors));

    if !errored.is_empty() {
        out.push_str("## Tools with errors\n\n");
        for (name, stat) in &errored {
            let pct = stat.errors * 100 / stat.total.max(1);
            out.push_str(&format!(
                "- **{name}**: {}/{} calls failed ({pct}%)",
                stat.errors, stat.total
            ));
            if let Some(hint) = &stat.last_error_hint {
                out.push_str(&format!(" — `{hint}`"));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    let clean: Vec<_> = stats.iter().filter(|(_, s)| s.errors == 0).collect();
    if !clean.is_empty() {
        out.push_str("## Tools with no errors\n\n");
        for (name, stat) in &clean {
            out.push_str(&format!("- **{name}**: {} calls\n", stat.total));
        }
        out.push('\n');
    }

    out
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn typed_mcp_record_decoder_preserves_outcomes() {
        let success = decode_mcp_transcript_record(
            r#"{"tool_name":"workspace:get","exit_status":0,"timed_out":false}"#,
        )
        .expect("success record decodes");
        assert_eq!(success.outcome, ToolOutcome::Success);
        assert!(!success.outcome.is_error());

        let exited = decode_mcp_transcript_record(
            r#"{"tool_name":"workspace:shell","exit_status":42,"timed_out":false}"#,
        )
        .expect("nonzero exit record decodes");
        assert_eq!(
            exited.outcome,
            ToolOutcome::ExitStatus(std::num::NonZeroU64::new(42).unwrap())
        );
        assert!(exited.outcome.is_error());

        let timed_out = decode_mcp_transcript_record(
            r#"{"tool_name":"workspace:shell","exit_status":0,"timed_out":true}"#,
        )
        .expect("timeout record decodes");
        assert_eq!(timed_out.outcome, ToolOutcome::TimedOut);
        assert_eq!(
            extract_error_hint(&timed_out)
                .map(ErrorHint::into_string)
                .as_deref(),
            Some("timed out")
        );
    }

    #[test]
    fn typed_mcp_record_decoder_rejects_missing_tool_identity() {
        assert_eq!(
            decode_mcp_transcript_record(r#"{"exit_status":0,"timed_out":false}"#),
            Err(McpTranscriptDecodeError::MissingToolName)
        );
        assert_eq!(
            decode_mcp_transcript_record(
                r#"{"tool_name":"   ","exit_status":0,"timed_out":false}"#
            ),
            Err(McpTranscriptDecodeError::EmptyToolName)
        );
        assert_eq!(
            decode_mcp_transcript_record("not-json"),
            Err(McpTranscriptDecodeError::MalformedRecord)
        );
    }

    #[test]
    fn collect_mcp_stats_decodes_typed_rows_and_skips_malformed_records() {
        let path = unique_temp_file("mcp-transcript");
        let shell_response = serde_json::json!({
            "content": [
                { "type": "text" },
                { "type": "text", "text": "Error: nested command failed" }
            ]
        })
        .to_string();
        let rows = vec![
            "not-json".to_string(),
            serde_json::json!({ "exit_status": 1, "timed_out": false }).to_string(),
            serde_json::json!({
                "tool_name": "workspace:shell",
                "exit_status": 0,
                "timed_out": false,
                "response_json": "{}"
            })
            .to_string(),
            serde_json::json!({
                "tool_name": "workspace:shell",
                "exit_status": 1,
                "timed_out": false,
                "response_json": shell_response
            })
            .to_string(),
            serde_json::json!({
                "tool_name": "workspace:slow",
                "exit_status": 0,
                "timed_out": true,
                "response_json": "{}"
            })
            .to_string(),
        ];
        std::fs::write(&path, rows.join("\n")).expect("write transcript fixture");

        let stats = collect_mcp_stats(&path).expect("stats decode");
        let _ = std::fs::remove_file(&path);

        let shell = stats.get("workspace:shell").expect("shell stats");
        assert_eq!(shell.total, 2);
        assert_eq!(shell.errors, 1);
        assert_eq!(
            shell.last_error_hint.as_deref(),
            Some("nested command failed")
        );

        let slow = stats.get("workspace:slow").expect("slow stats");
        assert_eq!(slow.total, 1);
        assert_eq!(slow.errors, 1);
        assert_eq!(slow.last_error_hint.as_deref(), Some("timed out"));
        assert_eq!(stats.len(), 2);
    }

    #[test]
    fn mcp_error_hints_are_truncated_on_character_boundaries() {
        let long_hint = format!("Error: {}", "é".repeat(90));
        let response = serde_json::json!({ "content": [{ "text": long_hint }] }).to_string();
        let line = serde_json::json!({
            "tool_name": "workspace:shell",
            "exit_status": 1,
            "timed_out": false,
            "response_json": response
        })
        .to_string();

        let record = decode_mcp_transcript_record(&line).expect("record decodes");
        let hint = extract_error_hint(&record)
            .map(ErrorHint::into_string)
            .unwrap();
        assert_eq!(hint.chars().count(), 81);
        assert!(hint.ends_with('…'));
    }

    fn unique_temp_file(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "canon-ai-learning-{name}-{}-{nanos}.ndjson",
            std::process::id()
        ))
    }
}
