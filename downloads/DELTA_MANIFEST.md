base_commit: b9830281da5618db55c12371ec1f17b3abdd0b00
head_commit: bac66f75055cb5238f38d8888110ea2a06ec224c

bundle: /mnt/data/repo-delta-004.bundle
bundle_range: b9830281da5618db55c12371ec1f17b3abdd0b00..bac66f75055cb5238f38d8888110ea2a06ec224c
bundle_verify: pass

commits_included:
8f635e9 Evaluate ai repository scorecard
eae2039 Observe ai repository evidence
e901518 Plan router artifact fixture gate
bac66f7 Close router artifact quality gate

changed_files:
M	.gitignore
M	IMPLEMENTATION_PLAN.md
M	ai-chromium/router-server/src/tools/validate-turn-artifacts.mjs
M	ai-chromium/router-server/test/artifact-quality.test.mjs
A	ai-chromium/router-server/test/fixtures/artifacts/malformed-evidence/turn-001/evaluation.json
A	ai-chromium/router-server/test/fixtures/artifacts/malformed-evidence/turn-001/evidence.ndjson
A	ai-chromium/router-server/test/fixtures/artifacts/malformed-evidence/turn-001/manifest.json
A	ai-chromium/router-server/test/fixtures/artifacts/malformed-evidence/turn-001/replay.json
A	ai-chromium/router-server/test/fixtures/artifacts/missing-manifest/turn-001/evaluation.json
A	ai-chromium/router-server/test/fixtures/artifacts/missing-manifest/turn-001/replay.json
A	ai-chromium/router-server/test/fixtures/artifacts/redaction-fail/turn-001/evaluation.json
A	ai-chromium/router-server/test/fixtures/artifacts/redaction-fail/turn-001/manifest.json
A	ai-chromium/router-server/test/fixtures/artifacts/redaction-fail/turn-001/replay.json
A	ai-chromium/router-server/test/fixtures/artifacts/replay-mismatch/turn-001/evaluation.json
A	ai-chromium/router-server/test/fixtures/artifacts/replay-mismatch/turn-001/manifest.json
A	ai-chromium/router-server/test/fixtures/artifacts/replay-mismatch/turn-001/replay.json
A	ai-chromium/router-server/test/fixtures/artifacts/valid/turn-001/evaluation.json
A	ai-chromium/router-server/test/fixtures/artifacts/valid/turn-001/evidence.ndjson
A	ai-chromium/router-server/test/fixtures/artifacts/valid/turn-001/manifest.json
A	ai-chromium/router-server/test/fixtures/artifacts/valid/turn-001/replay.json
M	score.md

validation_results:
- fixture JSON parse: pass, 14 JSON files parsed with python json
- node --check ai-chromium/router-server/src/server.mjs: pass
- cd ai-chromium/router-server && node src/tools/check-syntax.mjs: pass, syntax_ok files=49
- cd ai-chromium/router-server && node --test test/openai-contract.test.mjs: pass, 7/7
- cd ai-chromium/router-server && node --test test/mock-cdp-integration.test.mjs: pass, 3/3
- cd ai-chromium/router-server && node --test test/artifact-quality.test.mjs: pass, 5/5
- CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz bash scripts/observe_validation.sh: pass as observation harness; report emitted 10 valid NDJSON lines; git_status_clean=true; cargo/rustc unavailable; missing_signal_count=13
- git diff --check: pass

runtime_archive_evidence:
- /mnt/data/ai-runtime.tar.gz: 3 members; included_files=2; NDJSON log lines=2; downloadHistory=[]; conversation snapshots=0; cache files=0
- /mnt/data/router-server-runtime.tar.gz: 3 members; NDJSON log lines=2; downloadHistory=[]
- /mnt/data/chatgpt-project-agent-runtime.tar.gz: 3 members; NDJSON log lines=2; downloadHistory=[]

remaining_constraints:
- Root Rust validation not reproduced because cargo and rustc are unavailable in PATH.
- Configured rustc wrapper path is absent: /workspace/ai_sandbox/canon-rustc-v2/target/debug/canon-rustc-v2.
- No state/rustc graph.json was present or regenerated.
- No live CDP run, conversation snapshot, download history, or apply-worktree artifact was present in uploaded runtime archives.

receiver_apply_commands:

git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD

bundle_verify_output:
/mnt/data/repo-delta-004.bundle is okay
The bundle contains this ref:
bac66f75055cb5238f38d8888110ea2a06ec224c HEAD
The bundle requires this ref:
b9830281da5618db55c12371ec1f17b3abdd0b00 
The bundle uses this hash algorithm: sha1
