base_commit: d47aaa7d3487521df1d0b5d2ee11310934e9a359
head_commit: 54cf9b3fac63da09e12f4ffbcd2b9383924ad0cf

# Delta Manifest

## Changed Files
- IMPLEMENTATION_PLAN.md
- score.md
- scripts/observe_validation.sh
- scripts/write_delta_manifest.py

## Validation Results
- validation_status: partial
- validation_command_count: 6
- validation_test_count: 15
- router_test_count: 15
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 11
- report_path: target/observe/validation-report.ndjson
- report_sha256: 4b09b2d0f0c8bed69bf2871c3b8b64776aab283ff73d1e21b9c1672027d26294
- bundle_sha256: f15bd8fdf7f641aacf97328ace05b766ce5f2aba37e484a36a9547d6ab5ad488
- bundle_verify: pass

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
- missing_runtime_download_history: False
- missing_rustc_wrapper_telemetry: True
- missing_semantic_artifact_verification_test: True

## Receiver Apply Commands
```bash
git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD
```
