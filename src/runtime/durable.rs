//! Durable runtime entry points.
//!
//! Disk append happens before in-memory mutation.

use std::path::Path;

use crate::codec::ndjson::load_tlog_ndjson;
use crate::kernel::{Phase, RuntimeConfig, State, TLog};

use super::verify::{replay_report_from, replay_report_ndjson, verify_tlog_from, ReplayReport};
use super::{convergence_outcome, reduce, CanonError, CanonicalWriter, CommandLedger};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DurableRuntimeState {
    pub state: State,
    pub tlog: TLog,
    pub command_ledger: CommandLedger,
}

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

pub fn tick_durable_checked(
    state: &mut State,
    tlog: &mut TLog,
    tlog_path: impl AsRef<Path>,
    initial: State,
    cfg: RuntimeConfig,
) -> Result<ReplayReport, CanonError> {
    let path = tlog_path.as_ref();
    let disk_tlog = load_tlog_ndjson(path)?;
    if disk_tlog != *tlog {
        return Err(CanonError::InvalidReplay);
    }

    let before_report = replay_report_from(initial, &disk_tlog)?;
    if before_report.final_state != *state {
        return Err(CanonError::InvalidStateContinuity);
    }

    tick_durable(state, tlog, path, cfg)?;
    replay_report_from(initial, tlog)
}

pub fn durable_replay_report(
    initial: State,
    tlog_path: impl AsRef<Path>,
) -> Result<ReplayReport, CanonError> {
    replay_report_ndjson(initial, tlog_path)
}

pub fn resume_durable_runtime(
    initial: State,
    tlog_path: impl AsRef<Path>,
) -> Result<DurableRuntimeState, CanonError> {
    let path = tlog_path.as_ref();
    let tlog = load_tlog_ndjson(path)?;
    let command_ledger = CommandLedger::reconstruct_from_tlog(&tlog)?;
    let state = if tlog.is_empty() {
        initial
    } else {
        verify_tlog_from(initial, &tlog)?
    };

    Ok(DurableRuntimeState {
        state,
        tlog,
        command_ledger,
    })
}

pub fn run_until_done_durable(
    initial: State,
    cfg: RuntimeConfig,
    tlog_path: impl AsRef<Path>,
) -> Result<(State, TLog), CanonError> {
    let runtime = run_until_done_durable_with_ledger(initial, cfg, tlog_path)?;
    Ok((runtime.state, runtime.tlog))
}

pub fn run_until_done_durable_with_ledger(
    initial: State,
    cfg: RuntimeConfig,
    tlog_path: impl AsRef<Path>,
) -> Result<DurableRuntimeState, CanonError> {
    let path = tlog_path.as_ref();
    let mut runtime = resume_durable_runtime(initial, path)?;

    for _ in 0..cfg.max_steps {
        if runtime.state.phase == Phase::Done {
            return Ok(runtime);
        }

        tick_durable(&mut runtime.state, &mut runtime.tlog, path, cfg)?;
    }

    append_convergence_failure_durable(&mut runtime.state, &mut runtime.tlog, path, cfg)?;
    runtime.command_ledger = CommandLedger::reconstruct_from_tlog(&runtime.tlog)?;
    Ok(runtime)
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
