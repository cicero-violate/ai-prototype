# Canon Agent Scorecard

## Variables

```text
K = deterministic Rust kernel strength
C = codec / TLog durability strength
V = replay / verification strength
P = capability + policy + learning maturity
B = build and test reproducibility
E = runtime evidence quality
N = nested router-server evidence quality
D = documentation / goal alignment
S = overall repository score
GOOD = strongest current axis
```

## Equations

```text
S = (K · C · V · P · B · E · N · D)^(1/8)
GOOD = max(K,C,V,P,B,E,N,D)
```

One-line explanation: the architecture is strong, but reproduced validation evidence is weak.

## Score Summary

```text
K = 8.4 / 10
C = 7.7 / 10
V = 8.2 / 10
P = 6.1 / 10
B = 3.0 / 10
E = 4.2 / 10
N = 5.6 / 10
D = 6.8 / 10

S = 5.94 / 10
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10 = good
```

## Evidence Reviewed

```text
restored_bundle = /mnt/data/ai.bundle
restored_head = b9830281da5618db55c12371ec1f17b3abdd0b00
branch = main
goal_md_present = true
readme_present = false
cargo_toml_present = true
cargo_edition = 2024
cargo_dependency_count = 0
rust_files = 53
rust_test_markers = 103
rust_unwrap_calls = 316
panic_todo_unimplemented_count = 0
state_rustc_graph_json_present = false
```

Recent history shows root validation work followed by nested router-server work:

```text
b983028 ready for agent run
b16db0a Add observe command output evidence paths
653bb6f Plan validation evidence gate
ef2c405 Evaluate ai repository scorecard
ca0537f ready for agent run
```

## Reproduced Validation

Root observe harness:

```text
command = CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz bash scripts/observe_validation.sh
report = target/observe/validation-report.ndjson
git_status_clean_at_start = true
cargo_available = false
rustc_available = false
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
missing_signal_count = 13
```

Runtime archive:

```text
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_manifest_base_commit = b9830281da5618db55c12371ec1f17b3abdd0b00
runtime_member_count = 3
runtime_ndjson_logs = 2 lines total
download_history_count = 0
conversation_snapshot_count = 0
cache_file_count = 0
leak_scan_findings = 0
schema_scan_findings = 0
integrity_scan_findings = 0
```

Nested router-server evidence inside this repo:

```text
router_mjs_files = 49
node_check_all_mjs = pass
node src/tools/check-syntax.mjs = pass, syntax_ok files=49
node --test test/openai-contract.test.mjs = pass, 7/7
node --test test/mock-cdp-integration.test.mjs = pass, 3/3
node --test test/artifact-quality.test.mjs = fail, 0/5
artifact_quality_failure = missing test/fixtures/artifacts/* fixture directories
router_package_json_present = false
router_artifacts_dir_present = false
live_cdp_validation = not_reproduced
```

## Critical Judgment

`GOAL.md` defines a coherent target: frozen deterministic kernel, typed capability layer, append-only policy, replayable TLog, bounded recovery, and LLM promotion through learned policy. The Rust crate structure matches that target with `kernel`, `codec`, `runtime`, `api`, and `capability/*` modules.

The strongest axis is still the deterministic Rust kernel. The repo exports a broad replay/receipt/verification surface and has 103 checked-in Rust test markers. It also forbids unsafe Rust at crate entry points reviewed here.

The weakest axis is reproduced validation. This container has no `cargo` or `rustc`, the configured rustc wrapper path does not exist here, and no `state/rustc/ai/graph.json` is present. Therefore prior claims about cargo tests, graph metrics, and wrapper telemetry are not independently proven from this restored bundle.

The runtime archive is hygienic but thin. It proves sanitized preflight capture and clean scans, not execution quality. It contains no conversation snapshot, download history, build log, graph capture, apply worktree, or end-to-end artifact lineage.

The nested router-server has useful offline validation, but the latest committed artifact-quality test is broken because required fixture directories are absent. This directly contradicts any claim that the new artifact-quality gate is currently reproducible from the bundle.

## Module Scorecard

| Area | Score | Evidence | Risk |
|---|---:|---|---|
| Rust kernel | 8.4 | `src/kernel`, exported gates/phases/state, deterministic goal alignment | No formal proof artifact or reproduced cargo run here |
| Codec / TLog | 7.7 | `src/codec/ndjson.rs`, TLog encode/decode exports | Schema compatibility not stress-tested here |
| Runtime / replay / verification | 8.2 | `runtime/*`, verification proof exports, receipt concepts | Cargo tests and graph replay not reproduced |
| Capability layer | 6.1 | observation/context/memory/planning/llm/judgment/tooling/eval/policy/learning/orchestration modules exist | Mostly record/store surfaces; external loops not proven |
| Build/test reproducibility | 3.0 | observe harness records missing toolchain instead of hiding it | Root Rust validation unavailable; router artifact-quality test fails |
| Runtime evidence | 4.2 | sanitized archive, leak/schema/integrity scans all zero findings | Only 2 NDJSON log lines; no downstream execution evidence |
| Nested router-server | 5.6 | 49 MJS syntax pass; OpenAI contract and mock CDP tests pass | No package.json; missing fixtures; live CDP not reproduced |
| Documentation alignment | 6.8 | GOAL.md is detailed and architecture-specific | No README; several validation claims remain unreproduced |

## Missing Validation Signals

```text
missing_cargo_fmt = true
missing_cargo_test = true
missing_clippy = true
missing_ollama_judgment_run = true
missing_generated_graph_json = true
missing_rustc_wrapper_telemetry = true
missing_conversation_snapshot = true
missing_download_history = true
missing_apply_worktree = true
missing_external_observation_stream_test = true
missing_external_api_action_test = true
missing_policy_learning_replay_trace = true
missing_router_artifact_quality_fixtures = true
missing_live_cdp_validation = true
```

## Highest-Leverage Next Work

1. Restore Rust toolchain and wrapper availability, then rerun `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
2. Generate `state/rustc/ai/graph.json` and capture wrapper telemetry in the evidence bundle.
3. Add the missing router artifact-quality fixtures or remove the failing test until fixtures are committed.
4. Preserve runtime conversation snapshots, artifact manifests, and apply-worktree evidence in sanitized form.
5. Prove one external observation → action → semantic verification → policy learning loop.

## Verdict

```text
classification = serious_deterministic_runtime_prototype
not_yet = reproducibly_validated_autonomous_agent
main_blocker = missing_root_rust_validation + thin_runtime_evidence + broken_nested_artifact_quality_test
score_confidence = medium_static_low_runtime
```
