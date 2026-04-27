//! Planning capability.
//!
//! Planning lives above the kernel. It converts an objective into a ready task
//! record and submits only the kernel-visible `TaskReady` evidence token plus a
//! deterministic packet effect.

pub mod record;

pub use self::record::{PlanDecision, PlanRecord};