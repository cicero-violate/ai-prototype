# Performance Gate — Step 3

Reviewed: 2026-05-08
Scale report: `validation/reports/semantic-scale-step3.json` generated locally and ignored by git.
Witness report: `validation/reports/semantic-witness-step3.json` generated locally and ignored by git.
Gate report: `validation/reports/performance-step3.json` generated locally and ignored by git.

## Positive Gate

| Check | Result |
|---|---:|
| `python3 validation/semantic_scale_probe.py --nodes 5000 --fanout 2 --risk-additions 100 --threshold-ms 2000 --report validation/reports/semantic-scale-step3.json` | pass |
| `python3 validation/run_semantic_witness.py --require-cargo --report validation/reports/semantic-witness-step3.json` | pass |
| `python3 validation/performance_gate.py --scale-report validation/reports/semantic-scale-step3.json --native-overhead-report validation/reports/semantic-witness-step3.json --require-native-overhead --max-overhead-ratio 3.0 --max-wrapped-ms 1000 --report validation/reports/performance-step3.json` | pass |

## Thresholds

| Threshold | Value |
|---|---:|
| Maximum overhead ratio | 3.0 |
| Maximum wrapped fixture check | 1000 ms |
| Scale probe threshold | 2000 ms |

## Observed Metrics

| Metric | Value |
|---|---:|
| Scale nodes | 5000 |
| Scale edges | 10100 |
| Scale elapsed | 29.31 ms |
| Baseline fixture check | 204.609 ms |
| Wrapped fixture check | 201.617 ms |
| Native overhead ratio | 0.985 |

## Negative Gate

An intentional low threshold verified enforcement:

| Check | Result |
|---|---|
| `--max-overhead-ratio 0.5` | fail |
| Failure kind | `native_overhead_ratio_exceeded` |

## Receipts

| Receipt | Hash |
|---|---|
| scale report | `bfb7cfc28dae4dc8b8e7e89223ffe859ab3b3859f1d1596c20b2bc33877d6f7a` |
| witness report | `d7430e7a632d6102bae32f43251c6a5300a2450caf4cbe8ac39e7e04250f2533` |
| performance gate report | `53016ef3f57058fae2de7831b4c54f1b97968e00c0f21fe2febfffbe7d8514a9` |
