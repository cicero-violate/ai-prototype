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
I  = 6.5 / 10
E  = 6.5 / 10
C  = 5.5 / 10
A  = 8.0 / 10
R  = 5.9 / 10
P  = 5.8 / 10
S  = 5.8 / 10
D  = 7.5 / 10
T  = 7.3 / 10
Co = 7.2 / 10
Em = 6.8 / 10
B  = 6.5 / 10
L  = 5.4 / 10
Si = 5.7 / 10
F  = 6.9 / 10

G = 6.44 / 10
max(G) = good
```

Judgment: this is a strong deterministic-runtime prototype with serious evidence discipline, but it is not yet a reproducibly validated autonomous agent runtime in this restored sandbox. Phase 2 closes the default absolute-wrapper portability blocker, but Rust validation, graph telemetry, and current-head LLM proof remain missing here.

## Scope

```text
stage = PHASE_2_EXECUTE
source_changes_allowed = true
source_changes_made = true
scorecard_updated = true
restored_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/ai-phase1/repo
observed_branch = main
observed_head = a4f94113d15ee6e01b35fd87edf78c9f654ca305
runtime_archive = /mnt/data/ai-runtime.tar.gz
existing_goal_md = true
existing_score_md_before_eval = false
```

## Phase 2 Delta

```text
base_commit = a4f94113d15ee6e01b35fd87edf78c9f654ca305
plan_created = plan.md
default_absolute_rustc_wrapper_removed = true
graph_capture_policy = explicit_CANON_RUSTC_WRAPPER_only
regression_test_hardened = true
rust_source_files_changed = false
```

Implemented closure:

- `.cargo/config.toml` no longer installs a machine-local absolute
  `rustc-wrapper` by default.
- Graph telemetry remains available only when the operator explicitly provides
  `CANON_RUSTC_WRAPPER`.
- `tests/test_observe_validation_contract.py` now uses a real multiline search
  to reject any future default `rustc-wrapper = ...` config entry.
- `README.md` now uses `CANON_RUSTC_V3_ARTIFACT_DIR` consistently for graph
  capture instructions.

Remaining critical limits:

- `cargo`, `rustc`, `cargo fmt`, `cargo test`, and `cargo clippy` are still
  unavailable in this sandbox.
- `state/rustc/*/graph.json` is still absent.
- The current HEAD has not reproduced `examples/ollama_judgment.rs`.
- The panic-surface scan still reports 319 total panic-like calls, all inside
  test-classified Rust regions.

## Existing File Contents Observed

`GOAL.md` exists and defines Canon Agent as a deterministic, self-improving agent runtime with a frozen state-machine kernel, append-only replayable TLog, capability-layer intelligence, policy learning, and an LLM promotion ladder. It explicitly states that the kernel should never change, policy should encode learned patterns, LLM calls should become rarer and more targeted, and every decision/recovery/outcome should be replayable and auditable.

`score.md` did not exist before this Phase 1 evaluation.

## Repository Evidence

```text
tracked_files = 118
rust_files = 53
python_files = 4
shell_files = 4
rust_loc_src_examples = 15834
rust_test_attrs = 103
rust_cfg_test_modules = 2
unwrap_calls_src_examples = 316
expect_calls_src_examples = 3
panic_calls_src_examples = 0
unsafe_mentions_src_examples = 0
todo_fixme_mentions_in_source = 0
third_party_rust_dependencies = 0
```

Positive evidence:

- `src/lib.rs` and `src/main.rs` use `#![forbid(unsafe_code)]`.
- `Cargo.toml` declares no third-party Rust dependencies.
- The tree matches the intended layered architecture: `kernel`, `codec`, `runtime`, `capability`, and `api`.
- Capability modules exist for observation, context, memory, planning, LLM, judgment, tooling, verification, eval, policy, learning, and orchestration.
- Runtime code exposes TLog replay, durable runtime resume, semantic delta logic, command ledger receipts, and transition verification surfaces.
- There are 103 Rust `#[test]` declarations across source/examples.

Critical evidence:

- Root Rust validation could not be reproduced because `cargo` and `rustc` are unavailable in this sandbox.
- The restored `.cargo/config.toml` originally configured an absolute missing
  `rustc-wrapper`; Phase 2 removed that default so root validation no longer
  depends on the local graph-capture wrapper.
- No `state/rustc/*/graph.json` was present, so semantic graph telemetry, node counts, edge counts, and intent coverage are absent.
- The codebase still contains 316 `unwrap()` calls and 3 `expect()` calls in `src`/`examples`; these are not classified by production/test boundary in this scorecard.
- Runtime archive evidence is historical. It supports process discipline but does not replace current-head build/test proof.

## Validation Evidence

Commands attempted during this evaluation:

```text
python3 -m pytest -q                                  => pass, 12 tests
rustc --version                                      => unavailable
cargo --version                                      => unavailable
RUSTC_WRAPPER='' RUSTC_WORKSPACE_WRAPPER='' cargo fmt --check                 => unavailable
RUSTC_WRAPPER='' RUSTC_WORKSPACE_WRAPPER='' cargo test --all-targets          => unavailable
RUSTC_WRAPPER='' RUSTC_WORKSPACE_WRAPPER='' cargo clippy --all-targets -- -D warnings => unavailable
```

Runtime archive evidence from `/mnt/data/ai-runtime.tar.gz`:

```text
runtime_manifest_base_commit = a4f94113d15ee6e01b35fd87edf78c9f654ca305
runtime_archive_parse_status = pass
runtime_archive_member_count = 28
runtime_archive_log_total = 8545
runtime_archive_download_total = 78
runtime_archive_conversation_snapshots = 0
runtime_download_history_record_count = 66
runtime_download_history_by_classification = {delta_applied:19, download_event:28, live_evidence:18, stale_advisory:1}
runtime_performance_signal_present = true
runtime_performance_budget_status = pass
project_agent_elapsed_ms_count = 1050
project_agent_elapsed_ms_median = 189027.187
project_agent_elapsed_ms_p95 = 1297595.353
project_agent_elapsed_ms_max = 1778293.765
```

Historical live Ollama evidence exists in the runtime archive:

```text
provider = ollama
model = qwen2.5-coder:7b
receipt_verified = true
tamper_rejected = true
tampered_fields_rejected = 17/17
endpoint_verified = true
proof_order_verified = true
durable_proof_verified = true
judgment_passed = true
```

This is useful supporting evidence, but it was not re-executed at current restored HEAD in this sandbox.

## Axis Detail

| Axis | Score | Critical basis |
|---|---:|---|
| I | 6.5 | Strong typed architecture and capability decomposition; intelligence is mostly scaffolded and not proven as closed-loop autonomy. |
| E | 6.5 | Zero Rust dependencies and deterministic surfaces help efficiency; Phase 2 removes default wrapper friction, but long historical project-agent turn times and many artifact/log surfaces remain. |
| C | 5.5 | Python tests pass and config validation is stronger; root Rust fmt/test/clippy are still unavailable, so correctness cannot be claimed without toolchain reproduction. |
| A | 8.0 | Repository structure closely matches `GOAL.md`; score capped because implemented proof trails do not yet demonstrate the full stated end state. |
| R | 5.9 | No unsafe code and recovery concepts exist; the default missing-wrapper failure is closed, but missing graph, unavailable Rust tests, and many test unwraps still reduce robustness. |
| P | 5.8 | Deterministic runtime should be cheap once built; actual historical automation p95 is about 21.6 minutes and local performance was not reproduced. |
| S | 5.8 | Modules for orchestration and capability routing exist; no evidence of high-concurrency or multi-objective scaling was reproduced. |
| D | 7.5 | Kernel phases, gates, transition verification, hash-linked events, and replay surfaces are strong; formal proof and graph telemetry are absent. |
| T | 7.3 | GOAL, README, runtime archive metrics, validation scripts, and manifests are transparent; Phase 2 adds plan.md, but conversation snapshots are absent. |
| Co | 7.2 | Operational docs and delta discipline are useful; Phase 2 removes the default local wrapper path, but the Rust toolchain itself is still not portable here. |
| Em | 6.8 | The repo gives a clear skeleton for deterministic agents and cleaner validation setup; unavailable root Rust validation still limits operator trust. |
| B | 6.5 | Potential benefit is high for auditable automation; current proof supports prototype benefit more than deployed agent benefit. |
| L | 5.4 | Learning and policy modules exist; current-head policy promotion/replay trace was not reproduced. |
| Si | 5.7 | Zero third-party Rust deps and crisp layers help simplicity; removing the default wrapper path simplifies bootstrap, but 15.8k Rust LOC, broad re-exports, and 316 unwraps increase cognitive load. |
| F | 6.9 | Append-only logs, receipts, policy store, and capability separation are future-friendly; Phase 2 improves portability, while missing formal verification still caps confidence. |

## Risk Register

| Risk | Severity | Evidence | Closure requirement |
|---|---:|---|---|
| Root Rust validation unavailable | High | `cargo`/`rustc` not found | Provide toolchain and rerun fmt/test/clippy. |
| Non-portable wrapper config | Closed in Phase 2 | `.cargo/config.toml` no longer points to absent wrapper path | Keep regression test enforcing explicit wrapper opt-in. |
| Missing graph telemetry | High | no `state/rustc/*/graph.json` | Regenerate graph and publish node/edge/intent metrics. |
| Production panic surface unclassified | Medium | 316 unwraps, 3 expects | Classify test-only vs runtime paths; reduce hot-path panics. |
| Learning proof not closed | High | no reproduced policy learning replay trace | Add observation→eval→learning integration proof. |
| Semantic artifact proof not closed | High | no reproduced current-head semantic artifact verification | Bind artifact receipts to proof replay and test it. |
| Historical evidence can go stale | Medium | runtime archive contains one stale advisory | Enforce base/head checks before consuming downloaded artifacts. |
| Runtime snapshots absent | Medium | conversation snapshot count is 0 | Include redacted snapshots or document why excluded. |

## Next Closure Targets

1. Restore a Rust toolchain and rerun `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings` with wrapper overrides cleared.
2. Provide `CANON_RUSTC_WRAPPER` explicitly and regenerate `state/rustc/*/graph.json`.
3. Re-run `examples/ollama_judgment.rs` at current HEAD and attach receipt/proof replay output.
4. Add one current-head integration trace for observation → judgment → eval → learning → policy promotion.
5. Classify and reduce `unwrap()` / `expect()` surfaces in runtime and capability code.