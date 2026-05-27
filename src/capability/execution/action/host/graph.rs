//! Graph native tooling capability.

use serde_json::Value;

use super::{ActionHost, ActionToolOutcome};
use crate::capability::execution::action::canon_graph_editor;

pub fn execute<H: ActionHost>(name: &str, args: &Value, host: &H) -> ActionToolOutcome {
    let workspace = host.workspace();
    ActionToolOutcome::from_value(match name {
        "canon_graph_plan_patch" => canon_graph_editor::run_plan_patch(args, &workspace),
        "canon_graph_plan_cfg" => canon_graph_editor::run_plan_cfg(args, &workspace),
        "canon_graph_apply_ops" => canon_graph_editor::run_apply_ops(args, &workspace),
        "canon_graph_verify_cfg_delta" => {
            canon_graph_editor::run_verify_cfg_delta(args, &workspace)
        }
        "canon_graph_auto_refactor_cfg" => {
            canon_graph_editor::run_auto_refactor_cfg(args, &workspace)
        }
        _ => {
            return ActionToolOutcome::Error(crate::api::action::tool_error(format!(
                "Unknown graph tool: {name}"
            )))
        }
    })
}
