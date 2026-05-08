# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation slice from the repository state. This turn is intentionally limited to planning and scoring.

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
- deterministic policy reuse ledger summary receipt in the judgment layer.

Most recent recorded targeted validation from the prior implementation turn:

```text
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet

planning_contract: 2 passed, 0 failed
score_contract:    5 passed, 0 failed
validation_harness_contract: 128 passed, 0 failed
judgment::record unit tests: 12 passed, 0 failed
```

No code validation was run in this planning/scoring turn.

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

The weakest practical axis remains **Learning**, with **Simplicity** close behind.

Reason: the repository now has policy reuse, validation health, and judgment receipt surfaces, but the root validation/observe path still needs to expose the compact reuse-health signal directly. Until that signal is visible at the harness/catalog boundary, learning exists as typed internal evidence rather than as an obvious run-level improvement metric.

## Next Implementation Slice

Promote `PolicyReuseLedgerSummaryReceipt` into the validation harness/catalog path so the root observe report can expose one compact policy-reuse/validation-health signal without requiring consumers to join multiple receipts.

The slice should answer:

```text
For a completed run set, how much work was handled by policy, how much fell back to LLM/tooling, and did validation health regress while reuse increased?
```

Required constraints:

1. Keep the kernel untouched.
2. Do not add LLM authority or policy self-approval.
3. Add harness/catalog exposure only after preserving deterministic receipt semantics.
4. Prefer semantic assertions over brittle hash literals.
5. Keep the public signal compact enough for observe/report consumers.

Expected implementation tasks:

1. Add a validation-harness fixture or smoke path for `PolicyReuseLedgerSummaryReceipt`.
2. Register the summary signal in the dispatch/catalog evidence surface if it is not already present.
3. Add contract assertions for both healthy reuse and validation-regression cases.
4. Preserve replay/tamper coverage in judgment unit tests.
5. Run targeted validation:

```text
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet
```

## Acceptance Criteria

The next implementation turn should count as complete only if:

- the validation harness exposes policy reuse summary evidence with these semantic fields:

```text
policy_hits
policy_misses
llm_fallbacks
validation_passes
validation_failures
reuse_rate_bps
regression_flag
source_receipt_hash
```

- healthy reuse and regression cases are both represented;
- targeted tests pass with wrappers cleared;
- no kernel code changes are required;
- `plan.md` and `score.md` are updated again with actual validation evidence.

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
