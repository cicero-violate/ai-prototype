//! Deterministic command handlers.

use crate::api::protocol::{Command, ControlEventResponse};
use crate::capability::EvidenceSubmission;
use crate::kernel::{Cause, Decision, EventKind, RuntimeConfig, State, TLog};
use crate::runtime::{tick, CanonError, CanonicalWriter, Outcome};

pub fn handle_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    command: Command,
) -> Result<ControlEventResponse, CanonError> {
    match command {
        Command::SubmitEvidence(submission) => {
            append_submission_event(state, tlog, cfg, submission)?;
            tick(state, tlog, cfg)?;

            Ok(ControlEventResponse {
                event: *tlog.last().ok_or(CanonError::InvalidReplay)?,
            })
        }
        Command::SubmitEvidenceBatch(submissions) => {
            if submissions.is_empty() {
                return Err(CanonError::InvalidReplay);
            }

            for submission in submissions {
                append_submission_event(state, tlog, cfg, submission)?;
                tick(state, tlog, cfg)?;
            }

            Ok(ControlEventResponse {
                event: *tlog.last().ok_or(CanonError::InvalidReplay)?,
            })
        }
    }
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
