base_commit: da1ebc2656e5d48631b07ef830c7dbec2033739b
head_commit: 8b8fb6cb424d7d26f1ddd085c6bbdc30f219638d

# Delta Manifest

## Changed Files
- README.md
- score.md
- scripts/write_delta_manifest.py

## Validation Results
- validation_status: partial
- validation_command_count: 6
- validation_test_count: 15
- router_test_count: 15
- cargo_test_count_when_available: None
- failed_required_commands: []
- missing_signal_count: 11
- report_sha256: 9ec2782ba81de44bdebff25a71a1e519e8135fab87fda8260b5b26f7aff444f4
- bundle_sha256: 7ca3d903fe7327130f169e4348467bcb613b23833f3ea4d55ffa623fd28b883b
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
