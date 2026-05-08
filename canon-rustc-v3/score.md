# canon-rustc-v3 Scorecard

Reviewed: 2026-05-08
Stage: `implementation step 2`
Base commit: `2873d7d`
Scope: native fixture replay for `ai/canon-rustc-v3`; fixture isolation added, wrapper behavior unchanged.

## Validation Evidence

| Check | Result | Judgment |
|---|---:|---|
| `python3 validation/run_semantic_witness.py --require-cargo --report validation/reports/semantic-witness-step2.json` | pass | Baseline check, wrapper build, two wrapped fixture runs, graph replay comparison, and timing receipt completed. |
| `python3 validation/semantic_preflight.py --artifact-root state/rustc-test-1 --compare-artifact-root state/rustc-test-2 --require-live-replay --report validation/reports/semantic-preflight-step2.json` | pass_with_skip | Live replay validation is true; only skipped signal is uninitialized `vendor/rust-source`. |
| `cargo test --offline` | pass | 11 unit tests pass through the witness runner. |
| `cargo check --offline --no-default-features` | pass | Previously verified pass-through/non-capture boundary remains part of the score. |
| `validation/reports/semantic-witness-step2.md` | pass | Tracked replay evidence records the generated graph and receipt metrics while ignored raw JSON/state artifacts stay out of git. |

## Progress This Turn

- Fixed fixture replay by adding an empty `[workspace]` table to `validation/fixtures/witness_crate/Cargo.toml`.
- Captured two native wrapped fixture runs with identical schema-16 graph fingerprints.
- Recorded graph evidence: 1 graph, 11 nodes, 33 edges, graph hash `5e20cdb250bfedac3fda428f4043a0fda06e7e508d124b69f0efea61cdf15aa8`.
- Recorded replay fingerprint `a800b88569f202127c0fafcc79009b57608c0451c37cdb2258e1544def8bf09e` for both wrapped runs.
- Recorded timing evidence: baseline 139.385 ms, wrapped 188.694 ms, overhead ratio 1.354.

## Scores

| Dimension | Score | Evidence-backed judgment |
|---|---:|---|
| Correctness | 8 | Live fixture capture now passes and emits schema-16 graph evidence; full rust-source closure remains skipped. |
| Determinism | 9 | Two wrapped runs produced identical graph hashes and replay fingerprints. |
| Alignment | 8 | Fixture replay directly exercises the `RUSTC_WRAPPER` semantic witness use case. |
| Transparency | 8 | Tracked markdown evidence records graph, timing, and receipt hashes while generated artifacts stay ignored. |
| Performance | 6 | Timing evidence exists, but no thresholded performance gate is committed yet. |
| Simplicity | 7 | Fixture isolation is minimal and wrapper behavior is unchanged. |
| Future-proofing | 7 | Replay evidence strengthens regression confidence; schema prose still needs reconciliation. |

## Aggregate

Average score: `7.43 / 10`.

## Next Proof Target

Convert the replay timing measurement into a bounded performance gate with explicit acceptable overhead thresholds, then reconcile `GOAL.md` with schema version 16 and the current relation vocabulary.
