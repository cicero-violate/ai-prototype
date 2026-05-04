base_commit: a4f94113d15ee6e01b35fd87edf78c9f654ca305
head_commit: 37e6fec9576d4afc7b39c246cc8db046e01a69a8

# Delta Manifest

## Changed Files
- .cargo/config.toml
- README.md
- plan.md
- score.md
- scripts/observe_validation.sh
- tests/test_observe_validation_contract.py

## Validation Results
- validation_status: fail
- validation_command_count: 10
- validation_test_count: 12
- zero_test_reason: None
- python_unit_test_count: 12
- router_test_count: 0
- cargo_test_count_when_available: None
- failed_required_commands: ["router_offline_tests"]
- missing_signal_count: 14
- report_sha256: d4d5851b483c37fab4603fea0ca38d9174b0c47089fc2509b32c0f0528e6fb22
- bundle_sha256: e33c809ecbd2827257d4ed605689e0053eb57ee195611d935a6a76171a70cacf
- bundle_verify: pass
- bundle_heads: ["37e6fec9576d4afc7b39c246cc8db046e01a69a8 HEAD"]
- validation_report_git_head: 37e6fec9576d4afc7b39c246cc8db046e01a69a8
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
- runtime_archive_sha256: 5547cb4592feb48a9f24a200dea5416088ec74ffe48076b572dfffa5e982a389
- runtime_manifest_base_commit: a4f94113d15ee6e01b35fd87edf78c9f654ca305
- runtime_archive_log_total: 8545
- runtime_archive_download_total: 78
- runtime_performance_signal_present: True
- runtime_performance_budget_status: pass
- runtime_performance_budget_failures: {}
- runtime_performance_budgets: {"download_follow_get_ms": 60000.0, "download_initial_get_ms": 60000.0, "download_write_ms": 1000.0, "project_agent_elapsed_ms": 1800000.0}
- validation_command_duration_ms: {"count": 9, "max": 9549.0, "median": 0.0, "min": 0.0, "p95": 9549.0}
- project_agent_elapsed_ms_count: 1050
- project_agent_elapsed_ms_median: 189027.187
- project_agent_elapsed_ms_p95: 1297595.353
- project_agent_elapsed_ms_max: 1778293.765
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
- runtime_download_history_record_count: 66
- runtime_unique_download_alias_count: 0
- runtime_unique_download_aliases: []
- runtime_archive_conversation_snapshots: 0
- runtime_stale_advisory_count: 1
- runtime_candidate_error_count: 0
- runtime_duplicate_artifact_aliases: 0
- delta_base_is_ancestor: True
- delta_changed_file_count: 6
- tracked_file_count: 120
- rust_file_count_src_examples: 53
- rust_test_attr_count: 103
- rust_cfg_test_count: 2
- unwrap_call_count_src_examples: 316
- expect_call_count_src_examples: 3

## Validation Commands
- git_diff_check: pass :: git diff --check
- git_delta_diff_check: pass :: git diff --check a4f94113d15ee6e01b35fd87edf78c9f654ca305..HEAD
- python_unit_tests: pass :: python3 -m unittest discover -s tests -p test_*.py
- panic_surface_validation: pass :: python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
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
- missing_policy_learning_replay_trace: True
- missing_root_rust_toolchain: True
- missing_runtime_download_history: False
- missing_runtime_performance_signal: False
- missing_rustc_wrapper_telemetry: True
- missing_semantic_artifact_verification_test: True
- missing_wrapper_graph_validation: True

## Receiver Apply Commands
```bash
git fetch ./repo-delta-001.bundle HEAD && git merge --ff-only FETCH_HEAD
```
