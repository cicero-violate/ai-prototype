# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation slice from the repository state.

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, and validation command footprint evidence.

No implementation code was changed in this planning turn. The previous targeted validation evidence remains the most recent recorded test evidence in these planning artifacts:

```text
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet

planning_contract: 2 passed, 0 failed
score_contract:    5 passed, 0 failed
validation_harness_contract: 132 passed, 0 failed
judgment::record unit tests: 12 passed, 0 failed
```

Full-suite validation was not run in this planning turn.

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

## Priority Judgment

The weakest practical axis remains **Scalability**, with **Performance** and **Simplicity** close behind.

Reason: compact policy reuse and validation-health evidence is now visible at the harness/root-validator boundary, but the repository still lacks a deterministic larger-batch trace proving that avoided LLM calls scale without hiding validation regressions or inflating validation cost.

## Current Planning Decision

Do not expand kernel authority. The next implementation turn should add a deterministic **policy reuse scale trace** in the capability/validation-harness layer.

The planned slice should answer:

```text
When policy reuse is evaluated over a larger deterministic batch, does the system preserve validation health while increasing avoided LLM calls per batch?
```

## Next Implementation Slice

Add retained larger-batch evidence for policy reuse scaling.

Required properties:

1. Keep the kernel untouched.
2. Use deterministic fixtures or generated smoke records only.
3. Avoid live LLM, network, wall-clock, or environment-dependent calls.
4. Reuse existing judgment, validation-harness, root-validate, and retained-fixture surfaces where possible.
5. Preserve existing compact policy reuse ledger summary semantics.
6. Add a larger-batch healthy case and a larger-batch regression/cost-growth case.
7. Make the evidence readable without joining unrelated receipts.

Expected exposed fields:

```text
batch_size
policy_hits
policy_misses
llm_fallbacks
validation_passes
validation_failures
reuse_rate_bps
regression_flag
avoided_llm_calls_per_batch
source_receipt_hash
```

Candidate implementation tasks:

1. Add a `PolicyReuseScaleTraceReceipt` or equivalent compact receipt type in the judgment/capability evidence layer.
2. Add deterministic fixture constructors for:
   - healthy larger-batch reuse;
   - larger-batch validation regression;
   - larger-batch reuse with excessive fallback/cost growth, if distinct from regression.
3. Add root validator modes for the new compact trace, for example:
   - `--policy-reuse-scale-trace-smoke`
   - `--policy-reuse-scale-trace-regression-smoke`
4. Extend `validation_harness_contract` with semantic assertions for all exposed fields.
5. Add or update retained fixture files only when they make the trace easier to audit.
6. Keep planning and scoring contracts synchronized with the new score target.

Suggested targeted validation for the next implementation turn:

```text
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet
```

## Acceptance Criteria For Next Turn

The next implementation slice should be considered complete only if:

- a larger deterministic batch exposes policy-hit and fallback totals;
- avoided LLM calls per batch is explicit;
- validation failures and regression flags remain explicit;
- healthy and unhealthy paths are both covered;
- targeted validation passes;
- `plan.md` and `score.md` record the exact commands and results;
- the kernel remains unchanged unless a separate proof obligation is added.

## Deferred Work

- Full live Ollama validation remains environment-dependent.
- Graph telemetry still requires explicit wrapper capture.
- Student-model training is intentionally deferred until verified distillation data is large and clean.
- Parallel orchestration scaling should wait until larger-batch reuse and validation health are proven.
- Full-suite validation should be scheduled after the scale-trace learning signal lands.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Implement in capability/runtime-adjacent code without kernel authority drift.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.
