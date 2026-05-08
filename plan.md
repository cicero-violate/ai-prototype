# Canon Agent Implementation Plan

## Planning Turn Checkpoint - 2026-05-08 - Preserve Implementation Boundary

This planning turn keeps the repository focused on planning and scoring only. The active worktree still contains candidate implementation changes for deterministic auto-refactor graph evidence. Those files are not adopted or scored by this turn. The next implementation turn must either validate them with focused evidence and commit them deliberately, or revert/rework them before claiming progress.

Planning scope for this turn:

- update the implementation handoff around the observed auto-refactor slice;
- restore scoring to a planning-only posture;
- preserve unowned implementation changes without staging them;
- commit only `plan.md` and `score.md`.

Immediate implementation handoff remains unchanged: complete deterministic `similar`, `phase`, and `provider` graph relations, finish read-only auto-refactor report generation, prove advisory/non-authority semantics, and validate the resulting graph/report surface before any score increase.

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

## Implementation Step 2 - Retrieval Example Storage Write Commit Intent Verification

Completed this turn:

```text
Verified that the retrieval-example storage-write-commit-intent boundary is present in the tracked implementation surface:

PolicyReuseEvidenceRetrievalResultUseSummaryManifestApprovalAdmissionConsumptionLearningStorageWriteCommitIntentReceipt
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_commit_intent_smoke_receipt()
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_learning_storage_write_commit_intent_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-write-commit-intent-smoke
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-write-commit-intent-regression-smoke
```

Observed implementation semantics:

- The storage-write-commit-intent receipt consumes storage-write-admission evidence and remains deterministic and evidence-only.
- Healthy evidence exposes `retrieval_example_storage_write_commit_intent_ready = true`, status `retrieval_example_storage_write_commit_intent_ready`, and `not_ready_reason = "none"` only when storage-write-admission, storage-write-approval, storage-write-preflight, storage-commit-intent, materialization, learning admission, learning eligibility, approval-admission-consumption, approval-admission, approval, readiness, and admission evidence are present and positive.
- Regression evidence remains structurally valid while reporting `retrieval_example_storage_write_commit_intent_ready = false`, status `retrieval_example_storage_write_commit_intent_not_ready`, and a controlled not-ready reason when upstream admission evidence is not positive.
- The receipt binds to storage-write-admission, storage-write-approval, storage-write-preflight, storage-commit-intent, materialization-plan, learning-admission, learning-eligibility, approval-admission-consumption, approval-admission, approval, readiness, and admission source hashes.
- The receipt keeps retrieval storage reads/writes, query execution, runtime result approval, policy promotion, batch execution, live LLM calls, network calls, wall-clock-dependent measurement, and student training outside this boundary.
- Public root validation modes and fixture entries for healthy and regression storage-write-commit-intent evidence are already present.
- The kernel, transition table, runtime reducer, durable writer, and command ledger were not changed this turn.

Validation evidence for this implementation verification:

```text
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract storage_write_commit_intent --no-run --quiet: pass
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract storage_write_commit_intent --quiet: attempted, connector returned 502 before Rust test output was available
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo run --quiet --bin root_validate -- --policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-write-commit-intent-smoke: attempted, connector returned 502 before output was available
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo run --quiet --bin root_validate -- --policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-learning-storage-write-commit-intent-regression-smoke: attempted, connector returned 502 before output was available
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --quiet: pass
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test planning_contract --quiet: pass
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --quiet: pass
```

Current planning decision after storage-write-commit-intent verification:

Keep the next implementation turn focused on **Learning**, moving from storage-write-commit-intent evidence toward the next deterministic retrieval-example storage-write boundary while still avoiding actual retrieval storage mutation.

Current gap:

```text
Retrieval-example storage-write-commit-intent evidence is present and compile-verified, but the stack still lacks a verified next handoff after commit intent in this current loop before any actual retrieval-example storage mutation authority exists.
```

Recommended next slice:

```text
Inspect the tracked learning evidence chain after retrieval-example storage-write-commit-intent and select the next deterministic evidence-only boundary to verify or complete, without performing retrieval storage reads/writes, query execution, runtime result approval, policy promotion, batch execution, live LLM calls, network calls, wall-clock-dependent measurement, or student training.
```

Recommended constraints:

1. Keep the kernel, transition table, runtime reducer, durable writer, and command ledger untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse storage-write-commit-intent and upstream receipt hashes rather than creating mutation authority.
4. Keep any next receipt evidence-only: no retrieval storage reads/writes, no query execution, no policy promotion, no runtime result approval, no batch execution, and no student training.
5. Include healthy and controlled regression cases if a downstream boundary is selected.
6. Update external CLI mode fixtures and guarded validation counts only if new public modes or tests are added.
7. Keep unrelated `canon-rustc-v3/` working-tree changes out of this slice unless explicitly selected in a separate turn.

## Implementation Step 2 Commit Scope - Retrieval Example Storage Write Commit Intent Verification

This turn updates and commits:

```text
plan.md
score.md
```

Tracked implementation files for storage-write-commit-intent were already clean at the start of this turn. Observed `canon-rustc-v3/` working-tree changes remain outside this turn.
