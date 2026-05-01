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
ARCH = README-goal alignment score
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

One-line explanation: the system is scored by implemented, replayable evidence paths; claims that are architectural but not yet live, routed, or externally verified are discounted.

## Score Summary

```text
K  = 8.5 / 10
C  = 7.6 / 10
A  = 7.0 / 10
R  = 8.5 / 10

OB = 5.8 / 10
CX = 5.3 / 10
ME = 5.7 / 10
PL = 5.5 / 10
LL = 7.9 / 10
JG = 5.6 / 10
TO = 8.1 / 10
VF = 8.3 / 10
EV = 6.1 / 10
PO = 7.7 / 10
LE = 6.4 / 10
OR = 6.6 / 10

CORE = 7.87 / 10
CAP  = 6.50 / 10
IMPL = 7.73 / 10
ARCH = 6.82 / 10

max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR) = K = 8.5 / 10 = good
```

## Static Review Inputs

```text
review_date = 2026-04-30_static
source_review_scope = README.md + Cargo.toml + src/**/*.rs + state/rustc/ai/graph.json + rubric/score.md
cargo_build_status = not_run_in_this_review
cargo_test_status = not_run_in_this_review
cargo_example_status = not_run_in_this_review
last_known_user_validated_status = cargo_test_99_99_plus_ollama_example_passed_before_current_static_review
unsafe_policy = forbid_unsafe_code

readme_goal = reduce_cost_of_autonomous_reasoning_while_increasing_quality_and_trustworthiness
readme_core_claim = frozen_kernel_plus_capability_intelligence_plus_tlog_policy_learning
readme_status_accuracy = mostly_current_for_local_tooling_ollama_observation_and_generic_llm_proof_spine

source_files_reviewed = 51 rust files
source_loc_reviewed = 14020
test_count_in_src_lib = 100
cargo_dependencies = none

graph_schema_version = 9
graph_node_count = 2085
graph_edge_count = 5693
graph_cfg_node_count = 7820
graph_cfg_edge_count = 10650
graph_bridge_edge_count = 1904
graph_redundant_path_pair_count = 606
graph_alpha_pathway_count = 15
graph_intent_class_coverage = 584/584fn
graph_unknown_low_confidence_functions = 429
```

## Implemented Evidence Surfaces

```text
kernel_phase_count = 12
execution_gate_count = 7
total_gate_count = 8
tlog_schema_version = 5

tooling_record_split = present
tooling_live_sandbox_file_executor = present
tooling_live_sandbox_process_executor = present
tooling_durable_effect_receipts = present
tooling_registry_policy_hash_binding = present
tooling_command_allowlist = present
tooling_cwd_lock = present
tooling_environment_lock = present
tooling_timeout_kill = present
tooling_stdout_stderr_digests = present
tooling_exit_status_receipt = present
tooling_max_output_bytes = present
tooling_receipt_replay = present
tooling_api_process_receipt_route = present

bounded_file_observation_ingress = present
observation_cursor_persistence = present
observation_backpressure = present
live_sse_webhook_browser_observation = absent

provider_backed_llm_client = local_ollama_openai_compatible_request_adapter_present
provider_response_parser = present_for_openai_compatible_ollama_chat_completion_subset
ollama_effect_receipt_replay = present
ollama_receipt_tamper_matrix = present
ollama_endpoint_provenance = present
ollama_pre_receipt_non_local_rejection = present
ollama_durable_judgment_proof_event = present
ollama_bidirectional_receipt_proof_hash_binding = present
ollama_proof_event_seq_receipt_hash_binding = present
ollama_proof_event_order_replay = present
ollama_timeout_retry_idempotency_receipt_binding = present_in_source

generic_llm_verification_proof_projection = present
generic_verification_proof_binding_checker = present
generic_proof_spine_for_tooling_process_semantic_verification = absent
provider_signed_receipts = absent
streaming_llm_receipt_validation = absent
external_api_action_tools = absent
distributed_orchestration = absent
```

## Module Scores

| Module                     | State                      | Score | Judgment                                                                                                                                                                                                                                                                 |
|----------------------------+----------------------------+-------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `kernel`                   | frozen deterministic core  |   8.5 | Strongest module. Pure state transition, typed gates, phase discipline, and stable hash semantics remain the system anchor. Do not expand it.                                                                                                                            |
| `codec`                    | durable codec base         |   7.6 | NDJSON and receipt codecs are real. Weakness is schema migration and manual field evolution across several receipt families.                                                                                                                                             |
| `api`                      | credible command surface   |   7.0 | Commands expose evidence and process receipts. Still not a hardened network service with auth scopes, hostile-client boundaries, and streaming ingress.                                                                                                                  |
| `runtime`                  | strong replay engine       |   8.5 | Tick, durable run, transition legality, writer, command ledger, proof-event order checks, and replay verification are coherent.                                                                                                                                          |
| `capability/observation`   | bounded file ingress seed  |   5.8 | Append-only line source, cursor, batch limit, and backpressure exist. It is still not live autonomous observation because SSE/webhook/browser/API sources are absent.                                                                                                    |
| `capability/context`       | deterministic context seed |   5.3 | Context evidence exists. Missing retrieval, grounding, conflict handling, source ranking, and token-budget control.                                                                                                                                                      |
| `capability/memory`        | durable lookup seed        |   5.7 | Memory store/index surfaces exist. Missing namespaces, embeddings, decay, invalidation, provenance, and cross-run query planning.                                                                                                                                        |
| `capability/planning`      | typed plan evidence        |   5.5 | Plan records can drive gates. Missing dependency solving, alternatives, cost/risk search, and repair planning.                                                                                                                                                           |
| `capability/llm`           | local Ollama proof path    |   7.9 | Best intelligence-facing path. Local OpenAI-compatible adapter, receipt binding, endpoint provenance, retry/budget/idempotency fields, and generic proof projection exist. Missing streaming validation, constrained decoding, provider signatures, and provider policy. |
| `capability/judgment`      | minimal judgment evidence  |   5.6 | Judgment can be represented and driven by the Ollama path. Missing comparative deliberation, irreversible-boundary checks, and policy-backed judgment.                                                                                                                   |
| `capability/tooling`       | strongest live capability  |   8.1 | File/process execution is split, bounded, authorized, receipted, API-visible, and replay-checkable. Missing external API tools and universal effect routing.                                                                                                             |
| `capability/verification`  | strong internal verifier   |   8.3 | Verification has durable proof records and generic LLM proof binding. Still mostly verifies internal receipts rather than external semantic truth.                                                                                                                       |
| `capability/eval`          | record scorer              |   6.1 | Eval records exist. Missing calibrated benchmarks, adversarial scoring, evaluator provenance, and threshold governance.                                                                                                                                                  |
| `capability/policy`        | strong policy foundation   |   7.7 | Durable policy store, hashing, promotion, feedback, and registry binding exist. Missing conflict resolution, expiry, rollback, signatures, and migrations.                                                                                                               |
| `capability/learning`      | narrow promotion path      |   6.4 | Learning can promote from TLog into policy. Missing causal attribution, pattern mining, strategy synthesis, and automatic capability generation.                                                                                                                         |
| `capability/orchestration` | ordering layer             |   6.6 | Routing/submission order exists. Missing queues, leases, priorities, retry policy, worker isolation, and distributed scheduling.                                                                                                                                         |

## Artifact Judgment

```text
README = architecturally correct and now updated to distinguish implemented source from validation status
score_md_before_update = stale_graph_counts_and_overconfident_validation_language
src = substantial deterministic prototype with local effects, local Ollama, bounded observation, policy, learning, verification, and replay
graph = useful but noisy; 606 redundant path pairs and 15 alpha pathways show structural debt
kernel = strongest module
tooling = strongest live capability module
verification = second strongest capability module
weakest_live_gap = observation_is_file_backed_not_external
```

The codebase is no longer a sketch. It has enough typed evidence, receipt, replay, policy, learning, tooling, observation, and LLM surfaces to be scored as a real deterministic agent-runtime prototype.

The hard criticism: the system still does not satisfy the README autonomy target. It can execute and prove local effects, but it cannot yet observe the world through live external sources, act through authenticated external APIs, validate provider-signed evidence, or convert TLog history into broad strategic policy updates. The proof envelope is strong; the external substrate is still thin.

The graph confirms both progress and debt. The latest graph contains 2085 semantic nodes, 5693 semantic edges, 7820 CFG nodes, 10650 CFG edges, 1904 bridge edges, 606 redundant path pairs, and 15 alpha pathways. Function intent coverage is complete at 584/584fn, but 429 functions remain `unknown_low_confidence`, so the semantic manifest layer is wide but shallow.

## Regression / Improvement Delta

```text
previous_CORE = 7.87 / 10
current_CORE  = 7.87 / 10

previous_CAP = 6.48 / 10
current_CAP  = 6.50 / 10

previous_IMPL = 7.72 / 10
current_IMPL  = 7.73 / 10

previous_ARCH = 6.82 / 10
current_ARCH  = 6.82 / 10
```

The score only moves slightly. Generic LLM verification proof binding is a real source-level improvement, so `LL` and `VF` improve marginally. `ARCH` does not move because the large autonomy gaps are unchanged: live observation routing, external tools, streaming validation, signed provider receipts, and universal effect normal form are still incomplete.

## Highest Leverage Next Work

1. **Validate the 100-test source state.** Run the full suite and the Ollama example locally; do not raise scores again until validation is current.
2. **Route observation ingress through API/runtime.** The file-backed source must become a real live observation path into the Invariant gate.
3. **Canonicalize one execution normal form.** Collapse artifact, process, LLM, observation, and proof outputs into: `request → authorize → execute → Effect { kind, digest, metadata } → receipt → proof → TLog → replay`.
4. **Extend generic proof records beyond LLM.** Apply `VerificationProofRecord` to tooling, process, semantic verification, observation, and future providers.
5. **Add streaming LLM validation.** Hash, bound, type, and replay response chunks, not only completed response bodies.
6. **Add provider-signed receipts.** Current receipts prove local adapter behavior, not provider-origin authenticity.
7. **Reduce graph debt surgically.** Target the 15 alpha pathways and the highest-frequency redundant path owners. Do not blindly optimize all 606 redundant paths.
8. **Do not expand the kernel.** Keep intelligence, external semantics, and learning pressure above the frozen kernel.

## Updated Verdict

```text
objective_rating = ARCH = 6.82 / 10
system_level = deterministic evidence runtime with bounded local sandbox execution, local Ollama proof receipts, bounded file observation ingress, policy promotion, and a first generic LLM proof spine
best_property = frozen kernel plus replay discipline plus receipted local file/process/LLM effects
weakest_property = live autonomy remains mostly absent because observation and action are still local/file-backed or adapter-level
next_score_unlock = validated 100-test state + routed observation ingress + universal effect normal form
```

The project is moving correctly. The next unlock is not more abstraction. The next unlock is routing live inputs and outputs through the same receipt/proof/replay spine that already works locally.
