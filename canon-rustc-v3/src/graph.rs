//! Minimal semantic witness graph.
//!
//! The public schema is intentionally limited to GOAL.md facts:
//! `fn`, `trait`, `impl`, `call`, `mut`, `io`, `unsafe`, `panic`, and `alloc`.
//! Replay hashes are metadata, not semantic edges.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CrateGraph {
    pub meta: GraphMeta,
    pub nodes: BTreeMap<String, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub intents: BTreeMap<String, String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GraphMeta {
    pub crate_name: String,
    pub captured_at_ms: u64,
    pub node_count: usize,
    pub edge_count: usize,
    pub schema_version: u32,
    pub receipt_hash: String,
    pub graph_hash: String,
    pub intent_hash: String,
    pub risk_hash: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphNode {
    pub def_id: String,
    pub path: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub def: Option<SourceSpan>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GraphEdge {
    pub relation: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpan {
    pub file: String,
    pub line: u32,
    pub col: u32,
    pub lo: u32,
    pub hi: u32,
}
