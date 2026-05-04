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
E  = 6.6 / 10
C  = 5.9 / 10
A  = 8.2 / 10
R  = 6.3 / 10
P  = 5.9 / 10
S  = 5.9 / 10
D  = 7.8 / 10
T  = 8.0 / 10
Co = 7.3 / 10
Em = 6.9 / 10
B  = 6.6 / 10
L  = 5.9 / 10
Si = 5.8 / 10
F  = 7.0 / 10

G = 6.68 / 10
max(G) = good
```

Judgment: this phase improves the evidence spine by replacing a hardcoded missing learning signal with an executable source-level validator. The repository is still not production-ready: Rust compiler validation, graph telemetry, live Ollama execution, external observation/API tests, and semantic artifact verification remain missing in this sandbox.

## Scope

```text
stage = PHASE_2_EXECUTE_TURN_002
source_changes_allowed = true
source_changes_made = true
restored_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/ai-phase2/repo
observed_branch = main
base_commit = 7a8823141bc0e95c4689a3dff7f5a67840d1d4de
runtime_archive = /mnt/data/ai-runtime.tar.gz
existing_goal_md = true
existing_score_md_before_phase = true
```

## Phase 2 Delta

Implemented closure:

- `plan.md` now targets the highest executable Phase 1 gap: `missing_policy_learning_replay_trace`.
- Added `scripts/validate_policy_learning_trace.py`.
- `scripts/observe_validation.sh` now runs the policy-learning trace validator as a required validation command.
- Observe summaries now emit `policy_learning_trace_validation_result`, `policy_learning_trace_status`, `policy_learning_trace_function`, `policy_learning_trace_check_count`, and `policy_learning_trace_missing_count`.
- `missing_policy_learning_replay_trace` is now computed from validation result instead of hardcoded to `true`.
- Added `tests/test_policy_learning_trace_contract.py` for pass/fail trace contracts.
- Extended `tests/test_observe_validation_contract.py` to lock the new required validation contract.

Remaining critical limits:

- `cargo` and `rustc` are unavailable in this sandbox, so Rust fmt/test/clippy still did not run.
- No generated graph exists under `state/rustc`, so graph node/edge/intent telemetry is still absent.
- `examples/ollama_judgment.rs` was not run here because local Ollama environment variables are missing and `cargo` is unavailable.
- The policy-learning validator proves a source-level trace contract, not compiler execution of the Rust test.
- Runtime archive evidence is base-matched to `7a8823141bc0e95c4689a3dff7f5a67840d1d4de`, but it remains historical supporting evidence.

## Existing File Contents Observed

`GOAL.md` exists. It defines Canon Agent as a deterministic, self-improving agent runtime with a frozen state-machine kernel, append-only replayable TLog, capability-layer intelligence, policy learning, and an LLM promotion ladder. It requires completed run history to become policy and policy to reduce future reasoning cost while preserving auditability.

The Phase 1 `score.md` was carried forward before execution. It scored the restored repository at `G = 6.54 / 10`, reported no active TODO/FIXME markers outside `score.md` and `patch/**`, and identified the learning loop as undemonstrated because `missing_policy_learning_replay_trace` was always true.

## Repository Evidence

```text
tracked_files_before_commit = 120
new_files_added = 2
rust_files_src_examples = 53
python_files_after_change = 5
shell_files = 4
third_party_rust_dependencies = 0
rust_test_attrs = 103
python_unittest_tests = 19
active_source_todo_fixme_mentions = 0
archived_patch_todo_fixme_mentions = 4
panic_surface_production_total = 0
panic_surface_test_total = 319
largest_file = src/lib.rs, 3686 lines
state_graph_present = false
cargo_available = false
rustc_available = false
policy_learning_trace_validation_result = pass
policy_learning_trace_missing_count = 0
```

Positive evidence:

- `src/lib.rs` and `src/main.rs` use `#![forbid(unsafe_code)]`.
- `Cargo.toml` declares no third-party Rust dependencies, and `Cargo.lock` contains only the local `ai` package.
- The tree follows the intended layers: `kernel`, `codec`, `runtime`, `capability`, and `api`.
- Runtime code includes deterministic replay, TLog verification, command-ledger reconstruction, durable resume, bounded recovery policy, transition validation, and semantic delta checks.
- Capability code includes typed records for observation, context, memory, planning, LLM, judgment, tooling, verification, eval, policy, learning, and orchestration.
- `scripts/validate_policy_learning_trace.py` validates that `learning_policy_llm_feedback_loop_drives_judgment` connects run history, `PolicyPromotion::from_tlog`, `PolicyStore::promote_feedback`, policy hash/version injection into the LLM prompt, API evidence submission, judgment pass, transition to `Phase::Plan`, and `verify_tlog`.
- Runtime archive evidence is present, base-matched, and contains performance signals within configured budgets.

Critical evidence:

- Rust validation is still absent. Python tests and source scans cannot replace `cargo fmt`, `cargo test`, or `cargo clippy`.
- The new policy-learning validation is source-level. It prevents the repository from hiding an existing trace, but it is not a compiled Rust proof in this environment.
- No generated semantic graph exists under `state/rustc`; graph telemetry and intent coverage remain missing.
- The router test directory is absent in this generic restored repo, so router tests are visible as unavailable but not required.
- `src/lib.rs` still centralizes all 103 Rust `#[test]` attributes and remains the largest file.
- Production panic-surface scan is clean, but 319 test-bucket panic-like calls remain noisy.

## TODO/FIXME Evidence

```text
active search command = rg -n --hidden --glob '!.git' --glob '!target' --glob '!patch/**' --glob '!score.md' 'TODO|FIXME' .
active_result = no matches
archived search command = rg -n --hidden --glob '!.git' --glob '!target' 'TODO|FIXME' .
archived_result_count = 4 patch TODO markers plus score references
```

Archived TODO markers found:

```text
patch/improve_score_codebase.apply_patch:44  define command intake once the external API protocol is fixed
patch/improve_score_codebase.apply_patch:69  grow this payload once judgment policy becomes versioned
patch/improve_score_codebase.apply_patch:134 implement handlers after the protocol schema is frozen
patch/improve_score_codebase.apply_patch:158 stabilize wire encoding after HTTP/gRPC transport choice
```

## Validation Evidence

Commands run during this phase:

```text
python3 -m unittest discover -s tests -p 'test_*.py' -v
  => pass, 19 tests

python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json
  => pass, checks=4, missing_count=0

python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json
  => pass, production_total=0, test_total=319, example_total=0

git diff --check
  => pass

CANON_DELTA_BASE=7a8823141bc0e95c4689a3dff7f5a67840d1d4de CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz CANON_OBSERVE_REPORT=target/observe/validation-report-phase2.ndjson bash scripts/observe_validation.sh
  => partial, failed_required_commands=[]
```

Observe-validation result summary:

```text
validation_status = partial
validation_test_count = 19
failed_required_commands = []
policy_learning_trace_validation_result = pass
policy_learning_trace_status = pass
policy_learning_trace_missing_count = 0
missing_policy_learning_replay_trace = false
runtime_manifest_base_commit = 7a8823141bc0e95c4689a3dff7f5a67840d1d4de
runtime_manifest_base_expected = 7a8823141bc0e95c4689a3dff7f5a67840d1d4de
runtime_manifest_base_matches_delta_base = true
runtime_archive_log_total = 10070
runtime_archive_download_total = 95
runtime_performance_signal_present = true
runtime_performance_budget_status = pass
missing_signal_count = 14
```

Unavailable or skipped validation:

```text
cargo fmt --check                    => unavailable, cargo not found
cargo test --all-targets             => unavailable, cargo not found
cargo clippy --all-targets           => unavailable, cargo not found
wrapper_graph_validation             => skipped_env_missing, CANON_RUSTC_WRAPPER missing
cargo run --example ollama_judgment  => skipped_env_missing, Ollama env missing
router_offline_tests                 => unavailable, router subtree missing
```

## Axis Detail

| Axis | Score | Critical basis |
|---|---:|---|
| I | 6.7 | Source-level evidence now proves the intended learning/policy/judgment trace exists; closed-loop autonomous reduction of LLM work is still not measured. |
| E | 6.6 | Python validation remains executable and now captures a previously hidden signal; missing Rust tooling still blocks full validation. |
| C | 5.9 | Required trace and panic validators pass, but compiler validation remains absent. |
| A | 8.2 | The change directly supports GOAL.md’s learning-from-run-history requirement. |
| R | 6.3 | Validation no longer permanently reports a false learning-loop absence; major runtime/graph/Ollama gaps remain. |
| P | 5.9 | Runtime archive performance stays within configured budgets; no current Rust benchmark exists. |
| S | 5.9 | Validation is more modular, but external observation/API/orchestration scale remains unproven. |
| D | 7.8 | Required validator makes the learning signal deterministic and machine-checkable. |
| T | 8.0 | Observe summaries now expose policy-learning trace status and missing-token counts. |
| Co | 7.3 | Contributors get explicit failure tokens for missing learning-trace structure. |
| Em | 6.9 | Operators can distinguish “trace absent” from “Rust execution unavailable.” |
| B | 6.6 | Benefit improves for repo-loop evaluation, but deployed user benefit is still unproven. |
| L | 5.9 | Learning score improves because policy feedback is now validated as a required source-level trace. |
| Si | 5.8 | Added script increases file count slightly, but simplifies a hardcoded missing signal into one deterministic check. |
| F | 7.0 | Future iterations can fail fast if the learning/policy/judgment trace regresses. |

## Risk Register

| Risk | Severity | Evidence | Closure requirement |
|---|---:|---|---|
| Rust crate not compiler-validated here | High | `cargo` and `rustc` unavailable | Expose toolchain and rerun fmt/test/clippy. |
| Policy-learning proof is source-level only | Medium | validator scans Rust source contract | Re-run compiled Rust tests once toolchain exists. |
| Missing semantic graph telemetry | High | no `state/rustc/*/graph.json` | Run explicit graph capture with `CANON_RUSTC_WRAPPER`. |
| Current-head local LLM proof absent | High | Ollama example skipped | Run `examples/ollama_judgment.rs` and verify receipt/proof replay. |
| External API and observation tests absent | High | observe flags remain true | Add current-head API action and stream-ingress validation. |
| Monolithic Rust test surface | Medium | 103 Rust tests in `src/lib.rs`; 3,686-line file | Move tests into focused modules or integration tests. |
| Archived unresolved TODOs | Medium | four TODO markers in archived patch | Confirm obsolete patch TODOs or close protocol/API issues in active docs. |

## Next Closure Targets

1. Expose a Rust toolchain and rerun `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
2. Generate `state/rustc/*/graph.json` with `CANON_RUSTC_WRAPPER` and record node, edge, and intent-coverage metrics.
3. Run `cargo run --example ollama_judgment` against local Ollama and record durable receipt/proof replay evidence.
4. Convert the source-level policy-learning trace into a compiled validation receipt after Rust tooling is available.
5. Add external observation stream and API action tests so the remaining always-missing external-surface flags become executable evidence.
