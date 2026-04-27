//! Observation capability.
//!
//! Observation is the first world-facing evidence producer. It validates that
//! a perceived external signal is non-empty, ordered, and hash-addressable, then
//! submits only `Evidence::InvariantProof` into the kernel.

pub mod record;

pub use self::record::{
    ObservationCursor, ObservationDecision, ObservationFrame, ObservationFrameKind,
    ObservationRecord, MAX_OBSERVATION_PAYLOAD_BYTES,
};