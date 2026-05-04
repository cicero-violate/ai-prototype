# Canon Agent Scorecard

## Variables

```text
K = kernel determinism and state-machine clarity
C = codec, TLog, durability, and replay surface
V = verification, receipt binding, and proof replay depth
P = capability, policy, learning, and autonomy maturity
B = build, test, and toolchain reproducibility
E = runtime archive, cache, and download-history evidence
N = nested router-server validation evidence
D = documentation, delta discipline, and operational clarity
S = overall reproduced-goodness score
GOOD = strongest reproduced axis
```

## Equations

```text
S = (K · C · V · P · B · E · N · D)^(1/8)
GOOD = max(K,C,V,P,B,E,N,D)
```

One-line explanation: the project score is the geometric mean of reproduced evidence; missing validation on one axis must reduce the whole score.

## Score Summary

```text
K = 8.4 / 10
C = 7.8 / 10
V = 7.3 / 10
P = 6.0 / 10
B = 4.0 / 10
E = 8.0 / 10
N = 7.6 / 10
D = 7.4 / 10

S = 6.90 / 10
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10 = good
```

Judgment: strong deterministic-runtime prototype; not yet a reproducibly validated autonomous agent.

## Scope

```text
stage = EXECUTE_TURN_004
source_changes_allowed = true
source_changes_made = true
scorecard_updated = true
restored_bundle = /mnt/data/ai.bundle
restored_repo_path = /mnt/data/eval-ai-repo
observed_branch = main
observed_head = c2f34ba6de533809414f6b11cccdd6592926d619
primary_runtime_archive = /mnt/data/ai-runtime.tar.gz
related_runtime_archives = /mnt/data/chatgpt-project-agent-runtime.tar.gz, /mnt/data/router-server-runtime.tar.gz
validation_report = target/observe/validation-report-execute.ndjson
```

Tracked execute-stage changes are limited to validation tooling and evaluation documents. Kernel, runtime, capability, API, router, and application logic are unchanged.

## Goal Alignment

`GOAL.md` defines a deterministic self-improving agent runtime with a frozen kernel, append-only TLog, auditable replay, policy learning, LLM promotion, and capability layers. The restored topology matches that target at a static level:

```text
src/kernel
src/codec
src/runtime
src/api
src/capability/{context,eval,judgment,learning,llm,memory,observation,orchestration,planning,policy,tooling,verification}
```

The highest-confidence claim is architectural shape. The lowest-confidence claim is live autonomous self-improvement, because current-head root Rust tests, semantic graph telemetry, live Ollama execution, semantic artifact verification, and policy-learning replay were not reproduced in this sandbox.

## Restored Repository Evidence

```text
bundle_verify = pass
bundle_ref_count = 11
bundle_head = c2f34ba6de533809414f6b11cccdd6592926d619
bundle_history = complete
tracked_files = 188
rust_files_src_examples = 53
mjs_files = 41
rust_loc_src_examples = 15803
rust_test_attrs = 103 #[test] + 2 #[cfg(test)] = 105
unwrap_calls_src_examples = 316
expect_calls_src_examples = 9
panic_calls_src_examples = 0
unsafe_mentions_src_examples = 0 source unsafe sites observed; src/lib.rs and src/main.rs forbid unsafe_code
todo_fixme_mentions = 0
```

Positive evidence:

- `#![forbid(unsafe_code)]` is present on the Rust library and binary.
- `Cargo.toml` has package `ai 0.1.0`, edition `2024`, and zero third-party Rust dependencies.
- The source tree is decomposed into kernel, codec, runtime, API, and capability layers.
- The repo declares 103 Rust `#[test]` items plus two `#[cfg(test)]` sections.

Risk evidence:

- 316 `unwrap()` calls and 9 `expect()` calls remain unclassified; most are concentrated in `src/lib.rs`.
- Static tests exist, but cargo was unavailable here, so the root Rust test suite was not reproduced at this head.
- No generated `state/rustc/*/graph.json` exists in the restored repository.

## Recent Git History Evidence

```text
HEAD = c2f34ba starting agent run
recent_validation_change = 60364d4 Add portable router validation fallback
large_router_deletion = b74009c save removed ai-chromium/router-server and retained router-server_bak path
prior_router_change = 7beeff8 Apply local router validation changes
prior_validation_launcher = 29e3b46 Implement portable validation launcher
prior_observe_score = 418f672 Observe ai repository runtime evidence
prior_eval_score = ed0e480 Evaluate ai repository scorecard
embedded_upload_bundle = .repo-agent-runtime/upload-bundles/ai.bundle
apply_worktree_dirs_present = 07ad58b4bf0e, 12f02b6a0f39, b9830281da56, d47aaa7d3487
```

Risk: the recent history contains agent-loop artifacts and a large router-server removal/backup transition. That may be intentional simplification, but it raises lineage risk unless receiver-side bundle application and validation are always tied to exact base/head commits.

## Validation Evidence

Executed command:

```bash
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz \
CANON_DELTA_BASE=c2f34ba6de533809414f6b11cccdd6592926d619 \
CANON_OBSERVE_REPORT=target/observe/validation-report-execute.ndjson \
bash scripts/observe_validation.sh
```

Observed summary:

```text
validation_status = partial
validation_command_count = 6
validation_test_count = 41
failed_required_commands = []
git_status_clean = false
git_status_short = [" M score.md"]
git_diff_check = pass
router_offline_tests = pass
router_test_count = 41
cargo_available = false
rustc_available = false
rust_toolchain_source = missing
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
missing_signal_count = 12
delta_base_is_ancestor = true
tracked_file_count = 188
rust_test_attr_count = 103
unwrap_call_count_src_examples = 316
```

Interpretation: available non-Rust validation passed, including nested router offline checks. Root Rust validation is neither passed nor failed; the proof is absent because cargo/rustc are unavailable in this sandbox. The validation report now records repo metrics, wrapper-override state, and explicit delta-base ancestry so missing evidence is machine-readable instead of implied.

## Build And Toolchain Constraints

```text
Cargo.toml.package = ai 0.1.0 edition 2024
Cargo.toml.dependencies = none
.cargo/config.toml.rustc_wrapper = /workspace/ai_sandbox/canon-rustc-v2/target/debug/canon-rustc-v2
rustc_wrapper_path_exists = false
wrapper_override_required = true
wrapper_override_used = false
delta_base_is_ancestor = true
state_rustc_graph_json_present = false
strict_rustflags_present = -Dwarnings, -Ddead-code, -Dclippy::allow_attributes_without_reason
```

The strict wrapper configuration is directionally correct for graph-backed semantic validation, but it is not portable as restored here. Without cargo, rustc, and the wrapper binary, root correctness claims remain unclosed.

## Runtime Archive Evidence

Primary archive: `/mnt/data/ai-runtime.tar.gz`.

```text
runtime_archive_parse_status = pass
runtime_archive_member_count = 39
runtime_archive_manifest_schema = 1
runtime_manifest_base_commit = c2f34ba6de533809414f6b11cccdd6592926d619
runtime_archive_log_total = 4516
runtime_archive_download_total = 38
runtime_archive_conversation_snapshots = 0
runtime_archive_cache_files = 0
runtime_candidate_ledger_rows = 20
runtime_candidate_selected_rows = 20
runtime_message_roles = {system:70, user:35, tool:685, assistant:1368}
runtime_artifact_aliases = {repo-delta-004.bundle:33, DELTA_MANIFEST.md:21, ai-observe-validation-output.txt:2, ai-router-tests-run.txt:2}
runtime_download_history_by_classification = {stale_advisory:1}
runtime_download_history_reason = base_commit_mismatch
runtime_integrity_scan_findings = 0
runtime_leak_scan_findings = 0
excluded_runtime_files_by_reason = {apply-worktree:1719, generated-bundle:14, generated-runtime-archive:1, secret-token-cache:1, signed-url-cache:7}
```

This proves meaningful automation-loop activity, retained download ledgers, redacted message/download traces, and manifest-level integrity/leak scans. It does not prove that the restored root crate currently builds, executes the Ollama path, regenerates semantic graph telemetry, or completes an autonomous objective loop.

Related archive observations:

```text
chatgpt-project-agent-runtime.baseCommit = b13c6f672184ccbb117990482347457c0ba1899a
chatgpt-project-agent-runtime.downloadHistoryByClassification = {stale_advisory:1}
router-server-runtime.baseCommit = 9bc0bc2d3063d4412b1a6abdd7de85ca1b83d2c3
router-server-runtime.downloadHistoryByClassification = {stale_advisory:1}
all_observed_runtime_manifests.integrityScan.findingCount = 0
all_observed_runtime_manifests.leakScan.findingCount = 0
```

Risk: every inspected runtime manifest had one stale-advisory download-history classification caused by `base_commit_mismatch`; this reinforces the need for mandatory base/head gating before applying downloaded artifacts.

## Existing Download Artifact Evidence

Current `downloads/DELTA_MANIFEST.md` records:

```text
download_manifest_base_commit = 0037761cacb4b573af603bf28c7d5f3ca6f3637d
download_manifest_head_commit = 60364d4ca23ce92d345ca18a6a5bd436c309996d
changed_files = ai-chromium/router-server_bak/run_tests.sh, score.md, scripts/observe_validation.sh
validation_status = partial
router_test_count = 41
runtime_archive_log_total = 4018
runtime_archive_download_total = 34
runtime_archive_conversation_snapshots = 0
```

This is useful historical evidence, not current-head proof. Current observed head is `c2f34ba6de533809414f6b11cccdd6592926d619`, and the runtime archive now reports 4516 log rows and 38 download rows.

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

The validation report marks 12 missing signals. The highest-impact absences are root Rust validation, semantic graph telemetry, policy-learning replay, semantic artifact verification, and the live Ollama judgment path.

## Risk Register

| Risk | Severity | Evidence | Required closure |
|---|---:|---|---|
| Root Rust validation unavailable | High | `cargo` and `rustc` missing; fmt/test/clippy unavailable | Provide toolchain; rerun fmt/test/clippy |
| Configured rustc wrapper missing | High | wrapper path absent | Include/build wrapper or make override deterministic |
| Graph telemetry absent | High | no `state/rustc/*/graph.json` | Regenerate graph and record node/edge/intent metrics |
| Ollama judgment path unproven at current head | Medium | env missing; example skipped | Run local endpoint and verify receipt/proof events |
| Learning/policy replay not demonstrated | High | validation flag missing policy learning replay trace | Add current-head observation→eval→learning integration trace |
| Semantic artifact verification not demonstrated | High | validation flag missing semantic artifact verification test | Bind artifact receipts to proof replay |
| Stale download hazard | Medium | runtime manifests contain `base_commit_mismatch` stale advisory | Enforce mandatory base/head checks before applying artifacts |
| Runtime panics unclassified | Medium | 316 unwrap + 9 expect calls | Classify test-only vs production paths; reduce hot-path panics |
| Conversation snapshots absent | Medium | `runtime_archive_conversation_snapshots = 0` | Include redacted snapshots or document intentional omission |
| Large router-server backup transition | Medium | recent `b74009c` removed active router path and retained `router-server_bak` | Confirm intended topology and receiver validation path |

## Axis Detail

### K — Kernel determinism: 8.4 / 10

The kernel has explicit phases, gates, evidence kinds, packet/state invariants, and no unsafe code. The score is capped because formal proof is not present and current-head root tests were not reproduced.

### C — Codec / TLog durability: 7.8 / 10

The repo exposes NDJSON encode/decode/load/write paths, durable runtime resume, and command ledger reconstruction. The score is capped because cargo tests were unavailable and no current tlog replay proof was generated during this observe pass.

### V — Verification proof depth: 7.3 / 10

The source contains proof/receipt structures and replay functions for canonical effects, tooling, process, LLM, and verification records. The score is capped because semantic artifact verification and Ollama proof replay were not executed at this head.

### P — Capability / policy maturity: 6.0 / 10

The intended capability set exists, including observation, context, memory, planning, LLM, judgment, tooling, verification, eval, policy, learning, and orchestration. It remains prototype-level because validation still marks external observation stream, external API action, semantic verification, and policy learning replay as missing.

### B — Build / test reproducibility: 4.0 / 10

Nested router validation passed, and the validation script now deterministically records toolchain availability, missing-wrapper override state, repo metrics, and delta-base ancestry. The score remains capped because root Rust fmt/test/clippy still could not run.

### E — Runtime archive evidence: 8.0 / 10

The archive is parseable and contains 4516 log rows, 38 download rows, 20 selected candidate-ledger rows, redaction metadata, and zero integrity/leak findings. It is capped by absent conversation snapshots and because archive evidence is historical, not a substitute for current-head build validation.

### N — Router-server evidence: 7.6 / 10

The nested router offline test runner passed 41 checks. This supports embedded router coverage but does not validate the Rust agent runtime.

### D — Documentation / delta discipline: 7.4 / 10

`GOAL.md`, `README.md`, runtime archive handling, download history advisories, current-turn manifest generation, and explicit delta-base checks show strong operational discipline. The score is capped because some claims in `GOAL.md` exceed what was reproduced here, and runtime manifests show stale-advisory history.

## Next Closure Targets

1. Restore cargo/rustc plus the configured rustc wrapper; rerun fmt/test/clippy.
2. Regenerate `state/rustc/*/graph.json` and publish graph node/edge/intent coverage metrics.
3. Run `examples/ollama_judgment.rs` against a local endpoint and verify durable receipt/proof replay.
4. Add one integration trace that proves observation → judgment → eval → policy learning on current head.
5. Classify or reduce `unwrap()` / `expect()` surfaces in runtime paths.