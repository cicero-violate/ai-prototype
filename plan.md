# One-Shot Repository Improvement Plan

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

Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers the whole system.

## Boundary

```text
repository = ai
base_commit = b903059731873bcf0be6a7d324294f786917b9be
restored_head_before_changes = b903059731873bcf0be6a7d324294f786917b9be
bundle_output = /mnt/data/repo-delta-001.bundle
manifest_output = /mnt/data/DELTA_MANIFEST.md
```

## Executed Plan

1. Restored `/mnt/data/ai.bundle` and verified `GOAL.md` plus stale `score.md` against the restored files.
2. Inspected `/mnt/data/ai-runtime.tar.gz` for runtime manifests, audit logs, message/download ledgers, delta receipts, and loop-stop receipts.
3. Replaced `score.md` before source changes with a critical baseline and geometric mean `G = 6.44`.
4. Selected the weakest repairable dimensions: robustness, determinism, efficiency, transparency, and operator empowerment.
5. Patched `bootstrap_rustc_session.py` so toolchain reuse proves `rustc` and `cargo` actually run.
6. Patched extraction to stage files under `tmp/prefix-stage` and move into the final prefix only after a complete extraction.
7. Added Python regression tests for broken executable rejection, failed extraction prefix preservation, and successful atomic replacement.
8. Reran bootstrap, Rust validation, Python validation, policy-learning validation, panic-surface validation, and loop trace.
9. Updated `score.md` to describe the committed result and final `G = 7.00`.
10. Committed the delta and produced the required bundle plus manifest.

## Validation

```text
timeout 300s python3 /mnt/data/bootstrap_rustc_session.py --skip-library-probe --no-install-launchers = pass
CARGO_INCREMENTAL=0 timeout 300s cargo check --offline = pass
CARGO_INCREMENTAL=0 timeout 300s cargo test --all-targets --offline = pass
timeout 300s python3 -m unittest discover -s tests -p 'test_*.py' -v = pass, 30 tests
timeout 300s python3 scripts/validate_policy_learning_trace.py --root . --report /tmp/ai-one-shot-policy-learning-trace.json = pass
timeout 300s python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report /tmp/ai-one-shot-panic-surface.json = pass, production_total=0
CARGO_INCREMENTAL=0 timeout 300s cargo run --example loop_trace --offline = pass, 31 events, success=true
timeout 300s cargo fmt --check = unavailable, cargo-fmt not installed
timeout 300s cargo clippy --all-targets --offline -- -D warnings = unavailable, clippy not installed
git diff --check = pass
```

## Remaining Work

- Add a toolchain including `rustfmt` and `clippy`.
- Generate root wrapper graph telemetry under `state/rustc/*/graph.json`.
- Exercise live Ollama/OpenAI-compatible paths against an available endpoint.
- Reduce large-module simplicity debt in `src/lib.rs`, `openai.rs`, and `ollama.rs`.