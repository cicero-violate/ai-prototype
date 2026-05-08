# canon-rustc-v3 Scorecard

Reviewed: 2026-05-08
Stage: `implementation step 1`
Base commit: `bdd2893`
Scope: add a schema-16 relation vocabulary contract for `ai/canon-rustc-v3`; implementation behavior unchanged.

## Validation Evidence This Turn

| Check                                                                           | Result | Judgment                                                                                                          |
|---------------------------------------------------------------------------------+--------+-------------------------------------------------------------------------------------------------------------------|
| `python3 validation/schema16_relation_contract.py`                              | pass   | Rust-declared allowed/risk relations match the schema-16 contract; auto-refactor fixture relations are canonical. |
| `python3 validation/auto_refactor_surface_smoke.py`                             | pass   | Existing schema-16 surface report fixture remains stable.                                                         |
| `python3 validation/auto_refactor_ops_smoke.py`                                 | pass   | Advisory op generation still emits valid `SplitFn`, `MergeFns`, and `ExtractTrait` specs.                         |
| `python3 -m py_compile ...`                                                     | pass   | New and relevant validation scripts compile.                                                                      |
| `cargo test --offline`                                                          | pass   | 11 Rust unit tests pass.                                                                                          |
| `cargo check --offline --no-default-features`                                   | pass   | Pass-through/non-capture boundary still compiles.                                                                 |
| `git diff --check -- validation/schema16_relation_contract.py plan.md score.md` | pass   | No whitespace errors in this turn's changed files.                                                                |

## Progress This Turn

- Implemented the next concrete target from `plan.md`: a compact schema-16 relation contract test.
- Connected the contract to `src/facts.rs` instead of duplicating unchecked fixture expectations only.
- Verified the schema-16 auto-refactor fixture emits `call`, `phase`, `provider`, and `similar` and no non-canonical relations.
- Preserved all wrapper behavior and validation thresholds.
- Updated `plan.md` and `score.md` to reflect the completed implementation step.

## Scores

| Dimension       | Score | Evidence-backed judgment                                                                       |
|-----------------+-------+------------------------------------------------------------------------------------------------|
| Correctness     |     9 | Static Rust vocabulary and schema-16 fixture relations are now checked together.               |
| Determinism     |     9 | Contract reads stable source and fixture content; no runtime nondeterminism introduced.        |
| Alignment       |     9 | Plan, score, Rust constants, and fixture relation expectations now share one vocabulary.       |
| Transparency    |     9 | Fixture coverage is explicit and does not overclaim full relation coverage from one graph.     |
| Performance     |     8 | No capture-path change; existing performance gates remain the planned regression proof.        |
| Simplicity      |     9 | The validation script is small, direct, and focused on one contract.                           |
| Future-proofing |     9 | Future schema changes must update constants, fixture expectations, and this contract together. |

## Aggregate

Average score: `8.86 / 10`.

## Next Proof Target

Expand witness fixture coverage after replay determinism remains stable, then rerun replay, preflight, performance, and whitespace gates after any semantic implementation change.
