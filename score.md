# Canon Agent Scorecard

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

## Score Summary

```text
I  = 6.8 / 10
E  = 6.3 / 10
C  = 5.8 / 10
A  = 8.2 / 10
R  = 6.4 / 10
P  = 5.7 / 10
S  = 5.9 / 10
D  = 7.8 / 10
T  = 8.3 / 10
Co = 7.1 / 10
Em = 7.0 / 10
B  = 6.6 / 10
L  = 6.4 / 10
Si = 5.3 / 10
F  = 7.1 / 10

G = 6.66 / 10
max(G) = good
```

Judgment: the repository is architecturally serious and unusually audit-oriented, but still not production-ready. Its strongest properties are deterministic state modeling, typed capability boundaries, replay/receipt thinking, and transparent handoff evidence. Its weakest properties are current Rust validation under the supplied toolchain, live external integration proof, performance proof, graph telemetry, and simplicity.

## Scope

```text
stage = PHASE_1_REVIEW
source_changes_allowed = false
source_changes_made = false
score_md_updated = true
restored_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/ai-phase1/ai
observed_branch = main
head_commit = 4e76762f8011c4c40f85373a8dc9264d7a31746c
tracked_files = 123
rust_files_src_examples = 53
python_test_count = 22
rust_test_attr_count = 103
requested_external_rustc_guide_path_present = false
repo_local_rustc_guide_present = true
```

The requested guide path `/mnt/data/canon-mini-agent-extracted/canon-mini-agent/prototype/ai/rustc_installation_guide.md` was not present in this environment. The repository-local `rustc_installation_guide.md` was present and describes the corrected Python `tarfile` extraction procedure.

## Existing File Contents Observed

`GOAL.md` exists. It defines Canon Agent as a deterministic, self-improving runtime with a frozen kernel, append-only replayable TLog, bounded recovery, capability-layer intelligence, policy learning, and LLM promotion from routine reasoner to novelty specialist.

Prior `score.md` existed. It was a Phase 2 scorecard tied to older delta/manifest evidence and older validation context. This file replaces that stale score with a Phase 1 review of the restored bundle at `4e76762f8011c4c40f85373a8dc9264d7a31746c`.

## Evidence Summary

Repository shape:

```text
tracked_files = 123
rust_files_src_examples = 53
largest_rust_file = src/lib.rs, 3686 lines
large_integration_file = src/capability/llm/ollama.rs, 1929 lines
capability_modules = context, eval, judgment, learning, llm, memory, observation, orchestration, planning, policy, tooling, verification
```

Positive evidence:

- `src/lib.rs` forbids unsafe code at crate level with `#![forbid(unsafe_code)]`.
- `GOAL.md` and source layout agree on the intended split: kernel, codec, runtime, capability, API.
- `.cargo/config.toml` enforces strict Rust diagnostics with `-Dwarnings`, `-Dunused`, `-Ddead-code`, and related flags.
- `src/runtime/verify.rs` validates transition legality, hash linkage, state continuity, registry projection, API command receipt data, and replay semantics.
- `src/capability/verification/proof.rs` defines a canonical effect proof spine: request, authority, effect, receipt, proof, proof record, replay.
- `src/api/routes.rs` has idempotent envelope handling through `CommandLedger`, including conflicting-command rejection and replayed-event reuse.
- Python regression tests passed: `22/22`.
- Policy-learning trace validation passed with `missing_count = 0`.
- Panic-surface validation passed for production code with `production_total = 0`.
- Runtime archive parsing evidence was emitted before the observe command timed out; the archive base matched the restored head and runtime performance budget status was `pass`.

Negative evidence:

- `cargo check --offline` failed before compilation because the supplied corrected toolchain is Cargo/Rust `1.75.0`, while this crate declares `edition = "2024"`.
- `cargo fmt`, `cargo test --all-targets`, and `cargo clippy` were not proven for the current head under the supplied archive set.
- Wrapper graph telemetry is absent: no `state/rustc/*/graph.json` was observed.
- Live Ollama execution was not proven in this phase.
- External API/HTTP/gRPC behavior is represented by deterministic route functions, not by an executable server or end-to-end integration test in this restored repo.
- `scripts/observe_validation.sh` did not complete in the allotted run; it emitted early report events, then timed out before a full final validation summary.
- Test code contains a high number of panic-surface findings: `test_total = 319`.
- Simplicity is weak: `src/lib.rs` is a 3,686-line public surface with extensive embedded tests, and the Ollama adapter is a 1,929-line integration module.

## TODO / FIXME Marker Evidence

Search commands:

```bash
rg -n --hidden -g '!.git/**' -g '!target/**' -g '!patch/**' -g '!score.md' 'TODO|FIXME' .
rg -n --hidden -g '!.git/**' -g '!target/**' 'TODO|FIXME' .
```

Findings:

```text
active TODO/FIXME markers outside score.md, target, and patch archive = 0
archived patch TODO markers = 4
score.md marker references = expected scorecard evidence text
```

Archived patch markers remain in `patch/improve_score_codebase.apply_patch`:

```text
command intake awaiting external API protocol
judgment payload awaiting versioned policy
handlers awaiting protocol schema freeze
wire encoding awaiting HTTP/gRPC transport choice
```

Critical reading: active source does not defer requested work with live TODO/FIXME markers, but archived patch history still points to unresolved protocol/API and versioned-policy boundaries.

## Validation Evidence

Toolchain probe using Python `tarfile` extraction:

```text
archive = /mnt/data/rust-1.75.0-x86_64-unknown-linux-gnu.tar.gz
prefix = /mnt/data/rustc-python-install-prefix
cargo_home = /mnt/data/rustc-python-cargo-home
rustc --version = rustc 1.75.0 (82e1608df 2023-12-21)
cargo --version = cargo 1.75.0 (1d8b05cdd 2023-11-20)
dependency_free_probe = cargo run --offline => pass
```

Repository validation:

```text
cargo check --offline => fail, manifest requires edition2024 not stabilized in cargo 1.75.0
python3 -m unittest discover -s tests -p 'test_*.py' -v => pass, 22 tests
python3 -m py_compile scripts/write_delta_manifest.py scripts/validate_policy_learning_trace.py scripts/validate_rust_panic_surface.py => pass
python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json => pass
python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json => pass
git diff --check => pass
```

Partial observe evidence:

```text
CANON_DELTA_BASE = 4e76762f8011c4c40f85373a8dc9264d7a31746c
CANON_RUNTIME_ARCHIVE = /mnt/data/ai-runtime.tar.gz
runtime_archive_sha256 = bf7fe98ff934d09d12fb124d4e02e2023a05e2ecf7473de1526589e9db3f2ee6
runtime_archive_member_count = 67
runtime_manifest_base_matches_delta_base = true
runtime_performance_budget_status = pass
git_diff_check = pass
git_delta_diff_check = pass
observe_completion = timeout before final validation summary
```

## Axis Detail

| Axis | Score | Critical basis |
|---|---:|---|
| I | 6.8 | Strong typed runtime/capability model and policy-learning intent; still lacks measured autonomous improvement on current head. |
| E | 6.3 | Dependency-free Rust probe and Python tests are lightweight; observe validation timed out and the Rust crate cannot be checked with the supplied toolchain. |
| C | 5.8 | Python/test-script validation is strong, but current Rust correctness is unproven because manifest parsing fails before compilation. |
| A | 8.2 | Source layout and verification spine closely match `GOAL.md`: frozen kernel, capability evidence, replay, and auditability. |
| R | 6.4 | Replay, receipt, and durable modules exist; live integration and full compile/test proof are missing. |
| P | 5.7 | Runtime archive budget status is pass, but no current Rust benchmark exists and observe completion was not achieved. |
| S | 5.9 | Modular capability taxonomy exists, but orchestration/API scale is not proven end-to-end. |
| D | 7.8 | Deterministic transition tables, hash-linked events, and replay checks are central; the unvalidated current Rust build lowers confidence. |
| T | 8.3 | GOAL, README, plan, score, manifest scripts, and validation reports provide unusually explicit evidence trails. |
| Co | 7.1 | Handoff docs and scripts help future operators, but stale prior score context and large surfaces increase onboarding cost. |
| Em | 7.0 | Operators can run tests and inspect receipts; missing current-head Rust proof limits safe extension. |
| B | 6.6 | The architecture is useful for auditable autonomous execution, but deployed user value is still indirect. |
| L | 6.4 | Policy-learning trace exists and passes, but learning remains a validated pattern more than a demonstrated compounding loop. |
| Si | 5.3 | Large monolithic public surface, large LLM adapter, many archived patches, and many test unwraps reduce simplicity. |
| F | 7.1 | Canonical proof spine and strict contracts are future-compatible; edition/toolchain mismatch and unclosed integration proof remain risks. |

## Risk Register

| Risk | Severity | Evidence | Closure requirement |
|---|---:|---|---|
| Rust crate not validated | High | `cargo check --offline` fails on `edition2024` with Cargo 1.75.0 | Provide a Rust/Cargo toolchain that supports edition 2024, then run fmt/test/clippy. |
| Wrapper graph telemetry absent | High | no `state/rustc/*/graph.json` | Run explicit `CANON_RUSTC_WRAPPER` capture and record graph metrics. |
| Live LLM path unproven | High | `cargo run --example ollama_judgment` not run | Run with local Ollama and verify receipts/proof events. |
| Observe validation incomplete | Medium | report emitted 9 events, then timed out | Profile and bound slow/hanging observe checks. |
| External API not end-to-end proven | High | routes exist, server/transport proof absent | Add executable API transport and command-ingress tests. |
| Simplicity debt | Medium | `src/lib.rs` 3686 lines, `ollama.rs` 1929 lines | Split public exports/tests and adapter internals into smaller modules. |
| Test panic surface | Medium | `test_total = 319` | Reduce unnecessary `unwrap`/`expect` in tests where failure messages matter. |
| Archived protocol debt | Medium | 4 archived TODO markers | Resolve or retire obsolete patch debt around protocol/schema/transport. |

## Next Closure Targets

1. Install or provide a Rust/Cargo toolchain that supports `edition = "2024"`, then run `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
2. Generate wrapper graph telemetry under `state/rustc` and record node, edge, and intent coverage metrics.
3. Run the live Ollama judgment example and verify durable receipt/proof replay.
4. Fix or bound the observe-validation timeout path.
5. Add executable external API/transport tests instead of only deterministic route-level proof.