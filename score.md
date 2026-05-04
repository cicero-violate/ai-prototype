# Canon Agent Scorecard

## Variables

```text
I  = Intelligence          E  = Efficiency        C  = Correctness
A  = Alignment             R  = Robustness         P  = Performance
S  = Scalability           D  = Determinism        T  = Transparency
Co = Collaboration         Em = Empowerment        B  = Benefit
L  = Learning              Si = Simplicity         F  = Future-Proofing
G  = geometric-mean goodness
```

## Equation

```text
G = (I·E·C·A·R·P·S·D·T·Co·Em·B·L·Si·F)^(1/15)
G = 6.77 / 10
max(G) = good
```

One-line explanation: Goodness is the geometric mean; one weak dimension lowers
the whole system.

## Score Summary

```text
I  = 7.4 / 10
E  = 6.4 / 10
C  = 5.9 / 10
A  = 7.7 / 10
R  = 6.4 / 10
P  = 5.5 / 10
S  = 5.9 / 10
D  = 8.0 / 10
T  = 8.4 / 10
Co = 6.9 / 10
Em = 7.5 / 10
B  = 7.1 / 10
L  = 6.0 / 10
Si = 5.9 / 10
F  = 7.3 / 10

G = 6.77 / 10
GOOD = max(G) = 6.77 / 10 = good
```

Judgment: this is an above-average autonomous-agent prototype with a coherent
kernel/capability/TLog architecture, but it is still evidence-limited. The
uploaded runtime archive materially improves traceability and this execution
stage hardened delta/manifest verification. The current restored environment
still cannot reproduce the root Rust build, test suite, graph telemetry, Ollama
receipt path, semantic verification, or policy-learning promotion trace. Treat
this as a serious prototype score, not production proof.

## Evidence

```text
bundle = /mnt/data/ai.bundle
restored_repo = /mnt/data/ai-repo
git_head = dce0d671d547613915f35f00926b45d34bd821c6
recent_history_shape = repeated starting-agent-run commits plus one compact-validation commit
source_delta = validation-script hardening, manifest hardening, score/plan update
score_md_changed_only = false
goal_md_present = true
tracked_files = 189
rust_files_src_examples = 53
rust_test_attrs = 103
rust_cfg_test_sections = 2
unsafe_token_count_src_examples = 0
panic_call_count_src_examples = 0
unwrap_calls_src_examples = 316
expect_calls_src_examples = 9
third_party_rust_dependencies = 0
tracked_runtime_download_bundle_files = 9

runtime_archive_ai_present = true
runtime_archive_path = /mnt/data/ai-runtime.tar.gz
runtime_archive_sha256 = 9cb696748f26b34176114bfcfb02fa0f9cac896b84764d49a4886caedb70524e
runtime_manifest_base_commit = dce0d671d547613915f35f00926b45d34bd821c6
runtime_archive_member_count = 84
runtime_manifest_included_files = 83
runtime_manifest_redacted_files = 35
runtime_archive_cache_files = 0
runtime_archive_conversation_snapshots = 0
runtime_candidate_ledger_files = 15
runtime_download_ledger_files = 15
runtime_message_ledger_files = 15
runtime_process_network_log_files = 5
runtime_candidate_rows = 66
runtime_candidate_error_count = 0
runtime_message_rows = 4372
runtime_log_rows_parsed = 5155
runtime_archive_log_total_observe = 9760
runtime_archive_download_total_observe = 167
runtime_manifest_download_history_records = 149
runtime_download_history_by_classification = delta_applied:14, download_event:22, download_ledger:99, live_evidence:13, stale_advisory:1
runtime_duplicate_artifact_aliases = 8
runtime_unique_download_alias_count = 17
runtime_duplicate_alias_top = repo-delta-004.bundle:25, DELTA_MANIFEST.md:14
runtime_excluded_by_reason = apply-worktree:2997, generated-bundle:22, generated-runtime-archive:1, secret-token-cache:1, signed-url-cache:15
runtime_leak_scan_findings = 0
runtime_schema_scan_findings = 0
runtime_integrity_scan_findings = 0

router_offline_tests = pass
router_syntax_checks = 41
router_behavior_tests = 4
router_test_count = 45
git_diff_check = pass
git_delta_diff_check = pass
python_py_compile_write_delta_manifest = pass
bash_n_observe_validation = pass
delta_manifest_bundle_verify_gate = implemented
delta_manifest_expected_head_gate = implemented
runtime_unique_alias_reporting = implemented
cargo_available = false
rustc_available = false
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
configured_rustc_wrapper_path_exists = false
state_graph_present = false
validation_status = partial
missing_signal_count = 12
```

Positive evidence:

- `GOAL.md` states a coherent target: frozen deterministic kernel, append-only
  TLog, bounded recovery, layered capabilities, LLM promotion, policy learning,
  and replayable/auditable decisions.
- The Rust crate is intentionally low-supply-chain: root `Cargo.toml` contains
  no third-party Rust dependencies.
- Static inspection found `#![forbid(unsafe_code)]` at the library boundary and
  `0` unsafe tokens in `src` plus `examples`.
- The source tree contains meaningful test intent: `103` Rust `#[test]`
  attributes and `2` `cfg(test)` sections.
- `scripts/observe_validation.sh` emits machine-readable records for git state,
  repo metrics, toolchain state, graph state, runtime archive metrics, validation
  commands, and missing validation flags.
- `scripts/write_delta_manifest.py` now verifies the generated bundle itself and
  refuses manifests when the bundle does not expose the expected committed head.
- Runtime archive reporting now exposes unique accepted download aliases, so
  repeated candidate aliases are separated from accepted lineage evidence.
- Router-side offline validation passed: `41` syntax checks plus `4` behavior
  tests, total `45` reported checks.
- The uploaded runtime archive is sanitized and useful: `84` members, `35`
  redacted files, explicit exclusion of token/signed-url caches, and `0`
  leak/schema/integrity findings in the manifest.
- Runtime history is no longer absent: the archive contains candidate ledgers,
  download ledgers, message ledgers, network/process logs, delta-apply receipts,
  loop-stop receipts, and downloaded score/manifest/report artifacts.

Critical risks and constraints:

- Current root Rust validation is not reproducible in this environment because
  neither `cargo` nor `rustc` exists on `PATH`.
- `.cargo/config.toml` configures a rustc wrapper at
  `/mnt/data/canon-mini-agent-extracted/canon-mini-agent/prototype/canon-rustc-v3/target/debug/canon-rustc-v3`,
  but that path does not exist here. Wrapper override is required before normal
  Cargo validation can run.
- `GOAL.md` claims a prior local run with `cargo build` passing, `103` tests
  passing, Ollama judgment passing, graph intent coverage, and semantic pathway
  telemetry. Those are historical claims, not reproduced evidence in this
  restored environment.
- No `state/rustc/*/graph.json` exists, so graph node counts, edge counts,
  intent coverage, redundant path-pair trends, and wrapper telemetry are absent.
- Runtime archive logs improve traceability but do not prove current source
  correctness. They show prior ChatGPT/runtime activity and artifact handling,
  not a fresh Rust build/test/proof replay.
- The runtime archive has `0` `.conversation.json` snapshots. It has
  `.messages.ndjson` ledgers, but no full conversation JSON snapshots.
- Download history has one stale advisory caused by a base-commit mismatch.
- Candidate history has repeated artifact aliases: `repo-delta-004.bundle` appears
  `25` times and `DELTA_MANIFEST.md` appears `14` times. Accepted-lineage
  reporting is now collapsed, but the underlying runtime history remains noisy.
- Recent git history includes tracked runtime/upload/download artifacts: staged
  project history contains `.repo-agent-runtime/upload-bundles/ai.bundle`,
  apply-worktree markers, `downloads/DELTA_MANIFEST.md`, and
  `downloads/repo-delta-004.bundle`. This weakens simplicity and repository
  hygiene.
- `316` `unwrap()` calls remain in `src` and `examples`. That is a large implicit
  panic surface for a system claiming bounded deterministic recovery.
- Ollama judgment was skipped because `CANON_OLLAMA_BASE_URL` and
  `CANON_OLLAMA_MODEL` are unset and no local endpoint was validated.
- Policy learning exists as source surface, but this observation did not find a
  replay trace proving promotion from completed TLog history.

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
missing_runtime_download_history = false
missing_external_observation_stream_test = true
missing_external_api_action_test = true
missing_semantic_artifact_verification_test = true
missing_policy_learning_replay_trace = true
missing_artifact_apply_worktree = false
```

## Dimension Rationale

| Dimension | Score | Evidence |
|---|---:|---|
| I | 7.4 | Strong architecture: frozen kernel, capabilities, TLog, verification, policy learning, and LLM promotion surfaces. |
| E | 6.4 | Observe script compresses evidence and now separates unique accepted aliases; missing toolchain/wrapper still slows evaluation. |
| C | 5.9 | Router tests and manifest hardening pass, but root Rust build/test/clippy/fmt were unavailable. |
| A | 7.7 | Direction matches `GOAL.md`; evidence still does not prove live autonomous operation. |
| R | 6.4 | Bundle/head verification reduces artifact risk; missing graph/toolchain/Ollama/replay proof keeps robustness moderate. |
| P | 5.5 | No root runtime latency, throughput, benchmark, or cargo timing signal was reproduced. |
| S | 5.9 | Layered design can scale conceptually; current validation is single-repo and partially unavailable. |
| D | 8.0 | Git head, archive base binding, bundle-head checks, redaction scans, and validation records are strong; graph/proof replay is absent. |
| T | 8.4 | Missing signals, bundle verification, and runtime-history classifications are explicit and machine-readable. |
| Co | 6.9 | README/GOAL/score are useful; accepted-alias reporting improves navigation, but history clutter remains. |
| Em | 7.5 | Restore/observe/delta commands are clearer and safer; external Rust/Ollama/wrapper setup is still required. |
| B | 7.1 | Useful autonomous-runtime prototype; benefit is capped until validation is portable and reproducible. |
| L | 6.0 | Runtime ledgers show activity and feedback surfaces; no reproduced policy-promotion replay trace. |
| Si | 5.9 | Manifest validation and alias reporting are simpler; history clutter, duplicate artifacts, and many unwraps remain. |
| F | 7.3 | Safer delta artifacts improve forward compatibility; missing portable toolchain/graph proof weakens future confidence. |

## Required Next Work

```text
1. Provide or restore a reproducible Rust toolchain in this environment.
2. Run cargo fmt/test/clippy with wrapper variables cleared.
3. Restore/build canon-rustc-v3 and require state/rustc/*/graph.json output.
4. Run ollama_judgment against a local endpoint and verify receipt/proof replay.
5. Add offline semantic-artifact verification and policy-learning replay tests.
6. Convert high-risk unwrap/expect sites into typed errors or classify them as deterministic-safe.
7. Stop tracking generated runtime/upload/download artifacts in normal repo history.
8. Add full conversation snapshot capture or explicitly justify `.messages.ndjson` as the canonical substitute.
9. Stop generating repeated candidate aliases, not merely reporting accepted aliases separately.
```