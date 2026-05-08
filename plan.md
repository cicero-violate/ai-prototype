# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation plan from the repository state after implementation step 1 of the current agent loop.

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

## Completed Implementation Slice This Turn

Implemented deterministic **policy reuse cost catalog summary** evidence in the judgment/capability and validation-harness layers.

This slice answers:

```text
Can an auditor inspect one deterministic summary to see which retained policy reuse/cost evidence exists, which modes expose it, and whether the healthy/regression coverage is complete?
```

Implemented surfaces:

```text
PolicyReuseCostCatalogReceipt
policy_reuse_cost_catalog_smoke_receipt()
policy_reuse_cost_catalog_incomplete_smoke_receipt()
--policy-reuse-cost-catalog-smoke
--policy-reuse-cost-catalog-incomplete-smoke
```

Implemented fields:

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

Completed implementation tasks:

1. Added `PolicyReuseCostCatalogReceipt` with deterministic hash binding and validation.
2. Exported the new receipt through judgment and crate public surfaces.
3. Added deterministic healthy and controlled incomplete catalog smoke constructors.
4. Added root validator compact modes for both catalog receipts.
5. Extended `validation_harness_contract` with catalog completeness, missing-mode, source-hash, and root-mode assertions.
6. Updated external CLI mode fixture and compact-mode counts for the two new root validator modes.
7. Updated validation harness guarded-test count from 140 to 144.
8. Updated retained guarded-test fixture values from 150 to 154 and refreshed dependent deterministic validation-duration estimates.
9. Kept kernel authority unchanged.

Implemented deterministic semantics:

- `summary_complete = true` only when required healthy modes, required regression modes, and `missing_required_modes = "none"` agree.
- Incomplete catalog evidence remains valid but does not pass.
- Source hashes bind the catalog to the existing policy reuse, scale trace, performance-cost trend, validation health, validation duration, and runtime performance evidence.
- Catalog evidence reduces audit joins without duplicating the source receipt semantics.

## Validation Evidence For This Implementation Turn

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

Full-suite validation was not run in this implementation turn.

## Next Implementation Slice

The weakest remaining axis is **Performance**.

Current gap:

```text
Reuse/cost evidence is now easier to audit through a catalog, but performance remains bounded by deterministic trend evidence rather than a retained evaluator-facing estimate of how much LLM work the cataloged policy evidence avoids at run scale.
```

Recommended next slice:

```text
Add deterministic policy reuse evaluator savings evidence that summarizes avoided LLM calls, estimated avoided reasoning cost, and validation pass/fail status from the cataloged reuse/cost evidence.
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse the cost catalog and performance-cost trend receipts as source evidence.
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
- Parallel orchestration scaling should wait until larger-batch reuse, validation health, retained validation/runtime cost, catalog completeness, and evaluator savings are proven together.
- Full-suite validation should be scheduled after the next evaluator-savings slice if fixture or CLI mode churn is broader than expected.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Implement in capability/runtime-adjacent code without kernel authority drift.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.
