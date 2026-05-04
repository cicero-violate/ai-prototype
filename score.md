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

One-line explanation: the score is bounded by executable proof, not architectural intent.

## Score Summary

```text
K = 8.4 / 10
C = 7.8 / 10
V = 7.8 / 10
P = 6.2 / 10
B = 4.1 / 10
E = 7.4 / 10
N = 7.4 / 10
D = 7.3 / 10

S = 6.91 / 10
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10 = good
```

## Scope

```text
stage = EVAL
restored_head = 44945bf71389366f1566abf48f70a81b924fab96
restored_branch = main
uploaded_bundle = /mnt/data/ai.bundle
repo_path = /mnt/data/ai-eval-repo
source_runtime_changed = false
scorecard_updated = true
```

Judgment: serious deterministic-runtime prototype; not yet a reproducibly validated autonomous agent.

## GOAL.md Alignment

`GOAL.md` targets a frozen deterministic kernel, append-only TLog, typed capability layer, bounded recovery, policy learning, semantic verification, and LLM promotion.

Observed topology matches the target:

```text
src/kernel
src/codec
src/runtime
src/api
src/capability/{context,eval,judgment,learning,llm,memory,observation,orchestration,planning,policy,tooling,verification}
```

Static evidence:

```text
forbid_unsafe_code = src/lib.rs + src/main.rs
dependency_count = 0
rust_files_src_examples = 53
rust_loc_src_examples = 15803
rust_tests_declared = 103
public_items_observed = 1164
unwrap_calls = 316
expect_calls = 9
unsafe_code_sites = 0
```

Constraint: the GOAL.md claims around root Rust validation, graph telemetry, local Ollama, policy learning, and live replay are not fully reproduced in this sandbox.

## Validation Evidence

Executed current-head observation:

```bash
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz bash scripts/observe_validation.sh
```

Observed report summary:

```text
validation_status = partial
validation_command_count = 6
validation_test_count = 20
router_test_count = 20
git_diff_check = pass
router_offline_tests = pass
router_syntax_files = 51
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
missing_signal_count = 11
```

The validation status must remain `partial` until root Rust, configured wrapper graph telemetry, local Ollama, policy learning, semantic artifact verification, external observation/API effects, and live CDP are proven at the current head.

## Uploaded Runtime Archive Evidence

```text
archive = /mnt/data/ai-runtime.tar.gz
runtime_archive_member_count = 31
runtime_archive_log_total = 3342
runtime_archive_download_total = 30
runtime_archive_conversation_snapshots = 0
runtime_archive_parse_status = pass
```

Judgment: the archive proves automation-loop activity and downloadable artifact history. It does not prove the restored root Rust crate builds or executes.

## Git Evidence

```text
bundle_verify = pass
bundle_refs = 9
bundle_head = 44945bf71389366f1566abf48f70a81b924fab96
working_tree_before_score_update = clean
```

Recent restored history:

```text
44945bf ready for agent run
96c7efb ready for agent run
8b8fb6c Close delta evidence workflow
da1ebc2 Apply agent worktree delta
b57d8fa ready for agent run
d47aaa7 ready for agent run
ab4cd5b starting agent run
8fe6fec Fix router validation wrapper
```

## Risk Register

| Risk | Severity | Evidence | Required closure |
|---|---:|---|---|
| Root Rust validation unavailable | High | `cargo` and `rustc` absent from sandbox PATH | Provide toolchain and rerun fmt/test/clippy |
| Configured wrapper missing | High | `.cargo/config.toml` points to absent `/workspace/.../canon-rustc-v2` | Include wrapper or document portable override |
| Graph telemetry absent | High | no `state/rustc/*/graph.json` | Regenerate graph and record node/edge/intent metrics |
| Live Ollama unproven | Medium | env missing; example skipped | Run `examples/ollama_judgment.rs` and verify receipt |
| Policy learning trace missing | High | no current-head observation→eval→learning replay | Add one integration trace |
| Semantic artifact verification missing | High | no external artifact proof trace | Bind artifact checks to receipts |
| Live CDP unproven | Medium | live test gated off | Run against reachable browser/router |
| Production unwrap surface unclassified | Medium | 316 unwrap + 9 expect calls | Classify test-only vs runtime paths |

## Module Scorecard

| Area | Score | Evidence | Constraint |
|---|---:|---|---|
| Kernel | 8.4 | frozen-layer design, typed gates, `#![forbid(unsafe_code)]` | no formal proof artifact |
| Codec / TLog | 7.8 | NDJSON codec and durable replay APIs present | root tests unavailable |
| Replay / verification | 7.8 | transition validation and proof APIs present | Rust execution not reproduced |
| Capability layer | 6.2 | capability modules span context/eval/judgment/learning/llm/memory/observation/orchestration/planning/policy/tooling/verification | full objective loop not proven |
| Build/test reproducibility | 4.1 | git diff check and router offline tests pass | root Rust validation unavailable |
| Runtime archive evidence | 7.4 | 31 archive files, 3342 log lines, 30 download records | no conversation snapshots |
| Nested router-server | 7.4 | 51 syntax files and 20 offline tests pass | live CDP missing |
| Documentation alignment | 7.3 | GOAL, README, implementation plan, and scorecard exist | claims still exceed local proof |

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
  live_cdp_router_test
]
```

## Verdict

```text
classification = serious_deterministic_runtime_prototype
not_yet = reproducibly_validated_autonomous_agent
main_blocker = missing_root_rust_validation + missing_wrapper + absent_graph_telemetry + missing_live_runtime_replay
score_confidence = medium_static_low_runtime
```