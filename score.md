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
I  = 6.7 / 10
E  = 6.9 / 10
C  = 6.1 / 10
A  = 8.1 / 10
R  = 6.4 / 10
P  = 6.2 / 10
S  = 6.0 / 10
D  = 7.8 / 10
T  = 7.7 / 10
Co = 7.5 / 10
Em = 7.0 / 10
B  = 6.7 / 10
L  = 5.5 / 10
Si = 6.1 / 10
F  = 7.1 / 10

G = 6.75 / 10
max(G) = good
```

Judgment: this remains a disciplined deterministic-runtime prototype with strong audit goals and improving handoff safety. This phase improves evidence freshness by binding runtime archive manifest base evidence to the active delta base and by preventing a generic restored repository from failing required validation solely because an optional router subtree is absent. The score remains capped by unavailable Rust validation, missing graph telemetry, and missing current-head Ollama proof.

## Scope

```text
stage = PHASE_2_EXECUTE_TURN_002
source_changes_allowed = true
source_changes_made = true
rust_source_files_changed = false
scorecard_updated = true
restored_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/ai-phase2-work/repo
observed_branch = main
base_commit = b89bdd0766eb986d6de87887eeb991ce6f837ba5
runtime_archive = /mnt/data/ai-runtime.tar.gz
existing_goal_md = true
existing_score_md_before_phase = true
```

## Phase 2 Turn 002 Delta

Implemented closure:

- `plan.md` now targets runtime evidence freshness at the artifact boundary.
- `scripts/observe_validation.sh` compares `RUNTIME_MANIFEST.json.baseCommit` with `CANON_DELTA_BASE`.
- Validation summaries now emit `runtime_manifest_base_expected` and `runtime_manifest_base_matches_delta_base`.
- `missing_runtime_manifest_base_match` is now a missing-signal flag when a runtime archive is present but anchored to the wrong base.
- `router_offline_tests` stays visible as `missing_router_offline_tests`, but it is required only when the router test directory is actually available.
- `scripts/write_delta_manifest.py` preserves runtime-base-match evidence in receipts and manifests.
- `tests/test_observe_validation_contract.py` covers the new runtime-base and router-availability contracts.
- `tests/test_write_delta_manifest.py` covers manifest preservation of runtime-base evidence and now runs manifest checks in-process, reducing Python test runtime from tens of seconds to about one second for the suite logic.

Remaining critical limits:

- `cargo` and `rustc` are unavailable in this sandbox, so Rust fmt/test/clippy remain unexecuted here.
- `state/rustc/*/graph.json` is absent, so semantic graph telemetry remains missing.
- `examples/ollama_judgment.rs` was not re-run at current HEAD because no local Ollama environment is configured.
- Runtime archive evidence is useful and now base-matched, but it still cannot replace current-head Rust/compiler validation.
- Existing TODO/FIXME markers remain only in an archived patch file, not active source.

## Existing File Contents Observed

`GOAL.md` exists and defines Canon Agent as a deterministic, self-improving agent runtime with a frozen state-machine kernel, append-only replayable TLog, capability-layer intelligence, policy learning, and an LLM promotion ladder. It explicitly requires replayable and auditable decisions, recoveries, and outcomes.

`score.md` existed before this phase and recorded that artifact handoff safety had improved, while correctness remained capped by missing Rust validation, missing graph telemetry, and historical runtime evidence.

## Repository Evidence

```text
tracked_files = 120
rust_files_src_examples = 53
python_files = 4
shell_files = 4
third_party_rust_dependencies = 0
rust_test_attrs = 103
active_source_todo_fixme_mentions = 0
archived_patch_todo_mentions = 4
panic_surface_production_total = 0
panic_surface_test_total = 319
```

Positive evidence:

- `src/lib.rs` and `src/main.rs` use `#![forbid(unsafe_code)]`.
- `Cargo.toml` declares no third-party Rust dependencies.
- The tree matches the intended architecture: `kernel`, `codec`, `runtime`, `capability`, and `api`.
- Runtime and capability modules expose TLog replay, durable runtime resume, semantic delta logic, command ledger receipts, transition verification, policy, learning, and eval surfaces.
- Python validation covers observe-validation contracts, panic-surface validation requirements, runtime performance contract fields, delta manifest integrity, runtime archive base matching, and optional-router validation semantics.

Critical evidence:

- Root Rust validation still cannot be reproduced here because no Rust toolchain is installed in `/mnt/data` or `PATH`.
- No generated semantic graph is present under `state/rustc`.
- Runtime archive `RUNTIME_MANIFEST.json` is now observed at the active delta base when `CANON_DELTA_BASE=b89bdd0766eb986d6de87887eeb991ce6f837ba5`, but runtime archive data remains external historical evidence rather than compiler validation.
- The codebase has many panic-like calls in test-classified Rust regions. Production panic surface is clean according to the Python classifier, but this remains a classifier result rather than a Rust compiler proof.
- The router offline test path is absent in this restored repo; the validation system now records that absence without making it a required-command failure.

## Validation Evidence

Commands run during this phase:

```text
python3 -m unittest discover -s tests -p 'test_*.py' -v
  => pass, 16 tests

python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
  => pass, production_total=0, test_total=319, example_total=0

git diff --check
  => pass

CANON_DELTA_BASE=b89bdd0766eb986d6de87887eeb991ce6f837ba5 CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz CANON_OBSERVE_REPORT=target/observe/validation-report-phase2.ndjson bash scripts/observe_validation.sh
  => partial, failed_required_commands=[]
```

Expected unavailable or skipped commands in this sandbox:

```text
cargo fmt --check                    => unavailable, cargo not found
cargo test --all-targets             => unavailable, cargo not found
cargo clippy --all-targets           => unavailable, cargo not found
wrapper_graph_validation             => skipped, CANON_RUSTC_WRAPPER missing
cargo run --example ollama_judgment  => skipped, Ollama env missing
router_offline_tests                 => unavailable, router subtree missing
```

Runtime archive evidence from `/mnt/data/ai-runtime.tar.gz`:

```text
runtime_archive_parse_status = pass
runtime_manifest_base_commit = b89bdd0766eb986d6de87887eeb991ce6f837ba5
runtime_manifest_base_expected = b89bdd0766eb986d6de87887eeb991ce6f837ba5
runtime_manifest_base_matches_delta_base = true
runtime_archive_log_total = 9571
runtime_archive_download_total = 88
runtime_performance_signal_present = true
runtime_performance_budget_status = pass
```

## Axis Detail

| Axis | Score | Critical basis |
|---|---:|---|
| I | 6.7 | Strong typed architecture plus better runtime evidence classification; closed-loop autonomous intelligence is still mostly scaffolded. |
| E | 6.9 | Manifest tests now run in-process and finish much faster; Rust validation still cannot run. |
| C | 6.1 | Runtime archive base matching and optional-router required-command logic reduce false or stale validation states; compiler validation is absent. |
| A | 8.1 | Work directly supports GOAL.md auditability and replayability. |
| R | 6.4 | Wrong-base runtime archives are now visible; graph and Rust validation remain missing. |
| P | 6.2 | Python test runtime improved materially, and runtime performance evidence is budgeted. |
| S | 6.0 | Generic repo validation is less brittle because absent optional router tests no longer fail required validation. |
| D | 7.8 | Delta and runtime evidence are now anchored to explicit base/head contracts. |
| T | 7.7 | Manifest and receipt output now expose runtime base expectation and match status. |
| Co | 7.5 | Receiver handoff is clearer because optional component absence and runtime-base mismatch are distinct signals. |
| Em | 7.0 | Operators get stronger freshness evidence and fewer false validation failures. |
| B | 6.7 | Benefit increases for safe repo-loop automation, but deployed agent benefit is still unproven. |
| L | 5.5 | Learning/policy modules remain present but no new learning replay trace was executed. |
| Si | 6.1 | Test path is simpler and faster; broad Rust surface remains cognitively large. |
| F | 7.1 | Freshness checks improve future iteration safety. |

## Risk Register

| Risk | Severity | Evidence | Closure requirement |
|---|---:|---|---|
| Root Rust validation unavailable | High | `cargo`/`rustc` not found | Provide toolchain and rerun fmt/test/clippy. |
| Missing graph telemetry | High | no `state/rustc/*/graph.json` | Regenerate graph and publish node/edge/intent metrics. |
| Current-head LLM proof absent | High | Ollama example not run here | Re-run with local Ollama and record proof replay. |
| Runtime archive cannot replace compiler validation | Medium | archive is base-matched but external to current compiler run | Keep archive evidence as supporting signal only. |
| Classifier-only panic proof | Medium | panic-surface result is Python source scan | Reconfirm with Rust validation once toolchain exists. |
| Learning proof not closed | High | no observation→eval→learning replay trace | Add and validate a current-head policy promotion trace. |

## Next Closure Targets

1. Install or expose a Rust toolchain and rerun `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
2. Run graph capture explicitly with `CANON_RUSTC_WRAPPER` and regenerate `state/rustc/*/graph.json`.
3. Re-run `examples/ollama_judgment.rs` at current HEAD with local Ollama and attach receipt/proof replay evidence.
4. Add a current-head observation → judgment → eval → learning → policy promotion integration trace.
5. Keep artifact freshness strict: every bundle must expose `H`, require `B`, and report whether runtime archive evidence is anchored to the active delta base.