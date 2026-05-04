base_commit: 3f23e0884e9d8d0189f91fde8ee76244bc25a82f
head_commit: e870106ebbbe1700887f9bb6cb63344805f51c87

# Delta Manifest

## Changed Files
- .gitignore
- .repo-agent-runtime/apply-worktrees/07ad58b4bf0e
- .repo-agent-runtime/apply-worktrees/12f02b6a0f39
- .repo-agent-runtime/apply-worktrees/b9830281da56
- .repo-agent-runtime/apply-worktrees/d47aaa7d3487
- .repo-agent-runtime/upload-bundles/ai.bundle
- IMPLEMENTATION_PLAN.md
- downloads/DELTA_MANIFEST.md
- downloads/ai-observe-validation-output.txt
- downloads/ai-router-tests-run.txt
- downloads/repo-delta-004.bundle
- score.md
- scripts/observe_validation.sh
- scripts/validate_delta_artifacts.py
- scripts/write_delta_manifest.py

## Validation Results
- validation_status: partial
- validation_command_count: 6
- validation_test_count: 41
- router_test_count: 41
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 12
- report_sha256: b40a87b8df33047aecabf283638144d597be3c9e1396ea7842e0cf6bee438d7b
- bundle_size_bytes: 12025
- bundle_sha256: 7ca2ef8d872097e5cfdd6ee29258421aa9b5eef0e0f12dea49ce2e52883fc502
- bundle_verify: pass
- changed_file_count: 15
- cargo_available: False
- rustc_available: False
- toolchain_path_added: False
- rust_toolchain_source: missing
- wrapper_override_required: True
- wrapper_override_used: False
- wrapper_override_env: ["RUSTC_WORKSPACE_WRAPPER", "RUSTC_WRAPPER"]
- rustc_wrapper_path_exists: False
- state_graph_present: False
- runtime_archive_log_total: 5139
- runtime_archive_download_total: 53
- runtime_archive_conversation_snapshots: 0
- delta_base_is_ancestor: True
- delta_changed_file_count: 15
- tracked_file_count: 180
- tracked_generated_artifact_count: 0
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
- tracked_generated_artifacts: False

## Tracked Generated Artifacts
- none

## Receiver Apply Commands
```bash
git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD
```
