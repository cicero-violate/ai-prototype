# canon-rustc-v3 Scorecard

Reviewed: 2026-05-08
Stage: `implementation step 3`
Base commit: `bcb7fae`
Scope: thresholded performance validation for `ai/canon-rustc-v3`; wrapper behavior unchanged.

## Validation Evidence

| Check | Result | Judgment |
|---|---:|---|
| `python3 validation/semantic_scale_probe.py --nodes 5000 --fanout 2 --risk-additions 100 --threshold-ms 2000 --report validation/reports/semantic-scale-step3.json` | pass | Scale probe completed in 29.31 ms under 2000 ms. |
| `python3 validation/run_semantic_witness.py --require-cargo --report validation/reports/semantic-witness-step3.json` | pass | Fixture replay completed and emitted identical schema-16 graph hashes across wrapped runs. |
| `python3 validation/performance_gate.py --scale-report validation/reports/semantic-scale-step3.json --native-overhead-report validation/reports/semantic-witness-step3.json --require-native-overhead --max-overhead-ratio 3.0 --max-wrapped-ms 1000 --report validation/reports/performance-step3.json` | pass | Native overhead ratio 0.985 and wrapped check 201.617 ms are below thresholds. |
| Negative threshold probe with `--max-overhead-ratio 0.5` | fail_expected | Gate failed with `native_overhead_ratio_exceeded`, proving threshold enforcement. |
| `python3 -m py_compile validation/performance_gate.py validation/semantic_scale_probe.py validation/run_semantic_witness.py` | pass | Python validation scripts compile. |
| `cargo test --offline` | pass | 11 Rust unit tests pass. |
| `cargo check --offline --no-default-features` | pass | Pass-through/non-capture boundary still compiles. |

## Progress This Turn

- Added explicit native overhead thresholds to `validation/performance_gate.py`.
- Added failure modes for overhead ratio and absolute wrapped-time violations.
- Ran the full scale + witness + performance gate path with required native overhead evidence.
- Recorded passing metrics: scale elapsed `29.31 ms`, baseline `204.609 ms`, wrapped `201.617 ms`, overhead ratio `0.985`.
- Verified the gate fails when the configured overhead ratio threshold is too low.
- Added tracked evidence in `validation/reports/performance-step3.md`.

## Scores

| Dimension | Score | Evidence-backed judgment |
|---|---:|---|
| Correctness | 8 | Live replay and thresholded performance validation pass; full rust-source closure remains skipped. |
| Determinism | 9 | Fixture replay continues to produce identical graph hashes and fingerprints. |
| Alignment | 8 | Performance gate now measures the wrapper use case rather than only synthetic scale. |
| Transparency | 9 | Positive and negative gate evidence records thresholds, observed metrics, and receipt hashes. |
| Performance | 8 | Native overhead is now bounded by explicit thresholds and enforcement is tested. |
| Simplicity | 7 | Gate changes are local CLI additions without changing wrapper behavior. |
| Future-proofing | 8 | Future regressions can fail on ratio or absolute wrapped-time thresholds. |

## Aggregate

Average score: `8.00 / 10`.

## Next Proof Target

Reconcile `GOAL.md` with schema version 16 and the current relation vocabulary so project prose, implementation, tests, and validation evidence describe the same contract.
