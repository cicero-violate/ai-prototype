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
G  = geometric-mean goodness score
```

## Equation

```text
G = (I · E · C · A · R · P · S · D · T · Co · Em · B · L · Si · F)^(1/15)
GOOD = max(I,E,C,A,R,P,S,D,T,Co,Em,B,L,Si,F)
max(G) = good
```

One-line explanation: Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers the whole system.

## Score Summary

```text
I  = 7.6 / 10
E  = 6.4 / 10
C  = 6.3 / 10
A  = 7.8 / 10
R  = 6.2 / 10
P  = 6.0 / 10
S  = 6.2 / 10
D  = 7.5 / 10
T  = 8.0 / 10
Co = 6.7 / 10
Em = 7.2 / 10
B  = 7.5 / 10
L  = 6.3 / 10
Si = 5.5 / 10
F  = 6.9 / 10

G = 6.77 / 10
GOOD = max(I,E,C,A,R,P,S,D,T,Co,Em,B,L,Si,F) = T = 8.0 / 10 = good
max(G) = 6.77 / 10 = reproduced goodness under current evidence
```

Judgment: strong deterministic-agent architecture; not yet a reproducibly validated autonomous agent runtime.

## Scope

```text
stage = EXECUTE turn 004
source_code_changes_allowed = true
source_code_changes_made = true
scorecard_updated = true
restored_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/ai-eval/repo
observed_branch = main
observed_head = 4e8a41f7f6a4c31d275e986147d6193c1d1f3895
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_archive_sha256 = 4297b922e052b51bd094e31314127bb97a62118700c631d8e6c20c0be9f2a646
validation_report = target/observe/validation-report-execute.ndjson
validation_report_sha256 = emitted in /mnt/data/DELTA_MANIFEST.md
```

Execute turn 004 adds a small offline behavioral router contract test and updates the router validation runner to report syntax and behavior coverage separately. No Rust kernel, runtime, API, capability, or semantic state-machine source was changed.

## Goal Alignment

`GOAL.md` defines a deterministic, self-improving agent runtime with a frozen kernel, append-only transaction log, auditable replay, bounded recovery, policy learning, LLM promotion, and layered capabilities.

Static topology matches the declared architecture:

```text
src/kernel
src/codec
src/runtime
src/api
src/capability/{context,eval,judgment,learning,llm,memory,observation,orchestration,planning,policy,tooling,verification}
```

Highest-confidence claim: the repository has the intended architectural shape.

Lowest-confidence claim: live autonomous self-improvement is not reproduced, because Rust validation, semantic graph telemetry, live Ollama judgment, semantic artifact verification, external observation/action tests, and policy-learning replay are missing in this sandbox.

## Restored Repository Evidence

```text
bundle_verify = pass
bundle_ref_count = 13
bundle_head = 4e8a41f7f6a4c31d275e986147d6193c1d1f3895
bundle_history = complete
tracked_files = 188
rust_files_src_examples = 53
rust_loc_src_examples = 15803
rust_test_attrs = 103 #[test] + 2 #[cfg(test)]
unsafe_token_count_src_examples = 0
unwrap_calls_src_examples = 316
expect_calls_src_examples = 9
panic_calls_src_examples = 0
todo_fixme_mentions = 0
third_party_rust_dependencies = 0
tracked_artifact_or_generated_files = 51
patch_archive_files = 45
```

Positive evidence:

- `src/lib.rs` and `src/main.rs` use `#![forbid(unsafe_code)]`.
- `Cargo.toml` declares package `ai 0.1.0`, edition `2024`, and no third-party Rust dependencies.
- The source tree separates kernel, codec, runtime, API, and capability layers.
- The codebase contains substantial unit-test surface: 103 Rust `#[test]` items plus two `#[cfg(test)]` sections.
- `README.md` documents restore, validation, root Rust checks, Ollama checks, delta artifacts, and current constraints.
- The root manifest has zero third-party Rust dependencies, so dependency supply-chain risk is low for the Rust crate itself.

Risk evidence:

- 316 `unwrap()` calls and 9 `expect()` calls remain unclassified in `src` and `examples`.
- No generated `state/rustc/*/graph.json` exists in the restored repository.
- The configured rustc wrapper path is absolute and absent in this sandbox: `/workspace/ai_sandbox/canon-rustc-v2/target/debug/canon-rustc-v2`.
- The repository carries prior generated loop artifacts under `.repo-agent-runtime`, `downloads`, and 45 patch archive files; useful for lineage, but noisy for simplicity and collaboration.

## Recent Git History Evidence

```text
HEAD = 4e8a41f starting agent run
parent = 3f23e08 starting agent run
recent_validation_receipt_change = 4b4cf03 Close validation receipt surface
recent_validation_fallback = 60364d4 Add portable router validation fallback
large_router_transition = b74009c save removed ai-chromium/router-server and retained router-server_bak
embedded_upload_bundle = .repo-agent-runtime/upload-bundles/ai.bundle
worktree_refs_in_bundle = 9
```

Risk: the bundle includes worktree refs and agent-loop upload/download artifacts. That improves recovery context, but it also increases receiver confusion unless base/head validation and stale-artifact rejection stay mandatory.

## Validation Evidence

Executed command:

```bash
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz \
CANON_DELTA_BASE=4e8a41f7f6a4c31d275e986147d6193c1d1f3895 \
CANON_OBSERVE_REPORT=target/observe/validation-report-observe.ndjson \
bash scripts/observe_validation.sh
```

Observed summary:

```text
validation_status = partial
validation_command_count = 6
validation_test_count = 45
failed_required_commands = []
git_status_clean = false
git_status_short = execute worktree dirty before commit; final clean state is recorded in DELTA_MANIFEST.md
git_diff_check = pass
router_offline_tests = pass
router_test_count = 45
router_syntax_checks = 41
router_behavior_checks = 4
cargo_available = false
rustc_available = false
rust_toolchain_source = missing
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
missing_signal_count = 12
delta_base_is_ancestor = true
state_graph_present = false
```

Interpretation: available non-Rust validation passed. Root Rust correctness is not proven because `cargo` and `rustc` are unavailable in this sandbox. Router offline tests now cover 41 syntax checks plus four pure behavioral contract checks for redaction, receipts, provider capability rejection, and structure-only classification. This still does not prove live CDP/API behavior.

The dirty git status is expected before the final commit. Final clean validation is required in `/mnt/data/DELTA_MANIFEST.md`.

## Build and Test Metadata Evidence

```text
root_package = ai 0.1.0
rust_edition = 2024
rust_dependencies = 0
library_entry = src/lib.rs
binary_entry = src/main.rs
root_run_tests_script = missing
router_validation_script = ai-chromium/router-server_bak/run_tests.sh
router_validation_depth = node --check syntax pass over 41 .mjs files + node:test pass over 4 contract checks
strict_rustflags = -Dwarnings, -Dunused, -Ddead-code, -Dclippy::allow_attributes, -Dclippy::allow_attributes_without_reason
rustc_wrapper_config = absolute path to canon-rustc-v2
rustc_wrapper_portable_here = false
```

Constraint: the metadata is disciplined, but the validation entrypoint is split between a missing root runner, a shallow router syntax runner, and cargo commands that cannot execute in this sandbox.

## Runtime Archive Evidence

Primary archive: `/mnt/data/ai-runtime.tar.gz`.

```text
runtime_archive_parse_status = pass
runtime_archive_sha256 = 4297b922e052b51bd094e31314127bb97a62118700c631d8e6c20c0be9f2a646
runtime_archive_member_count = 49
runtime_manifest_schema = 1
runtime_manifest_base_commit = 4e8a41f7f6a4c31d275e986147d6193c1d1f3895
runtime_archive_log_total = 5677
runtime_archive_download_total = 62
runtime_network_request_log_rows = 2318
runtime_agent_log_rows = 448
runtime_archive_conversation_snapshots = 0
runtime_archive_cache_files = 0
runtime_message_rows = 2773
runtime_message_roles = {system:87, user:46, tool:878, assistant:1762}
runtime_candidate_ledger_rows = 34
runtime_candidate_selected_rows = 34
runtime_audit_rows = 104
runtime_audit_events = {candidate_ledger_written:45, loop_iteration_observed_audit_linked:23, runtime_archive_created:10, delta_pair_verified:9, delta_applied:9, live_cdp_evidence_summary:8}
runtime_download_history_by_classification = {stale_advisory:1}
runtime_download_history_reason = base_commit_mismatch
runtime_integrity_scan_findings = 0
runtime_leak_scan_findings = 0
excluded_runtime_files_by_reason = {apply-worktree:2083, generated-bundle:16, generated-runtime-archive:1, secret-token-cache:1, signed-url-cache:9}
```

This proves meaningful agent-loop and artifact-lineage activity. The archive intentionally excludes signed URL caches and secret-token cache material, which is correct for leakage control. It does not prove current-head Rust build success, graph extraction, Ollama judgment execution, external API behavior, semantic artifact verification, or learning-policy promotion.

## Missing Validation Signals

```text
missing_root_rust_toolchain = true
missing_cargo_fmt = true
missing_cargo_test = true
missing_clippy = true
missing_cargo_run_ollama_judgment = true
missing_generated_graph_json = true
missing_rustc_wrapper_telemetry = true
missing_conversation_snapshot = true
missing_external_observation_stream_test = true
missing_external_api_action_test = true
missing_semantic_artifact_verification_test = true
missing_policy_learning_replay_trace = true
missing_runtime_download_history = false
missing_artifact_apply_worktree = false
```

Critical gap: the system has strong structural and lineage evidence, but the highest-value correctness claims remain unclosed until root Rust validation and semantic proof surfaces run reproducibly.

## Dimension Rationale

| Dimension | Score | Evidence-backed rationale |
|---|---:|---|
| I | 7.6 | Strong capability taxonomy, LLM receipt/proof surfaces, policy-learning modules, and deterministic state-machine design; live self-improvement remains unproven. |
| E | 6.4 | Zero Rust dependencies and compact root manifest help; 15,803 Rust LOC, large public re-export surface, 45 patch files, and generated artifact lineage reduce efficiency. |
| C | 6.3 | Router syntax and offline behavioral contract tests plus git diff checks pass; root Rust fmt/test/clippy are unavailable, so correctness is still partial. |
| A | 7.8 | Repository structure closely matches `GOAL.md`: frozen kernel, TLog, capabilities, replay, learning, and API layers. |
| R | 6.2 | Bounded recovery and verification modules exist; router behavioral contracts reduce one validation gap, but 316 `unwrap()` calls, missing toolchain, and missing graph telemetry remain. |
| P | 6.0 | Router offline suite completes 45 checks, including four behavioral contracts; no root runtime benchmark, throughput metric, or LLM latency proof is reproduced. |
| S | 6.2 | Layering supports growth, but external observation/action surfaces and policy-learning replay are not validated. |
| D | 7.5 | Kernel, gate ordering, TLog, replay, hashes, receipts, deterministic reducers, and validation report/manifest binding are prominent; proof remains incomplete without cargo validation. |
| T | 8.0 | GOAL, README, scorecard, validation report, runtime manifest, audit logs, and bundle refs provide unusually high traceability. |
| Co | 6.7 | Documentation, manifests, and executable offline contracts help collaboration; worktree refs, generated artifacts, backup router path, and patch accumulation increase onboarding friction. |
| Em | 7.2 | Delta workflow, validation script, API protocol, receipt surfaces, and a behavioral router test runner enable users to operate and inspect the system. |
| B | 7.5 | The project targets valuable autonomous-runtime infrastructure with auditability, replay, and learning; benefit depends on closing validation gaps. |
| L | 6.3 | Learning and policy modules exist and runtime archives capture traces; no reproduced policy-promotion replay trace proves compounding yet. |
| Si | 5.5 | The added test surface is small and direct; source volume, broad exports, artifact clutter, and unclassified unwraps still reduce simplicity. |
| F | 6.9 | Edition 2024, zero dependencies, strict flags, deterministic design, and portable offline router contracts help; absolute wrapper path and missing Rust validation reduce future-proofing. |

## Risk Register

| Risk | Severity | Evidence | Required closure |
|---|---:|---|---|
| Root crate correctness unproven | High | `cargo`, `rustc`, fmt, test, and clippy unavailable | Re-run with portable Rust toolchain and wrapper override; record exact report. |
| Semantic graph absent | High | `state_graph_present = false` | Rebuild with `canon-rustc` wrapper and store `state/rustc/*/graph.json`. |
| Live LLM receipt path unproven | High | `ollama_judgment_example = skipped_env_missing` | Run `cargo run --example ollama_judgment` against a local endpoint and verify receipts. |
| Self-improvement not demonstrated | High | `missing_policy_learning_replay_trace = true` | Add and run policy-promotion replay over completed TLog evidence. |
| Runtime artifacts can become stale | Medium | one `stale_advisory` download-history classification from base mismatch | Keep base/head gating mandatory before artifact apply. |
| Portability gap | Medium | absolute rustc-wrapper path missing | Vendor or parameterize wrapper bootstrap; document required override. |
| Simplicity debt | Medium | 45 patch files, generated artifacts, backup router path, 316 unwraps | Classify generated files, archive old patches, and replace unguarded unwraps on production paths. |

## Required Next Validation

```bash
RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check
RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets
RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings
CANON_OLLAMA_BASE_URL=http://127.0.0.1:11434/v1 \
CANON_OLLAMA_MODEL=qwen2.5-coder:7b \
cargo run --example ollama_judgment
```

Then regenerate semantic graph telemetry with the configured wrapper and re-score only after the validation report contains root Rust pass/fail evidence, graph node/edge metrics, receipt proof results, and learning replay output.