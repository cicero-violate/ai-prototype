//! Schema-v16 graph.json types used by graph-editor.
//!
//! These are intentionally local to graph-editor so the editor remains usable
//! even when the rustc wrapper crate is not checked out as a sibling crate.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Schema version this editor understands. Must match canon-rustc-v3 output.
pub const SCHEMA_VERSION: u32 = 16;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrateGraph {
    pub meta: GraphMeta,
    #[serde(default)]
    pub nodes: BTreeMap<String, GraphNode>,
    #[serde(default)]
    pub edges: Vec<GraphEdge>,
    #[serde(default)]
    pub intents: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphMeta {
    #[serde(default)]
    pub crate_name: String,
    #[serde(default)]
    pub captured_at_ms: u64,
    #[serde(default)]
    pub node_count: usize,
    #[serde(default)]
    pub edge_count: usize,
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub receipt_hash: String,
    #[serde(default)]
    pub graph_hash: String,
    #[serde(default)]
    pub intent_hash: String,
    #[serde(default)]
    pub risk_hash: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphNode {
    #[serde(default)]
    pub def_id: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub def: Option<SourceSpan>,
    #[serde(default)]
    pub source_text: Option<String>,
    #[serde(default)]
    pub sig: Option<FnSig>,
    #[serde(default)]
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GraphEdge {
    pub relation: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub span: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SourceSpan {
    pub file: String,
    pub line: u32,
    pub col: u32,
    pub lo: u32,
    pub hi: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FnSig {
    #[serde(default)]
    pub params: Vec<FnParam>,
    #[serde(default)]
    pub ret: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FnParam {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub ty: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldDef {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub ty: String,
}

/// Deserialise a `graph.json` byte slice and validate its schema version.
pub fn parse_graph(bytes: &[u8]) -> anyhow::Result<CrateGraph> {
    let graph: CrateGraph = serde_json::from_slice(bytes)?;
    if graph.meta.schema_version != SCHEMA_VERSION {
        anyhow::bail!(
            "unsupported schema_version {}: this editor supports {}",
            graph.meta.schema_version,
            SCHEMA_VERSION
        );
    }
    Ok(graph)
}
