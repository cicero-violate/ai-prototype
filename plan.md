# Canon Agent Implementation Plan

## Current State

Canon Agent is a Rust prototype for a deterministic, auditable, self-improving agent runtime. The target architecture remains a formally constrained state-machine kernel with a capability layer around it. The kernel owns correctness, state transitions, durable records, replay boundaries, and audit evidence. LLMs and tools operate inside the capability layer and must produce typed, reviewable evidence rather than governing the runtime directly.

Current implementation snapshot, 2026-05-09 00:15:21 EDT America/Toronto / 2026-05-09T04:15:21Z UTC:

- Branch: `main`.
- Latest visible prior commit before this planning turn: `b76a0db Clarify loop driver retry evidence`.
- The requested working directory resolves to the connector workspace root: `/workspace/ai_sandbox/canon-mini-agent/prototype/ai`.
- This is a planning/scoring turn. No implementation files are in scope.
- The pre-existing implementation batch has now been reviewed at diff level and validated.
- P0 validation baseline is closed with fresh all-target, fmt, lib-test, and clippy evidence.
- Connector 502s occurred during long-running validation polling, but redirected/detached validation evidence completed with exit files and final test summaries.

Fresh validation evidence from implementation step 1:

- `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` passed through detached redirected execution.
  - exit: `0`
  - log: `target/validation-logs/test-all-detached.log`
  - exit file: `target/validation-logs/test-all-detached.exit`
  - evidence: 22 `test result:` sections, including `191 passed`, integration suites, `352 passed` for `validation_harness_contract`, worker process contract, and example test binaries.
- `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check` passed.
  - exit: `0`
  - log: `target/validation-logs/fmt-exec-step-1.log`
- `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib -- --test-threads=1` passed.
  - exit: `0`
  - result: `191 passed; 0 failed`
  - log: `target/validation-logs/test-lib-exec-step-1.log`
- `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings` passed.
  - exit: `0`
  - log: `target/validation-logs/clippy-exec-step-1.log`

## Operating Rules For Agent Turns

1. **Planning/scoring turns**
   - Update `plan.md` and `score.md` only.
   - Do not modify implementation files.
   - Commit planning/scoring changes as a standalone commit when possible.
   - If unrelated implementation files are already dirty, stage only `plan.md` and `score.md`.

2. **Execution turns**
   - Work the highest-priority incomplete item below.
   - Keep generated artifacts out of git.
   - Update `score.md` with commands run, exit status, and failure classification.
   - Commit only intentional source/docs/test changes.

3. **Commit hygiene**
   - Before editing: inspect `git status --short`.
   - Before committing: inspect the exact intended diff.
   - Stage by explicit path, not broad `git add .` unless the turn intentionally owns all modified paths.
   - Keep generated logs, target output, runtime archives, tokens, and SSE chunks out of git.

## Implementation Priority

### P0 — Finish validation baseline — complete

1. **Implementation batch review**
   - The dirty implementation batch was inspected at diff level before validation.
   - Diff shape is primarily validation-harness/reporting, receipt/proof metadata, record construction, API/transport verification, and test initialization hardening.
   - No implementation files were reverted during this execution step.

2. **Validation gate**
   - All-target validation now has final exit evidence and final `test result:` summaries.
   - The detached all-target run completed with exit `0` after connector 502s interrupted streaming/polling attempts.
   - The previous incomplete all-target evidence is superseded by `target/validation-logs/test-all-detached.log` and `.exit`.

3. **Complete baseline refresh**
   - Format, lib tests, and clippy have been refreshed and passed with exit `0`.
   - Correctness, robustness, and determinism score ceilings may be raised from this point.

### P1 — Make validation evidence first-class — complete

1. Added source-backed observe-validation reporting under ignored `target/observe/validation-report.ndjson` via `scripts/observe_validation.sh`.
2. The compact report now emits git hygiene, command outcomes, graph telemetry presence/absence, runtime archive counts, ignored artifact counts, missing-signal flags, and connector-failure classification fields.
3. Restored source-backed helper validators and manifest writer scripts so validation evidence can be generated and consumed from tracked code instead of stale pycache artifacts.
4. Added contract coverage for connector-failure classification and ignored artifact counts.
5. Validation evidence from implementation step 2:
   - `python3 -m unittest tests/test_observe_validation_contract.py tests/test_panic_surface_contract.py tests/test_policy_learning_trace_contract.py tests/test_write_delta_manifest.py` passed; 29 tests.
   - `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check` passed.
   - `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib -- --test-threads=1` passed; 191 tests.
   - `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings` passed.
6. Observe smoke evidence emitted connector and missing-signal fields, but the full all-target command inside the smoke failed due environment quota pressure (`Disk quota exceeded` in the cargo-test log), not a semantic assertion failure.

### P2 — Agent loop reliability — complete

1. Hardened streaming retry behavior and evidence preservation for the SSE/router boundary.
   - `finish_reason=length` is now classified as `length_finished`, incomplete, and never retry-safe.
   - Missing `[DONE]` is retry-safe only for a new chat without a target URL.
   - Missing `message_stream_complete` is retry-safe only for a new chat without a target URL.
   - Any observed target URL blocks retry so the runtime preserves thread/evidence continuity instead of creating duplicate work.
2. Added SSE fixtures for:
   - complete stream with `[DONE]` and `message_stream_complete`
   - truncated stream missing `[DONE]`
   - stream missing `message_stream_complete`
   - stream with target URL but missing stream-complete metadata
   - `finish_reason=length`
   - partial chunked transfer body preservation
3. Clarified project-loop mode versus phase-driven worker certification in the loop-driver boundary comments and production logs.
4. Added loop-driver fixtures for:
   - retry attempt labels preserving original labels and numbering retries
   - project-planning/project-execution/worker-certification mode classification
   - project prompts staying separate from AgentCycle certification
   - certification objective truncation preserving compact project evidence
5. Validation evidence from implementation step 5:
   - `cargo test agent::loop_driver::tests --lib -- --test-threads=1` passed; 4 tests.
   - `cargo fmt --check` passed.
   - `cargo test --lib -- --test-threads=1` passed; 201 tests.
   - `cargo clippy --all-targets -- -D warnings` passed.

### P3 — Runtime and receipt correctness — next execution target

1. Expand replay and receipt invariants for forged, duplicated, reordered, stale, and missing receipts.
   - Primary source surface discovered during this planning turn: `src/recovery.rs` for validation receipts and `src/validation_harness.rs` for receipt hashing/typed evidence contracts.
   - First execution slice should add focused failing/passing tests before changing receipt logic.
   - Required negative cases: forged receipt identity/hash, duplicated receipt, reordered receipt sequence, stale receipt, and missing receipt.
   - Required success case: valid receipt chain/replay remains accepted and produces compact evidence.
2. Keep policy promotion externally verified; never let the LLM approve its own candidates.
   - Any future policy-learning admission must remain gated by external evaluator evidence.
   - LLM-authored candidates may propose changes but cannot self-certify them into learning data.
3. Tighten API/worker compatibility for batch command limits, invalid envelopes, durable resume behavior, and supervisor reload.
   - Treat these as follow-on P3 slices after the receipt/replay invariant tests are established.

### P4 — Graph source-of-truth integration

1. Keep graph telemetry observable but optional unless wrapper variables are configured.
2. Verify graph mutation ops, patch receipts, snapshot contracts, and landing receipts as one end-to-end flow.
3. Clarify `canon-rustc-v3` and `graph-editor` repository/subproject boundaries.

### P5 — Domain intelligence layer

1. Keep domain docs unwired until contracts are stable.
2. Define record contracts before behavior.
3. Keep trading separate from production business automation.

## Next Execute-Turn Recommendation

1. Begin P3 with targeted receipt/replay invariant tests.
2. Inspect `src/recovery.rs` first, then locate the narrowest existing test module or add one near the receipt/replay code.
3. Add tests for forged, duplicated, reordered, stale, missing, and valid receipt-chain behavior.
4. Only then adjust implementation logic needed to make the invariant set pass.
5. Continue using `TMPDIR="$PWD/target/test-tmp"` for Rust tests in quota-sensitive sandboxes.
6. Run targeted receipt/replay tests, `cargo fmt --check`, `cargo test --lib -- --test-threads=1`, and `cargo clippy --all-targets -- -D warnings` after the next P3 slice.

## Planning-Turn Handoff

P0, P1, and P2 are complete. This planning turn keeps implementation unchanged and directs the next execution turn to P3 runtime and receipt correctness, beginning with replay/receipt invariant expansion.

## Current Non-Goals

- Do not wire domain specs into runtime behavior yet.
- Do not redesign the kernel.
- Do not add live trading behavior.
- Do not rely on external LLM/Ollama/OpenAI availability for baseline correctness.
- Do not commit token caches, runtime archives, target outputs, validation logs, or SSE chunk logs.
