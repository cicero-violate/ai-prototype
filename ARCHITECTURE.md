# Architecture

This document is the canonical location guide for the `ai` crate. When adding new code, find the layer it belongs to and follow the growth rules for that layer.

---

## Layer Stack

```
┌─────────────────────────────────────────┐
│  api/          External HTTP/MCP surface │
├─────────────────────────────────────────┤
│  service/      Autonomous loops & lifecycle coordination │
├─────────────────────────────────────────┤
│  capability/   Evidence producers & bounded effects     │
├─────────────────────────────────────────┤
│  runtime/      Reducer, replay, transitions             │
├─────────────────────────────────────────┤
│  codec/        Encode / decode only                     │
├─────────────────────────────────────────┤
│  kernel/       Frozen state machine types               │
└─────────────────────────────────────────┘
```

**Dependency rule**: each layer may only import from layers below it in the stack. No upward imports. `domain/` is a pure model layer with no I/O — any layer may reference it.

---

## Layers

### `kernel/`

Frozen. Owns the canonical state machine types: `State`, `TLog`, `ControlEvent`, `Gate`, `Phase`, `Packet`, `Evidence`, `RuntimeConfig`, `CanonError`.

No filesystem I/O. No process spawning. No imports from any other crate layer.

**Add here**: new state fields, new gate kinds, new evidence variants, new packet invariants. These require careful review — every downstream layer is affected.

**Do not add here**: I/O, parsing, receipts, capabilities, policy, scheduling.

```
kernel/
  state.rs          — State struct and field accessors
  gate.rs           — GateSet, GateStatus, GateId
  phase.rs          — Phase enum and transitions
  packet.rs         — Packet and objective model
  event.rs          — ControlEvent, Cause, Decision, EventKind, TLog
  recovery.rs       — FailureClass, RecoveryAction
  config.rs         — RuntimeConfig
  capability.rs     — CapabilityRegistryProjection (kernel-side view only)
  hash.rs           — mix() and low-level hash utilities
  error.rs          — CanonError
```

---

### `codec/`

Encode and decode records to/from NDJSON (and optionally binary). No validation logic, no replay, no I/O policy.

**Add here**: new NDJSON record formats, new binary serialization variants.

**Do not add here**: validation, replay policy, any state mutation.

```
codec/
  ndjson.rs         — TLog, ControlEvent, plan patch record encode/decode
  binary_tlog.rs    — Binary TLog format (feature-gated)
```

---

### `runtime/`

Owns reducer transitions, canonical event emission, recovery policy selection, replay, and workspace I/O primitives shared by capabilities and services.

**Add here**: new reducer arms, new transition rules, new replay modes, new workspace primitives (mailbox, transcripts, snapshots), new event bus projections.

**Do not add here**: capability-specific record types, HTTP handlers, scheduler policy, agent loop logic.

```
runtime/
  reducer.rs            — tick(), legal_transition(), core reducer
  transition_table.rs   — gate → evidence → next-phase mapping
  durable.rs            — durable replay and run_until_done variants
  recovery_policy.rs    — FailureClass → RecoveryAction policy
  writer.rs             — canonical TLog write path
  verify.rs             — TLog hash verification
  diff.rs               — semantic_diff between TLog snapshots
  introspection.rs      — introspect_canonical_tlog, WorkerStateReport
  command_ledger.rs     — CommandLedger, CommandReceipt deduplication
  event_bus.rs          — TaskReadyNotifier wakeup projection
  event_check.rs        — event invariant checks
  event_wire.rs         — event serialization wiring
  workspace.rs          — WorkspaceView, workspace_state_dir
  mailbox.rs            — MailboxMessage append/read
  mcp_transcript.rs     — legacy MCP transcript (draining)
  action_transcript.rs  — ActionTranscriptRecord append/replay
  snapshot.rs           — runtime snapshot helpers
```

---

### `capability/`

Pluggable evidence producers. Each capability owns the rich records, receipts, and external work needed to advance one or more kernel gates. A capability submits only a kernel-visible evidence token through the runtime — it never mutates `State` or `TLog` directly.

**Add a new capability** when you have a new class of evidence with its own record type, receipt hash, and gate contribution. Create `capability/<name>/mod.rs`.

**Do not add here**: HTTP routing, scheduler policy, agent loop decisions, process lifecycle.

```
capability/
  mod.rs                — CapabilityId, CapabilityRegistry, EvidenceSubmission, CAPABILITY_EFFECT_ROUTES
```

#### Capability modules

Each module below owns everything needed to produce its evidence:

```
  observation/          — InvariantProof: perceives external signals, validates ordering
    record.rs           — ObservationRecord, ObservationCursor, ObservationFrame
    source.rs           — BoundedLineObservationSource, ObservationIngressBatch

  context/              — AnalysisReport: assembles packet + memory into a context record
    record.rs           — ContextRecord, ContextAssemblyReceipt

  memory/               — lookup store consumed by context and judgment
    store.rs            — MemoryIndex, MemoryFact, MemoryLookupRecord

  llm/                  — JudgmentRecord via LLM call
    record.rs           — LlmRecord, LlmPromptRecord, LlmResponseRecord
    ollama.rs           — OllamaClient, OllamaLlmEffectReceipt, proof events
    openai.rs           — OpenAiClient, OpenAiLlmEffectReceipt, proof events
    browser_router.rs   — browser-routed LLM transport
    task_receipt.rs     — LlmTurnRecord, LlmTurnReceipt
    transport.rs        — shared SSE/HTTP transport helpers
    sse.rs              — SSE streaming

  judgment/             — PolicyJudgmentRecord, reuse ledger receipts
    record.rs           — JudgmentRecord, PolicyJudgmentDecision, reuse cost/trend receipts

  planning/             — TaskReady: converts objective to ready task
    record.rs           — PlanRecord, PlanReceipt, PlanDecision, PlanPatchRecord

  execution/            — ArtifactReceipt / ExecutionReceipt: bounded tool effects
    action/             — tool definitions, ActionHost trait, tool constants
      host/             — ActionHost implementations by domain
        agents.rs       — spawn agent, send message, read mailbox, browser tools
        graph.rs        — graph editor tools
        project.rs      — canon_score, canon_plan_read, canon_plan_update
        utility.rs      — echo, get_current_time
        workspace.rs    — apply_patch, shell, python
      canon_graph_editor.rs
      canon_plan.rs
      canon_read_mailbox.rs
      canon_score.rs
      canon_send_agent_message.rs
      canon_spawn_agent.rs
      common.rs         — tool_error, action_ok helpers
      landmarks.rs      — gateway tool manifest and landmark registry
      record.rs         — ActionCallRequest, ActionReceipt, LiveActionExecutor
    graph/              — GraphMutationReceipt, GraphPatchReceipt, contract verification
      artifacts.rs
      editor.rs
      patch_contract.rs
    patch/              — apply_patch runner (fs_copy, parser, runner)
    shell/              — shell tool (recorded and unrecorded variants)
    record/             — tool effect receipts, process receipts, sandbox records
      artifact.rs       — ToolExecutionRecord, ToolReceipt, DeterministicToolExecutor
      process.rs        — ProcessEffectReceipt, LiveSandboxProcessExecutor
      receipt.rs        — ToolEffectReceipt
      request.rs        — ToolRequest, SandboxProcessRequest
      types.rs          — ToolKind, ToolEffectKind, ToolDecision, Effect

  verification/         — LineageProof: semantic artifact checks
    proof.rs            — CanonicalEffect, CanonicalEffectProof, VerificationProofRecord
    record.rs           — VerificationRecord, VerificationReceipt
    validation_harness.rs

  eval/                 — EvalScore: scoring and threshold comparison
    record.rs           — EvalRecord, EvalDecision, EvalDimension
    score.rs            — ScoreVector, ScoreDelta, geometric_mean
    evolution.rs        — CandidateReceipt, EvalScorecardReceipt, SelectionRecord

  learning/             — PolicyPromotion: promote policy from TLog history
    promote.rs          — PolicyPromotion, DistillationRow, DistillationExportReceipt

  policy/               — versioned policy store consumed by judgment and learning
    store.rs            — PolicyStore, PolicyEntry, PolicyProofReceipt

  orchestration/        — multi-gate batch orchestration
    record.rs           — OrchestrationRecord, OrchestrationDecision
    cycle_event.rs      — AgentCycleEvent
    task_lifecycle.rs   — TaskLifecycleReceipt
    wave.rs             — WaveRecord

  analysis/             — AnalysisReceipt from rustc graph.json artifacts
    mod.rs              — invoke_rustc_analysis, receipt_from_graph_json

  exploration/          — fuzzy filename search and BM25 content search
    mod.rs              — search_files, search_files_bm25, SearchReceipt

  skills.rs             — CapabilitySkill registry (tool-capability mapping)
```

**Adding a new tool**: add `capability/execution/action/<tool_name>.rs`, register the constant and handler in `host/<domain>.rs`, add the name to `execute_native_tool` in `host.rs`.

**Adding a new LLM provider**: add `capability/llm/<provider>.rs` following the `ollama.rs` pattern (client, call, receipt, proof event, NDJSON helpers).

---

### `service/`

Autonomous agent loops, supervisor process lifecycle, and work coordination. Imports from `capability`, `runtime`, `codec`, `kernel`. May import from `api` for protocol types (`Command`, `CommandEnvelope`) that are shared across the boundary.

**Add here**: new agent loop phases, new supervisor lifecycle operations, new scheduler policies, new dispatch coordination between tools and tasks.

**Do not add here**: HTTP route handlers (those go in `api/routes/`), kernel types (those go in `kernel/`), tool implementations (those go in `capability/execution/action/`).

```
service/
  agent/                — autonomous observe/decide/act/verify loop driver
    cycle/              — one agent cycle: evidence submission, phase progression
      evidence.rs
      phase.rs
      recovery.rs
      state_parse.rs
      timing.rs
    loop_driver/        — drives the agent through a full objective
      dag_scheduler.rs
      evidence_submit.rs
      cycle_log.rs
      http.rs           — local HTTP helpers
      learning.rs
      mcp_workspace.rs
      prompt_builders.rs
      receipt.rs
    config.rs
    objective.rs
    prompt.rs
    router.rs           — routes loop step to the right driver
    sse.rs
    step.rs
    worker.rs           — run_with_heartbeat, heartbeat_claim, ActiveClaim
    worker_client.rs

  dispatch/             — work dispatch coordination
    action.rs           — dispatch_action_request: routes inbound tool calls through ActionHost
    task_client.rs      — HTTP client for task lifecycle API
    task_runner.rs      — wakeup-driven task runner (TaskReady → claim → run)

  scheduler/            — plan storage and task ready policy
    plan_store.rs       — load_plan, save_plan, append_evidence_patch
    handler.rs          — TaskReadyNotifier
    dispatch.rs         — scheduler dispatch helpers
    wave.rs             — wave progression

  supervisor/           — process orchestration and worker lifecycle
    config.rs           — SupervisorConfig, WorkspaceConfig
    state.rs            — SupervisorState, implements ActionHost
    process.rs          — WorkerProcess, spawn/restart/reload
    runtime.rs          — build_supervisor_router, HTTP server startup
    workspace.rs        — workspace management

  recovery/             — self-healing event loop
    event_loop.rs       — subscribes to GateFailed/LeaseExpired, resets stale nodes

  endpoints.rs          — shared service endpoint helpers
```

**Adding a new agent step type**: add the step logic in `service/agent/loop_driver/`, register it in `service/agent/router.rs`.

**Adding new supervisor lifecycle operations**: add to `service/supervisor/state.rs` (implements `ActionHost`) and wire the HTTP handler in `api/routes/supervisor/`.

---

### `api/`

External HTTP/MCP surface only. Owns route handlers, request/response DTOs, OAuth, and JSON-RPC framing. No kernel mutation, no direct TLog appending, no agent loop logic.

**Add here**: new HTTP routes, new request/response shapes, new auth middleware, new SSE streams.

**Do not add here**: tool implementations, agent loop logic, scheduler policy, capability records.

```
api/
  protocol.rs           — Command enum, CommandEnvelope, ControlEventResponse
  server.rs             — WorkerAppState, CommandEnvelopeDto, build_router
  transport.rs          — ApiTransportReceipt, receipt chain verification
  transport/
    replay.rs           — transport receipt replay and validation

  routes/
    command_routes.rs   — deterministic kernel command handlers (handle_command, handle_envelope)
    supervisor/         — Axum route handlers for supervisor API
      action.rs         — /action POST/GET SSE/DELETE (action protocol)
      mcp.rs            — /mcp POST/GET SSE/DELETE (MCP protocol, delegates to service::dispatch)
      control.rs        — health, reload, restart
      oauth.rs          — OAuth authorize/token/register
      router.rs         — build_supervisor_router, route wiring
      workspace.rs      — workspace get/update

  action/               — protocol-neutral action API (migration target for mcp/)
    dispatch.rs         — ActionDispatchPlan, dispatch_action_plan
    jsonrpc.rs          — action_ok, action_err, result_with_warning
    proxy.rs            — kernel command proxying
    schema.rs           — action_schema_list

  mcp/                  — MCP JSON-RPC helpers (external compat, draining to action/)
    dispatch.rs
    jsonrpc.rs
    proxy.rs
    schema.rs

  oauth/                — OAuth store, metadata, token handling
    store.rs
    types.rs
    metadata.rs
    crypto.rs
```

**Adding a new route**: add a handler function in the appropriate `api/routes/supervisor/<area>.rs`, register it in `api/routes/supervisor/router.rs`.

**`api/mcp/` is draining into `api/action/`**. New code goes in `api/action/`. The `api/mcp/` surface exists only for external client compatibility.

---

### `domain/`

Pure domain model — no I/O, no process spawning, no `State` mutation. Interprets world knowledge for capabilities and the agent loop to consume.

**Add here**: new domain entity types, new plan graph rules, new scoring/risk models, new semantic domain concepts.

**Do not add here**: anything with I/O, anything that touches `State` or `TLog` directly.

```
domain/
  plan.rs               — PlanDag, PlanNode, NodeStatus, ready_nodes_from_available_plan_state
  business.rs           — business domain entities
  finance.rs            — financial domain entities
  trading.rs            — trading domain entities
  risk.rs               — risk assessment models
  scoring.rs            — scoring domain rules
  contracts.rs          — contract domain types
  identity.rs           — identity domain types
  global_intelligence.rs
  semantic.rs           — semantic domain types
  bridge.rs             — domain bridging helpers
```

---

## Growth Rules

### Adding a new capability (new gate evidence type)
1. Create `capability/<name>/mod.rs` with ownership boundary doc comment
2. Add the record type, receipt hash, and `decision()` → `submission()` methods
3. Register the `CapabilityId` variant and `CapabilityEffectRoute` in `capability/mod.rs`
4. Wire the evidence token in `kernel/event.rs` if a new `Evidence` variant is needed

### Adding a new tool
1. Add `capability/execution/action/<tool_name>.rs` with the implementation
2. Add the tool constant (`pub const <TOOL>_TOOL: &str = "<name>"`) in the same file
3. Register the handler in the appropriate `capability/execution/action/host/<domain>.rs`
4. Add the name to the `match` in `capability/execution/action/host.rs::execute_native_tool`
5. Export the constant from `capability/execution/action/mod.rs`

### Adding a new LLM provider
1. Add `capability/llm/<provider>.rs` following the `ollama.rs` / `openai.rs` pattern
2. Implement: client, call builder, effect receipt, judgment proof event, NDJSON round-trip helpers
3. Export from `capability/llm/mod.rs`

### Adding a new service-layer operation
1. If it coordinates existing tools/tasks: add to `service/dispatch/`
2. If it manages agent loop phases: add to `service/agent/loop_driver/`
3. If it manages worker lifecycle: add to `service/supervisor/`
4. If it manages plan storage or task readiness: add to `service/scheduler/`

### Adding a new HTTP route
1. Add the handler function in `api/routes/supervisor/<area>.rs`
2. Register it in `api/routes/supervisor/router.rs`
3. If it needs new request/response shapes, add DTOs to `api/server.rs`

### Adding a new domain concept
1. Add to the appropriate file in `domain/` or create `domain/<concept>.rs`
2. No I/O in `domain/` — if the concept needs I/O, the I/O lives in the capability or service that consumes it

---

## Invariants

| Rule                                                     | Enforced by                               |
|----------------------------------------------------------+-------------------------------------------|
| `kernel` has zero imports from other layers              | `tests/architecture_boundary_contract.rs` |
| `runtime` does not import `service` or `api`             | `tests/architecture_boundary_contract.rs` |
| `capability` does not import `service`                   | `tests/architecture_boundary_contract.rs` |
| `api` has no tool implementations or agent loop logic    | code review                               |
| Every capability receipt type has a deterministic hash   | `is_contract_valid()` on every receipt    |
| Every TLog-appended record has a schema version constant | NDJSON encode/decode pair                 |

---

## Open cleanup items

- `api/mcp/` → drain remaining callers to `api/action/`, delete when external compat window closes
- `capability/execution/action/dispatch.rs` → one-line shim, delete after all callers use `service::dispatch`
- `service/supervisor/state.rs` → imports `api::oauth::OAuthStore` and `api::routes::build_supervisor_router`; those should be assembled at the `api` layer and injected
- `capability/execution/action/canon_plan.rs` → imports `service::scheduler::plan_store`; plan store projection should be accessed through a runtime interface, not a direct service import
- Legacy `Mcp*` type aliases (`McpCallReceipt`, `McpCallRequest`, `LiveMcpCallExecutor`) still surface in `lib.rs` and `api/server.rs`; rename to `Action*` equivalents when external wire format allows
