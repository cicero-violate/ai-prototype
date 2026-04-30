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
ARCH = declared README goal alignment score
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

One-line explanation: the project is judged by how much deterministic replay, bounded execution, durable evidence, and accumulated policy exist in source, not by the README aspiration alone.

## Score Summary

```text
K  = 8.5 / 10
C  = 7.6 / 10
A  = 7.1 / 10
R  = 8.4 / 10

OB = 4.4 / 10
CX = 5.4 / 10
ME = 5.8 / 10
PL = 5.6 / 10
LL = 6.9 / 10
JG = 5.6 / 10
TO = 8.2 / 10
VF = 8.0 / 10
EV = 6.1 / 10
PO = 7.7 / 10
LE = 6.4 / 10
OR = 6.6 / 10

CORE = 7.88 / 10
CAP  = 6.30 / 10
IMPL = 7.71 / 10
ARCH = 6.66 / 10

max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR) = K = 8.5 / 10 = good
```

## Static Review Inputs

```text
source_review_scope = README.md + Cargo.toml + src/**/*.rs + state/rustc/ai/graph.json + rubric/score.md
cargo_build_status = not_run_by_instruction
cargo_test_status = not_run_by_instruction
unsafe_policy = forbid_unsafe_code
cargo_dependencies = none

source_files_reviewed = 50 rust files excluding backups
source_loc_reviewed = 12538
backup_rust_files_present = 4
test_count_in_src_lib = 97

readme_goal = reduce_cost_of_autonomous_reasoning_while_increasing_quality_and_trustworthiness
readme_core_claim = frozen_kernel_plus_capability_intelligence_plus_tlog_policy_learning
readme_status_accuracy = stale_because_status_claims_capability_layer_is_defined_but_not_implemented

graph_schema_version = 9
graph_node_count = 1898
graph_edge_count = 5106
graph_cfg_node_count = 6926
graph_cfg_edge_count = 9412
graph_bridge_edge_count = 1716
graph_redundant_path_pair_count = 592
graph_alpha_pathway_count = 15
graph_function_count = 527
graph_function_intent_known = 135
graph_function_intent_low_confidence = 392
graph_function_intent_coverage = 25.62_percent

kernel_phase_count = 12
execution_gate_count = 7
total_gate_count = 8
tlog_schema_version = 5
api_protocol_schema_version = 3
```

## Implemented Capability Evidence

```text
tooling_record_split = present
tooling_execution_normal_form = present_for_tooling_only
tooling_effect_shape = Effect { kind, digest, metadata }
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
ollama_duplicate_proof_event_rejection = absent

llm_retry_budget_ledger = absent
llm_duplicate_call_idempotency = absent
constrained_decoding = absent
streaming_llm_ingress = absent
real_observation_stream_parser = absent
external_api_tool_runner = absent
external_artifact_verification = absent
distributed_orchestration = absent
generic_verification_proof_record = absent
```

## Critical Judgment

The architecture is still directionally correct. The frozen kernel, hash-chained TLog, evidence gates, recovery policy, command ledger, registry projection, policy store, and replay verifier form a real deterministic runtime core. The project is no longer only a sketch.

The current strongest implemented surface is local tooling. The split under `capability/tooling/record/` is meaningful: artifact effects, process effects, request construction, shared effect typing, receipts, hashing, and replay are separated. The code now contains the correct primitive for the next layer:

```text
request → authorize → execute → Effect { kind, digest, metadata } → receipt → tlog
```

The hard criticism is that this normal form is not universal yet. Tooling has it. Ollama has a parallel receipt/proof shape. Verification has profile/repair-specific records. API commands and runtime events are close, but not collapsed into one generic effect/proof contract. This means the system has several adjacent correctness mechanisms instead of one canonical correctness mechanism.

The Ollama path improved substantially. It now binds local endpoint provenance, request/response hashes, raw response hash, prompt hash, token count, command hash, event sequence, event hash, proof hash, and proof-event sequence. It also persists a typed proof event in mixed NDJSON and verifies proof ordering against the control-event stream. That is real provenance work.

The main remaining hole in the Ollama proof path is exact uniqueness. The source verifies that each proof event matches some receipt and that proof ordering is valid, but it does not yet enforce:

```text
receipt_hash → exactly_one_proof_event
```

That should be the next small correctness patch because it closes replay ambiguity without adding architecture.

Current classification:

```text
current_system = deterministic evidence runtime with bounded local sandbox execution and local Ollama proof receipts
not_yet = autonomous self-improving agent with live observation, external actions, semantic truth verification, and adaptive policy growth
main_strength = frozen kernel boundary + replayable TLog + receipted local effects + local LLM provenance
main_weakness = fragmented proof semantics and no live external observation/action loop
```

## Module Rating Table

| Module                     | Status                              | Score | Reason                                                                                                                                                                                                                                                                                |
|----------------------------+-------------------------------------+-------+---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `kernel`                   | strong deterministic core           |   8.5 | Clean phase/gate/state/evidence model, compact packet boundary, recovery classes, registry projection, and single hash primitive. Penalized for prototype packet semantics and narrow effect vocabulary.                                                                              |
| `codec`                    | useful durable codec                |   7.6 | NDJSON TLog, registry projection, and mixed receipt/proof encoding are real. Penalized for manual schema evolution, integer-field brittleness, and limited migration scaffolding.                                                                                                     |
| `api`                      | credible in-process command surface |   7.1 | Command protocol supports evidence submissions and process receipts with command hash binding. Still lacks network service, authentication, authorization scopes, hostile-client hardening, and streaming ingress.                                                                    |
| `runtime`                  | strong replay engine                |   8.4 | Tick, durable run, transition legality, convergence, writer, command ledger, and replay verification are coherent. Penalized because live capability semantics are not uniformly native.                                                                                              |
| `capability/observation`   | ordered frame facade                |   4.4 | Observation records and cursor replay rejection exist. Still lacks real SSE/webhook/browser/file stream parsing, cursor persistence, and backpressure.                                                                                                                                |
| `capability/context`       | deterministic context seed          |   5.4 | Context records can submit evidence. Still lacks retrieval, grounding, conflict handling, token budgeting, and source selection.                                                                                                                                                      |
| `capability/memory`        | deterministic lookup seed           |   5.8 | Memory facts/indexing exist and deterministic lookup is tested. Still lacks durable namespaces, embeddings, decay, invalidation, provenance, and cross-run query planning.                                                                                                            |
| `capability/planning`      | typed plan evidence                 |   5.6 | Planning records can drive gates. Still not a planner with dependency solving, risk/cost tradeoffs, repair search, or alternatives.                                                                                                                                                   |
| `capability/llm`           | local Ollama request/proof path     |   6.9 | OpenAI-compatible Ollama request construction, response parsing, local endpoint provenance, receipt replay, proof events, and proof sequence binding exist. Still lacks retry/budget/idempotency, constrained decoding, streaming, model policy, and duplicate proof-event rejection. |
| `capability/judgment`      | minimal judgment evidence           |   5.6 | Judgment is represented as typed evidence. Still lacks alternatives, irreversible-boundary checks, confidence calibration, and policy-backed deliberation.                                                                                                                            |
| `capability/tooling`       | strongest live capability           |   8.2 | File and process effects are split, bounded, authorized, receipted, API-visible, registry-bound, and replay-checkable. Still lacks external API tools, signed provider receipts, and universal effect routing.                                                                        |
| `capability/verification`  | strong internal verifier base       |   8.0 | Receipt/profile verification is meaningful and tied to tooling and lineage. Still mostly verifies internal artifacts and receipts, not external semantic truth.                                                                                                                       |
| `capability/eval`          | solid record scorer                 |   6.1 | Eval records and gate-driving evidence exist. Still lacks calibrated benchmarks, adversarial scoring, evaluator provenance, and threshold governance.                                                                                                                                 |
| `capability/policy`        | strong policy foundation            |   7.7 | Durable policy store, hashing, promotion, feedback, and registry binding exist. Still needs conflict resolution, expiry, rollback, signatures, and migrations.                                                                                                                        |
| `capability/learning`      | real but narrow                     |   6.4 | Learning can promote from TLog into policy. Still lacks causal attribution, pattern mining, strategy synthesis, and automatic capability generation.                                                                                                                                  |
| `capability/orchestration` | meaningful ordering layer           |   6.6 | Capability routing/submission order is represented. Still lacks distributed workers, leases, queues, priorities, retry policy, and backpressure.                                                                                                                                      |

## Structural Graph Judgment

```text
graph_growth = implementation_mass_increased
graph_debt = high
main_graph_signal = 592 redundant path pairs + 15 alpha pathways
intent_totalization_status = weak
intent_coverage = 135 / 527 functions = 25.62_percent
low_confidence_intents = 392 / 527 functions = 74.38_percent
```

The graph confirms that the codebase has real implementation mass, but semantic totalization is still weak. The wrapper sees only 25.62% confident function intent coverage. That means the source is still harder for the agent to reason about than it should be.

The 15 alpha pathways are not catastrophic, but they are a useful next cleanup target. Many are thin wrapper chains such as `EvidenceProducer::submission → Record::submission`, `durable_replay_report → replay_report_ndjson`, and adapter convenience calls. Do not chase all 592 redundant paths blindly. First remove wrapper ambiguity where it hides authority boundaries, receipt construction, or replay semantics.

## Artifact Judgment

```text
README = architecturally right_but_current_status_stale
score_md_before_update = stale_graph_stats_and_missing_duplicate_proof_event_assessment
src = real deterministic runtime prototype with local live effects and local Ollama proof receipts
graph = useful but shows rising semantic debt
kernel = strongest module
tooling = strongest capability module
llm = improved but still missing duplicate proof uniqueness and retry budget
```

The project has crossed the boundary from conceptual kernel prototype into deterministic local-effect runtime. It has not crossed into autonomous agent. The difference is external closure: real observation ingress, authenticated external actions, semantic verification, and adaptive policy updates from empirical outcomes are still missing.

## Regression / Improvement Delta

```text
previous_CORE = 7.85 / 10
current_CORE  = 7.88 / 10

previous_CAP = 6.23 / 10
current_CAP  = 6.30 / 10

previous_IMPL = 7.69 / 10
current_IMPL  = 7.71 / 10

previous_ARCH = 6.60 / 10
current_ARCH  = 6.66 / 10
```

The score increases slightly because the extracted source contains stronger LLM provenance and proof-ordering mechanics than the prior score text fully credited. The increase is capped because proof uniqueness is still missing, graph debt increased, README status is stale, and no cargo build/test was run in this review cycle by instruction.

## Highest Leverage Next Work

1. **Reject duplicate Ollama proof events for the same receipt hash.** Enforce `receipt_hash → exactly_one_proof_event` in both in-memory and NDJSON proof verification.
2. **Promote Ollama proof shape into a generic verification-proof record.** Reuse one proof contract for tooling, LLM, semantic verification, and future providers.
3. **Canonicalize one universal execution normal form.** Collapse artifact, process, LLM, and proof outputs into: `request → authorize → execute → Effect { kind, digest, metadata } → receipt → proof → TLog → replay`.
4. **Add retry/budget/idempotency policy for Ollama.** Bind timeout, retry count, call identity, and budget exhaustion into receipts.
5. **Add live observation ingress.** Implement one real source with cursor persistence and bounded backpressure.
6. **Reduce graph debt surgically.** Start with 15 alpha pathways and high-authority wrapper chains, not every redundant path.
7. **Update README current status.** Keep the architecture, but make present/absent boundaries exact.
8. **Do not expand the kernel.** Keep intelligence, providers, live effects, and learning pressure in capabilities and policy.

## Updated Verdict

```text
objective_rating = ARCH = 6.66 / 10
system_level = deterministic evidence runtime with bounded local sandbox execution and durable local Ollama effect proofs
best_property = kernel/runtime replay discipline plus split, receipted file/process/LLM effects
weakest_property = no generic proof abstraction, duplicate proof uniqueness, retry/budget ledger, or live observation/action loop yet
next_score_unlock = duplicate proof rejection + generic proof records + universal execution normal form + first real observation ingress
```

The project is moving correctly. The next win should be small and formal: reject duplicate proof events. That closes a precise replay ambiguity and strengthens the receipt/proof lattice without expanding scope.
