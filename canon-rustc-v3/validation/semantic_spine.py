#!/usr/bin/env python3
"""Validate the semantic witness spine.

Without arguments this is a static fallback for sandboxes without cargo/rustc.
With ``--graph`` or ``--compare`` it validates emitted graph artifacts too.
With ``--report`` it validates the portable validation receipt schema.
With ``--delta-report`` it validates semantic-delta gate output.
With ``--scale-report`` it validates semantic scale/performance evidence.
With ``--preflight-report`` it validates portable preflight evidence.
With ``--reproducibility-report`` it validates future-proofing evidence.
With ``--performance-report`` it validates hash-bound performance evidence.
With ``--runtime-receipt-report`` it validates runtime receipt gate evidence.
With ``--delta-contract-report`` it validates delta manifest evidence.
With ``--toolchain-archive-report`` it validates offline Rust archive evidence.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import pathlib
import re
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]
FACTS = {"fn", "trait", "impl", "call", "mut", "io", "unsafe", "panic", "alloc"}
RISK = {"mut", "io", "unsafe", "panic", "alloc"}
HASH_FIELDS = {"receipt_hash", "graph_hash", "intent_hash", "risk_hash"}
RECEIPT_FIELD = "receipt_hash"
UNCLASSIFIED_INTENT_HASH = hashlib.sha256(b"intent:unclassified").hexdigest()


def read(relative: str) -> str:
    return (ROOT / relative).read_text(encoding="utf-8")


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def string_literals(text: str) -> set[str]:
    return set(re.findall(r'"([a-z_]+)"', text))


def non_empty_config_string(text: str, key: str) -> bool:
    pattern = rf"^\s*{re.escape(key)}\s*=\s*\"([^\"]*)\"\s*$"
    return any(match.group(1) for match in re.finditer(pattern, text, re.MULTILINE))


def validate_static() -> None:
    config = read(".cargo/config.toml")
    toolchain = read("rust-toolchain.toml")
    facts = read("src/facts.rs")
    fixture = read("validation/fixtures/semantic_facts/src/lib.rs")
    witness = read("validation/fixtures/witness_crate/src/lib.rs")
    bin_main = read("src/bin/canon_rustc_v3.rs")
    lib = read("src/lib.rs")
    hir = read("src/hir.rs")
    mir = read("src/mir.rs")
    graph = read("src/graph.rs")
    emit = read("src/emit.rs")
    flags = read("src/flags.rs")
    index = read("src/index.rs")
    wrapper = read("src/wrapper.rs")
    toolchain_archive_gate = read("validation/toolchain_archive_gate.py")

    literals = string_literals(facts)
    require(FACTS <= literals, f"missing canonical facts: {sorted(FACTS - literals)}")
    require(RISK <= literals, f"missing risk facts: {sorted(RISK - literals)}")
    require("pub mod facts;" in lib, "facts module is not exported")
    require("fn fact_relations_for_callee" not in hir + mir, "callee classifier is duplicated")
    require("crate::facts::callee_relations" in hir, "HIR does not use shared classifier")
    require("crate::facts::callee_relations" in mir, "MIR does not use shared classifier")
    require("facts::allowed_relation" in wrapper, "wrapper does not use shared relation gate")
    require("facts::risk_relation" in wrapper, "wrapper does not use shared risk gate")
    require("pub intents: BTreeMap<String, String>" in graph, "graph does not persist intents")
    require("BTreeMap<String, CrateIndexEntry>" in emit, "index output is not ordered")
    require("should_keep_existing_graph" not in emit, "emit may retain stale graph artifacts")
    require("existing_nodes > new_nodes" not in emit, "emit may prefer stale larger graphs")
    require("unwrap_or_default()" not in emit, "emit index parsing can silently discard corrupted state")
    require("fn read_index(" in emit, "emit lacks an explicit fail-closed index reader")
    require("serde_json::from_slice(&bytes)" in emit, "emit does not parse existing index JSON explicitly")
    require("invalid index json" in emit, "emit parse failure does not name invalid index JSON")
    require("strip_prefix(artifact_root)" in emit, "index artifact_dir is not root-relative")
    require("fn write_graph_atomically(" in emit, "emit does not replace graph through an atomic helper")
    require("fs::write(&graph_path" not in emit, "emit writes graph artifacts directly")
    require("fs::rename(&tmp_path, graph_path)" in emit, "emit graph replacement is not rename-based")
    require("fn write_index_atomically(" in emit, "emit does not replace index through an atomic helper")
    require("fs::rename(&tmp_path, index_path)" in emit, "emit index replacement is not rename-based")
    require("set.insert(parent)" in index, "DefId parent closure does not add missing parents")
    require("set.remove(&def_id)" not in index, "DefId parent closure still removes children")
    require("collect_hir(tcx, workspace_root)" in wrapper, "wrapper does not pass workspace root into HIR extraction")
    require("workspace_root: &Path" in hir, "HIR extraction cannot normalize source spans against workspace root")
    require("strip_prefix(workspace_root)" in hir, "HIR source spans are not workspace-root relative")
    require("replace('\\\\', \"/\")" in hir, "HIR source spans are not slash-normalized")
    require("file: path.display().to_string()" not in hir, "HIR source spans may persist host-specific absolute paths")
    require('expect("failed to exec rustc")' not in bin_main, "probe passthrough still panics on rustc exec failure")
    require("canon-rustc-v3: failed to exec rustc" in bin_main, "probe passthrough lacks explicit exec failure diagnostics")
    require("CANON_RUSTC_V3_ARTIFACT_DIR" in flags, "v3 artifact env is missing")
    require(not non_empty_config_string(config, "rustc-wrapper"), "config forces a host-specific wrapper")
    require(not non_empty_config_string(config, "rustc"), "config forces a host-specific rustc binary")
    require("/mnt/data" not in config, "config contains sandbox-specific paths")
    require(
        bool(re.search(r'^\s*channel\s*=\s*"nightly-\d{4}-\d{2}-\d{2}"\s*$', toolchain, re.MULTILINE)),
        "rust-toolchain.toml must pin nightly as nightly-YYYY-MM-DD",
    )
    require('"rustc-dev"' in toolchain, "rust-toolchain.toml lacks rustc-dev")
    require('"llvm-tools-preview"' in toolchain, "rust-toolchain.toml lacks llvm-tools-preview")
    require("fn intent_hash(" in wrapper, "wrapper lacks derived intent hash")
    require("struct ReceiptEnvelope" in wrapper, "wrapper lacks typed receipt envelope")
    require("fn receipt_hash(" in wrapper, "wrapper lacks derived receipt hash helper")
    require("RECEIPT_SCHEMA_VERSION" in wrapper, "wrapper lacks receipt schema version binding")
    require("graph_schema_version" in wrapper, "receipt envelope does not bind graph schema version")
    require('stable_json_bytes(&envelope, "receipt envelope")' in wrapper, "receipt hash is not canonical JSON")
    require("format!(\"{crate_name}:{graph_hash}:{intent_hash}:{risk_hash}\")" not in wrapper, "receipt hash still uses string concatenation")
    require("fn function_intents(" in wrapper, "wrapper lacks persisted intent map")
    require("fn intent_for(" in wrapper, "wrapper lacks function intent classifier")
    require("fn stable_json_bytes" in wrapper, "wrapper lacks fail-closed stable hash serialization")
    require("unwrap_or_default()" not in wrapper, "wrapper stable hashes can silently default")
    require("intent:unclassified" not in wrapper, "intent hash is still constant")
    require("import tarfile" in toolchain_archive_gate, "toolchain archive gate must use Python tarfile")
    require("import subprocess" not in toolchain_archive_gate, "toolchain archive gate must not shell out to tar")
    require('"tar"' not in toolchain_archive_gate, "toolchain archive gate must not invoke shell tar")
    require("max_scanned_members" in toolchain_archive_gate, "toolchain archive gate lacks bounded total member scan")
    require("scanned >= max_scanned_members" in toolchain_archive_gate, "toolchain archive gate does not stop on total scanned members")

    preflight = read("validation/semantic_preflight.py")
    scale_probe = read("validation/semantic_scale_probe.py")
    require("def receipt_envelope(" in preflight, "preflight cannot recompute typed receipt envelopes")
    require("RECEIPT_SCHEMA_VERSION" in preflight, "preflight lacks receipt schema version")
    require("graph_schema_version" in preflight, "preflight receipt recomputation does not bind graph schema")
    require("f\"{crate_name}:{recomputed['graph_hash']}" not in preflight, "preflight still recomputes legacy receipt strings")
    require("RECEIPT_SCHEMA_VERSION" in scale_probe, "scale probe lacks receipt schema version")
    require("graph_schema_version" in scale_probe, "scale probe does not synthesize typed receipt envelope")
    runtime_receipt_gate = read("validation/runtime_receipt_gate.py")
    require("def normalized_record_path(" in runtime_receipt_gate, "runtime receipt gate lacks normalized path handling")
    require("def path_has_component(" in runtime_receipt_gate, "runtime receipt gate lacks component-aware path checks")
    require("path_has_component(path, \"delta-apply-receipts\")" in runtime_receipt_gate, "runtime receipt gate can miss relative delta receipt paths")
    require("path_has_component(path, \"loop-stop-receipts\")" in runtime_receipt_gate, "runtime receipt gate can miss relative loop-stop receipt paths")

    for intent in ["pure", "io", "mutation", "orchestration", "validation", "unsafe", "boundary"]:
        require(intent in string_literals(wrapper), f"missing intent class {intent!r}")

    for token in ["trait Worker", "impl Worker", "worker.run", "*value +=", "read_to_string", "expect", "vec!", "unsafe"]:
        require(token in fixture, f"fixture lacks {token!r}")
        require(token in witness, f"witness fixture lacks {token!r}")


def graph_paths(root: pathlib.Path) -> list[pathlib.Path]:
    if root.name == "graph.json":
        return [root]
    return sorted(root.rglob("graph.json"))


def load_graph(path: pathlib.Path) -> dict:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def validate_graph(path: pathlib.Path) -> None:
    graph = load_graph(path)
    meta = graph.get("meta", {})
    nodes = graph.get("nodes", {})
    edges = graph.get("edges", [])
    intents = graph.get("intents", {})

    require(HASH_FIELDS <= set(meta), f"{path}: missing replay hash fields")
    for field in HASH_FIELDS:
        require(bool(meta.get(field)), f"{path}: empty {field}")
    require(meta.get("intent_hash") != UNCLASSIFIED_INTENT_HASH, f"{path}: unclassified intent hash")

    observed = {node.get("kind") for node in nodes.values()}
    observed |= {edge.get("relation") for edge in edges}
    require(FACTS <= observed, f"{path}: missing facts {sorted(FACTS - observed)}")
    fn_paths = {node.get("path") for node in nodes.values() if node.get("kind") == "fn"}
    require(fn_paths <= set(intents), f"{path}: missing intents for functions")
    allowed = {"pure", "io", "mutation", "orchestration", "validation", "unsafe", "boundary"}
    require(set(intents.values()) <= allowed, f"{path}: invalid intent values")


def replay_view(path: pathlib.Path) -> dict:
    graph = copy.deepcopy(load_graph(path))
    graph.get("meta", {}).pop("captured_at_ms", None)
    return graph


def validate_graph_root(root: pathlib.Path) -> None:
    paths = graph_paths(root)
    require(bool(paths), f"no graph.json files under {root}")
    for path in paths:
        validate_graph(path)


def compare_roots(left: pathlib.Path, right: pathlib.Path) -> None:
    left_paths = {path.parent.name: path for path in graph_paths(left)}
    right_paths = {path.parent.name: path for path in graph_paths(right)}
    require(left_paths.keys() == right_paths.keys(), "graph crate sets differ")
    for crate_name, left_path in left_paths.items():
        require(
            replay_view(left_path) == replay_view(right_paths[crate_name]),
            f"{crate_name}: replay-critical graph changed",
        )


def load_json(path: pathlib.Path) -> dict:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def canonical_hash(value: object) -> str:
    payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(payload).hexdigest()


def require_receipt_hash(report: dict, path: pathlib.Path) -> None:
    receipt = report.get(RECEIPT_FIELD)
    stable = dict(report)
    stable.pop(RECEIPT_FIELD, None)
    require(isinstance(receipt, str) and receipt, f"{path}: missing receipt_hash")
    require(receipt == canonical_hash(stable), f"{path}: invalid receipt_hash")


def validate_report(path: pathlib.Path) -> None:
    report = load_json(path)
    require_receipt_hash(report, path)
    require(report.get("schema_version") == 1, f"{path}: unsupported report schema")
    require(report.get("status") in {"pass", "pass_with_skip", "fail", "running"}, f"{path}: invalid status")
    require(isinstance(report.get("commands"), list), f"{path}: missing command receipts")
    require(isinstance(report.get("tool_availability"), dict), f"{path}: missing tool availability")
    require(report.get("cargo_validation") in {"not_started", "static_only", "skipped", "required_unavailable", "running", "ran"}, f"{path}: invalid cargo validation state")

    for tool_name in ["cargo", "rustc", "rustup"]:
        tool = report["tool_availability"].get(tool_name)
        require(isinstance(tool, dict), f"{path}: missing {tool_name} tool receipt")
        require("available" in tool and "path" in tool and "version" in tool, f"{path}: incomplete {tool_name} tool receipt")

    for command in report["commands"]:
        require(isinstance(command.get("argv"), list), f"{path}: command missing argv")
        require(isinstance(command.get("elapsed_ms"), (int, float)), f"{path}: command missing elapsed_ms")
        require(isinstance(command.get("returncode"), int), f"{path}: command missing returncode")

    if report.get("cargo_validation") == "ran":
        require(report.get("replay_comparison") == "passed", f"{path}: replay did not pass")
        require(bool(report.get("graphs")), f"{path}: missing graph metrics")
        perf = report.get("performance")
        require(isinstance(perf, dict), f"{path}: missing performance receipt")
        require(isinstance(perf.get("baseline_ms"), (int, float)), f"{path}: missing baseline timing")
        require(isinstance(perf.get("wrapped_ms"), (int, float)), f"{path}: missing wrapped timing")
        require(isinstance(perf.get("overhead_ratio"), (int, float)), f"{path}: missing overhead ratio")
    if report.get("status") == "pass_with_skip":
        require(bool(report.get("skip_reason")), f"{path}: skip lacks reason")


def validate_delta_report(path: pathlib.Path) -> None:
    report = load_json(path)
    require(report.get("schema_version") == 1, f"{path}: unsupported delta schema")
    for field in [
        "old_graph_hash",
        "new_graph_hash",
        "old_intent_hash",
        "new_intent_hash",
        "old_risk_hash",
        "new_risk_hash",
    ]:
        require(isinstance(report.get(field), str) and report[field], f"{path}: missing {field}")

    list_fields = [
        "added_nodes",
        "removed_nodes",
        "added_edges",
        "removed_edges",
        "intent_changes",
        "risk_added",
        "risk_removed",
        "changed_files",
        "leverage_rank",
        "prompt_facts",
        "required_validation",
    ]
    for field in list_fields:
        require(isinstance(report.get(field), list), f"{path}: {field} is not a list")

    require(isinstance(report.get("risk_delta_score"), int), f"{path}: invalid risk_delta_score")
    decision = report.get("gate_decision")
    require(isinstance(decision, dict), f"{path}: missing gate_decision")
    require(decision.get("status") in {"allow", "deny"}, f"{path}: invalid gate status")
    require(isinstance(decision.get("reason"), str) and decision["reason"], f"{path}: missing gate reason")


def validate_scale_report(path: pathlib.Path) -> None:
    report = load_json(path)
    digest = report.get("report_hash")
    stable = dict(report)
    stable.pop("report_hash", None)
    require(isinstance(digest, str) and digest == canonical_hash(stable), f"{path}: invalid report_hash")
    require(report.get("schema_version") == 1, f"{path}: unsupported scale schema")
    require(report.get("status") in {"pass", "fail"}, f"{path}: invalid scale status")
    for field in ["node_count", "edge_count", "fanout", "risk_additions", "risk_delta_score", "changed_file_count", "prompt_fact_count"]:
        require(isinstance(report.get(field), int), f"{path}: {field} is not an int")
    for field in ["elapsed_ms", "threshold_ms"]:
        require(isinstance(report.get(field), (int, float)), f"{path}: {field} is not numeric")
    for field in ["old_graph_hash", "new_graph_hash", "gate_status", "gate_reason"]:
        require(isinstance(report.get(field), str) and report[field], f"{path}: missing {field}")
    require(report["node_count"] > 0, f"{path}: node_count must be positive")
    require(report["edge_count"] >= report["node_count"], f"{path}: edge_count is too small")
    require(report["risk_additions"] > 0, f"{path}: risk_additions must be positive")
    require(report["risk_delta_score"] > 0, f"{path}: risk delta must be positive")
    require(report["gate_status"] == "deny", f"{path}: unproved risk delta should be denied")
    expected = "pass" if report["elapsed_ms"] <= report["threshold_ms"] else "fail"
    require(report["status"] == expected, f"{path}: status does not match threshold")


def validate_preflight_report(path: pathlib.Path) -> None:
    report = load_json(path)
    require_receipt_hash(report, path)
    require(report.get("schema_version") == 1, f"{path}: unsupported preflight schema")
    require(report.get("mode") == "semantic_preflight", f"{path}: invalid preflight mode")
    require(report.get("status") in {"pass", "pass_with_skip", "fail"}, f"{path}: invalid preflight status")

    tools = report.get("tool_availability")
    repo = report.get("repo_state")
    deterministic = report.get("determinism_inputs")
    missing = report.get("missing_signals")
    failures = report.get("failures")
    require(isinstance(tools, dict), f"{path}: missing tool availability")
    require(isinstance(repo, dict), f"{path}: missing repo state")
    require(isinstance(deterministic, dict), f"{path}: missing determinism inputs")
    require(isinstance(missing, list), f"{path}: missing signals is not a list")
    require(isinstance(failures, list), f"{path}: failures is not a list")

    for tool_name in ["cargo", "rustc", "rustup"]:
        tool = tools.get(tool_name)
        require(isinstance(tool, dict), f"{path}: missing {tool_name} receipt")
        require("available" in tool and "path" in tool and "version" in tool, f"{path}: incomplete {tool_name} receipt")

    for field in ["cargo_lock_present", "rust_toolchain_drift_risk", "vendor_rust_source_initialized"]:
        require(isinstance(repo.get(field), bool), f"{path}: {field} must be boolean")
    require("rust_toolchain_channel" in repo, f"{path}: missing rust toolchain channel")

    removed = deterministic.get("volatile_fields_removed")
    require(isinstance(removed, list) and "meta.captured_at_ms" in removed, f"{path}: volatile timestamp is not normalized")
    require(deterministic.get("replay_comparison") in {"not_run", "passed", "failed"}, f"{path}: invalid replay comparison")
    require(isinstance(deterministic.get("graph_roots_comparable"), bool), f"{path}: graph comparability must be boolean")

    if report["status"] == "pass_with_skip":
        require(bool(missing), f"{path}: pass_with_skip lacks missing signals")
    if report["status"] == "fail":
        require(bool(failures), f"{path}: fail lacks failure evidence")
    if report.get("live_validation"):
        normalized = deterministic.get("normalized_replay")
        require(deterministic.get("replay_comparison") == "passed", f"{path}: live validation lacks passed replay")
        require(deterministic.get("graph_roots_comparable"), f"{path}: live validation lacks comparable graph roots")
        require(isinstance(normalized, dict) and bool(normalized.get("left_graphs")), f"{path}: live validation lacks graph receipts")
    else:
        require(deterministic.get("replay_comparison") != "passed", f"{path}: passed replay must set live_validation")


def validate_reproducibility_report(path: pathlib.Path) -> None:
    report = load_json(path)
    require_receipt_hash(report, path)
    require(report.get("schema_version") == 1, f"{path}: unsupported reproducibility schema")
    require(report.get("mode") == "reproducibility_gate", f"{path}: invalid reproducibility mode")
    require(report.get("status") in {"pass", "pass_with_skip", "fail"}, f"{path}: invalid reproducibility status")

    toolchain = report.get("toolchain")
    cargo_config = report.get("cargo_config")
    lockfile = report.get("lockfile")
    submodule = report.get("submodule")
    live = report.get("live_validation")
    missing = report.get("missing_signals")
    failures = report.get("failures")
    require(isinstance(toolchain, dict), f"{path}: missing toolchain state")
    require(isinstance(cargo_config, dict), f"{path}: missing cargo config state")
    require(isinstance(lockfile, dict), f"{path}: missing lockfile state")
    require(isinstance(submodule, dict), f"{path}: missing submodule state")
    require(isinstance(live, dict), f"{path}: missing live validation state")
    require(isinstance(missing, list), f"{path}: missing signals is not a list")
    require(isinstance(failures, list), f"{path}: failures is not a list")

    for field in ["pinned", "drift_risk"]:
        require(isinstance(toolchain.get(field), bool), f"{path}: toolchain.{field} must be boolean")
    for field in ["present", "forced_rustc"]:
        require(isinstance(cargo_config.get(field), bool), f"{path}: cargo_config.{field} must be boolean")
    for field in ["present", "required"]:
        require(isinstance(lockfile.get(field), bool), f"{path}: lockfile.{field} must be boolean")
    for field in ["configured", "initialized", "required"]:
        require(isinstance(submodule.get(field), bool), f"{path}: submodule.{field} must be boolean")
    for field in ["preflight_report_present", "preflight_receipt_valid", "witness_report_present", "witness_receipt_valid", "graph_roots_comparable", "wrapper_overhead_measured"]:
        require(isinstance(live.get(field), bool), f"{path}: live_validation.{field} must be boolean")
    require(live.get("replay_comparison") in {"not_run", "passed", "failed"}, f"{path}: invalid replay comparison")

    if report["status"] == "pass_with_skip":
        require(bool(missing), f"{path}: pass_with_skip lacks missing signals")
    if report["status"] == "fail":
        require(bool(failures), f"{path}: fail lacks failure evidence")
    if report["status"] == "pass":
        require(toolchain.get("pinned") and not toolchain.get("drift_risk"), f"{path}: pass with drifting toolchain")
        require(not cargo_config.get("forced_rustc"), f"{path}: pass with forced rustc config")
        require(lockfile.get("present"), f"{path}: pass without Cargo.lock")
        require(submodule.get("initialized"), f"{path}: pass without initialized rust source")
        require(live.get("preflight_receipt_valid") and live.get("witness_receipt_valid"), f"{path}: pass without live receipt proof")
        require(live.get("graph_roots_comparable") and live.get("replay_comparison") == "passed", f"{path}: pass without live replay")
        require(live.get("wrapper_overhead_measured"), f"{path}: pass without wrapper overhead")


def validate_performance_report(path: pathlib.Path) -> None:
    report = load_json(path)
    require_receipt_hash(report, path)
    require(report.get("schema_version") == 1, f"{path}: unsupported performance schema")
    require(report.get("mode") == "performance_gate", f"{path}: invalid performance mode")
    require(report.get("status") in {"pass", "pass_with_skip", "fail"}, f"{path}: invalid performance status")

    scale = report.get("scale")
    native = report.get("native_overhead")
    missing = report.get("missing_signals")
    failures = report.get("failures")
    require(isinstance(scale, dict), f"{path}: missing scale state")
    require(isinstance(native, dict), f"{path}: missing native overhead state")
    require(isinstance(missing, list), f"{path}: missing signals is not a list")
    require(isinstance(failures, list), f"{path}: failures is not a list")

    for field in ["report_present", "report_hash_valid"]:
        require(isinstance(scale.get(field), bool), f"{path}: scale.{field} must be boolean")
    for field in ["required", "report_present", "report_hash_valid"]:
        require(isinstance(native.get(field), bool), f"{path}: native_overhead.{field} must be boolean")
    for field in ["elapsed_ms", "threshold_ms"]:
        require(isinstance(scale.get(field), (int, float)), f"{path}: scale.{field} is not numeric")

    require(scale.get("report_present"), f"{path}: performance gate lacks scale evidence")
    if report["status"] != "fail":
        require(scale.get("report_hash_valid"), f"{path}: invalid scale report hash")
        require(scale.get("elapsed_ms") <= scale.get("threshold_ms"), f"{path}: scale threshold exceeded")
        require(scale.get("status") == "pass", f"{path}: scale report did not pass")
        require(scale.get("gate_status") == "deny", f"{path}: unproved risk delta should be denied")

    if native.get("report_present") and report["status"] != "fail":
        require(native.get("report_hash_valid"), f"{path}: invalid native overhead hash")
        for field in ["baseline_ms", "wrapped_ms", "overhead_ratio"]:
            require(isinstance(native.get(field), (int, float)) and native[field] > 0, f"{path}: invalid native {field}")
        computed = native.get("computed_overhead_ratio")
        require(isinstance(computed, (int, float)), f"{path}: missing computed overhead ratio")
        require(abs(computed - native["overhead_ratio"]) <= 0.001, f"{path}: overhead ratio mismatch")

    if report["status"] == "pass_with_skip":
        require(bool(missing), f"{path}: pass_with_skip lacks missing native evidence")
        require(not native.get("report_present"), f"{path}: pass_with_skip despite native overhead report")
    if report["status"] == "fail":
        require(bool(failures), f"{path}: fail lacks failure evidence")
    if report["status"] == "pass":
        require(not missing, f"{path}: pass with missing signals")
        require(native.get("report_present"), f"{path}: pass without native overhead report")
        require(native.get("baseline_ms") and native.get("wrapped_ms"), f"{path}: pass without baseline/wrapped timings")


def validate_runtime_receipt_report(path: pathlib.Path) -> None:
    report = load_json(path)
    require_receipt_hash(report, path)
    require(report.get("schema_version") == 1, f"{path}: unsupported runtime receipt schema")
    require(report.get("mode") == "runtime_receipt_gate", f"{path}: invalid runtime receipt mode")
    require(report.get("status") in {"pass", "pass_with_skip", "fail"}, f"{path}: invalid runtime receipt status")

    counts = report.get("evidence_counts")
    missing = report.get("missing_signals")
    failures = report.get("failures")
    observations = report.get("observations")
    require(isinstance(counts, dict), f"{path}: missing evidence counts")
    require(isinstance(missing, list), f"{path}: missing_signals is not a list")
    require(isinstance(failures, list), f"{path}: failures is not a list")
    require(isinstance(observations, list), f"{path}: observations is not a list")

    for field in [
        "delta_apply_receipts",
        "loop_stop_receipts",
        "download_ledger_rows",
        "candidate_ledger_rows",
        "message_ledger_rows",
        "network_log_rows",
        "delta_manifests",
        "stale_advisories",
    ]:
        require(isinstance(counts.get(field), int), f"{path}: evidence_counts.{field} must be int")

    failure_kinds = {item.get("kind") for item in failures if isinstance(item, dict)}
    missing_kinds = {item.get("kind") for item in missing if isinstance(item, dict)}
    false_pass = {"empty_validation_false_pass", "empty_current_validation_false_pass"} & failure_kinds
    runtime_blocked = {"turn_timeout", "blocking_network_failure"} & failure_kinds

    if report["status"] == "pass":
        require(not missing, f"{path}: pass with missing runtime signals")
        require(not failures, f"{path}: pass with runtime failures")
        require(counts.get("delta_apply_receipts", 0) > 0, f"{path}: pass without delta receipts")
        require(counts.get("delta_manifests", 0) > 0, f"{path}: pass without delta manifest evidence")
    if report["status"] == "pass_with_skip":
        require(bool(missing), f"{path}: pass_with_skip lacks missing runtime signals")
        require(not failures, f"{path}: pass_with_skip with runtime failures")
        require(
            missing_kinds <= {"delta_apply_receipts_absent", "delta_manifest_absent", "unsigned_delta_receipt", "unsigned_runtime_gate", "stale_advisory_present"},
            f"{path}: pass_with_skip has unbounded missing signals",
        )
    if report["status"] == "fail":
        require(bool(failures), f"{path}: fail lacks failure evidence")
        require(bool(false_pass or runtime_blocked), f"{path}: fail lacks actionable runtime failure class")


def validate_delta_contract_report(path: pathlib.Path) -> None:
    report = load_json(path)
    require_receipt_hash(report, path)
    require(report.get("schema_version") == 1, f"{path}: unsupported delta contract schema")
    require(report.get("mode") == "delta_contract_gate", f"{path}: invalid delta contract mode")
    require(report.get("status") in {"pass", "pass_with_skip", "fail"}, f"{path}: invalid delta contract status")

    manifest = report.get("manifest")
    runtime = report.get("runtime_receipt")
    native = report.get("native_witness")
    missing = report.get("missing_signals")
    failures = report.get("failures")
    require(isinstance(manifest, dict), f"{path}: missing manifest state")
    require(isinstance(runtime, dict), f"{path}: missing runtime receipt state")
    require(isinstance(native, dict), f"{path}: missing native witness state")
    require(isinstance(missing, list), f"{path}: missing_signals is not a list")
    require(isinstance(failures, list), f"{path}: failures is not a list")

    for field in ["path", "sha256", "base_commit", "head_commit"]:
        require(isinstance(manifest.get(field), str) and manifest[field], f"{path}: manifest.{field} missing")
    for field in ["validation_commands"]:
        require(isinstance(manifest.get(field), int), f"{path}: manifest.{field} is not int")
    for field in [
        "first_lines_present",
        "bundle_verify_present",
        "duration_receipts_present",
        "test_count_present",
        "native_skip_explicit",
        "changed_files_match",
        "receiver_apply_commands_present",
    ]:
        require(isinstance(manifest.get(field), bool), f"{path}: manifest.{field} is not bool")
    for field in ["changed_files", "expected_changed_files"]:
        require(isinstance(manifest.get(field), list), f"{path}: manifest.{field} is not list")
    for state_name, state in [("runtime_receipt", runtime), ("native_witness", native)]:
        for field in ["report_present", "receipt_hash_valid"]:
            require(isinstance(state.get(field), bool), f"{path}: {state_name}.{field} is not bool")

    missing_kinds = {item.get("kind") for item in missing if isinstance(item, dict)}
    if report["status"] == "pass":
        require(not missing, f"{path}: pass with missing signals")
        require(not failures, f"{path}: pass with failures")
        require(manifest["validation_commands"] > 0, f"{path}: pass without validation commands")
        require(manifest["bundle_verify_present"], f"{path}: pass without bundle verification")
        require(manifest["duration_receipts_present"], f"{path}: pass without validation durations")
        require(manifest["test_count_present"], f"{path}: pass without test count")
        require(manifest["first_lines_present"], f"{path}: pass without required first-line commits")
        require(manifest["changed_files_match"], f"{path}: pass without exact changed files")
        require(manifest["receiver_apply_commands_present"], f"{path}: pass without receiver apply commands")
        require(runtime["report_present"] and runtime["receipt_hash_valid"], f"{path}: pass without runtime receipt hash")
        require(native["report_present"] and native["receipt_hash_valid"], f"{path}: pass without native witness hash")
    if report["status"] == "pass_with_skip":
        require(bool(missing), f"{path}: pass_with_skip lacks missing signals")
        require(not failures, f"{path}: pass_with_skip with failures")
        require(manifest["native_skip_explicit"], f"{path}: skip lacks native_tools_absent contract")
        require(manifest["first_lines_present"], f"{path}: skip without required first-line commits")
        require(manifest["changed_files_match"], f"{path}: skip without exact changed files")
        require(manifest["receiver_apply_commands_present"], f"{path}: skip without receiver apply commands")
        require(missing_kinds <= {"native_tools_absent", "runtime_receipt_rejected", "runtime_receipt_absent"}, f"{path}: unbounded missing signals")
    if report["status"] == "fail":
        require(bool(failures), f"{path}: fail lacks failure evidence")


def validate_toolchain_archive_report(path: pathlib.Path) -> None:
    report = load_json(path)
    require_receipt_hash(report, path)
    require(report.get("schema_version") == 1, f"{path}: unsupported toolchain archive schema")
    require(report.get("mode") == "toolchain_archive_gate", f"{path}: invalid toolchain archive mode")
    require(report.get("status") in {"pass", "pass_with_skip", "fail"}, f"{path}: invalid toolchain archive status")

    repo = report.get("repo_toolchain")
    archive = report.get("archive")
    requirements = report.get("requirements")
    missing = report.get("missing_signals")
    failures = report.get("failures")
    require(isinstance(repo, dict), f"{path}: missing repo toolchain state")
    require(isinstance(archive, dict), f"{path}: missing archive state")
    require(isinstance(requirements, dict), f"{path}: missing requirements state")
    require(isinstance(missing, list), f"{path}: missing_signals is not a list")
    require(isinstance(failures, list), f"{path}: failures is not a list")

    require(isinstance(repo.get("components"), list), f"{path}: repo components must be list")
    require(isinstance(archive.get("present"), bool), f"{path}: archive.present must be bool")
    require(isinstance(archive.get("readable"), bool), f"{path}: archive.readable must be bool")
    require(isinstance(requirements.get("channel_match"), bool), f"{path}: requirements.channel_match must be bool")

    inspection = archive.get("inspection")
    if inspection is not None:
        require(isinstance(inspection, dict), f"{path}: archive.inspection must be dict")
        require(inspection.get("method") == "python_tarfile_stream", f"{path}: archive was not inspected with Python tarfile")
        require(inspection.get("shell_tar_used") is False, f"{path}: shell tar was used for archive inspection")
        require(isinstance(inspection.get("sampled_prefix_members"), int), f"{path}: sampled prefix count missing")

    failure_kinds = {item.get("kind") for item in failures if isinstance(item, dict)}
    missing_kinds = {item.get("kind") for item in missing if isinstance(item, dict)}

    if report["status"] == "pass":
        require(not missing, f"{path}: pass with missing signals")
        require(not failures, f"{path}: pass with failures")
        require(archive.get("present") and archive.get("readable"), f"{path}: pass without readable archive")
        require(isinstance(inspection, dict), f"{path}: pass without archive inspection evidence")
        require(set(repo.get("components", [])) <= set(archive.get("components", [])), f"{path}: pass without required components")
        if requirements.get("channel_match"):
            require(repo.get("pinned_date") == archive.get("date"), f"{path}: pass with channel/archive date mismatch")
    if report["status"] == "pass_with_skip":
        require(bool(missing), f"{path}: pass_with_skip lacks missing signals")
        require(not failures, f"{path}: pass_with_skip with failures")
        require(missing_kinds <= {"toolchain_archive_absent"}, f"{path}: unbounded missing toolchain archive signals")
    if report["status"] == "fail":
        require(bool(failures), f"{path}: fail lacks failure evidence")
        require(
            failure_kinds
            <= {
                "toolchain_archive_unreadable",
                "toolchain_archive_missing_components",
                "toolchain_archive_missing_cargo",
                "toolchain_archive_missing_rustc",
                "toolchain_archive_channel_mismatch",
            },
            f"{path}: unbounded toolchain archive failure class",
        )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--graph", action="append", type=pathlib.Path, default=[])
    parser.add_argument("--compare", nargs=2, type=pathlib.Path, metavar=("LEFT", "RIGHT"))
    parser.add_argument("--report", action="append", type=pathlib.Path, default=[])
    parser.add_argument("--delta-report", action="append", type=pathlib.Path, default=[])
    parser.add_argument("--scale-report", action="append", type=pathlib.Path, default=[])
    parser.add_argument("--preflight-report", action="append", type=pathlib.Path, default=[])
    parser.add_argument("--reproducibility-report", action="append", type=pathlib.Path, default=[])
    parser.add_argument("--performance-report", action="append", type=pathlib.Path, default=[])
    parser.add_argument("--runtime-receipt-report", action="append", type=pathlib.Path, default=[])
    parser.add_argument("--delta-contract-report", action="append", type=pathlib.Path, default=[])
    parser.add_argument("--toolchain-archive-report", action="append", type=pathlib.Path, default=[])
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    validate_static()
    for root in args.graph:
        validate_graph_root(root)
    if args.compare:
        compare_roots(*args.compare)
    for report in args.report:
        validate_report(report)
    for report in args.delta_report:
        validate_delta_report(report)
    for report in args.scale_report:
        validate_scale_report(report)
    for report in args.preflight_report:
        validate_preflight_report(report)
    for report in args.reproducibility_report:
        validate_reproducibility_report(report)
    for report in args.performance_report:
        validate_performance_report(report)
    for report in args.runtime_receipt_report:
        validate_runtime_receipt_report(report)
    for report in args.delta_contract_report:
        validate_delta_contract_report(report)
    for report in args.toolchain_archive_report:
        validate_toolchain_archive_report(report)

    print("semantic spine static validation: ok")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AssertionError as error:
        print(f"semantic spine static validation: fail: {error}", file=sys.stderr)
        raise SystemExit(1)
