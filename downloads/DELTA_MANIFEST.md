base_commit: 07ad58b4bf0e41e087a4584ecc97cd778a416a29
head_commit: 8fe6feca9a50fe7a97c004ad333b995da0c784f2

bundle: repo-delta-004.bundle
bundle_range: 07ad58b4bf0e41e087a4584ecc97cd778a416a29..8fe6feca9a50fe7a97c004ad333b995da0c784f2
bundle_create_command: git bundle create /mnt/data/repo-delta-004.bundle 07ad58b4bf0e41e087a4584ecc97cd778a416a29..HEAD

changed_files:
- IMPLEMENTATION_PLAN.md
- ai-chromium/router-server/run_tests.sh
- score.md

commits:
- 2f261eb Evaluate ai repository scorecard
- 107eb86 Observe ai repository evidence
- d0041c2 Plan router validation wrapper fix
- 8fe6fec Fix router validation wrapper

validation_results:
- node --version: v22.16.0
- node --check ai-chromium/router-server/src/server.mjs: pass
- cd ai-chromium/router-server && ./run_tests.sh: pass; syntax_ok files=49; 15 offline tests passed with dot reporter
- bash scripts/observe_validation.sh: pass as evidence emitter; root Rust validation unavailable because cargo is not in PATH
- target/observe/validation-report.ndjson parse: pass; valid_ndjson_lines=10
- runtime archive metrics inspection: pass with Python JSON/NDJSON parsing for ai-runtime.tar.gz, router-server-runtime.tar.gz, chatgpt-project-agent-runtime.tar.gz
- git diff --check: pass
- git bundle verify /mnt/data/repo-delta-004.bundle: pass

remaining_validation_gaps:
- cargo fmt --check: unavailable; cargo not found in PATH
- cargo test --all-targets: unavailable; cargo not found in PATH
- cargo clippy --all-targets -- -D warnings: unavailable; cargo not found in PATH
- configured rustc wrapper path missing: /workspace/ai_sandbox/canon-rustc-v2/target/debug/canon-rustc-v2
- state/rustc graph telemetry absent
- live CDP router test not run
- Ollama judgment example skipped because live Ollama env vars are missing

receiver_apply_commands:
```bash
git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD
```
