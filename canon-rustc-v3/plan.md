# canon-rustc-v3 Plan

Base commit: `2873d7d`
Stage: `implementation step 2`
Status: `fixture replay implemented`

## Objective

Keep `canon-rustc-v3` focused on compiler-backed semantic witness capture: run as a `RUSTC_WRAPPER`, preserve real compiler behavior, and emit deterministic graph facts usable by downstream refactoring agents.

## Current Findings

- `PURPOSE.md` is filled and remains within the requested 10 LOC limit.
- `Cargo.toml` currently defaults to `rustc-driver`, so native witness capture is the default path.
- `cargo test --offline` passes with 11 unit tests.
- `cargo check --offline --no-default-features` passes, proving the pass-through boundary still compiles.
- Native fixture replay now passes on `validation/fixtures/witness_crate` after isolating that fixture from the parent workspace.

## Executed In Step 1

1. Preserved default `rustc-driver` capture behavior; no wrapper execution semantics were changed.
2. Tightened schema-16 relation vocabulary tests in `src/facts.rs`.
3. Added wrapper unit tests for graph schema version 16, receipt schema version 1, allowed-edge filtering, BTreeMap-backed graph hash stability, and receipt hash sensitivity to schema changes.
4. Ran targeted validation: `cargo test --offline`, `cargo check --offline --no-default-features`, and `git diff --check`.

## Executed In Step 2

1. Isolated `validation/fixtures/witness_crate` with an empty `[workspace]` table so Cargo can check it by manifest path without workspace membership errors.
2. Ran `python3 validation/run_semantic_witness.py --require-cargo --report validation/reports/semantic-witness-step2.json`.
3. Captured two wrapped fixture runs with matching schema-16 graph evidence: 1 graph, 11 nodes, 33 edges, graph hash `5e20cdb250bfedac3fda428f4043a0fda06e7e508d124b69f0efea61cdf15aa8`, and fingerprint `a800b88569f202127c0fafcc79009b57608c0451c37cdb2258e1544def8bf09e`.
4. Ran `python3 validation/semantic_preflight.py --artifact-root state/rustc-test-1 --compare-artifact-root state/rustc-test-2 --require-live-replay --report validation/reports/semantic-preflight-step2.json`; live replay validation is true, with only `vendor/rust-source is not initialized` skipped.
5. Saved tracked replay evidence in `validation/reports/semantic-witness-step2.md`; raw JSON reports and `state/rustc-test-*` graph artifacts remain ignored.

## Remaining Implementation Plan

1. Convert the ad hoc timing evidence into a bounded performance gate with explicit thresholds.
2. Reconcile long-form `GOAL.md` schema prose with the current implementation, especially schema version and relation additions.
3. If fixture replay exposes graph or receipt drift in future runs, update tests before changing semantics.

## Non-Goals For This Turn

- No schema rewrite.
- No production-readiness claim without a committed performance threshold and full toolchain-source closure.
