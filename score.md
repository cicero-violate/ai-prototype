# Canon Agent Scorecard

## Variables

```text
K = deterministic kernel strength
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

One-line explanation: the repository has a serious deterministic-runtime shape, but reproduced validation remains the limiting axis.

## Score Summary

```text
K = 8.4 / 10
C = 7.7 / 10
V = 8.2 / 10
P = 6.1 / 10
B = 3.0 / 10
E = 4.1 / 10
N = 5.6 / 10
D = 6.8 / 10

S = 5.92 / 10
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10 = good
```

## Evidence Reviewed

```text
uploaded_bundle = /mnt/data/ai.bundle
repo_path = /mnt/data/ai-repo
bundle_main_head = b9830281da5618db55c12371ec1f17b3abdd0b00
current_head_after_eval = 8f635e9de6dfca3b7cc072e2b8482e06879ccbd8
branch = main
goal_md_present = true
readme_present = false
cargo_toml_present = true
cargo_edition = 2024
cargo_dependency_count = 0
rust_source_files = 53
rust_test_markers = 103
rust_unwrap_calls = 316
rust_expect_calls = 9
panic_todo_unimplemented_markers = 0
unsafe_policy = src/lib.rs and src/main.rs forbid unsafe_code
state_rustc_graph_json_present = false
```

Recent history inspected:

```text
8f635e9 Evaluate ai repository scorecard
b983028 ready for agent run
b16db0a Add observe command output evidence paths
653bb6f Plan validation evidence gate
ef2c405 Evaluate ai repository scorecard
ca0537f ready for agent run
4c9025d auto
82f7eb5 Add observe validation harness
```

Bundle heads inspected:

```text
b9830281da5618db55c12371ec1f17b3abdd0b00 refs/heads/main
0539a2f67d348d60b10368140dc01e649c026263 refs/heads/backup-before-bundle-use
7cd479809f5329ca70bb0c51baed405ddc878372 refs/remotes/origin/main
b16db0a1293a48f55d5ebc4aaee3bb3c00cb08f9 worktrees/ca0537f311aa/HEAD
```

## Reproduced Root Validation

Command run:

```text
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz bash scripts/observe_validation.sh
```

Observed output:

```text
report = target/observe/validation-report.ndjson
report_lines = 10
git_status_clean_at_start = true
cargo_available = false
rustc_available = false
rustc_wrapper_configured = true
rustc_wrapper_path = /workspace/ai_sandbox/canon-rustc-v2/target/debug/canon-rustc-v2
rustc_wrapper_path_exists = false
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
state_graph_present = false
missing_signal_count = 13
```

Judgment: the observe harness is useful because it records unavailable validation surfaces explicitly. It does not prove Rust correctness in this container.

## Uploaded Runtime Archive Evidence

Relevant archive:

```text
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_manifest_base_commit = b9830281da5618db55c12371ec1f17b3abdd0b00
runtime_member_count = 3
included_files = 2
excluded_files = 2
ndjson_log_lines_total = 2
download_history_count = 0
download_history_by_classification = {}
conversation_snapshot_count = 0
cache_file_count = 0
leak_scan_findings = 0
schema_scan_findings = 0
integrity_scan_findings = 0
```

Runtime records inspected:

```text
.repo-agent-runtime/audit.ndjson = 1 line, event=loop_iteration_observed_audit_linked
log/chatgpt_project_agent.ndjson = 1 line, event=loop_iteration_observed
RUNTIME_MANIFEST.json = sanitized manifest, schemaVersion=1
```

Adjacent uploaded runtime archives were also inspected for cache/download evidence:

```text
router-server-runtime.tar.gz = 3 members, 2 NDJSON lines total, downloadHistory=[]
chatgpt-project-agent-runtime.tar.gz = 3 members, 2 NDJSON lines total, downloadHistory=[]
```

Judgment: runtime evidence is hygienic but shallow. It proves sanitized preflight capture and zero scan findings, not end-to-end execution, artifact recovery, or model-turn lineage.

## Nested Router-Server Validation

Path inspected:

```text
ai-chromium/router-server
```

Static evidence:

```text
router_mjs_files = 49
package_json_present = false
test_fixture_artifact_dirs_present = false
```

Commands run:

```text
node src/tools/check-syntax.mjs                  = pass, syntax_ok files=49
node --test test/openai-contract.test.mjs        = pass, 7/7
node --test test/mock-cdp-integration.test.mjs   = pass, 3/3
node --test test/artifact-quality.test.mjs       = fail, 0/5
```

Failure cause:

```text
artifact_quality_failure = ENOENT on test/fixtures/artifacts/{valid,missing-manifest,replay-mismatch,redaction-fail,malformed-evidence}
```

Judgment: the nested router has real offline validation for syntax, OpenAI envelope behavior, and mocked CDP behavior. The newest artifact-quality gate is not reproducible because the committed tests refer to missing fixtures.

## Critical Judgment

`GOAL.md` describes a coherent architecture: frozen deterministic kernel, typed capability layer, append-only policy store, replayable TLog, bounded recovery, and LLM promotion through learned policy.

The source tree broadly matches that architecture:

```text
src/kernel
src/codec
src/runtime
src/api
src/capability/{observation,context,memory,planning,llm,judgment,tooling,verification,eval,policy,learning,orchestration}
```

The strongest evidence is static structure plus a large inline Rust test surface. The weakest evidence is reproduced execution: root Rust validation cannot run here, graph telemetry is absent, and runtime archives contain only preflight metadata.

This repository should be classified as a serious deterministic runtime prototype, not yet a reproducibly validated autonomous agent.

## Risk Register

| Risk | Severity | Evidence | Required closure |
|---|---:|---|---|
| Root Rust validation unavailable | High | `cargo` and `rustc` not in PATH | Provide toolchain or bundle validation logs with hashes |
| Configured rustc wrapper missing | High | wrapper path points to `/workspace/.../canon-rustc-v2`, absent here | Make wrapper optional or include/repoint it for validation |
| No generated graph evidence | High | no `state/rustc/*/graph.json` found | Regenerate graph and include telemetry |
| Runtime archive too thin | High | only 3 members, 2 NDJSON lines, no snapshots/downloads | Preserve sanitized conversation, download, and apply-worktree evidence |
| Router artifact-quality test broken | Medium | 0/5 fixture tests fail with ENOENT | Commit fixtures or remove the invalid gate |
| No README | Medium | `README.md` absent | Add operator-facing restore/build/test instructions |
| Heavy unwrap use | Medium | 316 `.unwrap()` calls | Separate test-only unwraps from production-path unwraps |
| External loop not proven | High | no observation→action→verification→learning replay trace | Add one durable integration trace |

## Module Scorecard

| Area | Score | Evidence | Constraint |
|---|---:|---|---|
| Kernel | 8.4 | deterministic module boundary, gate/phase/state exports, unsafe forbidden at crate root | no formal proof artifact here |
| Codec / TLog | 7.7 | NDJSON codec exports and replay-facing APIs | schema compatibility not independently stress-tested |
| Runtime / replay / verification | 8.2 | durable runtime, replay report, verification proof exports | cargo tests unavailable here |
| Capability layer | 6.1 | all planned capability directories present | maturity is mostly static without external loop evidence |
| Build/test reproducibility | 3.0 | observe harness reports missing surfaces | no root Rust validation reproduced |
| Runtime evidence | 4.1 | sanitized archive, zero scan findings | no conversation/download/apply evidence |
| Nested router-server | 5.6 | 49-file syntax pass, OpenAI contract pass, mock CDP pass | artifact-quality fixture failure; no live CDP run |
| Documentation alignment | 6.8 | detailed `GOAL.md`, scorecard, implementation plan | no README and several claims remain unproven locally |

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
missing_semantic_artifact_verification_test = true
missing_router_artifact_quality_fixtures = true
missing_live_cdp_validation = true
```

## Highest-Leverage Next Work

1. Restore a Rust toolchain and either provide the configured wrapper or disable it intentionally for container validation.
2. Run `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
3. Regenerate `state/rustc/ai/graph.json` and capture wrapper telemetry.
4. Fix router artifact-quality fixtures so the committed gate is reproducible.
5. Capture one sanitized end-to-end trace: observation ingress → command → effect receipt → semantic verification → eval → learning/policy promotion.

## Verdict

```text
classification = serious_deterministic_runtime_prototype
not_yet = reproducibly_validated_autonomous_agent
main_blocker = missing_root_rust_validation + absent_graph_telemetry + thin_runtime_archive + broken_router_fixture_gate
score_confidence = medium_static_low_runtime
```