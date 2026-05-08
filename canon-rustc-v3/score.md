# canon-rustc-v3 Scorecard

Reviewed: 2026-05-08
Stage: `implementation step 1`
Base commit: `8bb9d75`
Scope: schema/test hardening for `ai/canon-rustc-v3`; wrapper behavior unchanged.

## Validation Evidence

| Check | Result | Judgment |
|---|---:|---|
| `cargo test --offline` | pass | 11 unit tests pass, including new schema/version/hash/vocabulary tests. |
| `cargo check --offline --no-default-features` | pass | Pass-through/non-capture boundary still compiles. |
| `git diff --check -- src/wrapper.rs src/facts.rs plan.md score.md` | pass | No whitespace errors in this turn's changed files. |
| `PURPOSE.md` | pass | Still within the requested 10 LOC limit. |

## Progress This Turn

- Added schema contract coverage for `GRAPH_SCHEMA_VERSION = 16` and `RECEIPT_SCHEMA_VERSION = 1`.
- Added complete relation vocabulary assertions for schema-16 node, edge, and risk relation lists.
- Added allowed-edge filtering coverage for duplicate, unknown, and empty endpoint edges.
- Added stable graph hash coverage for BTreeMap node ordering.
- Added receipt hash sensitivity coverage for graph schema changes.

## Scores

| Dimension | Score | Evidence-backed judgment |
|---|---:|---|
| Correctness | 7 | Contract tests improved; live fixture capture and receipt validation remain open. |
| Determinism | 8 | Hash stability now has direct unit coverage. |
| Alignment | 8 | Tests reinforce the current wrapper schema without changing capture behavior. |
| Transparency | 8 | Plan and score now identify completed test hardening and remaining proof gaps. |
| Performance | 5 | No wrapped-vs-unwrapped overhead measurement yet. |
| Simplicity | 7 | Unit tests are local and avoid public API expansion. |
| Future-proofing | 7 | Schema and receipt constants now have regression coverage. |

## Aggregate

Average score: `7.00 / 10`.

## Next Proof Target

Run the wrapper against a small fixture crate, archive the emitted `graph.json`, validate schema version 16 receipts, and then measure wrapper overhead against plain `cargo check`.
