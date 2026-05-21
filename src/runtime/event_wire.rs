//! Wire-level event classification and transition validation.
//!
//! Adapts concepts from canon-runtime-events wire.rs to the ai kernel EventKind
//! without replacing kernel::ControlEvent as the truth authority.
//!
//! WireEventClass partitions kernel EventKind into observable semantic groups
//! that API and MCP boundaries can inspect without access to the full State.

use crate::kernel::{ControlEvent, EventKind, Phase};
use crate::runtime::transition_table::TRANSITIONS;

/// Observable semantic class of a kernel event at the wire boundary.
///
/// - `Control`: lifecycle-advancing events that move the agent through phases
/// - `Effect`: observational side-effects that record evidence without advancing phase
/// - `Terminal`: lifecycle-ending events (success, halt, convergence-failure)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WireEventClass {
    Control,
    Effect,
    Terminal,
}

impl WireEventClass {
    pub fn as_str(self) -> &'static str {
        match self {
            WireEventClass::Control => "control",
            WireEventClass::Effect => "effect",
            WireEventClass::Terminal => "terminal",
        }
    }
}

impl std::fmt::Display for WireEventClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Map a kernel EventKind to its wire-level class.
pub fn wire_class(kind: EventKind) -> WireEventClass {
    match kind {
        EventKind::Advanced | EventKind::Blocked | EventKind::Failed | EventKind::Recovered => {
            WireEventClass::Control
        }
        EventKind::Persisted | EventKind::Learned => WireEventClass::Effect,
        EventKind::Completed => WireEventClass::Terminal,
    }
}

/// Returns `true` if the `(from, to, kind)` triple appears in the canonical
/// transition table. Rejects transitions not sanctioned by the kernel.
///
/// Use at API/MCP boundaries to validate inbound event claims before they
/// reach the durable TLog writer.
pub fn wire_transition_allowed(from: Phase, to: Phase, kind: EventKind) -> bool {
    TRANSITIONS
        .iter()
        .any(|t| t.from == from && t.to == to && t.kind == kind)
}

/// Compact wire-level summary of a `ControlEvent` — safe to log or transmit
/// without carrying the full `State` payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireEventSummary {
    pub seq: u64,
    pub from: Phase,
    pub to: Phase,
    pub kind: EventKind,
    pub class: WireEventClass,
    pub self_hash: u64,
}

impl WireEventSummary {
    pub fn from_event(event: &ControlEvent) -> Self {
        WireEventSummary {
            seq: event.seq,
            from: event.from,
            to: event.to,
            kind: event.kind,
            class: wire_class(event.kind),
            self_hash: event.self_hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::Phase;

    #[test]
    fn wire_class_control_events() {
        assert_eq!(wire_class(EventKind::Advanced), WireEventClass::Control);
        assert_eq!(wire_class(EventKind::Blocked), WireEventClass::Control);
        assert_eq!(wire_class(EventKind::Failed), WireEventClass::Control);
        assert_eq!(wire_class(EventKind::Recovered), WireEventClass::Control);
    }

    #[test]
    fn wire_class_effect_events() {
        assert_eq!(wire_class(EventKind::Persisted), WireEventClass::Effect);
        assert_eq!(wire_class(EventKind::Learned), WireEventClass::Effect);
    }

    #[test]
    fn wire_class_terminal_events() {
        assert_eq!(wire_class(EventKind::Completed), WireEventClass::Terminal);
    }

    #[test]
    fn wire_class_as_str() {
        assert_eq!(WireEventClass::Control.as_str(), "control");
        assert_eq!(WireEventClass::Effect.as_str(), "effect");
        assert_eq!(WireEventClass::Terminal.as_str(), "terminal");
    }

    #[test]
    fn wire_transition_allows_known_transitions() {
        // Delta→Invariant Advanced is the first table entry.
        assert!(wire_transition_allowed(
            Phase::Delta,
            Phase::Invariant,
            EventKind::Advanced
        ));
    }

    #[test]
    fn wire_transition_rejects_invented_transition() {
        // Delta→Done with Completed is not a valid kernel transition.
        assert!(!wire_transition_allowed(
            Phase::Delta,
            Phase::Done,
            EventKind::Completed
        ));
    }

    #[test]
    fn wire_transition_rejects_wrong_kind_for_known_from_to() {
        // Delta→Invariant exists for Advanced, not for Failed.
        assert!(!wire_transition_allowed(
            Phase::Delta,
            Phase::Invariant,
            EventKind::Failed
        ));
    }

    #[test]
    fn wire_event_summary_from_event_preserves_fields() {
        use crate::kernel::{
            CapabilityRegistryProjection, Cause, Decision, Evidence, GateSet, Packet,
            RuntimeConfig, SemanticDelta, State,
        };
        let state = State {
            phase: Phase::Invariant,
            gates: GateSet::default(),
            packet: Packet::empty(),
            failure: None,
            recovery_action: None,
            recovery_attempts: 0,
            wave_pending: 0,
            plan_state_hash: 0,
        };
        let event = ControlEvent {
            seq: 3,
            from: Phase::Delta,
            to: Phase::Invariant,
            kind: EventKind::Advanced,
            cause: Cause::Start,
            delta: SemanticDelta::PhaseAdvanced,
            evidence: Evidence::DeltaComputed,
            decision: Decision::Continue,
            failure: None,
            recovery_action: None,
            affected_gate: None,
            runtime_config: RuntimeConfig {
                max_steps: 10,
                max_recovery_attempts: 3,
            },
            state_before: state,
            state_after: state,
            capability_registry_projection: CapabilityRegistryProjection::new(1, 1),
            api_command_id: 0,
            api_command_hash: 0,
            prev_hash: 0,
            self_hash: 0xabc,
        };

        let summary = WireEventSummary::from_event(&event);
        assert_eq!(summary.seq, 3);
        assert_eq!(summary.from, Phase::Delta);
        assert_eq!(summary.to, Phase::Invariant);
        assert_eq!(summary.kind, EventKind::Advanced);
        assert_eq!(summary.class, WireEventClass::Control);
        assert_eq!(summary.self_hash, 0xabc);
    }
}
