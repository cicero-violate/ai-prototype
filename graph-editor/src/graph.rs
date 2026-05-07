//! Re-exports the canonical graph types from canon-rustc-v3.
//!
//! graph-editor and the wrapper share the same Cargo workspace so their
//! CrateGraph, GraphNode, GraphEdge, SourceSpan, and GraphMeta are
//! literally the same types — no conversion needed.

pub use canon_rustc_v3::graph::{
    CrateGraph, FieldDef, FnParam, FnSig, GraphEdge, GraphMeta, GraphNode, SourceSpan,
};

/// Schema version this editor understands.  Must match the wrapper.
pub const SCHEMA_VERSION: u32 = 12;

/// Deserialise a `graph.json` byte slice and validate its schema version.
pub fn parse_graph(bytes: &[u8]) -> anyhow::Result<CrateGraph> {
    let g: CrateGraph = serde_json::from_slice(bytes)?;
    if g.meta.schema_version != SCHEMA_VERSION {
        anyhow::bail!(
            "unsupported schema_version {}: this editor supports {}",
            g.meta.schema_version,
            SCHEMA_VERSION
        );
    }
    Ok(g)
}
