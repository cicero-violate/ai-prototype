//! Durable runtime entry points.
//!
//! Disk append happens before in-memory mutation.

use std::path::Path;

use crate::codec::ndjson::load_tlog_ndjson;
use crate::kernel::{Phase, RuntimeConfig, State, TLog};

use super::verify::verify_tlog_from;
use super::{convergence_outcome, reduce, CanonError, CanonicalWriter};

pub fn tick_durable(
    state: &mut State,
    tlog: &mut TLog,
    tlog_path: impl AsRef<Path>,
    cfg: RuntimeConfig,
) -> Result<(), CanonError> {
    let before = *state;
    let outcome = reduce(before, cfg);
    let event = CanonicalWriter::append_durable(tlog, tlog_path, before, outcome, cfg)?;
    *state = event.state_after;
    Ok(())
}

pub fn run_until_done_durable(
    initial: State,
    cfg: RuntimeConfig,
    tlog_path: impl AsRef<Path>,
) -> Result<(State, TLog), CanonError> {
    let path = tlog_path.as_ref();
    let mut tlog = load_tlog_ndjson(path)?;
    let mut state = if tlog.is_empty() {
        initial
    } else {
        verify_tlog_from(initial, &tlog)?
    };

    for _ in 0..cfg.max_steps {
        if state.phase == Phase::Done {
            return Ok((state, tlog));
        }

        tick_durable(&mut state, &mut tlog, path, cfg)?;
    }

    append_convergence_failure_durable(&mut state, &mut tlog, path, cfg)?;
    Ok((state, tlog))
}

fn append_convergence_failure_durable(
    state: &mut State,
    tlog: &mut TLog,
    tlog_path: impl AsRef<Path>,
    cfg: RuntimeConfig,
) -> Result<(), CanonError> {
    let before = *state;
    let event = CanonicalWriter::append_durable(
        tlog,
        tlog_path,
        before,
        convergence_outcome(before),
        cfg,
    )?;
    *state = event.state_after;
    Ok(())
}
