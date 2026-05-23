//! Deterministic command handlers.

use crate::api::protocol::{
    action_authorization_submission, mailbox_authorization_submission, mailbox_receipt_submission,
    mcp_authorization_submission, process_authorization_submission, Command, CommandEnvelope,
    CommandLedger, ControlEventResponse,
};
use crate::capability::execution::{
    ActionCallRequest as McpCallRequest, ActionReceipt as McpCallReceipt,
};
use crate::capability::orchestration::{AgentCycleEvent, ChildCompleteRecord, WaveRecord};
use crate::capability::planning::PlanPatchRecord;
use crate::capability::execution::{SandboxProcessReceipt, SandboxProcessRequest};
use crate::capability::{CapabilityId, CapabilityRegistry, EvidenceSubmission};
use crate::kernel::{
    CapabilityRegistryProjection, Cause, Decision, EventKind, Evidence, Phase, RuntimeConfig,
    State, TLog,
};
use crate::runtime::{
    tick, tick_with_api_command, CanonError, CanonicalWriter, MailboxMessageReceipt,
    MailboxMessageRequest, Outcome,
};

pub fn handle_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    command: Command,
) -> Result<ControlEventResponse, CanonError> {
    handle_command_with_receipt(state, tlog, cfg, command, None)
}

fn handle_command_with_receipt(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    command: Command,
    receipt: Option<(u64, u64)>,
) -> Result<ControlEventResponse, CanonError> {
    if !command.is_contract_valid() {
        return Err(CanonError::InvalidApiCommand);
    }

    let mut candidate_state = *state;
    let mut candidate_tlog = tlog.clone();
    apply_command(
        &mut candidate_state,
        &mut candidate_tlog,
        cfg,
        command,
        receipt,
    )?;

    let response = ControlEventResponse {
        event: *candidate_tlog.last().ok_or(CanonError::InvalidReplay)?,
    };
    *state = candidate_state;
    *tlog = candidate_tlog;
    Ok(response)
}

fn apply_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    command: Command,
    receipt: Option<(u64, u64)>,
) -> Result<(), CanonError> {
    match command {
        Command::SubmitEvidence(submission) => {
            apply_submission_command(state, tlog, cfg, submission, receipt)
        }
        Command::SubmitEvidenceBatch(submissions) => {
            apply_submission_batch_command(state, tlog, cfg, submissions, receipt)
        }
        Command::SubmitObservationIngress(batch) => {
            apply_submission_command(state, tlog, cfg, batch.submission(), receipt)
        }
        Command::AuthorizeActionCall(request) => append_submission_event(
            state,
            tlog,
            cfg,
            action_authorization_submission(request),
            receipt,
        ),
        Command::AuthorizeMcpCall(request) => append_submission_event(
            state,
            tlog,
            cfg,
            mcp_authorization_submission(request),
            receipt,
        ),
        Command::AuthorizeProcessCall(request) => append_submission_event(
            state,
            tlog,
            cfg,
            process_authorization_submission(request),
            receipt,
        ),
        Command::AuthorizeMailboxMessage(request) => append_submission_event(
            state,
            tlog,
            cfg,
            mailbox_authorization_submission(request),
            receipt,
        ),
        Command::SubmitMcpCallReceipt(receipt_record) => {
            apply_mcp_receipt_command(state, tlog, cfg, receipt_record, receipt)
        }
        Command::SubmitActionReceipt(receipt_record) => {
            apply_action_receipt_command(state, tlog, cfg, receipt_record, receipt)
        }
        Command::SubmitProcessReceipt(receipt_record) => {
            apply_process_receipt_command(state, tlog, cfg, receipt_record, receipt)
        }
        Command::SubmitMailboxMessageReceipt(receipt_record) => {
            apply_mailbox_message_receipt_command(state, tlog, cfg, receipt_record, receipt)
        }
        Command::SubmitProcessReceiptBatch(receipts) => {
            apply_process_receipt_batch_command(state, tlog, cfg, receipts, receipt)
        }
        Command::SubmitAgentCycleEvent(event) => {
            apply_agent_cycle_event_command(state, tlog, cfg, event, receipt)
        }
        Command::SubmitWaveDispatch(record) => {
            apply_wave_dispatch_command(state, tlog, cfg, record, receipt)
        }
        Command::SubmitChildComplete(record) => {
            apply_child_complete_command(state, tlog, cfg, record, receipt)
        }
        Command::SubmitPlanPatch(record) => {
            apply_plan_patch_command(state, tlog, cfg, record, receipt)
        }
    }
}

fn apply_submission_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    submission: EvidenceSubmission,
    receipt: Option<(u64, u64)>,
) -> Result<(), CanonError> {
    append_submission_event(state, tlog, cfg, submission, receipt)?;
    tick_for_command_response(state, tlog, cfg, receipt)
}

fn apply_submission_batch_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    submissions: Vec<EvidenceSubmission>,
    receipt: Option<(u64, u64)>,
) -> Result<(), CanonError> {
    let submission_count = submissions.len();
    for (idx, submission) in submissions.into_iter().enumerate() {
        append_submission_event(state, tlog, cfg, submission, receipt)?;
        let response_receipt = if idx + 1 == submission_count {
            receipt
        } else {
            None
        };
        tick_for_command_response(state, tlog, cfg, response_receipt)?;
    }
    Ok(())
}

fn apply_mcp_receipt_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    receipt_record: McpCallReceipt,
    receipt: Option<(u64, u64)>,
) -> Result<(), CanonError> {
    ensure_action_or_mcp_receipt_authorized(tlog, &receipt_record)?;
    apply_submission_command(state, tlog, cfg, receipt_record.submission(), receipt)
}

fn apply_action_receipt_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    receipt_record: McpCallReceipt,
    receipt: Option<(u64, u64)>,
) -> Result<(), CanonError> {
    ensure_action_or_mcp_receipt_authorized(tlog, &receipt_record)?;
    apply_submission_command(state, tlog, cfg, receipt_record.submission(), receipt)
}

fn apply_process_receipt_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    receipt_record: SandboxProcessReceipt,
    receipt: Option<(u64, u64)>,
) -> Result<(), CanonError> {
    ensure_process_receipt_authorized(tlog, &receipt_record)?;
    apply_submission_command(state, tlog, cfg, receipt_record.submission(), receipt)
}

fn apply_mailbox_message_receipt_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    receipt_record: MailboxMessageReceipt,
    receipt: Option<(u64, u64)>,
) -> Result<(), CanonError> {
    ensure_mailbox_message_receipt_authorized(tlog, &receipt_record)?;
    apply_submission_command(
        state,
        tlog,
        cfg,
        mailbox_receipt_submission(receipt_record),
        receipt,
    )
}

fn apply_agent_cycle_event_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    event: AgentCycleEvent,
    receipt: Option<(u64, u64)>,
) -> Result<(), CanonError> {
    let before = *state;
    // Cycle events are observational: state_after == state_before.
    // No gate is mutated; no tick is needed.
    let outcome = Outcome {
        state: before,
        kind: EventKind::Persisted,
        cause: Cause::AgentCycleEventSubmitted,
        evidence: Evidence::AgentCycleEvent,
        decision: Decision::Continue,
        failure: None,
        recovery_action: None,
        affected_gate: None,
    };
    let tlog_event = append_observational_event(tlog, before, outcome, cfg, receipt)?;
    *state = tlog_event.state_after;
    let _ = event;
    Ok(())
}

fn apply_wave_dispatch_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    record: WaveRecord,
    receipt: Option<(u64, u64)>,
) -> Result<(), CanonError> {
    let before = *state;
    let mut after = before;
    after.wave_pending = record.node_count;
    let outcome = Outcome {
        state: after,
        kind: EventKind::Persisted,
        cause: Cause::WaveDispatched,
        evidence: Evidence::WaveDispatched,
        decision: Decision::Continue,
        failure: None,
        recovery_action: None,
        affected_gate: None,
    };
    let tlog_event = append_observational_event(tlog, before, outcome, cfg, receipt)?;
    *state = tlog_event.state_after;
    Ok(())
}

fn apply_child_complete_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    record: ChildCompleteRecord,
    receipt: Option<(u64, u64)>,
) -> Result<(), CanonError> {
    let before = *state;
    if before.wave_pending == 0 {
        return Err(CanonError::InvalidReplay);
    }
    let mut after = before;
    after.wave_pending = before.wave_pending - 1;
    let outcome = Outcome {
        state: after,
        kind: EventKind::Persisted,
        cause: Cause::ChildTaskCompleted,
        evidence: Evidence::ChildTaskComplete,
        decision: Decision::Continue,
        failure: None,
        recovery_action: None,
        affected_gate: None,
    };
    let tlog_event = append_observational_event(tlog, before, outcome, cfg, receipt)?;
    *state = tlog_event.state_after;
    let _ = record;
    Ok(())
}

fn apply_plan_patch_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    record: PlanPatchRecord,
    receipt: Option<(u64, u64)>,
) -> Result<(), CanonError> {
    let before = *state;
    let mut after = before;
    after.plan_state_hash = record.patch_hash;
    let outcome = Outcome {
        state: after,
        kind: EventKind::Persisted,
        cause: Cause::PlanReady,
        evidence: Evidence::PlanRecord,
        decision: Decision::Continue,
        failure: None,
        recovery_action: None,
        affected_gate: None,
    };
    let tlog_event = append_observational_event(tlog, before, outcome, cfg, receipt)?;
    *state = tlog_event.state_after;
    Ok(())
}

fn append_observational_event(
    tlog: &mut TLog,
    before: State,
    outcome: Outcome,
    cfg: RuntimeConfig,
    receipt: Option<(u64, u64)>,
) -> Result<crate::kernel::ControlEvent, CanonError> {
    // Observational events must use an empty projection (not the canonical registry).
    match receipt {
        Some((command_id, command_hash)) => {
            CanonicalWriter::append_with_command_and_registry_projection(
                tlog,
                before,
                outcome,
                cfg,
                command_id,
                command_hash,
                CapabilityRegistryProjection::none(),
            )
        }
        None => CanonicalWriter::append_with_command_and_registry_projection(
            tlog,
            before,
            outcome,
            cfg,
            0,
            0,
            CapabilityRegistryProjection::none(),
        ),
    }
}

fn apply_process_receipt_batch_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    receipts: Vec<SandboxProcessReceipt>,
    receipt: Option<(u64, u64)>,
) -> Result<(), CanonError> {
    for receipt_record in &receipts {
        ensure_process_receipt_authorized(tlog, receipt_record)?;
    }
    for receipt_record in receipts {
        append_submission_event(state, tlog, cfg, receipt_record.submission(), receipt)?;
    }
    tick_for_command_response(state, tlog, cfg, receipt)
}

fn tick_for_command_response(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    receipt: Option<(u64, u64)>,
) -> Result<(), CanonError> {
    if state.phase == Phase::Done {
        return Ok(());
    }

    match receipt {
        Some((command_id, command_hash)) => {
            tick_with_api_command(state, tlog, cfg, command_id, command_hash)
        }
        None => tick(state, tlog, cfg),
    }
}

pub fn handle_envelope(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    envelope: CommandEnvelope,
) -> Result<ControlEventResponse, CanonError> {
    if !envelope.is_contract_valid() {
        return Err(CanonError::InvalidApiCommand);
    }

    let command_id = envelope.command_id;
    let command_hash = envelope.command_hash;
    handle_command_with_receipt(
        state,
        tlog,
        cfg,
        envelope.into_command(),
        Some((command_id, command_hash)),
    )
}

pub fn handle_envelope_once(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    ledger: &mut CommandLedger,
    envelope: CommandEnvelope,
) -> Result<ControlEventResponse, CanonError> {
    if !envelope.is_contract_valid() || ledger.has_conflicting_command(&envelope) {
        return Err(CanonError::InvalidApiCommand);
    }

    if let Some(event) = ledger.replayed_event(&envelope, tlog) {
        return Ok(ControlEventResponse { event });
    }

    if ledger.receipt_for(&envelope).is_some() {
        return Err(CanonError::InvalidReplay);
    }

    let response = handle_command_with_receipt(
        state,
        tlog,
        cfg,
        envelope.clone().into_command(),
        Some((envelope.command_id, envelope.command_hash)),
    )?;
    ledger.push_response(&envelope, &response.event)?;
    Ok(response)
}

fn ensure_action_or_mcp_receipt_authorized(
    tlog: &TLog,
    receipt: &McpCallReceipt,
) -> Result<(), CanonError> {
    let request = McpCallRequest {
        capability: CapabilityId::Tooling,
        registry_policy_hash: receipt.registry_policy_hash,
        worker_url_hash: receipt.worker_url_hash,
        tool_name_hash: receipt.tool_name_hash,
        args_hash: receipt.args_hash,
        timeout_ms: receipt.timeout_ms,
        max_output_bytes: receipt.max_output_bytes,
    };
    if !receipt.is_valid_for(&request) {
        return Err(CanonError::InvalidApiCommand);
    }
    ensure_prior_authorization(tlog, Command::AuthorizeActionCall(request))
        .or_else(|_| ensure_prior_authorization(tlog, Command::AuthorizeMcpCall(request)))
}

fn ensure_process_receipt_authorized(
    tlog: &TLog,
    receipt: &SandboxProcessReceipt,
) -> Result<(), CanonError> {
    let request = SandboxProcessRequest {
        capability: CapabilityId::Tooling,
        registry_policy_hash: receipt.registry_policy_hash,
        command_hash: receipt.command_hash,
        argv_hash: receipt.argv_hash,
        cwd_hash: receipt.cwd_hash,
        env_hash: receipt.env_hash,
        timeout_ms: receipt.timeout_ms,
        max_output_bytes: receipt.max_output_bytes,
    };
    if !receipt.is_valid_for(&request) {
        return Err(CanonError::InvalidApiCommand);
    }
    ensure_prior_authorization(tlog, Command::AuthorizeProcessCall(request))
}

fn ensure_mailbox_message_receipt_authorized(
    tlog: &TLog,
    receipt: &MailboxMessageReceipt,
) -> Result<(), CanonError> {
    let request = MailboxMessageRequest {
        capability_id: crate::runtime::MAILBOX_MESSAGE_CAPABILITY_ID,
        registry_policy_hash: receipt.registry_policy_hash,
        sender_hash: receipt.sender_hash,
        target_hash: receipt.target_hash,
        kind_hash: receipt.kind_hash,
        payload_hash: receipt.payload_hash,
    };
    if !receipt.is_valid_for(&request) {
        return Err(CanonError::InvalidApiCommand);
    }
    ensure_prior_authorization(tlog, Command::AuthorizeMailboxMessage(request))
}

fn ensure_prior_authorization(tlog: &TLog, authorization: Command) -> Result<(), CanonError> {
    let authorized = tlog.iter().any(|event| {
        event.api_command_id != 0
            && event.api_command_hash != 0
            && CommandEnvelope::new(event.api_command_id, authorization.clone()).command_hash
                == event.api_command_hash
    });
    if authorized {
        Ok(())
    } else {
        Err(CanonError::InvalidReplay)
    }
}

fn append_submission_event(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    submission: EvidenceSubmission,
    receipt: Option<(u64, u64)>,
) -> Result<(), CanonError> {
    let before = *state;
    let mut after = before;
    submission.apply_to(&mut after);
    let outcome = Outcome {
        state: after,
        kind: EventKind::Persisted,
        cause: Cause::EvidenceSubmitted,
        evidence: submission.evidence,
        decision: if submission.passed {
            Decision::Continue
        } else {
            Decision::Block
        },
        failure: None,
        recovery_action: None,
        affected_gate: Some(submission.gate),
    };
    let registry_projection = CapabilityRegistry::canonical().projection();
    let event = match receipt {
        Some((command_id, command_hash)) => {
            CanonicalWriter::append_with_command_and_registry_projection(
                tlog,
                before,
                outcome,
                cfg,
                command_id,
                command_hash,
                registry_projection,
            )?
        }
        None => CanonicalWriter::append_with_command_and_registry_projection(
            tlog,
            before,
            outcome,
            cfg,
            0,
            0,
            registry_projection,
        )?,
    };
    *state = event.state_after;
    Ok(())
}
