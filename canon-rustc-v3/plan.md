# canon-rustc-v3 Plan

Base commit: `6a7275b`
Stage: `planning refresh`
Status: `implementation reviewed; docs aligned`

## Objective

Keep `canon-rustc-v3` focused on compiler-backed semantic witness capture: run as a `RUSTC_WRAPPER`, preserve real compiler behavior, and emit deterministic schema-16 graph facts usable by downstream refactoring agents.

## Current Findings

- `PURPOSE.md` is filled and remains within the requested 10 LOC limit.
- The default feature set includes `rustc-driver`; `--no-default-features` remains the pass-through build boundary.
- `GRAPH_SCHEMA_VERSION` is `16` and `RECEIPT_SCHEMA_VERSION` is `1` in `src/wrapper.rs`.
- Allowed relations include `call`, `impl`, `mut`, `io`, `unsafe`, `panic`, `alloc`, `use`, `similar`, `phase`, and `provider`.
- Risk relations are `mut`, `io`, `unsafe`, `panic`, `alloc`, `similar`, and `phase`; `provider` remains advisory/non-risk.
- Existing validation files cover native fixture replay, semantic preflight, performance gating, runtime receipts, delta contracts, toolchain archive checks, and advisory auto-refactor smoke tests.

## Current Implementation Plan

1. Preserve wrapper execution semantics; do not change compiler invocation or source rewriting behavior during planning/scoring turns.
2. Treat schema 16 as the active contract unless a later implementation turn updates constants, tests, fixtures, and docs together.
3. Keep auto-refactor functionality advisory-only: graph -> surface report -> op specs; no source edits inside this crate.
4. Use fixture replay and receipt/hash checks as the first proof target after any semantic graph change.
5. Use performance thresholds as regression gates after any capture-path or MIR/HIR traversal change.
6. Keep `PURPOSE.md` short and stable so future agents can identify the project boundary quickly.

## Next Implementation Targets

1. Add a compact contract test that verifies emitted fixture graphs contain the documented schema-16 relation vocabulary.
2. Expand witness fixture coverage only after replay determinism remains stable across agent turns.
3. Close the remaining native toolchain-source gap when `vendor/rust-source` is initialized.
4. Keep `plan-autorefactor.md` as secondary guidance unless a new root auto-refactor plan is explicitly requested.

## Non-Goals For This Turn

- No schema rewrite.
- No wrapper behavior change.
- No validation threshold change.
- No source rewrite authority added.
- No production-readiness claim without full toolchain-source closure.
