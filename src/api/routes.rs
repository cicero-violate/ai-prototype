//! Deterministic command handlers.

use crate::api::protocol::{Command, CommandEnvelope, ControlEventResponse};
use crate::capability::EvidenceSubmission;
use crate::kernel::{Cause, Decision, EventKind, RuntimeConfig, State, TLog};
use crate::runtime::{tick, CanonError, CanonicalWriter, Outcome};

pub fn handle_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    command: Command,
) -> Result<ControlEventResponse, CanonError> {
    if !command.is_contract_valid() {
        return Err(CanonError::InvalidApiCommand);
    }

    let mut candidate_state = *state;
    let mut candidate_tlog = tlog.clone();

    match command {
        Command::SubmitEvidence(submission) => {
            append_submission_event(&mut candidate_state, &mut candidate_tlog, cfg, submission)?;
            tick(&mut candidate_state, &mut candidate_tlog, cfg)?;
        }
        Command::SubmitEvidenceBatch(submissions) => {
            for submission in submissions {
                append_submission_event(&mut candidate_state, &mut candidate_tlog, cfg, submission)?;
                tick(&mut candidate_state, &mut candidate_tlog, cfg)?;
            }
        }
    }

    let response = ControlEventResponse {
        event: *candidate_tlog.last().ok_or(CanonError::InvalidReplay)?,
    };
    *state = candidate_state;
    *tlog = candidate_tlog;
    Ok(response)
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

    handle_command(state, tlog, cfg, envelope.into_command())
}

fn append_submission_event(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    submission: EvidenceSubmission,
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
    let event = CanonicalWriter::append(tlog, before, outcome, cfg)?;
    *state = event.state_after;
    Ok(())
}
