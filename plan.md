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

`GOAL.md` requires deterministic execution, auditable evidence, replayable run history, bounded recovery, policy learning outside the frozen kernel, and external capability surfaces. `score.md` shows a strong prototype with passing root Rust/Python tests and runtime archive evidence, but validation evidence for external observation, API action, and semantic verification was still too compressed: it emitted only booleans, not the tracked files and tokens that prove each surface exists.

The runtime archive `/mnt/data/ai-runtime.tar.gz` is present and contains runtime logs, candidate/download ledgers, audit records, delta receipts, prior-state indicators, runtime summaries, and a manifest whose base commit matches this turn's base commit.

## Work

1. Restore `/mnt/data/ai.bundle` at base `c8936ccbe7a4d8a3ca7d251de57fda9afd28b15a`.
2. Inspect `/mnt/data/ai-runtime.tar.gz` for logs, ledgers, indexes, summaries, manifests, and prior runtime state.
3. Improve `scripts/observe_validation.sh` so source-derived external capability evidence includes concrete tracked evidence files and token-to-file mappings, not only present/missing booleans.
4. Extend the Python contract test to require those evidence file/token fields.
5. Run validation after reading and running `/mnt/data/bootstrap_rustc_session.py`:
   - `cargo test --all-targets`
   - `python3 -m unittest discover -s tests -p 'test_*.py' -v`
   - `python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json`
   - `python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json`
   - `bash -n scripts/observe_validation.sh`
   - `python3 -m py_compile scripts/write_delta_manifest.py scripts/validate_policy_learning_trace.py scripts/validate_rust_panic_surface.py`
   - `git diff --check`
6. Update `score.md` with actual marker review, validation results, recomputed `G`, and remaining risks.
7. Commit the result and create `/mnt/data/repo-delta-002.bundle` plus `/mnt/data/DELTA_MANIFEST.md` for `B..H`.

## Boundary

This phase improves evidence traceability in the validation layer. It does not claim live Ollama execution, generated rustc-wrapper graph telemetry, complete rustfmt/clippy availability, full observe-summary completion inside this environment, or production deployment unless those signals appear in validation output.
