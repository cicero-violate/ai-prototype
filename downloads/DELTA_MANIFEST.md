base_commit: 0037761cacb4b573af603bf28c7d5f3ca6f3637d
head_commit: 60364d4ca23ce92d345ca18a6a5bd436c309996d

# Delta Manifest

## Changed Files
- ai-chromium/router-server_bak/run_tests.sh
- score.md
- scripts/observe_validation.sh

## Validation Results
- validation_status: partial
- validation_command_count: 6
- validation_test_count: 41
- router_test_count: 41
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 12
- report_sha256: d572053ac6a6dbed395a194dc5ef87b56a042d61950866b23677f48846cd77a6
- bundle_sha256: db88708c6ba1310b3a4c76557dc338e7c6bd9efb7eb949a812c8e2079570ec5d
- bundle_verify: pass
- cargo_available: False
- rustc_available: False
- toolchain_path_added: False
- rust_toolchain_source: missing
- wrapper_override_required: True
- wrapper_override_used: False
- wrapper_override_env: ["RUSTC_WORKSPACE_WRAPPER", "RUSTC_WRAPPER"]
- rustc_wrapper_path_exists: False
- state_graph_present: False
- runtime_archive_log_total: 4018
- runtime_archive_download_total: 34
- runtime_archive_conversation_snapshots: 0

## Validation Commands
- git_diff_check: pass :: git diff --check
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
