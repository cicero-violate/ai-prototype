//! Action execution compatibility facade.

pub mod canon_graph_editor;
pub mod canon_plan;
pub mod canon_read_mailbox;
pub mod canon_score;
pub mod canon_send_agent_message;
pub mod canon_spawn_agent;
pub mod common;
pub mod dispatch;
pub mod host;
pub mod landmarks;
pub mod record;

pub use canon_graph_editor::{
    CANON_GRAPH_APPLY_OPS_TOOL, CANON_GRAPH_AUTO_REFACTOR_CFG_TOOL, CANON_GRAPH_PLAN_CFG_TOOL,
    CANON_GRAPH_PLAN_PATCH_TOOL, CANON_GRAPH_VERIFY_CFG_DELTA_TOOL,
};
pub use canon_plan::{CANON_PLAN_READ_TOOL, CANON_PLAN_UPDATE_TOOL};
pub use canon_read_mailbox::CANON_READ_MAILBOX_TOOL;
pub use canon_score::CANON_SCORE_TOOL;
pub use canon_send_agent_message::CANON_SEND_AGENT_MESSAGE_TOOL;
pub use canon_spawn_agent::{SpawnAgentToolRequest, CANON_SPAWN_AGENT_TOOL};
pub use crate::capability::execution::patch::APPLY_PATCH_TOOL;
pub use crate::capability::execution::shell::SHELL_TOOL;
pub use dispatch::{dispatch_action_request, ActionHost};
pub use host::ActionHost as McpToolHost;
pub use record::{
    append_action_receipt_ndjson, decode_action_receipt_ndjson, encode_action_receipt_ndjson,
    load_action_receipts_ndjson, verify_action_receipts, ActionCallRequest, ActionReceipt,
    LiveActionExecutor, ACTION_RECEIPT_RECORD, ACTION_RECEIPT_SCHEMA_VERSION,
};
