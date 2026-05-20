//! Process scheduler policy and plan storage boundaries.
//!
//! Scheduler code owns process policy over domain plan state. The domain module
//! owns the graph model and pure rules; this module owns repository-state I/O
//! needed by scheduler/supervisor processes.

pub mod handler;
pub mod plan_store;
pub mod wave;

pub(crate) use wave::run_wave;
