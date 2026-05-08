# Canon Agent Implementation Plan

## Planning Turn Update - 2026-05-08 - Auto-Refactor Evidence Handoff

This planning turn updates only `plan.md` and `score.md`. Existing implementation changes in the worktree are treated as observed evidence for the next implementation turn, not as changes owned, validated, or scored by this turn.

## Current Objective

Advance Canon Agent toward deterministic, evidence-backed self-improvement while preserving the architecture boundary:

- the state machine kernel governs correctness;
- the capability layer performs reasoning and learning;
- the transaction log records typed evidence;
- policy is promoted only from externally validated outcomes;
- LLM output remains proposal/input evidence, never self-approval authority.

## Observed Worktree Evidence

The repository currently contains unowned implementation changes outside this planning turn:

```text
modified: canon-rustc-v3/src/facts.rs
modified: canon-rustc-v3/src/hir.rs
modified: canon-rustc-v3/src/mir.rs
modified: canon-rustc-v3/src/wrapper.rs
modified: canon-rustc-v3/validation/semantic_preflight.py
modified: canon-rustc-v3/validation/semantic_scale_probe.py
modified: src/validation_harness.rs
modified: tests/validation_harness_contract.rs
untracked: canon-rustc-v3/plan-autorefactor.md
untracked: canon-rustc-v3/validation/auto_refactor_surface.py
untracked: canon-rustc-v3/validation/auto_refactor_surface_smoke.py
```

These files appear to target graph-guided auto-refactor evidence and validation-harness coverage. The next implementation turn should either validate and commit this work or remove/rework it.

## Current Implementation Plan

1. Keep the next implementation turn scoped to **Structure** first and **Efficiency** second.
2. Resolve candidate `similar`, `phase`, and `provider` relations into deterministic graph evidence.
3. Preserve non-authority semantics for all new graph relations:
   - `similar` is a heuristic duplicate/merge signal only;
   - `phase` is split-boundary guidance only;
   - `provider` is boundary/provenance metadata only.
4. Ensure none of these relations can alter kernel transitions, reducer behavior, authorization, retry behavior, live provider routing, policy promotion, retrieval storage, or model training.
5. Finish `canon-rustc-v3/validation/auto_refactor_surface.py` as a read-only report surface over graph JSON.
6. Make report output stable: sorted objects, deterministic grouping, no mutation path, no network dependency, no live LLM dependency.
7. Finish `auto_refactor_surface_smoke.py` with deterministic fixtures or deterministic in-process fixture generation.
8. Explain and validate any changes to `semantic_preflight.py`, `semantic_scale_probe.py`, `src/validation_harness.rs`, and `tests/validation_harness_contract.rs` as contract coverage, not incidental drift.
9. Run focused validation before any implementation commit.
10. Commit implementation only if validation evidence is clean; otherwise record exact blockers and keep unvalidated implementation out of the committed baseline.

## Validation Gate For The Next Implementation Turn

Required evidence before scoring implementation progress:

```text
- relation vocabulary includes similar, phase, and provider with deterministic serialization
- graph extraction emits stable, sorted, deduplicated relation evidence across repeated runs
- similar/phase/provider are classified as advisory graph metadata, not kernel authority
- auto_refactor_surface.py reads graph JSON and writes stable sorted JSON only
- auto_refactor_surface_smoke.py has complete deterministic coverage
- semantic_preflight.py accepts the relation vocabulary without weakening risk checks
- semantic_scale_probe.py reports the new surface without hiding risk or scale regressions
- validation_harness.rs changes are tied to explicit contract evidence
- tests/validation_harness_contract.rs covers the new evidence path
- cargo fmt --check passes
- RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --quiet passes
- RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test planning_contract --test score_contract --quiet passes
- relevant validation-harness and graph/semantic Python smoke tests pass
```

## Explicit Non-Goals

The next implementation turn must not incidentally add or alter:

- state-machine kernel authority;
- transition table semantics;
- durable writer behavior;
- hash-chain transaction log semantics;
- policy promotion authority;
- retrieval writes or retrieval approval;
- live LLM/network behavior;
- student-model training;
- provider authorization or routing.

## Planning Decision

No implementation score increase is claimed in this turn. The next useful turn should convert the observed candidate auto-refactor work into validated, evidence-backed Structure/Efficiency progress or remove it from the worktree.
