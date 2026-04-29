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
C  = 7.5 / 10
A  = 6.9 / 10
R  = 8.3 / 10

OB = 4.2 / 10
CX = 5.3 / 10
ME = 5.7 / 10
PL = 5.5 / 10
LL = 5.0 / 10
JG = 5.5 / 10
TO = 8.1 / 10
VF = 7.5 / 10
EV = 6.1 / 10
PO = 7.7 / 10
LE = 6.4 / 10
OR = 6.6 / 10

CORE = 7.77 / 10
CAP  = 6.03 / 10
IMPL = 7.58 / 10
ARCH = 6.43 / 10

max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR) = K = 8.5 / 10 = good
```

## Static Review Inputs

```text
source_review_scope = README.md + Cargo.toml + src/**/*.rs + state/rustc/ai/graph.json + rubric/score.md
cargo_build_status = not_run_by_instruction
cargo_test_status = not_run_by_instruction
unsafe_policy = forbid_unsafe_code

readme_goal = reduce_cost_of_autonomous_reasoning_while_increasing_quality_and_trustworthiness
readme_core_claim = frozen_kernel_plus_capability_intelligence_plus_tlog_policy_learning
readme_status_accuracy = partially_stale_because_capability_layer_has_live_local_tooling_but_not_external_autonomy

source_files_reviewed = 43 rust files
test_count_in_src_lib = 86
cargo_dependencies = none

graph_schema_version = 9
graph_node_count = 1649
graph_edge_count = 4188
graph_cfg_node_count = 5355
graph_cfg_edge_count = 7139
graph_bridge_edge_count = 1475
graph_redundant_path_pair_count = 528
graph_alpha_pathway_count = 14

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

provider_backed_llm_client = absent
real_observation_stream_parser = absent
external_api_tool_runner = absent
external_artifact_verification = absent
distributed_orchestration = absent
```

## Critical Judgment

The README states the real goal clearly: preserve a frozen correctness kernel while intelligence accumulates in capabilities and policy. That goal is still the right target. The current source now partially serves that goal instead of merely describing it.

The main improvement since the last score is that tooling is no longer a monolithic placeholder. `capability/tooling/record.rs` has been split into typed submodules for artifact effects, process effects, request construction, receipts, hashing, and shared types. Process effects are also visible at the API protocol layer through a process receipt command route. That removes one of the previous highest-leverage cleanup items.

The hard criticism is that the system is still local-effect capable, not autonomous in the README sense. It can authorize, execute, receipt, encode, and replay local file/process effects. It still does not observe live external streams, call provider-backed LLMs, authenticate external APIs, verify semantic truth outside its own receipts, or synthesize broad learned strategy from empirical history.

Current classification:

```text
current_system = deterministic evidence runtime with bounded local sandbox tooling
not_yet = autonomous self-improving agent with real external observation/action loops
main_strength = frozen kernel boundary + replayable TLog + bounded receipted file/process effects
main_weakness = capability intelligence is mostly typed records rather than live adaptive behavior
```

## Module Rating Table

| Module                     | Status                              | Score | Reason                                                                                                                                                                                                  |
|----------------------------|-------------------------------------|-------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `kernel`                   | strong deterministic core           |   8.5 | Clean phase/gate/state/evidence model, compact packet boundary, recovery classes, registry projection, and deterministic hashing. Penalized for prototype packet semantics and narrow effect vocabulary. |
| `codec`                    | useful durable codec                |   7.5 | NDJSON TLog, registry projection, and receipt encoding are real. Penalized for manual schema evolution, integer-field brittleness, and limited migration scaffolding.                                  |
| `api`                      | credible in-process command surface |   6.9 | Command protocol supports evidence submissions and process receipts. Still lacks network service, authentication, authorization scopes, hostile-client hardening, and streaming ingress.                 |
| `runtime`                  | strong replay engine                |   8.3 | Tick, durable run, transition legality, convergence, writer, command ledger, and replay verification are coherent. Penalized because live capability semantics are still not uniformly native.          |
| `capability/observation`   | typed evidence facade               |   4.2 | Observation records exist, but no real SSE/webhook/browser/file stream parser, cursor persistence, or backpressure system exists.                                                                         |
| `capability/context`       | deterministic context seed          |   5.3 | Context records can submit evidence. Still lacks retrieval, grounding, conflict handling, token budgeting, and source selection.                                                                         |
| `capability/memory`        | deterministic lookup seed           |   5.7 | Memory facts/indexing exist. Still lacks durable namespaces, embeddings, decay, invalidation, provenance, and cross-run query planning.                                                                  |
| `capability/planning`      | typed plan evidence                 |   5.5 | Planning records can drive gates. Still not a planner with dependency solving, risk/cost tradeoffs, repair search, or alternatives.                                                                      |
| `capability/llm`           | structured adapter mock             |   5.0 | LLM request/response records exist. No provider-backed client, retries, streaming parser, constrained decoding, budget ledger, or model policy.                                                          |
| `capability/judgment`      | minimal judgment evidence           |   5.5 | Judgment is represented as typed evidence. Still lacks comparison of alternatives, irreversible-boundary checks, and policy-backed deliberation.                                                         |
| `capability/tooling`       | strongest live capability           |   8.1 | File and process effects are split, bounded, authorized, receipted, API-visible, and replay-checkable. Still lacks external API tools, signed provider receipts, and universal effect routing.          |
| `capability/verification`  | strong internal verifier base       |   7.5 | Receipt/profile verification is meaningful and tied to tooling. Still mostly verifies internal artifacts and process receipts, not external semantic truth.                                              |
| `capability/eval`          | solid record scorer                 |   6.1 | Eval records and gate-driving evidence exist. Still lacks calibrated benchmarks, adversarial scoring, evaluator provenance, and threshold governance.                                                    |
| `capability/policy`        | strong policy foundation            |   7.7 | Durable policy store, hashing, promotion, feedback, and registry binding exist. Still needs conflict resolution, expiry, rollback, signatures, and migrations.                                           |
| `capability/learning`      | real but narrow                     |   6.4 | Learning can promote from TLog into policy. Still lacks causal attribution, pattern mining, strategy synthesis, and automatic capability generation.                                                     |
| `capability/orchestration` | meaningful ordering layer           |   6.6 | Capability routing/submission order is represented. Still lacks distributed workers, leases, queues, priorities, retry policy, and backpressure.                                                         |

## Artifact Judgment

```text
README = architecturally right but implementation-status stale
score_md_before_update = stale_after_tooling_split_and_process_receipt_api_route
src = substantially implemented deterministic prototype with local live effects
graph = useful but noisier than prior review; 528 redundant paths and 14 alpha pathways show rising structural debt
kernel = still the strongest module
tooling = strongest capability module
```

The graph confirms growth and debt at the same time. The project has grown to 1649 semantic nodes and 4188 semantic edges, plus 5355 CFG nodes and 7139 CFG edges. That is enough implementation mass to validate the architecture, but 528 redundant paths and 14 alpha pathways show that complexity is compounding again.

The biggest architectural risk is no longer the tooling split. That split exists. The risk is now semantic fragmentation: artifact receipts, process receipts, API commands, runtime events, verification, and policy binding are close to one execution normal form, but the code still names and routes them as adjacent concepts instead of one universal `Effect { kind, digest, metadata }` contract.

## Regression / Improvement Delta

```text
previous_CORE = 7.64 / 10
current_CORE  = 7.77 / 10

previous_CAP = 5.94 / 10
current_CAP  = 6.03 / 10

previous_IMPL = 7.42 / 10
current_IMPL  = 7.58 / 10

previous_ARCH = 6.33 / 10
current_ARCH  = 6.43 / 10
```

The score improves because the source now shows the tooling record split, stronger process effect routing, and API-visible process receipt submission. The increase is capped because no build/test run was performed in this review cycle, external-world capabilities are still absent, and graph debt increased.

## Highest Leverage Next Work

1. **Canonicalize one execution normal form.** Collapse artifact and process outputs into one universal path: `request → authorize → execute → Effect { kind, digest, metadata } → receipt → TLog → replay`.
2. **Unify receipt verification.** Remove parallel artifact/process verification concepts where possible and verify by effect kind under one receipt contract.
3. **Make API tooling execution real, not receipt-only.** The API can submit process receipts, but it should also route authorized execution requests through the same normal form.
4. **Add live observation ingress.** Implement one real observation source with cursor persistence and bounded backpressure; otherwise the agent cannot close the external-world loop.
5. **Add provider-backed LLM capability.** Keep records typed, but introduce a real adapter with retries, budget accounting, model policy, and structured-output validation.
6. **Reduce graph debt surgically.** Target the 14 alpha pathways and highest-frequency redundant path owners first. Do not optimize all 528 redundant paths blindly.
7. **Update README current status.** Keep the goal, but replace stale status language with exact present/absent implementation boundaries.
8. **Do not expand the kernel.** Preserve the frozen kernel; put live intelligence, external semantics, and learning pressure in capabilities and policy.

## Updated Verdict

```text
objective_rating = ARCH = 6.43 / 10
system_level = deterministic evidence runtime with bounded local sandbox execution
best_property = kernel/runtime replay discipline plus split, receipted file/process effects
weakest_property = no live external observation/action/LLM loop yet
next_score_unlock = universal execution normal form + API-routed live execution + first real observation ingress
```

The project is moving in the right direction. It now has enough implemented machinery to be judged as a real deterministic runtime prototype, not just a kernel sketch. It is still below autonomous-agent level because the live loop is local-only and the capability layer has typed intent without enough real-world effect.
