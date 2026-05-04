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

`GOAL.md` requires deterministic execution, auditable evidence, replayable run history, and policy learning outside the frozen kernel. The Phase 1 `score.md` says the repository is coherent but still capped by missing Rust compiler proof, missing wrapper graph telemetry, missing live Ollama proof, and incomplete external observation/API evidence.

The runtime archive `/mnt/data/ai-runtime.tar.gz` exists and aligns to base commit `9ea8f585fc7d043d13cee60452a572ff29d1e796`, but the artifact boundary still under-reports one required workflow class: runtime tarball inspection. Existing validation records aggregate logs and downloads, but the final manifest does not preserve distinct counts for download indexes, conversation ledgers, prior runtime state, audit files, delta-apply receipts, current-run summaries, and runtime manifest presence.

## Work

1. Preserve runtime archive inspection evidence in `scripts/observe_validation.sh`:
   - download index files
   - conversation ledger files
   - prior runtime state files
   - delta receipt files
   - audit files
   - current run summary presence
   - runtime manifest presence
   - aggregate inspection status
2. Preserve the same evidence in `scripts/write_delta_manifest.py` and `DELTA_MANIFEST.md` so the receiver can inspect `B..H` without reopening the tarball.
3. Add Python regression tests proving the observe contract emits the new fields and the manifest writes each new runtime inspection metric exactly once.
4. Update `score.md` with the actual Phase 2 evidence, marker review, validation results, and recomputed `G`.
5. Run validation, commit, and create the cumulative delta artifacts for `9ea8f585fc7d043d13cee60452a572ff29d1e796..HEAD`.

## Validation Commands

```bash
python3 -m py_compile scripts/write_delta_manifest.py
python3 -m unittest discover -s tests -p 'test_*.py' -v
python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json
python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
git diff --check
CANON_DELTA_BASE=9ea8f585fc7d043d13cee60452a572ff29d1e796 CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz CANON_OBSERVE_REPORT=target/observe/validation-report-phase2.ndjson bash scripts/observe_validation.sh
git bundle create /mnt/data/repo-delta-002.bundle 9ea8f585fc7d043d13cee60452a572ff29d1e796..HEAD
git bundle verify /mnt/data/repo-delta-002.bundle
python3 scripts/write_delta_manifest.py --base 9ea8f585fc7d043d13cee60452a572ff29d1e796 --head <H> --report target/observe/validation-report-phase2.ndjson --bundle /mnt/data/repo-delta-002.bundle --out /mnt/data/DELTA_MANIFEST.md --receipt-out target/observe/delta-receipt-phase2.json
```

## Boundary

Do not claim unavailable proof. The work closes evidence preservation around runtime archive inspection. It does not prove Rust compilation, wrapper graph generation, live Ollama execution, or external API/observation behavior unless those commands pass in the current environment.
