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

One-line explanation: the README goal is correct-by-construction autonomy through a frozen kernel, replayable TLog, growing policy, and capability-layer intelligence; the score rewards implemented evidence paths and penalizes every missing live loop, semantic uncertainty, and duplicated execution concept.

## Score Summary

```text
K  = 8.3 / 10
C  = 7.5 / 10
A  = 6.8 / 10
R  = 8.1 / 10

OB = 4.4 / 10
CX = 5.2 / 10
ME = 5.6 / 10
PL = 5.4 / 10
LL = 4.9 / 10
JG = 5.4 / 10
TO = 8.2 / 10
VF = 7.3 / 10
EV = 6.0 / 10
PO = 7.5 / 10
LE = 6.2 / 10
OR = 6.4 / 10

CORE = 7.65 / 10
CAP  = 5.95 / 10
IMPL = 7.46 / 10
ARCH = 6.33 / 10

max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR) = K = 8.3 / 10 = good
```

## Static Review Inputs

```text
source_review_scope = README.md + Cargo.toml + src/**/*.rs + state/rustc/ai/graph.json + rubric/score.md
cargo_build_status = not_run_by_instruction
cargo_test_status = not_run_by_instruction
unsafe_policy = forbid_unsafe_code

readme_goal = reduce_cost_of_autonomous_reasoning_while_increasing_quality_and_trustworthiness
readme_core_claim = frozen_kernel_plus_capability_intelligence_plus_tlog_policy_learning
readme_status_accuracy = directionally_right_but_ahead_of_current_external_world_capability

source_files_reviewed = 49 rust files
source_lines_reviewed = 10340
source_pub_struct_count = 53
source_pub_enum_count = 28
source_pub_fn_count = 244
source_impl_count = 100
test_count_in_src_lib = 86
src_unwrap_count = 167
cargo_dependencies = none

graph_schema_version = 9
graph_node_count = 1649
graph_edge_count = 4188
graph_cfg_node_count = 5355
graph_cfg_edge_count = 7139
graph_bridge_edge_count = 1475
graph_redundant_path_pair_count = 528
graph_alpha_pathway_count = 14
graph_fn_node_count = 451
graph_known_intent_fn_count = 109
graph_unknown_low_confidence_fn_count = 342
graph_intent_coverage = 24.17_percent
graph_low_confidence_fn_rate = 75.83_percent
graph_manifest_status_low_confidence = 451_of_451_fn_nodes

top_redundant_path_owner = runtime::verify::validate_event
top_redundant_path_owner_pairs = 272
second_redundant_path_owner = runtime::diff::semantic_diff
second_redundant_path_owner_pairs = 72

kernel_phase_count = 12
execution_gate_count = 7
total_gate_count = 8
tlog_schema_version = 5

effect_normal_form_type = present_as_tooling_record_types_Effect
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
tooling_api_process_receipt_route = receipt_submission_only

provider_backed_llm_client = absent
real_network_api_server = absent
real_observation_stream_parser = absent
external_api_tool_runner = absent
external_artifact_verification = absent
distributed_orchestration = absent
```

## Critical Judgment

The README states the correct architecture: the kernel must remain deterministic and small, while intelligence grows in capabilities, policy, learning, and replayable evidence. The source now contains enough implementation mass to be judged as a real deterministic runtime prototype, not a paper architecture.

The strongest implemented surface is still the deterministic kernel/runtime/tooling chain. The project has a frozen-style kernel boundary, typed gates, NDJSON TLog, durable replay, policy hashing, a capability registry, sandbox file/process effects, command allowlists, locked environments, output digests, timeout handling, and receipt replay. That is substantial.

The hard criticism is semantic authority. The graph has 451 function nodes, but only 109 have non-unknown intent classes. All 451 function semantic manifests are marked low confidence. That means the graph is useful for structural review, but not yet reliable enough to serve as a high-trust semantic oracle for autonomous repair or policy generation.

Current classification:

```text
current_system = deterministic evidence runtime with bounded local sandbox tooling
not_yet = autonomous self-improving agent with live external observation, provider LLM execution, and real external action loops
main_strength = kernel/runtime replay discipline plus bounded receipted file/process effects
main_weakness = semantic graph confidence and external-world capability are both too low
```

## Module Rating Table

| Module                     | Status                              | Score | Reason |
|----------------------------|-------------------------------------|-------|--------|
| `kernel`                   | strong deterministic core           |   8.3 | Clean phase/gate/state/evidence model, deterministic hashing, recovery classes, registry projection, and execution gate discipline. Penalized because the kernel-freeze rule is architectural rather than mechanically enforced and some semantics still depend on broad packet/evidence conventions. |
| `codec`                    | useful durable codec                |   7.5 | NDJSON TLog and receipt/policy encoding are real. Penalized for hand-rolled schema evolution, integer conversion tables, migration brittleness, and high redundant-path concentration in codec decoding functions. |
| `api`                      | in-process command surface          |   6.8 | Command protocol supports evidence submissions, batches, and process receipt submission. Penalized because it is not yet a hardened network API and does not route live execution requests through a complete request-authorize-execute-receipt path. |
| `runtime`                  | strong replay engine                |   8.1 | Tick, durable run, transition legality, writer, command ledger, and replay verification are coherent. Penalized because `runtime::verify::validate_event` owns the largest redundant-path cluster and the live capability loop is still narrow. |
| `capability/observation`   | typed observation records           |   4.4 | Observation frames, cursors, and bounded payload hashes exist. Penalized because there is no real SSE/webhook/browser/file ingress runner, durable cursor service, or backpressure loop. |
| `capability/context`       | deterministic context seed          |   5.2 | Context can produce evidence. Penalized because retrieval, grounding, conflict detection, source ranking, and token budget control are absent. |
| `capability/memory`        | deterministic memory seed           |   5.6 | Memory facts and store mechanics exist. Penalized because there are no embeddings, namespaces, decay, invalidation, provenance search, or cross-run query planning. |
| `capability/planning`      | typed plan evidence                 |   5.4 | Planning records can drive gates. Penalized because it is not yet a planner with search, alternatives, dependency solving, cost/risk tradeoffs, or repair strategy selection. |
| `capability/llm`           | structured record adapter           |   4.9 | LLM records and adapter-style structures exist. Penalized because no provider-backed client, retries, streaming parser, constrained decoding, budget ledger, or model routing policy exists. |
| `capability/judgment`      | minimal judgment evidence           |   5.4 | Judgment is represented as typed evidence. Penalized because it lacks alternative comparison, irreversible-boundary handling, calibrated confidence, and policy-backed deliberation. |
| `capability/tooling`       | strongest live capability           |   8.2 | File/process effects are split, bounded, authorized, hashed, receipted, and replay-checkable; `Effect { kind, digest, metadata }` now exists. Penalized because artifact and process receipts still remain parallel concepts and external API tools are absent. |
| `capability/verification`  | strong internal verifier base       |   7.3 | Receipt/profile verification is meaningful and tied to tooling. Penalized because it mostly verifies internal artifacts and process receipts, not semantic truth against external reality. |
| `capability/eval`          | solid record scorer                 |   6.0 | Eval records and gate-driving evidence exist. Penalized because there are no calibrated benchmarks, adversarial tests, evaluator provenance chains, or threshold governance. |
| `capability/policy`        | strong policy foundation            |   7.5 | Durable policy store, hashing, promotion, feedback, and registry binding exist. Penalized because conflict resolution, expiry, rollback, signatures, and migrations are still missing. |
| `capability/learning`      | real but narrow                     |   6.2 | Learning can promote from TLog into policy. Penalized because causal attribution, pattern mining, strategy synthesis, and automatic capability generation are absent. |
| `capability/orchestration` | meaningful ordering layer           |   6.4 | Capability routing and submission order are represented. Penalized because there are no distributed workers, leases, queues, priorities, retry policy, or backpressure system. |

## Artifact Judgment

```text
README = architecturally right_but_status_should_be_more_precise
score_md_before_update = too_optimistic_about_semantic_graph_reliability
src = substantial deterministic prototype_with_local_live_effects
graph = structurally useful_but_semantically_low_confidence
kernel = strongest_module
tooling = strongest_capability_module
```

The graph confirms both progress and debt. The project has 1649 semantic nodes, 4188 semantic edges, 5355 CFG nodes, and 7139 CFG edges. That is enough implementation mass to evaluate. The penalty is that 528 redundant path pairs and 14 alpha pathways show structural duplication, while 75.83% unknown-low-confidence function intent shows semantic extraction is not yet authoritative.

The main architecture risk is not lack of code. The risk is premature trust in weak semantic labels. A self-improving system that uses this graph for repair decisions must first make intent classification reliable, or it will promote noisy patterns into policy.

## Regression / Improvement Delta

```text
previous_CORE = 7.77 / 10
current_CORE  = 7.65 / 10

previous_CAP = 6.03 / 10
current_CAP  = 5.95 / 10

previous_IMPL = 7.58 / 10
current_IMPL  = 7.46 / 10

previous_ARCH = 6.43 / 10
current_ARCH  = 6.33 / 10
```

The score decreases slightly because this review applied stricter graph-based criticism. The implementation did not visibly regress; the rubric became more exact about semantic confidence. Tooling remains strong, but the graph cannot yet carry high-trust autonomous interpretation.

## Highest Leverage Next Work

1. **Make semantic graph confidence first-class.** Treat unknown intent and low-confidence manifests as blockers for autonomous repair decisions.
2. **Totalize function intent classification.** Move from 24.17% useful function intent coverage toward 100%, with confidence thresholds and manual override records.
3. **Canonicalize receipt verification under one effect contract.** `Effect { kind, digest, metadata }` exists; now collapse artifact/process verification around that normal form.
4. **Make API tooling execution real, not receipt-only.** Add a command path for authorized execution requests that produces receipts, instead of only accepting submitted process receipts.
5. **Reduce `runtime::verify::validate_event` redundancy.** It owns 272 redundant-path counts and should be split or table-driven.
6. **Implement one real observation ingress.** Add a durable source cursor, bounded frame reader, replayable observation record, and backpressure behavior.
7. **Add provider-backed LLM capability.** Keep typed records, but add real provider execution, retry/budget policy, structured-output validation, and provider receipt evidence.
8. **Update README implementation status.** Preserve the goal, but explicitly separate implemented local deterministic runtime from absent external-world autonomy.

## Updated Verdict

```text
objective_rating = ARCH = 6.33 / 10
system_level = deterministic evidence runtime with bounded local sandbox execution
best_property = deterministic kernel/runtime/tooling receipt discipline
weakest_property = semantic graph confidence and live external-loop absence
next_score_unlock = reliable semantic graph intent + unified Effect receipt verification + API-routed live execution
```

The project is still moving in the right direction. The correct next move is not more kernel complexity. The next move is to make the semantic graph reliable enough to guide repair, then route real API execution and live observation through the existing deterministic receipt discipline.