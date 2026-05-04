base_commit: 7a8823141bc0e95c4689a3dff7f5a67840d1d4de
head_commit: 579e40aa3ff5e65289e5bffb40e91ff8c3e8953b

# Delta Manifest

## Changed Files
- plan.md
- score.md
- scripts/observe_validation.sh
- scripts/validate_policy_learning_trace.py
- tests/test_observe_validation_contract.py
- tests/test_policy_learning_trace_contract.py

## Validation Results
- validation_status: partial
- validation_command_count: 11
- validation_test_count: 19
- zero_test_reason: None
- python_unit_test_count: 19
- router_test_count: 0
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 14
- report_sha256: 11a7dc9a6d5eec543dfcd481b2161fd0a04935acc5a73cd9c198ab245f4b2b03
- bundle_sha256: df2dea36605ad76f30b52e0d86fee841704e23d7ebaeb8708b11484633f8c274
- bundle_verify: pass
- bundle_heads: ["579e40aa3ff5e65289e5bffb40e91ff8c3e8953b HEAD"]
- bundle_required_refs: ["7a8823141bc0e95c4689a3dff7f5a67840d1d4de"]
- bundle_requires_base_commit: True
- validation_report_git_head: 579e40aa3ff5e65289e5bffb40e91ff8c3e8953b
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
- runtime_archive_sha256: 7e37874ceebfcb6ac787628403650ca6a1431d3fd9ea2477dc845f5c221f17c1
- runtime_manifest_base_commit: 7a8823141bc0e95c4689a3dff7f5a67840d1d4de
- runtime_archive_log_total: 10070
- runtime_archive_download_total: 95
- runtime_performance_signal_present: True
- runtime_performance_budget_status: pass
- runtime_performance_budget_failures: {}
- runtime_performance_budgets: {"download_follow_get_ms": 60000.0, "download_initial_get_ms": 60000.0, "download_write_ms": 1000.0, "project_agent_elapsed_ms": 1800000.0}
- validation_command_duration_ms: {"count": 10, "max": 9379.0, "median": 8.0, "min": 0.0, "p95": 9379.0}
- project_agent_elapsed_ms_count: 1153
- project_agent_elapsed_ms_median: 211123.446
- project_agent_elapsed_ms_p95: 1266560.174
- project_agent_elapsed_ms_max: 1778293.765
- download_initial_get_ms_median: 12547.966
- download_initial_get_ms_p95: 19196.616
- download_initial_get_ms_max: 19196.616
- download_follow_get_ms_median: 6302.477
- download_follow_get_ms_p95: 17472.758
- download_follow_get_ms_max: 17472.758
- download_resolved_get_ms_median: 15624.488
- download_resolved_get_ms_p95: 17325.114
- download_resolved_get_ms_max: 17325.114
- download_write_ms_median: 0.202
- download_write_ms_p95: 0.396
- download_write_ms_max: 0.396
- runtime_download_history_record_count: 93
- runtime_unique_download_alias_count: 6
- runtime_unique_download_aliases: ["DELTA_MANIFEST.md", "ai-phase1-GOAL-and-score.md", "ai-score.md", "repo-delta-001.bundle", "repo-delta-002.bundle", "score.md"]
- runtime_archive_conversation_snapshots: 0
- runtime_stale_advisory_count: 1
- runtime_manifest_base_expected: 7a8823141bc0e95c4689a3dff7f5a67840d1d4de
- runtime_manifest_base_matches_delta_base: True
- runtime_candidate_error_count: 0
- runtime_duplicate_artifact_aliases: 2
- delta_base_is_ancestor: True
- delta_changed_file_count: 6
- tracked_file_count: 122
- rust_file_count_src_examples: 53
- rust_test_attr_count: 103
- rust_cfg_test_count: 2
- unwrap_call_count_src_examples: 316
- expect_call_count_src_examples: 3

## Validation Commands
- git_diff_check: pass :: git diff --check
- git_delta_diff_check: pass :: git diff --check 7a8823141bc0e95c4689a3dff7f5a67840d1d4de..HEAD
- python_unit_tests: pass :: python3 -m unittest discover -s tests -p test_*.py
- panic_surface_validation: pass :: python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
- policy_learning_trace_validation: pass :: python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json
- router_offline_tests: unavailable :: bash run_tests.sh
- cargo_fmt_check: unavailable :: cargo fmt --check
- cargo_test_all_targets: unavailable :: cargo test --all-targets
- cargo_clippy_all_targets: unavailable :: cargo clippy --all-targets -- -D warnings
- wrapper_graph_validation: skipped_env_missing :: cargo test --all-targets
- ollama_judgment_example: skipped_env_missing :: cargo run --example ollama_judgment

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
- missing_runtime_download_history: False
- missing_runtime_manifest_base_match: False
- missing_runtime_performance_signal: False
- missing_rustc_wrapper_telemetry: True
- missing_semantic_artifact_verification_test: True
- missing_wrapper_graph_validation: True

## Receiver Apply Commands
```bash
git fetch ./repo-delta-002.bundle HEAD && git merge --ff-only FETCH_HEAD
```
