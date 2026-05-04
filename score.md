# Canon Agent Scorecard

## Variables

```text
I  = Intelligence          E  = Efficiency        C  = Correctness
A  = Alignment             R  = Robustness         P  = Performance
S  = Scalability           D  = Determinism        T  = Transparency
Co = Collaboration         Em = Empowerment        B  = Benefit
L  = Learning              Si = Simplicity         F  = Future-Proofing
G  = geometric-mean goodness
```

## Equation

```text
G = (I·E·C·A·R·P·S·D·T·Co·Em·B·L·Si·F)^(1/15)
G = 6.66 / 10
max(G) = good
```

Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers
the whole system.

## Score Summary

```text
I  = 7.5 / 10
E  = 6.2 / 10
C  = 5.6 / 10
A  = 7.7 / 10
R  = 6.2 / 10
P  = 5.4 / 10
S  = 5.9 / 10
D  = 7.8 / 10
T  = 8.5 / 10
Co = 6.8 / 10
Em = 7.3 / 10
B  = 7.0 / 10
L  = 6.1 / 10
Si = 5.6 / 10
F  = 7.2 / 10

G = 6.66 / 10
GOOD = max(G) = 6.66 / 10 = good
```

Judgment: this is a serious autonomous-agent prototype with unusually strong
architecture for replay, receipts, deterministic gates, and policy learning.
It is not yet production-grade. The restored repository contains useful runtime
and router evidence, but current root Rust validation, graph telemetry, Ollama
proof replay, and policy-learning replay were not reproduced in this environment.

## Evidence Snapshot

```text
bundle = /mnt/data/ai.bundle
restored_repo = /mnt/data/ai-restored/ai
git_head = ebf09b8ad9d4b850404b2003f49d65c04af0b772
git_status_clean_before_score_update = true
goal_md_present = true
tracked_files = 189
rust_files_src_examples = 53
rust_test_attrs = 103
rust_cfg_test_sections = 2
unsafe_token_count_src_examples = 0
unwrap_call_count_src_examples = 316
expect_call_count_src_examples = 9
root_cargo_toml_dependencies = 0 third-party Rust dependencies
configured_rustc_wrapper_path_exists = false
state_graph_present = false
runtime_archive_present = true
runtime_archive_sha256 = 9439e3b0ca82ffd9f8e743948cd1f8d7fd9334d7e882f79f2db6118afad384f5
runtime_manifest_base_commit = ebf09b8ad9d4b850404b2003f49d65c04af0b772
runtime_archive_member_count = 88
runtime_archive_log_total = 10357
runtime_archive_download_total = 176
runtime_download_history_records = 160
runtime_unique_download_alias_count = 17
runtime_duplicate_artifact_aliases = 8
runtime_stale_advisory_count = 1
runtime_candidate_error_count = 0
runtime_conversation_snapshots = 0
router_offline_tests = pass
router_syntax_checks = 41
router_behavior_tests = 4
router_test_count = 45
git_diff_check = pass
git_delta_diff_check = fail
cargo_available = false
rustc_available = false
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
validation_status = fail
missing_signal_count = 12
```

## Positive Evidence

- `GOAL.md` defines a coherent architecture: frozen kernel, codec/runtime,
  capability layer, append-only TLog, bounded recovery, policy learning, and
  LLM promotion.
- `src/lib.rs` enforces `#![forbid(unsafe_code)]`; static source scan found no
  unsafe tokens in `src` or `examples`.
- The crate has no third-party Rust dependencies in the root `Cargo.toml`, which
  improves auditability and supply-chain control.
- The source surface is broad for a prototype: kernel, codec, runtime, API,
  observation, context, memory, planning, LLM, judgment, tooling, verification,
  eval, policy, learning, and orchestration modules are present.
- Test intent is substantial: static scan found `103` Rust `#[test]` attributes
  and `2` `cfg(test)` sections.
- Router-side offline validation passed: `41` syntax checks plus `4` behavior
  tests, total `45` checks.
- Runtime archive evidence is present and sanitized: `88` members, `10,357`
  log rows observed by the validation script, `176` download rows, and `0`
  candidate errors.
- Runtime archive base binding matches the restored head.

## Critical Risks

- Current validation status is `fail`, not `pass`.
- Root Rust validation was not reproducible here: `cargo` and `rustc` were not
  available on `PATH`, so fmt/test/clippy did not run.
- `.cargo/config.toml` points to a rustc wrapper path that does not exist in this
  restored environment. Wrapper override is required before normal Cargo checks.
- `git diff --check` passes for the current working tree, but base-to-head delta
  whitespace validation fails because historical committed files contain trailing
  whitespace and blank-line-at-EOF issues.
- `state/rustc/*/graph.json` is absent, so graph node/edge counts, intent
  coverage, redundant-path telemetry, and wrapper-derived semantic evidence are
  unavailable.
- `GOAL.md` records historical local successes, including Rust tests and Ollama
  judgment proof replay, but those claims were not reproduced in this stage.
- The runtime archive has message ledgers but `0` full `.conversation.json`
  snapshots.
- Runtime/download lineage is still noisy: duplicate artifact aliases remain,
  including repeated delta bundles and manifests.
- `316` `unwrap()` calls remain in `src` and `examples`, which is a large panic
  surface for a system claiming bounded recovery.
- Tracked runtime/upload/download artifacts remain in repo history, weakening
  simplicity and repository hygiene.
- Policy learning exists as source and tests, but this evaluation did not find a
  fresh replay trace proving promotion from completed TLog history.

## Dimension Rationale

| Dimension | Score | Evidence |
|---|---:|---|
| I | 7.5 | Strong architecture across deterministic kernel, evidence routing, verification, learning, and LLM adapter surfaces. |
| E | 6.2 | Observe script and router checks are compact, but missing root toolchain and wrapper friction slow validation. |
| C | 5.6 | Router checks pass and current diff is clean; root Rust tests/fmt/clippy and graph replay were unavailable, and delta whitespace check fails. |
| A | 7.7 | Repository direction closely matches `GOAL.md`; live autonomous operation remains unproven. |
| R | 6.2 | Receipts, replay, and runtime archive hardening help; missing graph/toolchain/Ollama proof and many unwraps cap robustness. |
| P | 5.4 | Router offline tests are fast, but no root benchmark, throughput, latency, or cargo timing signal was reproduced. |
| S | 5.9 | Layer boundaries and capability decomposition can scale; validation remains single-repo and partially unavailable. |
| D | 7.8 | Frozen-kernel intent, hash-chained logs, bundle/head binding, and deterministic records are strong; absent graph/proof replay weakens confidence. |
| T | 8.5 | Missing signals, runtime archive metrics, and validation command outcomes are explicit and machine-readable. |
| Co | 6.8 | README, GOAL, score, and scripts guide collaboration; tracked generated artifacts and duplicate aliases add noise. |
| Em | 7.3 | Restore, observe, and delta workflows are operational; external toolchain/Ollama/wrapper setup is still required. |
| B | 7.0 | High-potential autonomous-runtime prototype; benefit is capped until validation is portable and reproducible. |
| L | 6.1 | Learning/policy modules and tests exist; no fresh policy-promotion replay trace was reproduced. |
| Si | 5.6 | Dependency-minimal Rust helps; repo history clutter, generated artifacts, wrapper coupling, and unwrap surface hurt simplicity. |
| F | 7.2 | Canonical normal-form direction and sanitized runtime archives improve future compatibility; missing portable proof pipeline remains the blocker. |

## Missing Validation Signals

```text
missing_root_rust_toolchain = true
missing_cargo_fmt = true
missing_cargo_test = true
missing_clippy = true
missing_cargo_run_ollama_judgment = true
missing_generated_graph_json = true
missing_rustc_wrapper_telemetry = true
missing_conversation_snapshot = true
missing_runtime_download_history = false
missing_external_observation_stream_test = true
missing_external_api_action_test = true
missing_semantic_artifact_verification_test = true
missing_policy_learning_replay_trace = true
missing_artifact_apply_worktree = false
```

## Required Next Work

```text
1. Restore a portable Rust toolchain and run fmt/test/clippy with wrapper variables cleared.
2. Restore/build the configured rustc wrapper or remove the absolute wrapper dependency.
3. Regenerate state/rustc/*/graph.json and require wrapper telemetry in validation.
4. Fix committed base-to-head whitespace defects or scope delta whitespace checks to new changes.
5. Run ollama_judgment against a local endpoint and verify receipt/proof replay.
6. Add an offline semantic-artifact verification test and a policy-learning replay test.
7. Classify or replace high-risk unwrap/expect sites with typed error handling.
8. Stop tracking runtime/upload/download artifacts in normal repository history.
9. Capture full conversation snapshots or formally declare message ledgers canonical.
```