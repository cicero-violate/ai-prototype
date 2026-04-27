//! Deterministic command handlers.

use crate::api::protocol::{Command, ControlEventResponse};
use crate::kernel::{RuntimeConfig, State, TLog};
use crate::runtime::{tick, CanonError};

pub fn handle_command(
    state: &mut State,
    tlog: &mut TLog,
    cfg: RuntimeConfig,
    command: Command,
) -> Result<ControlEventResponse, CanonError> {
    match command {
        Command::SubmitEvidence(submission) => {
            submission.apply_to(state);
            tick(state, tlog, cfg)?;

            Ok(ControlEventResponse {
                event: *tlog.last().ok_or(CanonError::InvalidReplay)?,
            })
        }
    }
}
