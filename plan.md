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

- `GOAL.md` requires deterministic observation ingress, replayable TLog evidence, bounded recovery, and verified external inputs before learning or policy reuse.
- `score.md` identifies remaining risks around validation closure, graph telemetry, live provider proof, transport proof, and simplicity debt. `score.md` is preserved and must not be edited in this phase.
- Turn 1 tightened observation cursor lineage so persisted cursor rows must be fresh or fully lineage-bound.

## Boundary

- Do not edit `score.md`.
- Do not create final delta bundle or manifest artifacts on this intermediate turn.
- Keep this turn source-level and cumulative on top of Phase 2 Turn 1.
- Prefer deterministic replay-safety hardening over broad refactors.

## Tasks

1. Refresh `plan.md` from `GOAL.md` and `score.md`.
2. Inspect TODO/FIXME markers and avoid adding deferrals.
3. Close one concrete correctness/robustness gap in current source.
4. Add a targeted regression test proving the new boundary.
5. Run Rust bootstrap before validation.
6. Run bounded validation.
7. Commit the cumulative repository state for this turn.

## Completed This Turn

- Hardened observation cursor loading so only the latest non-empty cursor row has authority.
- Changed `load_observation_cursor_ndjson` to reject a latest corrupt cursor row with `InvalidData` instead of scanning backward to an older valid row.
- Prevented stale cursor rollback after persisted cursor corruption, which protects observation replay determinism and external ingress lineage.
- Added `observation_cursor_loader_rejects_latest_corrupt_row` to prove an older valid cursor row cannot mask a newer corrupt cursor row.

## Validation Result

```text
python3 /mnt/data/bootstrap_rustc_session.py: pass
cargo test observation_cursor_loader_rejects_latest_corrupt_row --offline -- --nocapture: pass
cargo test observation_cursor_rejects_partial_persisted_state --offline -- --nocapture: pass
cargo check --offline: pass
cargo test --lib --offline: pass, 109 passed
python3 -m unittest discover -s tests -p 'test_*.py' -v: pass, 25 passed
git diff --check: pass
score.md: preserved; not edited during this turn
```

## Remaining Risk

- Full all-target validation remains large for this environment.
- `cargo fmt` and `cargo clippy` remain unavailable in the supplied extracted toolchain.
- This turn improves observation cursor corruption handling but does not close graph telemetry, live LLM proof, transport integration proof, or large-module simplicity debt.