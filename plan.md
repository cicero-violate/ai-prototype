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

`GOAL.md` requires an auditable, replayable runtime where completed runs become policy and validation evidence survives handoff. The current repository already validates policy-learning and panic-surface contracts, but `scripts/write_delta_manifest.py` did not preserve those two high-value evidence classes in the final `DELTA_MANIFEST.md`. That made the final receiver artifact less transparent than the validation report that produced it.

The restored bundle head is `b7bbdbc0c30ce167e14e697ece4e928dafdcd3a0`. The runtime archive `/mnt/data/ai-runtime.tar.gz` includes `RUNTIME_MANIFEST.json` with the same base commit, plus conversation/download/runtime logs. That archive is useful supporting evidence, but the current turn still needs a fresh committed delta and fresh `/mnt/data/repo-delta-002.bundle` plus `/mnt/data/DELTA_MANIFEST.md`.

## Work

1. Preserve policy-learning trace evidence in delta receipts and manifests:
   - `policy_learning_trace_validation_result`
   - `policy_learning_trace_status`
   - `policy_learning_trace_function`
   - `policy_learning_trace_check_count`
   - `policy_learning_trace_missing_count`
2. Preserve panic-surface evidence in delta receipts and manifests:
   - production unwrap/expect/panic counts
   - test panic-surface total
   - example panic-surface total
3. Add regression tests proving:
   - manifest line 1 is `base_commit: <B>`
   - manifest line 2 is `head_commit: <H>`
   - policy-learning evidence survives into receipt and manifest
   - panic-surface evidence survives into receipt and manifest
   - repeated scalar metrics such as `router_test_count` are not duplicated
4. Update `score.md` with the new evidence, remaining environmental limits, marker review, validation results, and recomputed `G`.
5. Run validation, commit, and produce the cumulative bundle and manifest for `b7bbdbc0c30ce167e14e697ece4e928dafdcd3a0..HEAD`.

## Validation Commands

```bash
python3 -m unittest discover -s tests -p 'test_*.py' -v
python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json
python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
git diff --check
CANON_DELTA_BASE=b7bbdbc0c30ce167e14e697ece4e928dafdcd3a0 CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz CANON_OBSERVE_REPORT=target/observe/validation-report-phase2.ndjson bash scripts/observe_validation.sh
git bundle create /mnt/data/repo-delta-002.bundle b7bbdbc0c30ce167e14e697ece4e928dafdcd3a0..HEAD
git bundle verify /mnt/data/repo-delta-002.bundle
python3 scripts/write_delta_manifest.py --base b7bbdbc0c30ce167e14e697ece4e928dafdcd3a0 --head <H> --report target/observe/validation-report-phase2.ndjson --bundle /mnt/data/repo-delta-002.bundle --out /mnt/data/DELTA_MANIFEST.md --receipt-out target/observe/delta-receipt-phase2.json
```

## Boundary

Do not claim unavailable proof. `cargo`, `rustc`, wrapper graph telemetry, and live Ollama execution remain absent in this container unless the environment supplies them. This phase improves artifact evidence closure, not compiler-level runtime proof.