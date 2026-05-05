# canon-rustc-v3 Phase 2 Plan

Base commit: `0d08488e942e2a2e5572bc6e0de49b508701838c`
Stage: `PHASE 2 / turn 002`
Status: `implemented`

## Variables

`I`=Intelligence, `E`=Efficiency, `C`=Correctness, `A`=Alignment, `R`=Robustness, `P`=Performance, `S`=Scalability, `D`=Determinism, `T`=Transparency, `Co`=Collaboration, `Em`=Empowerment, `B`=Benefit, `L`=Learning, `Si`=Simplicity, `F`=Future-Proofing.

`G=(I*E*C*A*R*P*S*D*T*Co*Em*B*L*Si*F)^(1/15)`

`max(G)=good`

One-line explanation: split portable core validation from native rustc-driver witness validation, so the repository can prove what is buildable in this sandbox without pretending the rustc-private boundary is closed.

## Inputs From GOAL.md And score.md

`GOAL.md` requires a semantic rustc-wrapper witness: canonical fact extraction, semantic deltas, intent classification, invariant gates, compact facts, replay hashes, leverage ranking, proof routing, and agent prompt evidence.

`score.md` identified the dominant blockers as native rustc-private compilation, live replay, runtime receipt failures, exact nightly/rustc-dev closure, and unmeasured wrapper overhead. The highest-yield current work is therefore to make the non-rustc-private core compile under the bootstrapped stable toolchain while keeping witness capture gated behind an explicit feature.

## Executed Work

1. Restore and inspect cumulative context. **Implemented.**
   - Restored `/mnt/data/canon-rustc-v3.bundle` at base `0d08488e942e2a2e5572bc6e0de49b508701838c`.
   - Inspected `/mnt/data/canon-rustc-v3-runtime.tar.gz`; it contains `.repo-agent-runtime` ledgers, message snapshots, download indexes, network logs, delta apply receipts, loop-stop receipts, downloads, and `RUNTIME_MANIFEST.json`.

2. Add a portable build boundary. **Implemented.**
   - Added `features.default = []` and `features.rustc-driver = []` in `Cargo.toml`.
   - Gated rustc-private extern crates and rustc-dependent modules behind `feature = "rustc-driver"`.
   - Made the default binary forward to the real compiler without witness capture, while the `rustc-driver` feature preserves native callback capture behavior.

3. Preserve the native proof boundary. **Implemented.**
   - `cargo check --offline` now passes under bootstrapped stable `rustc 1.75.0`.
   - `cargo check --offline --features rustc-driver` still fails in this sandbox because `rustc-dev`/private crates and nightly are unavailable; this remains explicit evidence, not hidden failure.

4. Update score evidence. **Implemented.**
   - Updated `score.md` with current validation results, TODO/FIXME evidence, and new geometric mean.

## Non-Goals

- Do not claim witness-capture correctness from the default no-feature build.
- Do not claim native rustc-private correctness until `--features rustc-driver` builds under pinned nightly with `rustc-dev`.
- Do not fabricate runtime signatures, live replay receipts, or wrapper-overhead metrics.
