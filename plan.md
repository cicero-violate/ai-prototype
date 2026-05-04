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

Improve the highest-impact current blocker from `GOAL.md` and `score.md`: runtime evidence freshness at the artifact boundary.

## Evidence From GOAL.md

The goal requires every decision, recovery, and outcome to be replayable and auditable. Runtime archives are useful evidence only when their manifest base is explicitly tied to the same base commit as the current delta contract.

## Evidence From score.md

`score.md` keeps correctness capped because Rust validation, graph telemetry, and current-head Ollama proof are absent. It also records that the runtime archive is historical and contains stale-advisory evidence, so the next useful closure is to make runtime-base mismatch visible in validation summaries and delta manifests.

## Work

1. Extend `scripts/observe_validation.sh` to compare `RUNTIME_MANIFEST.json.baseCommit` against `CANON_DELTA_BASE`.
2. Emit `runtime_manifest_base_expected` and `runtime_manifest_base_matches_delta_base` in validation summaries.
3. Add `missing_runtime_manifest_base_match` to the missing-signal set when a runtime archive is present but anchored to the wrong base.
4. Extend `scripts/write_delta_manifest.py` so receipts and manifests preserve this runtime-base evidence.
5. Keep absent router tests visible as a missing signal without making a generic restored repo fail required validation.
6. Add regression tests for the observe-validation contract and manifest preservation.
7. Update `score.md` with the new evidence and recomputed geometric mean.

## Validation Commands

```bash
python3 -m unittest discover -s tests -p 'test_*.py' -v
python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
git diff --check
CANON_DELTA_BASE=b89bdd0766eb986d6de87887eeb991ce6f837ba5 CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz CANON_OBSERVE_REPORT=target/observe/validation-report-phase2.ndjson bash scripts/observe_validation.sh
git bundle create /mnt/data/repo-delta-002.bundle b89bdd0766eb986d6de87887eeb991ce6f837ba5..HEAD
git bundle verify /mnt/data/repo-delta-002.bundle
python3 scripts/write_delta_manifest.py --base b89bdd0766eb986d6de87887eeb991ce6f837ba5 --head <H> --report target/observe/validation-report-phase2.ndjson --bundle /mnt/data/repo-delta-002.bundle --out /mnt/data/DELTA_MANIFEST.md --receipt-out target/observe/delta-receipt-phase2.json
```

## Boundary

No Rust source changes in this pass because this sandbox has no `cargo` or `rustc`. The source change is limited to the Python delta verifier and its tests, which are executable here.
