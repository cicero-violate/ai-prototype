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
I  = 7.2 / 10
E  = 7.1 / 10
C  = 7.4 / 10
A  = 8.4 / 10
R  = 7.4 / 10
P  = 6.5 / 10
S  = 6.4 / 10
D  = 8.3 / 10
T  = 8.6 / 10
Co = 7.6 / 10
Em = 7.5 / 10
B  = 7.2 / 10
L  = 6.5 / 10
Si = 6.2 / 10
F  = 7.5 / 10

G = 7.29 / 10
max(G) = good
```

Judgment: the repository is stronger after this phase because validation now reports existing external-observation, external API-action, and semantic-artifact-verification coverage from tracked source evidence instead of permanently marking those surfaces missing. The repo remains prototype-grade because live Ollama execution, generated rustc-wrapper graph telemetry, rustfmt/clippy availability, and production API deployment are still not proven in this environment.

## Scope

```text
stage = PHASE_2_EXECUTE_TURN_002
source_changes_allowed = true
source_changes_made = true
restored_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/ai-phase2/ai
observed_branch = main
base_commit = c86c4438852e9a0779dcc53e860b9c4fc185824c
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_manifest_base_commit = c86c4438852e9a0779dcc53e860b9c4fc185824c
runtime_manifest_base_matches_delta_base = true
existing_goal_md = true
existing_score_md_before_phase = true
```

## Phase 2 Delta

Implemented closure:

- Updated `plan.md` from current `GOAL.md` and `score.md`.
- Added `source_evidence()` to `scripts/observe_validation.sh`.
- Replaced three hardcoded missing flags with source-derived evidence:
  - `missing_external_observation_stream_test`
  - `missing_external_api_action_test`
  - `missing_semantic_artifact_verification_test`
- Emitted the corresponding positive evidence in validation output.
- Added a Python contract test proving the validation script keeps those signals source-derived.

Why this matters: the validation layer previously underreported capability coverage even when deterministic tests existed. This weakened transparency and made the scorecard more pessimistic than the source evidence justified.

## Existing File Content Review

### `GOAL.md`

`GOAL.md` defines a deterministic, auditable, self-improving agent runtime with a frozen kernel, replayable TLog, bounded recovery, and external capability learning. Current implementation aligns at the prototype/test level through kernel/replay tests, capability records, observation ingress, API envelopes, semantic verification, policy learning, and LLM receipt/proof paths.

### Previous `score.md`

The previous scorecard correctly identified that the repo validates under the mandated bootstrap toolchain and that runtime archive evidence is useful but subordinate to current-head validation. Its remaining weakness was that validation still hardcoded some existing capability-test surfaces as missing.

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

Archived markers are in `patch/improve_score_codebase.apply_patch` and reference older placeholder work in API protocol, capability payload, API routes, and codec wire encoding. They are not active source markers, but they remain historical evidence of prior design pressure around external protocol stability.

## Validation Evidence

| Command | Result | Evidence |
|---|---:|---|
| `python3 /mnt/data/bootstrap_rustc_session.py` | pass | rustc 1.75.0, cargo 1.75.0, offline probe pass, internal registry dependency probe pass |
| `cargo test --all-targets` | pass | 103 Rust tests passed; examples and main harnesses compiled |
| `python3 -m unittest discover -s tests -p 'test_*.py' -v` | pass | 23 Python tests passed |
| `python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json` | pass | 4 check groups passed; missing count 0 |
| `python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json` | pass | production total 0; test total 319; example total 1 |
| `bash -n scripts/observe_validation.sh` | pass | validation script shell syntax valid |
| `python3 -m py_compile scripts/write_delta_manifest.py scripts/validate_policy_learning_trace.py scripts/validate_rust_panic_surface.py` | pass | Python validation scripts compile |
| `git diff --check` | pass | whitespace check passed |
| `scripts/observe_validation.sh` | partial | emitted `source_evidence=true` for all three repaired surfaces plus runtime archive metrics; full script did not complete inside the execution window |

## Runtime Archive Evidence

The runtime tarball `/mnt/data/ai-runtime.tar.gz` was inspected. It contains conversation ledgers, download ledgers, audit records, delta-apply receipts, a current-run summary, downloaded prior artifacts, and a runtime manifest whose base commit matches this turn's delta base.

Observed metrics from the validation report:

```text
runtime_archive_member_count = 101
runtime_archive_log_files = 41
runtime_archive_download_files = 23
runtime_archive_download_index_files = 37
runtime_archive_conversation_ledger_files = 18
runtime_archive_prior_state_files = 9
runtime_archive_delta_receipt_files = 7
runtime_archive_audit_files = 1
runtime_archive_current_run_summary_present = true
runtime_archive_runtime_manifest_present = true
runtime_manifest_base_matches_delta_base = true
runtime_performance_budget_status = pass
```

Critical reading: the archive provides useful historical evidence and performance signals, but it is not a replacement for source-level tests against the current HEAD.

## Dimension Scores

| Var | Score | Evidence-backed rationale |
|---|---:|---|
| I | 7.2 | Validation now recognizes source-level evidence for external observation, API action, and semantic verification. |
| E | 7.1 | Root Rust and Python validation remain runnable under the mandated bootstrap toolchain. |
| C | 7.4 | 103 Rust tests, 23 Python tests, policy trace validation, and panic-surface validation pass. |
| A | 8.4 | The change directly supports the `GOAL.md` demand for truthful, auditable evidence. |
| R | 7.4 | Source-derived validation reduces stale hardcoded missing-signal risk. |
| P | 6.5 | Runtime archive contains performance signals with budget pass, but no fresh benchmark suite exists. |
| S | 6.4 | Better evidence classification improves scalable repo-loop handoff; orchestration scale is still unproven. |
| D | 8.3 | Deterministic source-token evidence and replay tests improve repeatable validation. |
| T | 8.6 | Validation emits the repaired source-evidence booleans and runtime archive metrics explicitly. |
| Co | 7.6 | Contributors get less misleading validation output and a contract test for the behavior. |
| Em | 7.5 | Operators can distinguish missing live signals from existing deterministic test coverage. |
| B | 7.2 | The repo is more useful as a validated handoff artifact; deployed user value remains unproven. |
| L | 6.5 | Policy-learning trace still passes and is easier to interpret in the wider validation report. |
| Si | 6.2 | Replacing hardcoded flags with one source-evidence function simplifies the validation model. |
| F | 7.5 | Source-derived evidence is more maintainable than fixed missing flags as tests evolve. |

## Risk Register

| Risk | Severity | Evidence | Closure requirement |
|---|---:|---|---|
| `cargo fmt` unavailable | Medium | bootstrap Cargo lacks `cargo-fmt` / `rustfmt` | Add rustfmt component to bootstrap or validate formatting with a complete toolchain. |
| `cargo clippy` unavailable | Medium | bootstrap Cargo lacks clippy | Add clippy component or run with a complete toolchain. |
| Full observe script completion unstable here | Medium | validation emitted source/runtime metrics but did not finish in the execution window | Split long observe validation into smaller bounded commands or reduce nested subprocess cost. |
| Wrapper graph telemetry absent | High | no generated `state/rustc/*/graph.json` | Run wrapper graph capture with `CANON_RUSTC_WRAPPER`. |
| Live Ollama proof absent at current head | High | local Ollama example not executed here | Run `cargo run --example ollama_judgment` with configured local Ollama. |
| Production API deployment unproven | High | no live HTTP/gRPC deployment validation | Add executable API deployment and request/response tests. |
| Archived patch debt | Medium | four archived TODO markers | Confirm obsolete patch status or promote unresolved protocol work to active plan. |

## Next Closure Targets

1. Split `scripts/observe_validation.sh` into independently bounded validation steps so one slow subprocess cannot block summary emission.
2. Add rustfmt and clippy components to the bootstrap toolchain.
3. Generate `state/rustc/*/graph.json` with `CANON_RUSTC_WRAPPER` and record graph metrics.
4. Run `cargo run --example ollama_judgment` against local Ollama and record durable receipt/proof replay evidence.
5. Add live external API deployment validation.