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

One-line explanation: Goodness is the geometric mean; one weak dimension lowers
the whole system.

## Score Summary

```text
I  = 7.6 / 10
E  = 6.8 / 10
C  = 6.5 / 10
A  = 7.8 / 10
R  = 6.6 / 10
P  = 6.0 / 10
S  = 6.2 / 10
D  = 7.9 / 10
T  = 8.3 / 10
Co = 7.0 / 10
Em = 7.6 / 10
B  = 7.5 / 10
L  = 6.3 / 10
Si = 6.1 / 10
F  = 7.3 / 10

G = 7.00 / 10
GOOD = max(I,E,C,A,R,P,S,D,T,Co,Em,B,L,Si,F) = T = 8.3 / 10 = good
```

Judgment: the architecture is strong, but proof remains partial. This turn
improves validation closure and reduces artifact verbosity; it does not prove
Rust, graph telemetry, Ollama, semantic verification, or policy learning.

## Evidence

```text
base_commit = 181519a9fd945532cc6de825c71d197dd01e65ec
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_archive_sha256 = 2ccdf19e623a6be95924aac7ad0b2d696923ef96f42a895917ec73693f5c868a
runtime_manifest_base_commit = 181519a9fd945532cc6de825c71d197dd01e65ec
runtime_archive_members = 71
runtime_archive_log_files = 32
runtime_archive_log_total = 8416
runtime_archive_download_files = 15
runtime_archive_download_total = 133
runtime_archive_conversation_snapshots = 0
runtime_archive_cache_files = 0
runtime_candidate_error_count = 0
runtime_duplicate_artifact_aliases = 6
runtime_stale_advisory_count = 1
tracked_files = 189
rust_files_src_examples = 53
rust_test_attrs = 103
rust_cfg_test_sections = 2
unsafe_token_count_src_examples = 0
unwrap_calls_src_examples = 316
expect_calls_src_examples = 9
router_offline_tests = pass
router_test_count = 45
cargo_available = false
rustc_available = false
state_graph_present = false
configured_rustc_wrapper_present_here = false
```

Positive evidence:

- `GOAL.md` states a coherent frozen-kernel, append-only TLog, bounded recovery,
  policy-learning, and LLM-promotion architecture.
- `src/lib.rs` and `src/main.rs` forbid unsafe code.
- The root Rust crate has no third-party Rust dependencies.
- Router offline validation passes and reports `45` tests.
- Runtime archive inspection now records compact totals instead of noisy per-file
  maps while preserving log/download/cache evidence.
- Manifest generation now rejects missing `validation_summary` rows, stale report
  heads, and missing bundle paths.

Risks and constraints:

- Root Rust validation is unavailable in this sandbox because `cargo` and `rustc`
  are not on `PATH`.
- The configured rustc wrapper path does not exist here, so wrapper telemetry and
  `state/rustc/*/graph.json` are absent.
- Ollama judgment is skipped unless `CANON_OLLAMA_BASE_URL` and
  `CANON_OLLAMA_MODEL` are set and a local endpoint is reachable.
- Runtime evidence has `6` duplicate artifact aliases and `1` stale advisory.
- `316` unclassified `unwrap()` calls remain in `src` and `examples`.
- No conversation snapshot is present in the runtime archive.

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
| I | 7.6 | Strong deterministic-agent architecture and capability separation. |
| E | 6.8 | Observe output is smaller; runtime/artifact churn still costs attention. |
| C | 6.5 | Router tests pass; root Rust proof is unavailable. |
| A | 7.8 | Implementation direction matches `GOAL.md`; live autonomy is not reproduced. |
| R | 6.6 | Missing toolchain, wrapper, graph, and Ollama signals are explicit. |
| P | 6.0 | No current throughput or latency benchmark for the root system. |
| S | 6.2 | Layering supports scale; multi-run/load validation is absent. |
| D | 7.9 | Base/head/report/bundle checks are stricter. |
| T | 8.3 | Evidence is compact and missing signals are directly named. |
| Co | 7.0 | README, plan, and manifest flow are clearer. |
| Em | 7.6 | Receiver gets deterministic bundle and manifest commands. |
| B | 7.5 | Useful prototype; validation gaps limit operational trust. |
| L | 6.3 | Learning surfaces exist, but replay/promotion is not validated here. |
| Si | 6.1 | Documentation and observe output are shorter; codebase still has artifacts. |
| F | 7.3 | Portable validation and manifest rejection improve forward compatibility. |

## Required Next Work

```text
1. Run cargo fmt/test/clippy with a real Rust toolchain.
2. Restore/build the rustc wrapper and require graph telemetry.
3. Run ollama_judgment and verify receipt/proof replay.
4. Add offline semantic-artifact and policy-learning replay tests.
5. Reduce retained patch/bundle/runtime artifact noise in normal history.
```