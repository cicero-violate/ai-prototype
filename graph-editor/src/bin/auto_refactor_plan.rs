//! auto_refactor_plan — emit deterministic SplitFn/MergeFns/ExtractTrait plans.

use anyhow::{Context, Result};
use graph_editor::{autorefactor, graph::parse_graph};
use std::{fs, io::{self, Write}, path::PathBuf};

fn main() -> Result<()> {
    let args = parse_args()?;
    if args.schema_version {
        println!("{}", graph_editor::graph::SCHEMA_VERSION);
        return Ok(());
    }
    let graph_bytes = fs::read(&args.graph).with_context(|| format!("cannot read {}", args.graph.display()))?;
    let graph = parse_graph(&graph_bytes)?;
    let plan = autorefactor::plan(&graph);
    let json = serde_json::to_string_pretty(&plan).context("cannot serialize auto-refactor plan")?;
    match args.out {
        Some(path) => fs::write(&path, json.as_bytes()).with_context(|| format!("cannot write {}", path.display()))?,
        None => io::stdout().write_all(json.as_bytes()).context("cannot write stdout")?,
    }
    Ok(())
}

struct Args { graph: PathBuf, out: Option<PathBuf>, schema_version: bool }

fn parse_args() -> Result<Args> {
    let mut args = std::env::args().skip(1);
    let mut graph = None;
    let mut out = None;
    let mut schema_version = false;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--graph" => graph = Some(PathBuf::from(args.next().context("--graph requires a value")?)),
            "--out" => out = Some(PathBuf::from(args.next().context("--out requires a value")?)),
            "--schema-version" => schema_version = true,
            other => anyhow::bail!("unknown flag: {other}"),
        }
    }
    if !schema_version {
        graph = Some(graph.context("--graph is required")?);
    }
    Ok(Args { graph: graph.unwrap_or_default(), out, schema_version })
}
