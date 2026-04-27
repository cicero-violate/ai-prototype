//! State semantic-delta classifier.

use crate::kernel::{Phase, SemanticDelta, State};

pub fn semantic_diff(a: State, b: State) -> SemanticDelta {
    if a == b {
        return SemanticDelta::NoChange;
    }
    if a.phase == Phase::Learn && b.phase == Phase::Done {
        return SemanticDelta::LearningPromoted;
    }
    if b.phase == Phase::Done && b.failure.is_none() {
        return SemanticDelta::Completed;
    }
    if b.phase == Phase::Done && b.failure.is_some() {
        return SemanticDelta::Halted;
    }
    if a.phase == Phase::Recovery && b.phase == Phase::Persist {
        return SemanticDelta::RepairSelected;
    }
    if a.phase == Phase::Persist && a.recovery_action.is_some() {
        return SemanticDelta::RepairApplied;
    }
    if a.phase == Phase::Persist && b.phase == Phase::Learn {
        return SemanticDelta::Persisted;
    }
    if b.failure.is_some() && b.phase == Phase::Recovery {
        return SemanticDelta::FailureRaised;
    }
    if a.packet != b.packet {
        return SemanticDelta::PayloadChanged;
    }
    if a.phase != b.phase {
        return SemanticDelta::PhaseAdvanced;
    }
    SemanticDelta::NoChange
}
