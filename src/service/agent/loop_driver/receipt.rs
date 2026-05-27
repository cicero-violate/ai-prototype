//! Agent turn receipt writing and kernel command conversion.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use serde_json::json;

use crate::service::agent::router::RouterStreamingResult;
use crate::{CommandEnvelopeDto, Evidence, EvidenceSubmission, GateId, PacketEffect};

use super::common::{stable_agent_hash, timestamp_ms};
use super::evidence_submit::submit_judgment_evidence;
use super::http::submit_agent_turn_receipt;

#[expect(
    clippy::too_many_arguments,
    reason = "receipt finalization records all attempt metadata at one boundary"
)]
pub(super) fn finalize_run_cycle_attempt_result(
    receipt_dir: &Path,
    tag: &str,
    cycle_num: u64,
    label: &str,
    attempt: u32,
    request_hash: u64,
    has_target_url: bool,
    command_url: Option<&str>,
    result: RouterStreamingResult,
) -> RunCycleAttemptOutcome {
    let reason = result.reason.clone();
    let retry_is_safe = result.retry_is_safe(has_target_url);
    write_agent_turn_receipt(AgentTurnReceiptInput {
        dir: receipt_dir,
        tag,
        cycle_num,
        label,
        attempt,
        request_hash,
        status: AgentTurnStatus::from_completed(result.complete),
        reason: &reason,
        finish_reason: result.finish_reason.as_deref(),
        target_url: result.target_url.as_deref(),
        content: &result.content,
        retry_is_safe,
        command_url,
    });
    if result.complete {
        let preview: String = result.content.chars().take(120).collect();
        let preview = preview.replace('\n', " ");
        eprintln!(
            "[{tag}] turn {label} done — {} — {preview}…",
            result.target_url.as_deref().unwrap_or("no url"),
        );
        return RunCycleAttemptOutcome {
            completed: true,
            reason,
            retry_is_safe: false,
        };
    }

    eprintln!(
        "[{tag}] turn {label} incomplete — {}; finish={}",
        result.reason,
        result.finish_reason.as_deref().unwrap_or("none"),
    );

    RunCycleAttemptOutcome {
        completed: false,
        reason,
        retry_is_safe,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AgentTurnStatus {
    Completed,
    Incomplete,
}

impl AgentTurnStatus {
    pub(super) const fn from_completed(completed: bool) -> Self {
        if completed {
            Self::Completed
        } else {
            Self::Incomplete
        }
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Incomplete => "incomplete",
        }
    }

    pub(super) const fn is_completed(self) -> bool {
        matches!(self, Self::Completed)
    }
}

pub(super) struct AgentTurnReceiptInput<'a> {
    pub(super) dir: &'a Path,
    pub(super) tag: &'a str,
    pub(super) cycle_num: u64,
    pub(super) label: &'a str,
    pub(super) attempt: u32,
    pub(super) request_hash: u64,
    pub(super) status: AgentTurnStatus,
    pub(super) reason: &'a str,
    pub(super) finish_reason: Option<&'a str>,
    pub(super) target_url: Option<&'a str>,
    pub(super) content: &'a str,
    pub(super) retry_is_safe: bool,
    pub(super) command_url: Option<&'a str>,
}

pub(super) fn write_agent_turn_receipt(input: AgentTurnReceiptInput<'_>) {
    let _ = fs::create_dir_all(input.dir);
    let path = input.dir.join("agent-turn-receipts.ndjson");
    let receipt = json!({
        "schema": "canon.agent.router_turn_receipt.v1",
        "observed_at": timestamp_ms(),
        "agent": input.tag,
        "cycle": input.cycle_num,
        "label": input.label,
        "attempt": input.attempt,
        "status": input.status.as_str(),
        "reason": input.reason,
        "finish_reason": input.finish_reason,
        "request_hash": input.request_hash,
        "target_url_hash": input.target_url.map(|url| stable_agent_hash(url.as_bytes())),
        "content_hash": stable_agent_hash(input.content.as_bytes()),
        "content_len": input.content.len(),
        "retry_is_safe": input.retry_is_safe,
    });
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{receipt}");
    }
    if let Some(command_url) = input.command_url {
        submit_agent_turn_receipt(command_url, &receipt, input.status);
        // Analysis + Judgment gates: the planning turn's output is the judgment.
        if input.label == "plan" && input.status.is_completed() {
            let content_hash = stable_agent_hash(input.content.as_bytes());
            submit_judgment_evidence(command_url, content_hash);
        }
    }
}

pub(super) struct RunCycleAttemptOutcome {
    pub(super) completed: bool,
    pub(super) reason: String,
    pub(super) retry_is_safe: bool,
}

pub(super) fn agent_turn_kernel_command(
    receipt: &serde_json::Value,
    status: AgentTurnStatus,
) -> CommandEnvelopeDto {
    let payload_hash = stable_agent_hash(receipt.to_string().as_bytes());
    let plan_payload_hash = stable_agent_hash(
        json!({
            "schema": "canon.agent.router_turn_receipt.plan.v1",
            "agent_receipt_payload_hash": payload_hash,
        })
        .to_string()
        .as_bytes(),
    );
    let execution_passed = status.is_completed();
    let plan_submission = EvidenceSubmission::with_effect_payload(
        GateId::Plan,
        Evidence::TaskReady,
        true,
        PacketEffect::BindReadyTask,
        plan_payload_hash,
    );
    let execution_submission = EvidenceSubmission::with_payload(
        GateId::Execution,
        Evidence::ExecutionReceipt,
        execution_passed,
        payload_hash,
    );
    CommandEnvelopeDto::from_command(
        payload_hash,
        crate::Command::SubmitEvidenceBatch(vec![plan_submission, execution_submission]),
    )
}
