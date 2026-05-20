//! MCP adapter for Datalog-backed static analysis over captured compiler fact graphs.
//!
//! Internally, every query loads the graph as EDB ground facts (node/edge predicates)
//! and evaluates named Datalog programs using bottom-up BFS fixpoint.
//!
//! Two query families:
//!
//!   analysis=<kind>   — whole-graph analyses: scc, layers, intents, projections, all
//!                       Use node/module/filter to narrow results.
//!
//!   analysis=<query>  — Datalog named queries (require node= or module=):
//!                       reachable, callers, callees, effects, cycle_members,
//!                       io_reachable, unsafe_reachable, module_fanout, effects_in

use serde_json::{json, Value};
use std::path::Path;
use validation::refactor::{
    datalog::{self, GraphFacts},
    semantic_graph::{
        derived_edges,
        projections::{self, ProjectionKind},
    },
    static_analysis::{layers, scc},
};

use crate::runtime::WorkspaceView;

pub const CANON_GRAPH_ANALYSIS_TOOL: &str = "canon_graph_analysis";

pub const PROJECTIONS: &[(&str, ProjectionKind)] = &[
    ("call_graph", ProjectionKind::Call),
    ("cfg", ProjectionKind::ControlFlow),
    ("module_graph", ProjectionKind::Module),
    ("dependency", ProjectionKind::Dependency),
    ("def_use", ProjectionKind::DefUse),
    ("type_graph", ProjectionKind::Type),
    ("ownership", ProjectionKind::Ownership),
    ("effect_graph", ProjectionKind::Effect),
];

struct Q<'a> {
    node: Option<&'a str>,
    module: Option<&'a str>,
    filter: Option<&'a str>,
    limit: usize,
}

impl<'a> Q<'a> {
    fn from_args(args: &'a Value) -> Self {
        let limit = args
            .get("limit")
            .and_then(Value::as_u64)
            .map(|n| (n as usize).min(500))
            .unwrap_or(100);
        Self {
            node: args
                .get("node")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty()),
            module: args
                .get("module")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty()),
            filter: args
                .get("filter")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty()),
            limit,
        }
    }

    fn matches_path(&self, path: &str) -> bool {
        if let Some(node) = self.node {
            return path == node;
        }
        if let Some(module) = self.module {
            return GraphFacts::matches_module(path, module);
        }
        true
    }

    fn matches_edge(&self, from: &str, to: &str) -> bool {
        if let Some(node) = self.node {
            return from == node || to == node;
        }
        if let Some(module) = self.module {
            return GraphFacts::matches_module(from, module)
                || GraphFacts::matches_module(to, module);
        }
        true
    }
}

pub fn run(args: &Value, workspace: &WorkspaceView) -> Value {
    let graph_rel = match args
        .get("graph")
        .or_else(|| args.get("graph_path"))
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
    {
        Some(p) => p,
        None => {
            return error(
                "canon_graph_analysis requires 'graph' (workspace-relative graph.json path)",
            )
        }
    };

    let graph_path = workspace.root.join(graph_rel);
    let graph = match validation::read_json(Path::new(&graph_path)) {
        Ok(g) => g,
        Err(e) => return error(format!("Failed to read graph at {graph_rel}: {e}")),
    };

    let analysis = match args.get("analysis").and_then(Value::as_str).unwrap_or("") {
        "" => {
            return error(
                "canon_graph_analysis requires 'analysis'. \
             Whole-graph: scc, layers, intents, derived_edges, call_graph, cfg, \
             module_graph, dependency, def_use, type_graph, ownership, effect_graph, all. \
             Datalog queries (require node= or module=): reachable, callers, callees, \
             effects, cycle_members, io_reachable, unsafe_reachable, module_fanout, effects_in.",
            )
        }
        kind => kind,
    };

    let q = Q::from_args(args);
    let crate_name = graph
        .pointer("/meta/crate_name")
        .and_then(Value::as_str)
        .unwrap_or("<unknown>")
        .to_string();

    let result = match analysis {
        // ── Datalog named queries ───────────────────────────────────────────
        "reachable" | "callers" | "callees" | "effects" | "cycle_members" | "io_reachable"
        | "unsafe_reachable" => {
            let node = match q.node {
                Some(n) => n,
                None => return error("This Datalog query requires node=<exact symbol path>"),
            };
            let facts = GraphFacts::from_graph(&graph);
            match analysis {
                "reachable" => datalog::reachable_from(&facts, node, q.limit),
                "callers" => datalog::callers_of(&facts, node),
                "callees" => datalog::callees_of(&facts, node),
                "effects" => datalog::effects_of(&facts, node),
                "cycle_members" => datalog::cycle_members(&facts, node, &graph),
                "io_reachable" => datalog::io_reachable_from(&facts, node, q.limit),
                "unsafe_reachable" => datalog::unsafe_reachable_from(&facts, node, q.limit),
                _ => unreachable!(),
            }
        }
        "module_fanout" => {
            let module = match q.module {
                Some(m) => m,
                None => return error("module_fanout requires module=<module prefix>"),
            };
            let facts = GraphFacts::from_graph(&graph);
            datalog::module_fanout(&facts, module)
        }
        "effects_in" => {
            let facts = GraphFacts::from_graph(&graph);
            datalog::effects_in(&facts, q.module, q.filter, q.limit)
        }

        // ── Whole-graph analyses ────────────────────────────────────────────
        "scc" => run_scc(&graph, &q),
        "layers" => run_layers(&graph, &q),
        "intents" => {
            let facts = GraphFacts::from_graph(&graph);
            datalog::intents_in(&facts, &graph, q.module, q.filter, q.limit)
        }
        "derived_edges" => run_derived_edges(&graph, &q),
        "all" => run_all(&graph, &q),
        kind => match projection_kind(kind) {
            Some(pk) => run_projection(&graph, pk, &q),
            None => {
                return error(format!(
                    "Unknown analysis '{kind}'. Whole-graph: scc, layers, intents, derived_edges, \
                 call_graph, cfg, module_graph, dependency, def_use, type_graph, ownership, \
                 effect_graph, all. Datalog: reachable, callers, callees, effects, cycle_members, \
                 io_reachable, unsafe_reachable, module_fanout, effects_in."
                ))
            }
        },
    };

    ok(json!({
        "crate": crate_name,
        "analysis": analysis,
        "query": { "node": args.get("node"), "module": args.get("module"), "filter": args.get("filter"), "limit": q.limit },
        "result": result,
    }))
}

// ── Whole-graph analyses (query-filtered) ─────────────────────────────────────

fn run_scc(graph: &Value, q: &Q) -> Value {
    let report = scc::compute_scc(graph);
    let all_cycles = match report.get("cycles").and_then(Value::as_array) {
        Some(c) => c.clone(),
        None => return report,
    };
    let count = report["scc_count"].as_u64().unwrap_or(0);
    let max = report["max_cycle_size"].as_u64().unwrap_or(0);

    if q.node.is_none() && q.module.is_none() {
        let preview: Vec<&Value> = all_cycles.iter().take(5).collect();
        return json!({
            "scc_count": count,
            "max_cycle_size": max,
            "note": "Use node=<path> or module=<prefix> to focus. Use analysis=cycle_members for full SCC query. Showing first 5.",
            "cycles_preview": preview,
        });
    }
    let filtered: Vec<&Value> = all_cycles
        .iter()
        .filter(|cycle| {
            cycle["members"]
                .as_array()
                .map(|members| {
                    members
                        .iter()
                        .filter_map(Value::as_str)
                        .any(|m| q.matches_path(m))
                })
                .unwrap_or(false)
        })
        .take(q.limit)
        .collect();
    json!({ "scc_count": count, "max_cycle_size": max, "matched_cycles": filtered.len(), "cycles": filtered })
}

fn run_layers(graph: &Value, q: &Q) -> Value {
    let report = layers::layer_state(graph);
    let all_violations = match report.get("violations").and_then(Value::as_array) {
        Some(v) => v.clone(),
        None => return report,
    };
    let count = report["violation_count"].as_u64().unwrap_or(0);

    if q.node.is_none() && q.module.is_none() {
        if count == 0 {
            return report;
        }
        let preview: Vec<&Value> = all_violations.iter().take(5).collect();
        return json!({
            "present": report["present"], "skipped": report["skipped"],
            "violation_count": count,
            "note": "Use node=<path> or module=<prefix> to filter. Showing first 5.",
            "violations_preview": preview,
        });
    }
    let filtered: Vec<&Value> = all_violations
        .iter()
        .filter(|v| {
            let from = v["from"].as_str().unwrap_or("");
            let to = v["to"].as_str().unwrap_or("");
            q.matches_edge(from, to)
        })
        .take(q.limit)
        .collect();
    json!({
        "present": report["present"], "skipped": report["skipped"],
        "violation_count": count, "matched": filtered.len(), "violations": filtered,
    })
}

fn run_derived_edges(graph: &Value, q: &Q) -> Value {
    let all_edges = derived_edges::derived_edges(graph);

    if q.node.is_none() && q.module.is_none() && q.filter.is_none() {
        let mut by_rel: serde_json::Map<String, Value> = serde_json::Map::new();
        for e in &all_edges {
            let rel = e["relation"].as_str().unwrap_or("?");
            let cnt = by_rel.entry(rel.to_string()).or_insert(json!(0));
            *cnt = json!(cnt.as_u64().unwrap_or(0) + 1);
        }
        return json!({
            "derived_count": all_edges.len(), "by_relation": by_rel,
            "note": "Use node=<path>, module=<prefix>, or filter=<relation> to get edges.",
        });
    }
    let rel_filter = q.filter;
    let filtered: Vec<&Value> = all_edges
        .iter()
        .filter(|e| {
            let from = e["from"].as_str().unwrap_or("");
            let to = e["to"].as_str().unwrap_or("");
            let rel = e["relation"].as_str().unwrap_or("");
            q.matches_edge(from, to) && rel_filter.map(|f| rel == f).unwrap_or(true)
        })
        .take(q.limit)
        .collect();
    let total = all_edges
        .iter()
        .filter(|e| {
            let from = e["from"].as_str().unwrap_or("");
            let to = e["to"].as_str().unwrap_or("");
            let rel = e["relation"].as_str().unwrap_or("");
            q.matches_edge(from, to) && rel_filter.map(|f| rel == f).unwrap_or(true)
        })
        .count();
    json!({ "derived_count": all_edges.len(), "matched": total, "showing": filtered.len(), "truncated": total > filtered.len(), "edges": filtered })
}

fn run_projection(graph: &Value, kind: ProjectionKind, q: &Q) -> Value {
    if kind == ProjectionKind::ControlFlow {
        return run_cfg(graph, q);
    }
    let projected = projections::project(graph, kind);
    let all_edges = match projected.get("edges").and_then(Value::as_array) {
        Some(e) => e.clone(),
        None => return projected,
    };
    if q.node.is_none() && q.module.is_none() && q.filter.is_none() {
        return json!({
            "kind": projected["kind"], "node_count": projected["node_count"], "edge_count": projected["edge_count"],
            "note": "Use node=<path>, module=<prefix>, or filter=<relation> to get edges. Full dump omitted.",
        });
    }
    let rel_filter = q.filter;
    let filtered: Vec<&Value> = all_edges
        .iter()
        .filter(|e| {
            let from = e["from"].as_str().unwrap_or("");
            let to = e["to"].as_str().unwrap_or("");
            let rel = e["relation"].as_str().unwrap_or("");
            q.matches_edge(from, to) && rel_filter.map(|f| rel == f).unwrap_or(true)
        })
        .take(q.limit)
        .collect();
    let total = all_edges
        .iter()
        .filter(|e| {
            let from = e["from"].as_str().unwrap_or("");
            let to = e["to"].as_str().unwrap_or("");
            let rel = e["relation"].as_str().unwrap_or("");
            q.matches_edge(from, to) && rel_filter.map(|f| rel == f).unwrap_or(true)
        })
        .count();
    let mut nodes: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for e in &filtered {
        if let Some(f) = e["from"].as_str() {
            nodes.insert(f);
        }
        if let Some(t) = e["to"].as_str() {
            nodes.insert(t);
        }
    }
    json!({
        "kind": projected["kind"], "total_edge_count": projected["edge_count"],
        "matched_edges": total, "showing": filtered.len(), "truncated": total > filtered.len(),
        "nodes": nodes.into_iter().collect::<Vec<_>>(), "edges": filtered,
    })
}

fn run_cfg(graph: &Value, q: &Q) -> Value {
    let projected = projections::project(graph, ProjectionKind::ControlFlow);
    let all_fns = match projected.get("functions").and_then(Value::as_array) {
        Some(f) => f.clone(),
        None => return projected,
    };
    if q.node.is_none() && q.module.is_none() {
        return json!({
            "kind": "CFG", "function_count": all_fns.len(),
            "note": "Use node=<path> or module=<prefix> to get CFG data.",
        });
    }
    let filtered: Vec<&Value> = all_fns
        .iter()
        .filter(|f| q.matches_path(f["function"].as_str().unwrap_or("")))
        .take(q.limit)
        .collect();
    json!({ "kind": "CFG", "total_functions": all_fns.len(), "matched": filtered.len(), "functions": filtered })
}

fn run_all(graph: &Value, q: &Q) -> Value {
    let facts = GraphFacts::from_graph(graph);
    let mut projections_out = serde_json::Map::new();
    for (name, kind) in PROJECTIONS {
        projections_out.insert((*name).to_string(), run_projection(graph, *kind, q));
    }
    json!({
        "scc":           run_scc(graph, q),
        "layers":        run_layers(graph, q),
        "intents":       datalog::intents_in(&facts, graph, q.module, q.filter, q.limit),
        "derived_edges": run_derived_edges(graph, q),
        "projections":   Value::Object(projections_out),
    })
}

fn projection_kind(name: &str) -> Option<ProjectionKind> {
    PROJECTIONS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, k)| *k)
}

fn ok(payload: Value) -> Value {
    let text = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string());
    json!({ "content": [{ "type": "text", "text": text }], "isError": false })
}

fn error(msg: impl Into<String>) -> Value {
    json!({ "content": [{ "type": "text", "text": format!("Error: {}", msg.into()) }], "isError": true })
}
