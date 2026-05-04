base_commit: cf2f814e32f9b30966fbbd0c71f17bb52106e556
head_commit: 50590a67a119e8b857ecd2a0e77c02f21be47fc4

# Delta Manifest

## Changed Files
- IMPLEMENTATION_PLAN.md
- score.md
- scripts/observe_validation.sh
- scripts/validate_rust_panic_surface.py
- src/capability/policy/store.rs
- src/lib.rs
- src/runtime/recovery_policy.rs
- src/runtime/reducer.rs
- tests/test_observe_validation_contract.py

## Validation Results
- validation_status: partial
- validation_command_count: 10
- validation_test_count: 55
- zero_test_reason: None
- python_unit_test_count: 10
- router_test_count: 45
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 13
- report_sha256: 3baf75bcceeb5d42a7a4b951ed90a7f74c636489783578b32383c397f4eaf184
- bundle_sha256: 97ef0c0713facad2f6a5a767c9682b804adbb9b0ec1a5f9fd52241573f670b11
- bundle_verify: pass
- bundle_heads: ["50590a67a119e8b857ecd2a0e77c02f21be47fc4 HEAD"]
- validation_report_git_head: 50590a67a119e8b857ecd2a0e77c02f21be47fc4
- changed_file_count: 9
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
- runtime_archive_sha256: 2f98ab70d1a72df9f4447a5fe2c86b2965a762f8b00be10303640c975f87a0d3
- runtime_manifest_base_commit: cf2f814e32f9b30966fbbd0c71f17bb52106e556
- runtime_archive_log_total: 12316
- runtime_archive_download_total: 230
- runtime_download_history_record_count: 184
- runtime_unique_download_alias_count: 21
- runtime_unique_download_aliases: ["DELTA_MANIFEST.md", "IMPLEMENTATION_PLAN.md", "ai-IMPLEMENTATION_PLAN.md", "ai-eval-validation-report.ndjson", "ai-implementation-plan-update.patch", "ai-observe-score-update.patch", "ai-observe-score.md", "ai-observe-validation-current.ndjson", "ai-observe-validation-output-latest.txt", "ai-observe-validation-output.txt", "ai-observe-validation-report.ndjson", "ai-router-tests-run.txt", "ai-runtime-archive-metrics.json", "ai-runtime-observe-current-summary.json", "ai-runtime-observe-summary.json", "ai-score-update.patch", "ai-score.md", "ai-validation-report-eval.ndjson", "observe-validation.ndjson", "repo-delta-004.bundle", "score.md"]
- runtime_archive_conversation_snapshots: 0
- runtime_stale_advisory_count: 1
- runtime_candidate_error_count: 0
- runtime_duplicate_artifact_aliases: 8
- delta_base_is_ancestor: True
- delta_changed_file_count: 9
- tracked_file_count: 192
- rust_file_count_src_examples: 53
- rust_test_attr_count: 103
- rust_cfg_test_count: 2
- unwrap_call_count_src_examples: 316
- expect_call_count_src_examples: 3

## Validation Commands
- git_diff_check: pass :: git diff --check
- git_delta_diff_check: pass :: git diff --check cf2f814e32f9b30966fbbd0c71f17bb52106e556..HEAD
- python_unit_tests: pass :: python3 -m unittest discover -s tests -p test_*.py
- panic_surface_validation: pass :: python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
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
- missing_panic_surface_validation: False
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
