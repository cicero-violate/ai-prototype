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
I  = 7.3 / 10
E  = 7.1 / 10
C  = 7.5 / 10
A  = 8.5 / 10
R  = 7.5 / 10
P  = 6.6 / 10
S  = 6.5 / 10
D  = 8.4 / 10
T  = 8.8 / 10
Co = 7.7 / 10
Em = 7.6 / 10
B  = 7.3 / 10
L  = 6.6 / 10
Si = 6.3 / 10
F  = 7.6 / 10

G = 7.39 / 10
max(G) = good
```

Judgment: the repository is improving as an auditable prototype. The best current strength is deterministic replay and validation coverage. The main weakness remains incomplete live-system proof: no fresh Ollama execution, generated rustc-wrapper graph telemetry, clippy/rustfmt closure, or production API deployment was proven in this environment.

## Scope

```text
stage = PHASE_2_EXECUTE_TURN_002
source_changes_allowed = true
source_changes_made = true
restored_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/phase2-ai/ai
observed_branch = main
base_commit = c8936ccbe7a4d8a3ca7d251de57fda9afd28b15a
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_manifest_base_commit = c8936ccbe7a4d8a3ca7d251de57fda9afd28b15a
runtime_manifest_base_matches_delta_base = true
existing_goal_md = true
existing_score_md_before_phase = true
```

## Phase 2 Delta

Implemented closure:

- Rewrote `plan.md` from current `GOAL.md` and `score.md` for base `c8936ccbe7a4d8a3ca7d251de57fda9afd28b15a`.
- Improved `scripts/observe_validation.sh` so source-derived external capability evidence emits:
  - `external_observation_stream_evidence_files`
  - `external_observation_stream_evidence_tokens`
  - `external_api_action_evidence_files`
  - `external_api_action_evidence_tokens`
  - `semantic_artifact_verification_evidence_files`
  - `semantic_artifact_verification_evidence_tokens`
- Extended `tests/test_observe_validation_contract.py` to require those evidence paths and token mappings.

Why this matters: boolean capability signals are insufficient for evidence-backed audit. The validation report now exposes which tracked files and tokens support each external-surface claim, reducing ambiguity and making future regressions easier to localize.

## Existing File Content Review

### `GOAL.md`

`GOAL.md` defines a deterministic, auditable, self-improving agent runtime with a frozen kernel, replayable TLog, bounded recovery, external capabilities, policy learning, and LLM promotion. Current implementation aligns at prototype/test level through kernel/replay tests, capability records, observation ingress, API envelopes, semantic verification, policy learning, and LLM receipt/proof paths.

### Previous `score.md`

The previous scorecard correctly identified that root Rust and Python suites pass and that validation was source-derived for the major external surfaces. Its remaining weakness was that evidence was still too compressed for audit: present/missing booleans did not tell the operator where proof lived in tracked source.

## TODO / FIXME Review

Commands used:

```bash
rg -n --hidden -g '!.git/**' -g '!target/**' -g '!patch/**' -g '!score.md' 'TODO|FIXME' .
rg -n --hidden -g '!.git/**' -g '!target/**' 'TODO|FIXME' .
```

Findings after this phase:

```text
active TODO/FIXME markers outside score.md, target, and patch archive = 0
archived patch TODO markers = 4
```

Archived markers are in `patch/improve_score_codebase.apply_patch` and reference older placeholder work in API protocol, capability payload, API routes, and codec wire encoding. They are historical evidence of prior design pressure, not active source markers.

## Validation Evidence

| Command | Result | Evidence |
|---|---:|---|
| `python3 /mnt/data/bootstrap_rustc_session.py` | pass | rustc 1.75.0, cargo 1.75.0, offline probe pass, internal registry dependency probe pass |
| `cargo test --all-targets` | pass | 103 Rust tests passed; binary and examples compiled |
| `python3 -m unittest discover -s tests -p 'test_*.py' -v` | pass | 23 Python tests passed |
| `python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json` | pass | 4 check groups passed; missing count 0 |
| `python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json` | pass | production total 0; test total 319; example total 1 |
| `bash -n scripts/observe_validation.sh` | pass | shell syntax valid |
| `python3 -m py_compile scripts/write_delta_manifest.py scripts/validate_policy_learning_trace.py scripts/validate_rust_panic_surface.py` | pass | Python validation scripts compile |
| `git diff --check` | pass | whitespace check passed |
| `CANON_DELTA_BASE=<B> CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz CANON_OBSERVE_REPORT=target/observe/validation-report-phase2.ndjson bash scripts/observe_validation.sh` | partial | emitted source evidence, runtime archive metrics, and delta metrics; timed out before final summary in this environment |

## Runtime Archive Evidence

The runtime tarball `/mnt/data/ai-runtime.tar.gz` was inspected. It contains logs, candidate/download ledgers, audit records, delta receipts, runtime summaries, prior-state indicators, download indexes, and a runtime manifest whose base commit matches this turn's delta base.

Observed metrics from archive inspection and validation output:

```text
runtime_archive_member_count = 108
runtime_archive_log_files = 45
runtime_archive_download_files = 25
runtime_archive_download_index_files = 41
runtime_archive_conversation_ledger_files = 20
runtime_archive_prior_state_files = 10
runtime_archive_delta_receipt_files = 8
runtime_archive_audit_files = 1
runtime_archive_current_run_summary_present = true
runtime_archive_runtime_manifest_present = true
runtime_manifest_base_matches_delta_base = true
runtime_performance_budget_status = pass
runtime_download_history_record_count = 200
```

Critical reading: runtime archive evidence is useful for continuity and performance history, but source-level validation against current HEAD remains authoritative.

## Dimension Scores

| Var | Score | Evidence-backed rationale |
|---|---:|---|
| I | 7.3 | Source evidence now exposes concrete files and token mappings for external surfaces. |
| E | 7.1 | Root Rust and Python validation remain runnable under the mandated bootstrap toolchain. |
| C | 7.5 | 103 Rust tests, 23 Python tests, policy trace validation, and panic-surface validation pass. |
| A | 8.5 | The change supports the `GOAL.md` requirement for auditable evidence and truthful capability claims. |
| R | 7.5 | Traceable evidence files reduce stale boolean and false-positive validation risk. |
| P | 6.6 | Runtime archive contains performance signals with budget pass; no fresh benchmark suite exists. |
| S | 6.5 | Token-to-file evidence scales better as source surfaces grow. |
| D | 8.4 | Deterministic validation and source-token scans remain repeatable. |
| T | 8.8 | Transparency improves because report consumers can see where each capability proof lives. |
| Co | 7.7 | Contributors get contract tests that protect evidence trace fields. |
| Em | 7.6 | Operators can distinguish missing live signals from existing deterministic source coverage. |
| B | 7.3 | The repo is more useful as a validated handoff artifact; deployed user value remains unproven. |
| L | 6.6 | Policy-learning trace still passes; validation evidence is now more learnable by future agents. |
| Si | 6.3 | The validation model is slightly richer, but still centralized in one large observe script. |
| F | 7.6 | Explicit evidence fields are more maintainable than opaque booleans as tests evolve. |

## Risk Register

| Risk | Severity | Evidence | Closure requirement |
|---|---:|---|---|
| Full observe script completion unstable here | Medium | observe run emitted early metrics but timed out before final summary | Split observe validation into bounded subcommands or reduce nested subprocess cost. |
| `cargo fmt` unavailable | Medium | bootstrap Cargo lacks `cargo-fmt` / `rustfmt` | Add rustfmt component to bootstrap or validate formatting with a complete toolchain. |
| `cargo clippy` unavailable | Medium | bootstrap Cargo lacks clippy | Add clippy component or run with a complete toolchain. |
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
