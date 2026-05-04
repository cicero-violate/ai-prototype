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

Improve the highest-impact current blocker from `GOAL.md` and `score.md`: safe delta artifact correctness.

## Evidence From GOAL.md

The goal requires every decision, recovery, and outcome to be replayable and auditable. Delta artifacts are the handoff boundary for this repository loop, so a manifest must prove not only that a bundle exposes `H`, but also that the bundle is the requested `B..H` delta anchored to the receiver's base commit.

## Evidence From score.md

`score.md` keeps correctness capped because validation and artifact evidence were not fully reproducible at current HEAD. The runtime archive also contains stale-advisory history, so the next useful closure is to harden the local artifact verifier against stale, full-history, or wrong-base bundles.

## Work

1. Harden `scripts/write_delta_manifest.py` so bundle verification extracts required refs from `git bundle verify` output.
2. Reject any provided bundle that exposes `H` but does not require the requested base commit `B`.
3. Record `bundle_required_refs` and `bundle_requires_base_commit` in the delta receipt and manifest.
4. Add regression coverage for complete-history bundles, which must fail because they are not the requested `B..H` delta.
5. Speed up delta-manifest tests by sharing one temporary git fixture across test cases instead of rebuilding it for every assertion.
6. Update `score.md` with the new evidence and recomputed geometric mean.

## Validation Commands

```bash
python3 -m unittest discover -s tests -p 'test_*.py' -v
python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
git diff --check
CANON_DELTA_BASE=bbcaa3947d447396ee3599b63a9af8125b95b2a2 CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz CANON_OBSERVE_REPORT=target/observe/validation-report-phase2.ndjson bash scripts/observe_validation.sh
git bundle create /mnt/data/repo-delta-002.bundle bbcaa3947d447396ee3599b63a9af8125b95b2a2..HEAD
git bundle verify /mnt/data/repo-delta-002.bundle
python3 scripts/write_delta_manifest.py --base bbcaa3947d447396ee3599b63a9af8125b95b2a2 --head <H> --report target/observe/validation-report-phase2.ndjson --bundle /mnt/data/repo-delta-002.bundle --out /mnt/data/DELTA_MANIFEST.md --receipt-out target/observe/delta-receipt-phase2.json
```

## Boundary

No Rust source changes in this pass because this sandbox has no `cargo` or `rustc`. The source change is limited to the Python delta verifier and its tests, which are executable here.
