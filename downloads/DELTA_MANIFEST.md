base_commit: 44945bf71389366f1566abf48f70a81b924fab96
head_commit: 29e3b4645207693859de2352cf1885304012dc85

# Delta Manifest

## Changed Files
- IMPLEMENTATION_PLAN.md
- ai-chromium/router-server/run_tests.sh
- score.md
- scripts/observe_validation.sh
- scripts/write_delta_manifest.py

## Validation Results
- validation_status: partial
- validation_command_count: 6
- validation_test_count: 20
- router_test_count: 20
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 12
- report_sha256: eee456d39701b868d1bb258d01aae22fb0d84ed1ae94db8828f1c2a7d13a5f01
- bundle_sha256: 7a084ba83a26e1e1d078f0631f4fe29bc01c8b2e97073de97a68426b4776ae5b
- bundle_verify: pass
- cargo_available: False
- rustc_available: False
- toolchain_path_added: False
- rust_toolchain_source: missing
- wrapper_override_required: True
- wrapper_override_used: False
- wrapper_override_env: ["RUSTC_WORKSPACE_WRAPPER", "RUSTC_WRAPPER"]
- rustc_wrapper_path_exists: False
- state_graph_present: False
- runtime_archive_log_total: 3342
- runtime_archive_download_total: 30
- runtime_archive_conversation_snapshots: 0

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
