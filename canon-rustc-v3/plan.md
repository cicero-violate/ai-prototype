# canon-rustc-v3 Plan

Base commit: `bcb7fae`
Stage: `implementation step 3`
Status: `bounded performance gate implemented`

## Objective

Keep `canon-rustc-v3` focused on compiler-backed semantic witness capture: run as a `RUSTC_WRAPPER`, preserve real compiler behavior, and emit deterministic graph facts usable by downstream refactoring agents.

## Current Findings

- `PURPOSE.md` is filled and remains within the requested 10 LOC limit.
- `Cargo.toml` currently defaults to `rustc-driver`, so native witness capture is the default path.
- `cargo test --offline` passes with 11 unit tests.
- `cargo check --offline --no-default-features` passes, proving the pass-through boundary still compiles.
- Native fixture replay passes on `validation/fixtures/witness_crate` and emits identical schema-16 graphs across wrapped runs.
- `validation/performance_gate.py` now enforces explicit native overhead thresholds.

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

## Executed In Step 3

1. Added `--max-overhead-ratio` and `--max-wrapped-ms` to `validation/performance_gate.py`.
2. Enforced native overhead failure kinds: `native_overhead_ratio_exceeded` and `native_wrapped_time_exceeded`.
3. Ran scale, replay, and performance gate with thresholds: max overhead ratio `3.0`, max wrapped check `1000 ms`, scale threshold `2000 ms`.
4. Observed passing metrics: scale elapsed `29.31 ms`, baseline `204.609 ms`, wrapped `201.617 ms`, overhead ratio `0.985`.
5. Verified a negative gate with `--max-overhead-ratio 0.5` fails with `native_overhead_ratio_exceeded`.
6. Saved tracked performance evidence in `validation/reports/performance-step3.md`; raw JSON reports remain ignored.

## Remaining Implementation Plan

1. Reconcile long-form `GOAL.md` schema prose with the current implementation, especially schema version 16 and relation additions.
2. If fixture replay exposes graph or receipt drift in future runs, update tests before changing semantics.
3. Consider raising fixture coverage only after the schema prose and current validation contract are aligned.

## Non-Goals For This Turn

- No schema rewrite.
- No wrapper behavior change.
- No production-readiness claim without full toolchain-source closure.
