base_commit: ad35d4a71e71d02da4e28e16067cdfef5bb85baf
head_commit: 16e35d31c78e1b95ee753306a282943dfd0ecc65

# Delta Manifest

## Changed Files
- IMPLEMENTATION_PLAN.md
- score.md
- scripts/observe_validation.sh
- scripts/write_delta_manifest.py
- tests/test_observe_validation_contract.py

## Validation Results
- validation_status: partial
- validation_command_count: 10
- validation_test_count: 57
- zero_test_reason: None
- python_unit_test_count: 12
- router_test_count: 45
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 13
- report_sha256: 574b8c71148b3d28f066d934d85e6f6067a3d88ae30c6baaba6eddad6594d452
- bundle_sha256: e16a0f31d9141b7bb58c0217049034f8d479025f0394696b966e6719ddb88faf
- bundle_verify: pass
- bundle_heads: ["16e35d31c78e1b95ee753306a282943dfd0ecc65 HEAD"]
- validation_report_git_head: 16e35d31c78e1b95ee753306a282943dfd0ecc65
- changed_file_count: 5
- cargo_available: False
- rustc_available: False
- toolchain_path_added: False
- rust_toolchain_source: missing
- root_rust_env_overrides: ["RUSTC_WORKSPACE_WRAPPER", "RUSTC_WRAPPER"]
- wrapper_override_required: False
- wrapper_override_used: False
- wrapper_override_env: []
- wrapper_graph_validation_result: skipped_env_missing
- wrapper_graph_validation_requested: False
- wrapper_graph_validation_available: False
- rustc_wrapper_path_exists: False
- git_delta_diff_check_result: pass
- state_graph_present: False
- runtime_archive_sha256: 521b2a6334b1200c6422688ebf64bed792b826b2d616007d144fcb1593c45fb0
- runtime_manifest_base_commit: ad35d4a71e71d02da4e28e16067cdfef5bb85baf
- runtime_archive_log_total: 13728
- runtime_archive_download_total: 237
- runtime_performance_signal_present: True
- runtime_performance_budget_status: pass
- runtime_performance_budget_failures: {}
- runtime_performance_budgets: {"download_follow_get_ms": 60000.0, "download_initial_get_ms": 60000.0, "download_write_ms": 1000.0, "project_agent_elapsed_ms": 1800000.0}
- validation_command_duration_ms: {"count": 9, "max": 14582.0, "median": 22.0, "min": 0.0, "p95": 14582.0}
- project_agent_elapsed_ms_count: 921
- project_agent_elapsed_ms_median: 182049.489
- project_agent_elapsed_ms_p95: 1327638.462
- project_agent_elapsed_ms_max: 1778293.765
- download_initial_get_ms_median: 12075.163
- download_initial_get_ms_p95: 20825.032
- download_initial_get_ms_max: 34201.067
- download_follow_get_ms_median: 9909.547
- download_follow_get_ms_p95: 17904.051
- download_follow_get_ms_max: 23909.194
- download_resolved_get_ms_median: 6293.14
- download_resolved_get_ms_p95: 16276.383
- download_resolved_get_ms_max: 17609.301
- download_write_ms_median: 0.148
- download_write_ms_p95: 0.731
- download_write_ms_max: 6.135
- runtime_download_history_record_count: 194
- runtime_unique_download_alias_count: 21
- runtime_unique_download_aliases: ["DELTA_MANIFEST.md", "IMPLEMENTATION_PLAN.md", "ai-IMPLEMENTATION_PLAN.md", "ai-eval-validation-report.ndjson", "ai-implementation-plan-update.patch", "ai-observe-score-update.patch", "ai-observe-score.md", "ai-observe-validation-current.ndjson", "ai-observe-validation-output-latest.txt", "ai-observe-validation-output.txt", "ai-observe-validation-report.ndjson", "ai-router-tests-run.txt", "ai-runtime-archive-metrics.json", "ai-runtime-observe-current-summary.json", "ai-runtime-observe-summary.json", "ai-score-update.patch", "ai-score.md", "ai-validation-report-eval.ndjson", "observe-validation.ndjson", "repo-delta-004.bundle", "score.md"]
- runtime_archive_conversation_snapshots: 0
- runtime_stale_advisory_count: 1
- runtime_candidate_error_count: 0
- runtime_duplicate_artifact_aliases: 8
- delta_base_is_ancestor: True
- delta_changed_file_count: 5
- tracked_file_count: 192
- rust_file_count_src_examples: 53
- rust_test_attr_count: 103
- rust_cfg_test_count: 2
- unwrap_call_count_src_examples: 316
- expect_call_count_src_examples: 3

## Validation Commands
- git_diff_check: pass :: git diff --check
- git_delta_diff_check: pass :: git diff --check ad35d4a71e71d02da4e28e16067cdfef5bb85baf..HEAD
- python_unit_tests: pass :: python3 -m unittest discover -s tests -p test_*.py
- panic_surface_validation: pass :: python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
- router_offline_tests: pass :: bash run_tests.sh
- cargo_fmt_check: unavailable :: cargo fmt --check
- cargo_test_all_targets: unavailable :: cargo test --all-targets
- cargo_clippy_all_targets: unavailable :: cargo clippy --all-targets -- -D warnings
- wrapper_graph_validation: skipped_env_missing :: cargo test --all-targets
- ollama_judgment_example: skipped_env_missing :: cargo run --example ollama_judgment

## Missing Signal Flags
- missing_artifact_apply_worktree: False
- missing_cargo_fmt: True
- missing_cargo_run_ollama_judgment: True
- missing_cargo_test: True
- missing_clippy: True
- missing_conversation_snapshot: True
- missing_external_api_action_test: True
- missing_external_observation_stream_test: True
- missing_generated_graph_json: True
- missing_panic_surface_validation: False
- missing_policy_learning_replay_trace: True
- missing_root_rust_toolchain: True
- missing_runtime_download_history: False
- missing_runtime_performance_signal: False
- missing_rustc_wrapper_telemetry: True
- missing_semantic_artifact_verification_test: True
- missing_wrapper_graph_validation: True

## Receiver Apply Commands
```bash
git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD
```
