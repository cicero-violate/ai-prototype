# Canon Agent Scorecard

## Variables

```text
K = deterministic kernel strength
C = codec / TLog durability strength
V = replay / verification strength
P = capability + policy + learning maturity
B = build and test reproducibility
E = uploaded runtime evidence quality
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

One-line explanation: the geometric score keeps the repository bounded by the weakest proof surface.

## Score Summary

```text
K = 8.4 / 10
C = 7.8 / 10
V = 7.9 / 10
P = 6.1 / 10
B = 3.8 / 10
E = 7.0 / 10
N = 7.1 / 10
D = 7.3 / 10

S = 6.76 / 10
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10 = good
```

## Scope

```text
stage = EXECUTE turn 1
source_changes = validation receipt + delta manifest tooling only
scorecard_change = score.md evidence refresh
restored_repo_path = /mnt/data/ai-restored/repo
uploaded_bundle = /mnt/data/ai.bundle
base_commit = d47aaa7d3487521df1d0b5d2ee11310934e9a359
branch = main
kernel_runtime_source_changed = false
```

Judgment: this is a serious deterministic-runtime prototype with strong static architecture, but its autonomous-agent claims are still ahead of reproducible local proof in this sandbox.

## GOAL.md Alignment

`GOAL.md` defines a deterministic, self-improving agent runtime with a frozen kernel, append-only TLog, typed capability layer, bounded recovery, policy learning, and LLM promotion from routine work to novelty handling.

Observed source topology still matches that stated layer model:

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

Positive static evidence:

```text
src/lib.rs has #![forbid(unsafe_code)]
Cargo.toml package = ai
Cargo.toml edition = 2024
Cargo.toml dependency_count = 0
Cargo.lock package set = ai only
rust_files_src_examples = 53
rust_loc_src_examples = 15825
rust_tests_declared = 103
public_items = 215
unsafe_markers_in_src_examples = 0
todo_unimplemented_panic_markers = 0
```

Critical constraint: `GOAL.md` records prior local validation claims such as `cargo test`, Ollama judgment runs, graph metrics, and intent coverage, but those claims were not reproducible here because the Rust toolchain and configured wrapper path are unavailable.

## Recent Git History

Recent commits inspected:

```text
f81b2d9 Evaluate ai repository scorecard
  score.md only

d47aaa7 ready for agent run
  generated bundle bookkeeping only

ab4cd5b starting agent run
  generated runtime/bundle bookkeeping and downloads only

8fe6fec Fix router validation wrapper
  ai-chromium/router-server/run_tests.sh
  score.md

d0041c2 Plan router validation wrapper fix
  IMPLEMENTATION_PLAN.md only

107eb86 Observe ai repository evidence
  score.md only

2f261eb Evaluate ai repository scorecard
  score.md only

07ad58b starting agent run
  generated bundle bookkeeping only
```

Judgment: recent source-bearing work is concentrated in the nested router validation wrapper. Most recent top-level commits are scorecard or generated artifact bookkeeping, not core runtime implementation changes.

## Build and Test Metadata

Root package metadata:

```text
Cargo.toml package.name = ai
Cargo.toml package.version = 0.1.0
Cargo.toml package.edition = 2024
Cargo.toml lib.path = src/lib.rs
Cargo.toml bin.path = src/main.rs
Cargo.toml dependency_count = 0
```

Cargo configuration:

```text
rustc-wrapper = /workspace/ai_sandbox/canon-rustc-v2/target/debug/canon-rustc-v2
rustflags include -Dwarnings, -Dunused, -Ddead-code,
  -Dclippy::allow_attributes, -Dclippy::allow_attributes_without_reason
RUST_BACKTRACE = full
```

Observed toolchain state from `scripts/observe_validation.sh`:

```text
cargo_available = false
rustc_available = false
python3_available = true
rustc_wrapper_configured = true
rustc_wrapper_path_exists = false
state_graph_present = false
```

Constraint: the repository is configured for strict Rust validation and wrapper graph emission, but this sandbox cannot execute root Rust validation or wrapper telemetry.

## Validation Run Evidence

Commands run in this EXECUTE pass:

```text
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz bash scripts/observe_validation.sh = pass as evidence emitter
git diff --check = pass inside observe report
ai-chromium/router-server/run_tests.sh = pass inside observe report
python3 scripts/write_delta_manifest.py = pass when generating receipt/manifest
```

`target/observe/validation-report.ndjson` now binds the available current-head checks into one summary:

```text
validation_status = partial
validation_command_count = 6
validation_test_count = 15
router_test_count = 15
git_diff_check = pass
router_offline_tests = pass
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
runtime_archive_present = true
runtime_archive_download_total = 17
runtime_archive_log_total = 1895
runtime_archive_conversation_snapshots = 0
missing_signal_count = 11
```

Missing signal flags:

```text
missing_cargo_fmt = true
missing_cargo_test = true
missing_clippy = true
missing_cargo_run_ollama_judgment = true
missing_generated_graph_json = true
missing_rustc_wrapper_telemetry = true
missing_conversation_snapshot = true
missing_external_observation_stream_test = true
missing_external_api_action_test = true
missing_policy_learning_replay_trace = true
missing_semantic_artifact_verification_test = true
missing_runtime_download_history = false
missing_artifact_apply_worktree = false
```

Judgment: the observation harness now records available checks, unavailable checks, and router test counts in one receipt surface. It still does not prove root Rust build correctness.

## Delta Receipt Tooling Evidence

Implemented source surfaces:

```text
scripts/observe_validation.sh = compact schema-v2 validation emitter
scripts/write_delta_manifest.py = receipt + manifest writer from one validation report
```

The manifest generator records:

```text
base_commit
head_commit
changed_files
validation_status
validation_commands
validation_command_count
validation_test_count
missing_signal_flags
report_sha256
bundle_sha256
receiver_apply_commands
```

Judgment: this closes the specific null validation receipt gap for generated deltas. It does not close the missing root toolchain, wrapper telemetry, graph, live Ollama, or live CDP gaps.

## Uploaded Runtime Archive Evidence

Primary archive inspected:

```text
archive = /mnt/data/ai-runtime.tar.gz
members = 22
manifest_baseCommit = d47aaa7d3487521df1d0b5d2ee11310934e9a359
manifest_repo = /workspace/ai_sandbox/canon-mini-agent/prototype/ai
manifest_included = 21
manifest_excluded = 773
excluded_apply_worktree = 762
excluded_generated_bundle = 6
excluded_generated_runtime_archive = 1
excluded_secret_token_cache = 1
excluded_signed_url_cache = 3
integrity_findings = 0
leak_findings = 0
schema_findings = 0
```

Archive logs and cache evidence:

```text
candidate_ledger_records = 9
selected_candidate_records = 9
download_records = 17
message_snapshot_records = 921
audit_records = 36
process_log_records = 143
network_request_records = 786
bad_candidate_entries = 7
bad_candidate_reason = missing-file-extension
conversation_snapshots = 0
```

Runtime archive process signals:

```text
audit_top_events = candidate_ledger_written:15, loop_iteration_observed_audit_linked:9,
  runtime_archive_created:4, delta_pair_verified:3, delta_applied:3
process_log_top_events = turn_wait:79, turn_start:12, turn_complete:12,
  background_download_complete:12, loop_iteration_observed:9
network_turn_1 = GET:394, POST:113
network_turn_2 = GET:4, POST:92
network_turn_3 = POST:75
network_turn_4 = GET:2, POST:106
```

Runtime download history:

```text
classification = stale_advisory
currentBaseCommit = d47aaa7d3487521df1d0b5d2ee11310934e9a359
manifestBaseCommit = 07ad58b4bf0e41e087a4584ecc97cd778a416a29
manifestHeadCommit = 8fe6feca9a50fe7a97c004ad333b995da0c784f2
reason = base_commit_mismatch
```

Judgment: the AI runtime archive is useful operational evidence, but it predates the current scorecard-only HEAD and its downloaded manifest history is stale advisory evidence rather than current authority.

## Delta Receipt Evidence

Receipts in `/mnt/data/ai-runtime.tar.gz`:

```text
receipt = 8fe6feca9a50fe7a97c004ad333b995da0c784f2.json
baseCommit = 07ad58b4bf0e41e087a4584ecc97cd778a416a29
headCommit = 8fe6feca9a50fe7a97c004ad333b995da0c784f2
verified = true
merged = true
changed_files = 3
validation_status = null
validation_command_count = null
validation_test_count = null

receipt = 9a23cea59f874bd988cf75eee0210fb30ccb99b7.json
baseCommit = 12f02b6a0f3936ab5ffd8ff59762b71084c443fb
headCommit = 9a23cea59f874bd988cf75eee0210fb30ccb99b7
verified = true
merged = true
changed_files = 18
validation_status = null
validation_command_count = null
validation_test_count = null

receipt = bac66f75055cb5238f38d8888110ea2a06ec224c.json
baseCommit = b9830281da5618db55c12371ec1f17b3abdd0b00
headCommit = bac66f75055cb5238f38d8888110ea2a06ec224c
verified = true
merged = true
changed_files = 21
validation_status = null
validation_command_count = null
validation_test_count = null
```

Judgment: older merge proof exists but older receipts remain weak because their validation fields are null. The current execute path now generates a non-null validation receipt for the final delta artifact.

## Adjacent Uploaded Runtime Archives

```text
router-server-runtime.tar.gz:
  members = 3
  repo = /workspace/ai_sandbox/canon-mini-agent/prototype/ai/ai-chromium/router-server
  baseCommit = 01ec13fce5482e2d53d4097ec7a65d74fe19c11f
  audit_records = 1
  process_log_records = 1
  receipts = 0
  integrity/leak/schema_findings = 0/0/0

chatgpt-project-agent-runtime.tar.gz:
  members = 17
  repo = /workspace/ai_sandbox/canon-mini-agent/prototype/chatgpt-project-agent
  baseCommit = c3152468e3a623f3499b75c4603ac737e83cc5ab
  download_history = stale_advisory:1
  message_snapshot_records = 773
  audit_records = 22
  process_log_records = 153
  network_request_records = 612
  receipts = 2
  integrity/leak/schema_findings = 0/0/0
```

Judgment: adjacent archives confirm the broader automation harness existed, but they are not authoritative proof for the restored `ai` root repository.

## Nested Router-Server Evidence

Path inspected:

```text
ai-chromium/router-server
```

Current offline validation:

```text
node src/tools/check-syntax.mjs = pass, syntax_ok files=49
node --test --test-reporter=dot \
  test/openai-contract.test.mjs \
  test/mock-cdp-integration.test.mjs \
  test/artifact-quality.test.mjs = pass, 15/15 dots
./run_tests.sh = pass
live_cdp_test = not run; gated by RUN_LIVE_TESTS=1 or LIVE_ROUTER_URL
package.json_present = false
```

Judgment: router offline validation is script-backed and avoids the prior missing-`package.json` npm failure. Live CDP remains opt-in and unproven here.

## Risk Register

| Risk | Severity | Evidence | Required closure |
|---|---:|---|---|
| Root Rust validation unavailable | High | `cargo` and `rustc` absent | Provide portable toolchain or committed validation receipt with command hashes |
| Configured wrapper missing | High | wrapper path does not exist | Make wrapper optional for sandbox validation or include/repoint it |
| No graph telemetry | High | no `state/rustc/*/graph.json` | Regenerate graph and record node/edge/intent metrics |
| Weak AI delta validation receipts | High | receipt validation fields are null | Bind concrete validation commands and outputs to receipts |
| Runtime manifest history stale | Medium | archive download history has `base_commit_mismatch` | Tie downloaded manifests to current base/head |
| Runtime archive not current HEAD | Low | archive base is `d47aaa7`, observed HEAD is `f81b2d9` | Re-archive after scorecard update if needed |
| Live router path unvalidated | Medium | offline `./run_tests.sh` passes; live CDP not run | Run `RUN_LIVE_TESTS=1 ./run_tests.sh` against reachable CDP |
| Live Ollama path unproven here | Medium | skipped env + missing cargo | Run and archive current-head receipt/proof output |
| Learning loop not proven end-to-end | High | no durable observation→eval→learning trace observed | Add one replayable policy-promotion integration trace |
| Production unwrap surface unclassified | Medium | 316 `.unwrap()` and 9 `.expect()` calls | Classify test-only vs production path |
| Root operator docs incomplete | Medium | no root `README.md` observed | Add restore/build/test/wrapper/archive instructions |

## Module Scorecard

| Area | Score | Evidence | Constraint |
|---|---:|---|---|
| Kernel | 8.4 | frozen-layer intent, `forbid(unsafe_code)`, typed gates/phases/evidence | no formal proof artifact observed |
| Codec / TLog | 7.8 | NDJSON codec, durable runtime, command ledger exports | root tests unavailable |
| Replay / verification | 7.9 | replay, semantic diff, proof-record APIs, receipt checks, current delta receipt tooling | current-head Rust execution not reproduced |
| Capability layer | 6.1 | all named capability modules present | external objective loop not proven |
| Build/test reproducibility | 3.8 | router wrapper passes offline checks; observe harness emits command/test counts | cannot validate root crate here |
| Runtime evidence | 7.0 | 22-member AI archive, 1895 log lines, 17 downloads, current delta receipt generator | older runtime receipts still have null validation fields |
| Nested router-server | 7.1 | `./run_tests.sh` passes 49-file syntax and 15/15 offline Node tests | live CDP not reproduced |
| Documentation alignment | 7.3 | GOAL, scorecard, plan, and manifest schema now align | GOAL claims exceed current executable proof |

## Missing Validation Signals

```text
required_before_higher_score = [
  cargo_fmt_check,
  cargo_test_all_targets,
  cargo_clippy_all_targets,
  root_binary_run,
  ollama_judgment_example_run,
  state_rustc_graph_json,
  rustc_wrapper_telemetry,
  current_head_runtime_manifest_without_stale_download_history,
  policy_learning_replay_trace,
  external_observation_stream_test,
  external_api_action_test,
  semantic_artifact_verification_trace,
  live_cdp_router_test
]
```

## Highest-Leverage Next Work

1. Restore a Rust toolchain and either provide the configured wrapper or make wrapper use optional for portable validation.
2. Run `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
3. Regenerate `state/rustc/ai/graph.json` and capture wrapper telemetry.
4. Run the router live CDP gate with `RUN_LIVE_TESTS=1` or `LIVE_ROUTER_URL` against an available browser/router.
5. Capture one current-head trace: observation ingress → command → effect receipt → semantic verification → eval → learning/policy promotion.
6. Add root `README.md` with restore, wrapper override, build, test, graph, and runtime archive interpretation instructions.

## Verdict

```text
classification = serious_deterministic_runtime_prototype
not_yet = reproducibly_validated_autonomous_agent
main_blocker = missing_root_rust_validation + missing_wrapper + absent_graph_telemetry + missing_live_runtime_replay
score_confidence = medium_static_low_runtime
```
