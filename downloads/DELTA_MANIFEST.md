base_commit: 3d0c73cd5770487808d9b644a384697bc249fcae
head_commit: dcab364f54cd141518b4265b40a5dea299ddfc2d

# Delta Manifest

## Changed Files
- .cargo/config.toml
- IMPLEMENTATION_PLAN.md
- README.md
- score.md
- scripts/observe_validation.sh
- scripts/write_delta_manifest.py
- tests/test_observe_validation_contract.py

## Validation Results
- validation_status: partial
- validation_command_count: 9
- validation_test_count: 54
- zero_test_reason: None
- python_unit_test_count: 9
- router_test_count: 45
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 13
- report_sha256: 254604b9352c11e05664035cca72c626d1c9e198ac144350d7eed2334a718d24
- bundle_sha256: eae4063151f256ba0e61c9e7879208a43b3701a458df2296794eb8401c74920d
- bundle_verify: pass
- bundle_heads: ["dcab364f54cd141518b4265b40a5dea299ddfc2d HEAD"]
- validation_report_git_head: dcab364f54cd141518b4265b40a5dea299ddfc2d
- changed_file_count: 7
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
- runtime_archive_sha256: b2578606919ef545e5bda7a87ddef2b362d5913ba1f796a9e138b4f17ecbbae5
- runtime_manifest_base_commit: 3d0c73cd5770487808d9b644a384697bc249fcae
- runtime_archive_log_total: 10979
- runtime_archive_download_total: 202
- runtime_download_history_record_count: 174
- runtime_unique_download_alias_count: 19
- runtime_unique_download_aliases: ["DELTA_MANIFEST.md", "IMPLEMENTATION_PLAN.md", "ai-IMPLEMENTATION_PLAN.md", "ai-eval-validation-report.ndjson", "ai-implementation-plan-update.patch", "ai-observe-score-update.patch", "ai-observe-score.md", "ai-observe-validation-current.ndjson", "ai-observe-validation-output-latest.txt", "ai-observe-validation-output.txt", "ai-observe-validation-report.ndjson", "ai-router-tests-run.txt", "ai-runtime-observe-current-summary.json", "ai-runtime-observe-summary.json", "ai-score-update.patch", "ai-score.md", "ai-validation-report-eval.ndjson", "repo-delta-004.bundle", "score.md"]
- runtime_archive_conversation_snapshots: 0
- runtime_stale_advisory_count: 1
- runtime_candidate_error_count: 0
- runtime_duplicate_artifact_aliases: 8
- delta_base_is_ancestor: True
- delta_changed_file_count: 7
- tracked_file_count: 191
- rust_file_count_src_examples: 53
- rust_test_attr_count: 103
- rust_cfg_test_count: 2
- unwrap_call_count_src_examples: 316
- expect_call_count_src_examples: 9

## Validation Commands
- git_diff_check: pass :: git diff --check
- git_delta_diff_check: pass :: git diff --check 3d0c73cd5770487808d9b644a384697bc249fcae..HEAD
- python_unit_tests: pass :: python3 -m unittest discover -s tests -p test_*.py
- router_offline_tests: pass :: bash run_tests.sh
- cargo_fmt_check: unavailable :: cargo fmt --check
- cargo_test_all_targets: unavailable :: cargo test --all-targets
- cargo_clippy_all_targets: unavailable :: cargo clippy --all-targets -- -D warnings
- wrapper_graph_validation: skipped_env_missing :: cargo test --all-targets
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
- missing_wrapper_graph_validation: True

## Receiver Apply Commands
```bash
git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD
```
