base_commit: bbcaa3947d447396ee3599b63a9af8125b95b2a2
head_commit: 0f024c8c86f0cfded3678075ee6a571960be8824

# Delta Manifest

## Changed Files
- plan.md
- score.md
- scripts/write_delta_manifest.py
- tests/test_write_delta_manifest.py

## Validation Results
- validation_status: partial
- validation_command_count: 9
- validation_test_count: 13
- zero_test_reason: None
- python_unit_test_count: 13
- router_test_count: 0
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 7
- report_sha256: 292f8906547a59daa18b3580d1828986515f9f5e0527e2522b45f5341476b3a2
- bundle_sha256: 7103925f2c24b9c702dac9b5fc7e105a5d05c1d33eaf2359e4d70e8aec881236
- bundle_verify: pass
- bundle_heads: ["0f024c8c86f0cfded3678075ee6a571960be8824 HEAD"]
- bundle_required_refs: ["bbcaa3947d447396ee3599b63a9af8125b95b2a2"]
- bundle_requires_base_commit: True
- validation_report_git_head: 0f024c8c86f0cfded3678075ee6a571960be8824
- changed_file_count: 4
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
- runtime_archive_sha256: None
- runtime_manifest_base_commit: bbcaa3947d447396ee3599b63a9af8125b95b2a2
- runtime_archive_log_total: None
- runtime_archive_download_total: None
- runtime_performance_signal_present: None
- runtime_performance_budget_status: None
- runtime_performance_budget_failures: null
- runtime_performance_budgets: null
- validation_command_duration_ms: {"count": 9}
- project_agent_elapsed_ms_count: None
- project_agent_elapsed_ms_median: None
- project_agent_elapsed_ms_p95: None
- project_agent_elapsed_ms_max: None
- download_initial_get_ms_median: None
- download_initial_get_ms_p95: None
- download_initial_get_ms_max: None
- download_follow_get_ms_median: None
- download_follow_get_ms_p95: None
- download_follow_get_ms_max: None
- download_resolved_get_ms_median: None
- download_resolved_get_ms_p95: None
- download_resolved_get_ms_max: None
- download_write_ms_median: None
- download_write_ms_p95: None
- download_write_ms_max: None
- runtime_download_history_record_count: None
- runtime_unique_download_alias_count: None
- runtime_unique_download_aliases: null
- runtime_archive_conversation_snapshots: None
- runtime_stale_advisory_count: None
- runtime_candidate_error_count: None
- runtime_duplicate_artifact_aliases: None
- delta_base_is_ancestor: True
- delta_changed_file_count: 4
- tracked_file_count: 120
- rust_file_count_src_examples: 53
- rust_test_attr_count: 103
- rust_cfg_test_count: 2
- unwrap_call_count_src_examples: 316
- expect_call_count_src_examples: 3

## Validation Commands
- git_diff_check: pass :: git diff --check
- git_delta_diff_check: pass :: git diff --check bbcaa3947d447396ee3599b63a9af8125b95b2a2..HEAD
- python_unit_tests: pass :: python3 -m unittest discover -s tests -p test_*.py -v
- panic_surface_validation: pass :: python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
- cargo_fmt_check: unavailable :: cargo fmt --check
- cargo_test_all_targets: unavailable :: cargo test --all-targets
- cargo_clippy_all_targets: unavailable :: cargo clippy --all-targets -- -D warnings
- wrapper_graph_validation: skipped_env_missing :: cargo test --all-targets
- ollama_judgment_example: skipped_env_missing :: cargo run --example ollama_judgment

## Missing Signal Flags
- missing_cargo_fmt: True
- missing_cargo_run_ollama_judgment: True
- missing_cargo_test: True
- missing_clippy: True
- missing_generated_graph_json: True
- missing_root_rust_toolchain: True
- missing_wrapper_graph_validation: True

## Receiver Apply Commands
```bash
git fetch ./repo-delta-002.bundle HEAD && git merge --ff-only FETCH_HEAD
```
