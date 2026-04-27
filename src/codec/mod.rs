//! Serialization boundary.
//!
//! Codec code encodes and decodes records only. Validation and replay live in runtime.

pub mod ndjson;

pub use self::ndjson::{append_tlog_ndjson, load_tlog_ndjson, write_tlog_ndjson};
