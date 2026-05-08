# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation plan from the repository state.

## North Star

Canon Agent is a deterministic, self-improving runtime where:

- the state-machine kernel governs all control flow;
- capabilities produce typed evidence, never unchecked authority;
- every decision is replayable through hash-chained logs and receipts;
- learning promotes only externally verified wins into append-only policy;
- LLM calls shrink over time as policy handles repeated cases.

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, and external CLI mode evidence.

## Last Completed Implementation Slice

The last completed slice added deterministic **policy reuse scale trace** evidence in the judgment/capability and validation-harness layers.

That slice answers:

```text
When policy reuse is evaluated over a larger deterministic batch, does the system preserve validation health while increasing avoided LLM calls per batch?
```

Completed constraints:

1. Kept the kernel untouched.
2. Used deterministic generated smoke records only.
3. Avoided live LLM, network, wall-clock, or environment-dependent calls.
4. Reused existing judgment, validation-harness, root-validate, and retained-fixture surfaces.
5. Preserved compact policy reuse ledger summary semantics.
6. Added a larger-batch healthy case and a larger-batch validation-regression case.
7. Made scale evidence readable without joining unrelated receipts.

Completed implementation tasks:

1. Added `PolicyReuseScaleTraceReceipt` with deterministic hash binding and validation.
2. Exported the new receipt through judgment and crate public surfaces.
3. Added deterministic constructors:
   - `policy_reuse_scale_trace_smoke_receipt()`
   - `policy_reuse_scale_trace_regression_smoke_receipt()`
4. Added root validator compact modes:
   - `--policy-reuse-scale-trace-smoke`
   - `--policy-reuse-scale-trace-regression-smoke`
5. Extended `validation_harness_contract` with semantic assertions for:
   - `batch_size`
   - `policy_hits`
   - `policy_misses`
   - `llm_fallbacks`
   - `validation_passes`
   - `validation_failures`
   - `reuse_rate_bps`
   - `regression_flag`
   - `avoided_llm_calls_per_batch`
   - `source_receipt_hash`
6. Updated retained external CLI mode, validation command footprint, validation duration planning, and policy validation health fixtures for the new deterministic public modes and guarded-test count.

## Current Planning Turn

This turn is intentionally limited to planning and scoring. No source implementation changes are planned in this turn.

Planning objective:

```text
Select the smallest next implementation slice that improves the weakest current axis without changing kernel authority.
```

Current diagnosis:

- Weakest axis: **Performance**.
- Secondary risk: **Simplicity**.
- Adjacent risk: **Scalability**.

Reason: larger-batch policy reuse evidence now exposes avoided LLM calls and validation health, but the repository still lacks one deterministic receipt that joins reuse-scale wins to retained validation/runtime cost. Consumers must infer cost safety by reading separate fixtures, which weakens performance scoring and increases audit friction.

## Next Implementation Slice

Add deterministic **policy reuse performance-cost trend** evidence that ties larger-batch avoided LLM calls to retained validation/runtime cost.

The next slice should answer:

```text
When policy reuse scales across retained batches, does avoided LLM work grow without increasing validation/runtime cost beyond budget?
```

Implementation constraints:

1. Keep the kernel untouched.
2. Use deterministic fixture or smoke evidence only.
3. Reuse existing runtime performance, validation duration, policy capacity/cost, and scale-trace surfaces.
4. Avoid live LLM, network, wall-clock, or environment-dependent measurements.
5. Expose cost trend fields in one compact receipt where possible.
6. Include healthy and controlled cost-regression cases.
7. Preserve existing root validator and retained-fixture semantics unless a fixture must be extended for the new public evidence.

Suggested receipt name:

```text
PolicyReusePerformanceCostTrendReceipt
```

Suggested constructors:

```text
policy_reuse_performance_cost_trend_smoke_receipt()
policy_reuse_performance_cost_trend_regression_smoke_receipt()
```

Suggested root validator modes:

```text
--policy-reuse-performance-cost-trend-smoke
--policy-reuse-performance-cost-trend-regression-smoke
```

Suggested exposed fields:

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

Suggested deterministic semantics:

- `cost_regression_flag = false` only when validation cost and runtime budget are both healthy.
- `cost_regression_flag = true` when avoided LLM calls are visible but retained validation/runtime cost exceeds the deterministic budget.
- Source hashes must bind the cost-trend receipt to the underlying scale trace and retained duration/performance evidence.
- The receipt should make the healthy/regression distinction visible without requiring consumers to join unrelated receipts manually.

Suggested targeted validation for the next implementation turn:

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet
```

## Validation Evidence For This Planning Turn

No implementation validation was required for this planning-only turn.

Planning/scoring file validation:

```text
files inspected = plan.md, score.md, repository file list, git status
files changed   = plan.md, score.md
source changes  = none planned
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
- Parallel orchestration scaling should wait until larger-batch reuse, validation health, and retained validation/runtime cost are proven together.
- Full-suite validation should be scheduled after the performance-cost learning signal lands.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Implement in capability/runtime-adjacent code without kernel authority drift.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.
