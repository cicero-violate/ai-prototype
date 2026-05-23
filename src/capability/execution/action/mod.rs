//! Action execution compatibility facade.

pub mod dispatch;
pub mod record;

pub use crate::capability::tooling::mcp_tools::*;
pub use dispatch::{dispatch_action_request, ActionHost};
pub use record::{
    append_action_receipt_ndjson, decode_action_receipt_ndjson, encode_action_receipt_ndjson,
    load_action_receipts_ndjson, verify_action_receipts, ActionCallRequest, ActionReceipt,
    LiveActionExecutor, ACTION_RECEIPT_RECORD, ACTION_RECEIPT_SCHEMA_VERSION,
};
