# Phase 2 Turn 3 Plan

## Variables

```text
I  = Intelligence
E  = Efficiency
C  = Correctness
A  = Alignment
R  = Robustness
P  = Performance
S  = Scalability
D  = Determinism
T  = Transparency
Co = Collaboration
Em = Empowerment
B  = Benefit
L  = Learning
Si = Simplicity
F  = Future-Proofing
G  = Goodness
```

## Equation

```text
G = (I*E*C*A*R*P*S*D*T*Co*Em*B*L*Si*F)^(1/15)
max(G) = good
```

One-line explanation: Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers the whole system.

## Source Inputs

- `GOAL.md` requires replayable receipts, external evidence, runtime archive inspection, policy learning, and deterministic validation.
- `score.md` is preserved and not edited in this phase.
- Turn 1 closed the missing default wrapper and production panic-surface blockers.
- Turn 2 bounded long-running validation commands with the requested `300s` test budget.
- Runtime archive inspection found `/mnt/data/ai-runtime.tar.gz` with `RUNTIME_MANIFEST.json`, `.repo-agent-runtime/audit.ndjson`, and `log/chatgpt_project_agent.ndjson`; it has no conversation snapshot or message/download history ledger.

## Boundary

- Base commit: `c914c14987938a0d95c7904a2ffa5164bb30f2d9`.
- Current cumulative working HEAD before this turn: `c0489a03192f8f7974591b8dc97a71336e412cea`.
- Do not edit or stage `score.md`.
- This is the final artifact turn: create `/mnt/data/repo-delta-004.bundle` and `/mnt/data/DELTA_MANIFEST.md` only after committing the new HEAD.
- Keep the cumulative bundle scoped to `B..H`, not only the latest turn.

## Tasks

1. Refresh `plan.md` from `GOAL.md`, `score.md`, current runtime archive evidence, and prior Phase 2 commits.
2. Preserve conversation/download ledger absence as explicit missing signals, but stop treating that absence as a failed runtime inspection when current-loop audit/process logs and manifest evidence are present.
3. Add a Python contract proving runtime archive inspection now records process-log and semantic-history evidence fields.
4. Run Rust bootstrap before validation with `timeout 300s`.
5. Run bounded validation with `timeout 300s` for tests and validators.
6. Commit only the cumulative repository change; leave `score.md` unstaged.
7. Recreate `/mnt/data/repo-delta-004.bundle` and `/mnt/data/DELTA_MANIFEST.md` for `c914c14987938a0d95c7904a2ffa5164bb30f2d9..H`.

## Completed This Turn

- Updated `scripts/observe_validation.sh` to count `runtime_archive_process_log_files`.
- Added `runtime_archive_current_loop_evidence_files` and `runtime_archive_semantic_history_files`.
- Changed `runtime_archive_inspection_status` so manifest + logs + prior-state audit evidence can pass inspection even when conversation snapshots are absent.
- Kept `missing_runtime_conversation_ledger` and `missing_conversation_snapshot` as separate transparency signals.
- Updated `tests/test_observe_validation_contract.py` to require the new runtime archive evidence fields.

## Validation Result

```text
timeout 300s python3 /mnt/data/bootstrap_rustc_session.py ...: pass

timeout 300s cargo check --offline: pass
timeout 300s cargo test --lib --offline: pass, 110 passed
timeout 300s cargo test --all-targets --offline: pass, 110 library tests and 0-test bin/example targets completed
timeout 300s python3 -m unittest discover -s tests -p 'test_*.py' -v: pass, 27 passed
timeout 300s python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report /mnt/data/ai-phase2-turn3-panic-surface.json: pass, production_total=0, test_total=335, example_total=1
timeout 300s python3 scripts/validate_policy_learning_trace.py --root . --report /mnt/data/ai-phase2-turn3-policy-learning-trace.json: pass, missing_count=0
timeout 300s cargo run --example loop_trace --offline: pass, 31 events, Done, success=true
timeout 300s env CANON_DELTA_BASE=... CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz CANON_OBSERVE_REPORT=/mnt/data/ai-phase2-turn3-observe.ndjson bash scripts/observe_validation.sh: timed out before validation_summary; emitted runtime_archive_metrics before timeout with runtime_archive_inspection_status=pass, runtime_archive_process_log_files=1, runtime_archive_current_loop_evidence_files=2, runtime_archive_semantic_history_files=2, runtime_manifest_base_matches_delta_base=true
git diff --check: pass
```

## Remaining Risk

- Wrapper graph telemetry is still optional and requires an explicit built `CANON_RUSTC_WRAPPER` binary.
- `cargo fmt` and `cargo clippy` remain dependent on unavailable rustfmt/clippy components in the supplied extracted toolchain.
- Live Ollama/OpenAI provider paths remain gated on explicit local/provider endpoints.
- Runtime archive conversation snapshots and message/download ledgers remain absent from the supplied archive, but current-loop manifest/audit/process-log inspection is now separated from those missing history signals.
- Large-module simplicity debt remains, especially in `src/lib.rs` and LLM adapter modules.