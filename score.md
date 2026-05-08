# Canon Agent Score

This scorecard is the current implementation baseline, not a claim of full-system completion.

## Validation Evidence

Targeted validation run this turn:

```text
command = cargo fmt --check
command = RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
command = RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
command = RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet
result  = pass

cargo fmt --check: pass
judgment::record unit tests: 14 passed, 0 failed
validation_harness_contract: 136 passed, 0 failed
planning_contract: 2 passed, 0 failed
score_contract: 5 passed, 0 failed
```

Full-suite validation was not run in this implementation turn.

## Axis Scores

Scores are on a `0.00` to `1.00` scale and reflect the current repository evidence inspected this turn.

```text
I  Intelligence      = 0.68  larger-batch policy reuse scale evidence now shows avoided LLM work explicitly
E  Efficiency        = 0.66  avoided LLM calls per batch are exposed directly in the scale trace
C  Correctness       = 0.82  targeted judgment, validation-harness, score, and planning contracts pass
A  Alignment         = 0.82  kernel authority remains unchanged; evidence stays in capability/validation layers
R  Robustness        = 0.76  healthy and controlled-regression scale-trace paths are both covered
P  Performance       = 0.59  retained validation cost fixtures were updated, but no new cost trend joins scale with runtime budget
S  Scalability       = 0.63  larger deterministic retained batches now expose reuse, fallback, and avoided-call totals
D  Determinism       = 0.85  scale trace uses deterministic smoke records, hash binding, and fixture-backed root modes
T  Transparency      = 0.82  batch size, reuse, fallback, validation health, avoided calls, and source hash are in compact output
Co Collaboration     = 0.71  plan and score now hand off a concrete performance-cost trend slice
Em Empowerment       = 0.67  consumers can query root_validate for larger-batch reuse health without joining receipts
B  Benefit           = 0.73  implementation improves direct measurement of reasoning-cost reduction
L  Learning          = 0.65  verified reuse evidence is now visible over larger deterministic batches
St Structure         = 0.75  scale trace is isolated to judgment/validation surfaces with explicit contracts
Si Simplicity        = 0.61  compact scale output reduces audit joins, but cost correlation still requires separate fixtures
F  Future-Proofing   = 0.76  layered architecture remains intact while adding scalable evidence hooks
```

Approximate geometric mean:

```text
G ≈ 0.715
```

## Current Judgment

```text
weakest_axis = Performance
secondary_risk = Simplicity
completed_action = added deterministic policy reuse scale trace over larger retained batches
next_action = add deterministic policy reuse performance-cost trend evidence
scope = capability/validation-harness evidence only; do not change kernel authority
validation = targeted judgment/validation-harness/score/planning tests passed; full suite not run this turn
```

## Why Performance Is Weakest

The new scale trace proves that larger deterministic batches can expose policy hits, misses, LLM fallbacks, validation failures, regression flags, and avoided LLM calls per batch. The remaining weak point is proving that this avoided work scales without increasing retained validation/runtime cost beyond budget.

## Next Score Improvement Target

Raise `P` from `0.59` toward `0.64` by adding a deterministic performance-cost trend receipt that joins larger-batch reuse evidence with retained validation/runtime cost evidence.

The next improvement should only count if the harness exposes these fields for a deterministic retained batch:

```text
batch_size
avoided_llm_calls_per_batch
reuse_rate_bps
validation_expected_count_guarded_tests
estimated_ms_per_guarded_test
runtime_budget_status
validation_cost_verdict
cost_regression_flag
source_scale_trace_hash
source_validation_duration_hash
```

Required scoring evidence for the next implementation turn:

- retained larger-batch healthy reuse/cost case exposed by the harness;
- retained larger-batch cost-regression case exposed by the harness;
- targeted validation commands and results recorded in this file;
- no kernel authority expansion.
