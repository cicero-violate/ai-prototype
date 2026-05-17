//! Deterministic command handlers.

use crate::api::protocol::{
    mailbox_authorization_submission, mcp_authorization_submission,
    process_authorization_submission, Command, CommandEnvelope, CommandLedger,
    ControlEventResponse,
};
use crate::capability::tooling::{
    McpCallReceipt, McpCallRequest, SandboxProcessReceipt, SandboxProcessRequest,
};
use crate::capability::{CapabilityId, CapabilityRegistry, EvidenceSubmission};
use crate::kernel::{Cause, Decision, EventKind, Phase, RuntimeConfig, State, TLog};
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

    match command {
        Command::SubmitEvidence(submission) => {
            append_submission_event(
                &mut candidate_state,
                &mut candidate_tlog,
                cfg,
                submission,
                receipt,
            )?;
            tick_for_command_response(&mut candidate_state, &mut candidate_tlog, cfg, receipt)?;
        }
        Command::SubmitEvidenceBatch(submissions) => {
            let submission_count = submissions.len();
            for (idx, submission) in submissions.into_iter().enumerate() {
                append_submission_event(
                    &mut candidate_state,
                    &mut candidate_tlog,
                    cfg,
                    submission,
                    receipt,
                )?;
                let response_receipt = if idx + 1 == submission_count {
                    receipt
                } else {
                    None
                };
                tick_for_command_response(
                    &mut candidate_state,
                    &mut candidate_tlog,
                    cfg,
                    response_receipt,
                )?;
            }
        }
        Command::SubmitObservationIngress(batch) => {
            append_submission_event(
                &mut candidate_state,
                &mut candidate_tlog,
                cfg,
                batch.submission(),
                receipt,
            )?;
            tick_for_command_response(&mut candidate_state, &mut candidate_tlog, cfg, receipt)?;
        }
        Command::AuthorizeMcpCall(request) => {
            append_submission_event(
                &mut candidate_state,
                &mut candidate_tlog,
                cfg,
                mcp_authorization_submission(request),
                receipt,
            )?;
        }
        Command::AuthorizeProcessCall(request) => {
            append_submission_event(
                &mut candidate_state,
                &mut candidate_tlog,
                cfg,
                process_authorization_submission(request),
                receipt,
            )?;
        }
        Command::AuthorizeMailboxMessage(request) => {
            append_submission_event(
                &mut candidate_state,
                &mut candidate_tlog,
                cfg,
                mailbox_authorization_submission(request),
                receipt,
            )?;
        }
        Command::SubmitMcpCallReceipt(receipt_record) => {
            ensure_mcp_receipt_authorized(&candidate_tlog, &receipt_record)?;
            append_submission_event(
                &mut candidate_state,
                &mut candidate_tlog,
                cfg,
                receipt_record.submission(),
                receipt,
            )?;
            tick_for_command_response(&mut candidate_state, &mut candidate_tlog, cfg, receipt)?;
        }
        Command::SubmitProcessReceipt(receipt_record) => {
            ensure_process_receipt_authorized(&candidate_tlog, &receipt_record)?;
            append_submission_event(
                &mut candidate_state,
                &mut candidate_tlog,
                cfg,
                receipt_record.submission(),
                receipt,
            )?;
            tick_for_command_response(&mut candidate_state, &mut candidate_tlog, cfg, receipt)?;
        }
        Command::SubmitMailboxMessageReceipt(receipt_record) => {
            ensure_mailbox_message_receipt_authorized(&candidate_tlog, &receipt_record)?;
            append_submission_event(
                &mut candidate_state,
                &mut candidate_tlog,
                cfg,
                receipt_record.submission(),
                receipt,
            )?;
            tick_for_command_response(&mut candidate_state, &mut candidate_tlog, cfg, receipt)?;
        }
        Command::SubmitProcessReceiptBatch(receipts) => {
            for receipt_record in &receipts {
                ensure_process_receipt_authorized(&candidate_tlog, receipt_record)?;
            }
            for receipt_record in receipts {
                append_submission_event(
                    &mut candidate_state,
                    &mut candidate_tlog,
                    cfg,
                    receipt_record.submission(),
                    receipt,
                )?;
            }
            tick_for_command_response(&mut candidate_state, &mut candidate_tlog, cfg, receipt)?;
        }
    }

    let response = ControlEventResponse {
        event: *candidate_tlog.last().ok_or(CanonError::InvalidReplay)?,
    };
    *state = candidate_state;
    *tlog = candidate_tlog;
    Ok(response)
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

fn ensure_mcp_receipt_authorized(tlog: &TLog, receipt: &McpCallReceipt) -> Result<(), CanonError> {
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
    ensure_prior_authorization(tlog, Command::AuthorizeMcpCall(request))
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
        capability: CapabilityId::Tooling,
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
