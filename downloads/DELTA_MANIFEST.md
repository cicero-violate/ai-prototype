base_commit: 12f02b6a0f3936ab5ffd8ff59762b71084c443fb
head_commit: 9a23cea59f874bd988cf75eee0210fb30ccb99b7

bundle: repo-delta-004.bundle
bundle_command: git bundle create /mnt/data/repo-delta-004.bundle 12f02b6a0f3936ab5ffd8ff59762b71084c443fb..9a23cea59f874bd988cf75eee0210fb30ccb99b7 HEAD
bundle_range: 12f02b6a0f3936ab5ffd8ff59762b71084c443fb..9a23cea59f874bd988cf75eee0210fb30ccb99b7
bundle_exported_ref: HEAD
bundle_verify: pass

changed_files:
M	IMPLEMENTATION_PLAN.md
M	ai-chromium/router-server/src/tools/validate-turn-artifacts.mjs
D	ai-chromium/router-server/test/fixtures/artifacts/malformed-evidence/turn_malformed/evaluation.json
D	ai-chromium/router-server/test/fixtures/artifacts/malformed-evidence/turn_malformed/manifest.json
D	ai-chromium/router-server/test/fixtures/artifacts/malformed-evidence/turn_malformed/replay.json
D	ai-chromium/router-server/test/fixtures/artifacts/missing-manifest/turn_missing_manifest/evaluation.json
D	ai-chromium/router-server/test/fixtures/artifacts/missing-manifest/turn_missing_manifest/replay.json
D	ai-chromium/router-server/test/fixtures/artifacts/redaction-fail/turn_redaction_fail/evaluation.json
D	ai-chromium/router-server/test/fixtures/artifacts/redaction-fail/turn_redaction_fail/manifest.json
D	ai-chromium/router-server/test/fixtures/artifacts/redaction-fail/turn_redaction_fail/replay.json
D	ai-chromium/router-server/test/fixtures/artifacts/replay-mismatch/turn_replay_mismatch/evaluation.json
D	ai-chromium/router-server/test/fixtures/artifacts/replay-mismatch/turn_replay_mismatch/manifest.json
D	ai-chromium/router-server/test/fixtures/artifacts/replay-mismatch/turn_replay_mismatch/replay.json
D	ai-chromium/router-server/test/fixtures/artifacts/valid/turn_pass/action-receipts.ndjson
D	ai-chromium/router-server/test/fixtures/artifacts/valid/turn_pass/evaluation.json
D	ai-chromium/router-server/test/fixtures/artifacts/valid/turn_pass/manifest.json
D	ai-chromium/router-server/test/fixtures/artifacts/valid/turn_pass/replay.json
M	score.md

commits:
19e017c Evaluate ai repository scorecard
18db8c1 Observe ai repository evidence
7261f1c Plan router artifact quality fix
9a23cea Close router artifact quality gate

validation_results:
- node --version: v22.16.0
- node src/tools/check-syntax.mjs: pass, syntax_ok files=49
- node --check src/tools/validate-turn-artifacts.mjs: pass
- node --check src/server.mjs: pass
- node --test test/openai-contract.test.mjs: pass, 7/7
- node --test test/mock-cdp-integration.test.mjs: pass, 3/3
- node --test test/artifact-quality.test.mjs: pass, 5/5
- CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz bash scripts/observe_validation.sh: pass, valid_ndjson_lines=10, git_status_clean=true
- observe summary: cargo_available=false, cargo_fmt_check=unavailable, cargo_test=unavailable, clippy=unavailable, rustc_wrapper_path_exists=false, state_graph_present=false, runtime_archive_present=true, runtime_archive_log_total=640, runtime_archive_download_total=6, missing_signal_count=11
- git diff --check: pass

runtime_archive_evidence_inspected:
- archive: /mnt/data/ai-runtime.tar.gz
- member_count: 13
- download_records: 6
- message_snapshot_records: 314
- candidate_ledger_records: 3
- audit_records: 11
- process_log_records: 55
- network_request_records_total: 251
- stale_advisory_download_history_count: 1

receiver_apply_commands:
git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD
