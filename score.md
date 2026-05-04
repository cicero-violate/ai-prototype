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
I  = 7.0 / 10
E  = 6.8 / 10
C  = 6.4 / 10
A  = 8.4 / 10
R  = 6.8 / 10
P  = 6.1 / 10
S  = 6.1 / 10
D  = 8.1 / 10
T  = 8.5 / 10
Co = 7.4 / 10
Em = 7.2 / 10
B  = 6.9 / 10
L  = 6.3 / 10
Si = 5.9 / 10
F  = 7.3 / 10

G = 6.97 / 10
max(G) = good
```

Judgment: the repo improved as an auditable handoff system because runtime archive inspection evidence is now explicitly carried from validation into the final delta manifest. It is still not production-ready: Rust compiler validation, wrapper graph telemetry, live Ollama execution, external observation/API tests, and current semantic artifact integration proof remain unclosed.

## Scope

```text
stage = PHASE_2_EXECUTE_TURN_002
source_changes_allowed = true
source_changes_made = true
restored_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/ai-phase2
observed_branch = main
base_commit = 9ea8f585fc7d043d13cee60452a572ff29d1e796
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_manifest_base_commit = 9ea8f585fc7d043d13cee60452a572ff29d1e796
runtime_manifest_base_matches_delta_base = true
existing_goal_md = true
existing_score_md_before_phase = true
```

## Phase 2 Delta

Implemented closure:

- Updated `plan.md` from `GOAL.md` and current `score.md` evidence.
- Extended `scripts/observe_validation.sh` so runtime archive inspection records distinct evidence for:
  - download indexes
  - conversation ledgers
  - prior runtime state
  - delta apply receipts
  - audit files
  - current run summary presence
  - runtime manifest presence
  - aggregate inspection status
- Extended `scripts/write_delta_manifest.py` so delta receipts and `DELTA_MANIFEST.md` preserve those runtime archive inspection fields.
- Added regression coverage in `tests/test_observe_validation_contract.py` and `tests/test_write_delta_manifest.py` for the new evidence contract and duplicate metric prevention.
- Updated this scorecard with current evidence, current limits, TODO/FIXME review, and recomputed `G`.

Why this matters: the requested workflow explicitly requires runtime tarball inspection for logs, conversation snapshots, download indexes, and prior runtime state. Before this change, validation could inspect the archive while the receiver manifest under-reported the inspection surface. The final artifact boundary now exposes that evidence directly.

## Existing File Contents Observed

`GOAL.md` exists. It defines Canon Agent as a deterministic, self-improving runtime with a frozen kernel, append-only replayable TLog, policy learning, capability-layer intelligence, bounded recovery, and an LLM promotion ladder. The relevant requirement for this turn is auditable evidence preservation across handoff.

Prior `score.md` existed. It was stale for this base because it referenced older Phase 2 state in the restored bundle. This file now records the current base `9ea8f585fc7d043d13cee60452a572ff29d1e796`.

## Runtime Archive Evidence

`/mnt/data/ai-runtime.tar.gz` was inspected with Python JSON/tar parsing.

```text
runtime_archive_sha256 = 991dadda52b4b1f3f524f49e3a40e5951fd40eadfaf4734d70b2daf738ff9f2b
runtime_archive_member_count = 51
runtime_manifest_base_commit = 9ea8f585fc7d043d13cee60452a572ff29d1e796
runtime_manifest_base_matches_delta_base = true
runtime_archive_log_files = 16
runtime_archive_log_total = 11081
runtime_archive_download_files = 10
runtime_archive_download_total = 108
runtime_archive_download_index_files = 11
runtime_archive_conversation_snapshots = 0
runtime_archive_conversation_ledger_files = 5
runtime_archive_delta_receipt_files = 5
runtime_archive_audit_files = 1
runtime_archive_current_run_summary_present = true
runtime_archive_runtime_manifest_present = true
runtime_archive_prior_state_files = 7
runtime_archive_inspection_status = pass
runtime_download_history_record_count = 112
runtime_unique_download_alias_count = 6
runtime_stale_advisory_count = 1
runtime_candidate_error_count = 0
runtime_duplicate_artifact_aliases = 4
runtime_performance_budget_status = pass
project_agent_elapsed_ms_p95 = 1236515.064
```

Critical reading: runtime archive evidence is now visible and structured, but it is still historical supporting evidence. It does not replace current Rust compilation, graph capture, live LLM proof, or external API/observation tests.

## Marker Evidence

Search commands:

```bash
rg -n --hidden -g '!.git/**' -g '!target/**' -g '!patch/**' -g '!score.md' 'TODO|FIXME' .
rg -n --hidden -g '!.git/**' -g '!target/**' 'TODO|FIXME' .
```

Findings:

```text
active TODO/FIXME markers outside score.md, target, and patch archive = 0
archived patch TODO markers = 4
score.md self-references marker evidence by design
```

Archived patch markers remain in `patch/improve_score_codebase.apply_patch`:

```text
command intake awaiting external API protocol
judgment payload awaiting versioned policy
handlers awaiting protocol schema freeze
wire encoding awaiting HTTP/gRPC transport choice
```

Critical reading: active source is clean of live TODO/FIXME deferrals, but archived patch debt still points to unresolved external protocol/API and versioned-policy boundary work.

## Validation Evidence

Direct validation run before commit:

```text
python3 -m py_compile scripts/write_delta_manifest.py => pass
python3 -m unittest discover -s tests -p 'test_*.py' -v => pass, 22 tests
python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json => pass
python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json => pass
git diff --check => pass
```

Policy-learning trace result:

```text
status = pass
trace_function = learning_policy_llm_feedback_loop_drives_judgment
missing_count = 0
check_count = 4 groups / 28 token checks
```

Panic-surface result:

```text
production_total = 0
example_total = 0
test_total = 319
finding_count = 319
```

Unavailable or expected partial validation:

```text
cargo fmt --check                    => unavailable unless valid cargo/rustc are on PATH
cargo test --all-targets             => unavailable unless valid cargo/rustc are on PATH
cargo clippy --all-targets           => unavailable unless valid cargo/rustc are on PATH
wrapper_graph_validation             => skipped unless CANON_RUSTC_WRAPPER is supplied
cargo run --example ollama_judgment  => skipped unless cargo and Ollama env are supplied
router_offline_tests                 => unavailable, router subtree missing
```

## Axis Detail

| Axis | Score | Critical basis |
|---|---:|---|
| I | 7.0 | Runtime archive inspection now distinguishes logs, ledgers, prior state, and receiver evidence; autonomous policy reduction is still not measured. |
| E | 6.8 | Python validation remains dependency-light and adds evidence without runtime service cost. |
| C | 6.4 | New regression tests close a concrete manifest evidence gap; Rust compile/test proof remains absent. |
| A | 8.4 | Change directly supports `GOAL.md` auditability, replay, and evidence preservation. |
| R | 6.8 | Handoff robustness improves because archive inspection is no longer implicit. |
| P | 6.1 | Runtime performance evidence is captured from archive and budgeted, but no current Rust benchmark exists. |
| S | 6.1 | Stronger manifests scale repo-loop handoff; orchestration/API scale remains unproven. |
| D | 8.1 | Deterministic manifest fields and test-enforced first-line ordering reduce receiver ambiguity. |
| T | 8.5 | Final manifest exposes runtime indexes, conversation ledgers, and prior state counts. |
| Co | 7.4 | Contributors get clearer validation lineage and artifact-boundary evidence. |
| Em | 7.2 | Operators can distinguish historical runtime evidence from missing current-head proof. |
| B | 6.9 | Benefit improves for reliable delta handoff; deployed user value remains unvalidated. |
| L | 6.3 | Policy-learning evidence persists and runtime history is better classified. |
| Si | 5.9 | More fields increase surface area, but remove hidden inspection ambiguity. |
| F | 7.3 | Future deltas are less likely to lose runtime inspection proof at the manifest boundary. |

## Risk Register

| Risk | Severity | Evidence | Closure requirement |
|---|---:|---|---|
| Rust crate not compiler-validated here | High | cargo/rustc not proven available for current head | Run fmt/test/clippy with a valid toolchain. |
| Graph telemetry absent | High | no generated `state/rustc/*/graph.json` | Run wrapper graph capture with `CANON_RUSTC_WRAPPER`. |
| Live Ollama proof absent at current head | High | local cargo/Ollama path unavailable | Run `cargo run --example ollama_judgment` with configured Ollama. |
| External API and observation tests absent | High | no router subtree and no external stream/action test | Add executable API and stream-ingress tests. |
| Semantic artifact verification not fully closed | High | source exists but current integration proof is partial | Add artifact proof fixtures and enforce replay validation. |
| Archived patch debt | Medium | four archived TODO markers | Confirm obsolete patch status or promote unresolved protocol work to active tracked plan. |
| Test panic-surface volume | Medium | 319 test findings | Reduce unnecessary test unwrap/expect usage when Rust tooling is available. |

## Next Closure Targets

1. Expose a valid Rust toolchain and run `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
2. Generate `state/rustc/*/graph.json` with `CANON_RUSTC_WRAPPER` and record node, edge, and intent-coverage metrics.
3. Run `cargo run --example ollama_judgment` against local Ollama and record durable receipt/proof replay evidence.
4. Add executable external observation stream and API action tests.
5. Add semantic artifact verification fixtures that bind artifact digest, proof record, and replay receipt.
