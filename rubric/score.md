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

One-line explanation: geometric scoring punishes missing execution surfaces; a strong deterministic kernel cannot hide weak real-world intelligence capability.

## Score Summary

```text
K  = 8.1 / 10
C  = 6.7 / 10
A  = 5.6 / 10
R  = 7.7 / 10

OB = 3.8 / 10
CX = 5.0 / 10
ME = 5.4 / 10
PL = 5.0 / 10
LL = 4.5 / 10
JG = 4.9 / 10
TO = 4.2 / 10
VF = 6.4 / 10
EV = 5.6 / 10
PO = 7.0 / 10
LE = 5.9 / 10
OR = 6.0 / 10

CORE = 6.96 / 10
CAP  = 5.23 / 10
IMPL = 6.35 / 10
ARCH = 5.62 / 10

max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR) = K = 8.1 / 10 = good
```

## Static Review Inputs

```text
source_files = 41
source_lines = 7294
rust_functions_regex = 392
unit_tests_declared = 72
integration_tests_detected = 0
cargo_build_status = not_run_cargo_binary_missing_in_sandbox
cargo_test_status = not_run_cargo_binary_missing_in_sandbox

semantic_graph_nodes = 1123
semantic_graph_edges = 2718
cfg_nodes = 3317
cfg_edges = 4293
bridge_edges = 1017
redundant_path_pairs = 484
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

upward_layer_dependency_violations = 2
upward_layer_dependency_1 = codec -> runtime via src/codec/ndjson.rs using runtime::CanonError
upward_layer_dependency_2 = runtime -> api via src/runtime/durable.rs using api::protocol::CommandLedger

semantic_manifest_low_confidence_nodes = 1123 / 1123
largest_cfg_function_1 = codec::ndjson::pop_event, 123 blocks
largest_cfg_function_2 = runtime::verify::validate_event, 99 blocks
largest_cfg_function_3 = runtime::verify::verify_tlog_from, 71 blocks
largest_redundant_path_owner = runtime::verify::validate_event, 136 redundant pairs

docs_reviewed = README.md + docs/ai_architecture.md + docs/ai_architecture.dot
artifacts_reviewed = state/rustc/ai/graph.json + state/rustc/index.json + rubric/score.md
src_reviewed = src/**/*.rs
```

## Critical Judgment

The project is now a real deterministic evidence-runtime prototype. It has a typed kernel, a replayable TLog, deterministic event hashes, durable append/replay paths, command envelopes, idempotent command receipts, capability record submissions, a policy store, policy promotion, semantic verification records, deterministic tool receipts, and orchestration ordering.

It is still not an autonomous intelligent agent. The current system is best classified as:

```text
current_system = deterministic kernel + replayable runtime + typed evidence/capability facades + in-process command API
not_yet = autonomous external agent with live observation, real tool execution, real LLM provider calls, durable memory retrieval, authenticated service boundary, semantic artifact verification against reality, and adaptive strategy learning
```

The central architectural problem remains layer purity. The README declares a one-way dependency stack, but `codec` imports `runtime::CanonError`, and `runtime::durable` imports `api::protocol::CommandLedger`. That means the code cannot honestly claim a clean frozen kernel/runtime boundary yet. The defect is small in count but high in architectural gravity.

The second major problem is that intelligence is represented more than performed. Most capability modules produce typed records, hashes, and pass/fail submissions. That is useful. It is not enough. Observation does not observe an external stream. Tooling does not run real tools. LLM does not call a provider. Memory is deterministic in-process lookup, not durable retrieval. Learning promotes narrow facts from run history, not generalized execution policy.

The third major problem is graph semantics quality. `graph.json` is useful structurally, but every semantic manifest is low confidence. The graph can locate functions, edges, CFG concentration, and redundancy, but it is not yet a trustworthy semantic reasoning layer.

## Module Rating Table

| Module                     | Status                         | Score | Reason                                                                                                                                                                                                 |
|----------------------------|--------------------------------|-------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `kernel`                   | strong                         |   8.1 | Deterministic phase/gate/state core with structural validity, packet semantics, evidence application, recovery actions, and learning gate. Penalized for toy packet domain model and broad public state construction. |
| `codec`                    | useful but layer-leaky         |   6.7 | Durable NDJSON append/load and public roundtrip codec exist. Penalized for importing runtime error type, large positional decoders, audit-hostile numeric encoding, and high CFG complexity in parser functions. |
| `api`                      | credible in-process protocol   |   5.6 | Command envelopes, command hash binding, duplicate replay, atomic batches, and command receipts exist. Still lacks network service, auth, permissions, streaming, rate limiting, and external boundary hardening. |
| `runtime`                  | strong but boundary-impure     |   7.7 | Reducer, convergence handling, durable replay, disk/memory drift checks, TLog validation, and event hashing are meaningful. Penalized because runtime imports API ledger code.                         |
| `capability/observation`   | evidence facade                |   3.8 | ObservationRecord can submit invariant evidence and is routed through the API. It does not parse live SSE/webhook/filesystem/input streams or authenticate source observations.                         |
| `capability/context`       | deterministic assembler seed   |   5.0 | ContextRecord combines packet and memory signal into analysis evidence. It lacks retrieval over documents, prior runs, graph state, policy scope, or token-budgeted context construction.                |
| `capability/memory`        | deterministic seed             |   5.4 | MemoryIndex has stable weighted lookup and ordered facts. It lacks durable persistence, namespaces, provenance, embeddings, decay, invalidation, and cross-run retrieval.                               |
| `capability/planning`      | typed gate producer            |   5.0 | PlanRecord can bind a ready task and drive the Plan gate. It is not yet a planner: no graph expansion, dependency solving, budget accounting, schedule repair, or alternative plan search.              |
| `capability/llm`           | structured adapter mock        |   4.5 | Prompt/response records, token counts, schema hash, policy feedback, and judgment conversion exist. There is no provider client, constrained decoding, retry policy, streaming parser, or cost ledger.   |
| `capability/judgment`      | minimal record surface         |   4.9 | JudgmentRecord is typed and test-covered through API paths. It does not yet score alternatives, resolve conflicting evidence, model uncertainty, or enforce irreversible-boundary reasoning.            |
| `capability/tooling`       | deterministic receipt facade   |   4.2 | Tool requests, deterministic executor, receipts, and artifact materialization records exist. There is still no real tool registry, sandbox, permission layer, process execution, rollback, or side-effect ledger. |
| `capability/verification`  | best capability surface        |   6.4 | Artifact profiles, semantic hashes, receipt checks, lineage checks, denial checks, and repair decisions exist. Still validates toy artifacts, not real files, build outputs, APIs, or external claims.   |
| `capability/eval`          | solid record scorer            |   5.6 | Eval dimensions, threshold comparison, and gate submission exist. It lacks calibrated benchmarks, evaluator provenance, adversarial scoring, objective-specific metrics, and policy-driven threshold governance. |
| `capability/policy`        | strongest capability foundation|   7.0 | Append-only entries, durable promotion, feedback hash, versioning, fingerprinting, latest lookup, and load path exist. Needs conflict handling, scope, rollback, expiry, signatures, and migration policy. |
| `capability/learning`      | real but narrow                |   5.9 | PolicyPromotion reads TLog and emits policy facts. It is still narrow promotion logic, not pattern mining, counterexample learning, causal attribution, or strategy synthesis.                         |
| `capability/orchestration` | meaningful ordering layer      |   6.0 | OrchestrationRecord orders submissions and skips passed gates. It lacks distributed workers, leases, queues, priorities, retries, parallel isolation, and backpressure.                                  |

## Artifact Judgment

```text
graph_json = useful_structural_source_of_truth
index_json = stale_or_lossy_summary
README = architecturally clear but ahead of implementation
src = broad deterministic implementation with incomplete layer purity
rubric_score = updated_to_match_current_static_state
```

`graph.json` confirms meaningful implementation growth: 1123 semantic nodes, 2718 semantic edges, 3317 CFG nodes, 4293 CFG edges, 1017 bridge edges, and 11 alpha pathways. It also exposes risk: parser/replay/validation logic is concentrated in large CFG functions, and `runtime::verify::validate_event` alone owns 136 redundant path pairs.

`state/rustc/index.json` is not reliable as a scoring source. It reports only 35 nodes and 44 edges while `graph.json` reports 1123 nodes and 2718 edges. Treat it as a locator, not as the source of architectural truth.

## Regression / Improvement Delta

```text
previous_CORE = 6.94 / 10
current_CORE  = 6.96 / 10

previous_CAP = 4.98 / 10
current_CAP  = 5.23 / 10

previous_IMPL = 6.25 / 10
current_IMPL  = 6.35 / 10

previous_ARCH = 5.41 / 10
current_ARCH  = 5.62 / 10
```

The project improved because the static source and graph now show more capability breadth, stronger typed records, more command-ledger behavior, richer semantic verification, deterministic tooling receipts, and a larger test surface. The score is not higher because the declared architecture promises autonomous capability layers, but the implementation still mostly routes typed evidence through a deterministic state machine.

## Highest Leverage Next Work

1. **Fix layer purity before adding features.** Move `CanonError` below codec/runtime or into a neutral error module, and move command-ledger ownership out of runtime so runtime no longer imports API.
2. **Make one capability real.** Pick `tooling` first. Add a sandboxed, permissioned tool registry with durable side-effect receipts and verification-bound outputs.
3. **Split root tests out of `src/lib.rs`.** The root module carries 72 declared unit tests and 1873 lines. Move integration-style tests into `tests/` or focused module test files to reduce root gravity.
4. **Reduce CFG concentration.** Split `codec::ndjson::pop_event`, `runtime::verify::validate_event`, and `runtime::verify::verify_tlog_from` into smaller typed validators and decoders.
5. **Upgrade graph semantics.** The graph is structurally useful but semantically weak because all 1123 semantic manifests are low confidence. Promote manifest confidence into an explicit score gate.
6. **Replace toy artifact semantics.** Verification must eventually check real files, build/test outputs, API responses, or signed external receipts, not only packet-derived hashes.

## Updated Verdict

```text
objective_rating = ARCH = 5.62 / 10
system_level = deterministic evidence-runtime prototype
best_property = kernel/runtime correctness discipline
weakest_property = real-world autonomous execution
next_score_unlock = layer purity + one live external capability
```

The kernel is good. The runtime is close to good. Policy is the strongest capability surface. The architecture is promising but not clean. Intelligence currently exists as typed control surfaces; it does not yet exist as live autonomous capability. Fix the two upward dependencies, then make `tooling` perform real work under the TLog.