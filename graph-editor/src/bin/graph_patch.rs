//! graph_patch — CLI for applying typed mutation ops to a graph.json.
//!
//! Usage
//! -----
//!   graph_patch --graph <graph.json> --ops <ops.json> \
//!               [--source-root <dir>] [--out <patch.diff>] [--receipt <receipt.json>]
//!
//! Reads `ops.json` (a JSON array of GraphMutationOp), validates every op
//! against the current graph (stale-op guard on lo/hi), then:
//!   • writes a unified diff of source changes to --out (or stdout)
//!   • writes a JSON GraphMutationReceipt to --receipt (or stderr)
//!
//! After applying the patch with `patch(1)` or `apply_patch` and re-running
//! `cargo check` with the canon-rustc-v3 wrapper, pass the new graph.json
//! to `--new-graph` to get a verified Pass/Partial/Fail receipt.

use anyhow::{Context, Result};
use graph_editor::{
    graph::parse_graph,
    ops::OpsFile,
    patch,
    receipt::{AttrAdd, GraphMutationReceipt},
};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

fn main() -> Result<()> {
    let args = parse_args()?;

    let graph_bytes =
        fs::read(&args.graph).with_context(|| format!("cannot read {}", args.graph.display()))?;
    let graph = parse_graph(&graph_bytes)?;

    let ops_bytes =
        fs::read(&args.ops).with_context(|| format!("cannot read {}", args.ops.display()))?;
    let ops_file = OpsFile::from_json(&ops_bytes)?;

    let source_root = args.source_root.as_deref().unwrap_or(Path::new("."));
    let patch_set = patch::generate(&graph, &ops_file.0, source_root)?;

    if !patch_set.stale_ops.is_empty() {
        eprintln!("STALE OPS ({} rejected):", patch_set.stale_ops.len());
        for s in &patch_set.stale_ops {
            eprintln!("  {s}");
        }
    }

    // Write the unified diff.
    match &args.out {
        Some(path) => fs::write(path, patch_set.source_patch.as_bytes())
            .with_context(|| format!("cannot write diff to {}", path.display()))?,
        None => io::stdout()
            .write_all(patch_set.source_patch.as_bytes())
            .context("cannot write diff to stdout")?,
    }

    // Collect what was actually patched for the receipt (pre-verification).
    let nodes_removed: Vec<String> = ops_file
        .0
        .iter()
        .filter_map(|op| {
            if let graph_editor::ops::GraphMutationOp::RemoveNode { path, expected_lo, expected_hi } = op {
                let guard_ok = graph
                    .nodes
                    .get(path.as_str())
                    .and_then(|n| n.def.as_ref())
                    .map(|s| s.lo == *expected_lo && s.hi == *expected_hi)
                    .unwrap_or(false);
                if guard_ok { Some(path.clone()) } else { None }
            } else {
                None
            }
        })
        .collect();

    let attrs_added: Vec<AttrAdd> = ops_file
        .0
        .iter()
        .filter_map(|op| {
            if let graph_editor::ops::GraphMutationOp::AddAttribute { path, attr, expected_lo } = op {
                let guard_ok = graph
                    .nodes
                    .get(path.as_str())
                    .and_then(|n| n.def.as_ref())
                    .map(|s| s.lo == *expected_lo)
                    .unwrap_or(false);
                if guard_ok { Some(AttrAdd { path: path.clone(), attr: attr.clone() }) } else { None }
            } else {
                None
            }
        })
        .collect();

    // Optional: load a new graph for verified receipt.
    let new_graph = args
        .new_graph
        .as_ref()
        .map(|p| {
            fs::read(p)
                .with_context(|| format!("cannot read new graph {}", p.display()))
                .and_then(|b| parse_graph(&b))
        })
        .transpose()?;

    let receipt = GraphMutationReceipt::build(
        &ops_bytes,
        &graph,
        &patch_set,
        new_graph.as_ref(),
        nodes_removed,
        attrs_added,
    );

    let receipt_json = serde_json::to_string_pretty(&receipt).context("cannot serialize receipt")?;
    match &args.receipt {
        Some(path) => fs::write(path, receipt_json.as_bytes())
            .with_context(|| format!("cannot write receipt to {}", path.display()))?,
        None => eprintln!("{receipt_json}"),
    }

    if receipt.verdict == graph_editor::receipt::Verdict::Fail {
        std::process::exit(1);
    }

    Ok(())
}

// ─── arg parsing ─────────────────────────────────────────────────────────────

struct Args {
    graph: PathBuf,
    ops: PathBuf,
    source_root: Option<PathBuf>,
    out: Option<PathBuf>,
    receipt: Option<PathBuf>,
    new_graph: Option<PathBuf>,
}

fn parse_args() -> Result<Args> {
    let mut args = std::env::args().skip(1).peekable();
    let mut graph = None;
    let mut ops = None;
    let mut source_root = None;
    let mut out = None;
    let mut receipt = None;
    let mut new_graph = None;

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--graph"       => graph       = Some(PathBuf::from(next_val(&mut args, "--graph")?)),
            "--ops"         => ops         = Some(PathBuf::from(next_val(&mut args, "--ops")?)),
            "--source-root" => source_root = Some(PathBuf::from(next_val(&mut args, "--source-root")?)),
            "--out"         => out         = Some(PathBuf::from(next_val(&mut args, "--out")?)),
            "--receipt"     => receipt     = Some(PathBuf::from(next_val(&mut args, "--receipt")?)),
            "--new-graph"   => new_graph   = Some(PathBuf::from(next_val(&mut args, "--new-graph")?)),
            other => anyhow::bail!("unknown flag: {other}"),
        }
    }

    Ok(Args {
        graph:       graph.context("--graph is required")?,
        ops:         ops.context("--ops is required")?,
        source_root,
        out,
        receipt,
        new_graph,
    })
}

fn next_val(
    args: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String> {
    args.next().with_context(|| format!("{flag} requires a value"))
}
