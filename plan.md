# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation plan for the planning/scoring turn after implementation step 1 of the current agent loop.

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
- deterministic policy reuse distillation-readiness receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled regression distillation-readiness evidence;
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, and external CLI mode evidence.

## Current Completed Implementation Baseline

The current working tree contains deterministic **policy reuse distillation readiness** evidence in the validation-harness/root-validator layer, while retaining the evaluator-savings and scaling-projection implementation slices already present in this loop.

This slice answers:

```text
Can an evaluator inspect one deterministic receipt that says whether verified policy reuse, catalog completeness, evaluator savings, scaling projection, and validation health are clean enough to become future distillation or policy-promotion data?
```

Implemented surfaces:

```text
PolicyReuseDistillationReadinessReceipt
policy_reuse_distillation_readiness_smoke_receipt()
policy_reuse_distillation_readiness_regression_smoke_receipt()
--policy-reuse-distillation-readiness-smoke
--policy-reuse-distillation-readiness-regression-smoke
```

Implemented fields:

```text
schema
record_type
readiness_version
source_policy_reuse_hash
source_cost_catalog_hash
source_evaluator_savings_hash
source_scaling_projection_hash
source_validation_health_hash
verified_policy_hits
verified_llm_calls_avoided
projected_llm_calls_avoided_per_full_batch
projected_reasoning_cost_units_avoided_per_full_batch
validation_guarded_test_count
catalog_complete
evaluator_savings_passed
scaling_projection_passed
validation_health_passed
distillation_ready
regression_reason
readiness_hash
receipt_hash
```

Completed implementation tasks:

1. Added `PolicyReuseDistillationReadinessReceipt` with deterministic validation, JSON output, readiness hash, and receipt hash.
2. Added healthy and controlled catalog-incomplete regression smoke constructors.
3. Bound readiness evidence to policy reuse, cost catalog, evaluator-savings, scaling-projection, and validation-health source hashes.
4. Added root validator compact modes for healthy and regression readiness receipts.
5. Added validation harness contracts for readiness semantics, source binding, root-mode output, and controlled regression evidence.
6. Updated external CLI mode fixture and root compact-mode counts for the two new public modes.
7. Updated validation harness guarded-test count from 152 to 156.
8. Updated retained guarded-test fixture values from 162 to 166 and refreshed dependent validation-duration estimates.
9. Kept kernel authority unchanged and did not train a student model or promote policy.

Implemented deterministic semantics:

- `distillation_ready = true` only when catalog completeness, evaluator savings, scaling projection, validation health, and regression reason all pass.
- Regression evidence remains structurally valid while exposing `distillation_ready = false` and `regression_reason = "catalog_incomplete"`.
- Readiness source hashes bind to reuse, cost catalog, evaluator-savings, scaling projection, and validation-health evidence.
- The readiness receipt is evidence-only; it records future learning suitability without granting policy authority.
- The new receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence Recorded For This Baseline

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet

cargo fmt --check: pass
judgment::record unit tests: 20 passed, 0 failed
validation_harness_contract: 156 passed, 0 failed
planning_contract: 2 passed, 0 failed
score_contract: 5 passed, 0 failed
```

Full-suite validation was not run before this planning/scoring update.

## Planned Next Implementation Slice

The weakest remaining axis is **Simplicity**.

Current gap:

```text
The evidence surface now covers reuse, savings, projection, and distillation readiness, but public root modes and fixture-count churn are becoming a maintenance burden.
```

Recommended next slice:

```text
Add deterministic evidence-surface index or receipt-family catalog evidence that groups policy-reuse receipts by family, source dependencies, and root modes so evaluators can inspect the whole learning evidence surface without relying on scattered fixture/count knowledge.
```

Recommended concrete surfaces:

```text
PolicyReuseEvidenceSurfaceIndexReceipt
policy_reuse_evidence_surface_index_smoke_receipt()
policy_reuse_evidence_surface_index_regression_smoke_receipt()
--policy-reuse-evidence-surface-index-smoke
--policy-reuse-evidence-surface-index-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Summarize existing root modes and source dependencies instead of adding new authority.
4. Prefer reducing evaluator ambiguity over adding more independent receipt semantics.
5. Include healthy and controlled omitted-mode or dependency-missing regression cases.
6. Keep student-model training deferred.

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
- Parallel orchestration execution should wait until larger-batch reuse, validation health, retained validation/runtime cost, catalog completeness, evaluator savings, scaling projection, distillation readiness, and evidence-surface indexing are proven together.
- Full-suite validation should run after the evidence-surface index slice if fixture or CLI mode churn is broader than expected.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Implement in capability/runtime-adjacent code without kernel authority drift.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.
