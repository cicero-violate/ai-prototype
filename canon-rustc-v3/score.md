# canon-rustc-v3 Scorecard

Reviewed: 2026-05-04
Stage: `PHASE 2 implemented / turn 002`
Base commit: `0d08488e942e2a2e5572bc6e0de49b508701838c`
Repository: `canon-rustc-v3`
Scope: generic repository review after portable-core build boundary work.

## Variables

`I` = Intelligence, `E` = Efficiency, `C` = Correctness, `A` = Alignment, `R` = Robustness, `P` = Performance, `S` = Scalability, `D` = Determinism, `T` = Transparency, `Co` = Collaboration, `Em` = Empowerment, `B` = Benefit, `L` = Learning, `Si` = Simplicity, `F` = Future-Proofing.

`G = (I × E × C × A × R × P × S × D × T × Co × Em × B × L × Si × F)^(1/15)`

`max(G) = good`

Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers the whole system.

## GOAL.md Evidence

`GOAL.md` exists and defines the repository target as a semantic rustc-wrapper witness: canonical facts (`fn`, `trait`, `impl`, `call`, `mut`, `io`, `unsafe`, `panic`, `alloc`), graph deltas, intent classification, invariant gates, compact reconstruction, replay hashes, leverage ranking, proof routing, and prompt-feed evidence.

## Runtime Evidence

`/mnt/data/canon-rustc-v3-runtime.tar.gz` was inspected. It contains `.repo-agent-runtime` ledgers, message snapshots, download indexes, network request logs, delta-apply receipts, loop-stop receipts, downloaded score/plan artifacts, validation logs, and `RUNTIME_MANIFEST.json`. This is useful diagnostic evidence, not acceptance proof, because prior runtime receipt validation has reported missing/failing signals.

## TODO/FIXME Evidence

Search command:

`rg -n "TODO|FIXME" . --glob '!target/**' --glob '!vendor/**'`

Result: no source-code TODO or FIXME markers were found. Matches are documentation self-references in `score.md`. This is positive hygiene evidence, but it does not replace executable proof.

## Phase 2 Change Evidence

1. `Cargo.toml` now defines an explicit `rustc-driver` feature with no default features.
2. `src/lib.rs` gates `#![feature(rustc_private)]`, rustc-private extern crates, and rustc-dependent modules behind `feature = "rustc-driver"`.
3. `src/bin/canon_rustc_v3.rs` keeps native witness capture under `feature = "rustc-driver"` and makes the default binary a transparent pass-through to the real compiler without witness capture.
4. The repository now has a portable core validation path: `cargo check --offline` passes under the bootstrapped stable toolchain.
5. The native witness path remains explicitly blocked in this sandbox: `cargo check --offline --features rustc-driver` fails because `rustc-dev`/private crates and nightly are unavailable.

## Validation Evidence

| Command                                                                                                                                                         | Result         | Evidence-backed judgment                                                                                      |
|-----------------------------------------------------------------------------------------------------------------------------------------------------------------+----------------+---------------------------------------------------------------------------------------------------------------|
| `python3 /mnt/data/bootstrap_rustc_session.py`                                                                                                                  | pass           | Bootstrapped/reused `rustc 1.75.0`, `cargo 1.75.0`; offline probe and internal-registry library probe passed. |
| `cargo check --offline`                                                                                                                                         | pass           | Default no-feature portable core now compiles under stable.                                                   |
| `cargo check --offline --features rustc-driver`                                                                                                                 | fail_expected  | Native witness capture still requires pinned nightly plus `rustc-dev` private crates.                         |
| `python3 -m py_compile validation/*.py`                                                                                                                         | pass           | Python validators compile.                                                                                    |
| `python3 validation/semantic_spine.py`                                                                                                                          | pass           | Static semantic spine accepted current source invariants.                                                     |
| `python3 validation/semantic_preflight.py --report ...`                                                                                                         | pass_with_skip | Remaining unavailable native/live-replay signals are explicit.                                                |
| `python3 validation/run_semantic_witness.py --static-only --report ...`                                                                                         | pass           | Static witness passed; this is not native wrapper execution proof.                                            |
| `python3 validation/semantic_scale_probe.py --nodes 5000 --fanout 2 --risk-additions 100 --threshold-ms 2000 --report ...`                                      | pass           | Synthetic scale probe passed under threshold.                                                                 |
| `python3 validation/performance_gate.py --scale-report ... --report ...`                                                                                        | pass_with_skip | Native wrapper-overhead evidence remains absent.                                                              |
| `python3 validation/reproducibility_gate.py --require-portable-config --preflight-report ... --witness-report ... --report ...`                                 | pass_with_skip | Reproducibility is improved for portable core, but live validation/rust-source/native proof remain missing.   |
| `python3 validation/runtime_receipt_gate.py --runtime-archive /mnt/data/canon-rustc-v3-runtime.tar.gz --report ...`                                             | fail           | Runtime archive still has missing/failing receipt signals; diagnostic only.                                   |
| `timeout 20 python3 validation/toolchain_archive_gate.py --archive /mnt/data/rust-nightly-x86_64-unknown-linux-gnu.tar.gz --require-channel-match --report ...` | timeout        | Archive inspection did not complete within the bounded validation window; native proof remains absent.        |
| `git diff --check`                                                                                                                                              | pass           | No whitespace errors in the committed delta.                                                                  |

## Critical Scores

| Variable | Score | Evidence-backed judgment                                                                                                                         |
|----------+-------+--------------------------------------------------------------------------------------------------------------------------------------------------|
| `I`      |     8 | Strong semantic-witness architecture: HIR/MIR extraction, intent/risk hashing, receipts, runtime gates, delta contracts, and archive validation. |
| `E`      |     7 | Portable core now validates quickly under stable; native witness validation remains blocked.                                                     |
| `C`      |     5 | Default core compiles, but native rustc-driver correctness, live replay, runtime receipts, and exact toolchain proof remain incomplete.          |
| `A`      |     8 | Feature split aligns with the goal by separating core facts/schema validation from native compiler witness capture.                              |
| `R`      |     5 | Atomic writes and typed receipts help; runtime receipt validation still fails.                                                                   |
| `P`      |     4 | Synthetic scale is fast; real wrapper overhead remains unmeasured.                                                                               |
| `S`      |     5 | Portable validation scales better, but no large real-workspace live replay exists.                                                               |
| `D`      |     8 | Ordered schemas, stable hashes, lockfile, portable config, and explicit features improve deterministic boundaries.                               |
| `T`      |     8 | Native proof gaps are visible rather than hidden behind the portable build.                                                                      |
| `Co`     |     7 | Plans, scorecards, validators, and manifest discipline support handoff.                                                                          |
| `Em`     |     7 | Agents can now run a real stable build check while preserving the native witness feature for stronger environments.                              |
| `B`      |     6 | The repo is more useful as a portable prototype; it is still not production-ready.                                                               |
| `L`      |     7 | Validation distinguishes portable compile success from native witness failure.                                                                   |
| `Si`     |     7 | The no-default-feature split simplifies sandbox validation without deleting rustc-driver code.                                                   |
| `F`      |     6 | Future-proofing improves because environments can validate the core while exact nightly/rustc-dev remains an explicit upgrade path.              |

## Geometric Mean

`G = (8 × 7 × 5 × 8 × 5 × 4 × 5 × 8 × 8 × 7 × 7 × 6 × 7 × 7 × 6)^(1/15)`

`G = 6.40 / 10`

Normalized: `64.0 / 100`.

## Verdict

`canon-rustc-v3` is materially stronger after this phase because the repository now has a real portable build check instead of a total build failure under the available stable sandbox. The default build does not prove witness capture. Native correctness still requires the pinned nightly toolchain, `rustc-dev`, live wrapper replay, runtime receipt acceptance, and measured wrapper overhead.
