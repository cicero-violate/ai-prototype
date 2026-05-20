//! Orchestration capability.
//!
//! Orchestration is not a new kernel gate. It deterministically orders existing
//! capability submissions so the API can drive a run through the already typed
//! evidence path.

pub mod cycle_event;
pub mod record;
pub mod wave;

pub use self::cycle_event::{AgentCycleEvent, AgentCycleEventKind};
pub use self::wave::{ChildCompleteRecord, WaveRecord};
pub use self::record::{
    CapabilityRoute, OrchestrationBatchDecision, OrchestrationBatchRecord, OrchestrationBudget,
    OrchestrationDecision, OrchestrationRecord, SelectedCapabilityRoute,
};
