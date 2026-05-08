# Canon Agent Implementation Plan

This planning turn keeps implementation work out of scope and sets the next deterministic slice from the current repository state.

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
- policy, learning, eval, judgment, verification, tooling, memory, observation, and orchestration modules exported as crate surfaces.

Targeted validation run this turn:

```text
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet

planning_contract: 2 passed, 0 failed
score_contract:    5 passed, 0 failed
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
Si = Simplicity
F  = Future-Proofing
```

```text
G = (I*E*C*A*R*P*S*D*T*Co*Em*B*L*Si*F)^(1/15)
arg max(G) = good
```

## Priority Judgment

The weakest practical axis is now **Learning**.

Reason: many typed surfaces for policy, judgment, eval, receipts, and validation already exist, but the architecture goal depends on demonstrating measurable policy reuse over repeated runs. The next slice should make the cost-reduction loop more explicit without relaxing deterministic gates.

## Next Implementation Slice

Implement a minimal deterministic **policy reuse ledger summary** that answers:

```text
For a completed run set, how many eligible decisions were handled by policy, how many missed to LLM/tooling, and did reuse improve without validation regression?
```

Suggested constraints:

1. Keep the kernel untouched.
2. Add or extend a capability-layer receipt only.
3. Derive the summary from existing policy/judgment/eval receipts where possible.
4. Make the summary deterministic and NDJSON-encodable.
5. Add contract tests for:
   - empty ledger produces zero reuse and valid receipt;
   - mixed policy-hit / policy-miss ledger computes stable counts;
   - regression is flagged when reuse rises but validation health falls;
   - receipt replay rejects tampered counts or sequence order.
6. Run the narrow tests first, then `cargo test --all-targets` if toolchain time allows.

## Deferred Work

- Full live Ollama validation remains environment-dependent.
- Graph telemetry still requires explicit wrapper capture.
- Student-model training is intentionally deferred until verified distillation data is large and clean.
- Orchestration scaling should wait until single-run reuse and validation health are proven.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Implement in capability/runtime-adjacent code without kernel authority drift.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.
