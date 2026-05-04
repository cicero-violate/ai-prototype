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
I  = 7.1 / 10
E  = 7.0 / 10
C  = 7.3 / 10
A  = 8.4 / 10
R  = 7.3 / 10
P  = 6.4 / 10
S  = 6.3 / 10
D  = 8.2 / 10
T  = 8.5 / 10
Co = 7.5 / 10
Em = 7.4 / 10
B  = 7.1 / 10
L  = 6.4 / 10
Si = 6.1 / 10
F  = 7.4 / 10

G = 7.19 / 10
max(G) = good
```

Judgment: this phase materially improves correctness because the repository now validates under the mandated bootstrap toolchain. The prior hard blocker was not source logic; it was an incompatible manifest boundary: `edition = "2024"` and lockfile version 4 could not be parsed by Cargo 1.75.0. The repo is still not production-ready because generated rustc-wrapper graph telemetry, live Ollama execution, external API deployment, and production runtime benchmarks remain absent.

## Scope

```text
stage = PHASE_2_EXECUTE_TURN_002
source_changes_allowed = true
source_changes_made = true
restored_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/ai-phase2-work/ai
observed_branch = main
base_commit = ead9e72d22088d72ecf11bd9cf2acd5292afa319
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_manifest_base_commit = ead9e72d22088d72ecf11bd9cf2acd5292afa319
runtime_manifest_base_matches_delta_base = true
existing_goal_md = true
existing_score_md_before_phase = true
```

## Phase 2 Delta

Implemented closure:

- Updated `plan.md` from `GOAL.md` and `score.md` with the actual current blocker: bootstrap Cargo 1.75.0 rejecting Rust edition 2024.
- Changed `Cargo.toml` from Rust edition 2024 to edition 2021.
- Regenerated `Cargo.lock` from lockfile version 4 to version 3 with Cargo 1.75.0.
- Preserved the existing runtime archive inspection and delta manifest evidence contract.
- Recomputed the scorecard from current validation evidence.

Why this matters: Phase 1 could not prove the Rust crate because the mandated bootstrap toolchain rejected the package manifest before compilation. After this change, the Rust source compiles and the full root Rust test suite passes under the required bootstrap session.

## Existing File Content Review

### `GOAL.md`

`GOAL.md` defines a deterministic, auditable, self-improving agent runtime with a frozen kernel, replayable TLog, bounded recovery, and external capability learning. The implementation aligns with the goal at the architectural/test level: source modules cover kernel, codec, runtime, capability, API protocol, durable logs, policy learning, semantic verification, tool receipts, and Ollama receipt/proof paths.

### Previous `score.md`

The previous scorecard correctly identified the major blocker: Rust compiler validation was unavailable because Cargo 1.75.0 could not parse edition 2024. That risk is now closed by making the crate compatible with the mandated bootstrap toolchain.

## TODO / FIXME Review

Commands used:

```bash
rg -n --hidden -g '!.git/**' -g '!target/**' -g '!patch/**' -g '!score.md' 'TODO|FIXME' .
rg -n --hidden -g '!.git/**' -g '!target/**' 'TODO|FIXME' .
```

Findings:

```text
active TODO/FIXME markers outside score.md, target, and patch archive = 0
archived patch TODO markers = 4
```

Archived markers remain in `patch/improve_score_codebase.apply_patch`:

- `src/api/protocol.rs`: command intake TODO from an older patch.
- `src/capability/mod.rs`: judgment policy payload TODO from an older patch.
- `src/api/routes.rs`: handler TODO from an older patch.
- `src/codec/mod.rs`: wire encoding TODO from an older patch.

Critical reading: active source has no live TODO/FIXME deferrals. The archived patch debt is not active source debt, but it still indicates historical concern around API protocol stability, capability payload growth, route handlers, and wire encoding.

## Validation Evidence

| Command | Result | Evidence |
|---|---:|---|
| `python3 /mnt/data/bootstrap_rustc_session.py` | pass | rustc 1.75.0, cargo 1.75.0, offline probe pass, internal registry dependency probe pass |
| `cargo fmt --check` | unavailable | Cargo 1.75.0 bootstrap does not include `cargo-fmt` / `rustfmt` |
| `cargo test --all-targets -- --nocapture` | pass | 103 Rust tests passed; examples and main test harnesses compiled |
| `python3 -m py_compile scripts/write_delta_manifest.py` | pass | manifest writer syntax valid |
| `python3 -m unittest discover -s tests -p 'test_*.py' -v` | pass | 22 Python tests passed |
| `python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json` | pass | 4 check groups passed; missing count 0 |
| `python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json` | pass | production panic surface total 0; test total 319; example total 1 |
| `git diff --check` | pass | whitespace check passed |
| `scripts/observe_validation.sh` | partial | emitted runtime archive and git evidence, but did not complete inside this execution window |

## Runtime Archive Evidence

The runtime tarball `/mnt/data/ai-runtime.tar.gz` was inspected. It contains:

```text
runtime_archive_member_count = 97
runtime_archive_log_files = 39
runtime_archive_download_files = 22
runtime_archive_download_index_files = 35
runtime_archive_conversation_ledger_files = 17
runtime_archive_prior_state_files = 8
runtime_archive_delta_receipt_files = 6
runtime_archive_audit_files = 1
runtime_archive_current_run_summary_present = true
runtime_archive_runtime_manifest_present = true
runtime_manifest_base_matches_delta_base = true
```

Critical reading: the archive gives useful historical run evidence, but current-head source validation is stronger. Runtime archive evidence should not be treated as a substitute for current tests.

## Dimension Scores

| Var | Score | Evidence-backed rationale |
|---|---:|---|
| I | 7.1 | Strong architecture around deterministic gates, policy learning, semantic verification, and receipt proofs. |
| E | 7.0 | Cargo validation now runs directly under the required bootstrap toolchain; no edition/lockfile blocker remains. |
| C | 7.3 | 103 Rust tests and 22 Python tests pass; manifest parse blocker closed. |
| A | 8.4 | Work directly supports `GOAL.md`: deterministic, auditable, replayable execution. |
| R | 7.3 | Bootstrap-compatible lockfile and edition improve reproducibility. |
| P | 6.4 | Tests compile quickly after lockfile repair, but no benchmark suite or live runtime latency proof exists. |
| S | 6.3 | Capability layering and manifest evidence scale handoffs; orchestration/runtime scale remains unproven. |
| D | 8.2 | Deterministic TLog/replay tests and bootstrap-compatible Cargo state increase reproducibility. |
| T | 8.5 | Runtime archive evidence, manifest contracts, and scorecard validation surfaces are explicit. |
| Co | 7.5 | Contributors can now run the root Rust tests with the prescribed bootstrap instead of discovering manifest incompatibility. |
| Em | 7.4 | Operators get a clear plan, runnable tests, and exact remaining boundaries. |
| B | 7.1 | The repo is more useful as a validated handoff artifact; deployed user value is still unproven. |
| L | 6.4 | Policy learning tests pass, but learning is still test/prototype evidence rather than live accumulation. |
| Si | 6.1 | Edition downgrade removes tooling complexity; the overall system is still broad and concept-heavy. |
| F | 7.4 | Bootstrap compatibility and evidence manifests improve future repo-loop reliability. |

## Risk Register

| Risk | Severity | Evidence | Closure requirement |
|---|---:|---|---|
| `cargo fmt` unavailable | Medium | bootstrap Cargo lacks `cargo-fmt` and `rustfmt` | Add rustfmt component to bootstrap or validate formatting with a toolchain that includes it. |
| `cargo clippy` unavailable | Medium | bootstrap Cargo lacks clippy | Add clippy component to bootstrap or run with a complete toolchain. |
| Wrapper graph telemetry absent | High | no generated `state/rustc/*/graph.json` | Run wrapper graph capture with `CANON_RUSTC_WRAPPER`. |
| Live Ollama proof absent at current head | High | local Ollama example not executed here | Run `cargo run --example ollama_judgment` with configured local Ollama. |
| External API deployment unproven | High | no live HTTP/gRPC/API deployment validation | Add executable API deployment and request/response tests. |
| Archived patch debt | Medium | four archived TODO markers | Confirm obsolete patch status or promote unresolved protocol work to active plan. |

## Next Closure Targets

1. Extend `/mnt/data/bootstrap_rustc_session.py` or provide a complete Rust toolchain with `rustfmt` and `clippy`.
2. Run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` once those tools exist.
3. Generate `state/rustc/*/graph.json` with `CANON_RUSTC_WRAPPER` and record node, edge, and intent-coverage metrics.
4. Run `cargo run --example ollama_judgment` against local Ollama and record durable receipt/proof replay evidence.
5. Add live external API/observation stream validation.
