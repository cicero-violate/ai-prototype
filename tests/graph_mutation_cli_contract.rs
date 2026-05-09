use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn graph_mutation_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_graph_mutation"))
}

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "canon-graph-cli-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write(path: &Path, content: &str) {
    fs::write(path, content).expect("write test fixture");
}

fn copy_fixture_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("create fixture copy dir");
    for entry in fs::read_dir(src).expect("read fixture dir") {
        let entry = entry.expect("read fixture entry");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_fixture_tree(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).expect("copy fixture file");
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkflowManifest {
    files: Vec<String>,
    commands: Vec<Vec<String>>,
    integrity: Vec<WorkflowIntegrityRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkflowIntegrityRow {
    file: String,
    algorithm: String,
    hash: String,
}

fn parse_workflow_manifest(content: &str) -> WorkflowManifest {
    enum Section {
        None,
        Files,
        Commands,
        Integrity,
    }

    let mut section = Section::None;
    let mut files = Vec::new();
    let mut commands = Vec::new();
    let mut integrity = Vec::new();

    for raw_line in content.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        match trimmed {
            "files:" => {
                section = Section::Files;
                continue;
            }
            "commands:" => {
                section = Section::Commands;
                continue;
            }
            "integrity:" => {
                section = Section::Integrity;
                continue;
            }
            _ => {}
        }

        match section {
            Section::Files => {
                let Some(path) = raw_line.strip_prefix("  ") else {
                    panic!("manifest file rows must be indented: {raw_line}");
                };
                assert!(!path.is_empty(), "manifest file row must not be empty");
                files.push(path.to_owned());
            }
            Section::Commands => {
                let Some(command) = raw_line.strip_prefix("  graph_mutation ") else {
                    panic!("manifest command rows must use graph_mutation prefix: {raw_line}");
                };
                let args = command
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                assert!(!args.is_empty(), "manifest command row must not be empty");
                commands.push(args);
            }
            Section::Integrity => {
                let Some(row) = raw_line.strip_prefix("  ") else {
                    panic!("manifest integrity rows must be indented: {raw_line}");
                };
                let fields = row.split_whitespace().collect::<Vec<_>>();
                assert_eq!(
                    fields.len(),
                    3,
                    "integrity rows must be: <file> <algorithm> <hash>"
                );
                assert_eq!(
                    fields[1], "sha256",
                    "only sha256 fixture hashes are supported"
                );
                assert_eq!(fields[2].len(), 64, "sha256 hash must be hex encoded");
                assert!(
                    fields[2].chars().all(|c| c.is_ascii_hexdigit()),
                    "sha256 hash must be hex"
                );
                integrity.push(WorkflowIntegrityRow {
                    file: fields[0].to_owned(),
                    algorithm: fields[1].to_owned(),
                    hash: fields[2].to_ascii_lowercase(),
                });
            }
            Section::None => panic!("manifest row appears before section: {raw_line}"),
        }
    }

    WorkflowManifest {
        files,
        commands,
        integrity,
    }
}

fn sha256sum(path: &Path) -> String {
    let output = Command::new("sha256sum")
        .arg(path)
        .output()
        .unwrap_or_else(|err| panic!("run sha256sum for {}: {err}", path.display()));
    assert!(
        output.status.success(),
        "sha256sum failed for {}: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("sha256sum stdout utf8");
    stdout
        .split_whitespace()
        .next()
        .expect("sha256sum digest")
        .to_ascii_lowercase()
}

fn workflow_fixture_integrity_receipt(dir: &Path, integrity: &[WorkflowIntegrityRow]) -> String {
    let mut rows = integrity
        .iter()
        .map(|row| {
            format!(
                "{}|{}|{}",
                row.file,
                row.algorithm,
                sha256sum(&dir.join(&row.file))
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    rows.join("\n")
}

fn manifest_args_to_paths(dir: &Path, args: &[String]) -> Vec<String> {
    args.iter()
        .map(|arg| {
            if arg.ends_with(".ndjson")
                || arg.ends_with(".diff")
                || arg == "source"
                || arg == "patch.diff"
            {
                dir.join(arg).to_str().expect("manifest path").to_owned()
            } else {
                arg.clone()
            }
        })
        .collect()
}
fn run_str(args: &[&str]) -> Output {
    Command::new(graph_mutation_bin())
        .args(args)
        .output()
        .expect("run graph_mutation cli")
}

fn fixture_contracts(dir: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let old_graph = dir.join("old.ndjson");
    let new_graph = dir.join("new.ndjson");
    let ops = dir.join("ops.ndjson");
    let source_root = dir.join("source");
    fs::create_dir_all(source_root.join("src")).expect("create source root");

    write(
        &old_graph,
        "G|11|1\nN|crate::a|fn|0|src/lib.rs|1|1|0|10\nN|crate::b|fn|1|src/lib.rs|2|1|10|20\nE|call|crate::a|crate::b\nI|crate::a|pure\nI|crate::b|pure\n",
    );
    write(
        &new_graph,
        "G|11|2\nN|crate::a|fn|0|src/lib.rs|1|1|0|10\nI|crate::a|pure\n",
    );
    write(&source_root.join("src/lib.rs"), "fn a() {}\nfn b() {}\n");

    let op = ai::GraphMutationOp::RemoveNode {
        path: "crate::b".to_owned(),
        file: "src/lib.rs".to_owned(),
        lo: 10,
        hi: 20,
    };
    write(&ops, &ai::encode_graph_mutation_ops_ndjson(&[op]));
    (old_graph, new_graph, ops, source_root)
}

#[test]
fn graph_mutation_cli_verifies_ops_and_receipts() {
    let dir = temp_dir("verify");
    let (_old_graph, _new_graph, ops, _source_root) = fixture_contracts(&dir);
    let patch_receipts = dir.join("patch-receipts.ndjson");
    let mutation_receipts = dir.join("mutation-receipts.ndjson");
    write(&patch_receipts, "");
    write(&mutation_receipts, "");

    let ops_output = run_str(&["verify-ops", ops.to_str().expect("ops path")]);
    assert!(ops_output.status.success());
    let ops_receipt = String::from_utf8(ops_output.stdout).expect("utf8 stdout");
    assert!(ops_receipt.starts_with('['));
    assert!(ops_receipt.ends_with('\n'));

    let receipt_output = run_str(&[
        "verify-receipts",
        patch_receipts.to_str().expect("patch receipts path"),
        mutation_receipts.to_str().expect("mutation receipts path"),
    ]);
    assert!(receipt_output.status.success());
    let ledger_receipt = String::from_utf8(receipt_output.stdout).expect("utf8 stdout");
    assert!(ledger_receipt.starts_with('['));
    assert!(ledger_receipt.ends_with('\n'));
}

#[test]
fn graph_mutation_cli_generates_patch_and_landing_receipt() {
    let dir = temp_dir("roundtrip");
    let (old_graph, new_graph, ops, source_root) = fixture_contracts(&dir);
    let patch = dir.join("patch.diff");
    let patch_receipt = dir.join("patch-receipt.ndjson");
    let mutation_receipt = dir.join("mutation-receipt.ndjson");

    let patch_output = run_str(&[
        "generate-patch",
        old_graph.to_str().expect("old graph path"),
        source_root.to_str().expect("source root path"),
        ops.to_str().expect("ops path"),
        patch.to_str().expect("patch path"),
        patch_receipt.to_str().expect("patch receipt path"),
    ]);
    assert!(
        patch_output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&patch_output.stderr)
    );
    assert!(fs::read_to_string(&patch)
        .expect("read patch")
        .contains("--- a/src/lib.rs"));
    assert!(fs::read_to_string(&patch_receipt)
        .expect("read patch receipt")
        .starts_with('['));

    let landing_output = run_str(&[
        "verify-landing",
        old_graph.to_str().expect("old graph path"),
        new_graph.to_str().expect("new graph path"),
        ops.to_str().expect("ops path"),
        mutation_receipt.to_str().expect("mutation receipt path"),
    ]);
    assert!(
        landing_output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&landing_output.stderr)
    );
    let receipt = fs::read_to_string(&mutation_receipt).expect("read mutation receipt");
    assert!(receipt.starts_with('['));
    assert!(receipt.ends_with('\n'));
}

#[test]
fn graph_mutation_cli_roundtrips_generated_receipts_into_ledger_verifier() {
    let dir = temp_dir("ledger-roundtrip");
    let (old_graph, new_graph, ops, source_root) = fixture_contracts(&dir);
    let patch = dir.join("patch.diff");
    let patch_receipt = dir.join("patch-receipt.ndjson");
    let mutation_receipt = dir.join("mutation-receipt.ndjson");

    let patch_output = run_str(&[
        "generate-patch",
        old_graph.to_str().expect("old graph path"),
        source_root.to_str().expect("source root path"),
        ops.to_str().expect("ops path"),
        patch.to_str().expect("patch path"),
        patch_receipt.to_str().expect("patch receipt path"),
    ]);
    assert!(
        patch_output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&patch_output.stderr)
    );

    let landing_output = run_str(&[
        "verify-landing",
        old_graph.to_str().expect("old graph path"),
        new_graph.to_str().expect("new graph path"),
        ops.to_str().expect("ops path"),
        mutation_receipt.to_str().expect("mutation receipt path"),
    ]);
    assert!(
        landing_output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&landing_output.stderr)
    );

    let ledger_output = run_str(&[
        "verify-receipts",
        patch_receipt.to_str().expect("patch receipt path"),
        mutation_receipt.to_str().expect("mutation receipt path"),
    ]);
    assert!(
        ledger_output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&ledger_output.stderr)
    );
    let ledger_receipt = String::from_utf8(ledger_output.stdout).expect("utf8 stdout");
    assert!(ledger_receipt.starts_with('['));

    let fields = ledger_receipt
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .collect::<Vec<_>>();
    assert_eq!(fields.len(), 14);
    assert_eq!(fields[2], "1", "one generated patch receipt is counted");
    assert_eq!(fields[3], "1", "one generated landing receipt is counted");
    assert_eq!(fields[4], "1", "generated patch receipt passes");
    assert_eq!(fields[5], "1", "generated landing receipt passes");
    assert_eq!(fields[8], "0", "no generated receipt rows are invalid");
    assert_eq!(fields[12], "0", "aggregate ledger verdict passes");
}

#[test]
fn graph_mutation_cli_exposes_stable_usage_contract() {
    let output = run_str(&["help"]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    let fixture = include_str!("fixtures/graph_mutation_cli_usage.txt");
    assert_eq!(stdout, fixture);
}

#[test]
fn graph_mutation_cli_short_help_matches_usage_contract() {
    let long_help = run_str(&["--help"]);
    let short_help = run_str(&["-h"]);
    assert!(long_help.status.success());
    assert!(short_help.status.success());
    assert_eq!(long_help.stdout, short_help.stdout);
    assert!(String::from_utf8(long_help.stdout)
        .expect("utf8 stdout")
        .contains("generate-patch"));
}

#[test]
fn graph_mutation_cli_workflow_fixture_is_copyable_contract() {
    let fixture_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/graph_mutation_cli_workflow");
    let dir = temp_dir("workflow-fixture");
    copy_fixture_tree(&fixture_root, &dir);

    let old_graph = dir.join("old-graph.ndjson");
    let new_graph = dir.join("new-graph.ndjson");
    let ops = dir.join("ops.ndjson");
    let source_root = dir.join("source");
    let expected_patch = dir.join("expected-patch.diff");
    let patch = dir.join("patch.diff");
    let patch_receipt = dir.join("patch-receipt.ndjson");
    let mutation_receipt = dir.join("mutation-receipt.ndjson");

    let ops_output = run_str(&["verify-ops", ops.to_str().expect("ops path")]);
    assert!(
        ops_output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&ops_output.stderr)
    );

    let patch_output = run_str(&[
        "generate-patch",
        old_graph.to_str().expect("old graph path"),
        source_root.to_str().expect("source root path"),
        ops.to_str().expect("ops path"),
        patch.to_str().expect("patch path"),
        patch_receipt.to_str().expect("patch receipt path"),
    ]);
    assert!(
        patch_output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&patch_output.stderr)
    );
    assert_eq!(
        fs::read_to_string(&patch).expect("read generated patch"),
        fs::read_to_string(&expected_patch).expect("read expected patch")
    );

    let landing_output = run_str(&[
        "verify-landing",
        old_graph.to_str().expect("old graph path"),
        new_graph.to_str().expect("new graph path"),
        ops.to_str().expect("ops path"),
        mutation_receipt.to_str().expect("mutation receipt path"),
    ]);
    assert!(
        landing_output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&landing_output.stderr)
    );

    let ledger_output = run_str(&[
        "verify-receipts",
        patch_receipt.to_str().expect("patch receipt path"),
        mutation_receipt.to_str().expect("mutation receipt path"),
    ]);
    assert!(
        ledger_output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&ledger_output.stderr)
    );
    let ledger_receipt = String::from_utf8(ledger_output.stdout).expect("utf8 stdout");
    assert!(ledger_receipt.starts_with('['));
    assert!(ledger_receipt.ends_with('\n'));
}

#[test]
fn graph_mutation_cli_workflow_manifest_integrity_hashes_are_stable() {
    let fixture_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/graph_mutation_cli_workflow");
    let manifest = parse_workflow_manifest(
        &fs::read_to_string(fixture_root.join("MANIFEST.txt")).expect("read workflow manifest"),
    );

    assert_eq!(manifest.integrity.len(), manifest.files.len());
    let integrity_files = manifest
        .integrity
        .iter()
        .map(|row| row.file.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        integrity_files,
        manifest
            .files
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    );

    for row in &manifest.integrity {
        assert_eq!(
            sha256sum(&fixture_root.join(&row.file)),
            row.hash,
            "workflow fixture drifted: {}",
            row.file
        );
    }

    assert_eq!(
        workflow_fixture_integrity_receipt(&fixture_root, &manifest.integrity),
        "expected-patch.diff|sha256|f96d292a3e31b39b16354c6a0d3f42b25dc9ac6a40186a89c5c81fe7e7a703cc\nnew-graph.ndjson|sha256|dd16113ec6a5910977907345c2cea72567d692ea2a89e9e5e216e545b21b3468\nold-graph.ndjson|sha256|c005e0155ede42754f2b9c7e37eac2f1da1b268a0e5ce83d439eeb62e94654e1\nops.ndjson|sha256|550f07baac8b60bd66f63d89c3f602da77c58437bceef8a9ce0a9e9a61d2ccb1\nsource/src/lib.rs|sha256|f334ea00880bc9f3f958220d14eefc216769a299b1db33e7dd8f699feb933b2c"
    );
}

#[test]
fn graph_mutation_cli_workflow_manifest_integrity_detects_sample_drift() {
    let fixture_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/graph_mutation_cli_workflow");
    let dir = temp_dir("workflow-integrity-drift");
    copy_fixture_tree(&fixture_root, &dir);

    let manifest = parse_workflow_manifest(
        &fs::read_to_string(dir.join("MANIFEST.txt")).expect("read workflow manifest"),
    );
    let expected_receipt = workflow_fixture_integrity_receipt(&dir, &manifest.integrity);

    fs::write(dir.join("expected-patch.diff"), "drifted fixture patch\n")
        .expect("mutate copied workflow fixture");

    assert_ne!(
        workflow_fixture_integrity_receipt(&dir, &manifest.integrity),
        expected_receipt,
        "fixture integrity receipt must change when a copied sample file drifts"
    );

    let drifted = manifest
        .integrity
        .iter()
        .filter(|row| sha256sum(&dir.join(&row.file)) != row.hash)
        .map(|row| row.file.as_str())
        .collect::<Vec<_>>();
    assert_eq!(drifted, vec!["expected-patch.diff"]);
}

#[test]
fn graph_mutation_cli_workflow_manifest_is_executable_contract() {
    let fixture_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/graph_mutation_cli_workflow");
    let dir = temp_dir("workflow-manifest");
    copy_fixture_tree(&fixture_root, &dir);

    let manifest_path = dir.join("MANIFEST.txt");
    let manifest = parse_workflow_manifest(
        &fs::read_to_string(&manifest_path).expect("read workflow manifest"),
    );

    assert_eq!(
        manifest.files,
        vec![
            "ops.ndjson",
            "old-graph.ndjson",
            "new-graph.ndjson",
            "source/src/lib.rs",
            "expected-patch.diff",
        ]
    );
    assert_eq!(manifest.commands.len(), 4);

    for file in &manifest.files {
        assert!(
            dir.join(file).exists(),
            "manifest references missing workflow file: {file}"
        );
    }

    for row in &manifest.integrity {
        assert_eq!(
            sha256sum(&dir.join(&row.file)),
            row.hash,
            "manifest integrity hash mismatch before workflow execution: {}",
            row.file
        );
    }

    for command in &manifest.commands {
        let resolved = manifest_args_to_paths(&dir, command);
        let args = resolved.iter().map(String::as_str).collect::<Vec<_>>();
        let output = run_str(&args);
        assert!(
            output.status.success(),
            "manifest command failed: graph_mutation {}\nstderr={}",
            command.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    assert_eq!(
        fs::read_to_string(dir.join("patch.diff")).expect("read manifest generated patch"),
        fs::read_to_string(dir.join("expected-patch.diff")).expect("read expected patch")
    );
    assert!(dir.join("patch-receipt.ndjson").exists());
    assert!(dir.join("mutation-receipt.ndjson").exists());
}

#[test]
fn graph_mutation_cli_rejects_invalid_command_shape() {
    let output = run_str(&["verify-landing"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    assert!(stderr.contains("invalid graph_mutation command"));
    assert!(stderr.contains("verify-landing"));
}
