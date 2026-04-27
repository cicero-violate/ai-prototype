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

One-line explanation: geometric scoring punishes missing execution surfaces; a strong deterministic kernel cannot hide weak live-world capability.

## Score Summary

```text
K  = 8.3 / 10
C  = 7.1 / 10
A  = 6.2 / 10
R  = 8.0 / 10

OB = 4.0 / 10
CX = 5.2 / 10
ME = 5.6 / 10
PL = 5.2 / 10
LL = 4.7 / 10
JG = 5.1 / 10
TO = 5.3 / 10
VF = 6.6 / 10
EV = 5.8 / 10
PO = 7.3 / 10
LE = 6.0 / 10
OR = 6.2 / 10

CORE = 7.35 / 10
CAP  = 5.52 / 10
IMPL = 6.64 / 10
ARCH = 5.93 / 10

max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR) = K = 8.3 / 10 = good
```

## Static Review Inputs

```text
source_files = 43
source_lines = 8352
rust_functions_regex = 424
unit_tests_declared = 82
integration_tests_detected = 0
cargo_build_status = not_run_cargo_binary_missing_in_sandbox
cargo_test_status = not_run_cargo_binary_missing_in_sandbox

semantic_graph_nodes = 1235
semantic_graph_edges = 3058
cfg_nodes = 3647
cfg_edges = 4640
bridge_edges = 1158
redundant_path_pairs = 488
alpha_pathways = 13
graph_schema_version = 9
graph_captured_at_utc = 2026-04-27T18:27:13.119Z

index_json_top_level_crates = 1
index_json_status = locator_only_not_architectural_source_of_truth

kernel_files = 1
codec_files = 2
runtime_files = 8
api_files = 3
capability_files = 25
neutral_error_files = 1

upward_layer_dependency_violations = 0
semantic_manifest_low_confidence_nodes = 1235 / 1235
largest_cfg_function_1 = codec::ndjson::pop_event, 127 blocks
largest_cfg_function_2 = runtime::verify::validate_event, 99 blocks
largest_cfg_function_3 = runtime::verify::verify_tlog_from, 82 blocks
largest_redundant_path_owner = runtime::verify::validate_event, 136 redundant pairs

README_reviewed = yes
src_reviewed = src/**/*.rs
artifacts_reviewed = state/rustc/ai/graph.json + state/rustc/index.json + rubric/score.md
```

## Critical Judgment

The project has crossed from a toy state-machine sketch into a credible deterministic evidence-runtime prototype. The latest source shows better layer purity, a neutral error module, command-ledger reconstruction from TLog, durable replay checks, capability registry projection, registry policy hashes bound into execution receipts, and replay rejection for registry drift.

The important improvement is architectural: the previous runtime/API and codec/runtime impurity has been corrected in the current source structure. The foundation is cleaner now. That deserves a real score increase.

The hard criticism remains: this is still not an autonomous intelligent agent. The capability layer mostly produces typed deterministic records from packet-shaped internal state. It does not yet observe live inputs, plan non-trivial dependency graphs, call real tools, retrieve durable memory, use a provider-backed LLM, or verify external artifacts. The system is reliable as a replayable control kernel; it is not yet powerful as an execution engine.

Current classification:

```text
current_system = deterministic evidence-runtime with simulated capability surfaces
not_yet = live autonomous agent
main_strength = replayable state correctness
main_weakness = no real external execution loop
```

The graph confirms growth and also exposes risk. There are 1235 semantic nodes and 3647 CFG nodes, but every semantic manifest is still low confidence. The graph is useful for structure, CFG concentration, and redundancy detection. It is not yet a trustworthy semantic reasoning oracle.

## Module Rating Table

| Module                     | Status                              | Score | Reason                                                                                                                                                                                                 |
|----------------------------|-------------------------------------|------:|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `kernel`                   | strong                              |   8.3 | Deterministic phase/gate/state core with packet semantics, recovery evidence, learning gate, capability registry projection, and stable hash primitives. Penalized for still encoding a toy packet domain. |
| `codec`                    | cleaner but too manual              |   7.1 | NDJSON append/load/roundtrip paths are useful and no longer depend upward on runtime errors. Penalized for hand-written numeric decoding, large parser CFGs, and audit-hostile positional formats.       |
| `api`                      | credible in-process protocol        |   6.2 | Command envelopes, schema hash binding, duplicate replay, conflicting command rejection, atomic batches, and receipts exist. Still lacks network service, auth, quotas, streaming, and external hardening. |
| `runtime`                  | strong deterministic core           |   8.0 | Reducer, durable replay, disk/memory drift checks, command ledger reconstruction, TLog verification, and registry projection persistence are meaningful. Penalized for high verifier/reducer complexity. |
| `capability/observation`   | typed evidence facade               |   4.0 | ObservationRecord and cursor ordering exist. It still does not parse real SSE/webhook/filesystem/browser streams or authenticate observation provenance.                                                |
| `capability/context`       | deterministic assembler seed        |   5.2 | ContextRecord combines packet/memory signals into analysis evidence. It lacks real retrieval, token budgeting, policy-scoped context, document context, and prior-run synthesis.                       |
| `capability/memory`        | deterministic lookup seed           |   5.6 | MemoryIndex gives stable ordered facts and weighted lookup. It lacks durable namespaces, embeddings, provenance, invalidation, decay, and cross-run retrieval.                                         |
| `capability/planning`      | typed gate producer                 |   5.2 | PlanRecord binds ready tasks and drives the Plan gate. It is not yet a planner: no dependency solving, graph expansion, budget model, schedule repair, or alternative search.                           |
| `capability/llm`           | structured adapter mock             |   4.7 | Prompt/response records, schema hash, token counts, policy feedback, and judgment conversion exist. No provider client, constrained decoder, retries, streaming parser, or cost ledger.                |
| `capability/judgment`      | minimal typed record                |   5.1 | JudgmentRecord is typed and routed. It does not yet compare alternatives, resolve conflicting evidence, model uncertainty, or enforce irreversible-boundary reasoning.                                 |
| `capability/tooling`       | improved deterministic receipt seam |   5.3 | Tool requests, deterministic executor, explicit registry authorization, policy hash binding, effect receipts, and replay drift rejection exist. Still no real process/API/file tool execution or sandbox. |
| `capability/verification`  | strongest semantic capability       |   6.6 | Semantic profiles, request/receipt hashes, lineage repair checks, denial handling, and tamper rejection exist. Still validates internal packet artifacts rather than real files, tests, APIs, or claims. |
| `capability/eval`          | solid record scorer                 |   5.8 | Eval dimensions, thresholds, and gate submission exist. It lacks calibrated benchmarks, objective-specific metrics, evaluator provenance, adversarial scoring, and threshold governance.                |
| `capability/policy`        | strongest capability foundation     |   7.3 | Append-only entries, durable promotion, versioning, fingerprinting, policy feedback, and registry policy hashing are good. Needs conflicts, scope, expiry, rollback, signatures, and migration policy.  |
| `capability/learning`      | real but narrow                     |   6.0 | PolicyPromotion reads TLog and emits policy facts. It is still promotion glue, not pattern mining, counterexample learning, causal attribution, or strategy synthesis.                                  |
| `capability/orchestration` | meaningful ordering layer           |   6.2 | OrchestrationRecord orders capability submissions and skips passed gates. It lacks distributed workers, leases, queues, priorities, retries, parallel isolation, and backpressure.                       |

## Artifact Judgment

```text
graph_json = useful_structural_source_of_truth
index_json = locator_only
README = architecturally clear but ahead of implementation
src = broad deterministic implementation with cleaner layer boundaries
rubric_score = updated_to_match_current_static_state
```

`graph.json` reports 1235 semantic nodes, 3058 semantic edges, 3647 CFG nodes, 4640 CFG edges, 1158 bridge edges, 488 redundant path pairs, and 13 alpha pathways. This is enough structure to guide refactoring.

The danger remains concentrated complexity. `codec::ndjson::pop_event` has 127 CFG blocks. `runtime::verify::validate_event` has 99 CFG blocks and owns 136 redundant path pairs. `runtime::verify::verify_tlog_from` has 82 CFG blocks. These are correctness-critical functions and should become smaller typed validators before the project grows more surface area.

The graph semantics layer is not yet reliable for judgment. All 1235 semantic manifests are low confidence. Treat graph structure as evidence; do not treat graph intent labels as authority.

## Regression / Improvement Delta

```text
previous_CORE = 6.96 / 10
current_CORE  = 7.35 / 10

previous_CAP = 5.23 / 10
current_CAP  = 5.52 / 10

previous_IMPL = 6.35 / 10
current_IMPL  = 6.64 / 10

previous_ARCH = 5.62 / 10
current_ARCH  = 5.93 / 10
```

The project improved. The gain comes from fixed layer purity, expanded test surface, durable command-ledger replay, registry projection persistence, deny-by-default capability routing, registry policy hash binding, and replay rejection under policy drift.

The score is still below 6 because the architecture claims self-improving autonomous capability, while the implementation remains mostly deterministic internal evidence routing. The runtime is strong; live capability is weak.

## Highest Leverage Next Work

1. **Make tooling real under the current registry model.** Add a sandboxed file/process/API tool runner with explicit allowed effects, no ambient permissions, durable request/decision/effect receipts, and replay verification.
2. **Split correctness-critical CFG clusters.** Break `codec::ndjson::pop_event`, `runtime::verify::validate_event`, and `runtime::verify::verify_tlog_from` into smaller typed validators.
3. **Move tests out of `src/lib.rs`.** The root module is 2202 lines and carries 82 unit tests. Move integration-style tests into `tests/` and module-specific test files.
4. **Promote graph confidence from metadata to gate.** Fail or warn when semantic manifests are all low confidence. The graph should not silently imply more semantic knowledge than it has.
5. **Upgrade verification targets.** Verify real artifacts: file digests, build/test outputs, command stdout/stderr, API responses, or signed receipts.
6. **Add durable memory before smarter planning.** Planning without retrieval will remain toy planning. The next intelligence unlock is persisted memory with provenance and invalidation.

## Updated Verdict

```text
objective_rating = ARCH = 5.93 / 10
system_level = deterministic evidence-runtime prototype
best_property = kernel/runtime replay discipline
weakest_property = real-world autonomous execution
next_score_unlock = live sandboxed tooling with durable effect receipts
```

The kernel is good. Runtime is nearly good. Policy is the strongest capability foundation. The current architecture is clean enough to continue upward. The next move should not be more abstract state-machine work; it should be one live external capability executed through the registry, TLog, receipt, and verifier path.