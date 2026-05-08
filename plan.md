# Canon Agent Implementation Plan

This is the planning turn for the next agent loop. It records the current repository baseline, the selected next implementation slice, and the acceptance criteria for that slice.

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, and external CLI mode evidence.

## Last Completed Implementation Slice

The last completed slice added deterministic **policy reuse performance-cost trend** evidence in the judgment/capability and validation-harness layers.

That slice answers:

```text
When policy reuse scales across retained batches, does avoided LLM work grow without increasing validation/runtime cost beyond budget?
```

Completed implementation surfaces:

```text
PolicyReusePerformanceCostTrendReceipt
policy_reuse_performance_cost_trend_smoke_receipt()
policy_reuse_performance_cost_trend_regression_smoke_receipt()
--policy-reuse-performance-cost-trend-smoke
--policy-reuse-performance-cost-trend-regression-smoke
```

Important exposed fields:

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

Recorded targeted validation from the prior implementation turn:

```text
cargo fmt --check: pass
judgment::record unit tests: 16 passed, 0 failed
validation_harness_contract: 140 passed, 0 failed
planning_contract: 2 passed, 0 failed
score_contract: 5 passed, 0 failed
```

Full-suite validation was not run in that implementation turn.

## Current Planning Turn

This turn is intentionally limited to planning and scoring. No implementation code is planned for this turn.

The next implementation slice should improve the weakest current axis: **Simplicity**.

Current gap:

```text
Reuse, scale, validation-health, validation-duration, runtime-performance, and cost-trend evidence are individually deterministic, but audit consumers still need to know several fixture families and root validator modes to understand the complete reuse/cost story.
```

Selected next slice:

```text
Add a deterministic retained policy reuse cost catalog summary that consolidates the existing reuse/scale/cost fixture family into a compact, hash-bound audit surface.
```

The slice should not add new policy authority or kernel behavior. It should only add deterministic evidence consolidation for audit and validation consumers.

## Proposed Next Implementation Slice

### Name

```text
policy reuse cost catalog summary
```

### Primary Question

```text
Can an auditor inspect one deterministic summary to see which retained policy reuse/cost evidence exists, which modes expose it, and whether the healthy/regression coverage is complete?
```

### Scope

Implement a compact deterministic receipt and validation-harness exposure that catalogues the retained reuse/cost evidence family.

Recommended receipt name:

```text
PolicyReuseCostCatalogReceipt
```

Recommended deterministic constructors:

```text
policy_reuse_cost_catalog_smoke_receipt()
policy_reuse_cost_catalog_incomplete_smoke_receipt()
```

Recommended root validator modes:

```text
--policy-reuse-cost-catalog-smoke
--policy-reuse-cost-catalog-incomplete-smoke
```

Recommended fields:

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
```

### Constraints

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent evidence.
3. Reuse existing deterministic smoke receipts and retained fixtures.
4. Preserve existing compact root validator modes.
5. Keep the new receipt read-only and audit-oriented.
6. Include a healthy complete catalog and a controlled incomplete catalog.
7. Bind the catalog to source evidence hashes rather than duplicating source semantics.
8. Avoid broad fixture churn unless the external mode or guarded-test count fixtures must be extended.

### Acceptance Criteria

1. A deterministic healthy catalog receipt exists and validates successfully.
2. A deterministic incomplete catalog receipt exists and exposes the missing coverage explicitly.
3. Root validator exposes both compact modes.
4. `validation_harness_contract` asserts catalog completeness semantics and source hashes.
5. Existing planning and scoring contract tests pass after plan/score updates.
6. The kernel state machine remains unchanged.
7. The new summary reduces audit joins across retained reuse/cost fixture families.

### Suggested Targeted Validation

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet
```

Full-suite validation can remain deferred unless the implementation touches runtime, API, graph mutation, or script surfaces.

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
- Parallel orchestration scaling should wait until larger-batch reuse, validation health, retained validation/runtime cost, and catalog completeness are proven together.
- Full-suite validation should be scheduled after the catalog summary lands if fixture or CLI mode churn is broader than expected.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Implement in capability/runtime-adjacent code without kernel authority drift.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.