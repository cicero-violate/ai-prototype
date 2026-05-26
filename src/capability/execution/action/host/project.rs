//! Project native tooling capability: scoring and plan operations.

use serde_json::Value;

use super::ActionHost;
use crate::api::action::tool_error;
use crate::capability::execution::action::{
    canon_diagnostics, canon_invariants, canon_plan, canon_score,
};

pub fn execute<H: ActionHost>(name: &str, args: &Value, host: &H) -> Value {
    let workspace = host.workspace();
    match name {
        "canon_score" => canon_score::run(args, &workspace),
        "canon_diagnostics_read" => canon_diagnostics::run_read(args, &workspace),
        "canon_invariants_mine" => canon_invariants::run_mine(args, &workspace),
        "canon_invariants_validate" => canon_invariants::run_validate(args, &workspace),
        "canon_invariants_promote" => canon_invariants::run_promote(args, &workspace),
        "canon_invariants_read" => canon_invariants::run_read(args, &workspace),
        "canon_plan_read" => canon_plan::run_read(args, &workspace),
        "canon_plan_update" => canon_plan::run_update(args, &workspace),
        _ => tool_error(format!("Unknown project tool: {name}")),
    }
}
