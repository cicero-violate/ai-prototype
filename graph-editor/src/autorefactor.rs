//! Auto-refactor operation planner.
//!
//! This module converts graph relations emitted by canon-rustc-v3 schema v16
//! (`phase`, `similar`, `provider`) into deterministic graph-editor operation
//! records.  The operation records are intentionally conservative: they bind to
//! graph paths and source spans, carry stale-op guards where source spans exist,
//! and are safe for an editor to reject when signatures or source regions do not
//! match.

use crate::graph::CrateGraph;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const PHASE_PREFIX: &str = "phase::";
const PROVIDER_PREFIX: &str = "provider::";
const SIMILAR_PREFIX: &str = "similar::";
const SIMILARITY_THRESHOLD: f64 = 0.75;
const SPLIT_FANOUT_THRESHOLD: usize = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRefactorPlan {
    pub schema_version: u32,
    pub graph_schema_version: u32,
    pub crate_name: String,
    pub operation_count: usize,
    pub split_surface: Vec<SplitSurface>,
    pub merge_surface: Vec<MergeSurface>,
    pub canonicalize_surface: Vec<CanonicalizeSurface>,
    pub operations: Vec<AutoRefactorOp>,
    pub verification: AutoRefactorVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRefactorVerification {
    pub status: String,
    pub required_after_apply_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitSurface {
    pub fn_path: String,
    pub fan_out: usize,
    pub phases: Vec<String>,
    pub rank: usize,
    pub recommended_split_boundaries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeSurface {
    pub module: String,
    pub members: Vec<String>,
    pub similarity: f64,
    pub recommended_canonical_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalizeSurface {
    pub pair: Vec<String>,
    pub similarity: f64,
    pub providers: BTreeMap<String, Vec<String>>,
    pub recommended_trait_boundary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "PascalCase")]
pub enum AutoRefactorOp {
    SplitFn {
        id: String,
        fn_path: String,
        expected_lo: Option<u32>,
        expected_hi: Option<u32>,
        split_boundaries: Vec<String>,
        generated_names: Vec<String>,
        delegate_strategy: String,
    },
    MergeFns {
        id: String,
        members: Vec<String>,
        canonical_fn: String,
        canonical_name: String,
        similarity: f64,
        replacement_strategy: String,
    },
    ExtractTrait {
        id: String,
        functions: Vec<String>,
        trait_name: String,
        providers: BTreeMap<String, Vec<String>>,
        similarity: f64,
        extraction_strategy: String,
    },
}

pub fn plan(graph: &CrateGraph) -> AutoRefactorPlan {
    let fanout = call_fanout(graph);
    let phases = phases_by_fn(graph);
    let providers = providers_by_fn(graph);
    let similarities = similarity_pairs(graph);

    let split_surface = split_surface(&fanout, &phases);
    let merge_surface = merge_surface(&similarities);
    let canonicalize_surface = canonicalize_surface(&similarities, &providers);

    let mut operations = Vec::new();
    for surface in &split_surface {
        let span = graph.nodes.get(&surface.fn_path).and_then(|node| node.def.as_ref());
        let generated_names = surface
            .phases
            .iter()
            .map(|phase| format!("{}__{}", short_name(&surface.fn_path), phase))
            .collect::<Vec<_>>();
        let payload = format!("SplitFn:{}:{:?}", surface.fn_path, surface.recommended_split_boundaries);
        operations.push(AutoRefactorOp::SplitFn {
            id: stable_id(&payload),
            fn_path: surface.fn_path.clone(),
            expected_lo: span.map(|span| span.lo),
            expected_hi: span.map(|span| span.hi),
            split_boundaries: surface.recommended_split_boundaries.clone(),
            generated_names,
            delegate_strategy: "preserve_original_signature".to_string(),
        });
    }
    for surface in &merge_surface {
        let canonical_fn = surface.members.iter().min().cloned().unwrap_or_default();
        let payload = format!("MergeFns:{:?}:{}", surface.members, surface.similarity);
        operations.push(AutoRefactorOp::MergeFns {
            id: stable_id(&payload),
            members: surface.members.clone(),
            canonical_fn,
            canonical_name: surface.recommended_canonical_name.clone(),
            similarity: surface.similarity,
            replacement_strategy: "thin_wrappers_first".to_string(),
        });
    }
    for surface in &canonicalize_surface {
        let payload = format!("ExtractTrait:{:?}:{}", surface.pair, surface.similarity);
        operations.push(AutoRefactorOp::ExtractTrait {
            id: stable_id(&payload),
            functions: surface.pair.clone(),
            trait_name: surface.recommended_trait_boundary.clone(),
            providers: surface.providers.clone(),
            similarity: surface.similarity,
            extraction_strategy: "trait_with_provider_specific_impls".to_string(),
        });
    }

    AutoRefactorPlan {
        schema_version: 1,
        graph_schema_version: graph.meta.schema_version,
        crate_name: graph.meta.crate_name.clone(),
        operation_count: operations.len(),
        split_surface,
        merge_surface,
        canonicalize_surface,
        operations,
        verification: AutoRefactorVerification {
            status: "planned".to_string(),
            required_after_apply_checks: vec![
                "cargo check through wrapper".to_string(),
                "recaptured graph has rho_mut not greater than baseline".to_string(),
                "recaptured graph has lower or equal average mutation fan-out".to_string(),
                "recaptured graph has no additional panic surfaces".to_string(),
            ],
        },
    }
}

fn call_fanout(graph: &CrateGraph) -> BTreeMap<String, usize> {
    let mut targets: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for edge in &graph.edges {
        if edge.relation == "call" {
            targets.entry(edge.from.clone()).or_default().insert(edge.to.clone());
        }
    }
    targets.into_iter().map(|(k, v)| (k, v.len())).collect()
}

fn phases_by_fn(graph: &CrateGraph) -> BTreeMap<String, Vec<String>> {
    let mut phases: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for edge in &graph.edges {
        if edge.relation == "phase" {
            if let Some(phase) = edge.to.strip_prefix(PHASE_PREFIX) {
                phases.entry(edge.from.clone()).or_default().insert(phase.to_string());
            }
        }
    }
    phases.into_iter().map(|(k, v)| (k, v.into_iter().collect())).collect()
}

fn providers_by_fn(graph: &CrateGraph) -> BTreeMap<String, Vec<String>> {
    let mut providers: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for edge in &graph.edges {
        if edge.relation == "provider" {
            if let Some(provider) = edge.to.strip_prefix(PROVIDER_PREFIX) {
                providers.entry(edge.from.clone()).or_default().insert(provider.to_string());
            }
        }
    }
    providers.into_iter().map(|(k, v)| (k, v.into_iter().collect())).collect()
}

fn similarity_pairs(graph: &CrateGraph) -> Vec<(String, String, f64)> {
    let mut pairs = Vec::new();
    let mut seen = BTreeSet::new();
    for edge in &graph.edges {
        if edge.relation != "similar" {
            continue;
        }
        let Some((score, target)) = parse_similarity(&edge.to) else {
            continue;
        };
        if score < SIMILARITY_THRESHOLD {
            continue;
        }
        let key = ordered_pair(&edge.from, &target);
        if seen.insert(key.clone()) {
            pairs.push((key.0, key.1, score));
        }
    }
    pairs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal).then_with(|| a.0.cmp(&b.0)).then_with(|| a.1.cmp(&b.1)));
    pairs
}

fn parse_similarity(value: &str) -> Option<(f64, String)> {
    let rest = value.strip_prefix(SIMILAR_PREFIX)?;
    let (score, target) = rest.split_once("::")?;
    Some((score.parse().ok()?, target.to_string()))
}

fn split_surface(fanout: &BTreeMap<String, usize>, phases: &BTreeMap<String, Vec<String>>) -> Vec<SplitSurface> {
    let mut rows = Vec::new();
    for (fn_path, count) in fanout {
        let fn_phases = phases.get(fn_path).cloned().unwrap_or_default();
        if *count > SPLIT_FANOUT_THRESHOLD && fn_phases.len() >= 2 {
            rows.push(SplitSurface {
                fn_path: fn_path.clone(),
                fan_out: *count,
                rank: *count * fn_phases.len(),
                recommended_split_boundaries: fn_phases.iter().map(|p| format!("phase::{p}")).collect(),
                phases: fn_phases,
            });
        }
    }
    rows.sort_by(|a, b| b.rank.cmp(&a.rank).then_with(|| a.fn_path.cmp(&b.fn_path)));
    rows
}

fn merge_surface(similarities: &[(String, String, f64)]) -> Vec<MergeSurface> {
    similarities
        .iter()
        .map(|(left, right, score)| MergeSurface {
            module: module_name(left),
            members: vec![left.clone(), right.clone()],
            similarity: *score,
            recommended_canonical_name: short_name(left).to_string(),
        })
        .collect()
}

fn canonicalize_surface(
    similarities: &[(String, String, f64)],
    providers: &BTreeMap<String, Vec<String>>,
) -> Vec<CanonicalizeSurface> {
    similarities
        .iter()
        .filter_map(|(left, right, score)| {
            let left_providers = providers.get(left).cloned().unwrap_or_default();
            let right_providers = providers.get(right).cloned().unwrap_or_default();
            if left_providers == right_providers {
                return None;
            }
            let mut provider_map = BTreeMap::new();
            provider_map.insert(left.clone(), left_providers);
            provider_map.insert(right.clone(), right_providers);
            Some(CanonicalizeSurface {
                pair: vec![left.clone(), right.clone()],
                similarity: *score,
                providers: provider_map,
                recommended_trait_boundary: trait_boundary(left, right),
            })
        })
        .collect()
}

fn ordered_pair(left: &str, right: &str) -> (String, String) {
    if left <= right { (left.to_string(), right.to_string()) } else { (right.to_string(), left.to_string()) }
}

fn short_name(path: &str) -> &str {
    path.rsplit("::").next().unwrap_or(path)
}

fn module_name(path: &str) -> String {
    path.rsplit_once("::").map(|(module, _)| module.to_string()).unwrap_or_else(|| path.to_string())
}

fn trait_boundary(left: &str, right: &str) -> String {
    let left_name = short_name(left);
    let right_name = short_name(right);
    if left_name == right_name {
        format!("trait::{left_name}ProviderBoundary")
    } else {
        "trait::ProviderCanonicalBoundary".to_string()
    }
}

fn stable_id(payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    hex::encode(hasher.finalize())[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{CrateGraph, GraphEdge, GraphMeta, GraphNode, SourceSpan};
    use std::collections::BTreeMap;

    fn fixture_graph() -> CrateGraph {
        let mut nodes = BTreeMap::new();
        nodes.insert(
            "m::large".to_string(),
            GraphNode {
                def_id: "0:1".to_string(),
                path: "m::large".to_string(),
                kind: "fn".to_string(),
                def: Some(SourceSpan { file: "src/lib.rs".to_string(), line: 1, col: 1, lo: 10, hi: 20 }),
                source_text: None,
                sig: None,
                fields: vec![],
            },
        );
        let mut edges = vec![
            GraphEdge { relation: "phase".into(), from: "m::large".into(), to: "phase::parse".into(), span: None },
            GraphEdge { relation: "phase".into(), from: "m::large".into(), to: "phase::transform".into(), span: None },
            GraphEdge { relation: "similar".into(), from: "m::a".into(), to: "similar::0.90::m::b".into(), span: None },
            GraphEdge { relation: "provider".into(), from: "m::a".into(), to: "provider::openai".into(), span: None },
            GraphEdge { relation: "provider".into(), from: "m::b".into(), to: "provider::ollama".into(), span: None },
        ];
        for i in 0..31 {
            edges.push(GraphEdge { relation: "call".into(), from: "m::large".into(), to: format!("m::callee{i}"), span: None });
        }
        CrateGraph {
            meta: GraphMeta { schema_version: 16, crate_name: "fixture".into(), ..Default::default() },
            nodes,
            edges,
            intents: BTreeMap::new(),
        }
    }

    #[test]
    fn plans_all_three_auto_refactor_ops() {
        let plan = plan(&fixture_graph());
        assert_eq!(plan.graph_schema_version, 16);
        assert!(plan.operations.iter().any(|op| matches!(op, AutoRefactorOp::SplitFn { .. })));
        assert!(plan.operations.iter().any(|op| matches!(op, AutoRefactorOp::MergeFns { .. })));
        assert!(plan.operations.iter().any(|op| matches!(op, AutoRefactorOp::ExtractTrait { .. })));
    }
}
