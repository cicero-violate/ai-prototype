# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation plan from the repository state at the planning/scoring turn after implementation slice `d4cebb6` (`Add policy reuse cost catalog evidence`).

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, and external CLI mode evidence.

## Current Planning/Scoring Turn

No implementation code was changed in this turn. This turn refreshed the plan and score after inspecting the current repository state and prior commit history.

The most recent implementation slice added deterministic **policy reuse cost catalog summary** evidence in the judgment/capability and validation-harness layers.

That completed slice answers:

```text
Can an auditor inspect one deterministic summary to see which retained policy reuse/cost evidence exists, which modes expose it, and whether the healthy/regression coverage is complete?
```

Implemented surfaces already present:

```text
PolicyReuseCostCatalogReceipt
policy_reuse_cost_catalog_smoke_receipt()
policy_reuse_cost_catalog_incomplete_smoke_receipt()
--policy-reuse-cost-catalog-smoke
--policy-reuse-cost-catalog-incomplete-smoke
```

Implemented fields already present:

```text
catalog_version
evidence_family_count
healthy_mode_count
regression_mode_count
retained_fixture_count
required_healthy_modes_present
required_regression_modes_present
summary_complete
missing_required_modes
source_policy_reuse_hash
source_scale_trace_hash
source_performance_cost_trend_hash
source_validation_health_hash
source_validation_duration_hash
source_runtime_performance_hash
catalog_hash
receipt_hash
```

Completed implementation properties:

1. `PolicyReuseCostCatalogReceipt` has deterministic hash binding and validation.
2. Healthy and controlled incomplete catalog smoke constructors are exposed.
3. Root validator compact modes expose both catalog receipts.
4. Validation harness contracts assert catalog completeness, missing-mode, source-hash, and root-mode semantics.
5. External CLI mode and retained guarded-test fixtures were updated for the new public modes.
6. Kernel authority remained unchanged.

## Validation Evidence Carried Forward

Last targeted validation from the completed implementation slice:

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet

cargo fmt --check: pass
judgment::record unit tests: 18 passed, 0 failed
validation_harness_contract: 144 passed, 0 failed
planning_contract: 2 passed, 0 failed
score_contract: 5 passed, 0 failed
```

No validation suite was run in this planning/scoring-only turn.

## Next Implementation Slice

The weakest remaining axis is **Performance**.

Current gap:

```text
Reuse/cost evidence is now easier to audit through a catalog, but performance remains bounded by deterministic trend evidence rather than a retained evaluator-facing estimate of how much LLM work the cataloged policy evidence avoids at run scale.
```

Recommended next slice:

```text
Add deterministic policy reuse evaluator savings evidence that summarizes avoided LLM calls, estimated avoided reasoning cost, validation pass/fail status, and source catalog binding from the existing reuse/cost catalog and performance-cost trend receipts.
```

Recommended concrete surfaces:

```text
PolicyReuseEvaluatorSavingsReceipt
policy_reuse_evaluator_savings_smoke_receipt()
policy_reuse_evaluator_savings_regression_smoke_receipt()
--policy-reuse-evaluator-savings-smoke
--policy-reuse-evaluator-savings-regression-smoke
```

Recommended fields:

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

Recommended deterministic semantics:

1. `validation_passed = true` only when the source catalog is complete, avoided calls are positive, estimated savings are positive, and actual LLM calls are less than baseline LLM calls.
2. Regression evidence should remain structurally valid while exposing an explicit failure reason.
3. Hashes should bind the evaluator-savings receipt to the existing catalog, performance-cost trend, and policy-reuse source receipts.
4. The receipt must be evidence-only. It must not drive kernel transitions or approve policy promotion.
5. Estimates must be fixed deterministic units, not live cost, wall-clock, network, model-pricing, or environment-derived measurements.

Recommended validation tasks:

1. Add unit tests for healthy and regression evaluator savings receipts.
2. Add validation harness contract tests for compact root output and hash binding.
3. Update CLI mode fixture and guarded-test count only if new public root modes are added.
4. Run targeted validation:

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet
```

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
- Parallel orchestration scaling should wait until larger-batch reuse, validation health, retained validation/runtime cost, catalog completeness, and evaluator savings are proven together.
- Full-suite validation should run after the evaluator-savings slice if fixture or CLI mode churn is broader than expected.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Implement in capability/runtime-adjacent code without kernel authority drift.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.
