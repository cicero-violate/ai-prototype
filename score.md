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

One-line explanation: the repository has a coherent deterministic-agent architecture, but local proof is still bounded by missing Rust toolchain validation and one reproduced nested test failure.

## Score Summary

```text
K = 8.4 / 10
C = 7.7 / 10
V = 7.8 / 10
P = 6.1 / 10
B = 2.8 / 10
E = 5.4 / 10
N = 4.8 / 10
D = 6.7 / 10

S = 5.91 / 10
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10 = good
```

## Evidence Reviewed

```text
uploaded_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/ai-restored
uploaded_runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_extract_path = /mnt/data/ai-runtime-extracted
current_head = 12f02b6a0f3936ab5ffd8ff59762b71084c443fb
branch = main
goal_md_present = true
readme_present = false
cargo_toml_present = true
cargo_edition = 2024
cargo_dependency_count = 0
rust_source_files = 53
rust_test_markers = 104
rust_unwrap_calls = 316
rust_expect_calls = 9
panic_todo_unimplemented_markers = 0
state_rustc_graph_json_present = false
```

Recent history inspected:

```text
12f02b6 ready for agent run
bac66f7 Close router artifact quality gate
e901518 Plan router artifact fixture gate
eae2039 Observe ai repository evidence
8f635e9 Evaluate ai repository scorecard
b983028 ready for agent run
b16db0a Add observe command output evidence paths
653bb6f Plan validation evidence gate
```

Bundle refs inspected:

```text
12f02b6a0f3936ab5ffd8ff59762b71084c443fb refs/heads/main
0539a2f67d348d60b10368140dc01e649c026263 refs/heads/backup-before-bundle-use
7cd479809f5329ca70bb0c51baed405ddc878372 refs/remotes/origin/main
12f02b6a0f3936ab5ffd8ff59762b71084c443fb HEAD
b16db0a1293a48f55d5ebc4aaee3bb3c00cb08f9 worktrees/ca0537f311aa/HEAD
0539a2f67d348d60b10368140dc01e649c026263 worktrees/dc1f3f8227ed/HEAD
bac66f75055cb5238f38d8888110ea2a06ec224c worktrees/b9830281da56/HEAD
```

## GOAL.md Alignment

`GOAL.md` defines a deterministic, self-improving agent runtime with a frozen kernel, typed capability layer, append-only policy store, replayable TLog, bounded recovery, and LLM promotion through learned policy.

The source tree broadly matches the stated architecture:

```text
src/kernel
src/codec
src/runtime
src/api
src/capability/context
src/capability/eval
src/capability/judgment
src/capability/learning
src/capability/llm
src/capability/memory
src/capability/observation
src/capability/orchestration
src/capability/planning
src/capability/policy
src/capability/tooling
src/capability/verification
```

Judgment: architecture and module topology are strong. The evidence gap is not structure; it is reproducible execution proof in this restored environment.

## Reproduced Root Validation

Command run:

```text
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz bash scripts/observe_validation.sh
```

Observed output:

```text
report = target/observe/validation-report.ndjson
report_lines = 10
git_head = 12f02b6a0f3936ab5ffd8ff59762b71084c443fb
git_status_clean_at_start = true
cargo_available = false
rustc_available = false
python3_available = true
rustc_wrapper_configured = true
rustc_wrapper_path = /workspace/ai_sandbox/canon-rustc-v2/target/debug/canon-rustc-v2
rustc_wrapper_path_exists = false
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
state_graph_present = false
runtime_archive_present = true
runtime_archive_member_count = 13
runtime_archive_download_total = 6
runtime_archive_log_total = 640
missing_signal_count = 11
```

Judgment: the observe harness is valuable because it records unavailable validation surfaces explicitly. It does not prove Rust correctness in this container.

## Uploaded Runtime Archive Evidence

Runtime archive inspected:

```text
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_manifest_base_commit = 12f02b6a0f3936ab5ffd8ff59762b71084c443fb
runtime_member_count = 13
included_files = 12
excluded_files = 258
download_history_count_in_manifest = 1
download_history_by_classification = {stale_advisory: 1}
download_ndjson_records = 6
message_snapshot_records = 314
candidate_ledger_records = 3
audit_records = 11
process_log_records = 55
network_request_records_total = 251
log_total = 640
conversation_snapshot_count = 0
cache_file_count = 0
```

Runtime records inspected:

```text
.repo-agent-runtime/audit.ndjson events:
  loop_iteration_observed_audit_linked = 3
  runtime_archive_created = 1
  candidate_ledger_written = 5
  delta_pair_verified = 1
  delta_applied = 1

log/chatgpt_project_agent.ndjson events:
  startup = 1
  turn-start = 4
  turn-complete = 4
  background-download-complete = 4
  final-conversation-download = 1
  delta-applied = 1

.repo-agent-runtime/delta-apply-receipts/bac66f75055cb5238f38d8888110ea2a06ec224c.json:
  decision = accepted
  merged = true
  baseCommit = b9830281da5618db55c12371ec1f17b3abdd0b00
  afterHead = bac66f75055cb5238f38d8888110ea2a06ec224c
```

Judgment: runtime evidence is no longer thin. It contains sanitized message, network, download, audit, and delta-apply records. It is still not sufficient end-to-end proof for the current head because the manifest marks the downloaded `DELTA_MANIFEST.md` as `stale_advisory` due to a base-commit mismatch, and no generated graph or live Rust validation artifact is present.

## Nested Router-Server Validation

Path inspected:

```text
ai-chromium/router-server
```

Commands run:

```text
node src/tools/check-syntax.mjs                = pass, syntax_ok files=49
node --test test/openai-contract.test.mjs      = pass, 7/7
node --test test/mock-cdp-integration.test.mjs = pass, 3/3
node --test test/artifact-quality.test.mjs     = fail, 0/5
```

Failure detail:

```text
artifact-quality expected turn_count = 1
actual turn_count = 2 for each fixture case
fixture tree contains both turn-001 and turn_* directories per case
```

Judgment: router syntax, OpenAI-compatible envelope behavior, and mocked CDP behavior validate offline. Artifact-quality validation currently regresses because duplicated fixture turn directories make the test expectation wrong or the fixture set stale. This directly lowers nested evidence quality.

## Critical Judgment

This repository should be classified as a serious deterministic runtime prototype, not a reproducibly validated autonomous agent.

The strongest evidence is the architectural topology: kernel/codec/runtime/API/capability separation, explicit `GOAL.md`, strict `.cargo/config.toml` flags, unsafe-code forbiddance, NDJSON/TLog-oriented runtime code, and structured runtime archive capture.

The weakest evidence is reproduced execution: root Rust validation cannot run here, the configured rustc wrapper path is absent, graph telemetry is absent, the Ollama example is environment-gated, and one nested router-server test suite now fails.

## Risk Register

| Risk | Severity | Evidence | Required closure |
|---|---:|---|---|
| Root Rust validation unavailable | High | `cargo` and `rustc` not in PATH | Provide toolchain or committed validation logs with hashes |
| Configured rustc wrapper missing | High | wrapper path points to `/workspace/.../canon-rustc-v2`, absent here | Make wrapper optional for portable validation or include/repoint it |
| No generated graph evidence | High | no `state/rustc/*/graph.json` found | Regenerate graph and include telemetry |
| Nested artifact-quality regression | High | `artifact-quality.test.mjs` fails 0/5 because fixture turn count is 2 not 1 | Deduplicate fixture turns or update expectation semantics |
| Runtime archive has stale advisory artifact | Medium | manifest classifies `DELTA_MANIFEST.md` as `stale_advisory` due to base mismatch | Ensure archived downloads correspond to current base/head |
| Live CDP not reproduced | Medium | live test not run; only mock CDP test passed | Run live browser/CDP validation with sanitized outputs |
| No README | Medium | `README.md` absent at root | Add operator-facing restore/build/test instructions |
| Heavy unwrap use | Medium | 316 `.unwrap()` calls under `src`/`examples` | Separate test unwraps from production-path unwraps |
| External loop not proven | High | no current observation→action→verification→learning replay trace | Add one durable integration trace |

## Module Scorecard

| Area | Score | Evidence | Constraint |
|---|---:|---|---|
| Kernel | 8.4 | deterministic module boundary, gate/phase/state exports, unsafe forbidden at crate root | no formal proof artifact here |
| Codec / TLog | 7.7 | NDJSON codec exports and replay-facing APIs | schema compatibility not independently stress-tested here |
| Runtime / replay / verification | 7.8 | durable runtime, command ledger, verification proof modules, runtime archive records | root cargo tests unavailable |
| Capability layer | 6.1 | planned capability directories present | maturity remains mostly static without external-loop proof |
| Build/test reproducibility | 2.8 | observe harness reports unavailable Rust surfaces | no cargo/rustc in PATH and wrapper missing |
| Runtime evidence | 5.4 | 13 archive members, 640 log lines, 6 download records, audit chain records | stale advisory artifact and no graph/current validation proof |
| Nested router-server | 4.8 | syntax, OpenAI contract, and mock CDP pass | artifact-quality suite fails 0/5 |
| Documentation alignment | 6.7 | detailed `GOAL.md`, implementation plan, scorecard | no root README; claims outrun reproduced validation |

## Missing Validation Signals

```text
missing_cargo_fmt = true
missing_cargo_test = true
missing_clippy = true
missing_ollama_judgment_run = true
missing_generated_graph_json = true
missing_rustc_wrapper_telemetry = true
missing_conversation_snapshot = true
missing_download_history = false
missing_apply_worktree = false
missing_external_observation_stream_test = true
missing_external_api_action_test = true
missing_policy_learning_replay_trace = true
missing_semantic_artifact_verification_test = true
missing_router_artifact_quality_fixtures = true
missing_live_cdp_validation = true
```

## Highest-Leverage Next Work

1. Restore a Rust toolchain and either provide the configured wrapper or make wrapper use optional for portable validation.
2. Run `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
3. Fix or deduplicate `ai-chromium/router-server/test/fixtures/artifacts/*` so `artifact-quality.test.mjs` passes again.
4. Regenerate `state/rustc/ai/graph.json` and capture wrapper telemetry.
5. Capture one current-head end-to-end trace: observation ingress → command → effect receipt → semantic verification → eval → learning/policy promotion.
6. Run live CDP validation and retain sanitized artifact-quality outputs tied to the current base/head.

## Verdict

```text
classification = serious_deterministic_runtime_prototype
not_yet = reproducibly_validated_autonomous_agent
main_blocker = missing_root_rust_validation + missing_wrapper + absent_graph_telemetry + failing_router_artifact_quality_suite
score_confidence = medium_static_low_runtime
```