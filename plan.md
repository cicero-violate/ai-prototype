# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation plan after implementation step 1 of the current agent loop.

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
- deterministic policy reuse scaling projection receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled regression scaling-projection evidence;
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, and external CLI mode evidence.

## Completed Implementation Slice This Turn

Implemented deterministic **policy reuse scaling projection** evidence in the validation-harness/root-validator layer, while retaining the evaluator-savings implementation already present at the start of the turn.

This slice answers:

```text
Can an evaluator inspect one deterministic receipt that projects retained policy-reuse savings across configured orchestration batch capacity and binds that projection to evaluator-savings and capacity source evidence?
```

Implemented surfaces:

```text
PolicyReuseScalingProjectionReceipt
policy_reuse_scaling_projection_smoke_receipt()
policy_reuse_scaling_projection_regression_smoke_receipt()
--policy-reuse-scaling-projection-smoke
--policy-reuse-scaling-projection-regression-smoke
```

Implemented fields:

```text
schema
record_type
projection_version
source_evaluator_savings_hash
source_orchestration_capacity_hash
batch_capacity_limit
retained_sample_runs
retained_llm_calls_avoided
retained_cost_units_avoided
cost_units_per_llm_call
projected_llm_calls_avoided_per_full_batch
projected_reasoning_cost_units_avoided_per_full_batch
projected_llm_fallbacks_per_full_batch
projection_passed
regression_reason
projection_hash
receipt_hash
```

Completed implementation tasks:

1. Added `PolicyReuseScalingProjectionReceipt` with deterministic validation, JSON output, projection hash, and receipt hash.
2. Added healthy and controlled evaluator-savings-failed regression smoke constructors.
3. Added deterministic source capacity hashing for `PolicyOrchestrationCapacityReceipt` so projection receipts bind to capacity evidence without changing the capacity receipt schema.
4. Added root validator compact modes for healthy and regression scaling-projection receipts.
5. Added validation harness contracts for projection semantics, source binding, root-mode output, and controlled regression evidence.
6. Updated external CLI mode fixture and root compact-mode counts for the two new public modes.
7. Updated validation harness guarded-test count from 148 to 152.
8. Updated retained guarded-test fixture values from 158 to 162 and refreshed the dependent duration-planning estimate.
9. Kept kernel authority unchanged.

Implemented deterministic semantics:

- `projection_passed = true` only when evaluator-savings evidence passes, orchestration-capacity evidence passes, projected avoided calls are positive, projected avoided cost units are positive, fallback count is below full batch capacity, and `regression_reason = "none"`.
- Regression evidence remains structurally valid while exposing `projection_passed = false` and `regression_reason = "evaluator_savings_failed"`.
- Projection source hashes bind to evaluator-savings receipt hash and deterministic orchestration-capacity content hash.
- Projection estimates use fixed integer math from retained cost-per-call units and configured batch capacity.
- The new receipt is evidence-only and does not modify kernel transitions or policy-promotion authority.

## Validation Evidence For This Implementation Turn

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet

cargo fmt --check: pass
judgment::record unit tests: 20 passed, 0 failed
validation_harness_contract: 152 passed, 0 failed
planning_contract: 2 passed, 0 failed
score_contract: 5 passed, 0 failed
```

Full-suite validation was not run in this implementation turn.

## Next Implementation Slice

The weakest remaining axis is **Learning**.

Current gap:

```text
Scaling projection now shows retained and batch-level avoided LLM work, but the learning path still lacks a compact evaluator-facing distillation-readiness receipt that says whether verified reuse/savings/projection evidence is clean enough to become future policy or training data.
```

Recommended next slice:

```text
Add deterministic policy reuse distillation readiness evidence that summarizes verified reuse, evaluator savings, scaling projection, validation health, and catalog completeness into one non-authority-bearing receipt.
```

Recommended concrete surfaces:

```text
PolicyReuseDistillationReadinessReceipt
policy_reuse_distillation_readiness_smoke_receipt()
policy_reuse_distillation_readiness_regression_smoke_receipt()
--policy-reuse-distillation-readiness-smoke
--policy-reuse-distillation-readiness-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse policy reuse, cost catalog, evaluator-savings, scaling-projection, and validation-health receipts as source evidence.
4. Keep the output audit/evaluator-facing, not authority-bearing.
5. Include healthy and controlled regression/incomplete cases.
6. Do not train or invoke a student model; readiness only records whether the evidence would be clean enough later.

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
- Parallel orchestration execution should wait until larger-batch reuse, validation health, retained validation/runtime cost, catalog completeness, evaluator savings, scaling projection, and distillation readiness are proven together.
- Full-suite validation should run after the distillation-readiness slice if fixture or CLI mode churn is broader than expected.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Implement in capability/runtime-adjacent code without kernel authority drift.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.
