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

CORE = implemented foundation score
CAP  = implemented capability-layer score
IMPL = implemented-source score
ARCH = declared architecture score
GOOD = strongest present module
```

## Equations

```text
CORE = (K · C · A · R)^(1/4)
CAP  = (OB · CX · ME · PL · LL · JG · TO · VF · EV · PO · LE · OR)^(1/12)
IMPL = (K · C · A · R · JG · EV · PO · LE)^(1/8)
ARCH = (K · C · A · R · OB · CX · ME · PL · LL · JG · TO · VF · EV · PO · LE · OR)^(1/16)
GOOD = max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR)
```

One-line explanation: geometric scoring punishes missing execution surfaces; a strong deterministic kernel cannot hide weak external intelligence capability.

## Score Summary

```text
K  = 8.0 / 10
C  = 6.9 / 10
A  = 5.4 / 10
R  = 7.8 / 10

OB = 3.6 / 10
CX = 4.8 / 10
ME = 5.2 / 10
PL = 4.8 / 10
LL = 4.1 / 10
JG = 4.7 / 10
TO = 3.7 / 10
VF = 6.1 / 10
EV = 5.4 / 10
PO = 6.9 / 10
LE = 5.7 / 10
OR = 5.8 / 10

CORE = 6.94 / 10
CAP  = 4.98 / 10
IMPL = 6.25 / 10
ARCH = 5.41 / 10

max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR) = K = 8.0 / 10 = good
```

## Static Review Inputs

```text
source_files = 41
source_lines = 6906
rust_functions_regex = 347
unit_tests_declared = 68
integration_tests_detected = 0
cargo_build_status = skipped_per_user_instruction
cargo_test_status = skipped_per_user_instruction

semantic_graph_nodes = 1072
semantic_graph_edges = 2575
cfg_nodes = 3162
cfg_edges = 4133
bridge_edges = 916
redundant_path_pairs = 481
alpha_pathways = 11
graph_schema_version = 9

index_json_node_count = 35
index_json_edge_count = 44
index_json_status = stale_or_lossy_relative_to_graph_json

kernel_files = 1
codec_files = 2
runtime_files = 8
api_files = 3
capability_files = 25
docs_reviewed = README.md + docs/ai_architecture.md + docs/ai_architecture.dot
artifacts_reviewed = state/rustc/ai/graph.json + state/rustc/index.json + rubric/score.md
```

## Critical Judgment

The project is now a real deterministic evidence runtime, not merely an atomic kernel demo. It has a stronger phase model, durable TLog replay, command envelopes, idempotent command receipts, typed capability records, policy storage, learning promotions, semantic verification records, and orchestration ordering.

It is still not yet an autonomous intelligent agent system. The current system is best classified as:

```text
current_system = deterministic kernel + replayable runtime + in-process command API + typed evidence capability facades
not_yet = autonomous external agent with real observation, real tools, real LLM provider calls, durable memory retrieval, authenticated API service, and semantic world verification
```

The biggest improvement is breadth: most declared layers now have source files and tests. The biggest architectural defect is dependency direction: `codec` imports `runtime::CanonError`, and `runtime::durable` imports `api::protocol::CommandLedger`. That violates the README's layer rule and prevents the architecture from being honestly called frozen-layer clean.

The second major defect is intelligence depth. The capability modules encode records, hashes, and pass/fail submissions, but most do not yet perform live work. Observation does not observe an external stream. Tooling does not call tools. LLM does not call a provider. Memory is deterministic lookup, not retrieval over real accumulated state. Learning promotes narrow policy facts, not generalized strategies.

## Module Rating Table

| Module                     | Status                       | Score | Reason                                                                                                                                                                                                                                         |
|----------------------------+------------------------------+-------+------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `kernel`                   | strong                       |   8.0 | Clean deterministic phase/gate/state core with learning phase and structural invariants. Still exposes broad public state construction and uses compact toy packet semantics rather than rich typed domain state.                              |
| `codec`                    | useful but layer-leaky       |   6.9 | Deterministic NDJSON TLog codec with disk append and replay support. Penalized because codec imports runtime error type, uses positional numeric arrays that are hard to audit manually, and has no migration adapter beyond schema constants. |
| `api`                      | improving                    |   5.4 | Command envelopes, command hash binding, duplicate replay, atomic batch rejection, and receipts exist. Still in-process only: no HTTP/gRPC server, auth, capability permissions, streaming, rate limits, or external command boundary.         |
| `runtime`                  | strong but impure boundary   |   7.8 | Reducer, durable execution, convergence failure, replay verification, semantic deltas, and disk/memory drift checks are real. Penalized because runtime imports API ledger code and therefore is not cleanly below API.                        |
| `capability/observation`   | record facade                |   3.6 | ObservationRecord can submit evidence, but there is no external feed, SSE frame parser, webhook reader, filesystem watcher, or source authenticity model.                                                                                      |
| `capability/context`       | shallow but coherent         |   4.8 | ContextRecord composes packet and memory signal into analysis evidence. It is deterministic, but not a real context assembler over documents, runs, graph state, or policy.                                                                    |
| `capability/memory`        | deterministic seed           |   5.2 | MemoryIndex has deterministic weighted lookup and stable ordering. It lacks durable storage, embeddings, namespaces, provenance, decay, invalidation, and cross-run retrieval.                                                                 |
| `capability/planning`      | typed gate producer          |   4.8 | PlanRecord can bind ready tasks and drive Plan gate. It is not a planner: no task graph expansion, dependency solving, resource budget, or schedule repair.                                                                                    |
| `capability/llm`           | structured mock adapter      |   4.1 | LlmRecord, prompt/response hashes, token count, and policy feedback hooks exist. There is no provider client, schema-constrained decoding, retry model, streaming parser, or cost accounting.                                                  |
| `capability/judgment`      | minimal                      |   4.7 | JudgmentRecord exists and is test-covered through API. It does not yet evaluate alternatives, conflict evidence, uncertainty, or irreversible-boundary risk.                                                                                   |
| `capability/tooling`       | placeholder execution record |   3.7 | ToolExecutionRecord materializes artifacts by evidence. There is no tool registry, sandbox, permission model, command execution, output capture, rollback, or side-effect ledger.                                                              |
| `capability/verification`  | best capability surface      |   6.1 | ArtifactSemanticProfile verifies receipt, lineage, semantic hash, and repair decisions. Still domain-toy: not checking real files, code semantics, build/test outputs, or external artifact validity.                                          |
| `capability/eval`          | solid record scorer          |   5.4 | EvalRecord has dimensions, threshold, and gate submission. It lacks calibrated objective metrics, benchmark suites, evaluator provenance, and adversarial scoring.                                                                             |
| `capability/policy`        | strong capability foundation |   6.9 | Append-only policy entries, durable promotion, feedback hash, versioning, fingerprinting, and load path exist. Needs conflict resolution, rollback rules, policy scope, expiration, and signed provenance.                                     |
| `capability/learning`      | real but narrow              |   5.7 | PolicyPromotion reads TLog and can materialize policy facts. Learning is still narrow promotion logic, not pattern mining, counterexample handling, or strategy synthesis.                                                                     |
| `capability/orchestration` | meaningful ordering layer    |   5.8 | OrchestrationRecord builds ordered submissions and skips passed gates. It is not yet distributed orchestration: no leases, queues, workers, priorities, retries, or concurrent run isolation.                                                  |

## Artifact Judgment

```text
graph_json = useful_source_of_truth
index_json = stale_or_lossy_summary
README = architecturally clear but ahead of implementation
src = broad implementation with incomplete layer purity
rubric_score = updated_to_match_current_static_state
```

`graph.json` confirms the system has moved beyond a small kernel: 1072 semantic nodes, 2575 semantic edges, 3162 CFG nodes, 4133 CFG edges, 916 bridge edges, and 11 alpha pathways. The graph also shows concentration risk in large parsing/replay functions such as `codec::ndjson::pop_event`, `runtime::verify::validate_event`, and `runtime::verify::verify_tlog_from`.

`state/rustc/index.json` is not reliable as a full graph summary because it reports only 35 nodes and 44 edges while `graph.json` reports 1072 nodes and 2575 edges. Keep it as a locator, not as a scoring source.

## Regression / Improvement Delta

```text
previous_CORE = 6.62 / 10
current_CORE  = 6.94 / 10

previous_CAP = 4.50 / 10
current_CAP  = 4.98 / 10

previous_IMPL = 5.89 / 10
current_IMPL  = 6.25 / 10

previous_ARCH = 4.95 / 10
current_ARCH  = 5.41 / 10
```

The project improved overall because API protocol, durable replay, command ledger reconstruction, policy persistence, semantic verification, LLM feedback, and orchestration tests are now present. The score is not higher because the declared architecture promises autonomous capability layers, but the implementation is still mostly deterministic evidence routing plus typed records.

## Highest Leverage Next Work

1. **Fix layer purity first.** Move `CanonError` out of runtime into a lower shared/error module or kernel-adjacent type, and move `CommandLedger` dependency out of runtime so runtime no longer imports API.
2. **Split `src/lib.rs` tests.** The 1802-line root module is carrying integration-style tests. Move them into integration tests or focused module test files to reduce root-module gravity.
3. **Make one capability real.** Pick `tooling` or `observation` and connect it to an actual external side effect with receipts, permissions, and verification. This gives the kernel real work to govern.
4. **Replace positional NDJSON with auditable records or schema tooling.** Current numeric arrays are deterministic but hostile to manual audit and migration.
5. **Promote graph artifacts into score gates.** Treat redundant path pairs, layer leaks, CFG concentration, and stale graph indexes as first-class rubric inputs.

## Updated Verdict

```text
objective_rating = ARCH = 5.41 / 10
system_level = deterministic evidence-runtime prototype
best_property = kernel/runtime correctness discipline
weakest_property = real-world intelligence execution
next_score_unlock = layer purity + one live external capability
```

The kernel is good. The runtime is close to good. The architecture is promising but not clean yet. The intelligence layer is present as typed surfaces, not as autonomous capability. Fix the two upward dependencies, then make one capability perform real work under the TLog.
