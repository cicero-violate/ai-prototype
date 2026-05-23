//! Capability execution surface.
//!
//! This is the migration target for bounded effect execution that produces
//! runtime receipts. The existing `tooling` module is re-exported during the
//! transition so new code can use `capability::execution` without moving every
//! implementation in one diff.

pub mod action;
pub mod graph;
pub mod patch;
pub mod process;
pub mod record;
pub mod shell;

pub use crate::capability::tooling::*;
pub use action::record::{
    append_action_receipt_ndjson, decode_action_receipt_ndjson, encode_action_receipt_ndjson,
    load_action_receipts_ndjson, verify_action_receipts, ActionCallRequest, ActionReceipt,
    LiveActionExecutor, ACTION_RECEIPT_RECORD, ACTION_RECEIPT_SCHEMA_VERSION,
};
