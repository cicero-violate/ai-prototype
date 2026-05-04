base_commit: fae4c60c6fae6177f92837119930e412c9d02e65
head_commit: 1978744b23710ca1f4c7bd97b113f2815335f265

# Delta Manifest

## Changed Files
- IMPLEMENTATION_PLAN.md
- README.md
- score.md
- scripts/observe_validation.sh
- scripts/write_delta_manifest.py

## Validation Results
- validation_status: partial
- validation_command_count: 7
- validation_test_count: 45
- router_test_count: 45
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 12
- report_sha256: bd675a94d07d694ca150ca871aff4f75a2484339b2a3b93c52959544240ad420
- bundle_sha256: 878435adacffa89c24bed33e4604ea6f9d9518250e86cf6ecb0dbf61f7e238f4
- bundle_verify: pass
- validation_report_git_head: 1978744b23710ca1f4c7bd97b113f2815335f265
- changed_file_count: 5
- cargo_available: False
- rustc_available: False
- toolchain_path_added: False
- rust_toolchain_source: missing
- wrapper_override_required: True
- wrapper_override_used: False
- wrapper_override_env: ["RUSTC_WORKSPACE_WRAPPER", "RUSTC_WRAPPER"]
- rustc_wrapper_path_exists: False
- git_delta_diff_check_result: pass
- state_graph_present: False
- runtime_archive_sha256: 6aa85edd54e97fdd62f7484fda96bd2821ac96445ebf6a28f4a6ffc682268922
- runtime_manifest_base_commit: fae4c60c6fae6177f92837119930e412c9d02e65
- runtime_archive_log_total: 6259
- runtime_archive_download_total: 69
- runtime_archive_conversation_snapshots: 0
- runtime_stale_advisory_count: 1
- runtime_candidate_error_count: 0
- runtime_duplicate_artifact_aliases: 6
- delta_base_is_ancestor: True
- delta_changed_file_count: 5
- tracked_file_count: 189
- rust_file_count_src_examples: 53
- rust_test_attr_count: 103
- rust_cfg_test_count: 2
- unwrap_call_count_src_examples: 316
- expect_call_count_src_examples: 9

## Validation Commands
- git_diff_check: pass :: git diff --check
- git_delta_diff_check: pass :: git diff --check fae4c60c6fae6177f92837119930e412c9d02e65..HEAD
- router_offline_tests: pass :: bash run_tests.sh
- cargo_fmt_check: unavailable :: cargo fmt --check
- cargo_test_all_targets: unavailable :: cargo test --all-targets
- cargo_clippy_all_targets: unavailable :: cargo clippy --all-targets -- -D warnings
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
- missing_policy_learning_replay_trace: True
- missing_root_rust_toolchain: True
- missing_runtime_download_history: False
- missing_rustc_wrapper_telemetry: True
- missing_semantic_artifact_verification_test: True

## Receiver Apply Commands
```bash
git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD
```
