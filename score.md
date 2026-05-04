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
G = 6.89 / 10
max(G) = good
```

Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers
the whole system.

## Score Summary

```text
I  = 7.6 / 10
E  = 6.5 / 10
C  = 5.9 / 10
A  = 7.8 / 10
R  = 6.5 / 10
P  = 5.5 / 10
S  = 6.3 / 10
D  = 7.9 / 10
T  = 8.8 / 10
Co = 6.9 / 10
Em = 7.6 / 10
B  = 7.0 / 10
L  = 6.3 / 10
Si = 6.0 / 10
F  = 7.5 / 10

G = 6.89 / 10
GOOD = max(G) = 6.89 / 10 = good
```

Judgment: this is a serious autonomous-agent prototype with strong deterministic
architecture, replay concepts, validation receipts, runtime evidence capture,
and policy-learning direction. It is not production-grade. Current validation is
partial: Python and router checks pass, but root Rust fmt/test/clippy, graph
telemetry, Ollama proof replay, semantic artifact verification, and policy
promotion replay are still missing.

## Evidence Snapshot

```text
bundle = /mnt/data/ai.bundle
restored_repo = /mnt/data/eval-ai/repo
base_commit = 3d0c73cd5770487808d9b644a384697bc249fcae
execution_head = recorded in /mnt/data/DELTA_MANIFEST.md after EXECUTE turn 004 commit
git_branch = main
recent_history = 3d0c73c starting agent run; 1ca65f8 Harden delta validation receipts; ebf09b8 starting agent run; c8005dd Harden validation delta artifacts
goal_md_present = true
tracked_files = 194 after planned commit
rust_files_src_examples = 53
rust_test_attrs = 103
rust_cfg_test_sections = 2
unsafe_token_count_src_examples = 0
panic_call_count_src_examples = 0
unwrap_call_count_src_examples = 316
expect_call_count_src_examples = 9
root_cargo_toml_dependencies = 0 third-party Rust dependencies
configured_rustc_wrapper_path = none by default
configured_rustc_wrapper_path_exists = false
wrapper_graph_contract = explicit CANON_RUSTC_WRAPPER=/path/to/canon-rustc-v3
state_graph_present = false
runtime_archive_present = true
runtime_archive_sha256 = b2578606919ef545e5bda7a87ddef2b362d5913ba1f796a9e138b4f17ecbbae5
runtime_manifest_base_commit = 3d0c73cd5770487808d9b644a384697bc249fcae
runtime_archive_member_count = 94
runtime_manifest_included_files = 93
runtime_manifest_excluded_files = 3412
runtime_manifest_excluded_reasons = apply-worktree:3367, generated-bundle:26, signed-url-cache:17, secret-token-cache:1, generated-runtime-archive:1
runtime_archive_ndjson_count = 61
runtime_archive_json_count = 21
runtime_archive_log_total_observe_script = 10979
runtime_archive_download_total_observe_script = 202
runtime_download_history_records = 174
runtime_download_history_by_classification = delta_applied:16, download_event:25, download_ledger:117, live_evidence:15, stale_advisory:1
runtime_message_rows_python_scan = 4949
runtime_candidate_rows_python_scan = 81
runtime_candidate_error_count_python_scan = 0
runtime_unique_download_alias_count = 19
runtime_duplicate_artifact_aliases = 8
runtime_stale_advisory_count = 1
runtime_conversation_snapshots = 0
runtime_delta_apply_receipts = 16
runtime_agent_log_rows = 927
runtime_ndjson_parse_errors = 0
runtime_manifest_leak_findings = 0
runtime_manifest_schema_findings = 0
runtime_manifest_integrity_findings = 0
observe_validation_run = pass_with_partial_validation_status
observe_validation_command_count = 9
observe_validation_test_count = 54
observe_validation_summary_present = true
git_diff_check = pass
git_delta_diff_check = pass
python_unit_tests = pass, 9 tests
router_offline_tests = pass, 41 syntax checks + 4 behavior tests = 45
cargo_available = false
rustc_available = false
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
wrapper_graph_validation = skipped_env_missing without CANON_RUSTC_WRAPPER
ollama_judgment_example = skipped_env_missing
validation_status = partial
missing_signal_count = 13
```

## Positive Evidence

- `GOAL.md` defines a coherent architecture: frozen deterministic kernel,
  codec/runtime boundary, capability layer, append-only TLog, bounded recovery,
  policy learning, and LLM promotion.
- `src/lib.rs` and `src/main.rs` enforce `#![forbid(unsafe_code)]`; static scan
  found `0` unsafe tokens and `0` `panic!` calls in `src` and `examples`.
- The root `Cargo.toml` has no third-party Rust dependencies, improving
  auditability and supply-chain control.
- Static Rust test intent is substantial: `103` `#[test]` attributes and `2`
  `cfg(test)` sections were found.
- `scripts/observe_validation.sh` emits a final `validation_summary` with root
  Rust, wrapper graph, Ollama, router, and Python test states separated.
- `python3 -m unittest discover -s tests -p 'test_*.py'` passed `9` tests,
  including static validation-contract tests for wrapper isolation.
- Router offline validation passed: `41` syntax checks plus `4` behavior tests.
- Runtime archive evidence binds to the restored head: `RUNTIME_MANIFEST.json`
  reports base commit `3d0c73cd5770487808d9b644a384697bc249fcae`.
- Runtime archive hygiene is improved: manifest leak, schema, and integrity scans
  report `0` findings; token cache, signed URL cache, generated bundles, runtime
  archives, and apply worktrees were excluded.
- Runtime archive scan found `61` NDJSON files, `174` download-history records,
  `81` candidate rows, `16` delta-apply receipts, and `0` candidate errors.

## Critical Risks

- Current validation status is `partial`, not `pass`.
- Root Rust validation was not reproducible here: `cargo` and `rustc` were not
  available on `PATH`, so fmt/test/clippy did not run.
- `.cargo/config.toml` no longer points to an absolute rustc-wrapper path, but
  root Rust validation still cannot run here because `cargo` and `rustc` are
  unavailable.
- Wrapper graph capture is now explicit through `CANON_RUSTC_WRAPPER`, but no
  wrapper path was provided and no graph telemetry was reproduced.
- `state/rustc/*/graph.json` is absent, so graph node/edge counts, intent
  coverage, redundant-path telemetry, and wrapper-derived semantic evidence are
  unavailable.
- `GOAL.md` records historical local successes, including Rust tests and Ollama
  judgment proof replay, but those claims were not reproduced in this stage.
- Runtime archive has message ledgers but `0` full `.conversation.json`
  snapshots.
- Runtime/download lineage is still noisy: `8` duplicate artifact aliases remain,
  especially repeated `repo-delta-004.bundle`, `DELTA_MANIFEST.md`, and
  `score.md` candidates.
- `316` `unwrap()` calls and `9` `expect()` calls remain in `src` and `examples`,
  which is a large panic surface for a system claiming bounded recovery.
- Tracked runtime/upload/download artifacts remain in repo history, weakening
  simplicity and repository hygiene.
- Policy learning exists as source and tests, but this evaluation did not
  reproduce a fresh policy-promotion replay trace from completed TLog history.
- No external observation stream test, external API action test, or offline
  semantic-artifact verification test was reproduced.

## Dimension Rationale

| Dimension | Score | Evidence |
|---|---:|---|
| I | 7.6 | Strong architecture across deterministic kernel, typed evidence, verification, learning, policy, LLM, and orchestration surfaces. |
| E | 6.5 | Python/router checks complete and root Cargo checks no longer depend on a missing wrapper; Rust validation is still unavailable. |
| C | 5.9 | Git diff, Python tests, router checks, and validation-report semantics pass; Rust fmt/test/clippy, graph replay, and Ollama proof replay were not reproduced. |
| A | 7.8 | Repository direction closely matches `GOAL.md`; live autonomous operation remains unproven. |
| R | 6.5 | Root/wrapper/Ollama validation states are separated and no unsafe code was found; missing graph/toolchain/Ollama proof and many unwraps cap robustness. |
| P | 5.5 | Router and Python tests are bounded; no root benchmark, throughput, latency, cargo timing, or successful end-to-end Rust validation timing was reproduced. |
| S | 6.3 | Normal clones are less coupled to local wrapper paths; validation remains single-repo, toolchain-sensitive, and partially unavailable. |
| D | 7.9 | Frozen-kernel intent, hash-chained logs, bundle/head binding, deterministic reports, and explicit validation branches are strong; absent graph/proof replay weakens confidence. |
| T | 8.8 | Missing signals, runtime metrics, validation command outcomes, wrapper branch state, and archive exclusions are explicit and machine-readable. |
| Co | 6.9 | README, GOAL, score, plans, scripts, and manifests guide collaborators; generated artifacts and duplicate aliases add noise. |
| Em | 7.6 | Restore, observe, test, and delta workflows are easier to reproduce; external Rust/Ollama/wrapper setup is still required. |
| B | 7.0 | High-potential autonomous-runtime prototype; benefit is capped until validation is portable and reproducible. |
| L | 6.3 | Learning/policy modules and tests exist; fresh policy-promotion replay was not reproduced in this stage. |
| Si | 6.0 | Dependency-minimal Rust and removal of the default absolute wrapper help; repo-history clutter, generated artifacts, and unwrap surface still hurt simplicity. |
| F | 7.5 | Canonical normal-form direction, sanitized runtime archive, validation-receipt hardening, and portable wrapper contract improve compatibility; missing proof pipeline remains the blocker. |

## Missing Validation Signals

```text
missing_root_rust_toolchain = true
missing_cargo_fmt = true
missing_cargo_test = true
missing_clippy = true
missing_cargo_run_ollama_judgment = true
missing_generated_graph_json = true
missing_wrapper_graph_validation = true
missing_rustc_wrapper_telemetry = true
missing_conversation_snapshot = true
missing_runtime_download_history = false
missing_external_observation_stream_test = true
missing_external_api_action_test = true
missing_semantic_artifact_verification_test = true
missing_policy_learning_replay_trace = true
missing_full_observe_validation_summary = false
missing_full_python_manifest_suite_pass = false
```

## Required Next Work

```text
1. Restore a portable Rust toolchain and run fmt/test/clippy with wrapper variables cleared.
2. Provide `CANON_RUSTC_WRAPPER=/path/to/canon-rustc-v3` and run wrapper graph validation.
3. Regenerate state/rustc/*/graph.json and require wrapper telemetry in validation.
4. Run ollama_judgment against a local endpoint and verify receipt/proof replay.
5. Add offline semantic-artifact verification and policy-learning replay tests.
6. Add external observation stream and external API action tests.
7. Classify or replace high-risk unwrap/expect sites with typed error handling.
8. Stop tracking runtime/upload/download artifacts in normal repository history.
9. Collapse duplicate artifact aliases or make dedupe evidence first-class.
10. Capture full conversation snapshots or formally declare message ledgers canonical.
```