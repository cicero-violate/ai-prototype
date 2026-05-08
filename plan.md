# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation plan from the repository state at this planning/scoring turn. The working tree already contains an uncommitted evaluator-savings implementation slice; this turn records the plan, score, and next implementation target without expanding implementation scope.

## North Star

Canon Agent is a deterministic, self-improving runtime where:

- the state-machine kernel governs all control flow;
- capabilities produce typed evidence, never unchecked authority;
- every decision is replayable through hash-chained logs and receipts;
- learning promotes only externally verified wins into append-only policy;
- LLM calls shrink over time as policy handles repeated cases.

The architecture must keep safety and intelligence separated: the kernel enforces correctness, the capability layer accumulates intelligence, and policy learning never grants itself authority.

## Current Implementation Baseline

The repository currently exposes these meaningful surfaces:

- frozen-style Rust crate with `#![forbid(unsafe_code)]` at the public root;
- kernel/runtime/codec/API/capability separation through `src/lib.rs`;
- deterministic planning, scoring, timing, recovery, reducer, transition-table, durable-log, and verification contracts;
- API transport idempotence and replay receipts;
- graph-as-source-of-truth mutation contracts and CLI workflow tests;
- local/Ollama and OpenAI-compatible LLM effect/proof receipt surfaces;
- validation harness constants and observe-validation script contracts;
- policy, learning, eval, judgment, verification, tooling, memory, observation, and orchestration modules exported as crate surfaces;
- deterministic policy reuse ledger summary receipt in the judgment layer;
- validation-harness/root-validate smoke exposure for policy reuse ledger summary evidence, including a controlled validation-regression case;
- deterministic policy reuse scale trace receipt in the judgment layer;
- validation-harness/root-validate smoke exposure for larger-batch policy reuse scale evidence, including a controlled validation-regression case;
- deterministic policy reuse performance-cost trend receipt in the judgment layer;
- validation-harness/root-validate smoke exposure for reuse/cost trend evidence, including a controlled cost-regression case;
- deterministic policy reuse cost catalog receipt in the judgment layer;
- validation-harness/root-validate smoke exposure for complete and incomplete reuse/cost catalog evidence;
- deterministic policy reuse evaluator savings receipt in the judgment layer;
- validation-harness/root-validate smoke exposure for healthy and controlled regression evaluator-savings evidence;
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, and external CLI mode evidence.

## Current Implemented Slice In The Working Tree

The working tree contains deterministic **policy reuse evaluator savings** evidence in the judgment/capability and validation-harness layers.

This slice answers:

```text
Can an evaluator inspect one deterministic receipt showing how much LLM work policy reuse avoided, whether savings evidence passed, and which reuse/cost catalog sources bind the estimate?
```

Implemented surfaces:

```text
PolicyReuseEvaluatorSavingsReceipt
policy_reuse_evaluator_savings_smoke_receipt()
policy_reuse_evaluator_savings_regression_smoke_receipt()
--policy-reuse-evaluator-savings-smoke
--policy-reuse-evaluator-savings-regression-smoke
```

Implemented fields:

```text
savings_version
source_catalog_hash
source_performance_cost_trend_hash
source_policy_reuse_hash
sample_runs
policy_hits
llm_calls_avoided
estimated_reasoning_cost_units_avoided
baseline_llm_calls
actual_llm_calls
llm_call_reduction_ratio_bps
validation_passed
regression_reason
savings_hash
receipt_hash
```

Completed implementation tasks:

1. Added `PolicyReuseEvaluatorSavingsReceipt` with deterministic content and receipt hash binding.
2. Exported the new receipt through the judgment public surface.
3. Added deterministic healthy and controlled catalog-incomplete regression smoke constructors.
4. Added root validator compact modes for both evaluator-savings receipts.
5. Extended `validation_harness_contract` with savings semantics, source binding, and compact root-mode assertions.
6. Updated external CLI mode fixture and compact-mode counts for the two new root validator modes.
7. Updated validation harness guarded-test count from 144 to 148.
8. Updated retained guarded-test fixture values from 154 to 158 and refreshed dependent deterministic validation-duration estimates.
9. Kept kernel authority unchanged.

Implemented deterministic semantics:

- `validation_passed = true` only when source evidence passes, avoided calls are positive, estimated savings are positive, actual LLM calls are below baseline LLM calls, and `regression_reason = "none"`.
- Regression evidence remains structurally valid while exposing `validation_passed = false` and a concrete `regression_reason`.
- Source hashes bind the savings receipt to the cost catalog, performance-cost trend, and policy-reuse receipts.
- Savings estimates are fixed deterministic units, not live pricing, wall-clock, network, or model-derived measurements.
- The new receipt is evidence-only and does not modify kernel transitions or policy-promotion authority.

## Validation Evidence For This Planning/Scoring Turn

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet

cargo fmt --check: pass
judgment::record unit tests: 20 passed, 0 failed
validation_harness_contract: 148 passed, 0 failed
planning_contract: 2 passed, 0 failed
score_contract: 5 passed, 0 failed
```

Full-suite validation was not run in this planning/scoring turn.

## Next Implementation Slice

The weakest remaining axis is **Scalability**.

Current gap:

```text
Evaluator savings evidence now quantifies avoided LLM work for the retained reuse sample, but scalability evidence still lacks a compact cross-batch projection that combines savings, catalog completeness, and orchestration capacity into one evaluator-facing receipt.
```

Recommended next slice:

```text
Add deterministic policy reuse scaling projection evidence that summarizes projected avoided LLM calls and estimated savings across configured batch capacity, bound to evaluator-savings and orchestration-capacity source receipts.
```

Recommended concrete surfaces:

```text
PolicyReuseScalingProjectionReceipt
policy_reuse_scaling_projection_smoke_receipt()
policy_reuse_scaling_projection_regression_smoke_receipt()
--policy-reuse-scaling-projection-smoke
--policy-reuse-scaling-projection-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse evaluator-savings, cost catalog, and orchestration-capacity receipts as source evidence.
4. Keep the output audit/evaluator-facing, not authority-bearing.
5. Include healthy and controlled regression/incomplete cases.
6. Avoid broad fixture churn unless public root modes or retained counts must change.

## Evaluation Axes

```text
I  = Intelligence
E  = Efficiency
C  = Correctness
A  = Alignment
R  = Robustness
P  = Performance
S  = Scalability
D  = Determinism
T  = Transparency
Co = Collaboration
Em = Empowerment
B  = Benefit
L  = Learning
St = Structure
Si = Simplicity
F  = Future-Proofing
```

```text
G = (I*E*C*A*R*P*S*D*T*Co*Em*B*L*St*Si*F)^(1/16)
arg max(G) = good
```

## Deferred Work

- Full live Ollama validation remains environment-dependent.
- Graph telemetry still requires explicit wrapper capture.
- Student-model training is intentionally deferred until verified distillation data is large and clean.
- Parallel orchestration scaling should wait until larger-batch reuse, validation health, retained validation/runtime cost, catalog completeness, evaluator savings, and scaling projection are proven together.
- Full-suite validation should run after the scaling-projection slice if fixture or CLI mode churn is broader than expected.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Implement in capability/runtime-adjacent code without kernel authority drift.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.
