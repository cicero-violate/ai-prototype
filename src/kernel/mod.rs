//! Pure canonical kernel types and packet/state invariants.
//!
//! This module is intentionally free of filesystem I/O and runner policy.
//! It owns the stable state model that reducer, recovery, codec, writer, and
//! verifier code operate on.

mod capability;
mod config;
mod event;
mod gate;
mod packet;
mod phase;
mod recovery;
mod state;

pub use self::capability::CapabilityRegistryProjection;
pub use self::config::RuntimeConfig;
pub use self::event::{Cause, ControlEvent, Decision, EventKind, SemanticDelta, TLog};
pub use self::gate::{
    Evidence, Gate, GateId, GateSet, GateStatus, EXECUTION_GATE_ORDER, GATE_ORDER,
};
pub(crate) fn mix(mut h: u64, x: u64) -> u64 {
    h ^= x;
    h = h.wrapping_mul(0x100000001b3);
    h
}
pub use self::packet::Packet;
pub use self::phase::{Phase, PHASES};
pub use self::recovery::{FailureClass, RecoveryAction};
pub use self::state::State;
