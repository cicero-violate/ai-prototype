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
C  = 7.9 / 10
A  = 7.3 / 10
R  = 8.7 / 10

OB = 6.3 / 10
CX = 5.3 / 10
ME = 5.7 / 10
PL = 5.5 / 10
LL = 8.0 / 10
JG = 5.6 / 10
TO = 8.4 / 10
VF = 8.8 / 10
EV = 6.1 / 10
PO = 7.7 / 10
LE = 6.4 / 10
OR = 6.6 / 10

CORE = 8.08 / 10
CAP  = 6.60 / 10
IMPL = 7.92 / 10
ARCH = 6.95 / 10

max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR) = VF = 8.8 / 10 = good
```

## Static Review Inputs

```text
source_review_scope = README.md + Cargo.toml + src/**/*.rs + state/rustc/ai/graph.json + rubric/score.md
cargo_build_status = passed_user_validated_2026_05_01_after_canonical_execution_normal_form
cargo_test_status = passed_user_validated_103_103_after_canonical_execution_normal_form
cargo_example_status = passed_user_validated_ollama_judgment_after_canonical_execution_normal_form
unsafe_policy = forbid_unsafe_code

readme_goal = reduce_cost_of_autonomous_reasoning_while_increasing_quality_and_trustworthiness
readme_core_claim = frozen_kernel_plus_capability_intelligence_plus_tlog_policy_learning
readme_status_accuracy = current_for_validated_canonical_execution_normal_form_with_pending_authority_collapse

source_files_reviewed = 52 rust files
test_count_in_src_lib = 103
cargo_dependencies = none

graph_schema_version = 9
graph_node_count = 2321
graph_edge_count = 6566
graph_cfg_node_count = not_reported_in_latest_user_validation
graph_cfg_edge_count = not_reported_in_latest_user_validation
graph_bridge_edge_count = not_reported_in_latest_user_validation
graph_redundant_path_pair_count = 334
graph_redundant_path_pair_delta = 621 -> 497 -> 464 -> 349 -> 346 -> 326 -> 328 -> 323 -> 334
graph_alpha_pathway_count = 16
graph_alpha_pathway_delta = 16 -> 15 -> 16
graph_intent_class_coverage = 704/704fn
graph_unknown_low_confidence_functions = unknown_after_latest_user_validation
pending_patch = canonical_effect_route_authority_collapse
pending_patch_validation = not_yet_user_validated

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
observation_ingress_api_command_route = validated_by_cargo_test
api_protocol_schema_version = 4
generic_llm_verification_proof_projection = validated_by_cargo_test
generic_verification_proof_binding_checker = validated_by_cargo_test
generic_artifact_effect_verification_proof_projection = validated_by_cargo_test
generic_process_effect_verification_proof_projection = validated_by_cargo_test
generic_verification_proof_subject_adapter = validated_by_cargo_test
generic_policy_effect_verification_proof_projection = validated_by_cargo_test
canonical_effect_normal_form = validated_by_cargo_test_with_graph_regression
canonical_effect_receipt = validated_by_cargo_test_with_graph_regression
canonical_effect_proof = validated_by_cargo_test_with_graph_regression
ollama_canonical_effect_projection = validated_by_cargo_test_with_graph_regression
artifact_tooling_canonical_effect_projection = validated_by_cargo_test_with_graph_regression
process_tooling_canonical_effect_projection = validated_by_cargo_test_with_graph_regression
policy_canonical_effect_projection = validated_by_cargo_test_with_graph_regression
canonical_effect_route_authority_collapse = pending_local_validation
generic_verification_proof_replay_enforcement = validated_by_cargo_test
generic_verification_proof_order_ndjson = validated_by_cargo_test
generic_verification_proof_replay_ndjson = validated_by_cargo_test
verification_proof_spine_split = validated_by_cargo_test
runtime_validate_event_split = validated_by_cargo_test_and_graph_delta
runtime_validate_event_redundant_path_reduction = 124
semantic_diff_split = validated_by_cargo_test_and_graph_delta
codec_ndjson_enum_decoder_table_split = validated_by_cargo_test_and_graph_delta
ollama_mixed_ndjson_record_helper_collapse = validated_by_cargo_test_and_graph_delta
ollama_json_escape_split = validated_by_cargo_test_and_graph_delta
runtime_verify_tlog_from_split = pending_local_validation
generic_verification_proof_missing_duplicate_displaced_rejection = validated_by_cargo_test
real_observation_stream_parser = absent
external_api_tool_runner = absent
external_artifact_verification = absent
distributed_orchestration = absent
```

## Critical Judgment

The README states the real goal clearly: preserve a frozen correctness kernel while intelligence accumulates in capabilities and policy. That goal is still the right target. The current source now partially serves that goal instead of merely describing it.

The main improvement since the last score is that observation ingress API routing is now user-validated. `Command::SubmitObservationIngress(ObservationIngressBatch)` has a schema-v4 command hash, batch contract validation, aggregate ingress submission, and an API route that drives `Evidence::InvariantProof` through the runtime path. This keeps world-facing bytes outside kernel state while allowing source batches to enter the same command/TLog route as other evidence.

The newest user-validated delta proves the canonical execution normal form did
not break behavior: local validation reports `103/103` tests and a passing
Ollama judgment example. It also proves the implementation was additive rather
than authoritative: graph health regressed to `334` redundant path pairs and
`16` alpha pathways while intent stayed totalized at `704/704fn`. The current
patch therefore removes direct proof-subject routing and direct provider proof
record construction so `CanonicalEffectProof` becomes the only durable proof
record construction route.

The hard criticism is that the system is still local-effect capable, not autonomous in the README sense. It can authorize, execute, receipt, encode, and replay local file/process effects, run a local Ollama/OpenAI-compatible request adapter for `qwen2.5-coder`, validate the generic LLM proof spine, parse a cargo-validated bounded file-backed observation source, and route accepted observation batches through the API. It still does not observe live SSE/webhook/browser streams, authenticate external APIs, verify semantic truth outside its receipt/proof envelope, or synthesize broad learned strategy from empirical history.

Current classification:

```text
current_system = deterministic evidence runtime with bounded local sandbox tooling, local Ollama request wiring, validated file observation ingress, validated observation API route, validated tool/process proof projection, validated generic proof replay enforcement, validated generic proof-subject collapse, validated canonical execution normal form with graph regression, validated reduced-coupling runtime event verification, validated semantic-diff classification split, validated table-driven NDJSON enum decoding, validated Ollama mixed-record helpers, pending canonical effect route authority collapse, and validated Ollama JSON escaping helpers
not_yet = autonomous self-improving agent with validated external observation/action loops
main_strength = frozen kernel boundary + replayable TLog + bounded receipted file/process effects
main_weakness = capability intelligence is mostly typed records rather than live adaptive behavior
```

## Module Rating Table

| Module                     | Status                              | Score | Reason                                                                                                                                                                                                  |
|----------------------------|-------------------------------------|-------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `kernel`                   | strong deterministic core           |   8.5 | Clean phase/gate/state/evidence model, compact packet boundary, recovery classes, registry projection, and deterministic hashing. Penalized for prototype packet semantics and narrow effect vocabulary. |
| `codec`                    | stronger durable codec              |   7.9 | NDJSON TLog, registry projection, receipt encoding, and table-driven enum tag decoding are real and user-validated. Penalized for manual schema evolution, integer-field brittleness, and limited migration scaffolding. |
| `api`                      | credible in-process command surface |   7.3 | Command protocol supports evidence submissions, process receipts, and user-validated observation-ingress batch routing with schema-v4 command hashes. Still lacks network service, authentication, authorization scopes, hostile-client hardening, and streaming ingress. |
| `runtime`                  | strong replay engine                |   8.7 | Tick, durable run, transition legality, convergence, writer, command ledger, proof-event order checks, generic proof replay verification, decomposed event validators, and split semantic-diff classifiers are coherent. The debt passes reduced redundant paths without breaking tests. Penalized because live capability semantics are still not uniformly native. |
| `capability/observation`   | routed bounded file ingress seed    |   6.3 | Observation records, cursors, bounded line-file ingress, cursor persistence, backlog backpressure, batch contract validation, and API routing into the Invariant gate are user-validated. Still lacks real SSE/webhook/browser stream adapters. |
| `capability/context`       | deterministic context seed          |   5.3 | Context records can submit evidence. Still lacks retrieval, grounding, conflict handling, token budgeting, and source selection.                                                                         |
| `capability/memory`        | deterministic lookup seed           |   5.7 | Memory facts/indexing exist. Still lacks durable namespaces, embeddings, decay, invalidation, provenance, and cross-run query planning.                                                                  |
| `capability/planning`      | typed plan evidence                 |   5.5 | Planning records can drive gates. Still not a planner with dependency solving, risk/cost tradeoffs, repair search, or alternatives.                                                                      |
| `capability/llm`           | validated local Ollama proof path   |   8.0 | LLM records exist and local Ollama/OpenAI-compatible request construction is wired to `qwen2.5-coder`; receipts now validatedly bind endpoint provenance, tamper matrix, durable proof hash, proof-event sequence ordering, replay-level proof event position, timeout, retry count, maximum retries, attempt budget, duplicate request identity, retry budget hash, budget exhaustion, receipt/proof matching, generic proof projection, shared mixed-record IO helpers, and split JSON escaping helpers. Still lacks streaming, constrained decoding, provider-signed receipts, and model policy. |
| `capability/judgment`      | minimal judgment evidence           |   5.6 | Judgment is represented as typed evidence and the local Ollama example reaches a validated judgment proof path. Still lacks comparison of alternatives, irreversible-boundary checks, and policy-backed deliberation. |
| `capability/tooling`       | strongest live capability           |   8.4 | File and process effects are split, bounded, authorized, receipted, API-visible, replay-checkable, and now user-validated as generic proof-binding subjects. Still lacks external API tools and signed provider receipts. |
| `capability/verification`  | strong internal verifier base       |   8.8 | Receipt/profile verification is meaningful across tooling and the validated Ollama proof path, including receipt/proof ordering, tamper rejection, generic LLM proof checking, validated artifact/process proof binding, and cargo-validated generic proof replay enforcement for missing/duplicate/displaced proof records. Still mostly verifies internal artifacts and local/provider-adapter receipts, not external semantic truth. |
| `capability/eval`          | solid record scorer                 |   6.1 | Eval records and gate-driving evidence exist. Still lacks calibrated benchmarks, adversarial scoring, evaluator provenance, and threshold governance.                                                    |
| `capability/policy`        | strong policy foundation            |   7.7 | Durable policy store, hashing, promotion, feedback, and registry binding exist. Still needs conflict resolution, expiry, rollback, signatures, and migrations.                                           |
| `capability/learning`      | real but narrow                     |   6.4 | Learning can promote from TLog into policy. Still lacks causal attribution, pattern mining, strategy synthesis, and automatic capability generation.                                                     |
| `capability/orchestration` | meaningful ordering layer           |   6.6 | Capability routing/submission order is represented. Still lacks distributed workers, leases, queues, priorities, retry policy, and backpressure.                                                         |

## Artifact Judgment

```text
README = architecturally right and current status updated for validated generic LLM/tool/process proof spine, validated proof-subject collapse, validated canonical execution normal form with graph regression, validated observation API routing, validated generic proof replay enforcement, validated runtime event-validation split, validated semantic-diff split, validated NDJSON decoder table split, validated Ollama mixed-record helpers, pending canonical effect route authority collapse, and validated Ollama JSON escaping split
score_md_before_update = current_after_validated_canonical_execution_normal_form
src = substantially implemented deterministic prototype with local live effects, Ollama proof binding, validated observation routing, validated tool/process generic proof projection, validated generic proof replay enforcement, validated proof-subject collapse, validated canonical execution normal form with graph regression, validated reduced-coupling runtime event verification, validated semantic-diff classification, validated table-driven NDJSON enum decoding, validated Ollama mixed-record helpers, validated Ollama JSON escaping helpers, validated runtime replay-loop split, and pending canonical effect route authority collapse
graph = useful_latest_user_validated_capture; 334 redundant paths and 16 alpha pathways show the canonical execution normal form was additive and now needs authority collapse
kernel = still the strongest module
tooling = strongest capability module
```

The graph confirms that most debt-reduction moves worked and also exposes the
latest tradeoff. The latest user-validated capture reports 2321 semantic nodes,
6566 semantic edges, 334 redundant path pairs, 16 alpha pathways, and full
function intent coverage at 704/704fn. Tests stayed closed at `103/103`, but
the proof route became additive again; this patch makes the canonical route the
authority by removing direct proof-subject and direct provider proof-record
construction.
The validated semantic-debt passes split the generic proof spine out of the
semantic verification record module, decomposed `runtime::verify::validate_event`,
split `runtime::diff::semantic_diff`, collapsed repeated `codec::ndjson` enum
decoder match forests into canonical tag tables plus one lookup helper,
collapsed Ollama mixed-record IO duplication, split Ollama JSON escaping, and
split `verify_tlog_from` into replay-specific validators.

The biggest architectural risk is no longer the tooling split. That split exists. The risk is now whether the new `CanonicalEffect { kind, digest, metadata } -> CanonicalEffectReceipt -> CanonicalEffectProof -> VerificationProofRecord` route becomes the single route, or merely another adjacent proof path. The current patch removes the direct proof-subject trait path and removes direct Ollama proof-record construction, leaving provider-local proof fields as inputs to the canonical route rather than as an independent route.

## Regression / Improvement Delta

```text
previous_CORE = 8.06 / 10
current_CORE  = 8.08 / 10

previous_CAP = 6.60 / 10
current_CAP  = 6.60 / 10

previous_IMPL = 7.91 / 10
current_IMPL  = 7.92 / 10

previous_ARCH = 6.95 / 10
current_ARCH  = 6.95 / 10
```

The score is held flat for this patch because the canonical effect route
authority collapse is not locally cargo-validated. The prior canonical execution
normal form is credited for preserving behavior but criticized for graph-health
regression: alpha pathways moved `15 -> 16`, redundant path pairs moved
`323 -> 334`, and tests remained `103/103`. The Ollama path remains the latest fully validated live
effect path: it has a live external LLM effect receipt, replay verification,
seventeen-field tamper rejection, local endpoint provenance, pre-receipt
non-local rejection, a durable final proof event in the mixed tlog,
bidirectional receipt/proof hashing, proof-event sequence binding, replay-level
proof event position checks, validated retry/budget/idempotency binding,
validated projection into `VerificationProofRecord`, validated generic proof
replay rejection for missing, duplicate, and displaced proof records, validated
mixed-record IO helper collapse, and validated JSON escaping helper split. The
latest user-validated run passed `cargo build`, `cargo test` with 103/103 tests,
and `cargo run --example ollama_judgment`; the graph reported
`621 -> 497 -> 464 -> 349 -> 346 -> 326 -> 328 -> 323 -> 334` redundant path pairs and the
example emitted `durable_proof_verified=true`, `receipt_proof_matches=true`, and
`tampered_fields_rejected=17/17`.

Validated proof replay delta: `verify_verification_proof_record_replay`, `verify_verification_proof_record_order_ndjson`, and `verify_verification_proof_record_replay_ndjson` require proof records to match receipt bindings, require the receipt event to exist in the control TLog, and reject missing, duplicate, or displaced generic proof records in mixed NDJSON replay. The accepted path plus all three rejection classes are covered by the passing test suite. The proof-spine split moves those proof types and replay functions into `capability::verification::proof`, reducing coupling without changing the public verification exports. The runtime verifier split and semantic-diff split now keep the same behavior while breaking branch-dense logic into smaller validation/classification gates, and the graph confirms the intended redundant-path reduction.

Validated semantic-debt delta: `codec::ndjson` enum decoders now use canonical tag tables plus one `enum_from_u64` lookup helper instead of repeated match forests. `capability::llm::ollama` now also shares mixed-record NDJSON append/load helpers and splits JSON escaping into character and sequence helpers. These moves reduced duplicate branch signatures in the rustc graph capture while preserving every existing numeric tag, record schema, request JSON byte behavior, and failure path.

Pending authority-collapse delta: the generic proof spine now treats `CanonicalEffectProof` as the only durable proof-record construction route. Ollama, artifact tooling, process tooling, and policy promotion can still preserve provider/local receipt and proof data, but direct proof-subject and direct provider proof-record construction are removed. This patch is not yet counted in the score until local cargo validation confirms behavior and graph movement.

## Highest Leverage Next Work

1. **Validate the canonical effect authority collapse.** Run the local cargo/build/example loop and compare `T`, `ρ`, `α`, and `I` against `T=103/103`, `ρ=334`, `α=16`, `I=704/704`.
2. **Finish semantic verification and observation canonical effects.** Extend the same route beyond live LLM/tool/policy effects.
3. **Remove remaining compatibility-only proof helpers after validation.** Once the canonical path is proven, collapse remaining provider-local proof helpers into wrappers or delete them.
4. **Add an eval-backed semantic debt gate.** Fail or warn when graph health regresses without an explicit justification record.
5. **Add provider response streaming validation.** The current path validates the completed local response body, but streaming chunks are not yet typed, bounded, hashed, or replayed.
6. **Do not expand the kernel.** Preserve the frozen kernel; put live intelligence, external semantics, and learning pressure in capabilities and policy.

## Updated Verdict

```text
objective_rating = ARCH = 6.95 / 10
system_level = deterministic evidence runtime with bounded local sandbox execution, validated durable local Ollama retry-budget proofs, validated bounded file observation ingress, validated observation API routing, validated tool/process proof projection, validated generic proof replay enforcement, validated proof-subject collapse, validated canonical execution normal form with pending authority collapse, validated reduced-coupling runtime event verification, validated semantic-diff split, validated table-driven NDJSON decoding, validated Ollama mixed-record helper collapse, validated Ollama JSON escaping split, and validated runtime replay-loop split
best_property = kernel/runtime replay discipline plus split, receipted file/process/LLM effects, validated Ollama receipt/proof binding, generic proof replay hardening, and measurable graph-debt reduction from 621 to 334 despite the latest additive-route regression
weakest_property = observation remains file-backed and not connected to live external streams; graph debt is lower than peak but still nontrivial at 334 redundant path pairs, and canonical effect authority collapse needs validation plus semantic/observation migration
next_score_unlock = validate canonical effect authority collapse + migrate semantic/observation effects + add semantic debt eval gate + reduce redundant paths below 300 + provider streaming receipts
```

## Pending Patch: Canonical Effect Route Authority Collapse

```text
target = capability::verification::proof + capability::{llm,tooling,policy}
change = remove direct proof-subject and provider proof-record routes; force durable proof records through CanonicalEffectReceipt -> CanonicalEffectProof -> VerificationProofRecord
expected_effect = Ollama, artifact tooling, process tooling, and policy promotion keep local receipt fields but share one durable proof-record normal form without changing kernel state
validation_status = pending cargo build/test/example
```

The project is moving in the right direction. It now has enough implemented machinery to be judged as a real deterministic runtime prototype, not just a kernel sketch. It is still below autonomous-agent level because the observation loop is only file-backed, canonical effect authority collapse still needs validation plus semantic/observation migration, and the capability layer still needs external action tools.
