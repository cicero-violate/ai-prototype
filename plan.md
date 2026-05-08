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

The repository already contains these meaningful surfaces:

- frozen-style Rust crate with `#![forbid(unsafe_code)]` at the public root;
- kernel/runtime/codec/API/capability separation exposed through `src/lib.rs`;
- deterministic planning, scoring, timing, and recovery contracts;
- API transport idempotence and replay receipts;
- graph-as-source-of-truth mutation contracts and CLI workflow tests;
- local/Ollama and OpenAI-compatible LLM effect/proof receipt surfaces;
- validation harness constants and observe-validation script contracts;
- policy, learning, eval, judgment, verification, tooling, memory, observation, and orchestration modules exported as crate surfaces;
- deterministic policy reuse ledger summary receipt in the judgment layer;
- validation-harness/root-validate smoke exposure for policy reuse ledger summary evidence, including a controlled validation-regression case.

Targeted validation run this turn:

```text
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet

planning_contract: 2 passed, 0 failed
score_contract:    5 passed, 0 failed
validation_harness_contract: 132 passed, 0 failed
judgment::record unit tests: 12 passed, 0 failed
```

Full-suite validation was not run in this implementation turn.

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

The weakest practical axis is now **Scalability**, with **Performance** and **Simplicity** close behind.

Reason: the compact learning signal is now visible through the validation harness/root validator, but the runtime still has limited evidence that policy reuse scales across larger batches, parallel orchestration, or real multi-run traces without raising validation cost.

## Next Implementation Slice

Implemented this turn: promoted `PolicyReuseLedgerSummaryReceipt` into the validation harness/root-validate catalog path so consumers can read one compact policy-reuse/validation-health signal without joining multiple receipts.

The implemented slice answers:

```text
For a completed run set, how much work was handled by policy, how much fell back to LLM/tooling, and did validation health regress while reuse increased?
```

Completed constraints:

1. Keep the kernel untouched.
2. Added harness/root-validate compact modes only.
3. Preserved deterministic receipt validation and hash binding while allowing smoke-specific record types.
4. Added semantic assertions for healthy reuse and validation-regression cases.
5. Updated retained CLI/validation fixtures for the new public modes and expected guarded-test count.

Completed implementation tasks:

1. Added `policy_reuse_ledger_summary_smoke_receipt()`.
2. Added `policy_reuse_ledger_summary_regression_smoke_receipt()`.
3. Added root validator compact modes:
   - `--policy-reuse-ledger-summary-smoke`
   - `--policy-reuse-ledger-summary-regression-smoke`
4. Added validation-harness contract assertions for all required semantic fields:
   - `policy_hits`
   - `policy_misses`
   - `llm_fallbacks`
   - `validation_passes`
   - `validation_failures`
   - `reuse_rate_bps`
   - `regression_flag`
   - `source_receipt_hash`
5. Preserved judgment-layer replay/tamper coverage through targeted judgment tests.

## Next Implementation Slice

Add a deterministic **policy reuse scale trace** that shows whether the compact summary remains useful across larger retained batches.

The next slice should answer:

```text
When policy reuse is evaluated over a larger deterministic batch, does the system preserve validation health while increasing avoided LLM calls per batch?
```

Suggested constraints:

1. Keep the kernel untouched.
2. Use retained deterministic fixtures or generated smoke records only.
3. Avoid live LLM or environment-dependent calls.
4. Reuse existing orchestration/capacity surfaces where possible.
5. Add semantic tests for batch size, policy hits, LLM fallbacks, validation failures, and avoided-call trend.

Suggested targeted validation:

```text
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet
```

## Deferred Work

- Full live Ollama validation remains environment-dependent.
- Graph telemetry still requires explicit wrapper capture.
- Student-model training is intentionally deferred until verified distillation data is large and clean.
- Orchestration scaling should wait until single-run reuse and validation health are proven.
- Full-suite validation should be scheduled after the harness/catalog learning signal lands.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Implement in capability/runtime-adjacent code without kernel authority drift.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.
