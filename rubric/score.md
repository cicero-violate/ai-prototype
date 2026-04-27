# Canon Agent Module Score

## Variables

```text
K  = kernel score
C  = codec score
A  = api score
R  = runtime score
OB = capability/observation score
CX = capability/context score
ME = capability/memory score
PL = capability/planning score
LL = capability/llm score
JG = capability/judgment score
TO = capability/tooling score
VF = capability/verification score
EV = capability/eval score
PO = capability/policy score
LE = capability/learning score
OR = capability/orchestration score

CORE = deterministic foundation score
CAP  = implemented capability-layer score
IMPL = implemented-source score
ARCH = declared architecture score
GOOD = strongest present module
```

## Equations

```text
CORE = (K · C · A · R)^(1/4)
CAP  = (OB · CX · ME · PL · LL · JG · TO · VF · EV · PO · LE · OR)^(1/12)
IMPL = (K · C · A · R · TO · VF · PO · LE)^(1/8)
ARCH = (K · C · A · R · OB · CX · ME · PL · LL · JG · TO · VF · EV · PO · LE · OR)^(1/16)
GOOD = max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR)
```

One-line explanation: geometric scoring punishes hollow autonomy claims; real side effects raise the score only when they are authorized, receipted, replayable, and externally useful.

## Score Summary

```text
K  = 8.4 / 10
C  = 7.2 / 10
A  = 6.3 / 10
R  = 8.2 / 10

OB = 4.0 / 10
CX = 5.2 / 10
ME = 5.6 / 10
PL = 5.3 / 10
LL = 4.8 / 10
JG = 5.2 / 10
TO = 6.6 / 10
VF = 6.9 / 10
EV = 5.8 / 10
PO = 7.4 / 10
LE = 6.1 / 10
OR = 6.3 / 10

CORE = 7.48 / 10
CAP  = 5.69 / 10
IMPL = 7.10 / 10
ARCH = 6.09 / 10

max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR) = K = 8.4 / 10 = good
```

## Static Review Inputs

```text
source_files = 43
source_lines = 8843
rust_functions_regex = 477
rust_structs_regex = 55
rust_enums_regex = 27
rust_impls_regex = 86
unit_tests_declared = 83
integration_tests_detected = 0
cargo_build_status = not_run_by_instruction
cargo_test_status = not_run_by_instruction

semantic_graph_nodes = 1310
semantic_graph_edges = 3326
cfg_nodes = 4122
cfg_edges = 5344
bridge_edges = 1231
redundant_path_pairs = 499
alpha_pathways = 13
graph_schema_version = 9
graph_captured_at_ms = 1777318140708

semantic_manifest_status = low_confidence_or_unknown_for_1310 / 1310
largest_cfg_function_1 = codec::ndjson::pop_event, 127 blocks
largest_cfg_function_2 = runtime::verify::validate_event, 99 blocks
largest_cfg_function_3 = capability::tooling::record::LiveSandboxToolExecutor::execute_packet, 95 blocks
largest_cfg_function_4 = runtime::verify::verify_tlog_from, 82 blocks
largest_redundant_path_owner = runtime::verify::validate_event, 272 directional redundant path entries

tooling_live_sandbox_file_executor = present
tooling_durable_effect_receipts = present
tooling_registry_policy_hash_binding = present
tooling_process_execution = absent
tooling_api_execution = absent
provider_backed_llm_client = absent
real_observation_stream_parser = absent
external_artifact_verification = partial_internal_file_receipt_only

README_reviewed = yes
src_reviewed = src/**/*.rs
artifacts_reviewed = state/rustc/ai/graph.json + rubric/score.md
```

## Critical Judgment

The project improved again. The most important change is that tooling now has an actual live side-effect seam: `LiveSandboxToolExecutor` writes a sandbox artifact, binds artifact path/content/root hashes into a `ToolReceipt`, derives a `ToolEffectReceipt` from the persisted execution event, appends/loads receipt NDJSON, and verifies receipts against the TLog. This is no longer only simulated evidence routing.

That deserves a real score increase. The system now demonstrates the minimum useful pattern for external work:

```text
request -> registry authorization -> sandbox file effect -> receipt -> TLog event -> replay check
```

The hard criticism remains: this is still not a general autonomous execution engine. The live tooling path writes deterministic sandbox files. It does not yet run a process, call an API, stream stdout/stderr, enforce wall-clock/CPU/memory budgets, isolate environment variables, authenticate external providers, or verify real-world claims. It is a good first real effect, not yet a complete tool runtime.

Current classification:

```text
current_system = deterministic evidence-runtime with first live file-side-effect capability
not_yet = general autonomous agent
main_strength = replayable side effects under registry authority
main_weakness = real-world capability breadth and verifier complexity
```

The README is still architecturally ahead of the implementation. The source now supports the stated direction better than before, but the LLM, observation, memory, planning, and verification layers remain mostly typed deterministic scaffolds rather than live adaptive systems.

## Module Rating Table

| Module                     | Status                                      | Score | Reason                                                                                                                                                                                                 |
|----------------------------|---------------------------------------------|------:|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `kernel`                   | strong                                      |   8.4 | Pure deterministic phase/gate/state core with learning gate, packet effects, capability registry projection, hash-stable control events, and clean kernel boundary. Penalized for toy packet domain.     |
| `codec`                    | useful but too manual                       |   7.2 | NDJSON roundtrip now carries registry projection and TLog fields. Penalized for large hand-written parsers, numeric positional formats, and `pop_event` owning the largest CFG cluster.                |
| `api`                      | credible in-process protocol                |   6.3 | Command envelopes, schema hash binding, duplicate/conflict rejection, atomic batches, and receipts exist. Still lacks network service, auth, quotas, streaming, and hostile-client hardening.           |
| `runtime`                  | strong deterministic replay core            |   8.2 | Reducer, canonical writer, durable replay, drift checks, command-ledger reconstruction, registry projection validation, and TLog verification are meaningful. Penalized for verifier complexity.       |
| `capability/observation`   | typed evidence facade                       |   4.0 | Observation records and cursor ordering exist. No live SSE/webhook/filesystem/browser input parser, provenance authentication, or replayable observation ingestion.                                  |
| `capability/context`       | deterministic assembler seed                |   5.2 | Context records combine internal signals. Still lacks real retrieval, token budgeting, policy-scoped context, document grounding, and prior-run synthesis.                                             |
| `capability/memory`        | deterministic lookup seed                   |   5.6 | Stable ordered facts and weighted lookup exist. Still lacks durable namespaces, embeddings, provenance, invalidation, decay, and cross-run memory queries.                                             |
| `capability/planning`      | typed gate producer                         |   5.3 | Plan records can bind ready tasks and pass the Plan gate. It is not yet a planner: no dependency solving, graph expansion, scheduling, cost model, alternatives, or repair search.                    |
| `capability/llm`           | structured adapter mock                     |   4.8 | Prompt/response records, schema hash, token counts, policy feedback, and judgment conversion exist. No provider client, retries, streaming parser, constrained decoder, or cost ledger.               |
| `capability/judgment`      | minimal typed judgment record               |   5.2 | Judgment is represented as evidence, but it does not yet compare alternatives, model uncertainty, resolve conflicts, or enforce irreversible-boundary reasoning.                                      |
| `capability/tooling`       | first real live side-effect capability      |   6.6 | Live sandbox file writes, explicit registry authorization, policy hash binding, artifact hashes, durable effect receipts, sidecar NDJSON, and replay checks exist. Still lacks process/API execution.   |
| `capability/verification`  | strongest semantic capability after tooling |   6.9 | Semantic profiles, request/receipt hashes, lineage repair checks, tamper rejection, and tool-effect receipt replay exist. Still mostly verifies internal artifacts instead of external outputs.        |
| `capability/eval`          | solid record scorer                         |   5.8 | Eval dimensions, thresholds, and gate submission exist. Still lacks calibrated benchmarks, adversarial scoring, evaluator provenance, and threshold governance.                                       |
| `capability/policy`        | strongest capability foundation             |   7.4 | Append-only entries, durable promotion, versioning, fingerprinting, policy feedback, and registry policy hashing are good. Needs conflict resolution, expiry, rollback, signatures, and migrations.   |
| `capability/learning`      | real but narrow                             |   6.1 | Policy promotion reads TLog and emits policy facts. Still not pattern mining, counterexample learning, causal attribution, strategy synthesis, or automatic capability generation.                    |
| `capability/orchestration` | meaningful ordering layer                   |   6.3 | Orchestration records order capability submissions and skip passed gates. Still lacks distributed workers, leases, queues, priorities, retry policy, backpressure, and parallel isolation.             |

## Artifact Judgment

```text
graph_json = useful_structural_source_of_truth
README = correct_direction_but_ahead_of_source
src = deterministic implementation with first live file-effect seam
rubric_score = updated_to_match_current_static_state
```

`graph.json` reports 1310 semantic nodes, 3326 semantic edges, 4122 CFG nodes, 5344 CFG edges, 1231 bridge edges, 499 redundant path pairs, and 13 alpha pathways. This is enough structure to guide refactoring, but not enough semantic confidence to delegate judgment to the graph.

The graph semantics layer is still weak. Every semantic manifest is effectively low-confidence or unknown. Treat the graph as structural evidence: useful for ownership, size, CFG concentration, and redundancy. Do not treat its intent labels as authority.

The critical complexity moved. `codec::ndjson::pop_event` remains the largest CFG function at 127 blocks. `runtime::verify::validate_event` remains a high-risk verifier cluster at 99 blocks and owns the largest redundant-path concentration. `LiveSandboxToolExecutor::execute_packet` is now a new correctness-sensitive CFG cluster at 95 blocks. That is expected after adding live effects, but it should be split before more tool kinds are added.

## Regression / Improvement Delta

```text
previous_CORE = 7.35 / 10
current_CORE  = 7.48 / 10

previous_CAP = 5.52 / 10
current_CAP  = 5.69 / 10

previous_IMPL = 6.64 / 10
current_IMPL  = 7.10 / 10

previous_ARCH = 5.93 / 10
current_ARCH  = 6.09 / 10
```

The project improved. The gain comes from the first live sandbox file effect, artifact-content/root/path hashing, durable tool-effect receipt sidecar support, registry authorization at execution time, and replay verification for persisted tool effects.

The score is still only slightly above 6 because the implementation now has one real file-side-effect path, while the architecture claims a self-improving autonomous agent. The foundation is increasingly credible. The intelligence loop is still mostly absent.

## Highest Leverage Next Work

1. **Promote tooling from file artifact writer to real sandbox runner.** Add process execution with explicit command allowlist, cwd lock, environment lock, timeout, stdout/stderr digests, exit status, max output bytes, and receipt replay.
2. **Split `LiveSandboxToolExecutor::execute_packet`.** Separate request construction, authorization, artifact body construction, path validation, write/sync, and receipt construction into typed units.
3. **Split verifier and codec CFG clusters.** Break `codec::ndjson::pop_event`, `runtime::verify::validate_event`, and `runtime::verify::verify_tlog_from` into smaller validators with table-driven enum decoding.
4. **Move tests out of `src/lib.rs`.** The root module is 2268 lines and contains 83 unit tests. Move integration-style tests into `tests/` and module-specific test files.
5. **Verify external artifacts, not only packet effects.** Add verification over command receipts, file digests, build/test outputs, API response digests, and signed provider receipts.
6. **Add durable memory before smarter planning.** Planning will remain shallow until prior runs, artifacts, policies, and observations are queryable with provenance and invalidation.
7. **Make graph confidence a gate.** Low-confidence semantic manifests should warn or fail before graph-derived conclusions are treated as design authority.

## Updated Verdict

```text
objective_rating = ARCH = 6.09 / 10
system_level = deterministic evidence-runtime with first live sandboxed file capability
best_property = kernel/runtime replay discipline plus receipted side effects
weakest_property = missing process/API execution and adaptive intelligence loop
next_score_unlock = process-backed sandbox tooling with stdout/stderr receipts and verifier replay
```

The kernel is good. Runtime is strong. Policy is good. Tooling finally became real enough to matter. The next move should not be more abstract kernel work; it should be a process-backed sandbox tool executed through the same registry, receipt, TLog, and verifier path.
