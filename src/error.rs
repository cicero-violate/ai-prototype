//! Shared canonical error type.
//!
//! The error surface is intentionally below `codec`, `runtime`, and `api` so
//! lower layers do not import upward just to report deterministic failures.

use crate::kernel::{EventKind, Phase};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanonError {
    IllegalEvent {
        from: Phase,
        to: Phase,
        kind: EventKind,
    },
    MissingFailureClass,
    UnexpectedFailureClass,
    MissingRecoveryAction,
    UnexpectedRecoveryAction,
    InvalidLearnTarget,
    InvalidRepairTarget,
    InvalidCompletion,
    InvalidStateContinuity,
    InvalidPacketContinuity,
    InvalidSemanticDelta,
    InvalidHashChain,
    InvalidReplay,
    InvalidStateInvariant,
    InvalidRuntimeConfig,
    InvalidApiCommand,
    TlogIo,
    InvalidTlogRecord,
    MissingAffectedGate,
    UnexpectedAffectedGate,
}

impl core::fmt::Display for CanonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for CanonError {}