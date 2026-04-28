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

One-line explanation: geometric scoring punishes hollow autonomy claims; process-backed tooling raises the score only when execution is authorized, bounded, receipted, and replay-checkable.

## Score Summary

```text
K  = 8.5 / 10
C  = 7.3 / 10
A  = 6.4 / 10
R  = 8.3 / 10

OB = 4.0 / 10
CX = 5.2 / 10
ME = 5.6 / 10
PL = 5.3 / 10
LL = 4.8 / 10
JG = 5.3 / 10
TO = 7.5 / 10
VF = 7.2 / 10
EV = 5.9 / 10
PO = 7.5 / 10
LE = 6.2 / 10
OR = 6.4 / 10

CORE = 7.58 / 10
CAP  = 5.81 / 10
IMPL = 7.32 / 10
ARCH = 6.21 / 10

max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR) = K = 8.5 / 10 = good
```

## Static Review Inputs

```text
source_review_scope = README.md + Cargo.toml + src/lib.rs + src/kernel/mod.rs + src/capability/tooling/mod.rs + src/capability/tooling/record.rs + rubric/score.md
local_workspace_status = source tree/archive unavailable under /mnt/data
cargo_build_status = not_run_by_instruction
cargo_test_status = not_run_by_instruction
graph_metrics_status = not_recomputed_current_cycle

readme_architecture_claim = kernel frozen, capability intelligence, TLog evidence, policy learning, LLM promoted to novelty
readme_current_status = stale
cargo_dependencies = none
unsafe_policy = forbid_unsafe_code

tooling_live_sandbox_file_executor = present
tooling_durable_effect_receipts = present
tooling_registry_policy_hash_binding = present
tooling_process_execution = present_initial_sandbox_process_executor
process_command_allowlist = present
process_cwd_lock = present
process_environment_lock = present
process_timeout_kill = present
process_stdout_stderr_digests = present
process_exit_status_receipt = present
process_max_output_bytes = present
process_receipt_replay = present
process_tlog_integration = partial_sidecar_receipt_path
tooling_api_execution = absent
provider_backed_llm_client = absent
real_observation_stream_parser = absent
external_artifact_verification = partial_internal_file_and_process_receipts
```

## Critical Judgment

The project improved. The previous score file was stale because it still marked process execution as absent. The code now contains a process-backed sandbox executor with explicit command allowlist, canonical sandbox cwd validation, cleared environment plus locked env injection, timeout kill behavior, stdout/stderr capture, bounded output reads, output digests, exit status, timeout flag, normalized `Effect::Process`, `SandboxProcessReceipt`, and replay validation.

That is a real capability upgrade:

```text
request -> authorize -> execute process -> digest stdout/stderr -> receipt -> replay check
```

The README direction is still correct: kernel correctness stays below capability intelligence. The README current-status section is now stale: the capability layer is no longer merely declared structure. Tooling has crossed from file artifact execution into local process execution.

The hard criticism is integration quality. The process path exists, but it is not yet fully first-class in the execution normal form. `ToolRequest::is_admissible` still admits deterministic/file artifact tooling, not `SandboxProcess`. Process authorization still reuses `Evidence::ArtifactReceipt` and `PacketEffect::MaterializeArtifact`. That means process execution is implemented, but the semantic contract is still artifact-shaped.

Current classification:

```text
current_system = deterministic evidence-runtime with first process-backed sandbox capability
not_yet = general autonomous agent
main_strength = frozen kernel + replay discipline + bounded local effects
main_weakness = process receipts are not yet first-class TLog/evidence semantics
```

The system is better than the previous rubric stated. It is still not a complete autonomous execution engine. It cannot yet call external APIs, authenticate providers, verify signed external receipts, stream observations, perform provider-backed LLM calls, or run distributed work queues. It can now run tightly bounded local processes and prove what happened by receipt.

## Module Rating Table

| Module                     | Status                           | Score | Reason                                                                                                                                                                                                                   |
|----------------------------+----------------------------------+-------+--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `kernel`                   | strong                           |   8.5 | Pure deterministic state, phases, gates, packet invariants, recovery classes, registry projection, and hash-chain event surface remain clean. Penalized for toy packet domain and limited effect vocabulary.             |
| `codec`                    | useful but hand-written          |   7.3 | TLog and receipt codecs exist and export process receipt support. Penalized for manual parser complexity and lack of schema-generated decoding.                                                                          |
| `api`                      | credible in-process protocol     |   6.4 | Command envelopes and evidence submission exist. Still lacks network service, auth, quotas, streaming, and hostile-client hardening.                                                                                     |
| `runtime`                  | strong deterministic replay core |   8.3 | Runtime/replay/gate logic is coherent and keeps kernel deterministic. Penalized because process receipts are not yet fully native control-event semantics.                                                               |
| `capability/observation`   | typed evidence facade            |   4.0 | Observation records and cursor ordering exist. No live SSE/webhook/filesystem/browser stream ingestion.                                                                                                                  |
| `capability/context`       | deterministic assembler seed     |   5.2 | Context records exist. Still lacks retrieval, grounding, token budget control, and policy-scoped context synthesis.                                                                                                      |
| `capability/memory`        | deterministic lookup seed        |   5.6 | Weighted lookup exists. Still lacks durable namespaces, embeddings, invalidation, decay, provenance, and cross-run query planning.                                                                                       |
| `capability/planning`      | typed gate producer              |   5.3 | Plan evidence exists. Still not a real planner with dependency solving, alternatives, cost model, or repair search.                                                                                                      |
| `capability/llm`           | structured adapter mock          |   4.8 | Prompt/response records exist. No provider client, retries, streaming parser, constrained decoding, or cost ledger.                                                                                                      |
| `capability/judgment`      | minimal typed judgment record    |   5.3 | Judgment is represented as evidence. It still does not compare alternatives or enforce irreversible-boundary reasoning.                                                                                                  |
| `capability/tooling`       | real bounded local execution     |   7.5 | Sandbox file effects plus process execution, allowlist, cwd/env locks, timeout, stdout/stderr digests, exit receipt, and replay validation are present. Still lacks API execution and native process evidence semantics. |
| `capability/verification`  | strong internal verifier base    |   7.2 | Receipt validity and replay checks are meaningful. Still mostly verifies internal receipts, not external-world claims.                                                                                                   |
| `capability/eval`          | solid record scorer              |   5.9 | Eval records/gates exist. Still lacks calibrated benchmarks, adversarial scoring, evaluator provenance, and threshold governance.                                                                                        |
| `capability/policy`        | strong capability foundation     |   7.5 | Policy hashing, promotion, feedback, and registry binding are credible. Needs conflict resolution, expiry, rollback, signatures, and migrations.                                                                         |
| `capability/learning`      | real but narrow                  |   6.2 | Learning promotes from TLog into policy. Still not causal attribution, pattern mining, strategy synthesis, or automatic capability generation.                                                                           |
| `capability/orchestration` | meaningful ordering layer        |   6.4 | Ordered capability submissions and skipped passed gates exist. Still lacks distributed workers, leases, queues, priorities, retry policy, and backpressure.                                                              |

## Artifact Judgment

```text
README = architecturally right, current-status stale
score_md = stale on process execution before this update
src = stronger than rubric said
kernel = still clean boundary
tooling = now the highest leverage implemented capability
```

The implementation now deserves credit for real local effects. The largest remaining risk is semantic mismatch: the process executor is operationally real, but the kernel/evidence language still mostly speaks artifact receipt. That mismatch should be fixed before adding more process/tool kinds.

## Regression / Improvement Delta

```text
previous_CORE = 7.48 / 10
current_CORE  = 7.58 / 10

previous_CAP = 5.69 / 10
current_CAP  = 5.81 / 10

previous_IMPL = 7.10 / 10
current_IMPL  = 7.32 / 10

previous_ARCH = 6.09 / 10
current_ARCH  = 6.21 / 10
```

The score improved because process-backed tooling is now present. The increase is bounded because the process path is not yet fully first-class in the evidence model, and the system still lacks live observation, real LLM providers, external API execution, and durable memory.

## Highest Leverage Next Work

1. **Make sandbox process execution first-class.** Add process-specific admissibility, evidence/effect route, TLog event binding, and verifier replay path instead of reusing artifact-shaped semantics.
2. **Add focused process executor tests.** Cover successful command, denied command, cwd escape, invalid env, timeout kill, output limit, receipt tamper, and replay mismatch.
3. **Split `LiveSandboxProcessExecutor`.** Separate process request construction, authorization, cwd/env validation, execution, output capture, receipt construction, and replay validation.
4. **Split `capability/tooling/record.rs`.** Move file effects, process effects, receipt codecs, and replay verifiers into typed submodules before adding more tool kinds.
5. **Update README current status.** The current README understates implementation progress and should mention bounded local process tooling.
6. **Add API-backed tooling next.** Local process execution is useful; external API calls need signed/provider receipts, auth scope, and replayable response digests.
7. **Keep kernel frozen.** Add richer capability semantics above the kernel; keep kernel effect evidence compact and deterministic.

## Updated Verdict

```text
objective_rating = ARCH = 6.21 / 10
system_level = deterministic evidence-runtime with bounded process-backed sandbox tooling
best_property = kernel/runtime replay discipline plus real receipted local effects
weakest_property = process execution is operationally real but not yet first-class in the evidence contract
next_score_unlock = first-class ProcessReceipt route in TLog + verifier + API protocol
```

The next move should be contract integration, not more abstract architecture: promote process execution from sidecar-capability reality into the canonical execution evidence path.
