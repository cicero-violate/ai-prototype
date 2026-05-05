//! Minimal deterministic runtime surfaces for score-improvement deltas.
//!
//! This patch keeps the kernel boundary small: failures are classified once,
//! policy decisions are pure values, and tests assert the recovery lattice.

pub mod runtime;

pub use runtime::{FailureClass, RecoveryAction, RecoveryDecision, RecoveryPolicy};