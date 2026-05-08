# canon-rustc-v3 Plan

Base commit: `bdd2893`
Stage: `implementation step 1`
Status: `schema-16 relation contract added`

## Objective

Keep `canon-rustc-v3` focused on compiler-backed semantic witness capture: run as a `RUSTC_WRAPPER`, preserve real compiler behavior, and emit deterministic schema-16 graph facts usable by downstream refactoring agents.

## Current Findings

- `PURPOSE.md` is filled and remains within the requested 10 LOC limit.
- The default feature set includes `rustc-driver`; `--no-default-features` remains the pass-through build boundary.
- `GRAPH_SCHEMA_VERSION` is `16` and `RECEIPT_SCHEMA_VERSION` is `1` in `src/wrapper.rs`.
- Allowed relations include `call`, `impl`, `mut`, `io`, `unsafe`, `panic`, `alloc`, `use`, `similar`, `phase`, and `provider`.
- Risk relations are `mut`, `io`, `unsafe`, `panic`, `alloc`, `similar`, and `phase`; `provider` remains advisory/non-risk.
- `validation/schema16_relation_contract.py` now checks the Rust-declared relation vocabulary and verifies the schema-16 auto-refactor fixture emits only canonical advisory relations.

## Completed In Step 1

1. Added `validation/schema16_relation_contract.py` as a compact fixture-level schema-16 relation contract.
2. Bound the Python contract to `src/facts.rs` so declared `EDGE_RELATIONS` and `RISK_RELATIONS` must match the documented schema-16 vocabulary.
3. Verified `validation/fixtures/auto_refactor_surface/graph.json` is schema 16 and emits the expected relation set: `call`, `phase`, `provider`, and `similar`.
4. Preserved wrapper execution semantics; no compiler invocation, graph emission, or validation threshold behavior was changed.

## Current Implementation Plan

1. Treat schema 16 as the active contract unless a later implementation turn updates constants, tests, fixtures, and docs together.
2. Keep auto-refactor functionality advisory-only: graph -> surface report -> op specs; no source edits inside this crate.
3. Use fixture replay and receipt/hash checks as the first proof target after any semantic graph change.
4. Use performance thresholds as regression gates after any capture-path or MIR/HIR traversal change.
5. Keep `PURPOSE.md` short and stable so future agents can identify the project boundary quickly.

## Next Implementation Targets

1. Expand witness fixture coverage only after replay determinism remains stable across agent turns.
2. Close the remaining native toolchain-source gap when `vendor/rust-source` is initialized.
3. Keep `plan-autorefactor.md` as secondary guidance unless a new root auto-refactor plan is explicitly requested.

## Non-Goals For This Turn

- No schema rewrite.
- No wrapper behavior change.
- No validation threshold change.
- No source rewrite authority added.
- No production-readiness claim without full toolchain-source closure.
