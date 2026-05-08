# canon-rustc-v3 Plan

Base commit: `8bb9d75`
Stage: `implementation step 1`
Status: `schema/test hardening implemented`

## Objective

Keep `canon-rustc-v3` focused on compiler-backed semantic witness capture: run as a `RUSTC_WRAPPER`, preserve real compiler behavior, and emit deterministic graph facts usable by downstream refactoring agents.

## Current Findings

- `PURPOSE.md` is filled and remains within the requested 10 LOC limit.
- `Cargo.toml` currently defaults to `rustc-driver`, so native witness capture is the default path.
- `cargo test --offline` passes with 11 unit tests.
- `cargo check --offline --no-default-features` passes, proving the pass-through boundary still compiles.
- Source-code `TODO`/`FIXME` scan previously found no implementation markers outside planning docs.

## Executed In Step 1

1. Preserved default `rustc-driver` capture behavior; no wrapper execution semantics were changed.
2. Tightened schema-16 relation vocabulary tests in `src/facts.rs`.
3. Added wrapper unit tests for graph schema version 16, receipt schema version 1, allowed-edge filtering, BTreeMap-backed graph hash stability, and receipt hash sensitivity to schema changes.
4. Ran targeted validation: `cargo test --offline`, `cargo check --offline --no-default-features`, and `git diff --check`.

## Remaining Implementation Plan

1. Re-run native wrapper capture on a small fixture crate and save the emitted `graph.json` as validation evidence.
2. Add a bounded performance check comparing wrapped and unwrapped `cargo check` on the same fixture.
3. Reconcile long-form `GOAL.md` schema prose with the current implementation, especially schema version and relation additions.
4. If fixture replay exposes graph or receipt drift, update tests before changing semantics.

## Non-Goals For This Turn

- No schema rewrite.
- No fixture replay claim yet.
- No production-readiness claim without wrapper replay, receipt validation, and overhead evidence.
