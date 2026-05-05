# Phase 2 Turn 2 Plan

## Variables

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
G  = Goodness
```

## Equation

```text
G = (I*E*C*A*R*P*S*D*T*Co*Em*B*L*Si*F)^(1/15)
max(G) = good
```

One-line explanation: Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers the whole system.

## Source Inputs

- `GOAL.md` requires deterministic validation evidence, external evaluator authority, replayable receipts, and policy learning from proven traces.
- `score.md` is preserved and not edited in this phase. Its remaining closure targets include missing wrapper graph telemetry, unavailable fmt/clippy components, live provider proof gaps, and large-module simplicity debt.
- Turn 1 already closed the default missing wrapper, Python contract, and production panic-surface blockers.
- TODO/FIXME search found only meta-plan/scorecard references; no active source TODO/FIXME markers were used to defer this turn's work.

## Boundary

- Base commit: `c914c14987938a0d95c7904a2ffa5164bb30f2d9`.
- Current cumulative working HEAD before this turn: `2e2558482dbbb360d1f1f40edea5c511144b37df`.
- Do not edit or stage `score.md`.
- Do not create final delta bundle or manifest in this intermediate turn.
- Keep changes narrow and validation-focused.

## Tasks

1. Refresh `plan.md` from `GOAL.md` and `score.md`.
2. Make long-running validation command budgets match the requested `300s` test bound.
3. Preserve an environment override for operators who need a longer local validation budget.
4. Add a Python contract test proving long-running observe-validation commands use the shared test timeout.
5. Run Rust bootstrap before validation with `timeout 300s`.
6. Run bounded validation with `timeout 300s` for tests.
7. Commit only this turn's cumulative repository change; leave `score.md` unstaged.

## Completed This Turn

- Added `DEFAULT_TEST_TIMEOUT_SECONDS = 300` to `scripts/observe_validation.sh`.
- Added `CANON_TEST_TIMEOUT_SECONDS` as a positive-integer override for operators who intentionally need a different local budget.
- Routed long-running observe-validation commands through the shared timeout: `cargo_test_all_targets`, `cargo_clippy_all_targets`, `wrapper_graph_validation`, and `ollama_judgment_example`.
- Added `tests/test_observe_validation_contract.py` coverage proving the 300-second timeout contract and rejecting stale `timeout=600` calls.

## Validation Result

```text
timeout 300s python3 /mnt/data/bootstrap_rustc_session.py ...: pass

timeout 300s cargo check --offline: pass
timeout 300s cargo test --lib --offline: pass, 110 passed
timeout 300s cargo test --all-targets --offline: pass, 110 library tests and 0-test bin/example targets completed
timeout 300s python3 -m unittest discover -s tests -p 'test_*.py' -v: pass, 27 passed
timeout 300s python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report /mnt/data/ai-phase2-turn2-panic-surface.json: pass, production_total=0, test_total=335, example_total=1
timeout 300s python3 scripts/validate_policy_learning_trace.py --root . --report /mnt/data/ai-phase2-turn2-policy-learning-trace.json: pass, missing_count=0
timeout 300s cargo run --example loop_trace --offline: pass, 31 events, Done, success=true
git diff --check: pass
```

## Remaining Risk

- Wrapper graph telemetry is still optional and requires an explicit built `CANON_RUSTC_WRAPPER` binary.
- `cargo fmt` and `cargo clippy` remain dependent on unavailable rustfmt/clippy components in the supplied extracted toolchain.
- Live Ollama/OpenAI provider paths remain gated on explicit local/provider endpoints.
- Large-module simplicity debt remains, especially in `src/lib.rs` and LLM adapter modules.
