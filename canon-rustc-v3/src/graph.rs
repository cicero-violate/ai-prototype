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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub def: Option<SourceSpan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig: Option<FnSig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldDef>,
    /// Extracted invariant clauses for `fn` nodes whose body is a pure boolean
    /// conjunction (e.g. `is_valid`, `is_structurally_valid`).  Each entry is
    /// one `&&`-separated predicate captured at compile time with full type info.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariants: Vec<InvariantClause>,
}

/// One predicate clause extracted from a boolean-conjunction validator fn.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InvariantClause {
    /// Human-readable operator string: "!=", "==", "<", "<=", ">", ">=", "!".
    pub op: String,
    /// Left-hand side of the predicate as source text (e.g. `self.retry_count`).
    pub lhs: String,
    /// Right-hand side as source text, empty for unary `!` predicates.
    pub rhs: String,
    /// Source span of this clause expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<SourceSpan>,
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
