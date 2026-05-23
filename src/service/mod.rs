//! Service-level orchestration surfaces.
//!
//! This is the migration target for the current top-level `process` module.
//! Services own long-lived workflows, daemons, supervisors, schedulers, task
//! runners, and agent loops. Bounded effect execution belongs under
//! `capability::execution`.

pub mod agent;
pub mod dispatch;
pub mod recovery;
pub mod scheduler;
pub mod supervisor;

pub use crate::process::*;
