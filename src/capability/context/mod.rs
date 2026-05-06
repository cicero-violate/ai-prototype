//! Context capability.
//!
//! Context assembles the current packet, fresh observations, and memory recall
//! into a deterministic analysis record. The kernel sees only
//! `Evidence::AnalysisReport`.

pub mod record;

pub use self::record::{ContextDecision, ContextRecord};