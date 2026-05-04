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

## Current Truth

`GOAL.md` requires deterministic execution, auditable evidence, replayable run history, bounded recovery, policy learning outside the frozen kernel, and external capability surfaces. `score.md` shows that the root Rust and Python suites now pass under the mandated bootstrap toolchain, but the validation report still undercounts existing coverage by hardcoding missing external-observation, API-action, and semantic-verification tests even though the repository contains those deterministic tests.

The runtime archive `/mnt/data/ai-runtime.tar.gz` is present and contains conversation ledgers, download ledgers, audit records, delta-apply receipts, a current-run summary, and downloaded prior artifacts. It is useful evidence, but current-head validation remains authoritative.

## Work

1. Preserve the restored source history from `/mnt/data/ai.bundle` at base `c86c4438852e9a0779dcc53e860b9c4fc185824c`.
2. Inspect `/mnt/data/ai-runtime.tar.gz` for logs, ledgers, indexes, snapshots, and prior runtime state.
3. Improve `scripts/observe_validation.sh` so missing external-observation, external API-action, and semantic-artifact-verification signals are derived from tracked source evidence instead of hardcoded to missing.
4. Run validation after reading and running `/mnt/data/bootstrap_rustc_session.py`:
   - `cargo test --all-targets`
   - `python3 -m unittest discover -s tests -p 'test_*.py' -v`
   - `python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json`
   - `python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json`
   - `CANON_DELTA_BASE=<B> CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz CANON_OBSERVE_REPORT=target/observe/validation-report-phase2.ndjson bash scripts/observe_validation.sh`
   - `git diff --check`
5. Update `score.md` with the actual evidence, TODO/FIXME review, validation results, recomputed `G`, and remaining risks.
6. Commit the result and create `/mnt/data/repo-delta-002.bundle` plus `/mnt/data/DELTA_MANIFEST.md` for `B..H`.

## Boundary

This phase improves evidence correctness in the validation layer. It does not claim live Ollama execution, generated rustc-wrapper graph telemetry, complete rustfmt/clippy availability, or production deployment unless those signals appear in validation output.