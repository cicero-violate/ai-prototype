//! Recovery routing projection for AgentCycle.
//!
//! Runtime owns recovery-policy selection. This module only adapts typed
//! kernel/runtime recovery data to worker prompt strings and state JSON fields.

#![allow(dead_code)]

use crate::kernel::{FailureClass, RecoveryAction};
use crate::runtime::recovery_action_for;

#[derive(Clone, Copy)]
pub(super) struct RecoveryActionSpec {
    pub(super) gate: Option<(&'static str, &'static str)>,
    pub(super) target_phase: &'static str,
}

pub(super) struct RecoveryRoute {
    pub(super) action: &'static str,
    pub(super) failures: Vec<&'static str>,
    pub(super) spec: RecoveryActionSpec,
}

pub(super) fn recovery_route_for_action(action: &str) -> Option<RecoveryRoute> {
    let action = RecoveryAction::parse(action)?;
    Some(route_for_action(action))
}

pub(super) fn recovery_route_for_failure(failure: &str) -> Option<RecoveryRoute> {
    let failure = FailureClass::parse(failure)?;
    Some(route_for_action(recovery_action_for(failure)))
}

pub(super) fn recovery_action_for_failure(failure: &str) -> Option<&'static str> {
    let failure = FailureClass::parse(failure)?;
    Some(recovery_action_for(failure).name())
}

pub(super) fn recovery_action_spec(action: &str) -> Option<RecoveryActionSpec> {
    Some(spec_for_action(RecoveryAction::parse(action)?))
}

fn route_for_action(action: RecoveryAction) -> RecoveryRoute {
    RecoveryRoute {
        action: action.name(),
        failures: FailureClass::ALL
            .into_iter()
            .filter(|failure| recovery_action_for(*failure) == action)
            .map(FailureClass::name)
            .collect(),
        spec: spec_for_action(action),
    }
}

fn spec_for_action(action: RecoveryAction) -> RecoveryActionSpec {
    RecoveryActionSpec {
        gate: action
            .repaired_gate()
            .zip(action.produced_evidence())
            .map(|(gate, evidence)| (stable_name(gate), stable_name(evidence))),
        target_phase: stable_name(action.target()),
    }
}

fn stable_name(value: impl std::fmt::Debug) -> &'static str {
    Box::leak(format!("{value:?}").into_boxed_str())
}
