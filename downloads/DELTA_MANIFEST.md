base_commit: c86c4438852e9a0779dcc53e860b9c4fc185824c
head_commit: 63a31574bafd6d64a0a4cf860750d2a7c5c8fd95

# Delta Manifest

## Changed Files
- plan.md
- score.md
- scripts/observe_validation.sh
- tests/test_observe_validation_contract.py

## Validation Results
- validation_status: partial
- validation_command_count: 9
- validation_test_count: 126
- zero_test_reason: None
- python_unit_test_count: 23
- router_test_count: 0
- cargo_test_count_when_available: 103
- failed_required_commands: []
- missing_signal_count: 6
- report_sha256: e59eb58423f28310e9116d3f1fe6df6cb13d7045fc9a3f9bde32396513a91c61
- bundle_sha256: e26aa4072b8d8862e1ec4eb31bb49ef24d5a9e0a39199d6e22e6268b3dd18ee2
- bundle_verify: pass
- bundle_heads: ["63a31574bafd6d64a0a4cf860750d2a7c5c8fd95 HEAD"]
- bundle_required_refs: ["c86c4438852e9a0779dcc53e860b9c4fc185824c"]
- bundle_requires_base_commit: True
- validation_report_git_head: 63a31574bafd6d64a0a4cf860750d2a7c5c8fd95
- changed_file_count: 4
- cargo_available: True
- rustc_available: True
- toolchain_path_added: False
- rust_toolchain_source: path
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
- runtime_archive_sha256: c3b78d1d1037b07d6e0e9263e1e18db044a91dece0a411807934e06d7052c293
- runtime_manifest_base_commit: c86c4438852e9a0779dcc53e860b9c4fc185824c
- runtime_archive_log_total: 16610
- runtime_archive_download_total: 183
- runtime_archive_inspection_status: pass
- runtime_archive_download_index_files: 37
- runtime_archive_prior_state_files: 9
- runtime_archive_conversation_ledger_files: 18
- runtime_archive_delta_receipt_files: 7
- runtime_archive_audit_files: 1
- runtime_archive_current_run_summary_present: True
- runtime_archive_runtime_manifest_present: True
- runtime_performance_signal_present: True
- runtime_performance_budget_status: pass
- runtime_performance_budget_failures: {}
- runtime_performance_budgets: {"download_follow_get_ms": 60000.0, "download_initial_get_ms": 60000.0, "download_write_ms": 1000.0, "project_agent_elapsed_ms": 1800000.0}
- validation_command_duration_ms: {}
- project_agent_elapsed_ms_count: 0
- project_agent_elapsed_ms_median: None
- project_agent_elapsed_ms_p95: None
- project_agent_elapsed_ms_max: None
- download_initial_get_ms_median: 11795.016
- download_initial_get_ms_p95: 26107.765
- download_initial_get_ms_max: 26194.124
- download_follow_get_ms_median: 6406.886
- download_follow_get_ms_p95: 18025.712
- download_follow_get_ms_max: 36122.814
- download_resolved_get_ms_median: 6317.745
- download_resolved_get_ms_p95: 17630.812
- download_resolved_get_ms_max: 17751.214
- download_write_ms_median: 0.192
- download_write_ms_p95: 0.422
- download_write_ms_max: 0.941
- runtime_download_history_record_count: 187
- runtime_unique_download_alias_count: 17
- runtime_unique_download_aliases: []
- runtime_archive_conversation_snapshots: 0
- runtime_stale_advisory_count: 1
- runtime_manifest_base_expected: c86c4438852e9a0779dcc53e860b9c4fc185824c
- runtime_manifest_base_matches_delta_base: True
- runtime_candidate_error_count: 0
- runtime_duplicate_artifact_aliases: 11
- delta_base_is_ancestor: True
- delta_changed_file_count: 4
- tracked_file_count: 129
- rust_file_count_src_examples: 58
- rust_test_attr_count: 103
- rust_cfg_test_count: 2
- unwrap_call_count_src_examples: 316
- expect_call_count_src_examples: 4
- policy_learning_trace_validation_result: pass
- policy_learning_trace_status: pass
- policy_learning_trace_function: learning_policy_llm_feedback_loop_drives_judgment
- policy_learning_trace_check_count: 4
- policy_learning_trace_missing_count: 0
- panic_surface_production_unwrap_count: 0
- panic_surface_production_expect_count: 0
- panic_surface_production_panic_count: 0
- panic_surface_test_total: 319
- panic_surface_example_total: 1

## Validation Commands
- bootstrap_rustc_session: pass :: python3 /mnt/data/bootstrap_rustc_session.py
- cargo_test_all_targets: pass :: cargo test --all-targets
- python_unit_tests: pass :: python3 -m unittest discover -s tests -p test_*.py -v
- policy_learning_trace_validation: pass :: python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json
- panic_surface_validation: pass :: python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
- observe_validation_syntax: pass :: bash -n scripts/observe_validation.sh
- python_script_compile: pass :: python3 -m py_compile scripts/write_delta_manifest.py scripts/validate_policy_learning_trace.py scripts/validate_rust_panic_surface.py
- git_diff_check: pass :: git diff --check
- runtime_archive_inspection: pass :: CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz source-derived inspection

## Missing Signal Flags
- missing_cargo_fmt: True
- missing_cargo_run_ollama_judgment: True
- missing_clippy: True
- missing_external_api_action_test: False
- missing_external_observation_stream_test: False
- missing_generated_graph_json: True
- missing_panic_surface_validation: False
- missing_policy_learning_replay_trace: False
- missing_runtime_inspection_contract: False
- missing_runtime_performance_signal: False
- missing_rustc_wrapper_telemetry: True
- missing_semantic_artifact_verification_test: False
- missing_wrapper_graph_validation: True

## Receiver Apply Commands
```bash
git fetch ./repo-delta-002.bundle HEAD && git merge --ff-only FETCH_HEAD
```
