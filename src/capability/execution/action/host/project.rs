//! Project native tooling capability: scoring and plan operations.

use serde_json::Value;

use super::ActionHost;
use crate::api::action::tool_error;
use crate::capability::execution::action::{canon_plan, canon_score};

pub fn execute<H: ActionHost>(name: &str, args: &Value, host: &H) -> Value {
    let workspace = host.workspace();
    match name {
        "canon_score" => canon_score::run(args, &workspace),
        "canon_plan_read" => canon_plan::run_read(args, &workspace),
        "canon_plan_update" => canon_plan::run_update(args, &workspace),
        _ => tool_error(format!("Unknown project tool: {name}")),
    }
}
