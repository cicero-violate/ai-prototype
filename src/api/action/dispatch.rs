//! Action dispatch planning compatibility facade.

pub use crate::api::mcp::dispatch::*;
pub use crate::api::mcp::dispatch::{
    dispatch_ai_mcp_plan as dispatch_action_plan, AiMcpDispatchPlan as ActionDispatchPlan,
};
