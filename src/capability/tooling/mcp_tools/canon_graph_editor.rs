//! MCP adapter for graph-editor patch generation.

use serde_json::Value;

use crate::runtime::WorkspaceView;

pub const CANON_GRAPH_PLAN_PATCH_TOOL: &str = "canon_graph_plan_patch";
pub const CANON_GRAPH_PLAN_CFG_TOOL: &str = "canon_graph_plan_cfg";
pub const CANON_GRAPH_APPLY_OPS_TOOL: &str = "canon_graph_apply_ops";
pub const CANON_GRAPH_VERIFY_CFG_DELTA_TOOL: &str = "canon_graph_verify_cfg_delta";
pub const CANON_GRAPH_AUTO_REFACTOR_CFG_TOOL: &str = "canon_graph_auto_refactor_cfg";

pub fn run_plan_patch(args: &Value, workspace: &WorkspaceView) -> Value {
    crate::capability::execution::graph::plan_patch_tool(args, workspace)
}

pub fn run_plan_cfg(args: &Value, workspace: &WorkspaceView) -> Value {
    crate::capability::execution::graph::plan_cfg_tool(args, workspace)
}

pub fn run_apply_ops(args: &Value, workspace: &WorkspaceView) -> Value {
    crate::capability::execution::graph::apply_ops_tool(args, workspace)
}

pub fn run_verify_cfg_delta(args: &Value, workspace: &WorkspaceView) -> Value {
    crate::capability::execution::graph::verify_cfg_delta_tool(args, workspace)
}

pub fn run_auto_refactor_cfg(args: &Value, workspace: &WorkspaceView) -> Value {
    crate::capability::execution::graph::auto_refactor_cfg_tool(args, workspace)
}
