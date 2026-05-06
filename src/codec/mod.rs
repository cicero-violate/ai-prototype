//! Serialization boundary.
//!
//! Codec code encodes and decodes records only. Validation and replay live in runtime.

pub mod ndjson;

pub use self::ndjson::{
    append_tlog_ndjson, decode_control_event_ndjson, decode_tlog_ndjson_str,
    encode_control_event_ndjson, encode_tlog_ndjson_string, load_tlog_ndjson, write_tlog_ndjson,
    TLOG_RECORD_EVENT, TLOG_SCHEMA_VERSION,
};
