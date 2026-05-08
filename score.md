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
judgment::record unit tests: 16 passed, 0 failed
validation_harness_contract: 140 passed, 0 failed
planning_contract: 2 passed, 0 failed
score_contract: 5 passed, 0 failed
```

Full-suite validation was not run in this implementation turn.

## Axis Scores

Scores are on a `0.00` to `1.00` scale and reflect the current repository evidence inspected this turn.

```text
I  Intelligence      = 0.69  performance-cost trend evidence makes policy reuse quality more actionable
E  Efficiency        = 0.68  avoided LLM calls are now joined to retained validation/runtime cost evidence
C  Correctness       = 0.83  targeted judgment, validation-harness, score, and planning contracts pass
A  Alignment         = 0.83  kernel authority remains unchanged; evidence stays in capability/validation layers
R  Robustness        = 0.77  healthy and controlled-regression performance-cost paths are covered
P  Performance       = 0.64  deterministic trend receipt ties reuse scale to validation/runtime cost within budget
S  Scalability       = 0.65  larger deterministic retained batches now expose reuse plus cost safety in one receipt
D  Determinism       = 0.86  cost trend uses deterministic smoke records and hash-bound source receipts
T  Transparency      = 0.84  reuse, guarded-test count, runtime budget, validation verdict, and source hashes are compactly visible
Co Collaboration     = 0.73  plan and score now hand off the next weakest-axis slice after cost-trend completion
Em Empowerment       = 0.69  consumers can query root_validate for reuse/cost health without manual receipt joins
B  Benefit           = 0.74  implementation improves direct measurement of reasoning-cost reduction under budget
L  Learning          = 0.66  verified reuse evidence is now linked to cost-safety promotion signals
St Structure         = 0.76  trend evidence is isolated to judgment/validation surfaces with explicit contracts
Si Simplicity        = 0.64  compact performance-cost output reduces audit joins across retained fixtures
F  Future-Proofing   = 0.77  layered architecture remains intact while adding scalable cost evidence hooks
```

Approximate geometric mean:

```text
G ≈ 0.730
```

## Current Judgment

```text
turn_type = implementation step 1
weakest_axis = Simplicity
secondary_risk = Scalability
completed_action = added deterministic policy reuse performance-cost trend evidence
current_gap = cost-trend evidence is compact, but retained fixture/cost surfaces are still spread across several fixture families
next_action = add a retained policy reuse performance-cost fixture or catalog summary if the next loop continues evidence consolidation
scope = capability/validation-harness evidence only; kernel authority unchanged
validation = targeted judgment/validation-harness/score/planning tests passed; full suite not run this turn
```

## Why Simplicity Is Now Weakest

Performance improved because the new receipt joins larger-batch avoided LLM calls with retained validation/runtime cost signals. The next weak point is simplicity: related retained evidence still exists across separate fixture families, and a consumer may still need fixture/catalog context for broader audit workflows.

## Completed Score Improvement Target

Raised `P` from `0.59` to `0.64` by adding a deterministic performance-cost trend receipt that joins larger-batch reuse evidence with retained validation/runtime cost evidence.

The harness now exposes these fields for deterministic retained batches:

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
source_runtime_performance_hash
```

Completed scoring evidence:

- retained larger-batch healthy reuse/cost case exposed by the harness;
- retained larger-batch cost-regression case exposed by the harness;
- targeted validation commands and results recorded in this file;
- no kernel authority expansion.

## Acceptance Criteria Completed This Turn

1. A deterministic healthy performance-cost trend receipt exists.
2. A deterministic controlled cost-regression receipt exists.
3. Root validator compact modes expose both receipts.
4. `validation_harness_contract` asserts the semantic fields and source hashes.
5. Planning and score contract tests pass.
6. The kernel state machine remains unchanged.
