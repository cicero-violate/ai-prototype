# Phase 2 Turn 3 Plan

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
- `score.md` identifies remaining risk around validation closure, graph telemetry, live provider proof, transport proof, and simplicity debt. `score.md` is preserved and must not be edited in this phase.
- Runtime tarball inspection found prior agent-runtime logs, candidate/download ledgers, audit records, and prior delta-apply receipts; no source authority was taken from those runtime artifacts beyond confirming prior workflow context.
- Turn 1 tightened observation cursor lineage validation.
- Turn 2 rejected corrupt latest cursor rows instead of rolling back to older valid rows.

## Boundary

- Do not edit `score.md`.
- Preserve all committed Phase 2 changes since base commit `c5e91b717c253bf1383ed3344d0e4ffd2c5b1cd0`.
- Create the final cumulative turn artifact only after this turn is committed and validated.
- Prefer deterministic replay-safety hardening over broad refactors.

## Tasks

1. Refresh `plan.md` from `GOAL.md` and `score.md`.
2. Inspect TODO/FIXME markers and avoid adding deferrals.
3. Inspect runtime tarball context for logs, snapshots, indexes, and prior runtime state.
4. Close one concrete correctness/robustness gap in current source.
5. Add a targeted regression test proving the new boundary.
6. Run Rust bootstrap before validation.
7. Run bounded validation.
8. Commit the cumulative repository state for this turn.
9. Create `/mnt/data/repo-delta-004.bundle` and `/mnt/data/DELTA_MANIFEST.md` for `B..H`.

## Completed This Turn

- Hardened observation cursor persistence so cursor writes go through a synced sibling temporary file and atomic rename instead of direct overwrite.
- Reduced crash/interruption risk where a partial write could corrupt the only cursor authority and break deterministic observation replay.
- Added `observation_cursor_write_replaces_existing_cursor_atomically` to prove replacement leaves exactly one authoritative cursor row and reloads the latest cursor.

## Validation Result

```text
python3 /mnt/data/bootstrap_rustc_session.py --skip-library-probe: pass
cargo test observation_cursor_write_replaces_existing_cursor_atomically --offline -- --nocapture: pass
cargo test observation_cursor_loader_rejects_latest_corrupt_row --offline -- --nocapture: pass
cargo check --offline: pass
cargo test --lib --offline: pass, 110 passed
python3 -m unittest discover -s tests -p 'test_*.py' -v: pass, 25 passed
git diff --check: pass
score.md: preserved; not edited during this turn
```

## Remaining Risk

- Full all-target validation remains large for this environment.
- `cargo fmt` and `cargo clippy` remain unavailable in the supplied extracted toolchain.
- This turn improves observation cursor persistence but does not close graph telemetry, live LLM proof, transport integration proof, or large-module simplicity debt.
