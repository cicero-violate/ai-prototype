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
V = 7.8 / 10
P = 6.1 / 10
B = 3.0 / 10
E = 6.3 / 10
N = 6.6 / 10
D = 6.8 / 10

S = 6.35 / 10
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10 = good
```

## Evidence Reviewed

```text
uploaded_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/work-ai/repo
uploaded_runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_extract_path = /mnt/data/ai-runtime-extracted
current_head = 07ad58b4bf0e41e087a4584ecc97cd778a416a29
branch = main
goal_md_present = true
readme_present = false
cargo_toml_present = true
cargo_edition = 2024
cargo_dependency_count = 0
rust_source_files_src = 52
rust_source_files_total = 53
rust_test_markers = 106
rust_unwrap_calls = 316
rust_expect_calls = 9
panic_todo_unimplemented_markers = 0
unsafe_markers = 0
state_rustc_graph_json_present = false
```

Recent history inspected:

```text
07ad58b starting agent run
a23ba19 ready for agent run
9a23cea Close router artifact quality gate
7261f1c Plan router artifact quality fix
18db8c1 Observe ai repository evidence
19e017c Evaluate ai repository scorecard
```

Judgment: this is a serious deterministic runtime prototype with improving artifact discipline, but it is not yet a reproducibly validated autonomous agent in this environment.

## GOAL.md Alignment

`GOAL.md` defines a deterministic self-improving agent runtime: frozen kernel, append-only TLog, typed capability layer, bounded recovery, policy learning, and LLM promotion from routine labor to novelty handling.

The source topology matches the goal:

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

Strength: the architecture is coherent and explicitly layered.

Constraint: the strongest claims still need current-head executable proof: root Rust checks, graph telemetry, live LLM receipts, and one complete learning-policy replay trace.

## Build and Test Metadata

Root package metadata:

```text
package = ai
version = 0.1.0
edition = 2024
library = src/lib.rs
binary = src/main.rs
dependencies = none
```

Cargo configuration evidence:

```text
rustc-wrapper = /workspace/ai_sandbox/canon-rustc-v2/target/debug/canon-rustc-v2
rustflags include -Dwarnings, -Ddead-code, -Dunused, -Dclippy::allow_attributes, -Dclippy::allow_attributes_without_reason
RUST_BACKTRACE = full
```

Constraint: the configured wrapper path does not exist in this restored sandbox. `cargo` and `rustc` are also unavailable, so root Rust validation could not run here.

## Reproduced Root Observation Harness

Command run:

```text
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz bash scripts/observe_validation.sh
```

Observed summary:

```text
git_head = 07ad58b4bf0e41e087a4584ecc97cd778a416a29
git_status_clean_at_start = true
cargo_available = false
rustc_available = false
python3_available = true
rustc_wrapper_configured = true
rustc_wrapper_path_exists = false
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
state_graph_present = false
runtime_archive_present = true
runtime_archive_member_count = 18
runtime_archive_download_total = 13
runtime_archive_log_total = 1266
missing_signal_count = 11
```

Judgment: the observation harness is useful because it turns missing validation into explicit evidence. It does not replace the missing Rust toolchain, wrapper, graph regeneration, or live Ollama run.

## Uploaded Runtime Archive Evidence

Runtime archive evidence:

```text
archive_members = 18
manifest_included = 17
manifest_excluded = 515
download_history_count = 1
download_history_by_classification = { stale_advisory: 1 }
download_ndjson_records = 13
message_snapshot_records = 622
candidate_ledger_records = 7
audit_records = 23
process_log_records = 100
network_request_records_total = 514
conversation_snapshot_count = 0
cache_file_count = 0
bad_candidate_count = 3
integrity_findings = 0
schema_findings = 0
leak_findings = 0
```

Runtime audit event counts:

```text
candidate_ledger_written = 10
loop_iteration_observed_audit_linked = 6
runtime_archive_created = 2
delta_pair_verified = 2
delta_applied = 2
live_cdp_evidence_summary = 1
```

Live CDP evidence summary:

```text
proof_complete = true
evidence_score = 1.0
missing_signals = []
signals = message_sent_to_chatgpt, assistant_turn_completed_from_network_payload,
          repo_delta_bundle_downloaded_from_chatgpt, DELTA_MANIFEST.md_downloaded_from_chatgpt,
          git_bundle_verify_on_downloaded_chatgpt_artifact,
          manifest_base_head_checked_against_local_repo,
          ff_merge_completed_from_live_downloaded_artifact,
          post_apply_validation_receipt_from_live_run,
          end_to_end_runtime_archive_after_live_apply
```

Critical constraint: the live CDP artifact chain is stronger than a transcript, but the downloaded `DELTA_MANIFEST.md` in the archive is still classified `stale_advisory` for the current restored head because its manifest base is `12f02b6a0f3936ab5ffd8ff59762b71084c443fb` while the archive manifest base is `07ad58b4bf0e41e087a4584ecc97cd778a416a29`.

## Delta Receipt Evidence

Two delta-apply receipts were inspected from the runtime archive:

```text
receipt = bac66f75055cb5238f38d8888110ea2a06ec224c.json
decision = accepted
merged = true
changed_files = 21
commands = git bundle verify, git fetch, git merge --ff-only
validation_status = pass
validation_commands = []
test_count = 0

receipt = 9a23cea59f874bd988cf75eee0210fb30ccb99b7.json
decision = accepted
merged = true
changed_files = 18
commands = git bundle verify, git fetch, git merge --ff-only
validation_status = pass
validation_commands = []
test_count = 0
```

Judgment: merge proof exists. Validation proof remains weak because both receipts allow a zero-command, zero-test validation pass.

## Nested Router-Server Validation

Path inspected:

```text
ai-chromium/router-server
```

Direct commands reproduced:

```text
node --version = v22.16.0
node src/tools/check-syntax.mjs = pass, syntax_ok files=49
node --test test/openai-contract.test.mjs = pass, 7/7
node --test test/mock-cdp-integration.test.mjs = pass, 3/3
node --test test/artifact-quality.test.mjs = pass, 5/5
```

Negative evidence:

```text
./run_tests.sh = fail
failure = npm ENOENT, package.json missing
script_lines = npm run test:live; LIVE_ROUTER_URL=http://127.0.0.1:8081 npm run test:live
```

Judgment: offline router checks are real and pass, including artifact-quality fixtures. The wrapper script is currently stale or invalid because it assumes a `package.json` that is not present.

## Critical Judgment

The repository has strong internal structure and a useful proof vocabulary: pure kernel, TLog, reducer, transition table, recovery policy, durable runtime, semantic verification, policy promotion, LLM receipt types, tool receipts, and runtime evidence capture.

The primary blocker is not architecture. The blocker is executable reproducibility. Current evidence cannot prove that the root Rust crate builds, tests, lints, emits graph facts, or completes the local Ollama judgment path at current `HEAD`.

## Risk Register

| Risk | Severity | Evidence | Required closure |
|---|---:|---|---|
| Root Rust validation unavailable | High | `cargo` and `rustc` absent in PATH | Provide portable toolchain path or committed validation log with hashes |
| Configured wrapper missing | High | wrapper path points to absent `/workspace/.../canon-rustc-v2` | Make wrapper optional or include/repoint it |
| No generated graph evidence | High | no `state/rustc/*/graph.json` found | Regenerate graph and capture node/edge/intent telemetry |
| Zero-test validation receipts | High | delta receipts have `commands=[]`, `testCount=0` | Bind concrete validation commands and outputs to receipts |
| Router test wrapper broken | Medium | `./run_tests.sh` fails with missing `package.json` | Replace npm script assumptions with direct node commands or add package metadata |
| Runtime archive advisory mismatch | Medium | `DELTA_MANIFEST.md` classified `stale_advisory` | Archive downloads tied to current base/head |
| No root README | Medium | `README.md` absent | Add restore/build/test/operator instructions |
| Production unwrap surface unknown | Medium | 316 `.unwrap()` calls across source/examples | Classify test-only vs production-path unwraps |
| Live Ollama path not reproduced | Medium | env missing, cargo missing | Run and archive current-head receipt/proof output |
| Learning loop not proven end-to-end | High | no observation→verification→learning trace | Add one durable integration trace |

## Module Scorecard

| Area | Score | Evidence | Constraint |
|---|---:|---|---|
| Kernel | 8.4 | `#![forbid(unsafe_code)]`, pure kernel module, typed phases/gates/evidence, no unsafe markers | no formal proof artifact here |
| Codec / TLog | 7.8 | NDJSON codec, writer, durable runtime, command ledger, verification modules | schema stress validation not reproduced here |
| Runtime / replay / verification | 7.8 | reducer, transition table, recovery policy, semantic diff, proof records | root tests unavailable |
| Capability layer | 6.1 | context/eval/judgment/learning/llm/memory/observation/orchestration/planning/policy/tooling/verification modules exist | full external loop not proven |
| Build/test reproducibility | 3.0 | observe harness records missing cargo/rustc/wrapper; no graph | no root Rust validation reproduced |
| Runtime evidence | 6.3 | 18 archive members, 1266 log lines, 13 downloads, live CDP summary complete | stale advisory manifest and zero-test receipts |
| Nested router-server | 6.6 | syntax, OpenAI contract, mock CDP, artifact-quality tests pass | `run_tests.sh` broken; live test not reproduced |
| Documentation alignment | 6.8 | detailed `GOAL.md`, implementation plan, scorecard | no root README; claims outrun current executable proof |

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
missing_live_cdp_validation = false
```

## Highest-Leverage Next Work

1. Restore a Rust toolchain and either provide the configured wrapper or make wrapper use optional for portable validation.
2. Run `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
3. Regenerate `state/rustc/ai/graph.json` and capture wrapper telemetry.
4. Replace zero-command receipt validation with required command outputs and hashes.
5. Fix `ai-chromium/router-server/run_tests.sh` so it runs the direct Node checks that currently pass.
6. Capture one current-head trace: observation ingress → command → effect receipt → semantic verification → eval → learning/policy promotion.
7. Add root `README.md` with restore, wrapper override, build, test, graph, and runtime archive interpretation instructions.

## Verdict

```text
classification = serious_deterministic_runtime_prototype
not_yet = reproducibly_validated_autonomous_agent
main_blocker = missing_root_rust_validation + missing_wrapper + absent_graph_telemetry + zero-test_receipts
score_confidence = medium_static_low_runtime
```
