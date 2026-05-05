# Phase 2 Turn 1 Plan

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

- `GOAL.md` requires a frozen deterministic kernel, explicit capability growth, verified TLog evidence, policy learning from passed traces, and external evaluator authority over LLM proposals.
- `score.md` is preserved and not edited in this phase. Its highest-priority closure items are broken default Cargo validation, failing root Python validation, production panic-surface findings, missing wrapper graph telemetry, missing fmt/clippy components, and unproven live provider paths.
- TODO/FIXME search found only meta-plan/scorecard references; no active source TODO/FIXME markers were used to defer this turn's work.

## Boundary

- Base commit: `c914c14987938a0d95c7904a2ffa5164bb30f2d9`.
- Do not edit or stage `score.md`.
- Do not create final delta bundle or manifest in this intermediate turn.
- Prefer closure of validation blockers over broad refactors.

## Tasks

1. Refresh `plan.md` from `GOAL.md` and `score.md`.
2. Remove the default Cargo dependency on a missing local `rustc-wrapper` while preserving explicit opt-in wrapper graph capture.
3. Reduce production panic surface to zero for the current scanner.
4. Add a regression test proving validation fixtures are not counted as production panic surface.
5. Run Rust bootstrap before validation with `timeout 300s`.
6. Run bounded validation with `timeout 300s` for tests.
7. Commit only this turn's cumulative repository change; leave `score.md` unstaged.

## Completed This Turn

- Made root `.cargo/config.toml` wrapper-free by default and documented explicit opt-in usage via `CANON_RUSTC_WRAPPER=/path/to/canon-rustc-v3`.
- Replaced the wrapper's `stable_json_bytes` serialization panic with a deterministic fallback byte string so hash material generation does not crash the wrapper.
- Classified `canon-rustc-v3/validation/fixtures/**` panic-surface findings as test evidence, not production evidence.
- Added `tests/test_panic_surface_contract.py` to prove fixture panic facts do not fail `--fail-production-unwrap`.

## Validation Result

```text
timeout 300s python3 /mnt/data/bootstrap_rustc_session.py ...: pass

timeout 300s cargo check --offline: pass
timeout 300s cargo test --lib --offline: pass, 110 passed
timeout 300s cargo test --all-targets --offline: pass, 110 library tests and 0-test bin/example targets completed
timeout 300s python3 -m unittest discover -s tests -p 'test_*.py' -v: pass, 26 passed
timeout 300s python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report /mnt/data/ai-phase2-turn1-panic-surface.json: pass, production_total=0, test_total=335, example_total=1
timeout 300s python3 scripts/validate_policy_learning_trace.py --root . --report /mnt/data/ai-phase2-turn1-policy-learning-trace.json: pass, missing_count=0
timeout 300s cargo run --example loop_trace --offline: pass, 31 events, Done, success=true
git diff --check: pass
```

## Remaining Risk

- Wrapper graph telemetry is still optional and was not generated in this turn because no built `CANON_RUSTC_WRAPPER` binary was available.
- `cargo fmt` and `cargo clippy` remain unavailable in the supplied extracted toolchain.
- Live Ollama/OpenAI provider paths remain unproven by this turn.
- Large-module simplicity debt remains, especially in `src/lib.rs` and LLM adapter modules.