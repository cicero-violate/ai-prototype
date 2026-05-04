base_commit: 9f15edff4a67b82a38d83209829e9b4311e69fee
head_commit: c4c5a254063e54b7715d9cefa9105b2ecbd26363

# Delta Manifest

## Changed Files
- .gitignore
- .repo-agent-runtime/apply-worktrees/07ad58b4bf0e
- .repo-agent-runtime/apply-worktrees/12f02b6a0f39
- .repo-agent-runtime/apply-worktrees/b9830281da56
- .repo-agent-runtime/apply-worktrees/d47aaa7d3487
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
- validation_command_count: 7
- validation_test_count: 45
- router_test_count: 45
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 12
- report_sha256: 0cb1908f4171aba87fbcd09f8efbe66f36bf5f3d280cd294794bc6a4f1c8060e
- bundle_sha256: 738786cba45a8b85270e2b8eb83b34aec6beb4a67a45b47f209d9b150c59f44b
- bundle_verify: pass
- validation_report_git_head: c4c5a254063e54b7715d9cefa9105b2ecbd26363
- changed_file_count: 15
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
- runtime_archive_present: True
- runtime_archive_sha256: d914f6597dda00ea6eb5b6b6cd8a77980891325a3cd476116c4233d7386f495d
- runtime_manifest_base_commit: 9f15edff4a67b82a38d83209829e9b4311e69fee
- runtime_manifest_head_commit: None
- runtime_download_history_by_classification: {"delta_applied": 11, "download_event": 14, "download_ledger": 61, "live_evidence": 10, "stale_advisory": 1}
- runtime_excluded_by_reason: {"apply-worktree": 2451, "generated-bundle": 20, "generated-runtime-archive": 1, "secret-token-cache": 1, "signed-url-cache": 11}
- runtime_secret_token_cache_excluded: True
- runtime_signed_url_cache_excluded: True
- runtime_generated_bundle_excluded: True
- runtime_apply_worktree_excluded: True
- runtime_leak_scan_finding_count: 0
- runtime_schema_scan_finding_count: 0
- runtime_integrity_scan_finding_count: 0
- runtime_archive_log_total: 7212
- runtime_archive_download_total: 81
- runtime_archive_conversation_snapshots: 0
- runtime_stale_advisory_count: 1
- runtime_candidate_error_count: 0
- runtime_duplicate_artifact_aliases: 6
- delta_base_is_ancestor: True
- delta_changed_file_count: 15
- tracked_file_count: 180
- rust_file_count_src_examples: 53
- rust_test_attr_count: 103
- rust_cfg_test_count: 2
- unwrap_call_count_src_examples: 316
- expect_call_count_src_examples: 9

## Validation Commands
- git_diff_check: pass :: git diff --check
- git_delta_diff_check: pass :: git diff --check 9f15edff4a67b82a38d83209829e9b4311e69fee..HEAD
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
- missing_runtime_secret_cache_exclusion: False
- missing_runtime_signed_url_cache_exclusion: False
- missing_rustc_wrapper_telemetry: True
- missing_semantic_artifact_verification_test: True

## Receiver Apply Commands
```bash
git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD
```
