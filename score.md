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

One-line explanation: the geometric score keeps the rating bounded by the weakest proof surface.

## Score Summary

```text
K = 8.4 / 10
C = 7.8 / 10
V = 7.7 / 10
P = 6.1 / 10
B = 3.3 / 10
E = 6.4 / 10
N = 6.9 / 10
D = 7.0 / 10

S = 6.49 / 10
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10 = good
```

## Scope

```text
stage = EXECUTE turn 1
source_changes = ai-chromium/router-server/run_tests.sh
scorecard_change = score.md execution evidence update
restored_repo_path = /mnt/data/work-ai/repo
uploaded_bundle = /mnt/data/ai.bundle
base_commit = 07ad58b4bf0e41e087a4584ecc97cd778a416a29
branch = main
working_tree_before_execution = clean at d0041c2f17785076bbd5dc4190c901239bf95ceb
```

Judgment: this remains a serious deterministic runtime prototype, but current sandbox evidence does not prove root Rust build/test/lint, graph emission, or live Ollama execution.

## GOAL.md Alignment

`GOAL.md` defines a deterministic self-improving agent runtime with a frozen kernel, append-only TLog, typed capability layer, bounded recovery, policy learning, and LLM promotion from routine work to novelty handling.

Observed source topology matches the stated layer model:

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

Positive evidence:

```text
src/lib.rs has #![forbid(unsafe_code)]
Cargo.toml package = ai
Cargo.toml edition = 2024
Cargo.toml dependencies = none
Cargo.lock package set = ai only
src_rust_files = 52
all_rust_files = 53
test_markers_in_src_examples = 104
unsafe_markers_in_src_examples = 0
panic_todo_unimplemented_markers = 0
```

Constraint: the strongest GOAL.md claims cite previous local validation, but that proof was not reproducible in this sandbox at current `HEAD`.

## Recent Git History Inspected

```text
2f261eb Evaluate ai repository scorecard
07ad58b starting agent run
a23ba19 ready for agent run
9a23cea Close router artifact quality gate
7261f1c Plan router artifact quality fix
18db8c1 Observe ai repository evidence
19e017c Evaluate ai repository scorecard
12f02b6 ready for agent run
bac66f7 Close router artifact quality gate
e901518 Plan router artifact fixture gate
```

Recent history shows repeated scorecard/plan turns plus nested router artifact-quality work. The prior `2f261eb` commit changed only `score.md`.

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

Observed toolchain state:

```text
cargo_available = false
rustc_available = false
python3_available = true
rustc_wrapper_configured = true
rustc_wrapper_path_exists = false
```

Constraint: the repository is configured for strict Rust validation, but this sandbox lacks both the Rust toolchain and the configured wrapper path.

## Root Observation Harness

Command run:

```text
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz bash scripts/observe_validation.sh
```

Result summary from `target/observe/validation-report.ndjson`:

```text
git_head = 2f261eb6f63fe4b9634f83bfb5915db4a568d468
git_status_clean = true
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
state_graph_present = false
runtime_archive_present = true
runtime_archive_member_count = 18
runtime_archive_download_total = 13
runtime_archive_log_total = 1266
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

Judgment: the observation harness is valuable because it records absence as evidence. It does not prove build correctness.

## Uploaded Runtime Archive Evidence

Primary archive:

```text
archive = /mnt/data/ai-runtime.tar.gz
members = 18
manifest_included = 17
manifest_excluded = 515
ndjson_log_files = 12
download_records = 13
candidate_ledger_records = 7
message_records = 622
audit_records = 23
process_log_records = 100
network_request_records = 514
bad_candidate_entries = 3
conversation_snapshots = 0
cache_files = 0
runtime_manifest_download_history = 1
runtime_manifest_download_history_classification = stale_advisory
```

Top runtime events:

```text
turn-wait = 58
candidate_ledger_written = 10
turn-start = 8
turn-complete = 8
background-download-complete = 8
loop_iteration_observed_audit_linked = 6
loop_iteration_observed = 6
turn-signal = 4
runtime_archive_created = 2
delta_pair_verified = 2
delta_applied = 2
live_cdp_evidence_summary = 2
```

Download history examples:

```text
repo-delta-004.bundle downloaded via json-download-url
DELTA_MANIFEST.md downloaded via json-download-url
ai-observe-validation-output.txt downloaded via json-download-url
resolved URL cache was refreshed from network, not reused, in sampled records
```

Critical constraint:

```text
currentBaseCommit = 07ad58b4bf0e41e087a4584ecc97cd778a416a29
manifestBaseCommit = 12f02b6a0f3936ab5ffd8ff59762b71084c443fb
classification = stale_advisory
reason = base_commit_mismatch
```

Judgment: the runtime archive proves useful CDP/download activity, but the archived manifest is advisory for this repo state, not authoritative for current `HEAD`.

## Delta Receipt Evidence

Receipts in `/mnt/data/ai-runtime.tar.gz`:

```text
receipt = bac66f75055cb5238f38d8888110ea2a06ec224c.json
baseCommit = b9830281da5618db55c12371ec1f17b3abdd0b00
headCommit = bac66f75055cb5238f38d8888110ea2a06ec224c
verified = true
merged = true
changed_files = 21
commands = git bundle verify; git fetch; git merge --ff-only
validation_status = pass
validation_test_count = 0
validation_commands = []

receipt = 9a23cea59f874bd988cf75eee0210fb30ccb99b7.json
baseCommit = 12f02b6a0f3936ab5ffd8ff59762b71084c443fb
headCommit = 9a23cea59f874bd988cf75eee0210fb30ccb99b7
verified = true
merged = true
changed_files = 18
commands = git bundle verify; git fetch; git merge --ff-only
validation_status = pass
validation_test_count = 0
validation_commands = []
```

Judgment: merge proof exists. Validation proof is weak because both AI delta receipts mark validation success with zero commands and zero tests.

## Additional Uploaded Runtime Archives

`/mnt/data/chatgpt-project-agent-runtime.tar.gz`:

```text
members = 13
manifest_included = 12
manifest_excluded = 66
download_records = 5
candidate_records = 2
message_records = 379
audit_records = 11
network_request_records = 302
receipt_head = c2e90c48afa6286e1cfd9b9635e792fe05721252
receipt_validation = pass
receipt_validation_commands = py_compile + unittest discover
receipt_test_count = 73
```

`/mnt/data/router-server-runtime.tar.gz`:

```text
members = 3
manifest_included = 2
manifest_excluded = 2
download_records = 0
candidate_records = 0
audit_records = 1
process_log_records = 1
```

Judgment: the ChatGPT Project Agent archive contains stronger validation discipline than the AI archive. The router-server archive is too small to carry meaningful validation proof.

## Nested Router-Server Validation

Path inspected:

```text
ai-chromium/router-server
```

Commands reproduced:

```text
node --version = v22.16.0
npm --version = 10.9.2
package.json_present = false
node src/tools/check-syntax.mjs = pass, syntax_ok files=49
node --test test/openai-contract.test.mjs = pass, 7/7
node --test test/mock-cdp-integration.test.mjs = pass, 3/3
node --test test/artifact-quality.test.mjs = pass, 5/5
./run_tests.sh before EXECUTE = fail, npm ENOENT package.json missing
./run_tests.sh after EXECUTE = pass, 15/15 offline tests via dot reporter
```

Judgment: direct offline Node validation is now script-backed. The wrapper no longer depends on absent npm metadata; live CDP remains opt-in and unproven here.

## EXECUTE Turn 1 Evidence

Changed source file:

```text
ai-chromium/router-server/run_tests.sh
```

Implementation:

```text
removed = npm run test:live assumptions
added = direct node syntax check + three offline node:test suites
verbosity_reduction = node --test --test-reporter=dot
live_gate = RUN_LIVE_TESTS=1 or LIVE_ROUTER_URL present
```

Validation commands and results:

```text
node --version = v22.16.0
node --check ai-chromium/router-server/src/server.mjs = pass
(cd ai-chromium/router-server && ./run_tests.sh) = pass, syntax_ok files=49, 15 dots
bash scripts/observe_validation.sh = pass as evidence emitter, root Rust checks unavailable
target/observe/validation-report.ndjson = pass JSON parse, 10 records
git diff --check = pass
```

Remaining constraints:

```text
cargo_fmt_check = unavailable, cargo not found in PATH
cargo_test_all_targets = unavailable, cargo not found in PATH
cargo_clippy_all_targets = unavailable, cargo not found in PATH
rustc_wrapper_path_exists = false
state_graph_present = false
live_cdp_test = not run
ollama_judgment_example = skipped_env_missing
```

## Risk Register

| Risk | Severity | Evidence | Required closure |
|---|---:|---|---|
| Root Rust validation unavailable | High | `cargo` and `rustc` absent | Provide portable toolchain or committed validation receipt with command hashes |
| Configured wrapper missing | High | wrapper path does not exist | Make wrapper optional for sandbox validation or include/repoint it |
| No graph telemetry | High | no `state/rustc/*/graph.json` | Regenerate graph and record node/edge/intent metrics |
| Zero-test AI validation receipts | High | AI receipts have `commands=[]`, `testCount=0` | Bind concrete validation commands and outputs to receipts |
| Runtime manifest is stale advisory | Medium | base commit mismatch in archive download history | Tie archive manifest to current base/head |
| Live router path unvalidated | Medium | offline `./run_tests.sh` passes; live CDP is opt-in and not run here | Run `RUN_LIVE_TESTS=1 ./run_tests.sh` against reachable CDP |
| Live Ollama path unproven here | Medium | skipped env + missing cargo | Run and archive current-head receipt/proof output |
| Learning loop not proven end-to-end | High | no durable observation→eval→learning trace observed | Add one replayable policy-promotion integration trace |
| Production unwrap surface unclassified | Medium | 316 `.unwrap()` and 9 `.expect()` calls | Classify test-only vs production path |
| Root operator docs incomplete | Medium | no root `README.md` observed | Add restore/build/test/wrapper/archive instructions |

## Module Scorecard

| Area | Score | Evidence | Constraint |
|---|---:|---|---|
| Kernel | 8.4 | frozen-layer intent, `forbid(unsafe_code)`, typed gates/phases/evidence | no formal proof artifact observed |
| Codec / TLog | 7.8 | NDJSON codec, durable runtime, command ledger exports | root tests unavailable |
| Replay / verification | 7.7 | replay, semantic diff, proof-record APIs, receipt checks | current-head execution not reproduced |
| Capability layer | 6.1 | all named capability modules present | external objective loop not proven |
| Build/test reproducibility | 3.3 | router wrapper now runs offline checks; cargo/rustc/wrapper unavailable; graph absent | cannot validate root crate here |
| Runtime evidence | 6.4 | 18-member AI archive, 1266 logs, 13 downloads, 2 delta receipts | stale advisory manifest; zero-test receipts |
| Nested router-server | 6.9 | `./run_tests.sh` now passes 49-file syntax and 15/15 offline Node tests | live CDP not reproduced |
| Documentation alignment | 7.0 | plan target implemented and score evidence updated | GOAL claims still exceed current executable proof |

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
  current_head_runtime_archive,
  current_head_delta_receipts_with_nonzero_tests,
  policy_learning_replay_trace,
  external_observation_stream_test,
  external_api_action_test,
  semantic_artifact_verification_trace
]
```

## Highest-Leverage Next Work

1. Restore a Rust toolchain and either provide the configured wrapper or make wrapper use optional for portable validation.
2. Run `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
3. Regenerate `state/rustc/ai/graph.json` and capture wrapper telemetry.
4. Replace zero-command validation receipts with required command outputs, hashes, and test counts.
5. Run the router live CDP gate with `RUN_LIVE_TESTS=1` or `LIVE_ROUTER_URL` against an available browser/router.
6. Capture one current-head trace: observation ingress → command → effect receipt → semantic verification → eval → learning/policy promotion.
7. Add root `README.md` with restore, wrapper override, build, test, graph, and runtime archive interpretation instructions.

## Verdict

```text
classification = serious_deterministic_runtime_prototype
not_yet = reproducibly_validated_autonomous_agent
main_blocker = missing_root_rust_validation + missing_wrapper + absent_graph_telemetry + zero-test_AI_receipts
score_confidence = medium_static_low_runtime
```