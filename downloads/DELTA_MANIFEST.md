base_commit: 8ffd5555a409a11d23b884879071f4659e6ef7b1
head_commit: 370be5f7e5ea128066ba78dc6f5cb013721d9774

# Delta Manifest

## Changed Files
- .gitignore
- .repo-agent-runtime/upload-bundles/ai.bundle
- IMPLEMENTATION_PLAN.md
- README.md
- downloads/DELTA_MANIFEST.md
- downloads/ai-observe-validation-output.txt
- downloads/ai-router-tests-run.txt
- downloads/repo-delta-004.bundle
- score.md
- scripts/observe_validation.sh
- scripts/write_delta_manifest.py

## Validation Results
- validation_status: partial
- validation_command_count: 8
- validation_test_count: 45
- router_test_count: 45
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 12
- report_sha256: 24d000a4ba32cfd8bca8362ad43db8d2e2704e65e267daa3067b75bae60e363a
- bundle_sha256: 3f01faa296d82be4d9755b0642a0010522563022af50a8bdf3228ce434a3893f
- bundle_verify: pass
- validation_report_git_head: 370be5f7e5ea128066ba78dc6f5cb013721d9774
- changed_file_count: 11
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
- runtime_archive_sha256: 74fc89b95d610ee0f0e5bc51ce5a704b0b34f9a37d624db00084ad0e7d8edc74
- runtime_manifest_base_commit: 8ffd5555a409a11d23b884879071f4659e6ef7b1
- runtime_archive_log_total: 7883
- runtime_archive_download_total: 128
- runtime_archive_conversation_snapshots: 0
- runtime_stale_advisory_count: 1
- runtime_candidate_error_count: 0
- runtime_duplicate_artifact_aliases: 6
- delta_base_is_ancestor: True
- delta_changed_file_count: 11
- tracked_file_count: 184
- tracked_generated_artifact_count: 0
- tracked_generated_artifacts: []
- rust_file_count_src_examples: 53
- rust_test_attr_count: 103
- rust_cfg_test_count: 2
- unwrap_call_count_src_examples: 316
- expect_call_count_src_examples: 9

## Validation Commands
- git_diff_check: pass :: git diff --check
- git_delta_diff_check: pass :: git diff --check 8ffd5555a409a11d23b884879071f4659e6ef7b1..HEAD
- tracked_generated_artifact_gate: pass :: local tracked_generated_artifact_gate
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
- missing_tracked_generated_artifact_hygiene: False

## Receiver Apply Commands
```bash
git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD
```
