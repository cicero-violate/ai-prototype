//! Observation capability.
//!
//! Observation is the first world-facing evidence producer. It validates that
//! a perceived external signal is non-empty, ordered, and hash-addressable, then
//! submits only `Evidence::InvariantProof` into the kernel.

pub mod record;
pub mod source;

pub use self::record::{
    ObservationCursor, ObservationDecision, ObservationFrame, ObservationFrameKind,
    ObservationRecord, MAX_OBSERVATION_PAYLOAD_BYTES,
};
pub use self::source::{
    decode_observation_cursor_ndjson, encode_observation_cursor_ndjson,
    load_observation_cursor_ndjson, write_observation_cursor_ndjson,
    BoundedLineObservationSource, ObservationIngressBatch, ObservationIngressConfig,
    ObservationIngressDecision, OBSERVATION_CURSOR_RECORD,
    OBSERVATION_CURSOR_SCHEMA_VERSION,
};