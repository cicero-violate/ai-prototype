base_commit: 181519a9fd945532cc6de825c71d197dd01e65ec
head_commit: 64e1cac2e0d4146c9dc810612eed5d2f48f9ebd2

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
- report_sha256: 1e6ea9c10470aadda64cce25610a757415b4e8a7450da62a9a84d7c3aa0f3b9a
- bundle_sha256: 784b153e50458cf01ae052e0386546ca93a953472a42b119194228f7c391ff08
- bundle_verify: pass
- validation_report_git_head: 64e1cac2e0d4146c9dc810612eed5d2f48f9ebd2
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
- runtime_archive_sha256: 2ccdf19e623a6be95924aac7ad0b2d696923ef96f42a895917ec73693f5c868a
- runtime_manifest_base_commit: 181519a9fd945532cc6de825c71d197dd01e65ec
- runtime_archive_log_total: 8416
- runtime_archive_download_total: 133
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
- git_delta_diff_check: pass :: git diff --check 181519a9fd945532cc6de825c71d197dd01e65ec..HEAD
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
