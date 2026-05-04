# Canon Agent Scorecard

## Variables

```text
K = kernel determinism
C = codec / TLog durability
V = replay / verification proof
P = capability + policy maturity
B = build / test reproducibility
E = runtime archive evidence
N = nested router-server evidence
D = documentation / delta discipline
S = overall score
GOOD = strongest current axis
```

## Equations

```text
S = (K · C · V · P · B · E · N · D)^(1/8)
GOOD = max(K,C,V,P,B,E,N,D)
```

One-line explanation: the score is bounded by the weakest executable proof surface, not by architecture claims.

## Score Summary

```text
K = 8.4 / 10
C = 7.8 / 10
V = 8.0 / 10
P = 6.1 / 10
B = 3.9 / 10
E = 7.1 / 10
N = 7.1 / 10
D = 7.5 / 10

S = 6.82 / 10
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10 = good
```

## Scope

```text
stage = EXECUTE turn 1
base_commit = da1ebc2656e5d48631b07ef830c7dbec2033739b
restored_repo_path = /mnt/data/work-ai-eval
uploaded_bundle = /mnt/data/ai.bundle
source_runtime_changed = false
changed_surfaces = README.md, score.md, scripts/write_delta_manifest.py
```

Judgment: the repository is a serious deterministic-runtime prototype. It is not yet a reproducibly validated autonomous agent in this sandbox.

## GOAL.md Alignment

`GOAL.md` targets a frozen deterministic kernel, append-only TLog, typed capability layer, bounded recovery, policy learning, semantic verification, and LLM promotion.

Observed topology matches that target:

```text
src/kernel
src/codec
src/runtime
src/api
src/capability/{context,eval,judgment,learning,llm,memory,observation,orchestration,planning,policy,tooling,verification}
```

Static evidence:

```text
forbid_unsafe_code = src/lib.rs + src/main.rs
dependency_count = 0
rust_files_src_examples = 53
rust_loc_src_examples = 15825
rust_tests_declared = 103
public_items = 618
unwrap_calls = 316
expect_calls = 9
unsafe_code_sites = 0
```

Constraint: `GOAL.md` describes Rust, graph, Ollama, and replay claims that are not reproducible here without `cargo`, `rustc`, the configured wrapper, and local Ollama.

## Git Evidence

Restored HEAD before this execute stage:

```text
da1ebc2 Apply agent worktree delta
b57d8fa ready for agent run
d47aaa7 ready for agent run
ab4cd5b starting agent run
8fe6fec Fix router validation wrapper
d0041c2 Plan router validation wrapper fix
107eb86 Observe ai repository evidence
2f261eb Evaluate ai repository scorecard
```

The final delta must be cumulative from `da1ebc2656e5d48631b07ef830c7dbec2033739b..HEAD` and must be applied with:

```bash
git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD
```

## Implemented Improvement

Highest-impact implementable target from `IMPLEMENTATION_PLAN.md`:

```text
target = root operator reproducibility + current-head delta evidence
reason = closes stale/manual manifest risk without claiming unavailable Rust proof
```

Changes made:

```text
README.md = concise restore, validate, Rust, Ollama, and delta-artifact workflow
scripts/write_delta_manifest.py = schema-v2 receipt, ancestor check, empty-delta rejection, exact receiver command
score.md = shorter critical evidence ledger
```

Why this improves score:

```text
V: manifest receipt now rejects invalid base/head and empty deltas
B: operator can run one documented available-validation path
E: runtime archive evidence remains captured but no longer overclaimed
D: README + manifest schema align restore, validation, and receiver apply commands
```

## Validation Evidence

Available validation path:

```bash
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz bash scripts/observe_validation.sh
python3 -m py_compile scripts/write_delta_manifest.py
git diff --check
git bundle verify /mnt/data/repo-delta-004.bundle
python3 scripts/write_delta_manifest.py --base B --head H --report target/observe/validation-report.ndjson --bundle /mnt/data/repo-delta-004.bundle --bundle-verify pass --out /mnt/data/DELTA_MANIFEST.md --receipt-out target/observe/delta-validation-receipt.json
```

Current observe report signals:

```text
validation_status = partial
validation_command_count = 6
validation_test_count = 15
router_test_count = 15
git_diff_check = pass
router_offline_tests = pass
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
missing_signal_count = 11
```

The validation status must stay `partial` until root Rust, wrapper graph telemetry, Ollama, policy learning, semantic artifact verification, and live CDP signals run.

## Uploaded Runtime Archive Evidence

Primary archive:

```text
archive = /mnt/data/ai-runtime.tar.gz
files = 27
manifest_baseCommit = da1ebc2656e5d48631b07ef830c7dbec2033739b
manifest_repo = /workspace/ai_sandbox/canon-mini-agent/prototype/ai
manifest_included = 26
manifest_excluded = 1032
excluded_apply_worktree = 1018
excluded_generated_bundle = 8
excluded_secret_token_cache = 1
excluded_signed_url_cache = 4
integrity_findings = 0
leak_findings = 0
```

Observe-derived archive counters:

```text
runtime_archive_download_total = 24
runtime_archive_log_total = 2637
runtime_archive_conversation_snapshots = 0
download_history_classification = stale_advisory
download_history_reason = base_commit_mismatch
```

Judgment: the archive proves the automation loop produced logs, downloads, and apply receipts. It does not prove the current root Rust crate builds.

## Nested Router-Server Evidence

```text
path = ai-chromium/router-server
syntax_check = node src/tools/check-syntax.mjs
syntax_files = 49
offline_tests = 15
run_tests_sh = pass
live_cdp_test = gated by RUN_LIVE_TESTS=1 or LIVE_ROUTER_URL
package_json_present = false
```

Judgment: offline router proof is useful and reproducible. Live browser/CDP behavior remains unproven here.

## Risk Register

| Risk | Severity | Evidence | Required closure |
|---|---:|---|---|
| Root Rust validation unavailable | High | `cargo` and `rustc` absent | Provide toolchain and rerun fmt/test/clippy |
| Configured wrapper missing | High | wrapper configured, path absent | Include/repoint wrapper or allow portable override |
| Graph telemetry absent | High | no `state/rustc/*/graph.json` | Regenerate graph and record node/edge/intent metrics |
| Live Ollama unproven | Medium | env missing and cargo absent | Run example and verify receipt |
| Policy learning trace missing | High | no observation→eval→learning replay | Add one current-head integration trace |
| Semantic artifact verification missing | High | no external artifact trace | Bind artifact checks to receipts |
| Live CDP unproven | Medium | live test gated off | Run against reachable browser/router |
| Runtime history stale-advisory | Medium | base mismatch in downloaded manifest history | Bind downloads to current base/head |
| Production unwrap surface unclassified | Medium | 316 unwrap + 9 expect calls | Classify test-only vs runtime path |

## Module Scorecard

| Area | Score | Evidence | Constraint |
|---|---:|---|---|
| Kernel | 8.4 | frozen-layer design, typed gates, no unsafe code sites | no formal proof artifact |
| Codec / TLog | 7.8 | NDJSON codec and durable replay APIs | root tests unavailable |
| Replay / verification | 8.0 | proof APIs plus stricter delta manifest receipt | Rust execution not reproduced |
| Capability layer | 6.1 | named capability modules present | full objective loop not proven |
| Build/test reproducibility | 3.9 | observe harness + router offline tests | root crate cannot build here |
| Runtime archive evidence | 7.1 | 27 files, 2637 log lines, 24 download records | no conversation snapshots; stale advisory downloads |
| Nested router-server | 7.1 | 49 syntax files and 15 offline tests pass | live CDP missing |
| Documentation alignment | 7.5 | README, GOAL, plan, score, and manifest now align | GOAL still exceeds executable proof |

## Missing Validation Signals

```text
required_before_higher_score = [
  cargo_fmt_check,
  cargo_test_all_targets,
  cargo_clippy_all_targets,
  root_binary_run,
  ollama_judgment_example_run,
  state_rustc_graph_json,
  rustc_wrapper_telemetry,
  policy_learning_replay_trace,
  external_observation_stream_test,
  external_api_action_test,
  semantic_artifact_verification_trace,
  live_cdp_router_test
]
```

## Verdict

```text
classification = serious_deterministic_runtime_prototype
not_yet = reproducibly_validated_autonomous_agent
main_blocker = missing_root_rust_validation + missing_wrapper + absent_graph_telemetry + missing_live_runtime_replay
score_confidence = medium_static_low_runtime
```