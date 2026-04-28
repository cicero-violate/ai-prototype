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

One-line explanation: geometric scoring rewards the deterministic core but sharply penalizes every declared capability that is still only typed evidence rather than live, replayable work.

## Score Summary

```text
K  = 8.5 / 10
C  = 7.4 / 10
A  = 6.6 / 10
R  = 8.2 / 10

OB = 4.2 / 10
CX = 5.3 / 10
ME = 5.7 / 10
PL = 5.4 / 10
LL = 5.0 / 10
JG = 5.4 / 10
TO = 7.7 / 10
VF = 7.3 / 10
EV = 6.0 / 10
PO = 7.6 / 10
LE = 6.3 / 10
OR = 6.5 / 10

CORE = 7.64 / 10
CAP  = 5.94 / 10
IMPL = 7.42 / 10
ARCH = 6.33 / 10

max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR) = K = 8.5 / 10 = good
```

## Static Review Inputs

```text
source_review_scope = README.md + Cargo.toml + src/**/*.rs + state/rustc/ai/graph.json + rubric/score.md
cargo_build_status = not_run_by_instruction
cargo_test_status = not_run_by_instruction
unsafe_policy = forbid_unsafe_code

readme_architecture_claim = kernel frozen, capability intelligence, TLog evidence, policy learning, LLM promoted to novelty
readme_current_status = stale_because_capability_layer_is_now_partially_implemented

source_files_reviewed = 43 rust files
test_count_in_src_lib = 85
cargo_dependencies = none

graph_schema_version = 9
graph_node_count = 1432
graph_edge_count = 3815
graph_cfg_node_count = 4951
graph_cfg_edge_count = 6541
graph_bridge_edge_count = 1399
graph_redundant_path_pair_count = 512
graph_alpha_pathway_count = 13

kernel_phase_count = 12
execution_gate_count = 7
total_gate_count = 8
tlog_schema_version = 5

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
tooling_api_execution = absent

provider_backed_llm_client = absent
real_observation_stream_parser = absent
external_artifact_verification = absent
distributed_orchestration = absent
```

## Critical Judgment

The repository is stronger than the README says. The README still claims the capability layer is only defined in structure and not yet implemented. That is now false. Tooling, verification, policy, learning, orchestration, memory, context, LLM records, planning records, judgment records, and observation records all exist as typed source modules. The capability layer is not complete, but it is no longer empty.

The highest-value improvement is still tooling. The code now contains a bounded local process executor with command allowlisting, cwd locking, environment locking, timeout handling, stdout/stderr capture, digests, exit status, output-size limits, effect receipts, and replay verification. That moves the system from artifact-writing simulation toward real sandboxed work.

The hard criticism is that integration is still not clean enough. Process execution exists, but it is not yet the canonical execution normal form across API, runtime, TLog, verification, and capability routing. The source has strong pieces, but the contract still feels partially bolted on rather than globally normalized.

Current classification:

```text
current_system = deterministic evidence-runtime with bounded local process tooling
not_yet = autonomous agent runtime with live external-world operation
main_strength = frozen kernel boundary + replayable TLog + receipted local effects
main_weakness = live capabilities are uneven and not all first-class in the canonical evidence path
```

The project has crossed an important threshold: it can represent and verify more than abstract state transitions. It can now bind local effects to receipts. It still cannot observe the world, call real providers, authenticate external APIs, perform semantic verification of external claims, or learn strategies from broad empirical evidence.

## Module Rating Table

| Module                     | Status                        | Score | Reason                                                                                                                                                                                                   |
|----------------------------+-------------------------------+-------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `kernel`                   | strong deterministic core     |   8.5 | Clean phase/gate/state/evidence model, compact packet boundary, recovery classes, registry projection, and deterministic hashing. Penalized for narrow effect vocabulary and prototype packet semantics. |
| `codec`                    | useful but manual             |   7.4 | NDJSON TLog and receipt encoding are real and durable. Penalized for hand-written schema evolution, integer-field brittleness, and no generated schema tests.                                            |
| `api`                      | credible in-process protocol  |   6.6 | Command/protocol/routes exist and are more coherent than a stub. Still lacks network service, authentication, authorization scopes, hostile-client hardening, and streaming ingress.                     |
| `runtime`                  | strong replay engine          |   8.2 | Tick, durable run, transition legality, convergence, writer, command ledger, and replay verification are coherent. Penalized for process/tool effects not being fully native end-to-end semantics.       |
| `capability/observation`   | typed evidence facade         |   4.2 | Observation records and cursor-style fields exist. Still no real SSE/webhook/file/browser stream parser or backpressure.                                                                                 |
| `capability/context`       | deterministic context seed    |   5.3 | Context records exist and can submit evidence. Still lacks retrieval, grounding, conflict handling, token budgeting, and source selection.                                                               |
| `capability/memory`        | deterministic lookup seed     |   5.7 | Memory facts/indexing exist. Still lacks durable namespaces, embeddings, decay, invalidation, provenance, and cross-run query planning.                                                                  |
| `capability/planning`      | typed plan evidence           |   5.4 | Planning records can drive gates. Still not a planner with dependency solving, risk/cost tradeoffs, repair search, or alternatives.                                                                      |
| `capability/llm`           | structured adapter mock       |   5.0 | LLM request/response records exist. No provider-backed client, retries, streaming parser, constrained decoding, budget ledger, or model policy.                                                          |
| `capability/judgment`      | minimal judgment evidence     |   5.4 | Judgment is represented as typed evidence. Still lacks comparison of alternatives, irreversible-boundary checks, and policy-backed deliberation.                                                         |
| `capability/tooling`       | best implemented capability   |   7.7 | File and process effects are bounded, authorized, receipted, and replay-checkable. Still lacks external API tools, signed provider receipts, and a single canonical execution normal form.               |
| `capability/verification`  | strong internal verifier base |   7.3 | Receipt/profile verification is meaningful and tied to tooling. Still mostly verifies internal artifacts and process receipts, not external semantic truth.                                              |
| `capability/eval`          | solid record scorer           |   6.0 | Eval records and gate-driving evidence exist. Still lacks calibrated benchmarks, adversarial scoring, evaluator provenance, and threshold governance.                                                    |
| `capability/policy`        | strong policy foundation      |   7.6 | Durable policy store, hashing, promotion, feedback, and registry binding exist. Still needs conflict resolution, expiry, rollback, signatures, and migrations.                                           |
| `capability/learning`      | real but narrow               |   6.3 | Learning can promote from TLog into policy. Still lacks causal attribution, pattern mining, strategy synthesis, and automatic capability generation.                                                     |
| `capability/orchestration` | meaningful ordering layer     |   6.5 | Capability routing/submission order is represented. Still lacks distributed workers, leases, queues, priorities, retry policy, and backpressure.                                                         |

## Artifact Judgment

```text
README = architecturally right but status-stale
score_md_before_update = mostly right but under-reported current graph/source scope
src = substantially implemented prototype, not just skeleton
graph = useful but noisy; 512 redundant path pairs and 13 alpha pathways show refactor debt
kernel = still the strongest module
tooling = strongest capability module
```

The graph confirms growth and also confirms debt. The system now has 1432 semantic nodes and 3815 semantic edges, plus 4951 CFG nodes and 6541 CFG edges. That is enough structure to justify the architecture, but the 512 redundant path pairs and 13 alpha pathways show that wrapper-visible duplication and pass-through call chains are accumulating again.

The biggest architectural risk is not the kernel. The kernel is still clean enough. The risk is that capability complexity grows faster than canonical evidence normalization. If each capability invents its own receipt semantics, replay remains technically possible but semantically fragmented.

## Regression / Improvement Delta

```text
previous_CORE = 7.58 / 10
current_CORE  = 7.64 / 10

previous_CAP = 5.81 / 10
current_CAP  = 5.94 / 10

previous_IMPL = 7.32 / 10
current_IMPL  = 7.42 / 10

previous_ARCH = 6.21 / 10
current_ARCH  = 6.33 / 10
```

The score improves because the reviewed source tree shows broader implemented capability modules than the README admits, and because tooling/process receipts are now materially stronger. The increase is capped because the live surfaces are still local-only, the README is stale, no build/test run was performed in this review cycle, and graph debt is visible.

## Highest Leverage Next Work

1. **Canonicalize execution normal form.** Enforce one path: `request → authorize → execute → Effect { kind, digest, metadata } → receipt → TLog`.
2. **Make process effects first-class everywhere.** API protocol, runtime evidence, verifier replay, TLog codec, and tooling should all recognize process receipts without artifact-shaped leakage.
3. **Split `capability/tooling/record.rs`.** It is 1752 lines and now holds too many responsibilities. Move file effects, process effects, receipt codecs, replay verification, and executor authorization into typed submodules.
4. **Reduce graph debt.** Target the 13 alpha pathways and highest-frequency redundant path owners first. Do not optimize all 512 pairs blindly.
5. **Update README current status.** Replace the stale “capability layer not yet implemented” statement with exact present/absent capability boundaries.
6. **Add focused process/tool tests if missing in user environment.** Cover allowlist deny, cwd escape, env injection deny, timeout kill, output cap, digest mismatch, receipt tamper, and replay mismatch.
7. **Do not expand the kernel.** Put intelligence and external-world semantics in capabilities, then collapse stable patterns into policy.

## Updated Verdict

```text
objective_rating = ARCH = 6.33 / 10
system_level = deterministic evidence runtime with partially live capability layer
best_property = kernel/runtime replay discipline plus bounded receipted local execution
weakest_property = capability semantics are growing faster than one canonical effect contract
next_score_unlock = single execution normal form + first-class process effect route + tooling module split
```

The correct next move is not more abstract architecture. The next move is normalization: make every tool output collapse into the same typed `Effect` shape, then bind that shape through receipt, TLog, replay, and verifier without side paths.
