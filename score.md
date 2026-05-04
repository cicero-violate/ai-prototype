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

One-line explanation: the geometric score keeps the repository anchored to its weakest proof surface, not its strongest architectural claim.

## Score Summary

```text
K = 8.4 / 10
C = 7.7 / 10
V = 7.8 / 10
P = 6.0 / 10
B = 3.1 / 10
E = 5.7 / 10
N = 6.8 / 10
D = 6.9 / 10

S = 6.31 / 10
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10 = good
```

## Evidence Reviewed

```text
uploaded_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/ai-restored
uploaded_runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_extract_path = /mnt/data/ai-runtime-extracted
observed_head_before_execute_stage = 7261f1c0cbf3033d431dffb9db9fd11446dad4cb
base_runtime_manifest_commit = 12f02b6a0f3936ab5ffd8ff59762b71084c443fb
branch = main
goal_md_present = true
readme_present = false
cargo_toml_present = true
cargo_edition = 2024
cargo_dependency_count = 0
rust_source_files_src = 52
rust_source_files_total = 53
rust_test_markers = 105
rust_unwrap_calls = 316
rust_expect_calls = 9
panic_todo_unimplemented_markers = 0
state_rustc_graph_json_present = false
```

Recent history inspected:

```text
19e017c Evaluate ai repository scorecard
12f02b6 ready for agent run
bac66f7 Close router artifact quality gate
e901518 Plan router artifact fixture gate
eae2039 Observe ai repository evidence
8f635e9 Evaluate ai repository scorecard
b983028 ready for agent run
b16db0a Add observe command output evidence paths
653bb6f Plan validation evidence gate
```

Judgment: the repository has accumulated multiple evaluation and observation commits, but the latest committed source state is still not fully reproducible in this sandbox because the Rust toolchain and configured wrapper are unavailable.

## GOAL.md Alignment

`GOAL.md` defines a deterministic self-improving agent runtime with a frozen kernel, append-only TLog, typed capability layer, bounded recovery, learned policy promotion, and an LLM that is progressively demoted from routine reasoning to novelty handling.

The source tree aligns structurally with that goal:

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

Judgment: module topology is a strength. The risk is not conceptual drift; it is that several claims remain validated only by prior text, not by current reproducible local execution.

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

Constraint: the wrapper path is not present in this restored environment. Even if `cargo` existed, validation would require either the wrapper binary or `RUSTC_WORKSPACE_WRAPPER=""` override.

## Reproduced Root Observation Harness

Command run:

```text
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz bash scripts/observe_validation.sh
```

Observed output:

```text
report = target/observe/validation-report.ndjson
report_lines = 10
git_head = 19e017cd1dad4295a4f7b67b82b60dc7361e9a7d
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

Judgment: the observe harness is useful because it converts missing proof into explicit records. It is not a substitute for `cargo fmt`, `cargo test`, `cargo clippy`, graph regeneration, or a live Ollama judgment run.

## Uploaded Runtime Archive Evidence

Archive listing evidence:

```text
archive_members = 13
included_manifest_files = 12
excluded_manifest_files = 258
download_history_count_in_manifest = 1
download_history_by_classification = { stale_advisory: 1 }
download_ndjson_records = 6
message_snapshot_records = 314
candidate_ledger_records = 3
audit_records = 11
process_log_records = 55
network_request_records_total = 251
runtime_archive_log_total = 640
conversation_snapshot_count = 0
cache_file_count = 0
bad_candidate_count = 1
bad_candidate_reason = missing-file-extension
```

Runtime audit event counts:

```text
candidate_ledger_written = 5
loop_iteration_observed_audit_linked = 3
runtime_archive_created = 1
delta_pair_verified = 1
delta_applied = 1
```

Runtime process-log event counts:

```text
startup = 1
turn_start = 4
turn_complete = 4
turn_wait = 35
background_download_complete = 4
loop_iteration_observed = 3
turn_signal = 2
final_conversation_download = 1
delta_applied = 1
```

Delta-apply receipt evidence:

```text
receipt = .repo-agent-runtime/delta-apply-receipts/bac66f75055cb5238f38d8888110ea2a06ec224c.json
decision = accepted
merged = true
baseCommit = b9830281da5618db55c12371ec1f17b3abdd0b00
headCommit = bac66f75055cb5238f38d8888110ea2a06ec224c
changed_files = 21
validation_status = pass
test_count = 0
validation_command = []
```

Critical constraint: the runtime archive is stronger than a plain transcript because it has audit, message, network, download, and receipt surfaces. It is still advisory for the current restored head because the manifest classifies the archived `DELTA_MANIFEST.md` as `stale_advisory` due to `base_commit_mismatch`, and the delta receipt validation had no test command and zero tests.

## Nested Router-Server Validation

Path inspected:

```text
ai-chromium/router-server
```

Commands reproduced:

```text
node --version                                      = v22.16.0
node src/tools/check-syntax.mjs                    = pass, syntax_ok files=49
node --test test/openai-contract.test.mjs          = pass, 7/7
node --test test/mock-cdp-integration.test.mjs     = pass, 3/3
node --test test/artifact-quality.test.mjs         = pass, 5/5
```

Closed failure evidence:

```text
removed stale duplicate fixture turn records:
valid/turn_pass
missing-manifest/turn_missing_manifest
replay-mismatch/turn_replay_mismatch
redaction-fail/turn_redaction_fail
malformed-evidence/turn_malformed

retained canonical fixture turn records:
valid/turn-001
missing-manifest/turn-001
replay-mismatch/turn-001
redaction-fail/turn-001
malformed-evidence/turn-001
```

Judgment: syntax, OpenAI-compatible response shape, deterministic usage estimates, redaction classifier alignment, replay comparison, mocked non-streaming CDP, mocked streaming CDP, unsupported-tool rejection, and artifact-quality fixture validation are now validated offline. This closes the highest-impact validation failure that was implementable in this sandbox.

## Critical Judgment

This repository should be classified as a serious deterministic runtime prototype, not a reproducibly validated autonomous agent.

The strongest evidence is structural: frozen-kernel intent, capability decomposition, no third-party Rust dependencies, strict rustflags, explicit replay/verification modules, and runtime archive capture.

The weakest evidence is still reproducibility: root Rust checks cannot run here, the configured wrapper is absent, no generated graph exists, no live Ollama path was reproduced, and no live CDP path was reproduced. The nested router artifact-quality regression is closed.

## Risk Register

| Risk | Severity | Evidence | Required closure |
|---|---:|---|---|
| Root Rust validation unavailable | High | `cargo` and `rustc` absent in PATH | Provide toolchain or committed validation logs with hashes |
| Configured wrapper missing | High | wrapper path points to absent `/workspace/.../canon-rustc-v2` | Make wrapper optional for portable validation or include/repoint it |
| No generated graph evidence | High | no `state/rustc/*/graph.json` found | Regenerate graph and include node/edge/intent telemetry |
| Artifact-quality regression | Closed | router artifact-quality test passes `5/5` after fixture deduplication | Keep one canonical turn directory per fixture |
| Runtime archive advisory mismatch | Medium | manifest classifies `DELTA_MANIFEST.md` as `stale_advisory` | Archive downloads tied to the current base/head |
| Receipt validation too weak | Medium | delta receipt has `validation_status=pass`, `test_count=0`, `commands=[]` | Bind concrete validation commands and outputs to receipts |
| Live CDP not reproduced | Medium | live CDP test not run | Run with sanitized current-head output artifacts |
| No root README | Medium | `README.md` absent | Add restore/build/test/operator instructions |
| Production unwrap surface unknown | Medium | 316 `.unwrap()` calls across source/examples | Classify test-only vs production-path unwraps |
| Learning loop not proven end-to-end | High | no current observation→action→verification→learning replay trace | Add one durable integration trace |

## Module Scorecard

| Area | Score | Evidence | Constraint |
|---|---:|---|---|
| Kernel | 8.4 | kernel module, deterministic framing, strict crate flags, no unsafe marker found | no formal proof artifact here |
| Codec / TLog | 7.7 | NDJSON codec, writer, durable runtime, command ledger, verify modules | schema compatibility not stress-tested here |
| Runtime / replay / verification | 7.8 | reducer, transition table, recovery policy, diff, verification proof modules | root tests unavailable |
| Capability layer | 6.0 | context/eval/judgment/learning/llm/memory/observation/orchestration/planning/policy/tooling/verification modules exist | external loop maturity not proven |
| Build/test reproducibility | 3.1 | observe harness records unavailable cargo/rustc and missing wrapper; available Node gates pass | no root Rust validation reproduced |
| Runtime evidence | 5.7 | 13 archive members, 640 log lines, 6 download records, audit and receipt records | stale advisory manifest and zero-test receipt validation |
| Nested router-server | 6.8 | syntax, OpenAI contract, mock CDP, and artifact-quality suites pass | live CDP still not reproduced |
| Documentation alignment | 6.9 | detailed `GOAL.md`, implementation plan, scorecard | no root README; claims outrun current proof |

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
missing_router_artifact_quality_fixtures = false
missing_live_cdp_validation = true
```

## Highest-Leverage Next Work

1. Restore a Rust toolchain and either provide the configured wrapper or make wrapper use optional for portable validation.
2. Run `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
3. Regenerate `state/rustc/ai/graph.json` and capture wrapper telemetry.
4. Bind concrete validation commands and outputs into delta-apply receipts instead of allowing zero-command validation passes.
5. Capture one current-head trace: observation ingress → command → effect receipt → semantic verification → eval → learning/policy promotion.
6. Add root `README.md` with restore, build, test, wrapper override, and runtime archive interpretation instructions.

## Verdict

```text
classification = serious_deterministic_runtime_prototype
not_yet = reproducibly_validated_autonomous_agent
main_blocker = missing_root_rust_validation + missing_wrapper + absent_graph_telemetry + missing_live_runtime_trace
score_confidence = medium_static_low_runtime
```