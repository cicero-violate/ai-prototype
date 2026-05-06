//! Eval capability.
//!
//! Eval is intentionally outside the kernel. The capability computes scores,
//! compares against policy thresholds, stores `EvalRecord`, then submits
//! `Evidence::EvalScore` as pass/fail evidence for `GateId::Eval`.

pub mod record;

pub use self::record::{EvalDecision, EvalDimension, EvalRecord};
