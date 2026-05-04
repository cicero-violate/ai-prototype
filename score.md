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
G = 6.94 / 10
max(G) = good
```

Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers
the whole system.

## Score Summary

```text
I  = 7.7 / 10
E  = 6.6 / 10
C  = 6.2 / 10
A  = 7.9 / 10
R  = 6.8 / 10
P  = 5.8 / 10
S  = 6.2 / 10
D  = 7.8 / 10
T  = 9.0 / 10
Co = 6.9 / 10
Em = 7.3 / 10
B  = 7.0 / 10
L  = 6.2 / 10
Si = 6.0 / 10
F  = 7.4 / 10

G = 6.94 / 10
GOOD = max(G) = 6.94 / 10 = good
```

Judgment: this repository has a serious deterministic-agent architecture and a
strong audit/receipt direction, but it remains below production confidence. The
current local EXECUTE stage reproduced Python tests, router offline tests,
panic-surface validation, runtime-archive parsing, runtime-performance budget
extraction, and final observe summary creation. It did not reproduce Rust
build/test/clippy, wrapper graph capture, local Ollama proof replay, semantic
artifact validation, policy-learning replay, or external observation/API tests.

## Evidence Snapshot

```text
bundle = /mnt/data/ai.bundle
restored_repo = /mnt/data/work-ai-eval/ai-repo
restored_base_commit = ad35d4a71e71d02da4e28e16067cdfef5bb85baf
branch = main
git_status_during_observe = dirty before commit, clean after final observe
recent_history = ad35d4a starting agent run; 7ccb995 starting agent run; 50590a6 Reduce production panic surface
goal_md_present = true
cargo_toml_package = ai 0.1.0, edition 2024
root_dependencies = 0 third-party Rust dependencies
tracked_file_count = 192
rust_files_src_examples = 53
rust_test_attrs_static_scan = 103
rust_cfg_test_sections_static_scan = 2
unsafe_token_count_src_examples = 0
panic_call_count_src_examples = 0
unwrap_call_count_src_examples = 316
expect_call_count_src_examples = 3
panic_surface_production_unwrap_count = 0
panic_surface_production_expect_count = 0
panic_surface_production_panic_count = 0
panic_surface_test_total = 319
panic_surface_example_total = 0
state_graph_present = false
cargo_available = false
rustc_available = false
node_available = true, v22.16.0
npm_available = true, 10.9.2
runtime_archive_present = true
runtime_archive_sha256 = 521b2a6334b1200c6422688ebf64bed792b826b2d616007d144fcb1593c45fb0
runtime_manifest_base_commit = ad35d4a71e71d02da4e28e16067cdfef5bb85baf
runtime_manifest_schema_version = 1
runtime_manifest_included_files = 111
runtime_manifest_excluded_files = 3796
runtime_manifest_excluded_reasons = apply-worktree:3744, generated-bundle:30, signed-url-cache:20, secret-token-cache:1, generated-runtime-archive:1
runtime_manifest_scans = leak:0, schema:0, integrity:0 findings
runtime_archive_members_observe_scan = 112
runtime_archive_ndjson_files_extracted = 74
runtime_archive_ndjson_total_lines_extracted = 13965
runtime_archive_log_total_observe_scan = 13728
runtime_archive_download_total_observe_scan = 237
runtime_download_history_records = 194
runtime_download_history_by_classification = delta_applied:18, download_event:27, download_ledger:131, live_evidence:17, stale_advisory:1
runtime_candidate_error_count = 0
runtime_duplicate_artifact_aliases = 8
runtime_conversation_snapshots = 0
observe_validation_status = partial
observe_validation_report_rows = 21
observe_validation_summary_present = true
validation_command_count = 10
validation_test_count = 57
python_unittest_discover = pass, 12 tests
python_observe_contract_tests = pass, 7 tests
python_write_delta_manifest_tests = pass, 5 tests
panic_surface_validation = pass, production_total 0, test_total 319
router_offline_tests = pass, syntax 41 + behavior 4 = 45 tests
runtime_performance_event = present
runtime_performance_budget_status = pass
project_agent_elapsed_ms_count = 921
project_agent_elapsed_ms_median = 182049.489
project_agent_elapsed_ms_p95 = 1327638.462
project_agent_elapsed_ms_max = 1778293.765
download_initial_get_ms_median = 12075.163
download_initial_get_ms_p95 = 20825.032
download_follow_get_ms_median = 9909.547
download_follow_get_ms_p95 = 17904.051
download_resolved_get_ms_median = 6293.140
download_write_ms_median = 0.148
cargo_fmt_check = unavailable, cargo not found
cargo_test_all_targets = unavailable, cargo not found
cargo_clippy_all_targets = unavailable, cargo not found
wrapper_graph_validation = skipped_env_missing, missing CANON_RUSTC_WRAPPER
ollama_judgment_example = skipped_env_missing, missing CANON_OLLAMA_BASE_URL or CANON_OLLAMA_MODEL
missing_signal_count = 13
```

## Positive Evidence

- `GOAL.md` defines a coherent system target: frozen deterministic kernel,
  codec/runtime/capability layering, append-only TLog, bounded recovery,
  learning promotion, policy store, and LLM specialization.
- `Cargo.toml` has no third-party Rust dependencies, which improves auditability
  and lowers supply-chain surface.
- `src/lib.rs` and `src/main.rs` forbid unsafe code; static scan found `0`
  unsafe tokens and `0` `panic!` calls under `src` and `examples`.
- `scripts/validate_rust_panic_surface.py --fail-production-unwrap` passed with
  `production_total=0`, `example_total=0`, and `test_total=319`.
- Python validation is now reproducible in this environment:
  `python3 -m unittest discover -s tests -v` passed `12/12` tests.
- `scripts/observe_validation.sh` now emits a dedicated
  `runtime_performance_metrics` row plus summary fields for project-agent
  elapsed time, download GET/write timings, command durations, and budget
  status.
- Router offline validation passed with `syntax 41`, `behavior 4`, and total
  `tests 45`.
- Runtime archive lineage binds to the restored head commit and includes `194`
  download-history records with `18` delta-applied records and `17` live-evidence
  records.
- Runtime archive hygiene is strong: leak, schema, and integrity scans each
  report `0` findings; secret-token cache and signed-url caches were excluded.
- The archive contains large operational evidence: `74` extracted NDJSON files
  and `13965` NDJSON rows, including project-agent and network request logs.

## Critical Risks

- OBSERVE status is still `partial`, not clean production validation.
- The repository could not be Rust-validated here: `cargo` and `rustc` are not
  present, so `cargo fmt --check`, `cargo test --all-targets`, and
  `cargo clippy --all-targets -- -D warnings` are unavailable.
- Wrapper graph validation is not reproduced because `CANON_RUSTC_WRAPPER` is
  missing and no `state/rustc/*/graph.json` exists in the restored worktree.
- Current graph metrics are absent: node count, edge count, intent coverage,
  redundant-path pairs, and alpha pathways are historical claims from `GOAL.md`,
  not fresh OBSERVE evidence.
- `cargo run --example ollama_judgment` was skipped because both local Ollama
  environment variables and the Rust toolchain are unavailable.
- Policy-learning replay, semantic artifact verification, external observation
  stream tests, and external API action tests were not reproduced.
- Runtime archive performance evidence is budgeted but not optimized: p95
  project-agent elapsed time is `1327638.462ms`, so the signal is useful for
  regression detection but not yet a speed win.
- Runtime archive has no full conversation snapshots and has `8` duplicate
  artifact aliases, so lineage is useful but still noisy.
- Test fixture panic surface remains large: `316` `unwrap()` and `3` `expect()`
  calls are classified as test-only, but they still inflate review noise.
- Recent git history includes repeated `starting agent run` bundle-update commits,
  which preserve loop evidence but are weak semantic history for collaborators.
- Restored project context contains generated downloads, bundles, runtime traces,
  and many patch artifacts, making the canonical source boundary harder to read.

## Dimension Rationale

| Dimension | Score | Evidence                                                                                                                                               |
|-----------+-------+--------------------------------------------------------------------------------------------------------------------------------------------------------|
| I         |   7.7 | Strong typed architecture across kernel, runtime, capabilities, receipts, verification, learning, policy, observation, and LLM surfaces.               |
| E         |   6.6 | Observe now completes with explicit runtime performance extraction; missing Rust toolchain and graph path still block complete evaluation.             |
| C         |   6.2 | Git diff, Python tests, panic validation, router tests, and performance budget checks pass; Rust, graph, and Ollama checks are unavailable or skipped. |
| A         |   7.9 | Source layout and `GOAL.md` are aligned around deterministic kernel plus growing capability layer; autonomous operation remains unproven here.         |
| R         |   6.8 | No production unsafe/panic/unwrap/expect surface detected; unvalidated Rust build and proof paths cap confidence.                                      |
| P         |   5.8 | Runtime archive performance evidence is now parsed and budgeted; no Rust timing, benchmark, LLM latency, memory, or sustained-load signal exists.      |
| S         |   6.2 | Layered capability model and zero Rust dependencies help scale; validation is still environment-sensitive and single-repo.                             |
| D         |   7.8 | Hash/log/replay direction and observe summary are deterministic; missing graph/toolchain replay prevents stronger score.                               |
| T         |   9.0 | Missing signals, runtime archive hygiene, command outputs, panic buckets, performance budgets, and archive lineage are explicit.                       |
| Co        |   6.9 | README/GOAL/score/scripts/reports help handoff; generated artifacts and repeated bundle commits reduce review clarity.                                 |
| Em        |   7.3 | Restore/observe/delta scripts and runtime performance evidence empower independent work; missing cargo/rustc blocks full verification.                 |
| B         |   7.0 | Targets high-value autonomous correctness and auditability; demonstrated benefit remains prototype-level.                                              |
| L         |   6.2 | Learning/policy surfaces and runtime history exist; no fresh policy-promotion replay trace was reproduced.                                             |
| Si        |   6.0 | Frozen-kernel/capability split is simple; broad exports, fixtures, receipts, patches, and generated context keep complexity high.                      |
| F         |   7.4 | Canonical effect/proof direction is future-proof; missing graph/Ollama/Rust/semantic validation limits confidence.                                     |

## Missing Validation Signals

```text
missing_observe_validation_summary = false
missing_full_python_unittest_discover = false
missing_write_delta_manifest_stale_head_completion = false
missing_root_rust_toolchain = true
missing_cargo_fmt = true
missing_cargo_test = true
missing_clippy = true
missing_wrapper_graph_validation = true
missing_generated_graph_json = true
missing_rustc_wrapper_telemetry = true
missing_cargo_run_ollama_judgment = true
missing_semantic_artifact_verification_test = true
missing_policy_learning_replay_trace = true
missing_external_observation_stream_test = true
missing_external_api_action_test = true
missing_conversation_snapshot = true
missing_panic_surface_validation = false
missing_runtime_download_history = false
missing_runtime_performance_signal = false
```

## Required Next Work

1. Restore `cargo` and `rustc`, then run root Rust `fmt`, `test`, and `clippy`.
2. Run wrapper graph validation with `CANON_RUSTC_WRAPPER` and require fresh
   `state/rustc/*/graph.json` telemetry in OBSERVE.
3. Run `cargo run --example ollama_judgment` with local Ollama and record
   receipt/proof/tamper-rejection evidence.
4. Add or reproduce semantic artifact verification, policy-learning replay,
   external observation stream, external API action, and sustained performance
   tests.
5. Reduce generated/download/patch artifact noise so collaborators can separate
   canonical source from loop evidence.
