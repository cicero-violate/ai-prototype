base_commit: c2f34ba6de533809414f6b11cccdd6592926d619
head_commit: 4b4cf034d3fd762cd57f4b27a5171d803b3ec6d9

# Delta Manifest

## Changed Files
- IMPLEMENTATION_PLAN.md
- score.md
- scripts/observe_validation.sh
- scripts/write_delta_manifest.py

## Validation Results
- validation_status: partial
- validation_command_count: 6
- validation_test_count: 41
- router_test_count: 41
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 12
- report_sha256: 5988b12fb96f61860d4c4d0c63d2abfa649d40b9daaa486608c5d7728fab2a1c
- bundle_sha256: 9b6ba5f721ded0cda664a3f2bc444e53f28ade73b055dcd76d1996f7b72b020c
- bundle_verify: pass
- changed_file_count: 4
- cargo_available: False
- rustc_available: False
- toolchain_path_added: False
- rust_toolchain_source: missing
- wrapper_override_required: True
- wrapper_override_used: False
- wrapper_override_env: ["RUSTC_WORKSPACE_WRAPPER", "RUSTC_WRAPPER"]
- rustc_wrapper_path_exists: False
- state_graph_present: False
- runtime_archive_log_total: 4516
- runtime_archive_download_total: 38
- runtime_archive_conversation_snapshots: 0
- delta_base_is_ancestor: True
- delta_changed_file_count: 4
- tracked_file_count: 188
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
