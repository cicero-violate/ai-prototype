//! Group-chat agent loop — simulates tool calls over ChatGPT group chat text protocol.
//!
//! ChatGPT group chats cannot use MCP connectors, so tool calls are expressed as
//! structured XML-like tags that the agent parses from the model's text response:
//!
//! Model → agent:
//!   <tool_call>{"name":"artifact_facts","call_id":"tc_1","args":{...}}</tool_call>
//!   <task_complete>{"created_at":"...","exit_status":0,"request_json":"...","response_json":"...","tool_name":"chatgpt_group",...}</task_complete>
//!
//! Agent → model (next user message):
//!   <tool_result>{"call_id":"tc_1","name":"artifact_facts","ok":true,"data":{...}}</tool_result>
//!
//! The group chat URL is pinned; the RouterClient keeps the conversation alive across turns
//! via target_url continuity (same tab, same conversation thread).

use std::thread;
use std::time::Duration;

use serde_json::{json, Value};

use crate::capability::llm::browser_router::RouterClient;
use crate::capability::llm::openai::{
    OpenAiChatRequest, OpenAiConfig, OpenAiMessage,
};
use crate::capability::llm::sse::ChunkLogger;

/// The pinned group chat URL for refactoring tasks.
pub const GROUP_CHAT_REFACTOR_URL: &str =
    "https://chatgpt.com/gg/6a13faffb7bc819a8b2c3a7320a238e1";

/// Model name that resolves to the `chatgpt_group` provider adapter in the browser-router.
const GROUP_MODEL: &str = "chatgpt-group";

const SYSTEM_PROMPT: &str = "\
You are performing a compiler artifact analysis and refactoring task. \
You have access to one tool: `artifact_facts`, which returns structured compiler facts for an artifact by ID.

When you need information, output a single tool call:

<tool_call>
{\"name\":\"artifact_facts\",\"call_id\":\"<unique_id>\",\"args\":{\"artifact_id\":\"<id>\"}}
</tool_call>

The system will respond with:

<tool_result>
{\"call_id\":\"<id>\",\"name\":\"artifact_facts\",\"ok\":true,\"data\":{...}}
</tool_result>

When your task is complete, output:

<task_complete>
{\"created_at\":\"<RFC3339 timestamp>\",\"exit_status\":0,\"receipt_hash\":0,\"request_json\":\"{\\\"action\\\":\\\"group_chat_task\\\"}\",\"response_bytes\":0,\"response_hash\":0,\"response_json\":\"{\\\"content\\\":[{\\\"type\\\":\\\"text\\\",\\\"text\\\":\\\"{\\\\\\\"summary\\\\\\\":\\\\\\\"<one sentence>\\\\\\\",\\\\\\\"conclusion\\\\\\\":\\\\\\\"<detailed findings>\\\\\\\"}\\\"}],\\\"isError\\\":false}\",\"timed_out\":false,\"tool_name\":\"chatgpt_group\"}
</task_complete>

Rules:
- One <tool_call> OR one <task_complete> block per reply — no other XML tags.
- call_id must be unique within the session.
- Do not repeat a call_id you already used.";

// ── Config ───────────────────────────────────────────────────────────────────

pub struct GroupChatLoopConfig {
    /// Browser-router supervisor base URL.
    pub router_base_url: String,
    /// Maximum number of agent↔model turns before giving up.
    pub max_turns: u32,
    /// Per-turn timeout passed to the browser-router (ms).
    pub turn_max_ms: u64,
    /// First-capture timeout hint (ms).
    pub first_capture_ms: u64,
    /// Idle timeout hint (ms).
    pub idle_ms: u64,
    /// Directory for SSE chunk logs (reuses agent infra).
    pub sse_chunks_dir: std::path::PathBuf,
}

impl GroupChatLoopConfig {
    pub fn new(router_base_url: impl Into<String>, sse_chunks_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            router_base_url: router_base_url.into(),
            max_turns: 20,
            turn_max_ms: 120_000,
            first_capture_ms: 30_000,
            idle_ms: 2_500,
            sse_chunks_dir: sse_chunks_dir.into(),
        }
    }
}

// ── Protocol types ────────────────────────────────────────────────────────────

pub struct ParsedToolCall {
    pub name: String,
    pub call_id: String,
    pub args: Value,
}

pub struct TaskComplete {
    pub summary: String,
    pub conclusion: String,
}

pub enum GroupChatResponse {
    ToolCall(ParsedToolCall),
    TaskComplete(TaskComplete),
    /// Model replied but included neither tag — treat as a recoverable protocol violation.
    NoTag(String),
}

// ── Protocol parsing ──────────────────────────────────────────────────────────

fn extract_tag(text: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = text.find(&open)? + open.len();
    let end = text[start..].find(&close)?;
    Some(text[start..start + end].trim().to_string())
}

pub fn parse_response(text: &str) -> GroupChatResponse {
    if let Some(inner) = extract_tag(text, "task_complete") {
        if let Ok(v) = serde_json::from_str::<Value>(&inner) {
            if let Some(done) = parse_task_complete_payload(&v) {
                return GroupChatResponse::TaskComplete(done);
            }
        }
    }
    if let Some(inner) = extract_tag(text, "tool_call") {
        if let Ok(v) = serde_json::from_str::<Value>(&inner) {
            return GroupChatResponse::ToolCall(ParsedToolCall {
                name: string_field(&v, "name"),
                call_id: string_field(&v, "call_id"),
                args: v.get("args").cloned().unwrap_or(Value::Null),
            });
        }
    }
    GroupChatResponse::NoTag(text.to_string())
}

fn parse_task_complete_payload(v: &Value) -> Option<TaskComplete> {
    if let Some(response_json) = v.get("response_json").and_then(Value::as_str) {
        let response = serde_json::from_str::<Value>(response_json).ok()?;
        let text = response
            .get("content")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(|item| item.get("text"))
            .and_then(Value::as_str)?;
        let answer = serde_json::from_str::<Value>(text)
            .unwrap_or_else(|_| json!({ "summary": text, "conclusion": text }));
        return Some(task_complete_from_answer(&answer));
    }
    Some(task_complete_from_answer(v))
}

fn task_complete_from_answer(v: &Value) -> TaskComplete {
    TaskComplete {
        summary: string_field(v, "summary"),
        conclusion: string_field(v, "conclusion"),
    }
}

fn string_field(v: &Value, key: &str) -> String {
    v.get(key).and_then(Value::as_str).unwrap_or("").to_string()
}

// ── Tool result formatter ─────────────────────────────────────────────────────

pub fn format_tool_result(call_id: &str, name: &str, ok: bool, data: &Value) -> String {
    let payload = serde_json::to_string(&json!({
        "call_id": call_id,
        "name": name,
        "ok": ok,
        "data": data,
    }))
    .unwrap_or_default();
    format!("<tool_result>\n{payload}\n</tool_result>")
}

// ── Main loop ─────────────────────────────────────────────────────────────────

/// Run the group chat refactoring loop.
///
/// `task` is the initial task description sent to ChatGPT after the system prompt.
/// `artifact_lookup` is called whenever ChatGPT issues an `artifact_facts` tool call;
/// it receives `(name, args)` and should return the fact payload or an error string.
pub fn run_group_chat_task(
    config: &GroupChatLoopConfig,
    task: &str,
    artifact_lookup: &dyn Fn(&str, &Value) -> Result<Value, String>,
) -> Result<TaskComplete, String> {
    let openai_config = OpenAiConfig {
        base_url: config.router_base_url.clone(),
        model: GROUP_MODEL.to_string(),
        timeout_ms: config.turn_max_ms.saturating_add(30_000),
    };
    let mut router = RouterClient::new(openai_config).map_err(|e| e.to_string())?;
    // Pin to the group chat from the start — first turn uses continue_at, not new_chat.
    router.pin_target_url(GROUP_CHAT_REFACTOR_URL.to_string());

    let initial_message = format!("{SYSTEM_PROMPT}\n\n---\n\n{task}");
    let mut next_user_message = initial_message;

    for turn in 0..config.max_turns {
        eprintln!(
            "[group_chat_loop] turn={turn} url={} msg_len={}",
            GROUP_CHAT_REFACTOR_URL,
            next_user_message.len(),
        );

        let request = OpenAiChatRequest::new(vec![OpenAiMessage::user(next_user_message.clone())]);

        let mut logger = ChunkLogger::new(
            &config.sse_chunks_dir,
            "group_chat",
            u64::from(turn),
            "turn",
        )
        .map_err(|e| format!("chunk logger: {e}"))?;

        let result = router
            .streaming_turn(
                request,
                &mut logger,
                config.turn_max_ms,
                config.first_capture_ms,
                config.idle_ms,
            )
            .map_err(|e| format!("router turn {turn} failed: {e}"))?;

        eprintln!(
            "[group_chat_loop] turn={turn} complete={} content_len={}",
            result.complete,
            result.content.len(),
        );

        match parse_response(&result.content) {
            GroupChatResponse::TaskComplete(done) => {
                eprintln!("[group_chat_loop] task_complete summary={}", done.summary);
                return Ok(done);
            }
            GroupChatResponse::ToolCall(call) => {
                eprintln!(
                    "[group_chat_loop] tool_call name={} call_id={}",
                    call.name, call.call_id,
                );
                let (ok, data) = match artifact_lookup(&call.name, &call.args) {
                    Ok(v) => (true, v),
                    Err(e) => (false, json!({ "error": e })),
                };
                next_user_message =
                    format_tool_result(&call.call_id, &call.name, ok, &data);
            }
            GroupChatResponse::NoTag(raw) => {
                let preview = raw.chars().take(120).collect::<String>();
                return Err(format!(
                    "turn {turn}: model replied without <tool_call> or <task_complete>: {preview}"
                ));
            }
        }

        thread::sleep(Duration::from_millis(600));
    }

    Err(format!(
        "group chat loop exhausted max_turns={} without task_complete",
        config.max_turns
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_response_accepts_action_result_envelope_shape() {
        let answer = json!({
            "summary": "run_update should be decomposed.",
            "conclusion": "Split the coordinator into stages."
        });
        let response_json = json!({
            "content": [{ "type": "text", "text": answer.to_string() }],
            "isError": false
        })
        .to_string();
        let payload = json!({
            "created_at": "2026-05-25T09:49:54Z",
            "exit_status": 0,
            "receipt_hash": 0,
            "request_json": "{\"action\":\"group_chat_task\"}",
            "response_bytes": response_json.len(),
            "response_hash": 0,
            "response_json": response_json,
            "timed_out": false,
            "tool_name": "chatgpt_group"
        });
        let text = format!("<task_complete>\n{}\n</task_complete>", payload);

        let GroupChatResponse::TaskComplete(done) = parse_response(&text) else {
            panic!("expected task complete");
        };

        assert_eq!(done.summary, "run_update should be decomposed.");
        assert_eq!(done.conclusion, "Split the coordinator into stages.");
    }
}
