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

`GOAL.md` requires deterministic execution, auditable evidence, replayable run history, and policy learning outside the frozen kernel. The current scorecard identifies a hard validation blocker: the repository declares Rust edition 2024 while the required bootstrap script installs Cargo 1.75.0, which rejects edition 2024 during manifest parsing.

The runtime archive `/mnt/data/ai-runtime.tar.gz` is present and contains conversation ledgers, download ledgers, audit records, delta-apply receipts, a current-run summary, and downloaded prior artifacts. Those records are useful evidence, but they do not replace current-head source validation.

## Work

1. Preserve the existing Phase 1/Phase 2 audit improvements and runtime archive inspection contract.
2. Change the Rust package edition from `2024` to `2021` so the repository can be validated with the mandated bootstrap toolchain.
3. Run Rust validation with wrapper variables cleared:
   - `cargo fmt --check`
   - `cargo test --all-targets`
4. Run Python and repository validation:
   - `python3 -m py_compile scripts/write_delta_manifest.py`
   - `python3 -m unittest discover -s tests -p 'test_*.py' -v`
   - `python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json`
   - `python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json`
   - `git diff --check`
   - `CANON_DELTA_BASE=<B> CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz CANON_OBSERVE_REPORT=target/observe/validation-report-phase2.ndjson bash scripts/observe_validation.sh`
5. Update `score.md` with the actual evidence, TODO/FIXME review, validation results, recomputed geometric mean `G`, and remaining risks.
6. Commit the result and create `/mnt/data/repo-delta-002.bundle` plus `/mnt/data/DELTA_MANIFEST.md` for `B..H`.

## Boundary

This phase closes the manifest-parse validation blocker and produces current-head Rust test evidence. It does not claim live Ollama execution, external API deployment, generated rustc-wrapper graph telemetry, or production deployment unless those signals are actually present in validation output.
