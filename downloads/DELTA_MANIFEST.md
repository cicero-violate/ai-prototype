base_commit: 9ea8f585fc7d043d13cee60452a572ff29d1e796
head_commit: a84964c1a02a7abca7bf8689fd960faf679f5d05

# Delta Manifest

## Changed Files
- plan.md
- score.md
- scripts/observe_validation.sh
- scripts/write_delta_manifest.py
- tests/test_observe_validation_contract.py
- tests/test_write_delta_manifest.py

## Validation Results
- validation_status: partial
- validation_command_count: 12
- validation_test_count: 22
- zero_test_reason: None
- python_unit_test_count: 22
- router_test_count: 0
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 14
- report_sha256: 62371d20733d898cb6e3a8f82dd1a3382144f6ac3bdebcbc3698fd9be4dba2bc
- bundle_sha256: eb82df42333d6566f5599ae66894272d4bd95a4380c7327b62fdd757c9782c53
- bundle_verify: pass
- bundle_heads: ["a84964c1a02a7abca7bf8689fd960faf679f5d05 HEAD"]
- bundle_required_refs: ["9ea8f585fc7d043d13cee60452a572ff29d1e796"]
- bundle_requires_base_commit: True
- validation_report_git_head: a84964c1a02a7abca7bf8689fd960faf679f5d05
- changed_file_count: 6
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
- runtime_archive_sha256: 991dadda52b4b1f3f524f49e3a40e5951fd40eadfaf4734d70b2daf738ff9f2b
- runtime_manifest_base_commit: 9ea8f585fc7d043d13cee60452a572ff29d1e796
- runtime_archive_log_total: 11081
- runtime_archive_download_total: 108
- runtime_archive_inspection_status: pass
- runtime_archive_download_index_files: 11
- runtime_archive_prior_state_files: 7
- runtime_archive_conversation_ledger_files: 5
- runtime_archive_delta_receipt_files: 5
- runtime_archive_audit_files: 1
- runtime_archive_current_run_summary_present: True
- runtime_archive_runtime_manifest_present: True
- runtime_performance_signal_present: True
- runtime_performance_budget_status: pass
- runtime_performance_budget_failures: {}
- runtime_performance_budgets: {}
- validation_command_duration_ms: {"count": 6, "max": 0, "median": 0, "min": 0, "p95": 0}
- project_agent_elapsed_ms_count: 1221
- project_agent_elapsed_ms_median: 211391.881
- project_agent_elapsed_ms_p95: 1236515.064
- project_agent_elapsed_ms_max: 1778293.765
- download_initial_get_ms_median: 13476.278
- download_initial_get_ms_p95: 26194.124
- download_initial_get_ms_max: 26194.124
- download_follow_get_ms_median: 6404.516
- download_follow_get_ms_p95: 17612.516
- download_follow_get_ms_max: 17612.516
- download_resolved_get_ms_median: 6128.98
- download_resolved_get_ms_p95: 17325.114
- download_resolved_get_ms_max: 17325.114
- download_write_ms_median: 0.226
- download_write_ms_p95: 0.713
- download_write_ms_max: 0.861
- runtime_download_history_record_count: 112
- runtime_unique_download_alias_count: 6
- runtime_unique_download_aliases: ["DELTA_MANIFEST.md", "ai-phase1-GOAL-and-score.md", "ai-score.md", "repo-delta-001.bundle", "repo-delta-002.bundle", "score.md"]
- runtime_archive_conversation_snapshots: 0
- runtime_stale_advisory_count: 1
- runtime_manifest_base_expected: 9ea8f585fc7d043d13cee60452a572ff29d1e796
- runtime_manifest_base_matches_delta_base: True
- runtime_candidate_error_count: 0
- runtime_duplicate_artifact_aliases: 4
- delta_base_is_ancestor: True
- delta_changed_file_count: 6
- tracked_file_count: 122
- rust_file_count_src_examples: 53
- rust_test_attr_count: 107
- rust_cfg_test_count: 25
- unwrap_call_count_src_examples: 319
- expect_call_count_src_examples: 0
- policy_learning_trace_validation_result: pass
- policy_learning_trace_status: pass
- policy_learning_trace_function: learning_policy_llm_feedback_loop_drives_judgment
- policy_learning_trace_check_count: 4
- policy_learning_trace_missing_count: 0
- panic_surface_production_unwrap_count: 0
- panic_surface_production_expect_count: 0
- panic_surface_production_panic_count: 0
- panic_surface_test_total: 319
- panic_surface_example_total: 0

## Validation Commands
- py_compile_write_delta_manifest: pass :: python3 -m py_compile scripts/write_delta_manifest.py
- python_unit_tests: pass :: python3 -m unittest discover -s tests -p test_*.py
- policy_learning_trace_validation: pass :: python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json
- panic_surface_validation: pass :: python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
- git_diff_check: pass :: git diff --check
- git_delta_diff_check: pass :: git diff --check 9ea8f585fc7d043d13cee60452a572ff29d1e796..HEAD
- cargo_fmt_check: unavailable :: cargo fmt --check
- cargo_test_all_targets: unavailable :: cargo test --all-targets
- cargo_clippy_all_targets: unavailable :: cargo clippy --all-targets -- -D warnings
- wrapper_graph_validation: skipped_env_missing :: cargo test --all-targets
- ollama_judgment_example: skipped_env_missing :: cargo run --example ollama_judgment
- router_offline_tests: unavailable :: bash run_tests.sh

## Missing Signal Flags
- missing_artifact_apply_worktree: True
- missing_cargo_fmt: True
- missing_cargo_run_ollama_judgment: True
- missing_cargo_test: True
- missing_clippy: True
- missing_conversation_snapshot: True
- missing_external_api_action_test: True
- missing_external_observation_stream_test: True
- missing_generated_graph_json: True
- missing_panic_surface_validation: False
- missing_policy_learning_replay_trace: False
- missing_root_rust_toolchain: True
- missing_router_offline_tests: True
- missing_runtime_conversation_ledger: False
- missing_runtime_download_history: False
- missing_runtime_download_index: False
- missing_runtime_inspection_contract: False
- missing_runtime_manifest_base_match: False
- missing_runtime_performance_signal: False
- missing_runtime_prior_state: False
- missing_rustc_wrapper_telemetry: True
- missing_semantic_artifact_verification_test: True
- missing_wrapper_graph_validation: True

## Receiver Apply Commands
```bash
git fetch ./repo-delta-002.bundle HEAD && git merge --ff-only FETCH_HEAD
```
