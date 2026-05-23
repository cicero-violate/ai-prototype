use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../state/tmp");
    fs::create_dir_all(&root).expect("create canonical temp root");
    let dir = root.join(format!(
        "canon-graph-mcp-{name}-{}-{nanos}",
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
    actions: Vec<WorkflowAction>,
    integrity: Vec<WorkflowIntegrityRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkflowAction {
    action: String,
    args: Vec<String>,
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
        Actions,
        Integrity,
    }

    let mut section = Section::None;
    let mut files = Vec::new();
    let mut actions = Vec::new();
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
            "actions:" => {
                section = Section::Actions;
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
            Section::Actions => {
                let Some(row) = raw_line.strip_prefix("  ") else {
                    panic!("manifest action rows must be indented: {raw_line}");
                };
                let parts = row
                    .split_whitespace()
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                assert!(!parts.is_empty(), "manifest action row must not be empty");
                assert!(
                    parts[0].starts_with("graph:"),
                    "manifest actions must use graph MCP action ids: {raw_line}"
                );
                actions.push(WorkflowAction {
                    action: parts[0].clone(),
                    args: parts[1..].to_vec(),
                });
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
        actions,
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

fn workspace_for(dir: &Path) -> ai::runtime::WorkspaceView {
    ai::runtime::WorkspaceView::new(dir.to_path_buf(), dir.to_path_buf()).expect("workspace view")
}

fn tool_text(value: Value) -> String {
    assert_eq!(
        value.get("isError"),
        Some(&Value::Bool(false)),
        "tool failed: {value}"
    );
    value["content"][0]["text"]
        .as_str()
        .expect("tool text content")
        .to_owned()
}

fn fixture_contracts(dir: &Path) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let old_graph = dir.join("old-graph.ndjson");
    let new_graph = dir.join("new-graph.ndjson");
    let ops = dir.join("ops.ndjson");
    let source_root = dir.join("source");
    fs::create_dir_all(source_root.join("src")).expect("create source root");

    write(
        &old_graph,
        &format!(
            "G|{}|1\nN|crate::a|fn|0|src/lib.rs|1|1|0|10\nN|crate::b|fn|1|src/lib.rs|2|1|10|20\nE|call|crate::a|crate::b\nI|crate::a|pure\nI|crate::b|pure\n",
            ai::GRAPH_JSON_SCHEMA_VERSION
        ),
    );
    write(
        &new_graph,
        &format!(
            "G|{}|2\nN|crate::a|fn|0|src/lib.rs|1|1|0|10\nI|crate::a|pure\n",
            ai::GRAPH_JSON_SCHEMA_VERSION
        ),
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

fn run_manifest_action(dir: &Path, action: &WorkflowAction) {
    match action.action.as_str() {
        "graph:verify_ops" => {
            assert_eq!(action.args.len(), 1);
            let input = fs::read_to_string(dir.join(&action.args[0])).expect("read ops");
            let receipt = ai::verify_graph_mutation_ops_ndjson(&input);
            assert_eq!(receipt.verdict, ai::GraphMutationVerdict::Pass);
        }
        "graph:plan_patch" => {
            assert_eq!(action.args.len(), 5);
            let workspace = workspace_for(dir);
            let value = ai::capability::execution::graph::plan_patch_tool(
                &json!({
                    "graph_contract": action.args[0],
                    "source_root": action.args[1],
                    "ops": action.args[2],
                    "patch_out": action.args[3],
                    "receipt_out": action.args[4],
                }),
                &workspace,
            );
            let payload: Value = serde_json::from_str(&tool_text(value)).expect("plan patch json");
            assert_eq!(payload["ok"], true);
        }
        "graph:verify_landing" => {
            assert_eq!(action.args.len(), 4);
            let old_graph = ai::load_graph_snapshot_contract_ndjson(dir.join(&action.args[0]))
                .expect("load old graph")
                .expect("decode old graph");
            let new_graph = ai::load_graph_snapshot_contract_ndjson(dir.join(&action.args[1]))
                .expect("load new graph")
                .expect("decode new graph");
            let ops_input = fs::read_to_string(dir.join(&action.args[2])).expect("read ops");
            let ops = ai::decode_graph_mutation_ops_ndjson(&ops_input).expect("decode ops");
            let receipt = ai::verify_graph_mutation_landing(&old_graph, &new_graph, &ops)
                .expect("verify landing");
            assert_eq!(receipt.verdict, ai::GraphMutationVerdict::Pass);
            write(
                &dir.join(&action.args[3]),
                &ai::encode_graph_mutation_receipt_ndjson(&receipt),
            );
        }
        "graph:verify_receipts" => {
            assert_eq!(action.args.len(), 2);
            let patch_input =
                fs::read_to_string(dir.join(&action.args[0])).expect("read patch receipt");
            let mutation_input =
                fs::read_to_string(dir.join(&action.args[1])).expect("read mutation receipt");
            let receipt = ai::verify_graph_receipt_ledgers_ndjson(&patch_input, &mutation_input);
            assert_eq!(receipt.verdict, ai::GraphMutationVerdict::Pass);
        }
        other => panic!("unsupported manifest action: {other}"),
    }
}

#[test]
fn graph_mcp_actions_verify_ops_and_receipts() {
    let dir = temp_dir("verify");
    let (_old_graph, _new_graph, ops, _source_root) = fixture_contracts(&dir);
    let patch_receipts = dir.join("patch-receipts.ndjson");
    let mutation_receipts = dir.join("mutation-receipts.ndjson");
    write(&patch_receipts, "");
    write(&mutation_receipts, "");

    let ops_input = fs::read_to_string(&ops).expect("read ops");
    let ops_receipt = ai::verify_graph_mutation_ops_ndjson(&ops_input);
    assert_eq!(ops_receipt.verdict, ai::GraphMutationVerdict::Pass);
    let encoded = ai::encode_graph_mutation_opset_receipt_ndjson(&ops_receipt);
    assert!(encoded.starts_with('['));
    assert!(encoded.ends_with('\n'));

    let ledger_receipt =
        ai::verify_graph_receipt_ledger_files_ndjson(&patch_receipts, &mutation_receipts)
            .expect("verify empty ledger files");
    assert_eq!(ledger_receipt.verdict, ai::GraphMutationVerdict::Pass);
    let encoded = ai::encode_graph_receipt_ledger_receipt_ndjson(&ledger_receipt);
    assert!(encoded.starts_with('['));
    assert!(encoded.ends_with('\n'));
}

#[test]
fn graph_mcp_action_generates_patch_and_landing_receipt() {
    let dir = temp_dir("roundtrip");
    let (old_graph, new_graph, ops, source_root) = fixture_contracts(&dir);
    let patch = dir.join("patch.diff");
    let patch_receipt = dir.join("patch-receipt.ndjson");
    let mutation_receipt = dir.join("mutation-receipt.ndjson");
    let workspace = workspace_for(&dir);

    let value = ai::capability::execution::graph::plan_patch_tool(
        &json!({
            "graph_contract": old_graph.strip_prefix(&dir).expect("test path should be under temp dir").to_str().expect("test path should be valid UTF-8"),
            "source_root": source_root.strip_prefix(&dir).expect("test path should be under temp dir").to_str().expect("test path should be valid UTF-8"),
            "ops": ops.strip_prefix(&dir).expect("test path should be under temp dir").to_str().expect("test path should be valid UTF-8"),
            "patch_out": patch.strip_prefix(&dir).expect("test path should be under temp dir").to_str().expect("test path should be valid UTF-8"),
            "receipt_out": patch_receipt.strip_prefix(&dir).expect("test path should be under temp dir").to_str().expect("test path should be valid UTF-8"),
        }),
        &workspace,
    );
    let payload: Value = serde_json::from_str(&tool_text(value)).expect("plan patch json");
    assert_eq!(payload["ok"], true);
    assert!(fs::read_to_string(&patch)
        .expect("read patch")
        .contains("--- a/src/lib.rs"));
    assert!(fs::read_to_string(&patch_receipt)
        .expect("read patch receipt")
        .starts_with('['));

    let old_graph = ai::load_graph_snapshot_contract_ndjson(&old_graph)
        .expect("load old graph")
        .expect("decode old graph");
    let new_graph = ai::load_graph_snapshot_contract_ndjson(&new_graph)
        .expect("load new graph")
        .expect("decode new graph");
    let ops_input = fs::read_to_string(&ops).expect("read ops");
    let ops = ai::decode_graph_mutation_ops_ndjson(&ops_input).expect("decode ops");
    let receipt =
        ai::verify_graph_mutation_landing(&old_graph, &new_graph, &ops).expect("landing receipt");
    assert_eq!(receipt.verdict, ai::GraphMutationVerdict::Pass);
    write(
        &mutation_receipt,
        &ai::encode_graph_mutation_receipt_ndjson(&receipt),
    );
    let receipt = fs::read_to_string(&mutation_receipt).expect("read mutation receipt");
    assert!(receipt.starts_with('['));
    assert!(receipt.ends_with('\n'));
}

#[test]
fn graph_mcp_actions_roundtrip_generated_receipts_into_ledger_verifier() {
    let dir = temp_dir("ledger-roundtrip");
    let (old_graph, new_graph, ops, source_root) = fixture_contracts(&dir);
    let patch = dir.join("patch.diff");
    let patch_receipt = dir.join("patch-receipt.ndjson");
    let mutation_receipt = dir.join("mutation-receipt.ndjson");
    let workspace = workspace_for(&dir);

    let value = ai::capability::execution::graph::plan_patch_tool(
        &json!({
            "graph_contract": old_graph.strip_prefix(&dir).expect("test path should be under temp dir").to_str().expect("test path should be valid UTF-8"),
            "source_root": source_root.strip_prefix(&dir).expect("test path should be under temp dir").to_str().expect("test path should be valid UTF-8"),
            "ops": ops.strip_prefix(&dir).expect("test path should be under temp dir").to_str().expect("test path should be valid UTF-8"),
            "patch_out": patch.strip_prefix(&dir).expect("test path should be under temp dir").to_str().expect("test path should be valid UTF-8"),
            "receipt_out": patch_receipt.strip_prefix(&dir).expect("test path should be under temp dir").to_str().expect("test path should be valid UTF-8"),
        }),
        &workspace,
    );
    let payload: Value = serde_json::from_str(&tool_text(value)).expect("plan patch json");
    assert_eq!(payload["ok"], true);

    let old_contract = ai::load_graph_snapshot_contract_ndjson(&old_graph)
        .expect("load old graph")
        .expect("decode old graph");
    let new_contract = ai::load_graph_snapshot_contract_ndjson(&new_graph)
        .expect("load new graph")
        .expect("decode new graph");
    let ops_input = fs::read_to_string(&ops).expect("read ops");
    let decoded_ops = ai::decode_graph_mutation_ops_ndjson(&ops_input).expect("decode ops");
    let mutation = ai::verify_graph_mutation_landing(&old_contract, &new_contract, &decoded_ops)
        .expect("verify landing");
    write(
        &mutation_receipt,
        &ai::encode_graph_mutation_receipt_ndjson(&mutation),
    );

    let ledger = ai::verify_graph_receipt_ledger_files_ndjson(&patch_receipt, &mutation_receipt)
        .expect("ledger receipt");
    assert_eq!(ledger.verdict, ai::GraphMutationVerdict::Pass);
    assert_eq!(ledger.patch_receipt_count, 1);
    assert_eq!(ledger.mutation_receipt_count, 1);
    assert_eq!(ledger.patch_receipt_count, 1);
    assert_eq!(ledger.mutation_receipt_count, 1);
    assert_eq!(ledger.invalid_line_count, 0);
}

#[test]
fn graph_mcp_actions_expose_stable_contract_surface() {
    assert_eq!(
        ai::capability::execution::action::CANON_GRAPH_PLAN_PATCH_TOOL,
        "canon_graph_plan_patch"
    );
    assert_eq!(
        ai::capability::execution::action::CANON_GRAPH_APPLY_OPS_TOOL,
        "canon_graph_apply_ops"
    );
    assert_eq!(
        ai::capability::tooling::mcp_tools::CANON_GRAPH_PLAN_CFG_TOOL,
        "canon_graph_plan_cfg"
    );
    assert_eq!(
        ai::capability::tooling::mcp_tools::CANON_GRAPH_VERIFY_CFG_DELTA_TOOL,
        "canon_graph_verify_cfg_delta"
    );
    assert_eq!(
        ai::capability::tooling::mcp_tools::CANON_GRAPH_AUTO_REFACTOR_CFG_TOOL,
        "canon_graph_auto_refactor_cfg"
    );
}

#[test]
fn graph_mcp_action_plan_patch_rejects_invalid_request_shape() {
    let dir = temp_dir("invalid-shape");
    let workspace = workspace_for(&dir);
    let value = ai::capability::execution::graph::plan_patch_tool(&json!({}), &workspace);
    assert_eq!(value.get("isError"), Some(&Value::Bool(true)));
    let text = value["content"][0]["text"].as_str().expect("error text");
    assert!(text.contains("graph_contract"));
}

#[test]
fn graph_mcp_workflow_fixture_is_copyable_contract() {
    let fixture_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/graph_mcp_workflow");
    let dir = temp_dir("workflow-fixture");
    copy_fixture_tree(&fixture_root, &dir);

    let expected_patch = dir.join("expected-patch.diff");
    let patch = dir.join("patch.diff");
    let patch_receipt = dir.join("patch-receipt.ndjson");
    let mutation_receipt = dir.join("mutation-receipt.ndjson");
    let manifest = parse_workflow_manifest(
        &fs::read_to_string(dir.join("MANIFEST.txt")).expect("read workflow manifest"),
    );

    for action in &manifest.actions {
        run_manifest_action(&dir, action);
    }

    assert_eq!(
        fs::read_to_string(&patch).expect("read generated patch"),
        fs::read_to_string(&expected_patch).expect("read expected patch")
    );
    assert!(patch_receipt.exists());
    assert!(mutation_receipt.exists());
}

#[test]
fn graph_mcp_workflow_manifest_integrity_hashes_are_stable() {
    let fixture_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/graph_mcp_workflow");
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
        "expected-patch.diff|sha256|f96d292a3e31b39b16354c6a0d3f42b25dc9ac6a40186a89c5c81fe7e7a703cc\nnew-graph.ndjson|sha256|142639ef8ebc7530ad951c9d9b092d7aa2a730e409f04a988520a07cd0be5047\nold-graph.ndjson|sha256|0b588d276bb30a01100fcc9deaa4bc8c420fe76d047afd5682108b8ff9f6bdbb\nops.ndjson|sha256|550f07baac8b60bd66f63d89c3f602da77c58437bceef8a9ce0a9e9a61d2ccb1\nsource/src/lib.rs|sha256|f334ea00880bc9f3f958220d14eefc216769a299b1db33e7dd8f699feb933b2c"
    );
}

#[test]
fn graph_mcp_workflow_manifest_integrity_detects_sample_drift() {
    let fixture_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/graph_mcp_workflow");
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
fn graph_mcp_workflow_manifest_is_executable_contract() {
    let fixture_root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/graph_mcp_workflow");
    let dir = temp_dir("workflow-manifest");
    copy_fixture_tree(&fixture_root, &dir);

    let manifest = parse_workflow_manifest(
        &fs::read_to_string(dir.join("MANIFEST.txt")).expect("read workflow manifest"),
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
    assert_eq!(manifest.actions.len(), 4);

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

    for action in &manifest.actions {
        run_manifest_action(&dir, action);
    }

    assert_eq!(
        fs::read_to_string(dir.join("patch.diff")).expect("read manifest generated patch"),
        fs::read_to_string(dir.join("expected-patch.diff")).expect("read expected patch")
    );
    assert!(dir.join("patch-receipt.ndjson").exists());
    assert!(dir.join("mutation-receipt.ndjson").exists());
}
