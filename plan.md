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

`GOAL.md` asks for a deterministic, auditable agent runtime where the frozen kernel stays small and capability evidence, policy learning, verification, receipts, and replay carry growth outside the kernel.

The current repository already has strong prototype coverage: Rust tests, Python contract tests, policy-learning trace validation, panic-surface validation, runtime archive inspection, and delta manifest tooling. The main active gap from `score.md` is evidence handoff: `scripts/observe_validation.sh` emits external capability evidence files and token mappings, but `scripts/write_delta_manifest.py` does not preserve those fields in the final delta receipt and manifest.

Runtime archive `/mnt/data/ai-runtime.tar.gz` is present. Python/tarfile inspection found runtime logs, download ledgers, message ledgers, candidate ledgers, audit records, delta receipts, summaries, and `RUNTIME_MANIFEST.json`.

## Work

1. Restore `/mnt/data/ai.bundle` and use base `ba8714dbbe2b57c625611dffb9a5f079d3e2aa65`.
2. Inspect `/mnt/data/ai-runtime.tar.gz` for logs, ledgers, indexes, summaries, manifests, and prior runtime state.
3. Update `scripts/write_delta_manifest.py` so final receipts and `DELTA_MANIFEST.md` preserve:
   - external observation evidence files and token mappings
   - external API action evidence files and token mappings
   - semantic artifact verification evidence files and token mappings
4. Extend `tests/test_write_delta_manifest.py` to enforce one-time preservation of those evidence fields.
5. Update `score.md` with actual validation, marker review, changed files, remaining risks, and recomputed `G`.
6. Run required validation after `/mnt/data/bootstrap_rustc_session.py`:
   - `cargo test --all-targets`
   - `python3 -m unittest discover -s tests -p 'test_*.py' -v`
   - `python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json`
   - `python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json`
   - `bash -n scripts/observe_validation.sh`
   - `python3 -m py_compile scripts/write_delta_manifest.py scripts/validate_policy_learning_trace.py scripts/validate_rust_panic_surface.py`
   - `git diff --check`
7. Commit the result, create `/mnt/data/repo-delta-002.bundle` for `B..H`, and write `/mnt/data/DELTA_MANIFEST.md`.

## Boundary

This phase closes final-manifest evidence loss. It does not claim fresh live Ollama execution, rustc-wrapper graph generation, production API deployment, rustfmt/clippy availability, or full observe-summary execution unless validated in this environment.
