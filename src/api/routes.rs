//! Deterministic command handlers.

use crate::api::protocol::{Command, CommandEnvelope, CommandLedger, ControlEventResponse};
use crate::capability::{CapabilityRegistry, EvidenceSubmission};
use crate::kernel::{Cause, Decision, EventKind, RuntimeConfig, State, TLog};
use crate::runtime::{tick, tick_with_api_command, CanonError, CanonicalWriter, Outcome};

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
        Command::SubmitProcessReceipt(receipt_record) => {
            append_submission_event(
                &mut candidate_state,
                &mut candidate_tlog,
                cfg,
                receipt_record.submission(),
                receipt,
            )?;
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
    ledger.push_response(&envelope, &response.event);
    Ok(response)
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
        Some((command_id, command_hash)) => CanonicalWriter::append_with_command_and_registry_projection(
            tlog,
            before,
            outcome,
            cfg,
            command_id,
            command_hash,
            registry_projection,
        )?,
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
