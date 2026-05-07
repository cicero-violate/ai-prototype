//! Typed mutation operations against graph.json.
//!
//! Each op carries the minimum fields needed to (a) locate the target in the
//! graph and (b) generate a source patch or a direct graph override.
//! `expected_lo` / `expected_hi` fields act as stale-op guards: if the byte
//! offsets in the current graph differ, the op is rejected rather than
//! silently patching the wrong lines.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum GraphMutationOp {
    /// Remove a definition from source entirely.
    ///
    /// Produces a source patch that excises the lines covering [lo, hi].
    /// Also removes all edges incident on `path` from a derived new graph.
    RemoveNode {
        /// Fully-qualified path as it appears in graph.json `nodes`.
        path: String,
        /// Must match `nodes[path].def.lo` — stale-op guard.
        expected_lo: u32,
        /// Must match `nodes[path].def.hi` — stale-op guard.
        expected_hi: u32,
    },

    /// Insert an attribute line immediately before the node's definition.
    ///
    /// Produces a source patch that inserts `attr` (e.g. `#[must_use]`)
    /// at the item's indentation level, one line above `def.line`.
    AddAttribute {
        /// Fully-qualified path as it appears in graph.json `nodes`.
        path: String,
        /// Attribute text including `#[...]`, e.g. `"#[must_use]"`.
        attr: String,
        /// Must match `nodes[path].def.lo` — stale-op guard.
        expected_lo: u32,
    },

    /// Override the intent label of a node directly in graph.json.
    ///
    /// Does NOT produce a source patch.  Writes a direct override into the
    /// `intents` map of the mutated graph.  Useful for annotation-only
    /// reclassifications before touching source.
    RetypeIntent {
        /// Fully-qualified path as it appears in graph.json `intents`.
        path: String,
        /// New label, e.g. `"pure"` or `"mutation"`.
        new_label: String,
    },
    // RemoveEdge is intentionally absent: edges carry no source-location
    // information in graph.json, so automatic patch generation is not
    // possible.  Remove the call site in source first; re-capture will
    // drop the edge.
}

impl GraphMutationOp {
    pub fn path(&self) -> &str {
        match self {
            Self::RemoveNode { path, .. } => path,
            Self::AddAttribute { path, .. } => path,
            Self::RetypeIntent { path, .. } => path,
        }
    }
}

/// A set of ops loaded from a JSON array file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpsFile(pub Vec<GraphMutationOp>);

impl OpsFile {
    pub fn from_json(bytes: &[u8]) -> anyhow::Result<Self> {
        let ops: Vec<GraphMutationOp> = serde_json::from_slice(bytes)?;
        Ok(OpsFile(ops))
    }
}
