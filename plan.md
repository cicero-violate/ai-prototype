# Canon Agent Implementation Plan

## Planning Checkpoint - 2026-05-08T12:10Z

This turn is planning and scoring only. It does not adopt, validate, or score the implementation changes currently present outside `plan.md` and `score.md`. Its purpose is to keep the next implementation turn constrained to deterministic evidence production rather than opportunistic runtime expansion.

Committed scope for this turn:

```text
plan.md
score.md
```

The active project direction remains Canon Agent as a deterministic, evidence-backed, self-improving runtime where the state-machine kernel governs correctness and the capability layer accumulates intelligence without gaining authority over the kernel.

## Boundary To Preserve

The next implementation work must preserve these design boundaries:

1. The kernel owns transition authority and correctness enforcement.
2. The transaction log records structured, typed, hash-chained evidence.
3. The capability layer may propose, evaluate, summarize, and learn from evidence.
4. LLM output is proposal evidence only; it is never approval authority.
5. Policy promotion requires external validation evidence.
6. Retrieval examples remain evidence-bound and non-mutating until an explicit storage authority boundary is validated.
7. Student-model training remains out of scope until the dataset is large, clean, and externally validated.

## Current Worktree Evidence

Observed non-planning implementation changes remain present in the worktree:

```text
modified: canon-rustc-v3/src/facts.rs
modified: canon-rustc-v3/src/hir.rs
modified: canon-rustc-v3/src/mir.rs
modified: canon-rustc-v3/src/wrapper.rs
modified: canon-rustc-v3/validation/semantic_preflight.py
modified: canon-rustc-v3/validation/semantic_scale_probe.py
untracked: canon-rustc-v3/plan-autorefactor.md
untracked: canon-rustc-v3/validation/auto_refactor_surface.py
untracked: canon-rustc-v3/validation/auto_refactor_surface_smoke.py
```

These files appear to target graph-guided auto-refactor relation evidence and read-only semantic/reporting surfaces. They are not scored in this planning turn. The next implementation turn must either validate and commit them deliberately, or defer/revert them before selecting another lane.

## Root Validation Evidence For Planning Artifacts

The existing root planning and score contracts were run after inspecting the current planning artifacts:

```text
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test planning_contract --test score_contract --quiet

planning_contract: pass, 2 tests
score_contract: pass, 5 tests
```

This validates the root planning/scoring contract files only. It does not validate the uncommitted `canon-rustc-v3` implementation changes.

## Primary Next Slice: Deterministic Auto-Refactor Graph Evidence

The recommended next implementation turn should focus on **Structure** first and **Efficiency** second.

Target outcome:

```text
similar, phase, and provider relations are emitted as deterministic graph metadata and consumed only by read-only advisory reporting surfaces.
```

Implementation steps:

1. Inspect `canon-rustc-v3/plan-autorefactor.md` and decide whether it is the authoritative lane plan or only a draft.
2. Confirm the relation vocabulary for `similar`, `phase`, and `provider` in the graph/facts layer.
3. Ensure relation extraction and serialization are stable, sorted, and deduplicated across repeated runs.
4. Define relation semantics explicitly:
   - `similar` = heuristic duplicate or merge signal only;
   - `phase` = split-boundary or refactor-stage guidance only;
   - `provider` = provenance or boundary metadata only.
5. Prove that these relations cannot alter kernel transitions, reducer behavior, authorization, retry behavior, provider routing, policy promotion, retrieval writes, or model training.
6. Finish `canon-rustc-v3/validation/auto_refactor_surface.py` as a read-only report generator over graph JSON.
7. Make report output deterministic: sorted objects, stable grouping, no mutation path, no network dependency, and no live LLM dependency.
8. Finish `canon-rustc-v3/validation/auto_refactor_surface_smoke.py` with deterministic fixture coverage for healthy and controlled edge cases.
9. Explain any changes to `semantic_preflight.py` and `semantic_scale_probe.py` as validation/reporting coverage, not weakened risk handling.
10. Commit implementation only after focused validation evidence is clean.

## Secondary Next Slice: Learning Evidence Boundary

If the next turn selects Learning instead of auto-refactor evidence, inspect the tracked evidence chain after retrieval-example storage-write-commit-intent and choose the next deterministic evidence-only handoff.

Constraints for any Learning continuation:

```text
- no retrieval storage mutation
- no retrieval query execution
- no runtime result approval
- no policy promotion authority
- no batch execution
- no live LLM or network call
- no wall-clock-dependent measurement
- no student-model training
```

Any new public validation mode must include healthy evidence, controlled-regression evidence, and explicit upstream receipt-hash binding.

## Validation Gate Before Any Implementation Score Increase

Minimum evidence before raising the implementation score:

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test planning_contract --test score_contract --quiet
focused Rust contract tests for touched evidence paths
relevant Python smoke tests for semantic/reporting surfaces
stable repeated output evidence for graph/report generation
explicit proof that new evidence remains advisory and non-authoritative
```

Additional gate if the auto-refactor slice is selected:

```text
- relation vocabulary includes similar, phase, and provider with deterministic serialization
- graph extraction emits sorted, deduplicated relation evidence across repeated runs
- auto_refactor_surface.py reads graph JSON and writes stable sorted JSON only
- auto_refactor_surface_smoke.py covers healthy and controlled edge cases
- semantic_preflight.py accepts the new relation vocabulary without weakening checks
- semantic_scale_probe.py reports the new surface without hiding risk or scale regressions
```

## Explicit Non-Goals

Do not incidentally add or alter:

```text
state-machine kernel authority
transition table semantics
runtime reducer behavior
durable writer behavior
hash-chain transaction log semantics
policy promotion authority
retrieval storage mutation
retrieval query execution
runtime result approval
live LLM or network behavior
student-model training
provider authorization or routing
```

## Current Decision

No implementation progress is claimed in this planning turn. The next useful turn should either validate and commit the deterministic auto-refactor graph/reporting work as advisory evidence, or defer/revert that work and select the next evidence-only Learning boundary.

## Handoff Checklist For Next Agent Turn

Before modifying implementation files, the next agent should choose exactly one lane:

```text
lane = auto_refactor_graph_evidence | learning_evidence_boundary | cleanup_defer
```

Lane-specific entry criteria:

```text
auto_refactor_graph_evidence:
  - inspect canon-rustc-v3/plan-autorefactor.md if present
  - verify modified facts/hir/mir/wrapper files preserve read-only evidence semantics
  - complete deterministic relation/report validation before commit

learning_evidence_boundary:
  - leave auto-refactor work untouched or explicitly defer it
  - select one evidence-only receipt boundary
  - keep retrieval writes, policy promotion, and model training disabled

cleanup_defer:
  - make no implementation changes
  - document why observed worktree changes are deferred
  - preserve existing score unless validation evidence is added later
```

The default recommendation is `auto_refactor_graph_evidence` because current uncommitted files already point at that lane and can be evaluated without granting new runtime authority.
