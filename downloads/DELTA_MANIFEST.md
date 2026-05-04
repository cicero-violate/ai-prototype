base_commit: b89bdd0766eb986d6de87887eeb991ce6f837ba5
head_commit: 1902fd15de6decaf971fd7149dcb53ba520e3241

# Delta Manifest

## Changed Files
- plan.md
- score.md
- scripts/observe_validation.sh
- scripts/write_delta_manifest.py
- tests/test_observe_validation_contract.py
- tests/test_write_delta_manifest.py

## Validation Results
- validation_status: partial
- validation_command_count: 10
- validation_test_count: 16
- zero_test_reason: None
- python_unit_test_count: 16
- router_test_count: 0
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 15
- report_sha256: c5bd5b41ff626c632a3e93a1ca7ca81f5b63e92e1c70e1b309b506a5e8dcd218
- bundle_sha256: f81be56473a2ec058dcea8057b5be509511e56edf3e6fe2092f86a480df4cca3
- bundle_verify: pass
- bundle_heads: ["1902fd15de6decaf971fd7149dcb53ba520e3241 HEAD"]
- bundle_required_refs: ["b89bdd0766eb986d6de87887eeb991ce6f837ba5"]
- bundle_requires_base_commit: True
- validation_report_git_head: 1902fd15de6decaf971fd7149dcb53ba520e3241
- changed_file_count: 6
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
- runtime_archive_sha256: 3580ea602d0ef9fc74819cf7158e1881ade58ff64593f8d991563222180c1f0a
- runtime_manifest_base_commit: b89bdd0766eb986d6de87887eeb991ce6f837ba5
- runtime_archive_log_total: 9571
- runtime_archive_download_total: 88
- runtime_performance_signal_present: True
- runtime_performance_budget_status: pass
- runtime_performance_budget_failures: {}
- runtime_performance_budgets: {"download_follow_get_ms": 60000.0, "download_initial_get_ms": 60000.0, "download_write_ms": 1000.0, "project_agent_elapsed_ms": 1800000.0}
- validation_command_duration_ms: {"count": 9, "max": 6469.0, "median": 0.0, "min": 0.0, "p95": 6469.0}
- project_agent_elapsed_ms_count: 1116
- project_agent_elapsed_ms_median: 210822.081
- project_agent_elapsed_ms_p95: 1295541.25
- project_agent_elapsed_ms_max: 1778293.765
- download_initial_get_ms_median: 15677.708
- download_initial_get_ms_p95: 17962.321
- download_initial_get_ms_max: 17962.321
- download_follow_get_ms_median: 9435.399
- download_follow_get_ms_p95: 17472.758
- download_follow_get_ms_max: 17472.758
- download_resolved_get_ms_median: 16415.166
- download_resolved_get_ms_p95: 17325.114
- download_resolved_get_ms_max: 17325.114
- download_write_ms_median: 0.189
- download_write_ms_p95: 0.395
- download_write_ms_max: 0.395
- runtime_download_history_record_count: 82
- runtime_unique_download_alias_count: 5
- runtime_unique_download_aliases: ["DELTA_MANIFEST.md", "ai-score.md", "repo-delta-001.bundle", "repo-delta-002.bundle", "score.md"]
- runtime_archive_conversation_snapshots: 0
- runtime_stale_advisory_count: 1
- runtime_manifest_base_expected: b89bdd0766eb986d6de87887eeb991ce6f837ba5
- runtime_manifest_base_matches_delta_base: True
- runtime_candidate_error_count: 0
- runtime_duplicate_artifact_aliases: 1
- delta_base_is_ancestor: True
- delta_changed_file_count: 6
- tracked_file_count: 120
- rust_file_count_src_examples: 53
- rust_test_attr_count: 103
- rust_cfg_test_count: 2
- unwrap_call_count_src_examples: 316
- expect_call_count_src_examples: 3

## Validation Commands
- git_diff_check: pass :: git diff --check
- git_delta_diff_check: pass :: git diff --check b89bdd0766eb986d6de87887eeb991ce6f837ba5..HEAD
- python_unit_tests: pass :: python3 -m unittest discover -s tests -p test_*.py
- panic_surface_validation: pass :: python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
- router_offline_tests: unavailable :: bash run_tests.sh
- cargo_fmt_check: unavailable :: cargo fmt --check
- cargo_test_all_targets: unavailable :: cargo test --all-targets
- cargo_clippy_all_targets: unavailable :: cargo clippy --all-targets -- -D warnings
- wrapper_graph_validation: skipped_env_missing :: cargo test --all-targets
- ollama_judgment_example: skipped_env_missing :: cargo run --example ollama_judgment

## Missing Signal Flags
- missing_artifact_apply_worktree: True
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
- missing_router_offline_tests: True
- missing_runtime_download_history: False
- missing_runtime_manifest_base_match: False
- missing_runtime_performance_signal: False
- missing_rustc_wrapper_telemetry: True
- missing_semantic_artifact_verification_test: True
- missing_wrapper_graph_validation: True

## Receiver Apply Commands
```bash
git fetch ./repo-delta-002.bundle HEAD && git merge --ff-only FETCH_HEAD
```
