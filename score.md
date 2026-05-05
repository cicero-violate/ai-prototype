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
I  = 7.4 / 10
E  = 7.2 / 10
C  = 7.6 / 10
A  = 8.5 / 10
R  = 7.6 / 10
P  = 6.7 / 10
S  = 6.6 / 10
D  = 8.4 / 10
T  = 8.9 / 10
Co = 7.8 / 10
Em = 7.7 / 10
B  = 7.4 / 10
L  = 6.7 / 10
Si = 6.3 / 10
F  = 7.7 / 10

G = 7.47 / 10
max(G) = good
```

Judgment: the repository is a strong auditable prototype. This phase closes a real handoff gap: source-derived external capability evidence now survives into the final delta receipt and manifest instead of being lost after observe validation. Remaining weaknesses are still live-system proof gaps: no fresh Ollama run, no generated rustc-wrapper graph telemetry, no production API deployment proof, and no rustfmt/clippy components in the mandated bootstrap toolchain.

## Scope

```text
stage = PHASE_2_EXECUTE_TURN_002
source_changes_allowed = true
source_changes_made = true
restored_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/work-ai-phase2/ai
observed_branch = main
base_commit = ba8714dbbe2b57c625611dffb9a5f079d3e2aa65
runtime_archive = /mnt/data/ai-runtime.tar.gz
existing_goal_md = true
existing_score_md_before_phase = true
```

## Phase 2 Delta

Changed files:

- `plan.md`
- `scripts/write_delta_manifest.py`
- `tests/test_write_delta_manifest.py`
- `score.md`

Implemented closure:

- Updated `plan.md` from the current `GOAL.md` and `score.md`.
- Extended `scripts/write_delta_manifest.py` so final receipts and `DELTA_MANIFEST.md` preserve source-derived external evidence fields from observe validation:
  - external observation evidence files and token mappings
  - external API action evidence files and token mappings
  - semantic artifact verification evidence files and token mappings
- Added `tests/test_write_delta_manifest.py::test_preserves_external_surface_evidence_files_and_tokens_once` so the manifest cannot silently drop these proof fields again.

Why this matters: `scripts/observe_validation.sh` already derives concrete source evidence, but final handoff artifacts are the receiver's authority. Losing evidence fields at manifest time weakens auditability even when validation was correct.

## GOAL.md Review

`GOAL.md` defines a frozen deterministic kernel with growth in external capabilities, a replayable TLog, policy learning, structured LLM evidence, bounded recovery, and auditable run history. This phase aligns with that goal by preserving evidence lineage at the delta handoff boundary.

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

Archived markers remain in `patch/improve_score_codebase.apply_patch` and reference older placeholder work in API protocol, capability payload, API routes, and codec wire encoding. They are historical patch debt, not active source markers.

## Runtime Archive Evidence

`/mnt/data/ai-runtime.tar.gz` was inspected with Python/tarfile.

```text
runtime_archive_member_count = 118
runtime_archive_log_like_files = 80
runtime_archive_download_related_files = 53
runtime_archive_candidate_ledgers = 23
runtime_archive_download_ledgers = 23
runtime_archive_message_ledgers = 23
runtime_archive_delta_receipts = 9
runtime_archive_audit_files = 1
runtime_archive_summary_files = 3
runtime_archive_manifest_files = 1
```

Critical reading: runtime state is rich enough to support continuity review, but current HEAD source validation remains authoritative for this score.

## Validation Evidence

| Command                                                                                                                                  | Result | Evidence                                                                                |
|------------------------------------------------------------------------------------------------------------------------------------------+--------+-----------------------------------------------------------------------------------------|
| `python3 /mnt/data/bootstrap_rustc_session.py`                                                                                           | pass   | rustc 1.75.0, cargo 1.75.0, offline probe pass, internal registry dependency probe pass |
| `cargo test --all-targets -- --test-threads=1`                                                                                           | pass   | 103 Rust tests passed; binary and examples compiled                                     |
| `python3 -m unittest tests.test_observe_validation_contract -v`                                                                          | pass   | 12 contract tests passed                                                                |
| `python3 -m unittest tests.test_policy_learning_trace_contract -v`                                                                       | pass   | 2 contract tests passed                                                                 |
| `python3 -m unittest tests.test_write_delta_manifest -v`                                                                                 | pass   | 10 contract tests passed                                                                |
| `python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json`                          | pass   | 4 check groups passed; missing count 0                                                  |
| `python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json`            | pass   | production total 0; test total 319; example total 1                                     |
| `bash -n scripts/observe_validation.sh`                                                                                                  | pass   | shell syntax valid                                                                      |
| `python3 -m py_compile scripts/write_delta_manifest.py scripts/validate_policy_learning_trace.py scripts/validate_rust_panic_surface.py` | pass   | validation scripts compile                                                              |
| `git diff --check`                                                                                                                       | pass   | whitespace check passed                                                                 |

Note: `cargo test --all-targets` without serialized test execution hit the tool timeout during display/collection, so the successful validation used `-- --test-threads=1` after the build was warm.

## Dimension Scores

| Var | Score | Evidence-backed rationale                                                                                               |
|-----+-------+-------------------------------------------------------------------------------------------------------------------------|
| I   |   7.4 | Evidence handoff now preserves more operational context for future agents and reviewers.                                |
| E   |   7.2 | Validation remains executable with a deterministic bootstrap toolchain; tests are grouped cleanly.                      |
| C   |   7.6 | Rust tests, Python contract tests, policy trace validation, panic-surface validation, py_compile, and diff checks pass. |
| A   |   8.5 | Change directly supports `GOAL.md` auditability and replayable evidence lineage.                                        |
| R   |   7.6 | Manifest-level contract test reduces risk of silent evidence loss at handoff.                                           |
| P   |   6.7 | Runtime archive contains performance signals; no fresh benchmark or full observe run is proven here.                    |
| S   |   6.6 | Token-to-file evidence survives handoff and scales better than opaque booleans.                                         |
| D   |   8.4 | Delta receipts and manifests remain deterministic functions of validation report and `B..H`.                            |
| T   |   8.9 | Final artifact transparency improves materially by including exact evidence files and tokens.                           |
| Co  |   7.8 | Contributors get a precise regression test for final-manifest evidence preservation.                                    |
| Em  |   7.7 | Operators can inspect external capability evidence from the manifest without reopening observe logs.                    |
| B   |   7.4 | Receiver value improves because handoff artifacts now carry stronger proof context.                                     |
| L   |   6.7 | Preserved evidence is more useful as learning data for future policy and score updates.                                 |
| Si  |   6.3 | The manifest gained fields; complexity rose slightly but stayed localized.                                              |
| F   |   7.7 | Handoff schema is more future-proof because evidence mappings are explicit.                                             |

## Risk Register

| Risk                                               | Severity | Evidence                                              | Closure requirement                                                                  |
|----------------------------------------------------+----------+-------------------------------------------------------+--------------------------------------------------------------------------------------|
| Full observe script completion still unproven here | Medium   | only syntax and subvalidations were run this phase    | Split observe validation into bounded subcommands or reduce nested subprocess cost.  |
| `cargo fmt` unavailable                            | Medium   | bootstrap Cargo lacks `cargo-fmt` / `rustfmt`         | Add rustfmt component to bootstrap or validate formatting with a complete toolchain. |
| `cargo clippy` unavailable                         | Medium   | bootstrap Cargo lacks clippy                          | Add clippy component or run with a complete toolchain.                               |
| Wrapper graph telemetry absent                     | High     | no generated `state/rustc/*/graph.json` in this phase | Run wrapper graph capture with `CANON_RUSTC_WRAPPER`.                                |
| Live Ollama proof absent at current head           | High     | local Ollama example not executed here                | Run `cargo run --example ollama_judgment` with configured local Ollama.              |
| Production API deployment unproven                 | High     | no live HTTP/gRPC deployment validation               | Add executable API deployment and request/response tests.                            |
| Archived patch debt                                | Medium   | four archived TODO markers                            | Confirm obsolete patch status or promote unresolved protocol work to active plan.    |

## Next Closure Targets

1. Split `scripts/observe_validation.sh` into independently bounded validation steps so one slow subprocess cannot block summary emission.
2. Add rustfmt and clippy components to the bootstrap toolchain.
3. Generate `state/rustc/*/graph.json` with `CANON_RUSTC_WRAPPER` and record graph metrics.
4. Run `cargo run --example ollama_judgment` against local Ollama and record durable receipt/proof replay evidence.
5. Add live external API deployment validation.
