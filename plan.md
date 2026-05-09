# Canon Agent Implementation Plan

## Current State

Canon Agent is a Rust prototype for a deterministic, auditable, self-improving agent runtime. The target architecture remains a formally constrained state-machine kernel with a capability layer around it. The kernel owns correctness, state transitions, durable records, replay boundaries, and audit evidence. LLMs and tools operate inside the capability layer and must produce typed, reviewable evidence rather than governing the runtime directly.

Current implementation snapshot, 2026-05-09 00:54:25 EDT America/Toronto / 2026-05-09T04:54:25Z UTC:

- Branch: `main`.
- Latest visible prior commit before this implementation turn: `723e7f7 Add supervisor reload replay continuity test`.
- The requested working directory resolves to the connector workspace root: `/workspace/ai_sandbox/canon-mini-agent/prototype/ai`.
- This implementation step added persisted validation-report evidence for compact receipt replay classifications.
- The observe-validation summary row now records the receipt replay classification inventory, source evidence files, source evidence tokens, and a missing-signal flag for classification report availability.
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

### P3 — Runtime and receipt correctness — in progress

1. Expand replay and receipt invariants for forged, duplicated, reordered, stale, and missing receipts — consumer integration complete.
   - Primary source surface discovered during this planning turn: `src/recovery.rs` for validation receipts and `src/validation_harness.rs` for receipt hashing/typed evidence contracts.
   - Added a deterministic `ReceiptChainEntry` / `ReceiptReplayReport` verifier in `src/recovery.rs`.
   - Added compact `ReceiptReplayFailure` classifications for `forged_receipt`, `duplicated_receipt`, `reordered_receipt`, `stale_receipt`, and `missing_receipt`.
   - Added focused recovery tests for valid replay plus forged receipt hash, forged run identity, duplicated receipt, reordered receipts, stale extra receipts, missing receipts, and compact classification strings.
   - Validation evidence from implementation step 1:
     - `cargo fmt --check` passed; exit `0`; log `target/validation-logs/fmt-step1.log`.
     - `cargo test recovery::tests --lib -- --test-threads=1` passed; 8 tests; exit `0`; log `target/validation-logs/recovery-tests-step1.log`.
     - `cargo test --lib -- --test-threads=1` passed; 209 tests; exit `0`; log `target/validation-logs/test-lib-step1.log`.
     - `cargo clippy --all-targets -- -D warnings` passed; exit `0`; log `target/validation-logs/clippy-step1.log`.
   - Environment note: broad validation returned a connector-level `502`, but redirected logs and exit files showed lib tests and clippy completed with exit `0`.
   - Implementation step 2 wired receipt-chain verification into the durable API transport receipt consumer.
   - Added `verify_api_transport_receipt_chain` and `verify_api_transport_receipt_chain_with_expected_count` so API transport ledgers expose compact replay reports and optional expected-count missing-tail checks.
   - Added `api_transport_receipt_replay_classification` and `api_transport_receipt_replay_classification_with_expected_count` so consumer-facing failures expose `forged_receipt`, `duplicated_receipt`, `reordered_receipt`, `stale_receipt`, and `missing_receipt` strings.
   - Existing `verify_api_transport_receipts` now uses the receipt-chain verifier and maps compact receipt failures to stable `CanonError` values.
   - Added API transport contract coverage for valid receipt-chain replay plus missing, stale, duplicated, reordered, and forged API transport receipt ledgers.
   - Validation evidence from implementation step 2:
     - `cargo test --test api_transport_contract -- --test-threads=1` passed; 19 tests; exit `0`; log `target/validation-logs/api-transport-contract-step2.log`.
     - `cargo fmt --check` passed; exit `0`; log `target/validation-logs/fmt-step2.log`.
     - `cargo test --lib -- --test-threads=1` passed; 209 tests; exit `0`; log `target/validation-logs/test-lib-step2.log`.
     - `cargo clippy --all-targets -- -D warnings` passed; exit `0`; log `target/validation-logs/clippy-step2.log`.
   - Environment note: broad validation and clippy streaming returned connector-level `502`, but redirected exit files and logs showed completed commands with exit `0`.
   - Validation evidence from current implementation step:
     - `cargo test --test api_transport_contract -- --test-threads=1` passed; 19 tests; exit `0`; log `target/validation-logs/api-transport-contract-impl-step1.log`.
     - `cargo fmt --check` passed; exit `0`; log `target/validation-logs/fmt-impl-step1.log`.
     - `cargo test --lib -- --test-threads=1` passed; 209 tests; exit `0`; log `target/validation-logs/test-lib-impl-step1.log`.
     - `cargo clippy --all-targets -- -D warnings` passed; exit `0`; log `target/validation-logs/clippy-impl-step1.log`.
2. Keep policy promotion externally verified; never let the LLM approve its own candidates.
   - Any future policy-learning admission must remain gated by external evaluator evidence.
   - LLM-authored candidates may propose changes but cannot self-certify them into learning data.
3. Tighten API/worker compatibility for batch command limits, invalid envelopes, durable resume behavior, and supervisor reload.
   - Added route-level worker API contract coverage for oversized `SubmitEvidenceBatch` payloads exceeding `API_COMMAND_BATCH_LIMIT`.
   - Added route-level worker API contract coverage for malformed batch payloads that decode incorrectly.
   - Added route-level worker API contract coverage for tampered batch envelope hashes.
   - Each new test asserts `400 BAD_REQUEST`, unchanged worker state snapshot, and no durable TLog write.
   - Validation evidence from current implementation step:
     - `cargo test --test api_server_contract -- --test-threads=1` passed; 8 tests; exit `0`; log `target/validation-logs/api-server-contract-impl-step2.log`.
     - `cargo fmt --check` passed; exit `0`; log `target/validation-logs/fmt-impl-step2.log`.
     - `cargo test --lib -- --test-threads=1` passed; 209 tests; exit `0`; log `target/validation-logs/test-lib-impl-step2.log`.
     - `cargo clippy --all-targets -- -D warnings` passed; exit `0`; log `target/validation-logs/clippy-impl-step2.log`.
   - Added route-level worker API durable-resume coverage that rebuilds a session from a persisted TLog through `resume_durable_runtime` and verifies duplicate command replay without appending disk state.
   - Validation evidence from current implementation step:
     - `cargo test --test api_server_contract -- --test-threads=1` passed; 9 tests; exit `0`; log `target/validation-logs/api-server-contract-impl-step3.log`.
     - `cargo fmt --check` passed; exit `0`; log `target/validation-logs/fmt-impl-step3.log`.
     - `cargo test --lib -- --test-threads=1` passed; 209 tests; exit `0`; log `target/validation-logs/test-lib-impl-step3.log`.
     - `cargo clippy --all-targets -- -D warnings` passed; exit `0`; log `target/validation-logs/clippy-impl-step3.log`.
   - Added supervisor reload edge coverage that submits a command to generation 1, reloads to generation 2, and resubmits the same command to verify replay continuity across workers.
   - The reload test now asserts generation 2 sees the same TLog length, returns `replayed`, preserves original event sequence/hash, and does not grow the durable TLog file.
   - Validation evidence from current implementation step:
     - `cargo test --test supervisor_binary_contract -- --test-threads=1` passed; 2 tests; exit `0`; log `target/validation-logs/supervisor-binary-contract-impl-step4.log`.
     - `cargo fmt --check` passed; exit `0`; log `target/validation-logs/fmt-impl-step4.log`.
     - `cargo test --lib -- --test-threads=1` passed; 209 tests; exit `0`; log `target/validation-logs/test-lib-impl-step4.log`.
     - `cargo clippy --all-targets -- -D warnings` passed; exit `0`; log `target/validation-logs/clippy-impl-step4.log`.
   - P3 API/worker compatibility slices for batch command limits, invalid envelopes, durable resume, and supervisor reload now have targeted coverage.
4. Persist compact receipt-chain classification evidence in validation/report rows.
   - Added source-derived receipt replay classification inventory fields to `scripts/observe_validation.sh`.
   - The persisted `validation_summary` row now emits `receipt_replay_classification_present`, `receipt_replay_classifications`, `receipt_replay_classification_evidence_files`, and `receipt_replay_classification_evidence_tokens`.
   - Added `missing_receipt_replay_classification_report` to missing-signal flags so report consumers can distinguish unavailable classification evidence from ordinary validation failures.
   - Added Python contract coverage for the new report schema tokens and all five compact classes: `forged_receipt`, `duplicated_receipt`, `reordered_receipt`, `stale_receipt`, and `missing_receipt`.
   - Smoke report evidence with `CANON_TEST_TIMEOUT_SECONDS=1` confirmed the new summary fields are emitted and `missing_receipt_replay_classification_report` is `false`; the smoke validation status itself was expectedly `fail` because the timeout was intentionally too short.
   - Validation evidence from current implementation step:
     - `python3 -m unittest tests/test_observe_validation_contract.py` passed; 16 tests; exit `0`; log `target/validation-logs/observe-validation-contract-impl-step5.log`.
     - `cargo test --test api_transport_contract -- --test-threads=1` passed; 19 tests; exit `0`; log `target/validation-logs/api-transport-contract-impl-step5.log`.
     - `cargo fmt --check` passed; exit `0`; log `target/validation-logs/fmt-impl-step5.log`.
     - `cargo test --lib -- --test-threads=1` passed; 209 tests; exit `0`; log `target/validation-logs/test-lib-impl-step5.log`.
     - `cargo clippy --all-targets -- -D warnings` passed; exit `0`; log `target/validation-logs/clippy-impl-step5.log`.

### P4 — Graph source-of-truth integration

1. Keep graph telemetry observable but optional unless wrapper variables are configured.
2. Verify graph mutation ops, patch receipts, snapshot contracts, and landing receipts as one end-to-end flow.
3. Clarify `canon-rustc-v3` and `graph-editor` repository/subproject boundaries.

### P5 — Domain intelligence layer

1. Keep domain docs unwired until contracts are stable.
2. Define record contracts before behavior.
3. Keep trading separate from production business automation.

## Next Execute-Turn Recommendation

1. Continue by moving to P4 graph source-of-truth integration unless another P3 report consumer needs deeper receipt replay rows.
2. Keep compact receipt-chain classifications source-derived in observe-validation until a durable runtime receipt ledger format is added for failed replay attempts.
3. Keep policy promotion externally verified; never let the LLM approve its own candidates.
4. Continue using `TMPDIR="$PWD/target/test-tmp"` for Rust tests in quota-sensitive sandboxes.
5. Run targeted tests for the next consumer, `cargo fmt --check`, `cargo test --lib -- --test-threads=1`, and `cargo clippy --all-targets -- -D warnings` after the next P3 slice.

## Planning-Turn Handoff

P0, P1, and P2 are complete. P3 now has receipt-chain invariant verification, API transport consumption, worker/supervisor boundary coverage, and persisted observe-validation summary evidence for compact replay classifications. The next execution turn should move to the next P4 graph source-of-truth slice unless a deeper P3 runtime receipt ledger for failed replay attempts is required.

## Current Non-Goals

- Do not wire domain specs into runtime behavior yet.
- Do not redesign the kernel.
- Do not add live trading behavior.
- Do not rely on external LLM/Ollama/OpenAI availability for baseline correctness.
- Do not commit token caches, runtime archives, target outputs, validation logs, or SSE chunk logs.
