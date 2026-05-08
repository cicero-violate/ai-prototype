# canon-rustc-v3 Plan

Base commit: `3486120`
Stage: `planning / scoring turn`
Status: `reviewed; next implementation not started`

## Objective

Keep `canon-rustc-v3` focused on compiler-backed semantic witness capture: run as a `RUSTC_WRAPPER`, preserve real compiler behavior, and emit deterministic graph facts usable by downstream refactoring agents.

## Current Findings

- `PURPOSE.md` was empty and has been filled with a 7-line purpose statement.
- `Cargo.toml` currently defaults to `rustc-driver`, so native witness capture is the default path.
- `cargo check --offline` passes in this workspace.
- `cargo check --offline --no-default-features` also passes, proving the pass-through boundary still compiles.
- Source-code `TODO`/`FIXME` scan found no implementation markers outside planning docs.

## Implementation Plan

1. Preserve the working default `rustc-driver` build and avoid changing capture behavior during this planning turn.
2. Add or tighten tests around graph schema version 16, relation vocabulary, and stable hash inputs.
3. Re-run native wrapper capture on a small fixture crate and save the emitted `graph.json` as validation evidence.
4. Add a bounded performance check comparing wrapped and unwrapped `cargo check` on the same fixture.
5. Reconcile long-form `GOAL.md` schema prose with the current implementation, especially schema version and relation additions.

## Non-Goals For This Turn

- No source-code implementation changes.
- No schema rewrite.
- No claim of production readiness without fixture replay, receipt validation, and overhead evidence.
