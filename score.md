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
max(G) = good
```

One-line explanation: Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers the whole system.

## Score Summary

```text
I  = 7.6 / 10
E  = 6.5 / 10
C  = 6.4 / 10
A  = 7.8 / 10
R  = 6.4 / 10
P  = 6.0 / 10
S  = 6.2 / 10
D  = 7.7 / 10
T  = 8.2 / 10
Co = 6.8 / 10
Em = 7.4 / 10
B  = 7.5 / 10
L  = 6.3 / 10
Si = 5.7 / 10
F  = 7.1 / 10

G = 6.87 / 10
GOOD = max(I,E,C,A,R,P,S,D,T,Co,Em,B,L,Si,F) = T = 8.2 / 10 = good
```

Judgment: architecture is strong; current-head proof is still partial because Rust, graph, Ollama, and semantic replay signals are unavailable in this sandbox.

## Current Scope

```text
base_commit = fae4c60c6fae6177f92837119930e412c9d02e65
restored_bundle = /mnt/data/ai.bundle
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_archive_sha256 = 6aa85edd54e97fdd62f7484fda96bd2821ac96445ebf6a28f4a6ffc682268922
implemented_focus = portable current-head validation closure
source_semantics_changed = false
validation_artifacts_committed = false
```

## Evidence

### Repository

```text
tracked_files = 189
rust_files_src_examples = 53
rust_test_attrs = 103
rust_cfg_test_sections = 2
rust_dependencies = 0
unsafe_token_count_src_examples = 0
unwrap_calls_src_examples = 316
expect_calls_src_examples = 9
configured_rustc_wrapper = canon-rustc-v3/target/debug/canon-rustc-v2
configured_rustc_wrapper_present_here = false
generated_state_graph_present = false
```

Positive: `src/lib.rs` and `src/main.rs` forbid unsafe code, the root crate has no third-party Rust dependencies, and the source tree separates kernel, codec, runtime, API, and capability layers.

Risk: many `unwrap()` calls remain unclassified and graph telemetry is absent until the vendored rustc wrapper is built and used.

### Git History

```text
HEAD_at_eval = fae4c60 starting agent run
parent = 80ace85 Add portable router validation contracts
recent_receipt_work = 4b4cf03 Close validation receipt surface
recent_fallback_work = 60364d4 Add portable router validation fallback
artifact_churn_present = true
embedded_uploaded_bundle = .repo-agent-runtime/upload-bundles/ai.bundle
```

Risk: recent commits improve validation and artifact lineage, but generated bundles/manifests and retained runtime artifacts add collaboration noise.

### Runtime Archive

```text
runtime_archive_parse_status = pass
runtime_manifest_base_commit = fae4c60c6fae6177f92837119930e412c9d02e65
runtime_archive_member_count = 53
runtime_archive_log_total = 6259
runtime_archive_download_total = 69
runtime_stale_advisory_count = 1
runtime_duplicate_artifact_aliases = 6
runtime_archive_conversation_snapshots = 0
runtime_archive_cache_files = 0
excluded_runtime_files_include_secret_token_cache = true
excluded_runtime_files_include_signed_url_cache = true
```

Positive: the archive preserves useful logs, download ledgers, candidate ledgers, and apply evidence while excluding token and signed URL caches.

Risk: duplicate artifact aliases and one stale advisory show automation hygiene debt. No conversation snapshot is present.

### Validation Metadata

`scripts/observe_validation.sh` now records one NDJSON stream covering working-tree diff hygiene, base-to-head diff hygiene, router tests, cargo availability, wrapper override state, runtime evidence, graph absence, Ollama skip reason, and missing-signal flags.

Pre-commit validation of the updated observe command produced:

```text
validation_status = partial
validation_command_count = 7
validation_test_count = 45
failed_required_commands = []
git_diff_check = pass
git_delta_diff_check = pass
router_offline_tests = pass
router_test_count = 45
cargo_available = false
rustc_available = false
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
runtime_archive_download_total = 69
runtime_archive_log_total = 6259
```

`scripts/write_delta_manifest.py` now rejects stale validation reports when the report head does not match the manifest head.

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

## Dimension Rationale

| Dimension | Score | Evidence |
|---|---:|---|
| I | 7.6 | Clear deterministic-agent architecture and capability layering. |
| E | 6.5 | Validation closure reduces manual inspection; artifact/runtime noise remains. |
| C | 6.4 | Router checks pass; root Rust validation unavailable. |
| A | 7.8 | Strong match to `GOAL.md`; live autonomy not reproduced here. |
| R | 6.4 | Missing toolchain/wrapper are detected and recorded instead of hidden. |
| P | 6.0 | No current performance benchmark beyond runtime loop metadata. |
| S | 6.2 | Layering supports scale; no current multi-run/load validation. |
| D | 7.7 | Base/head/report/bundle receipt checks improved. |
| T | 8.2 | Evidence and missing signals are explicit. |
| Co | 6.8 | README and manifest workflow are clearer; generated artifacts still noisy. |
| Em | 7.4 | Receiver gets exact apply command and validation manifest. |
| B | 7.5 | Useful deterministic-agent prototype with concrete validation scaffolding. |
| L | 6.3 | Policy-learning surfaces exist, but replay/promotion not validated here. |
| Si | 5.7 | Documentation was compressed; codebase still carries many patch/artifact files. |
| F | 7.1 | Portable validation fallback improves forward compatibility. |

## Required Next Work

```text
1. Run cargo fmt/test/clippy with a real Rust toolchain and wrapper override.
2. Restore or build canon-rustc wrapper and require state/rustc/*/graph.json.
3. Run ollama_judgment against a local endpoint and verify receipt/tamper output.
4. Add one offline replay test for semantic artifact verification and policy-learning promotion.
5. Reduce artifact churn: keep runtime evidence, but exclude stale generated bundles from normal repo history.
```
