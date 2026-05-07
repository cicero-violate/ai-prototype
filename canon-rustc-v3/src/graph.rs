//! Minimal semantic witness graph.
//!
//! The public schema covers: `fn`, `trait`, `impl`, `struct`, `enum`,
//! `ty_alias` nodes; `call`, `impl`, `mut`, `io`, `unsafe`, `panic`,
//! `alloc`, `use` edges.  Replay hashes are metadata, not semantic edges.

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
    /// Full source span extended backwards to cover outer attributes (Gap 3).
    /// None for compiler-generated or macro-expanded items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub def: Option<SourceSpan>,
    /// Verbatim source text [def.lo .. def.hi], including outer attributes.
    /// None when `def` is None.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_text: Option<String>,
    /// Structured function signature for `fn` nodes (Gap 5).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<FnSig>,
    /// Field definitions for `struct` nodes; variant names for `enum` nodes (Gap 2).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldDef>,
}

/// Structured function signature.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FnSig {
    pub params: Vec<FnParam>,
    pub return_ty: String,
}

/// One parameter in a function signature.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FnParam {
    pub name: String,
    pub ty: String,
}

/// One field in a struct, or one variant name in an enum.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FieldDef {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GraphEdge {
    pub relation: String,
    pub from: String,
    pub to: String,
    /// Source location of the call expression (Gap 1).
    /// Populated for `call` edges; None for all other relations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SourceSpan>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct SourceSpan {
    pub file: String,
    pub line: u32,
    pub col: u32,
    pub lo: u32,
    pub hi: u32,
}
