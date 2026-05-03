# Canon Agent Scorecard

## Variables

```text
K  = kernel determinism score
C  = codec / durable format score
A  = api command-surface score
R  = runtime / replay score
OB = observation capability score
CX = context capability score
ME = memory capability score
PL = planning capability score
LL = llm capability score
JG = judgment capability score
TO = tooling capability score
VF = verification capability score
EV = eval capability score
PO = policy capability score
LE = learning capability score
OR = orchestration capability score

CORE = deterministic foundation score
CAP  = implemented capability-layer score
ARCH = declared GOAL.md alignment score
GOOD = strongest present module
```

## Equations

```text
CORE = (K · C · A · R)^(1/4)
CAP  = (OB · CX · ME · PL · LL · JG · TO · VF · EV · PO · LE · OR)^(1/12)
ARCH = (K · C · A · R · OB · CX · ME · PL · LL · JG · TO · VF · EV · PO · LE · OR)^(1/16)
GOOD = max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR)
```

One-line explanation: the repo has a strong deterministic/replay core, but the observed evidence still shows a local prototype rather than a validated autonomous system.

## Score Summary

```text
K  = 8.4 / 10
C  = 7.7 / 10
A  = 7.0 / 10
R  = 8.4 / 10

OB = 6.0 / 10
CX = 5.1 / 10
ME = 5.4 / 10
PL = 5.3 / 10
LL = 7.6 / 10
JG = 5.4 / 10
TO = 8.0 / 10
VF = 8.4 / 10
EV = 6.4 / 10
PO = 7.3 / 10
LE = 6.1 / 10
OR = 6.2 / 10

CORE = 7.85 / 10
CAP  = 6.35 / 10
ARCH = 6.69 / 10
GOOD = VF = 8.4 / 10
max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR) = VF = 8.4 / 10 = good
```

## Observed Evidence

```text
base_commit = dc1f3f8227ed7f67f6adbf728315160cdd71c920
restored_head = dc1f3f8227ed7f67f6adbf728315160cdd71c920
bundle_verify = passed_inside_restored_repo
bundle_history = complete_history_sha1_bundle
recent_history = dc1f3f8 ready_for_agent_run; 919fd87 auto; 7cd4798 auto; b80bf42/eeb6d95/446f965/698f0eb uploading_to_chatgpt_projects

goal_md_present = yes
readme_present = no
cargo_toml_present = yes
cargo_edition = 2024
cargo_dependencies = 0
src_rust_files = 52
rust_files_reviewed = 53
inline_test_markers = 103
state_graph_present = no
panic_todo_unimplemented_count = 0
unwrap_call_count = 316

local_rust_toolchain = unavailable_in_this_container
observe_validation_script = scripts/observe_validation.sh
observe_validation_report = target/observe/validation-report.ndjson
observe_validation_ndjson_lines = 10
cargo_fmt_check = unavailable_cargo_missing_recorded_by_harness
cargo_test_all_targets = unavailable_cargo_missing_recorded_by_harness
cargo_clippy_all_targets = unavailable_cargo_missing_recorded_by_harness
ollama_judgment_example = skipped_env_missing_recorded_by_harness
cargo_wrapper_config = points_to_/workspace/ai_sandbox/canon-rustc-v2/target/debug/canon-rustc-v2
wrapper_path_in_this_container = unavailable
validation_executed_here = observe_validation_harness_passed_ndjson_parse + git_diff_check_passed + cargo_fmt_check_unavailable + cargo_test_unavailable
```

## Runtime Archive Evidence

```text
ai_runtime_archive = present
ai_runtime_base_commit = dc1f3f8227ed7f67f6adbf728315160cdd71c920
ai_runtime_included_files = .repo-agent-runtime/audit.ndjson + log/chatgpt_project_agent.ndjson
ai_runtime_audit_lines = 1
ai_runtime_process_log_lines = 1
ai_runtime_download_history_count = 0
ai_runtime_download_history_by_classification = {}
ai_runtime_excluded_generated_bundle_count = 1
ai_runtime_excluded_generated_runtime_archive_count = 1
ai_runtime_leak_scan_findings = 0
ai_runtime_schema_scan_findings = 0
ai_runtime_integrity_scan_findings = 0

other_uploaded_runtime_archives = chatgpt-project-agent-runtime.tar.gz + router-server-runtime.tar.gz
other_runtime_download_history_count = 0_for_each_archive
```

The runtime archive proves only the current loop preflight was captured. It does not contain conversation snapshots, downloaded artifact histories, build logs, cargo logs, graph captures, or prior apply worktrees. That is useful for confirming sanitized upload hygiene, but weak as execution evidence.

## Critical Judgment

The `GOAL.md` target is coherent: a frozen deterministic kernel, replayable TLog, append-only policy growth, and capability-layer intelligence. The repository structure supports that target with explicit `kernel`, `codec`, `runtime`, `api`, and `capability/*` modules.

The strongest implemented surface is still verification/runtime discipline. The repo contains a replay-oriented runtime, command ledger, durable writer, transition table, semantic diff, verification proof module, NDJSON codec, policy store, local tooling records, observation records, and an Ollama/OpenAI-compatible LLM path. The 103 checked-in test markers show serious local validation intent.

The hard limitation is validation freshness. The new `scripts/observe_validation.sh` harness now makes that gap machine-readable, but in this container `rustc` and `cargo` are absent, so the current EXECUTE pass still cannot prove `cargo fmt`, `cargo test`, the Ollama example, graph regeneration, or wrapper-backed rustc telemetry. Prior claims in `GOAL.md` and the previous `score.md` may be true in the original workstation, but they are not reproduced here.

The second limitation is artifact evidence. No `README.md` exists even though older scoring text referenced one. No `state/rustc/ai/graph.json` exists in this restored bundle, so graph-node, graph-edge, redundant-path, alpha-pathway, and intent-coverage claims cannot be independently verified from the restored repository.

The third limitation is autonomous behavior. Observation remains represented as bounded/file/API-ingress machinery rather than live external stream ingestion. Tooling and LLM receipts are meaningful, but there is no observed end-to-end loop proving external perception, authenticated external action, semantic artifact verification, policy promotion from empirical history, and orchestration under load.

## Module Scorecard

| Module                     | Score | Evidence                                                                                                                                                                        | Risk                                                                                                                                 |
|----------------------------+-------+---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+--------------------------------------------------------------------------------------------------------------------------------------|
| `kernel`                   |   8.4 | Dedicated module; GOAL requires frozen deterministic reduce/hash/state; runtime tests exist in source.                                                                          | Formal proof layer and external invariant proof are not present in restored evidence.                                                |
| `codec`                    |   7.7 | `codec/ndjson.rs`; TLog/receipt encoding surfaces are exported from `lib.rs`.                                                                                                   | Schema migration and compatibility guarantees are not validated here.                                                                |
| `api`                      |   7.0 | `api/protocol.rs` and `api/routes.rs`; command-style route layer exists.                                                                                                        | No network service, authentication, or hostile-client validation observed.                                                           |
| `runtime`                  |   8.4 | `runtime/{verify,reducer,durable,writer,command_ledger,transition_table}.rs`; replay and durable-writing concerns are separated.                                                | Cargo replay tests could not be run here; graph telemetry absent.                                                                    |
| `capability/observation`   |   6.0 | Observation record/source modules exist; GOAL describes bounded ingress and API routing.                                                                                        | No live SSE/webhook/browser adapter evidence in the bundle.                                                                          |
| `capability/context`       |   5.1 | Context record module exists.                                                                                                                                                   | No retrieval, grounding, conflict handling, or token-budget evidence.                                                                |
| `capability/memory`        |   5.4 | Memory store module exists.                                                                                                                                                     | No embeddings, durable namespace model, invalidation, or cross-run retrieval proof.                                                  |
| `capability/planning`      |   5.3 | Planning record module exists.                                                                                                                                                  | No observed planner search, dependency solving, or risk/cost tradeoff engine.                                                        |
| `capability/llm`           |   7.6 | `capability/llm/ollama.rs`; example `ollama_judgment.rs`; GOAL documents local OpenAI-compatible path.                                                                          | No live Ollama run was possible here; streaming and provider-signed receipts absent.                                                 |
| `capability/judgment`      |   5.4 | Judgment record module exists and is wired by exported APIs.                                                                                                                    | Judgment appears record-based, not a validated comparative decision process.                                                         |
| `capability/tooling`       |   8.0 | Tooling record module and proof/receipt concepts are present in source and GOAL.                                                                                                | External API action tools and hostile execution validation are not observed.                                                         |
| `capability/verification`  |   8.4 | Verification proof module exists and is heavily represented in exports/tests.                                                                                                   | Semantic truth outside receipt/proof structure is still not proven.                                                                  |
| `capability/eval`          |   6.4 | Eval record module exists, and `scripts/observe_validation.sh` now emits a deterministic NDJSON evidence report with build/test/toolchain/graph/runtime/missing-signal records. | No calibrated benchmark suite, threshold governance, adversarial eval evidence, or successful Rust validation run in this container. |
| `capability/policy`        |   7.3 | Policy store module exists; GOAL emphasizes append-only policy.                                                                                                                 | Promotion governance is not validated by runtime archive evidence.                                                                   |
| `capability/learning`      |   6.1 | Learning promotion module exists.                                                                                                                                               | No empirical learning loop or policy compounding trace in runtime archive.                                                           |
| `capability/orchestration` |   6.2 | Orchestration record module exists.                                                                                                                                             | Distributed/parallel orchestration under load is not evidenced.                                                                      |

## Missing Validation Signals

```text
missing_cargo_fmt = true
missing_cargo_test = true
missing_cargo_run_ollama_judgment = true
missing_clippy = true
missing_generated_graph_json = true
missing_rustc_wrapper_telemetry = true
missing_runtime_download_history = true
missing_conversation_snapshot = true
missing_artifact_apply_worktree = true
missing_external_observation_stream_test = true
missing_external_api_action_test = true
missing_semantic_artifact_verification_test = true
missing_policy_learning_replay_trace = true
```

## Highest-Leverage Next Work

1. Re-run `bash scripts/observe_validation.sh` in an environment with Rust, cargo, and the configured wrapper available; require `cargo fmt --check`, `cargo test --all-targets`, and `cargo clippy --all-targets -- -D warnings` to produce pass/fail evidence.
2. Run `cargo run --example ollama_judgment` with `CANON_OLLAMA_BASE_URL` and `CANON_OLLAMA_MODEL` set to a local Ollama endpoint, then preserve the resulting receipt/proof trace.
3. Regenerate and commit or archive `state/rustc/ai/graph.json` so graph-health claims are inspectable from the delta artifacts and captured by the harness.
4. Add one live external observation fixture and one external action fixture behind deterministic receipts.
5. Add semantic artifact verification fixtures that prove more than hash lineage.
6. Reduce reliance on `unwrap()` in non-test logic, or classify each unwrap as test-only, validated invariant, or technical debt.

## Verdict

```text
classification = serious_deterministic_runtime_prototype
not_yet = validated_autonomous_self_improving_agent
deployment_readiness = low_without_reproduced_cargo_validation_and_external_loop_tests
score_confidence = medium_for_static_architecture_low_for_runtime_behavior
```
