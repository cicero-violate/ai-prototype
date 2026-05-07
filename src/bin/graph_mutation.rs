#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::Path;

fn usage() -> &'static str {
    "usage:\n  graph_mutation verify-ops <ops.ndjson>\n  graph_mutation verify-receipts <patch-receipts.ndjson> <mutation-receipts.ndjson>\n  graph_mutation generate-patch <graph-contract.ndjson> <source-root> <ops.ndjson> <patch.out> <patch-receipt.out>\n  graph_mutation verify-landing <old-graph-contract.ndjson> <new-graph-contract.ndjson> <ops.ndjson> <mutation-receipt.out>"
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        eprintln!("{}", usage());
        std::process::exit(2);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("help" | "--help" | "-h") if args.len() == 2 => {
            println!("{}", usage());
            Ok(())
        }
        Some("verify-ops") if args.len() == 3 => verify_ops(&args[2]),
        Some("verify-receipts") if args.len() == 4 => verify_receipts(&args[2], &args[3]),
        Some("generate-patch") if args.len() == 7 => {
            generate_patch(&args[2], &args[3], &args[4], &args[5], &args[6])
        }
        Some("verify-landing") if args.len() == 6 => {
            verify_landing(&args[2], &args[3], &args[4], &args[5])
        }
        _ => Err("invalid graph_mutation command".to_string()),
    }
}

fn verify_ops(path: &str) -> Result<(), String> {
    let input = read_to_string(path)?;
    let receipt = ai::verify_graph_mutation_ops_ndjson(&input);
    print!(
        "{}",
        ai::encode_graph_mutation_opset_receipt_ndjson(&receipt)
    );
    exit_for_verdict(receipt.verdict);
    Ok(())
}

fn verify_receipts(patch_path: &str, mutation_path: &str) -> Result<(), String> {
    let patch_input = read_to_string(patch_path)?;
    let mutation_input = read_to_string(mutation_path)?;
    let receipt = ai::verify_graph_receipt_ledgers_ndjson(&patch_input, &mutation_input);
    print!(
        "{}",
        ai::encode_graph_receipt_ledger_receipt_ndjson(&receipt)
    );
    exit_for_verdict(receipt.verdict);
    Ok(())
}

fn generate_patch(
    graph_path: &str,
    source_root: &str,
    ops_path: &str,
    patch_out: &str,
    receipt_out: &str,
) -> Result<(), String> {
    let graph_input = read_to_string(graph_path)?;
    let graph = ai::decode_graph_snapshot_contract_ndjson(&graph_input)
        .ok_or_else(|| format!("failed to decode graph contract: {graph_path}"))?;
    let ops_input = read_to_string(ops_path)?;
    let ops_receipt = ai::verify_graph_mutation_ops_ndjson(&ops_input);
    if ops_receipt.verdict == ai::GraphMutationVerdict::Fail {
        return Err("operation ledger failed verification".to_string());
    }
    let ops = ai::decode_graph_mutation_ops_ndjson(&ops_input)
        .ok_or_else(|| format!("failed to decode operation ledger: {ops_path}"))?;
    let sources = read_sources(source_root, &ops)?;
    let plan = ai::generate_graph_patch(&graph, &sources, &ops)
        .map_err(|err| format!("failed to generate graph patch: {err}"))?;
    fs::write(Path::new(patch_out), plan.diff)
        .map_err(|err| format!("failed to write {patch_out}: {err}"))?;
    fs::write(
        Path::new(receipt_out),
        ai::encode_graph_patch_receipt_ndjson(&plan.receipt),
    )
    .map_err(|err| format!("failed to write {receipt_out}: {err}"))?;
    Ok(())
}

fn verify_landing(
    old_graph_path: &str,
    new_graph_path: &str,
    ops_path: &str,
    receipt_out: &str,
) -> Result<(), String> {
    let old_graph_input = read_to_string(old_graph_path)?;
    let old_graph = ai::decode_graph_snapshot_contract_ndjson(&old_graph_input)
        .ok_or_else(|| format!("failed to decode old graph contract: {old_graph_path}"))?;
    let new_graph_input = read_to_string(new_graph_path)?;
    let new_graph = ai::decode_graph_snapshot_contract_ndjson(&new_graph_input)
        .ok_or_else(|| format!("failed to decode new graph contract: {new_graph_path}"))?;
    let ops_input = read_to_string(ops_path)?;
    let ops_receipt = ai::verify_graph_mutation_ops_ndjson(&ops_input);
    if ops_receipt.verdict == ai::GraphMutationVerdict::Fail {
        return Err("operation ledger failed verification".to_string());
    }
    let ops = ai::decode_graph_mutation_ops_ndjson(&ops_input)
        .ok_or_else(|| format!("failed to decode operation ledger: {ops_path}"))?;
    let receipt = ai::verify_graph_mutation_landing(&old_graph, &new_graph, &ops)
        .map_err(|err| format!("failed to verify graph mutation landing: {err}"))?;
    fs::write(
        Path::new(receipt_out),
        ai::encode_graph_mutation_receipt_ndjson(&receipt),
    )
    .map_err(|err| format!("failed to write {receipt_out}: {err}"))?;
    exit_for_verdict(receipt.verdict);
    Ok(())
}

fn read_sources(
    source_root: &str,
    ops: &[ai::GraphMutationOp],
) -> Result<Vec<ai::GraphSourceFile>, String> {
    let mut files = Vec::<String>::new();
    for op in ops {
        let file = op.file().to_string();
        if !files.contains(&file) {
            files.push(file);
        }
    }
    files.sort();

    let mut sources = Vec::new();
    for file in files {
        let path = Path::new(source_root).join(&file);
        let content = fs::read_to_string(&path)
            .map_err(|err| format!("failed to read source {}: {err}", path.display()))?;
        sources.push(ai::GraphSourceFile::new(file, content));
    }
    Ok(sources)
}

fn read_to_string(path: &str) -> Result<String, String> {
    fs::read_to_string(Path::new(path)).map_err(|err| format!("failed to read {path}: {err}"))
}

fn exit_for_verdict(verdict: ai::GraphMutationVerdict) {
    if verdict == ai::GraphMutationVerdict::Fail {
        std::process::exit(1);
    }
}
