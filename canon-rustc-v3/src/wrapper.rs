//! `rustc_driver::Callbacks` implementation for the minimal witness graph.

use crate::flags::{
    default_artifact_dir, find_flag_value, find_flag_values, is_cargo_registry_path,
    should_capture_crate, workspace_root_from_output_dir,
};
use crate::graph::{CrateGraph, GraphEdge, GraphMeta, GraphNode};
use crate::{emit, facts, hir, mir};
use rustc_driver::{Callbacks, Compilation};
use rustc_middle::ty::TyCtxt;
use rustc_span::FileName;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::SystemTime;

const GRAPH_SCHEMA_VERSION: u32 = 16;
const RECEIPT_SCHEMA_VERSION: u32 = 1;

pub struct AnalysisCallbacks {
    output_dir: PathBuf,
    crate_name: Option<String>,
    crate_types: Vec<String>,
}

impl AnalysisCallbacks {
    pub fn new(argv: &[String]) -> Self {
        let crate_name = find_flag_value(argv, "--crate-name");
        let crate_types = find_flag_values(argv, "--crate-type");
        let output_dir = find_flag_value(argv, "--out-dir")
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        Self {
            output_dir,
            crate_name,
            crate_types,
        }
    }
}

impl Callbacks for AnalysisCallbacks {
    fn after_analysis<'tcx>(
        &mut self,
        _compiler: &rustc_interface::interface::Compiler,
        tcx: TyCtxt<'tcx>,
    ) -> Compilation {
        if is_cargo_registry_path(&self.output_dir) {
            return Compilation::Continue;
        }

        let workspace_root = workspace_root_from_output_dir(&self.output_dir);
        if !should_capture_crate(self.crate_name.as_deref(), &self.crate_types) {
            return Compilation::Continue;
        }
        if !is_workspace_crate(tcx, &workspace_root) {
            return Compilation::Continue;
        }

        let crate_name = self.crate_name.as_deref().unwrap_or("unknown");
        let primary_type = self
            .crate_types
            .first()
            .map(String::as_str)
            .unwrap_or("lib");
        let output_key = if primary_type == "bin" {
            format!("{crate_name}__bin")
        } else {
            crate_name.to_string()
        };
        let artifact_root = default_artifact_dir(&workspace_root);
        let graph = build_graph(tcx, crate_name, &workspace_root);

        if let Err(err) = emit::write_graph(&artifact_root, &output_key, &graph) {
            eprintln!("canon-rustc-v3: artifact write failed for {output_key}: {err:?}");
        } else {
            eprintln!(
                "canon-rustc-v3: captured {output_key} witness: {} nodes, {} facts, graph_hash {}",
                graph.nodes.len(),
                graph.edges.len(),
                graph.meta.graph_hash,
            );
        }

        Compilation::Continue
    }
}

fn build_graph(tcx: TyCtxt<'_>, crate_name: &str, workspace_root: &PathBuf) -> CrateGraph {
    let hir_out = hir::collect_hir(tcx, workspace_root);
    let mir_out = mir::collect_mir(tcx);

    let nodes = hir_out.nodes;
    let edges = dedup_allowed_edges(hir_out.edges.into_iter().chain(mir_out.edges));

    let intents = function_intents(&nodes, &edges);
    let graph_hash = graph_hash(&nodes, &edges);
    let intent_hash = intent_hash(&intents);
    let risk_hash = risk_hash(&edges);
    let receipt_hash = receipt_hash(
        crate_name,
        nodes.len(),
        edges.len(),
        &graph_hash,
        &intent_hash,
        &risk_hash,
    );
    let captured_at_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    CrateGraph {
        meta: GraphMeta {
            crate_name: crate_name.to_string(),
            captured_at_ms,
            node_count: nodes.len(),
            edge_count: edges.len(),
            schema_version: GRAPH_SCHEMA_VERSION,
            receipt_hash,
            graph_hash,
            intent_hash,
            risk_hash,
        },
        nodes,
        edges,
        intents,
    }
}

fn dedup_allowed_edges(edges: impl Iterator<Item = GraphEdge>) -> Vec<GraphEdge> {
    let set: BTreeSet<GraphEdge> = edges
        .filter(|edge| facts::allowed_relation(&edge.relation))
        .filter(|edge| !edge.from.is_empty() && !edge.to.is_empty())
        .collect();
    set.into_iter().collect()
}

#[derive(Serialize)]
struct StableGraph<'a> {
    nodes: &'a BTreeMap<String, GraphNode>,
    edges: &'a [GraphEdge],
}

#[derive(Serialize)]
struct ReceiptEnvelope<'a> {
    schema_version: u32,
    graph_schema_version: u32,
    crate_name: &'a str,
    node_count: usize,
    edge_count: usize,
    graph_hash: &'a str,
    intent_hash: &'a str,
    risk_hash: &'a str,
}

fn graph_hash(nodes: &BTreeMap<String, GraphNode>, edges: &[GraphEdge]) -> String {
    let stable = StableGraph { nodes, edges };
    let bytes = stable_json_bytes(&stable, "graph");
    stable_hash(&bytes)
}

fn risk_hash(edges: &[GraphEdge]) -> String {
    let risk_edges: Vec<&GraphEdge> = edges
        .iter()
        .filter(|edge| facts::risk_relation(&edge.relation))
        .collect();
    let bytes = stable_json_bytes(&risk_edges, "risk edges");
    stable_hash(&bytes)
}

fn function_intents(
    nodes: &BTreeMap<String, GraphNode>,
    edges: &[GraphEdge],
) -> BTreeMap<String, String> {
    nodes
        .values()
        .filter(|node| node.kind == "fn")
        .map(|node| {
            (
                node.path.clone(),
                intent_for(node.path.as_str(), edges).to_string(),
            )
        })
        .collect()
}

fn intent_hash(intents: &BTreeMap<String, String>) -> String {
    stable_hash(&stable_json_bytes(intents, "intents"))
}

fn receipt_hash(
    crate_name: &str,
    node_count: usize,
    edge_count: usize,
    graph_hash: &str,
    intent_hash: &str,
    risk_hash: &str,
) -> String {
    let envelope = ReceiptEnvelope {
        schema_version: RECEIPT_SCHEMA_VERSION,
        graph_schema_version: GRAPH_SCHEMA_VERSION,
        crate_name,
        node_count,
        edge_count,
        graph_hash,
        intent_hash,
        risk_hash,
    };
    stable_hash(&stable_json_bytes(&envelope, "receipt envelope"))
}

fn stable_json_bytes<T: Serialize + ?Sized>(value: &T, label: &str) -> Vec<u8> {
    match serde_json::to_vec(value) {
        Ok(bytes) => bytes,
        Err(err) => format!("canon-rustc-v3:{label}:stable-json-error:{err}").into_bytes(),
    }
}

fn intent_for(function: &str, edges: &[GraphEdge]) -> &'static str {
    let name = function.to_ascii_lowercase();
    let mut relations = BTreeSet::new();
    for edge in edges.iter().filter(|edge| edge.from == function) {
        relations.insert(edge.relation.as_str());
    }

    if name.contains("boundary") {
        "boundary"
    } else if relations.contains("unsafe") {
        "unsafe"
    } else if relations.contains("io") {
        "io"
    } else if relations.contains("mut") {
        "mutation"
    } else if relations.contains("panic") {
        "validation"
    } else if relations.contains("alloc") || relations.contains("call") {
        "orchestration"
    } else {
        "pure"
    }
}

fn stable_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_workspace_crate(tcx: TyCtxt<'_>, workspace_root: &PathBuf) -> bool {
    let workspace_root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.clone());

    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let manifest_path = PathBuf::from(&manifest_dir)
            .canonicalize()
            .unwrap_or_else(|_| PathBuf::from(&manifest_dir));
        return manifest_path.starts_with(&workspace_root)
            && !manifest_path.starts_with(workspace_root.join("target"));
    }

    let source_map = tcx.sess.source_map();
    let mut local_files = BTreeSet::new();
    for sf in source_map.files().iter() {
        if let FileName::Real(rn) = &sf.name {
            if let Some(p) = rn.local_path() {
                local_files.insert(p.to_path_buf());
            }
        }
    }
    local_files.iter().any(|p| {
        let abs = p.canonicalize().unwrap_or_else(|_| p.clone());
        abs.starts_with(&workspace_root)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{GraphEdge, GraphNode};

    fn sample_node(path: &str) -> GraphNode {
        GraphNode {
            def_id: "0:1".to_string(),
            path: path.to_string(),
            kind: "fn".to_string(),
            def: None,
            source_text: None,
            sig: None,
            fields: Vec::new(),
        }
    }

    fn edge(relation: &str, from: &str, to: &str) -> GraphEdge {
        GraphEdge {
            relation: relation.to_string(),
            from: from.to_string(),
            to: to.to_string(),
            span: None,
        }
    }

    #[test]
    fn schema_and_receipt_versions_are_current_contract() {
        assert_eq!(GRAPH_SCHEMA_VERSION, 16);
        assert_eq!(RECEIPT_SCHEMA_VERSION, 1);
    }

    #[test]
    fn dedup_allowed_edges_filters_unknown_and_empty_edges() {
        let edges = vec![
            edge("call", "crate::a", "crate::b"),
            edge("call", "crate::a", "crate::b"),
            edge("unknown", "crate::a", "crate::b"),
            edge("mut", "", "fact::mut"),
            edge("io", "crate::a", ""),
        ];

        let kept = dedup_allowed_edges(edges.into_iter());

        assert_eq!(kept, vec![edge("call", "crate::a", "crate::b")]);
    }

    #[test]
    fn graph_hash_is_stable_for_btree_ordered_inputs() {
        let edge_ab = edge("call", "crate::a", "crate::b");
        let edge_mut = edge("mut", "crate::b", "fact::mut");

        let mut nodes_a = BTreeMap::new();
        nodes_a.insert("crate::b".to_string(), sample_node("crate::b"));
        nodes_a.insert("crate::a".to_string(), sample_node("crate::a"));

        let mut nodes_b = BTreeMap::new();
        nodes_b.insert("crate::a".to_string(), sample_node("crate::a"));
        nodes_b.insert("crate::b".to_string(), sample_node("crate::b"));

        let edges = vec![edge_ab, edge_mut];

        assert_eq!(graph_hash(&nodes_a, &edges), graph_hash(&nodes_b, &edges));
    }

    #[test]
    fn receipt_hash_changes_when_graph_schema_changes() {
        let baseline = receipt_hash("demo", 1, 1, "graph", "intent", "risk");

        let envelope = ReceiptEnvelope {
            schema_version: RECEIPT_SCHEMA_VERSION,
            graph_schema_version: GRAPH_SCHEMA_VERSION + 1,
            crate_name: "demo",
            node_count: 1,
            edge_count: 1,
            graph_hash: "graph",
            intent_hash: "intent",
            risk_hash: "risk",
        };
        let changed = stable_hash(&stable_json_bytes(&envelope, "test receipt envelope"));

        assert_ne!(baseline, changed);
    }
}
