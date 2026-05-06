//! Orchestration capability.
//!
//! Orchestration is not a new kernel gate. It deterministically orders existing
//! capability submissions so the API can drive a run through the already typed
//! evidence path.

pub mod record;

pub use self::record::{CapabilityRoute, OrchestrationDecision, OrchestrationRecord};