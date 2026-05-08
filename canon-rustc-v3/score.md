# canon-rustc-v3 Scorecard

Reviewed: 2026-05-08
Stage: `implementation step 5`
Base commit: `421d3ed`
Scope: project-contract prose reconciliation for `ai/canon-rustc-v3`; implementation behavior unchanged.

## Validation Evidence

| Check | Result | Judgment |
|---|---:|---|
| `GOAL.md` schema/prose scan | pass | Required schema-16 terms are present and stale schema-12 claims are absent. |
| `python3 validation/auto_refactor_surface_smoke.py` | pass | Auto-refactor surface report still matches the schema-16 fixture. |
| `python3 validation/auto_refactor_ops_smoke.py` | pass | Advisory op generation still emits valid `SplitFn`, `MergeFns`, and `ExtractTrait` specs. |
| `python3 -m py_compile validation/auto_refactor_surface.py validation/auto_refactor_ops.py validation/performance_gate.py validation/semantic_preflight.py` | pass | Relevant Python validators compile. |
| `cargo test --offline` | pass | 11 Rust unit tests pass. |
| `cargo check --offline --no-default-features` | pass | Pass-through/non-capture boundary still compiles. |
| `git diff --check -- GOAL.md plan.md score.md` | pass | No whitespace errors in this turn's changed files. |

## Progress This Turn

- Updated `GOAL.md` to describe `schema_version = 16` instead of stale schema 12.
- Added current schema metadata, including `receipt_hash`.
- Added current edge relations: `similar`, `phase`, and `provider`.
- Corrected captured node-kind prose for `struct`, `enum`, and `ty_alias`.
- Documented the current validation stack and fixture replay evidence.
- Clarified that auto-refactor is advisory-only and does not rewrite source.

## Scores

| Dimension | Score | Evidence-backed judgment |
|---|---:|---|
| Correctness | 8 | Documentation now matches the tested schema-16 contract; full rust-source closure remains skipped. |
| Determinism | 9 | Existing replay/hash evidence remains unchanged and documented accurately. |
| Alignment | 9 | GOAL, implementation constants, tests, and validation evidence now describe the same contract. |
| Transparency | 9 | The source-rewrite boundary and advisory-only auto-refactor status are explicit. |
| Performance | 8 | Thresholded performance evidence remains in force; no behavior changes were made. |
| Simplicity | 8 | Stale schema prose was replaced with one current contract rather than adding competing guidance. |
| Future-proofing | 8 | Future agents have a synchronized contract for schema, validation, and auto-refactor boundaries. |

## Aggregate

Average score: `8.29 / 10`.

## Next Proof Target

Increase fixture coverage or add a small contract test that validates emitted witness graphs contain the documented schema-16 relation vocabulary without changing wrapper semantics.
