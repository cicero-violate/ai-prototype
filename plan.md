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

Close the most executable Phase 1 gap from `score.md`: `missing_policy_learning_replay_trace` was always true even though `src/lib.rs` contains a concrete learning → policy → LLM → judgment test path. The goal is not to claim Rust execution without a Rust toolchain; the goal is to make the source-level trace evidence deterministic, required, and visible in validation summaries.

## Evidence From GOAL.md

`GOAL.md` requires a self-improving agent where completed run history is promoted into policy, policy changes future reasoning, and every decision remains replayable and auditable. That makes the learning/policy feedback trace central to the repository rather than optional documentation.

## Evidence From score.md

Phase 1 capped Learning at `5.4 / 10` and listed “Learning loop not demonstrated” as a high-severity risk. It also recorded unavailable Rust validation, missing graph telemetry, and no live Ollama proof. Those blockers remain environmental here, so this pass targets a validation surface that can execute in the current sandbox.

## Work

1. Add `scripts/validate_policy_learning_trace.py` to validate the source-level learning → policy → LLM → judgment trace.
2. Require the trace validator from `scripts/observe_validation.sh` alongside Python unit tests and panic-surface validation.
3. Emit `policy_learning_trace_*` fields in observe-validation summaries.
4. Set `missing_policy_learning_replay_trace` from the validator result instead of a hardcoded `true`.
5. Add Python regression tests proving the current repo passes and a synthetic missing trace fails with explicit missing tokens.
6. Update `score.md` with the new evidence, remaining limits, marker status, validation results, and recomputed `G`.

## Validation Commands

```bash
python3 -m unittest discover -s tests -p 'test_*.py' -v
python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json
python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
git diff --check
CANON_DELTA_BASE=7a8823141bc0e95c4689a3dff7f5a67840d1d4de CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz CANON_OBSERVE_REPORT=target/observe/validation-report-phase2.ndjson bash scripts/observe_validation.sh
git bundle create /mnt/data/repo-delta-002.bundle 7a8823141bc0e95c4689a3dff7f5a67840d1d4de..HEAD
git bundle verify /mnt/data/repo-delta-002.bundle
python3 scripts/write_delta_manifest.py --base 7a8823141bc0e95c4689a3dff7f5a67840d1d4de --head <H> --report target/observe/validation-report-phase2.ndjson --bundle /mnt/data/repo-delta-002.bundle --out /mnt/data/DELTA_MANIFEST.md --receipt-out target/observe/delta-receipt-phase2.json
```

## Boundary

Do not claim compiler-level proof in this sandbox. `cargo`, `rustc`, graph-wrapper telemetry, and live Ollama remain unavailable unless an external environment provides them. This phase improves deterministic evidence and prevents an existing learning trace from being hidden behind a permanent missing-signal flag.
