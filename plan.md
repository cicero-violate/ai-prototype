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

- `GOAL.md` requires deterministic external observation, typed evidence, replayable TLog transitions, bounded recovery, and verified learning inputs.
- `score.md` identifies remaining risks around full validation closure, graph telemetry, live provider proof, transport proof, and simplicity debt. `score.md` is preserved and must not be edited in this phase.
- The restored repository already contains API batch bounding, command hash binding, command-ledger idempotence, process receipt proof surfaces, and observation ingress.

## Boundary

- Do not edit `score.md`.
- Do not create final delta bundle or manifest artifacts on this intermediate turn.
- Keep the change bounded and source-level.
- Prefer deterministic contract tightening over broad refactors.

## Tasks

1. Refresh `plan.md` from `GOAL.md` and `score.md`.
2. Inspect TODO/FIXME markers and keep them out of source unless needed for real future work.
3. Close one concrete correctness/robustness gap in the current source.
4. Add a targeted regression test proving the new boundary.
5. Run Rust bootstrap before validation.
6. Run bounded validation.
7. Commit the cumulative repository state for this turn.

## Completed This Turn

- Tightened observation cursor validity so persisted cursor state must be either fresh `(last_sequence=0,last_observed_hash=0)` or fully lineage-bound `(last_sequence!=0,last_observed_hash!=0)`.
- Made `ObservationCursor::accepts` reject corrupt partial cursor state before advancing external observation ingress.
- Made `decode_observation_cursor_ndjson` reject partial persisted cursor rows instead of accepting lineage-breaking state.
- Added `observation_cursor_rejects_partial_persisted_state` to prove corrupt cursor rows are rejected and cannot accept a later frame.

## Validation Result

```text
python3 /mnt/data/bootstrap_rustc_session.py --archive /mnt/data/rust-nightly-x86_64-unknown-linux-gnu.tar.gz --prefix /mnt/data/rust-sandbox --cargo-home /mnt/data/.cargo --env-file /mnt/data/rustc-session.env --no-install-launchers: pass
cargo test observation_cursor_rejects_partial_persisted_state --offline -- --nocapture: pass
cargo test bounded_line_observation_source_persists_cursor_and_applies_backpressure --offline -- --nocapture: pass
cargo check --offline: pass
cargo test --lib --offline: pass, 108 passed
python3 -m unittest discover -s tests -p 'test_*.py' -v: pass, 25 passed
git diff --check: pass
score.md: preserved from Phase 1; not edited during this turn
```

## Remaining Risk

- Full all-target validation is still large for this environment.
- `cargo fmt` and `cargo clippy` are still unavailable in the supplied extracted toolchain.
- This turn improves observation replay lineage but does not close transport API proof, graph telemetry, live LLM proof, or large-module simplicity debt.
