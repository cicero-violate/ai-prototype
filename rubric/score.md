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

One-line explanation: the README goal is correct-by-construction autonomy through a frozen kernel, replayable TLog, growing policy, and capability-layer intelligence; the score rewards implemented evidence paths and penalizes every place where the system still cannot perform live autonomous work.

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
LL = 7.8 / 10
JG = 5.6 / 10
TO = 8.1 / 10
VF = 8.2 / 10
EV = 6.1 / 10
PO = 7.7 / 10
LE = 6.4 / 10
OR = 6.6 / 10

CORE = 7.87 / 10
CAP  = 6.48 / 10
IMPL = 7.72 / 10
ARCH = 6.82 / 10

max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR) = K = 8.5 / 10 = good
```

## Static Review Inputs

```text
source_review_scope = README.md + Cargo.toml + src/**/*.rs + state/rustc/ai/graph.json + rubric/score.md
cargo_build_status = passed_user_validated_2026_04_30
cargo_test_status = passed_user_validated_2026_04_30_99_99_after_observation_ingress_patch; current_source_adds_one_pending_generic_proof_spine_test
cargo_example_status = passed_user_validated_ollama_judgment_after_observation_ingress_patch
unsafe_policy = forbid_unsafe_code

readme_goal = reduce_cost_of_autonomous_reasoning_while_increasing_quality_and_trustworthiness
readme_core_claim = frozen_kernel_plus_capability_intelligence_plus_tlog_policy_learning
readme_status_accuracy = current_for_validated_local_tooling_and_ollama_but_external_autonomy_pending

source_files_reviewed = 51 rust files
test_count_in_src_lib = 100
cargo_dependencies = none

graph_schema_version = 9
graph_node_count = 2060
graph_edge_count = 5617
graph_cfg_node_count = 6926
graph_cfg_edge_count = 9412
graph_bridge_edge_count = 1716
graph_redundant_path_pair_count = 605
graph_alpha_pathway_count = 15
graph_intent_class_coverage = 575/575fn

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
llm_retry_budget_ledger = validated_by_cargo_test_and_example
ollama_timeout_retry_idempotency_receipt_binding = validated_by_cargo_test_and_example
ollama_tampered_fields_rejected = 17/17
ollama_durable_proof_verified = true
ollama_receipt_proof_matches = true
ollama_timeout_ms = 30000
ollama_retry_count = 0
ollama_max_retries = 0
ollama_attempt_budget = 1
ollama_budget_exhausted = true
ollama_duplicate_request = false
bounded_line_observation_ingress = validated_by_cargo_test
observation_cursor_persistence = validated_by_cargo_test
observation_backpressure = validated_by_cargo_test
generic_llm_verification_proof_projection = implemented_pending_cargo_validation
generic_verification_proof_binding_checker = implemented_pending_cargo_validation
real_observation_stream_parser = absent
external_api_tool_runner = absent
external_artifact_verification = absent
distributed_orchestration = absent
```

## Critical Judgment

The README states the real goal clearly: preserve a frozen correctness kernel while intelligence accumulates in capabilities and policy. That goal is still the right target. The current source now partially serves that goal instead of merely describing it.

The main improvement since the last score is the first source-level observation ingress. `capability/observation/source.rs` now implements a bounded append-only line source, cursor persistence, and explicit backlog backpressure without changing the frozen kernel. That directly attacks the weakest prior module while keeping world-facing bytes outside kernel state.

The hard criticism is that the system is still local-effect capable, not autonomous in the README sense. It can authorize, execute, receipt, encode, and replay local file/process effects, run a local Ollama/OpenAI-compatible request adapter for `qwen2.5-coder`, and now parse a cargo-validated bounded file-backed observation source. It still does not observe live SSE/webhook/browser streams, authenticate external APIs, verify semantic truth outside its receipt/proof envelope, or synthesize broad learned strategy from empirical history.

Current classification:

```text
current_system = deterministic evidence runtime with bounded local sandbox tooling, local Ollama request wiring, and validated file observation ingress
not_yet = autonomous self-improving agent with validated external observation/action loops
main_strength = frozen kernel boundary + replayable TLog + bounded receipted file/process effects
main_weakness = capability intelligence is mostly typed records rather than live adaptive behavior
```

## Module Rating Table

| Module                     | Status                              | Score | Reason                                                                                                                                                                                                  |
|----------------------------|-------------------------------------|-------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `kernel`                   | strong deterministic core           |   8.5 | Clean phase/gate/state/evidence model, compact packet boundary, recovery classes, registry projection, and deterministic hashing. Penalized for prototype packet semantics and narrow effect vocabulary. |
| `codec`                    | useful durable codec                |   7.5 | NDJSON TLog, registry projection, and receipt encoding are real. Penalized for manual schema evolution, integer-field brittleness, and limited migration scaffolding.                                  |
| `api`                      | credible in-process command surface |   6.9 | Command protocol supports evidence submissions and process receipts. Still lacks network service, authentication, authorization scopes, hostile-client hardening, and streaming ingress.                 |
| `runtime`                  | strong replay engine                |   8.5 | Tick, durable run, transition legality, convergence, writer, command ledger, proof-event order checks, and replay verification are coherent. Penalized because live capability semantics are still not uniformly native. |
| `capability/observation`   | validated bounded file ingress seed |   5.8 | Observation records and cursors exist, and the bounded line-file ingress is now cargo-validated for cursor persistence and explicit backlog backpressure. Still lacks real SSE/webhook/browser stream adapters and runtime/API routing from live external sources.                                                                         |
| `capability/context`       | deterministic context seed          |   5.3 | Context records can submit evidence. Still lacks retrieval, grounding, conflict handling, token budgeting, and source selection.                                                                         |
| `capability/memory`        | deterministic lookup seed           |   5.7 | Memory facts/indexing exist. Still lacks durable namespaces, embeddings, decay, invalidation, provenance, and cross-run query planning.                                                                  |
| `capability/planning`      | typed plan evidence                 |   5.5 | Planning records can drive gates. Still not a planner with dependency solving, risk/cost tradeoffs, repair search, or alternatives.                                                                      |
| `capability/llm`           | validated local Ollama proof path   |   7.8 | LLM records exist and local Ollama/OpenAI-compatible request construction is wired to `qwen2.5-coder`; receipts now validatedly bind endpoint provenance, tamper matrix, durable proof hash, proof-event sequence ordering, replay-level proof event position, timeout, retry count, maximum retries, attempt budget, duplicate request identity, retry budget hash, budget exhaustion, and receipt/proof matching. Still lacks streaming, constrained decoding, provider-signed receipts, and model policy. |
| `capability/judgment`      | minimal judgment evidence           |   5.6 | Judgment is represented as typed evidence and the local Ollama example reaches a validated judgment proof path. Still lacks comparison of alternatives, irreversible-boundary checks, and policy-backed deliberation. |
| `capability/tooling`       | strongest live capability           |   8.1 | File and process effects are split, bounded, authorized, receipted, API-visible, and replay-checkable. Still lacks external API tools, signed provider receipts, and universal effect routing.          |
| `capability/verification`  | strong internal verifier base       |   8.2 | Receipt/profile verification is meaningful across tooling and the validated Ollama proof path, including receipt/proof ordering and tamper rejection. Still mostly verifies internal artifacts and local/provider-adapter receipts, not external semantic truth. |
| `capability/eval`          | solid record scorer                 |   6.1 | Eval records and gate-driving evidence exist. Still lacks calibrated benchmarks, adversarial scoring, evaluator provenance, and threshold governance.                                                    |
| `capability/policy`        | strong policy foundation            |   7.7 | Durable policy store, hashing, promotion, feedback, and registry binding exist. Still needs conflict resolution, expiry, rollback, signatures, and migrations.                                           |
| `capability/learning`      | real but narrow                     |   6.4 | Learning can promote from TLog into policy. Still lacks causal attribution, pattern mining, strategy synthesis, and automatic capability generation.                                                     |
| `capability/orchestration` | meaningful ordering layer           |   6.6 | Capability routing/submission order is represented. Still lacks distributed workers, leases, queues, priorities, retry policy, and backpressure.                                                         |

## Artifact Judgment

```text
README = architecturally right and current status updated for validated bounded observation ingress plus validated local Ollama retry_budget proof
score_md_before_update = stale_before_validated_observation_ingress_run
src = substantially implemented deterministic prototype with local live effects and Ollama request construction
graph = useful but noisier than prior review; 604 redundant paths and 15 alpha pathways show rising structural debt
kernel = still the strongest module
tooling = strongest capability module
```

The graph confirms growth and debt at the same time. The latest validated capture reports 2060 semantic nodes, 5617 semantic edges, 605 redundant path pairs, 15 alpha pathways, and full function intent coverage at 575/575fn. That is enough implementation mass to validate the architecture, but the redundant path count shows that complexity is compounding again.

The biggest architectural risk is no longer the tooling split. That split exists. The risk is now semantic fragmentation: artifact receipts, process receipts, API commands, runtime events, verification, and policy binding are close to one execution normal form, but the code still names and routes them as adjacent concepts instead of one universal `Effect { kind, digest, metadata }` contract.

## Regression / Improvement Delta

```text
previous_CORE = 7.87 / 10
current_CORE  = 7.87 / 10

previous_CAP = 6.41 / 10
current_CAP  = 6.48 / 10

previous_IMPL = 7.72 / 10
current_IMPL  = 7.72 / 10

previous_ARCH = 6.75 / 10
current_ARCH  = 6.82 / 10
```

The score improves because the first bounded observation ingress surface is now cargo-validated: an append-only line-file source with cursor persistence and explicit backlog backpressure. The Ollama path remains the latest fully validated live effect path: it has a live external LLM effect receipt, replay verification, seventeen-field tamper rejection, local endpoint provenance, pre-receipt non-local rejection, a durable final proof event in the mixed tlog, bidirectional receipt/proof hashing, proof-event sequence binding, replay-level proof event position checks, and validated retry/budget/idempotency binding. The latest user-validated run passed `cargo build`, `cargo test` with 99/99 tests, and `cargo run --example ollama_judgment`; the example emitted `durable_proof_verified=true`, `receipt_proof_matches=true`, and `tampered_fields_rejected=17/17`.

Current unvalidated source delta: the Ollama proof event now projects into a generic `VerificationProofRecord`, and `verify_ollama_judgment_proof_events` routes through `verify_verification_proof_record_bindings`. This is a first concrete generic verification spine for LLM proof records, but it is intentionally not scored as cargo-validated until the next local run passes the new 100th test.

## Highest Leverage Next Work

1. **Validate generic LLM proof spine.** Run the new 100-test suite and the Ollama example to confirm `OllamaJudgmentProofEvent -> VerificationProofRecord -> VerificationProofBinding` remains replay-safe.
2. **Route observation ingress.** Expose the validated bounded line-file observation source through the API/runtime path so external observations can enter the Invariant gate without hand-built tests.
3. **Canonicalize one execution normal form.** Collapse artifact, process, LLM, observation, and proof outputs into one universal path: `request → authorize → execute → Effect { kind, digest, metadata } → receipt → proof → TLog → replay`.
4. **Extend generic proof records beyond LLM.** Apply the same `VerificationProofRecord` binding checker to tooling, process, semantic verification, and future providers.
5. **Add provider response streaming validation.** The current path validates the completed local response body, but streaming chunks are not yet typed, bounded, hashed, or replayed.
6. **Reduce graph debt surgically.** Target the 15 alpha pathways and highest-frequency redundant path owners first. Do not optimize all 604 redundant paths blindly.
7. **Do not expand the kernel.** Preserve the frozen kernel; put live intelligence, external semantics, and learning pressure in capabilities and policy.

## Updated Verdict

```text
objective_rating = ARCH = 6.82 / 10
system_level = deterministic evidence runtime with bounded local sandbox execution, validated durable local Ollama retry-budget proofs, and validated bounded file observation ingress
best_property = kernel/runtime replay discipline plus split, receipted file/process/LLM effects and validated Ollama receipt/proof binding
weakest_property = observation ingress is file-backed and not routed from live external streams; generic proof spine is only implemented for LLM and pending validation
next_score_unlock = validate generic LLM proof spine + route observation ingress + universal execution normal form
```

The project is moving in the right direction. It now has enough implemented machinery to be judged as a real deterministic runtime prototype, not just a kernel sketch. It is still below autonomous-agent level because the observation loop is only file-backed, the generic proof spine is not yet extended to every effect kind, and the capability layer still needs external action tools.
