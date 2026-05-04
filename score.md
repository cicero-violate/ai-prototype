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
G = 7.03 / 10
max(G) = good
```

Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers
the whole system.

## Score Summary

```text
I  = 7.7 / 10
E  = 6.7 / 10
C  = 6.3 / 10
A  = 7.9 / 10
R  = 6.9 / 10
P  = 5.5 / 10
S  = 6.4 / 10
D  = 8.0 / 10
T  = 9.0 / 10
Co = 7.0 / 10
Em = 7.6 / 10
B  = 7.1 / 10
L  = 6.4 / 10
Si = 6.2 / 10
F  = 7.6 / 10

G = 7.03 / 10
GOOD = max(G) = 7.03 / 10 = good
```

Judgment: this is a serious deterministic autonomous-agent prototype with a
clear kernel/capability split, typed receipts, validation reporting, runtime
archive hygiene, and policy-learning direction. It is still below production
quality because the root Rust crate, graph telemetry, local LLM proof replay,
semantic artifact verification, and policy-promotion replay were not reproduced
in this evaluation environment. This turn reduced the production panic surface
to zero by static validation, but full Rust validation remains blocked.

## Evidence Snapshot

```text
bundle = /mnt/data/ai.bundle
restored_repo = /mnt/data/ai-eval-work/ai
base_head = cf2f814e32f9b30966fbbd0c71f17bb52106e556
branch = main
git_status_at_execute_validation = implementation files modified before commit
goal_md_present = true
cargo_toml_package = ai 0.1.0, edition 2024
root_dependencies = 0 third-party Rust dependencies
source_surfaces = kernel, codec, runtime, api, capability
rust_files_src_examples = 53
rust_test_attrs_static_scan = 103
rust_cfg_test_sections_static_scan = 2
unsafe_token_count_src_examples = 0
panic_call_count_src_examples = 0
unwrap_call_count_src_examples = 316
expect_call_count_src_examples = 3
panic_surface_production_unwrap_count = 0
panic_surface_production_expect_count = 0
panic_surface_production_panic_count = 0
panic_surface_test_total = 319
panic_surface_example_total = 0
tracked_file_count = 192
state_graph_present = false
runtime_archive_present = true
runtime_archive_sha256 = 2f98ab70d1a72df9f4447a5fe2c86b2965a762f8b00be10303640c975f87a0d3
runtime_manifest_base_commit = cf2f814e32f9b30966fbbd0c71f17bb52106e556
runtime_manifest_included_files = 103
runtime_manifest_excluded_files = 3604
runtime_manifest_excluded_reasons = apply-worktree:3555, generated-bundle:28, signed-url-cache:19, secret-token-cache:1, generated-runtime-archive:1
runtime_manifest_scans = leak:0, schema:0, integrity:0 findings
runtime_archive_members_observe_scan = 104
runtime_archive_log_total_observe_scan = 12316
runtime_archive_download_total_observe_scan = 230
runtime_download_history_records = 184
runtime_download_history_by_classification = delta_applied:17, download_event:26, download_ledger:124, live_evidence:16, stale_advisory:1
runtime_candidate_error_count = 0
runtime_duplicate_artifact_aliases = 8
runtime_conversation_snapshots = 0
observe_validation_status = partial
observe_validation_command_count = 10
observe_validation_test_count = 55
python_unit_tests = pass, 10 tests
panic_surface_validation = pass, production_total 0
router_offline_tests = pass, 45 tests
cargo_available = false
rustc_available = false
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
wrapper_graph_validation = skipped_env_missing
ollama_judgment_example = skipped_env_missing
missing_signal_count = 13
```

## Positive Evidence

- `GOAL.md` defines a coherent target: frozen deterministic kernel, strict
  codec/runtime/capability layering, append-only TLog, bounded recovery,
  policy-learning, and LLM promotion from routine worker to novelty specialist.
- `src/lib.rs` exposes a broad typed surface for API commands, capability
  records, runtime replay, durable execution, LLM receipts, tooling receipts,
  semantic verification, learning, policy, and observation ingress.
- `src/lib.rs` and `src/main.rs` both use `#![forbid(unsafe_code)]`; static scan
  found `0` unsafe tokens and `0` `panic!` calls in `src` and `examples`.
- `scripts/validate_rust_panic_surface.py --fail-production-unwrap` reports
  `production_total=0`, separating test fixture unwrap/expect calls from
  production runtime risk.
- Root `Cargo.toml` has no third-party Rust dependencies, which improves
  auditability and reduces supply-chain risk.
- Static test surface is substantial: `103` Rust `#[test]` attributes plus `10`
  Python tests and `45` router offline tests reported by observe validation.
- `scripts/observe_validation.sh` emits separate statuses for Git hygiene,
  Python tests, router tests, root Rust checks, wrapper graph validation, graph
  presence, Ollama example execution, runtime archive evidence, and missing
  signals.
- Current validation reports `git_diff_check=pass`, `git_delta_diff_check=pass`,
  `python_unit_tests=pass`, `panic_surface_validation=pass`, and
  `router_offline_tests=pass`.
- Runtime archive evidence is bound to the restored head:
  `runtime_manifest_base_commit=cf2f814e32f9b30966fbbd0c71f17bb52106e556`.
- Runtime archive hygiene is strong: manifest leak, schema, and integrity scans
  each report `0` findings, and token/signed URL caches are explicitly excluded.
- Runtime archive has useful execution lineage: `184` download-history records,
  `17` delta-applied records, `16` live-evidence records, `12316` log rows from
  observe scan, and `0` candidate errors.

## Critical Risks

- Validation status is `partial`, not `pass`.
- `cargo` and `rustc` are unavailable in this environment, so root Rust
  `fmt`, `test --all-targets`, and `clippy -D warnings` were not reproduced.
- Wrapper graph capture was not reproduced: `CANON_RUSTC_WRAPPER` was not
  provided and `state/rustc/*/graph.json` is absent.
- Graph node/edge counts, intent coverage, redundant-path pairs, alpha pathways,
  and wrapper semantic telemetry are therefore missing from current evidence.
- `cargo run --example ollama_judgment` was skipped because the local Ollama env
  was absent; receipt/proof replay and tamper rejection remain historical claims
  in this stage, not freshly reproduced evidence.
- `316` `unwrap()` calls and `3` `expect()` calls remain in `src` and
  `examples`; static validation classifies all of them as test-only, but the
  large fixture surface still adds review noise.
- Runtime archive has `0` full `.conversation.json` snapshots and `8` duplicate
  artifact aliases, so lineage is useful but still noisy.
- No external observation stream test, external API action test, semantic
  artifact verification test, or policy-learning replay trace was reproduced.
- No benchmark, latency budget, throughput metric, memory profile, or sustained
  concurrent orchestration validation was reproduced.
- Repository hygiene remains noisy: generated/download/patch artifacts are part
  of the restored project context and make collaborator review harder.

## Dimension Rationale

| Dimension | Score | Evidence |
|---|---:|---|
| I | 7.7 | Strong typed architecture across kernel, runtime, capabilities, receipts, verification, learning, policy, observation, and LLM surfaces. |
| E | 6.7 | Observe script gives compact evidence and Python/router/panic-surface checks pass; root Rust validation is blocked by missing toolchain. |
| C | 6.3 | Git checks, Python tests, router tests, and production panic-surface validation pass; Rust tests, graph replay, clippy, fmt, and Ollama proof replay are unavailable. |
| A | 7.9 | Source layout and exports align closely with `GOAL.md`; autonomous live operation remains unproven. |
| R | 6.9 | No unsafe code, panic macros, or production unwrap/expect paths are detected; missing proof/graph/Rust runs still cap robustness. |
| P | 5.5 | No fresh benchmark, timing budget, cargo timing, LLM latency, throughput, or memory evidence. |
| S | 6.4 | Layered capability model and zero Rust dependencies help scale; validation remains single-repo and toolchain/environment-sensitive. |
| D | 8.0 | Frozen-kernel intent, hash/log/replay surfaces, deterministic reports, and runtime archive binding are strong; absent graph/proof replay prevents higher score. |
| T | 9.0 | Missing signals, archive hygiene, command outcomes, panic-surface buckets, and validation state are explicit and machine-readable. |
| Co | 7.0 | README, GOAL, score, plans, scripts, manifests, and validation reports help collaborators; generated artifact noise still slows review. |
| Em | 7.6 | Restore/observe/test/delta instructions empower handoff; missing local toolchain blocks complete independent validation. |
| B | 7.1 | The project targets high-value autonomous correctness and auditability; realized benefit is still prototype-level. |
| L | 6.4 | Learning/policy surfaces exist and the goal is clear; no fresh policy-promotion replay trace was reproduced. |
| Si | 6.2 | Kernel/capability separation is conceptually simple and production panic paths are reduced; broad exports, receipts, patches, and test fixtures still add complexity. |
| F | 7.6 | Canonical effect/proof direction and deterministic boundaries are future-proof; current missing graph/Ollama/semantic validation prevents stronger confidence. |

## Missing Validation Signals

```text
missing_root_rust_toolchain = true
missing_cargo_fmt = true
missing_cargo_test = true
missing_clippy = true
missing_wrapper_graph_validation = true
missing_generated_graph_json = true
missing_rustc_wrapper_telemetry = true
missing_cargo_run_ollama_judgment = true
missing_semantic_artifact_verification_test = true
missing_policy_learning_replay_trace = true
missing_external_observation_stream_test = true
missing_external_api_action_test = true
missing_conversation_snapshot = true
missing_panic_surface_validation = false
```

## Required Next Work

1. Restore a Rust toolchain and run:
   `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check`,
   `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings`.
2. Run wrapper graph validation with `CANON_RUSTC_WRAPPER` and require
   `state/rustc/*/graph.json` evidence in the observe report.
3. Run `cargo run --example ollama_judgment` against a local endpoint and record
   receipt/proof/tamper-rejection evidence.
4. Add or reproduce policy-learning replay, semantic artifact verification,
   external observation, and external API action validation.
5. Keep production panic-surface validation enforced and reduce test fixture
   unwrap noise only where it improves review clarity.
6. Add performance evidence: command timings, LLM latency, throughput, memory,
   and bounded concurrent orchestration tests.