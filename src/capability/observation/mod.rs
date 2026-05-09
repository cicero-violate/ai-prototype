//! Observation capability.
//!
//! Observation is the first world-facing evidence producer. It validates that
//! a perceived external signal is non-empty, ordered, and hash-addressable, then
//! submits only `Evidence::InvariantProof` into the kernel.

pub mod record;
pub mod source;

pub use self::record::{
    MAX_OBSERVATION_PAYLOAD_BYTES, ObservationCursor, ObservationDecision, ObservationFrame,
    ObservationFrameKind, ObservationRecord,
};
pub use self::source::{
    BoundedLineObservationSource, OBSERVATION_CURSOR_RECORD, OBSERVATION_CURSOR_SCHEMA_VERSION,
    OBSERVATION_INGRESS_RECEIPT_RECORD, OBSERVATION_INGRESS_RECEIPT_SCHEMA_VERSION,
    ObservationIngressBatch, ObservationIngressConfig, ObservationIngressDecision,
    ObservationIngressReceipt, decode_observation_cursor_ndjson, encode_observation_cursor_ndjson,
    load_observation_cursor_ndjson, write_observation_cursor_ndjson,
};
