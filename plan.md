# Canon Agent Implementation Plan

## Planning Checkpoint - 2026-05-08

Implementation step 1 resolved the duplicate auto-refactor plan authority. The
root `plan-autorefactor.md` is now the single authoritative auto-refactor plan;
`canon-rustc-v3/plan-autorefactor.md` is retained only as a pointer that marks
the older schema-v13 guidance as superseded.

Committed scope for this turn:

```text
plan.md
score.md
plan-autorefactor.md
canon-rustc-v3/plan-autorefactor.md
```

## Project Direction

Canon Agent remains a deterministic, self-improving agent runtime governed by a
formally verifiable state-machine kernel. The kernel owns correctness and
transition authority. The capability layer may propose, evaluate, report, and
learn from evidence, but it must not gain authority over the kernel.

The target architecture is still:

```text
kernel correctness -> typed evidence -> external validation -> policy/retrieval learning
```

The LLM remains a proposal source only. It does not approve itself, mutate the
transaction log, promote policy, or bypass validation.

## Invariants To Preserve

1. The state-machine kernel governs all authoritative transitions.
2. The TLog records structured, typed, hash-chained evidence.
3. Capability outputs are evidence inputs, not authority.
4. LLM output is proposal evidence only.
5. Policy promotion requires external validation evidence.
6. Retrieval examples remain evidence-bound and non-authoritative unless a
   separate storage authority boundary is explicitly validated.
7. Student-model training remains out of scope until the dataset is large,
   clean, and externally validated.
8. Planning/reporting surfaces must be deterministic, sorted where practical,
   and free of live network or live LLM dependencies.

## Current Worktree Evidence

The current worktree still contains uncommitted non-planning changes from other
lanes. They appear to target deterministic auto-refactor implementation and
validation-harness expectation maintenance:

```text
modified: graph-editor/Cargo.toml
modified: graph-editor/src/graph.rs
modified: graph-editor/src/lib.rs
modified: src/validation_harness.rs
modified: tests/fixtures/validation_command_footprint_receipts.txt
modified: tests/fixtures/validation_duration_planning_receipts.txt
modified: tests/validation_harness_contract.rs
untracked: canon-rustc-v3/validation/auto_refactor_ops.py
untracked: canon-rustc-v3/validation/auto_refactor_ops_smoke.py
untracked: graph-editor/src/autorefactor.rs
untracked: graph-editor/src/bin/auto_refactor_plan.rs
```

These files are not included in this step-1 commit. The next implementation
turn must either validate and commit them deliberately as one coherent lane, or
defer/revert them before selecting a different lane.

## Step 1 Result: Plan Authority

The duplicate plan decision is complete:

```text
authoritative_auto_refactor_plan = plan-autorefactor.md
nested_canon_plan_status = pointer only
superseded_material = schema-v13 transitional plan text
current_schema_target = schema-v16 graph capture
```

Rationale:

- The root plan reflects the current schema-v16 graph-editor integration.
- The nested plan contained stale schema-v13/v16 transitional execution notes.
- Keeping both as full plans created conflicting implementation authority.
- A pointer preserves discoverability without duplicating guidance.

## Selected Next Lane

Recommended next lane:

```text
lane = auto_refactor_graph_evidence
primary_axis = Structure
secondary_axis = Efficiency
guard_axis = Correctness
```

Target outcome:

```text
schema-v16 graph relations are consumed by deterministic advisory
auto-refactor planners that emit typed operation specifications without
rewriting source or changing runtime authority.
```

The auto-refactor lane is valuable only if it remains advisory. It may identify
split, merge, and provider-boundary candidates. It must not mutate source,
authorize runtime behavior, promote policy, approve candidates, or alter kernel
transitions.

## Implementation Plan For Next Turn

1. Verify the graph schema expectations in `graph-editor/src/graph.rs`, with
   special attention to schema version, relation parsing, missing fields, and
   deterministic ordering.
2. Validate `graph-editor/src/autorefactor.rs` as an advisory planner only:
   - consumes `call`, `phase`, `similar`, and `provider` relations;
   - emits stable `SplitFn`, `MergeFns`, and `ExtractTrait` operation specs;
   - sorts and deduplicates surfaces and operations;
   - carries stale-operation guards where source spans exist;
   - performs no source rewrite.
3. Validate `graph-editor/src/bin/auto_refactor_plan.rs` as a deterministic CLI
   surface over graph JSON.
4. Validate `canon-rustc-v3/validation/auto_refactor_ops.py` and smoke coverage
   as planning/reporting tools only. They must not weaken semantic validation or
   create a mutation path.
5. Separate validation-harness expectation drift from auto-refactor graph-editor
   work if possible. Do not combine unrelated fixes unless the tests require a
   single coherent commit.
6. Add or confirm focused tests for:
   - healthy graph input;
   - malformed or unsupported schema input;
   - stable repeated output;
   - advisory/non-mutating behavior;
   - operation-count consistency.
7. Run focused validation before staging anything.

## Validation Gate Before Implementation Commit

Minimum validation for the next implementation turn:

```text
CARGO_BUILD_RUSTC_WRAPPER= cargo fmt --check
CARGO_BUILD_RUSTC_WRAPPER= cargo check --quiet
CARGO_BUILD_RUSTC_WRAPPER= cargo test --test planning_contract --test score_contract --quiet
```

Additional validation for the auto-refactor lane:

```text
cd graph-editor && CARGO_BUILD_RUSTC_WRAPPER= cargo fmt --check
cd graph-editor && CARGO_BUILD_RUSTC_WRAPPER= cargo check --quiet
cd graph-editor && CARGO_BUILD_RUSTC_WRAPPER= cargo test --quiet
python3 canon-rustc-v3/validation/auto_refactor_ops_smoke.py
repeat auto-refactor plan generation twice and compare byte-for-byte output
```

If validation-harness files remain part of the selected implementation scope,
also run focused `validation_harness_contract` tests for the touched fixture
surfaces and record the exact filters used.

## Commit Discipline

The next implementation commit should include only files required for the chosen
lane. Candidate commit scopes:

```text
auto_refactor_graph_evidence:
  graph-editor/Cargo.toml
  graph-editor/src/graph.rs
  graph-editor/src/lib.rs
  graph-editor/src/autorefactor.rs
  graph-editor/src/bin/auto_refactor_plan.rs
  canon-rustc-v3/validation/auto_refactor_ops.py
  canon-rustc-v3/validation/auto_refactor_ops_smoke.py
  exactly one authoritative auto-refactor plan document, if needed

validation_harness_expectation_repair:
  src/validation_harness.rs
  tests/fixtures/validation_command_footprint_receipts.txt
  tests/fixtures/validation_duration_planning_receipts.txt
  tests/validation_harness_contract.rs
```

Do not stage both scopes together unless the next turn proves they are coupled
by tests and documents that coupling.

## Explicit Non-Goals

Do not add or alter:

```text
state-machine transition authority
kernel reducer behavior
durable TLog writer semantics
hash-chain semantics
policy promotion authority
retrieval storage mutation
retrieval query execution
runtime candidate approval
live LLM or network behavior
student-model training
provider authorization or routing
automatic source rewrite application
```

## Handoff Decision

Proceed with `auto_refactor_graph_evidence` only after focused validation proves
that the new surfaces are deterministic, advisory, non-mutating, and sorted. If
that evidence cannot be produced cleanly, defer the auto-refactor work and repair
only the validation-harness expectation drift as a separate maintenance lane.