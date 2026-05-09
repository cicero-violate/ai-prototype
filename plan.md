# Canon Agent Implementation Plan

## Current State

Canon Agent is a Rust prototype for a deterministic, auditable, self-improving agent runtime. The target architecture remains a formally constrained state-machine kernel with a capability layer around it. The kernel owns correctness, state transitions, durable records, replay boundaries, and audit evidence. LLMs and tools operate inside the capability layer and must produce typed, reviewable evidence rather than governing the runtime directly.

Current implementation snapshot, 2026-05-09 America/Toronto / 2026-05-09 UTC:

- Branch: `main`.
- Latest visible commit before this implementation turn: `27853af Classify graph evidence in validation summary`.
- Working directory: `/workspace/ai_sandbox/canon-mini-agent/prototype/ai`.
- P0 validation baseline is complete.
- P1 validation evidence reporting is complete.
- P2 agent loop reliability is complete.
- P3 runtime and receipt correctness is complete for the current scope.
- P4 graph source-of-truth integration now includes persisted graph evidence classification plus deterministic fixture-backed positive evidence for landed graph mutations with receipt snapshots.

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

- All-target validation has final exit evidence and final `test result:` summaries from prior implementation work.
- Format, lib tests, and clippy have passed in the current baseline.
- Correctness, robustness, and determinism score ceilings may remain raised while the validation baseline is preserved.

### P1 — Make validation evidence first-class — complete

- Observe-validation reporting emits source-backed validation summary evidence under ignored `target/observe/validation-report.ndjson`.
- The compact report includes git hygiene, command outcomes, graph telemetry presence/absence, runtime archive counts, ignored artifact counts, missing-signal flags, connector-failure classification fields, receipt replay classification inventory, and graph evidence state.
- Helper validators and manifest writer scripts are tracked source rather than stale generated artifacts.

### P2 — Agent loop reliability — complete

- Streaming retry behavior preserves evidence continuity across incomplete SSE/router cases.
- Retry decisions are constrained by target URL presence, missing `[DONE]`, missing `message_stream_complete`, and `finish_reason=length` classifications.
- Loop-driver mode classification distinguishes project planning, project execution, and worker certification prompts.

### P3 — Runtime and receipt correctness — complete for current scope

- Receipt-chain invariant verification covers valid replay and compact classifications for forged, duplicated, reordered, stale, and missing receipts.
- API transport consumers expose compact replay reports and expected-count missing-tail checks.
- Worker API route coverage exists for oversized batches, malformed batch payloads, tampered envelope hashes, durable resume replay, and supervisor reload replay continuity.
- Observe-validation summary evidence persists compact receipt-chain classification inventory and missing-signal state.
- Remaining optional P3 extension: add a durable runtime ledger format for failed replay attempts if future consumers require failed-replay rows beyond source-derived validation summary evidence.

### P4 — Graph source-of-truth integration — in progress

1. **Persist graph evidence classification in validation/report rows — complete**
   - Added source-derived graph evidence inventory fields to `scripts/observe_validation.sh`.
   - Added deterministic graph evidence status classifications:
     - `graph_wrapper_absent_by_configuration`
     - `graph_wrapper_configured_missing`
     - `graph_wrapper_configured_no_telemetry`
     - `graph_mutation_evidence_contract_missing`
     - `graph_mutation_evidence_emitted_not_landed`
     - `graph_mutation_landed_without_receipt_ledger`
     - `graph_mutation_landed_with_receipt_snapshot`
   - Added summary fields for graph source contract and graph workflow contract evidence files/tokens.
   - Added missing-signal flags for graph source/workflow contract report absence.
   - Added observe-validation contract coverage for the graph evidence report schema.

2. **Add deterministic positive graph evidence fixture/report path — complete**
   - Added `inspect_graph_workflow_fixture()` to validate the tracked graph workflow fixture manifest, file inventory, SHA-256 integrity rows, required workflow commands, landing command, ledger command, and receipt-snapshot evidence.
   - The observe-validation summary now emits `graph_workflow_fixture_*` fields and clears `missing_graph_workflow_fixture_receipt_snapshot` when the deterministic fixture proves a landed graph mutation with receipt snapshot evidence.
   - The positive graph status `graph_mutation_landed_with_receipt_snapshot` is now reachable without live wrapper telemetry when fixture evidence is complete.
   - Fixed the observe-validation `state_graph_present` assignment so graph status and missing-signal fields are defined consistently.

3. **Verify graph mutation ops, patch receipts, snapshot contracts, and landing receipts end-to-end — partially complete**
   - Existing graph CLI contract tests cover verify-ops, generate-patch, verify-landing, verify-receipts, stable usage, workflow fixture integrity, copyable workflow fixtures, and executable manifest flow.
   - Current evidence: `cargo test --test graph_mutation_cli_contract -- --test-threads=1` passed; 10 tests.
   - Current observe smoke evidence: `graph_evidence_status=graph_mutation_landed_with_receipt_snapshot`, `graph_workflow_fixture_status=pass`, and `missing_graph_workflow_fixture_receipt_snapshot=false`.

4. **Next P4 slice**
   - Consider adding a compact CLI/report command that emits the deterministic graph fixture evidence without running broad cargo validation.
   - Keep wrapper telemetry optional unless `CANON_RUSTC_WRAPPER` or `CANON_RUSTC_V3_ARTIFACT_DIR` is configured.
   - Clarify `canon-rustc-v3` and `graph-editor` repository/subproject boundaries if implementation work crosses those directories.

### P5 — Domain intelligence layer

1. Keep domain docs unwired until contracts are stable.
2. Define record contracts before behavior.
3. Keep trading separate from production business automation.

## Validation Evidence From Current Implementation Step

```text
CANON_TEST_TIMEOUT_SECONDS=1 CANON_OBSERVE_REPORT=target/observe/validation-report-p4-step2-smoke.ndjson python3 scripts/observe_validation.sh
exit: 1 expected from intentionally short cargo all-target timeout
result: validation_summary emitted graph_evidence_status=graph_mutation_landed_with_receipt_snapshot, graph_workflow_fixture_status=pass, graph_workflow_fixture_receipt_snapshot_present=true, missing_graph_workflow_fixture_receipt_snapshot=false
log: target/validation-logs/observe-smoke-p4-impl-step2.log
```

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 17 passed; 0 failed
log: target/validation-logs/observe-validation-contract-p4-impl-step2.log
```

```text
TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test graph_mutation_cli_contract -- --test-threads=1
exit: 0
result: 10 passed; 0 failed
log: target/validation-logs/graph-mutation-cli-contract-p4-impl-step2.log
```

```text
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check
exit: 0
log: target/validation-logs/fmt-p4-impl-step2.log
```

```text
TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib -- --test-threads=1
exit: 0
result: 209 passed; 0 failed
log: target/validation-logs/test-lib-p4-impl-step2.log
```

```text
TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings
exit: 0
log: target/validation-logs/clippy-p4-impl-step2.log
```

## Next Execute-Turn Recommendation

Continue P4 by making graph fixture evidence easier to consume separately from broad observe-validation, or proceed to clarifying subproject boundaries for `canon-rustc-v3` and `graph-editor` if the next implementation work crosses those project roots.

## Planning-Turn Handoff

P0, P1, P2, and current P3 scope are complete. P4 now has both source-derived graph evidence classification and deterministic fixture-backed positive landed-with-receipt-snapshot evidence in persisted validation summary rows.

## Current Non-Goals

- Do not wire domain specs into runtime behavior yet.
- Do not redesign the kernel.
- Do not add live trading behavior.
- Do not rely on external LLM/Ollama/OpenAI availability for baseline correctness.
- Do not commit token caches, runtime archives, target outputs, validation logs, or SSE chunk logs.
