//! Artifact serialization to the state directory.
//!
//! Writes one compact `graph.json` per crate plus an ordered `index.json`.

use crate::graph::CrateGraph;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Summary written into the top-level index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateIndexEntry {
    pub crate_name: String,
    pub captured_at_ms: u64,
    pub node_count: usize,
    pub edge_count: usize,
    pub artifact_dir: String,
}

/// Write the consolidated graph for one crate and update the index.
pub fn write_graph(artifact_root: &Path, crate_name: &str, graph: &CrateGraph) -> Result<()> {
    let crate_dir = artifact_root.join(crate_name);
    let graph_path = crate_dir.join("graph.json");

    fs::create_dir_all(&crate_dir)?;
    let json = serde_json::to_vec_pretty(graph)?;
    write_graph_atomically(&graph_path, &json)?;

    update_index(artifact_root, crate_name, graph, &crate_dir)?;
    Ok(())
}

fn update_index(
    artifact_root: &Path,
    crate_name: &str,
    graph: &CrateGraph,
    crate_dir: &Path,
) -> Result<()> {
    let index_path = artifact_root.join("index.json");

    let mut index = read_index(&index_path)?;

    let captured_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    index.insert(
        crate_name.to_string(),
        CrateIndexEntry {
            crate_name: crate_name.to_string(),
            captured_at_ms,
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
            artifact_dir: relative_artifact_dir(artifact_root, crate_dir, crate_name),
        },
    );

    let json = serde_json::to_vec_pretty(&index)?;
    write_index_atomically(&index_path, &json)?;
    Ok(())
}

fn read_index(index_path: &Path) -> Result<BTreeMap<String, CrateIndexEntry>> {
    if !index_path.exists() {
        return Ok(BTreeMap::new());
    }

    let bytes = fs::read(index_path)
        .with_context(|| format!("failed to read index {}", index_path.display()))?;
    serde_json::from_slice(&bytes)
        .with_context(|| format!("invalid index json at {}", index_path.display()))
}

fn write_graph_atomically(graph_path: &Path, json: &[u8]) -> Result<()> {
    let tmp_path = graph_path.with_extension("json.tmp");
    fs::write(&tmp_path, json)
        .with_context(|| format!("failed to write temp graph {}", tmp_path.display()))?;
    fs::rename(&tmp_path, graph_path).with_context(|| {
        format!(
            "failed to replace graph {} with {}",
            graph_path.display(),
            tmp_path.display()
        )
    })?;
    Ok(())
}

fn relative_artifact_dir(artifact_root: &Path, crate_dir: &Path, crate_name: &str) -> String {
    crate_dir
        .strip_prefix(artifact_root)
        .ok()
        .map(|path| path.to_string_lossy().into_owned())
        .filter(|path| !path.is_empty())
        .unwrap_or_else(|| crate_name.to_string())
}

fn write_index_atomically(index_path: &Path, json: &[u8]) -> Result<()> {
    let tmp_path = index_path.with_extension("json.tmp");
    fs::write(&tmp_path, json)
        .with_context(|| format!("failed to write temp index {}", tmp_path.display()))?;
    fs::rename(&tmp_path, index_path).with_context(|| {
        format!(
            "failed to replace index {} with {}",
            index_path.display(),
            tmp_path.display()
        )
    })?;
    Ok(())
}
