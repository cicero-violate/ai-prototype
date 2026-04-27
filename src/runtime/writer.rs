//! Canonical event writer.

use std::path::Path;

use crate::codec::ndjson::append_tlog_ndjson;
use crate::kernel::{ControlEvent, Phase, RuntimeConfig, State, TLog};

use super::verify::{hash_event, validate_event, EventHashInput, EventView};
use super::{semantic_diff, CanonError, Outcome};

pub(crate) struct CanonicalWriter;

impl CanonicalWriter {
    pub(crate) fn build(
        tlog: &[ControlEvent],
        before: State,
        outcome: Outcome,
        cfg: RuntimeConfig,
    ) -> Result<ControlEvent, CanonError> {
        let after = outcome.state;
        let delta = semantic_diff(before, after);

        validate_event(EventView {
            from: before.phase,
            to: after.phase,
            kind: outcome.kind,
            cause: outcome.cause,
            decision: outcome.decision,
            failure: outcome.failure,
            recovery_action: outcome.recovery_action,
            affected_gate: outcome.affected_gate,
        })?;

        if after.phase == Phase::Done && after.failure.is_none() && !after.is_success() {
            return Err(CanonError::InvalidCompletion);
        }

        let seq = tlog.len() as u64 + 1;
        let prev_hash = tlog.last().map(|e| e.self_hash).unwrap_or(0);
        let self_hash = hash_event(EventHashInput {
            seq,
            prev_hash,
            from: before.phase,
            to: after.phase,
            kind: outcome.kind,
            cause: outcome.cause,
            delta,
            evidence: outcome.evidence,
            decision: outcome.decision,
            failure: outcome.failure,
            recovery_action: outcome.recovery_action,
            affected_gate: outcome.affected_gate,
            runtime_config: cfg,
            state_before: before,
            state_after: after,
        });

        Ok(ControlEvent {
            seq,
            from: before.phase,
            to: after.phase,
            kind: outcome.kind,
            cause: outcome.cause,
            delta,
            evidence: outcome.evidence,
            decision: outcome.decision,
            failure: outcome.failure,
            recovery_action: outcome.recovery_action,
            affected_gate: outcome.affected_gate,
            runtime_config: cfg,
            state_before: before,
            state_after: after,
            prev_hash,
            self_hash,
        })
    }

    pub(crate) fn append(
        tlog: &mut TLog,
        before: State,
        outcome: Outcome,
        cfg: RuntimeConfig,
    ) -> Result<ControlEvent, CanonError> {
        let event = Self::build(tlog, before, outcome, cfg)?;
        tlog.push(event);
        Ok(event)
    }

    pub(crate) fn append_durable(
        tlog: &mut TLog,
        tlog_path: impl AsRef<Path>,
        before: State,
        outcome: Outcome,
        cfg: RuntimeConfig,
    ) -> Result<ControlEvent, CanonError> {
        let event = Self::build(tlog, before, outcome, cfg)?;
        append_tlog_ndjson(tlog_path, &event)?;
        tlog.push(event);
        Ok(event)
    }
}
