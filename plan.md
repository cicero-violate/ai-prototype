# Phase 2 Plan

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

## Target

Improve the highest-impact blocker from `score.md`: root validation portability.

## Evidence From GOAL.md

The goal requires a frozen deterministic kernel, replayable logs, append-only
policy learning, and trustworthy validation. That depends on a repository that a
new operator can validate without local machine paths.

## Evidence From score.md

`score.md` identified these critical blockers:

- missing Rust toolchain in the sandbox
- `.cargo/config.toml` points to an absent absolute wrapper path
- missing generated graph telemetry
- root Rust validation not reproducible here

## Work

1. Remove the default absolute `rustc-wrapper` from `.cargo/config.toml`.
2. Keep graph capture explicit through `CANON_RUSTC_WRAPPER`.
3. Harden the Python regression test so it actually rejects any multiline
   `rustc-wrapper = ...` default config entry.
4. Update README validation commands to use the current wrapper artifact
   variable name consistently.
5. Update `score.md` with the Phase 2 delta and recomputed `G`.

## Validation Commands

```bash
python3 -m unittest discover -s tests -p 'test_*.py'
python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
CANON_DELTA_BASE=<base> CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz CANON_OBSERVE_REPORT=target/observe/validation-report-phase2.ndjson bash scripts/observe_validation.sh
git diff --check
git bundle verify /mnt/data/repo-delta-001.bundle
```

## Boundary

No Rust source changes in this pass because the sandbox still lacks `cargo` and
`rustc`; changing kernel/runtime code without Rust validation would reduce
correctness.