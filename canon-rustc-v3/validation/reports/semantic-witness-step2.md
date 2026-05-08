# Semantic Witness Replay — Step 2

Reviewed: 2026-05-08
Fixture: `validation/fixtures/witness_crate/Cargo.toml`
Runner report: `validation/reports/semantic-witness-step2.json` generated locally and ignored by git.
Preflight report: `validation/reports/semantic-preflight-step2.json` generated locally and ignored by git.

## Result

| Check | Result |
|---|---:|
| `python3 validation/run_semantic_witness.py --require-cargo --report validation/reports/semantic-witness-step2.json` | pass |
| `python3 validation/semantic_preflight.py --artifact-root state/rustc-test-1 --compare-artifact-root state/rustc-test-2 --require-live-replay --report validation/reports/semantic-preflight-step2.json` | pass_with_skip |
| Live replay validation | true |
| Remaining skipped signal | `vendor/rust-source is not initialized` |

## Graph Evidence

| Run | Schema | Crate | Nodes | Edges | Graph hash | Fingerprint |
|---|---:|---|---:|---:|---|---|
| `state/rustc-test-1` | 16 | `witness_crate` | 11 | 33 | `5e20cdb250bfedac3fda428f4043a0fda06e7e508d124b69f0efea61cdf15aa8` | `a800b88569f202127c0fafcc79009b57608c0451c37cdb2258e1544def8bf09e` |
| `state/rustc-test-2` | 16 | `witness_crate` | 11 | 33 | `5e20cdb250bfedac3fda428f4043a0fda06e7e508d124b69f0efea61cdf15aa8` | `a800b88569f202127c0fafcc79009b57608c0451c37cdb2258e1544def8bf09e` |

## Timing Evidence

| Metric | Value |
|---|---:|
| Baseline fixture check | 139.385 ms |
| Wrapped fixture check | 188.694 ms |
| Overhead ratio | 1.354 |

## Receipt Evidence

| Receipt | Hash |
|---|---|
| semantic witness report | `6386f442613c3f0bfc87b6503c7b4b8d11a355c9e343c23806202d109bd9755b` |
| semantic preflight report | `3b27735a84d66b17a0e2661c53cc2e41257b65520c64b0c73294421391ddd571` |
