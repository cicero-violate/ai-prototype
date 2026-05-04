base_commit: dce0d671d547613915f35f00926b45d34bd821c6
head_commit: c8005dd47357c47350a61fe2faf80c1050b1c165

# Delta Manifest

## Changed Files
- IMPLEMENTATION_PLAN.md
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
- report_sha256: 2df19fd38a6824e79cba6461f0e8964e02c1e84e5e3fcaf891a6e1784d19ecff
- bundle_sha256: de2db2537812b1d3f7366d7dafdc1d4f4a678d3562cb4f7f04cf03b1d1d46ea4
- bundle_verify: pass
- bundle_heads: ["c8005dd47357c47350a61fe2faf80c1050b1c165 HEAD"]
- validation_report_git_head: c8005dd47357c47350a61fe2faf80c1050b1c165
- changed_file_count: 4
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
- runtime_archive_sha256: 9cb696748f26b34176114bfcfb02fa0f9cac896b84764d49a4886caedb70524e
- runtime_manifest_base_commit: dce0d671d547613915f35f00926b45d34bd821c6
- runtime_archive_log_total: 9760
- runtime_archive_download_total: 167
- runtime_download_history_record_count: 149
- runtime_unique_download_alias_count: 17
- runtime_unique_download_aliases: ["DELTA_MANIFEST.md", "IMPLEMENTATION_PLAN.md", "ai-IMPLEMENTATION_PLAN.md", "ai-eval-validation-report.ndjson", "ai-implementation-plan-update.patch", "ai-observe-score-update.patch", "ai-observe-score.md", "ai-observe-validation-output-latest.txt", "ai-observe-validation-output.txt", "ai-observe-validation-report.ndjson", "ai-router-tests-run.txt", "ai-runtime-observe-summary.json", "ai-score-update.patch", "ai-score.md", "ai-validation-report-eval.ndjson", "repo-delta-004.bundle", "score.md"]
- runtime_archive_conversation_snapshots: 0
- runtime_stale_advisory_count: 1
- runtime_candidate_error_count: 0
- runtime_duplicate_artifact_aliases: 8
- delta_base_is_ancestor: True
- delta_changed_file_count: 4
- tracked_file_count: 189
- rust_file_count_src_examples: 53
- rust_test_attr_count: 103
- rust_cfg_test_count: 2
- unwrap_call_count_src_examples: 316
- expect_call_count_src_examples: 9

## Validation Commands
- git_diff_check: pass :: git diff --check
- git_delta_diff_check: pass :: git diff --check dce0d671d547613915f35f00926b45d34bd821c6..HEAD
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
