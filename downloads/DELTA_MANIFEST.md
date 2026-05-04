base_commit: 4e8a41f7f6a4c31d275e986147d6193c1d1f3895
head_commit: 80ace85a63fb69c4899bbca8e29fce1c7b17ee73

# Delta Manifest

## Changed Files
- .gitignore
- IMPLEMENTATION_PLAN.md
- README.md
- ai-chromium/router-server_bak/run_tests.sh
- ai-chromium/router-server_bak/test/offline-contract.test.mjs
- score.md

## Validation Results
- validation_status: partial
- validation_command_count: 6
- validation_test_count: 45
- router_test_count: 45
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 12
- report_sha256: e1d0a3aed0e6bacc7b16b9c73eb5d92ef672823b0552e4717c053bccd23bb110
- bundle_sha256: 7d0b26b1c63d0629cc21a328b17f3f676b67571661c05a133fa45b4b6624fb60
- bundle_verify: pass
- changed_file_count: 6
- cargo_available: False
- rustc_available: False
- toolchain_path_added: False
- rust_toolchain_source: missing
- wrapper_override_required: True
- wrapper_override_used: False
- wrapper_override_env: ["RUSTC_WORKSPACE_WRAPPER", "RUSTC_WRAPPER"]
- rustc_wrapper_path_exists: False
- state_graph_present: False
- runtime_archive_log_total: 5677
- runtime_archive_download_total: 62
- runtime_archive_conversation_snapshots: 0
- delta_base_is_ancestor: True
- delta_changed_file_count: 6
- tracked_file_count: 189
- rust_file_count_src_examples: 53
- rust_test_attr_count: 103
- rust_cfg_test_count: 2
- unwrap_call_count_src_examples: 316
- expect_call_count_src_examples: 9

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
