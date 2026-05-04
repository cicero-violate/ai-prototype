base_commit: ead9e72d22088d72ecf11bd9cf2acd5292afa319
head_commit: cbb2bb78918178670a3026627b87c499eca30c69

# Delta Manifest

## Changed Files
- Cargo.lock
- Cargo.toml
- plan.md
- score.md

## Validation Results
- validation_status: partial
- validation_command_count: 9
- validation_test_count: 125
- zero_test_reason: None
- python_unit_test_count: 22
- router_test_count: 0
- cargo_test_count_when_available: 103
- failed_required_commands: []
- missing_signal_count: 6
- report_sha256: f4fba4260c727b83b24411a5168fa4dc2df4c76715414d0ad6d15e45de16ee3a
- bundle_sha256: 76f05900de241b27e357281417c52f486d7aff341e40247648050285f9297205
- bundle_verify: pass
- bundle_heads: ["cbb2bb78918178670a3026627b87c499eca30c69 HEAD"]
- bundle_required_refs: ["ead9e72d22088d72ecf11bd9cf2acd5292afa319"]
- bundle_requires_base_commit: True
- validation_report_git_head: cbb2bb78918178670a3026627b87c499eca30c69
- changed_file_count: 4
- cargo_available: True
- rustc_available: True
- toolchain_path_added: False
- rust_toolchain_source: /mnt/data/rustc-python-install-prefix
- root_rust_env_overrides: ["RUSTC_WRAPPER", "RUSTC_WORKSPACE_WRAPPER"]
- wrapper_override_required: False
- wrapper_override_used: False
- wrapper_override_env: []
- wrapper_graph_validation_result: skipped_env_missing
- wrapper_graph_validation_requested: False
- wrapper_graph_validation_available: False
- rustc_wrapper_path_exists: False
- git_delta_diff_check_result: pass
- state_graph_present: False
- runtime_archive_sha256: 56a6a552e5511618b45d570eeead3a20c58044cb62307bf595b356c816c5d423
- runtime_manifest_base_commit: ead9e72d22088d72ecf11bd9cf2acd5292afa319
- runtime_archive_log_total: 16466
- runtime_archive_download_total: 47
- runtime_archive_inspection_status: pass
- runtime_archive_download_index_files: 64
- runtime_archive_prior_state_files: 61
- runtime_archive_conversation_ledger_files: 17
- runtime_archive_delta_receipt_files: 6
- runtime_archive_audit_files: 1
- runtime_archive_current_run_summary_present: True
- runtime_archive_runtime_manifest_present: True
- runtime_performance_signal_present: True
- runtime_performance_budget_status: pass
- runtime_performance_budget_failures: {}
- runtime_performance_budgets: {"download_follow_get_ms": 60000.0, "download_initial_get_ms": 60000.0, "download_write_ms": 1000.0, "project_agent_elapsed_ms": 1800000.0}
- validation_command_duration_ms: {}
- project_agent_elapsed_ms_count: None
- project_agent_elapsed_ms_median: None
- project_agent_elapsed_ms_p95: None
- project_agent_elapsed_ms_max: None
- download_initial_get_ms_median: 11619.654
- download_initial_get_ms_p95: 26107.765
- download_initial_get_ms_max: 26194.124
- download_follow_get_ms_median: 6417.586
- download_follow_get_ms_p95: 18025.712
- download_follow_get_ms_max: 36122.814
- download_resolved_get_ms_median: 6273.593
- download_resolved_get_ms_p95: 17586.025
- download_resolved_get_ms_max: 17751.214
- download_write_ms_median: 0.176
- download_write_ms_p95: 0.713
- download_write_ms_max: 0.941
- runtime_download_history_record_count: 177
- runtime_unique_download_alias_count: 17
- runtime_unique_download_aliases: ["DELTA_MANIFEST.md", "GOAL.md", "score.md", "repo-delta-002.bundle"]
- runtime_archive_conversation_snapshots: 0
- runtime_stale_advisory_count: 1
- runtime_manifest_base_expected: ead9e72d22088d72ecf11bd9cf2acd5292afa319
- runtime_manifest_base_matches_delta_base: True
- runtime_candidate_error_count: 0
- runtime_duplicate_artifact_aliases: 10
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
- cargo_fmt_check: unavailable :: cargo fmt --check
- cargo_test_all_targets: pass :: cargo test --all-targets -- --nocapture
- py_compile_manifest_writer: pass :: python3 -m py_compile scripts/write_delta_manifest.py
- python_unit_tests: pass :: python3 -m unittest discover -s tests -p test_*.py -v
- policy_learning_trace_validation: pass :: python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json
- panic_surface_validation: pass :: python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
- git_diff_check: pass :: git diff --check
- observe_validation: partial :: bash scripts/observe_validation.sh

## Missing Signal Flags
- missing_cargo_fmt: True
- missing_clippy: True
- missing_generated_graph_json: True
- missing_ollama_live_proof: True
- missing_rustc_wrapper_telemetry: True
- missing_wrapper_graph_validation: True

## Receiver Apply Commands
```bash
git fetch ./repo-delta-002.bundle HEAD && git merge --ff-only FETCH_HEAD
```
