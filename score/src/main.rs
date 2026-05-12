//! Structural code-quality scorer for any Rust repo instrumented with canon-rustc-v3.
//!
//! Reads every graph.json under --artifact-root, skips graphs whose
//! schema_version does not match SCHEMA_VERSION, then emits a markdown
//! score report covering six axes derivable from call-graph topology and
//! the canon-rustc-v3 intent/relation model.
//!
//! Axes
//! ----
//! Architecture   — coupling density + abstraction ratio (trait+impl nodes)
//! Structure      — uniformity of call in-degree distribution (1 − gini)
//! Simplicity     — absence of fan-out bloat and duplicate-code pressure
//! Maintainability— phase decomposition coverage minus duplication pressure
//! Determinism    — intent/relation consistency: pure fns must have no risk edges
//! Coherency      — intent classification completeness + entropy of intent classes

use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// Must match the schema_version emitted by canon-rustc-v3.
const SCHEMA_VERSION: u32 = 16;

// ── graph types (subset of canon-rustc-v3 schema) ────────────────────────────

#[derive(Deserialize)]
struct CrateGraph {
    meta: GraphMeta,
    #[serde(default)]
    nodes: BTreeMap<String, GraphNode>,
    #[serde(default)]
    edges: Vec<GraphEdge>,
    #[serde(default)]
    intents: BTreeMap<String, String>,
}

#[derive(Deserialize, Default)]
struct GraphMeta {
    #[serde(default)]
    crate_name: String,
    #[serde(default)]
    schema_version: u32,
    #[serde(default)]
    node_count: usize,
    #[serde(default)]
    edge_count: usize,
}

#[derive(Deserialize)]
struct GraphNode {
    #[serde(default)]
    kind: String,
}

#[derive(Deserialize)]
struct GraphEdge {
    relation: String,
    from: String,
    to: String,
}

// ── per-crate stats ───────────────────────────────────────────────────────────

struct CrateStats {
    crate_name: String,
    node_count: usize,
    edge_count: usize,
    fn_count: usize,
    trait_impl_count: usize,
    call_fanout_total: u64,
    call_in_degrees: Vec<u32>,
    similar_count: usize,
    phase_fn_count: usize,
    intents: BTreeMap<String, String>,
    pure_with_risk: usize, // fns classified pure but with risk edges (mut/io/unsafe/panic/alloc)
}

fn collect_stats(graph: &CrateGraph) -> CrateStats {
    let mut fn_count = 0usize;
    let mut trait_impl_count = 0usize;
    let mut structural_fns: BTreeSet<&str> = BTreeSet::new();
    for (name, node) in &graph.nodes {
        match node.kind.as_str() {
            "fn" => {
                if !is_synthetic_derive_fn(name) {
                    fn_count += 1;
                    structural_fns.insert(name.as_str());
                }
            }
            "trait" | "impl" => trait_impl_count += 1,
            _ => {}
        }
    }

    let mut call_out: BTreeMap<&str, u32> = BTreeMap::new();
    let mut call_in: BTreeMap<&str, u32> = BTreeMap::new();
    let mut similar_count = 0usize;
    let mut phase_fns: BTreeSet<&str> = BTreeSet::new();
    let mut risk_fns: BTreeSet<&str> = BTreeSet::new();

    for edge in &graph.edges {
        match edge.relation.as_str() {
            "call" if structural_fns.contains(edge.from.as_str()) => {
                *call_out.entry(edge.from.as_str()).or_insert(0) += 1;
                if structural_fns.contains(edge.to.as_str()) {
                    *call_in.entry(edge.to.as_str()).or_insert(0) += 1;
                }
            }
            "similar" => similar_count += 1,
            "phase" => {
                if structural_fns.contains(edge.from.as_str()) {
                    phase_fns.insert(edge.from.as_str());
                }
            }
            "mut" | "io" | "unsafe" | "panic" | "alloc" => {
                if structural_fns.contains(edge.from.as_str()) {
                    risk_fns.insert(edge.from.as_str());
                }
            }
            _ => {}
        }
    }

    let pure_with_risk = graph
        .intents
        .iter()
        .filter(|(path, intent)| {
            structural_fns.contains(path.as_str())
                && *intent == "pure"
                && risk_fns.contains(path.as_str())
        })
        .count();

    let call_fanout_total: u64 = call_out.values().map(|&v| v as u64).sum();
    let mut call_in_degrees: Vec<u32> = call_in.values().copied().collect();
    // pad with zero-in-degree fns so the gini reflects the full fn population
    call_in_degrees
        .extend(std::iter::repeat(0u32).take(fn_count.saturating_sub(call_in_degrees.len())));

    CrateStats {
        crate_name: graph.meta.crate_name.clone(),
        node_count: graph.meta.node_count,
        edge_count: graph.meta.edge_count,
        fn_count,
        trait_impl_count,
        call_fanout_total,
        call_in_degrees,
        similar_count,
        phase_fn_count: phase_fns.len(),
        intents: graph.intents.clone(),
        pure_with_risk,
    }
}

fn is_synthetic_derive_fn(name: &str) -> bool {
    if !name.starts_with('<') {
        return false;
    }

    let Some((_, method)) = name.rsplit_once("::") else {
        return false;
    };

    let derives_trait = [
        " as std::clone::Clone>",
        " as std::cmp::Eq>",
        " as std::cmp::PartialEq>",
        " as std::default::Default>",
        " as std::fmt::Debug>",
        " as std::hash::Hash>",
    ]
    .iter()
    .any(|needle| name.contains(needle));

    derives_trait
        && matches!(
            method,
            "clone" | "assert_fields_are_eq" | "eq" | "default" | "fmt" | "hash"
        )
}

// ── scoring ───────────────────────────────────────────────────────────────────

fn gini(values: &[u32]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let n = values.len() as f64;
    let mut sorted: Vec<f64> = values.iter().map(|&v| v as f64).collect();
    sorted.sort_by(f64::total_cmp);
    let sum: f64 = sorted.iter().sum();
    if sum == 0.0 {
        return 0.0;
    }
    let weighted: f64 = sorted
        .iter()
        .enumerate()
        .map(|(i, &v)| (i as f64 + 1.0) * v)
        .sum();
    (2.0 * weighted) / (n * sum) - (n + 1.0) / n
}

fn score_architecture(s: &CrateStats) -> f64 {
    // Very small crates (<20 nodes) have artificially high edge/node ratios from
    // external-function references in edges; return neutral score rather than penalise.
    if s.node_count < 20 {
        return 5.0;
    }
    let coupling = s.edge_count as f64 / s.node_count as f64;
    // ramp up to ideal coupling=7, then Gaussian falloff for over-coupling
    let c = (coupling / 7.0).min(1.0) * (-(coupling - 7.0).max(0.0).powi(2) / 50.0).exp();
    let abstraction = (s.trait_impl_count as f64 / s.node_count as f64 / 0.3).min(1.0);
    10.0 * (0.55 * c + 0.45 * abstraction)
}

fn score_structure(s: &CrateStats) -> f64 {
    if s.fn_count == 0 {
        return 10.0;
    }

    let observed_in_degree = s
        .call_in_degrees
        .iter()
        .filter(|&&degree| degree != 0)
        .count();
    let coverage = observed_in_degree as f64 / s.fn_count as f64;
    let inequality = gini(&s.call_in_degrees);

    // Sparse Rust call graphs often contain many leaf functions that are valid
    // public/API endpoints, trait shims, or generated impl methods. Penalizing
    // every zero-in-degree leaf as structural disorder makes the score volatile
    // for modular crates and overstates centralization. Retain the in-degree
    // Gini signal, but blend it with direct call-in coverage so the structure
    // axis rewards both balanced call distribution and reachable decomposition.
    let balanced_distribution = 1.0 - inequality;
    let reachable_decomposition = coverage.sqrt();
    10.0 * (0.45 * balanced_distribution + 0.55 * reachable_decomposition)
}

fn score_simplicity(s: &CrateStats) -> f64 {
    if s.fn_count == 0 {
        return 10.0;
    }
    let mean_fanout = s.call_fanout_total as f64 / s.fn_count as f64;
    // exponential penalty that starts at fanout=5
    let fanout_score = (-(mean_fanout - 5.0).max(0.0) / 10.0).exp();
    // each similar edge pair is one duplicate signal; cap at 50% penalty
    let dup_penalty = (s.similar_count as f64 / s.fn_count as f64).min(0.5);
    10.0 * fanout_score * (1.0 - dup_penalty)
}

fn score_maintainability(s: &CrateStats) -> f64 {
    if s.fn_count == 0 {
        return 10.0;
    }
    // phase-decomposed fns are already split along semantic boundaries (good)
    // clamp to 1.0: phase edges can reference external fns not in node map
    let phase_coverage = (s.phase_fn_count as f64 / s.fn_count as f64).min(1.0);
    // similar edges signal latent merge pressure (bad)
    let dup_pressure = (s.similar_count as f64 / s.fn_count as f64).min(1.0);
    (10.0 * (0.5 + 0.5 * phase_coverage) * (1.0 - dup_pressure * 0.4)).max(0.0)
}

fn score_determinism(s: &CrateStats) -> f64 {
    if s.fn_count == 0 {
        return 10.0;
    }
    let coverage = (s.intents.len() as f64 / s.fn_count as f64).min(1.0);
    let violation_rate = (s.pure_with_risk as f64 / s.fn_count as f64).min(1.0);
    10.0 * coverage * (1.0 - violation_rate * 2.0).max(0.0)
}

fn score_coherency(s: &CrateStats) -> f64 {
    if s.fn_count == 0 {
        return 10.0;
    }
    let coverage = (s.intents.len() as f64 / s.fn_count as f64).min(1.0);
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for intent in s.intents.values() {
        *counts.entry(intent.as_str()).or_insert(0) += 1;
    }
    let total = s.intents.len() as f64;
    let entropy: f64 = counts
        .values()
        .map(|&c| {
            let p = c as f64 / total;
            -p * p.ln()
        })
        .sum();
    // 7 canon intent classes: pure/io/mutation/orchestration/validation/unsafe/boundary
    let max_entropy = 7.0_f64.ln();
    let normalized_entropy = (entropy / max_entropy).min(1.0);
    10.0 * (0.7 * coverage + 0.3 * normalized_entropy)
}

fn per_crate_scores(s: &CrateStats) -> [f64; 6] {
    [
        score_architecture(s).clamp(0.0, 10.0),
        score_structure(s).clamp(0.0, 10.0),
        score_simplicity(s).clamp(0.0, 10.0),
        score_maintainability(s).clamp(0.0, 10.0),
        score_determinism(s).clamp(0.0, 10.0),
        score_coherency(s).clamp(0.0, 10.0),
    ]
}

fn geometric_mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let log_sum: f64 = values
        .iter()
        .map(|&v| if v > 0.0 { v.ln() } else { f64::NEG_INFINITY })
        .sum();
    if log_sum.is_infinite() {
        return 0.0;
    }
    (log_sum / values.len() as f64).exp()
}

// ── report ────────────────────────────────────────────────────────────────────

const AXES: [&str; 6] = [
    "Architecture",
    "Structure",
    "Simplicity",
    "Maintainability",
    "Determinism",
    "Coherency",
];

const AXIS_DEFS: [(&str, &str, &str); 6] = [
    (
        "Architecture",
        "edges/nodes coupling (peak 7) + trait+impl/total abstraction ratio",
        "coupling density, abstraction ratio",
    ),
    (
        "Structure",
        "blend of call-in coverage and 1 − call in-degree gini",
        "call-in coverage, call in-degree gini",
    ),
    (
        "Simplicity",
        "exponential fanout penalty (>5) × (1 − duplicate-pair ratio)",
        "mean call fanout, similar-edge ratio",
    ),
    (
        "Maintainability",
        "phase-decomposition coverage × (1 − duplication pressure)",
        "phase edge coverage, similar ratio",
    ),
    (
        "Determinism",
        "intent coverage × (1 − 2×violation rate for pure+risk conflicts)",
        "pure fns with risk edges",
    ),
    (
        "Coherency",
        "0.7×coverage + 0.3×normalized Shannon entropy of intent classes",
        "intent coverage, intent entropy",
    ),
];

/// Write a column-aligned markdown table.
///
/// `right[i] = true` right-aligns column i (uses `---:` separator and `>`
/// padding). All other columns are left-aligned. Column widths are the max
/// of the header length, every data cell length, and 4 (ensures ≥3 dashes in
/// the separator regardless of alignment marker).
fn write_md_table(
    buf: &mut Vec<u8>,
    headers: &[&str],
    right: &[bool],
    rows: &[Vec<String>],
) -> std::io::Result<()> {
    let ncols = headers.len();
    let is_right = |i: usize| right.get(i).copied().unwrap_or(false);

    // minimum 4 so separator always has ≥3 dashes after the alignment colon
    let mut w: Vec<usize> = headers.iter().map(|h| h.len().max(4)).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate().take(ncols) {
            w[i] = w[i].max(cell.len());
        }
    }

    // header (always left-aligned text)
    write!(buf, "|")?;
    for (i, h) in headers.iter().enumerate() {
        write!(buf, " {:<width$} |", h, width = w[i])?;
    }
    writeln!(buf)?;

    // separator  — `:---` left  /  `---:` right  (each exactly w[i] chars)
    write!(buf, "|")?;
    for i in 0..ncols {
        let dashes = w[i] - 1;
        if is_right(i) {
            write!(buf, " {}{} |", "-".repeat(dashes), ":")?;
        } else {
            write!(buf, " {}{} |", ":", "-".repeat(dashes))?;
        }
    }
    writeln!(buf)?;

    // data rows
    for row in rows {
        write!(buf, "|")?;
        for i in 0..ncols {
            let cell = row.get(i).map(String::as_str).unwrap_or("");
            if is_right(i) {
                write!(buf, " {:>width$} |", cell, width = w[i])?;
            } else {
                write!(buf, " {:<width$} |", cell, width = w[i])?;
            }
        }
        writeln!(buf)?;
    }
    Ok(())
}

fn write_report(
    path: &Path,
    stats: &[CrateStats],
    agg: &[f64; 6],
    g: f64,
    date: &str,
) -> Result<()> {
    let mut buf: Vec<u8> = Vec::new();

    writeln!(buf, "# Code Quality Score Report")?;
    writeln!(buf)?;
    writeln!(
        buf,
        "Generated: {date}  |  Schema version: {SCHEMA_VERSION}  |  Crates: {}",
        stats.len()
    )?;
    writeln!(buf)?;
    writeln!(buf, "## Aggregate Scores")?;
    writeln!(buf)?;
    writeln!(buf, "```text")?;
    for (i, &name) in AXES.iter().enumerate() {
        let score = agg[i];
        writeln!(buf, "{name:<20} = {score:.1}")?;
    }
    writeln!(buf)?;
    let g_str = format!("{g:.2}");
    writeln!(buf, "G (geometric mean)   = {g_str} / 10")?;
    writeln!(buf, "```")?;
    writeln!(buf)?;

    // per-crate table: crate name left, all numbers right
    writeln!(buf, "## Per-Crate Breakdown")?;
    writeln!(buf)?;
    let pc_headers = &[
        "Crate", "Nodes", "Edges", "Fns", "Arch", "Struct", "Simple", "Maint", "Determ", "Coher",
    ];
    let pc_right = &[false, true, true, true, true, true, true, true, true, true];
    let pc_rows: Vec<Vec<String>> = stats
        .iter()
        .map(|s| {
            let sc = per_crate_scores(s);
            vec![
                s.crate_name.clone(),
                s.node_count.to_string(),
                s.edge_count.to_string(),
                s.fn_count.to_string(),
                format!("{:.1}", sc[0]),
                format!("{:.1}", sc[1]),
                format!("{:.1}", sc[2]),
                format!("{:.1}", sc[3]),
                format!("{:.1}", sc[4]),
                format!("{:.1}", sc[5]),
            ]
        })
        .collect();
    write_md_table(&mut buf, pc_headers, pc_right, &pc_rows)?;
    writeln!(buf)?;

    // axis definitions table: all left-aligned
    writeln!(buf, "## Axis Definitions")?;
    writeln!(buf)?;
    let ax_headers = &["Axis", "Formula", "Graph signal"];
    let ax_right = &[false, false, false];
    let ax_rows: Vec<Vec<String>> = AXIS_DEFS
        .iter()
        .map(|(name, formula, signal)| {
            vec![name.to_string(), formula.to_string(), signal.to_string()]
        })
        .collect();
    write_md_table(&mut buf, ax_headers, ax_right, &ax_rows)?;
    writeln!(buf)?;

    writeln!(
        buf,
        "*Scores are structural proxies from graph.json topology and intent classification. \
They do not capture test coverage, runtime correctness, or domain semantics.*"
    )?;

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
    }
    fs::write(path, &buf).with_context(|| format!("cannot write {}", path.display()))
}

// ── graph discovery ───────────────────────────────────────────────────────────

fn find_graphs(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    collect_graphs(root, &mut out);
    out.sort();
    out
}

fn collect_graphs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_graphs(&path, out);
        } else if path.file_name().and_then(|n| n.to_str()) == Some("graph.json") {
            out.push(path);
        }
    }
}

// ── CLI ───────────────────────────────────────────────────────────────────────

struct Args {
    artifact_root: PathBuf,
    report: PathBuf,
    date: String,
}

fn parse_args() -> Result<Args> {
    let mut artifact_root = PathBuf::from("state/rustc");
    let mut report = PathBuf::from("SCORE_REPORT.md");
    let mut date = String::from("unknown");
    let mut iter = std::env::args().skip(1);
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--artifact-root" => {
                artifact_root =
                    PathBuf::from(iter.next().context("--artifact-root requires a value")?);
            }
            "--report" => {
                report = PathBuf::from(iter.next().context("--report requires a value")?);
            }
            "--date" => {
                date = iter.next().context("--date requires a value")?;
            }
            other => anyhow::bail!("unknown flag: {other}"),
        }
    }
    Ok(Args {
        artifact_root,
        report,
        date,
    })
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let args = parse_args()?;
    let graph_paths = find_graphs(&args.artifact_root);

    let mut stats: Vec<CrateStats> = Vec::new();
    let mut skipped = 0usize;

    for path in &graph_paths {
        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("skip {}: {e}", path.display());
                skipped += 1;
                continue;
            }
        };
        let graph: CrateGraph = match serde_json::from_slice(&bytes) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("skip {}: {e}", path.display());
                skipped += 1;
                continue;
            }
        };
        if graph.meta.schema_version != SCHEMA_VERSION {
            eprintln!(
                "skip {}: schema_version {} (want {SCHEMA_VERSION})",
                path.display(),
                graph.meta.schema_version,
            );
            skipped += 1;
            continue;
        }
        stats.push(collect_stats(&graph));
    }

    anyhow::ensure!(
        !stats.is_empty(),
        "no compatible graphs under {} ({skipped} skipped)",
        args.artifact_root.display(),
    );

    // fn-count-weighted mean across crates
    let total_fn: usize = stats.iter().map(|s| s.fn_count).sum();
    let weight = |s: &CrateStats| -> f64 {
        if total_fn == 0 {
            1.0 / stats.len() as f64
        } else {
            s.fn_count as f64 / total_fn as f64
        }
    };

    let mut agg = [0.0f64; 6];
    for s in &stats {
        let sc = per_crate_scores(s);
        let w = weight(s);
        for (i, &v) in sc.iter().enumerate() {
            agg[i] += w * v;
        }
    }
    let g = geometric_mean(&agg);

    write_report(&args.report, &stats, &agg, g, &args.date)?;

    println!(
        "score: G = {g:.2} / 10  ({} crates, {skipped} skipped)",
        stats.len()
    );
    for (i, &name) in AXES.iter().enumerate() {
        let score = agg[i];
        println!("  {name}: {score:.1}");
    }
    Ok(())
}
