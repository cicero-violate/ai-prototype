//! Serialization boundary.
//!
//! Codec code encodes and decodes records only. Validation and replay live in runtime.

pub mod ndjson;

#[cfg(feature = "binary-tlog")]
pub mod binary_tlog;
#[cfg(feature = "binary-tlog")]
pub use self::binary_tlog::{is_binary_tlog, read_binary_tlog, BinaryTlogWriter, SegmentConfig};

pub use self::ndjson::{
    append_plan_patch_record_ndjson, append_tlog_ndjson, decode_control_event_ndjson,
    decode_plan_patch_record_ndjson, decode_tlog_ndjson_str, encode_control_event_ndjson,
    encode_plan_patch_record_ndjson, encode_tlog_ndjson_string, load_plan_patch_records_ndjson,
    load_tlog_ndjson, write_tlog_ndjson, PlanPatchTlogRecord, TLOG_RECORD_EVENT,
    TLOG_SCHEMA_VERSION,
};
