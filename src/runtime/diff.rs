//! State semantic-delta classifier.

use crate::kernel::{Phase, SemanticDelta, State};

pub fn semantic_diff(a: State, b: State) -> SemanticDelta {
    if a == b {
        return SemanticDelta::NoChange;
    }
    terminal_delta(a, b)
        .or_else(|| recovery_delta(a, b))
        .or_else(|| raised_failure_delta(b))
        .or_else(|| payload_delta(a, b))
        .or_else(|| phase_delta(a, b))
        .unwrap_or(SemanticDelta::NoChange)
}

fn terminal_delta(a: State, b: State) -> Option<SemanticDelta> {
    if a.phase == Phase::Learn && b.phase == Phase::Done {
        return Some(SemanticDelta::LearningPromoted);
    }
    if b.phase != Phase::Done {
        return None;
    }
    if b.failure.is_some() {
        Some(SemanticDelta::Halted)
    } else {
        Some(SemanticDelta::Completed)
    }
}

fn recovery_delta(a: State, b: State) -> Option<SemanticDelta> {
    if a.phase == Phase::Recovery && b.phase == Phase::Persist {
        return Some(SemanticDelta::RepairSelected);
    }
    if a.phase == Phase::Persist && a.recovery_action.is_some() {
        return Some(SemanticDelta::RepairApplied);
    }
    if a.phase == Phase::Persist && b.phase == Phase::Learn {
        return Some(SemanticDelta::Persisted);
    }
    None
}

fn raised_failure_delta(b: State) -> Option<SemanticDelta> {
    if b.failure.is_some() && b.phase == Phase::Recovery {
        Some(SemanticDelta::FailureRaised)
    } else {
        None
    }
}

fn payload_delta(a: State, b: State) -> Option<SemanticDelta> {
    if a.packet != b.packet || a.gates != b.gates {
        Some(SemanticDelta::PayloadChanged)
    } else {
        None
    }
}

fn phase_delta(a: State, b: State) -> Option<SemanticDelta> {
    if a.phase != b.phase {
        Some(SemanticDelta::PhaseAdvanced)
    } else {
        None
    }
}
