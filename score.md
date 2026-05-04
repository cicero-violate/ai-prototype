# Canon Agent Scorecard

## Variables

```text
K = kernel determinism
C = codec / TLog durability
V = replay / verification proof
P = capability + policy maturity
B = build / test reproducibility
E = runtime archive evidence
N = nested router-server evidence
D = documentation / delta discipline
S = overall score
GOOD = strongest current axis
```

## Equations

```text
S = (K · C · V · P · B · E · N · D)^(1/8)
GOOD = max(K,C,V,P,B,E,N,D)
```

One-line explanation: the score is bounded by reproduced validation, not by
architectural claims or prior-run assertions.

## Score Summary

```text
K = 8.4 / 10
C = 7.8 / 10
V = 7.6 / 10
P = 6.2 / 10
B = 4.2 / 10
E = 7.5 / 10
N = 7.4 / 10
D = 7.1 / 10

S = 6.90 / 10
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10 = good
```

## Scope

```text
stage = EXECUTE_TURN_004_PRECOMMIT_EVIDENCE
restored_bundle = /mnt/data/ai.bundle
restored_bundle_head = 44945bf71389366f1566abf48f70a81b924fab96
observed_repo_head_before_update = a813e4af63eade054a7033c5b8689dd4aab2698e
observed_branch = main
repo_path = /mnt/data/ai-eval-repo
runtime_archive = /mnt/data/ai-runtime.tar.gz
source_runtime_changed = false
scorecard_updated = true
```

Judgment: serious deterministic-runtime prototype; not yet a reproducibly
validated autonomous agent.

## GOAL.md Alignment

`GOAL.md` targets a frozen deterministic kernel, append-only TLog, typed
capability layer, bounded recovery, policy learning, semantic verification, and
LLM promotion. The source topology matches that intent at a static level:

```text
src/kernel
src/codec
src/runtime
src/api
src/capability/{context,eval,judgment,learning,llm,memory,observation,orchestration,planning,policy,tooling,verification}
```

Static evidence from the restored repository:

```text
forbid_unsafe_code = src/lib.rs + src/main.rs
dependency_count = 0
rust_files_src_examples = 53
rust_loc_src_examples = 15803
rust_tests_declared = 103
router_mjs_files = 51
unwrap_calls = 316
expect_calls = 9
unsafe_code_sites = 0
```

Constraint: the GOAL.md claims around root Rust validation, graph telemetry,
local Ollama, policy learning, and live replay are not fully reproduced at the
current observed head in this sandbox.

## Current Validation Evidence

Executed after adding the portable validation launcher:

```bash
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz \
CANON_OBSERVE_REPORT=target/observe/validation-report-execute-precommit.ndjson \
bash scripts/observe_validation.sh
```

Observed report summary:

```text
validation_status = partial
validation_command_count = 6
validation_test_count = 20
router_test_count = 20
git_diff_check = pass
router_offline_tests = pass
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
cargo_available = false
rustc_available = false
toolchain_path_added = false
rust_toolchain_source = missing
wrapper_override_required = true
wrapper_override_used = false
missing_signal_count = 12
```

Required commands passed only for repository cleanliness and nested router
offline coverage. Root Rust validation still did not run because neither PATH nor
`/mnt/data/rust-sandbox/bin` exposed cargo/rustc in this sandbox.

## Build Metadata Constraints And Launcher Improvement

```text
Cargo.toml.package = ai 0.1.0 edition 2024
Cargo.toml.dependencies = none
.cargo/config.toml.rustc_wrapper = /workspace/ai_sandbox/canon-rustc-v2/target/debug/canon-rustc-v2
rustc_wrapper_path_exists = false
rustflags_include = -Dwarnings, -Dunused, -Ddead-code, -Dclippy::allow_attributes_without_reason
state_rustc_graph_json_present = false
portable_toolchain_detection = implemented
absent_wrapper_override_env = RUSTC_WRAPPER + RUSTC_WORKSPACE_WRAPPER
router_test_force_exit = implemented
```

The strict build configuration is positive, but it is non-portable in this
sandbox because the configured wrapper is absent. The new launcher can prepend
`/mnt/data/rust-sandbox/bin` and disable the absent wrapper for root Rust checks,
but that path was not present, so this is an implemented recovery surface rather
than reproduced Rust proof. The missing wrapper still blocks graph telemetry.
The nested router test runner now uses Node's force-exit test option so completed
offline tests cannot leave validation hanging on stray handles.

## Uploaded Runtime Archive Evidence

Archive parsed successfully:

```text
archive = /mnt/data/ai-runtime.tar.gz
runtime_archive_member_count = 31
runtime_archive_included_payloads = 30
runtime_archive_parse_status = pass
runtime_archive_log_total = 3342
runtime_archive_download_total = 30
runtime_archive_conversation_snapshots = 0
runtime_archive_cache_files_included = 0
```

Runtime log/cache/download history:

```text
messages_ndjson_rows = 1616
candidate_ledger_rows = 16
downloads_ndjson_rows = 30
audit_ndjson_rows = 60
chatgpt_project_agent_log_rows = 265
network_request_rows = 1376
delta_apply_receipts = 5
delta_apply_receipts_verified = 5
live_cdp_evidence_summary_events = 4
process_turn_start_events = 20
process_turn_complete_events = 20
process_background_download_complete_events = 20
process_final_conversation_download_events = 5
```

Sanitization evidence from `RUNTIME_MANIFEST.json`:

```text
leak_scan_findings = 0
schema_scan_findings = 0
integrity_scan_findings = 0
excluded_apply_worktrees = 1276
excluded_generated_bundles = 10
excluded_secret_token_cache = 1
excluded_signed_url_cache = 5
```

Important constraint: the archive proves automation-loop activity, download
history, redacted runtime logging, and five verified delta-apply receipts. It
does not prove the root Rust crate builds or that the observed head can execute
the full autonomous objective loop.

## Download History Risk

The runtime manifest contains one retained download-history advisory:

```text
downloadHistory[0].classification = stale_advisory
downloadHistory[0].reason = base_commit_mismatch
downloadHistory[0].currentBaseCommit = 44945bf71389366f1566abf48f70a81b924fab96
downloadHistory[0].manifestBaseCommit = da1ebc2656e5d48631b07ef830c7dbec2033739b
downloadHistory[0].manifestHeadCommit = 8b8fb6cb424d7d26f1ddd085c6bbdc30f219638d
```

This is useful evidence of stale-artifact detection, but it is also a real risk:
downloaded manifests can be valid artifacts for another base and must not be
trusted without base/head checks.

## Git Evidence

```text
bundle_verify = pass
bundle_ref_count = 9
bundle_head = 44945bf71389366f1566abf48f70a81b924fab96
observed_head = ed0e480ba6b4fcc13a2fddf7705fe47de4e9bf9d
working_tree_before_score_update = clean
```

Recent history shows multiple agent-run staging commits and accepted deltas:

```text
ed0e480 Evaluate ai repository scorecard
44945bf ready for agent run
96c7efb ready for agent run
8b8fb6c Close delta evidence workflow
da1ebc2 Apply agent worktree delta
b57d8fa ready for agent run
d47aaa7 ready for agent run
ab4cd5b starting agent run
8fe6fec Fix router validation wrapper
d0041c2 Plan router validation wrapper fix
107eb86 Observe ai repository evidence
2f261eb Evaluate ai repository scorecard
```

The uploaded runtime archive contains five verified delta-apply receipts for
heads `54cf9b3`, `8b8fb6c`, `8fe6fec`, `9a23cea`, and `bac66f7`. Receipt
validation was limited to bundle verify/fetch/fast-forward merge and did not
provide root Rust unit-test evidence.

## Risk Register

| Risk | Severity | Evidence | Required closure |
|---|---:|---|---|
| Root Rust validation unavailable | High | `cargo` and `rustc` absent from PATH and sandbox path | Provide toolchain and rerun fmt/test/clippy |
| Configured wrapper missing | High | `.cargo/config.toml` points to absent `/workspace/.../canon-rustc-v2`; override path implemented but unused | Include wrapper or run with cargo available |
| Graph telemetry absent | High | no `state/rustc/*/graph.json` | Regenerate graph and record node/edge/intent metrics |
| Current-head Ollama path unproven | Medium | env missing; example skipped | Run `examples/ollama_judgment.rs` with local endpoint |
| Policy learning trace missing | High | no current-head observation→eval→learning replay | Add one integration trace |
| Semantic artifact verification missing | High | no external artifact proof trace | Bind artifact checks to receipts |
| Stale download artifact hazard | Medium | runtime manifest records `base_commit_mismatch` stale advisory | Keep base/head verification mandatory |
| Production unwrap surface unclassified | Medium | 316 unwrap + 9 expect calls | Classify test-only vs runtime paths |
| Runtime archive lacks conversation snapshots | Medium | `.conversation.json` count is 0 | Include redacted conversation snapshots or explain omission |

## Module Scorecard

| Area | Score | Evidence | Constraint |
|---|---:|---|---|
| Kernel | 8.4 | frozen-layer design, typed gates, `#![forbid(unsafe_code)]` | no formal proof artifact |
| Codec / TLog | 7.8 | NDJSON codec and durable replay APIs present | root tests unavailable |
| Replay / verification | 7.6 | transition/proof APIs and runtime receipts present | current-head Rust replay not reproduced |
| Capability layer | 6.2 | capability modules span context/eval/judgment/learning/llm/memory/observation/orchestration/planning/policy/tooling/verification | full objective loop not proven |
| Build/test reproducibility | 4.2 | git diff check and router offline tests pass; portable cargo detection and wrapper override implemented | root Rust validation unavailable |
| Runtime archive evidence | 7.5 | 31 members, 3342 log rows, 30 download records, 5 verified delta receipts | no conversation snapshots; root crate unproven |
| Nested router-server | 7.4 | 51 syntax files and 20 offline tests pass; runner exits deterministically | live CDP not rerun at current head |
| Documentation / delta discipline | 7.1 | GOAL, README, implementation plan, scorecard, manifest discipline, stale advisory detection, validation receipt fields expanded | claims still exceed reproduced proof |

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
  policy_learning_replay_trace,
  external_observation_stream_test,
  external_api_action_test,
  semantic_artifact_verification_trace,
  redacted_conversation_snapshot,
  live_cdp_router_test_at_current_head
]
```

## Verdict

```text
classification = serious_deterministic_runtime_prototype
not_yet = reproducibly_validated_autonomous_agent
main_blocker = missing_root_rust_validation + missing_wrapper + absent_graph_telemetry + missing_current_head_live_replay
score_confidence = medium_static_low_runtime
```

The next score increase must come from executable proof: portable Rust toolchain,
valid wrapper or wrapper override, generated graph telemetry, current-head
Ollama/example replay, and one complete observation→policy/eval learning trace.
