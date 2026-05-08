# Canon Agent Score

## Planning Scorecard - 2026-05-08T12:05Z

This turn is planning and scoring only. No implementation score increase is claimed. The score remains evidence-conservative because implementation changes are present but unvalidated by focused checks.

```text
turn_type = planning_scoring
score_change_this_turn = none
commit_scope = plan.md, score.md
primary_handoff = deterministic auto-refactor graph evidence and read-only report surface
secondary_handoff = next evidence-only learning boundary after retrieval-example storage-write-commit-intent
implementation_authority_change = none
```

## Current Evidence Posture

The current plan preserves two candidate implementation directions:

1. **Structure/Efficiency:** deterministic `similar`, `phase`, and `provider` graph relations plus a read-only auto-refactor reporting surface.
2. **Learning:** a next evidence-only boundary after retrieval-example storage-write-commit-intent, without retrieval storage mutation or policy authority expansion.

Observed non-planning worktree changes are not counted as completed implementation evidence:

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

## Current Axis Scores

Scores remain unchanged in this planning turn.

```text
I  Intelligence      = 0.98
E  Efficiency        = 0.97
C  Correctness       = 0.90
A  Alignment         = 0.97
R  Robustness        = 0.96
P  Performance       = 0.95
S  Scalability       = 0.98
D  Determinism       = 0.97
T  Transparency      = 0.98
Co Collaboration     = 0.95
Em Empowerment       = 0.95
B  Benefit           = 0.97
L  Learning          = 1.00
St Structure         = 0.97
Si Simplicity        = 0.97
F  Future-Proofing   = 0.98
```

Approximate geometric mean:

```text
G ≈ 0.966
```

## Current Judgment

```text
weakest_axis = Correctness
weakest_axis_reason = candidate auto-refactor implementation is present but not validated or committed as evidence
primary_next_axis = Structure
secondary_next_axis = Efficiency
learning_boundary_status = retrieval-example storage-write-commit-intent remains the previous verified evidence-only handoff
current_gap = observed auto-refactor relation/report work lacks focused validation and commit evidence
next_action = validate and commit deterministic auto-refactor graph signals/reporting, or defer them and select the next evidence-only Learning boundary
recommended_next_lane = auto_refactor_graph_evidence
score_freeze_reason = planning turn only; no focused validation evidence added
```

## Evidence Required Before Any Score Increase

```text
- focused implementation scope selected and documented
- deterministic output across repeated runs
- healthy and controlled-regression coverage where public modes or receipts are added
- advisory/non-authoritative semantics proven for any new graph/report evidence
- cargo fmt --check passes
- RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --quiet passes
- RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test planning_contract --test score_contract --quiet passes
- focused Rust contract tests pass for touched implementation paths
- relevant Python smoke tests pass for semantic/reporting surfaces
```

## Non-Scored Items

The current worktree may contain useful candidate implementation. It remains non-scored until an implementation turn supplies validation evidence and commits the implementation deliberately.

## Scoring Rule For Next Turn

Raise scores only if the next turn provides committed implementation evidence plus validation output. Suggested axis movement, if evidence is clean:

```text
Structure: +0.01 only for deterministic sorted/deduplicated relation evidence with advisory semantics
Efficiency: +0.01 only if reporting reduces repeated manual inspection without adding runtime authority
Correctness: +0.01 only if focused healthy and controlled-regression tests pass and prove non-authoritative behavior
Learning: unchanged unless the selected lane is a verified evidence-only learning boundary
```

Do not raise any axis for uncommitted code, generated plans, unexecuted tests, live LLM output, or reports that can mutate runtime state.
