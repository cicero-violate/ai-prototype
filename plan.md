# Internal Naming and Runtime Migration Plan

## Goal

Remove confusing transport and lifecycle names from the internal architecture.

The permanent model should describe what the system owns:

- `api` receives external protocol requests.
- `service` owns long-lived workflows, daemons, supervisors, schedulers, and agent loops.
- `capability` defines bounded capabilities and executes effects that produce receipts.
- `runtime` records, reduces, replays, persists, and verifies state transitions.
- `kernel` defines the stable deterministic vocabulary and state machine types.

The names `mcp`, `tooling`, and top-level `process` should not be part of the internal domain model.
If MCP remains supported externally, it should exist only as an edge protocol adapter.

## Target Tree

```text
ai/src
├── api
│   ├── action
│   │   ├── dispatch.rs
│   │   ├── jsonrpc.rs
│   │   ├── mod.rs
│   │   ├── proxy.rs
│   │   └── schema.rs
│   ├── oauth
│   ├── protocol.rs
│   ├── routes
│   │   ├── command_routes.rs
│   │   ├── mod.rs
│   │   └── supervisor
│   │       ├── action.rs
│   │       ├── control.rs
│   │       ├── mod.rs
│   │       ├── oauth.rs
│   │       ├── router.rs
│   │       └── workspace.rs
│   ├── server.rs
│   └── transport.rs
├── capability
│   ├── context
│   ├── eval
│   ├── execution
│   │   ├── action
│   │   │   ├── dispatch.rs
│   │   │   ├── gateway.rs
│   │   │   ├── host.rs
│   │   │   ├── landmarks.rs
│   │   │   ├── mod.rs
│   │   │   ├── record.rs
│   │   │   ├── runner.rs
│   │   │   └── transcript.rs
│   │   ├── graph
│   │   │   ├── artifacts.rs
│   │   │   ├── editor.rs
│   │   │   ├── mod.rs
│   │   │   └── patch_contract.rs
│   │   ├── mailbox
│   │   │   └── mod.rs
│   │   ├── mod.rs
│   │   ├── patch
│   │   │   ├── fs_copy.rs
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs
│   │   │   └── runner.rs
│   │   ├── process
│   │   │   ├── mod.rs
│   │   │   ├── record.rs
│   │   │   └── runner.rs
│   │   ├── search.rs
│   │   ├── shell
│   │   │   ├── mod.rs
│   │   │   ├── recorded.rs
│   │   │   └── unrecorded.rs
│   │   └── workspace.rs
│   ├── judgment
│   ├── learning
│   ├── llm
│   ├── memory
│   ├── observation
│   ├── orchestration
│   ├── planning
│   ├── policy
│   ├── skills.rs
│   └── verification
├── codec
├── domain
├── kernel
├── runtime
│   ├── action_transcript.rs
│   ├── command_ledger.rs
│   ├── durable.rs
│   ├── event_bus.rs
│   ├── mailbox.rs
│   ├── reducer.rs
│   └── ...
└── service
    ├── agent
    │   ├── config.rs
    │   ├── cycle
    │   ├── doc
    │   ├── loop_driver
    │   ├── mod.rs
    │   ├── objective.rs
    │   ├── prompt.rs
    │   ├── router.rs
    │   ├── sse.rs
    │   ├── step.rs
    │   ├── worker.rs
    │   └── worker_client.rs
    ├── dispatch
    │   ├── mod.rs
    │   ├── task_client.rs
    │   └── task_runner.rs
    ├── endpoints.rs
    ├── mod.rs
    ├── recovery
    │   ├── event_loop.rs
    │   └── mod.rs
    ├── scheduler
    │   ├── dispatch.rs
    │   ├── handler.rs
    │   ├── mod.rs
    │   ├── plan_store.rs
    │   └── wave.rs
    └── supervisor
        ├── config.rs
        ├── mod.rs
        ├── runtime.rs
        ├── state.rs
        ├── worker_runtime.rs
        └── workspace.rs
```

## Current to Target Map

### External Protocol Edge

| Current | Target | Notes |
| --- | --- | --- |
| `api/mcp/mod.rs` | `api/action/mod.rs` | Public protocol module becomes action-oriented. |
| `api/mcp/dispatch.rs` | `api/action/dispatch.rs` | Keep pure request planning. Rename `AiMcpDispatchPlan` to `ActionDispatchPlan`. |
| `api/mcp/jsonrpc.rs` | `api/action/jsonrpc.rs` | JSON-RPC is a wire format, not the domain name. |
| `api/mcp/proxy.rs` | `api/action/proxy.rs` | Rename command tags from MCP terms to action terms. |
| `api/mcp/schema.rs` | `api/action/schema.rs` | Rename tool schema helpers to action schema helpers. |
| `api/routes/supervisor/mcp.rs` | `api/routes/supervisor/action.rs` | Route may still serve `/ai/mcp` temporarily, but handler names should be action based. |

Temporary compatibility:

- Keep `/ai/mcp` route during migration.
- Allow wire payloads with existing MCP method names while mapping them into action calls internally.
- Do not expose new internal `mcp` module paths.

### Capability Execution

| Current | Target | Notes |
| --- | --- | --- |
| `capability/tooling/mod.rs` | `capability/execution/mod.rs` | Replace the top-level capability name. |
| `capability/tooling/mcp_tools/dispatch.rs` | `capability/execution/action/dispatch.rs` and `runner.rs` | Split dispatch orchestration from runner implementation. |
| `capability/tooling/mcp_tools/common.rs` | `capability/execution/action/common.rs` or remove | Keep only if still shared. |
| `capability/tooling/mcp_tools/landmarks.rs` | `capability/execution/action/landmarks.rs` | Rename gateway landmarks later if that concept changes. |
| `capability/tooling/mcp_tools/canon_*.rs` | `capability/execution/action/*.rs` or specific capability folders | Prefer domain folders for graph, plan, score, mailbox, agent messaging. |
| `capability/tooling/native/mod.rs` | `capability/execution/action/host.rs` | Rename `NativeToolHost` to `ActionHost`. |
| `capability/tooling/native/workspace.rs` | `capability/execution/workspace.rs` | Workspace action implementations. |
| `capability/tooling/native/agents.rs` | `capability/execution/action/agent.rs` or `service/supervisor/action_host.rs` | Host-bound supervisor actions likely belong near service host implementation. |
| `capability/tooling/native/graph.rs` | `capability/execution/graph/mod.rs` | Graph execution capability. |
| `capability/tooling/native/project.rs` | `capability/execution/project.rs` | Project-local action capability. |
| `capability/tooling/native/utility.rs` | `capability/execution/action/utility.rs` | Small built-in actions. |
| `capability/tooling/record/mcp.rs` | `capability/execution/action/record.rs` | Rename request and receipt types. |
| `capability/tooling/record/process.rs` | `capability/execution/process/record.rs` | This is bounded sandbox process execution, not service lifecycle. |
| `capability/tooling/record/request.rs` | `capability/execution/record/request.rs` | Shared execution request records if still needed. |
| `capability/tooling/record/receipt.rs` | `capability/execution/record/receipt.rs` | Shared execution receipt records if still needed. |
| `capability/tooling/record/artifact.rs` | `capability/execution/record/artifact.rs` | Artifact receipt records. |
| `capability/tooling/record/hash.rs` | `capability/execution/record/hash.rs` | Shared hash helpers. |
| `capability/tooling/record/types.rs` | `capability/execution/record/types.rs` | Shared execution effect types. |
| `capability/tooling/patch.rs` | `capability/execution/patch/mod.rs` | Patch capability surface. |
| `capability/tooling/mcp_tools/apply_patch/*` | `capability/execution/patch/*` | Patch runner implementation. |
| `capability/tooling/mcp_tools/shell/*` | `capability/execution/shell/*` | Shell runner implementation. |
| `capability/tooling/graph_artifacts.rs` | `capability/execution/graph/artifacts.rs` | Graph artifact support. |
| `capability/tooling/graph_editor.rs` | `capability/execution/graph/editor.rs` | Graph editor support. |
| `capability/tooling/graph_patch_contract.rs` | `capability/execution/graph/patch_contract.rs` | Graph patch contract. |
| `capability/tooling/rustc_analysis.rs` | `capability/execution/rustc_analysis.rs` | Keep if it is an execution capability. |
| `capability/tooling/search.rs` | `capability/execution/search.rs` | Search action capability. |

### Runtime Records

| Current | Target | Notes |
| --- | --- | --- |
| `runtime/mcp_transcript.rs` | `runtime/action_transcript.rs` | Transcript is an action transcript, not MCP-specific. |
| `McpTranscriptRecord` | `ActionTranscriptRecord` | Store tool/action name, request JSON, response JSON, receipt hash. |
| `append_mcp_transcript` | `append_action_transcript` | Preserve existing file replay with migration support. |
| `mcp-transcript.tlog.ndjson` | `action-transcript.tlog.ndjson` | Read legacy file during transition. |
| `state/agent_state/mcp/tool-results.ndjson` | `state/agent_state/action/results.ndjson` | Mirror legacy path until migration is complete. |

### Kernel Command Protocol

| Current | Target | Notes |
| --- | --- | --- |
| `McpCallRequest` | `ActionCallRequest` | Protocol-neutral call request. |
| `McpCallReceipt` | `ActionCallReceipt` or `ActionReceipt` | Prefer `ActionReceipt` if unambiguous. |
| `LiveMcpCallExecutor` | `LiveActionExecutor` | Used by tests and agent docs. |
| `AuthorizeMcpCall` | `AuthorizeActionCall` | Command enum variant. |
| `SubmitMcpCallReceipt` | `SubmitActionReceipt` | Command enum variant. |
| `mcp_authorization_submission` | `action_authorization_submission` | Helper in `api/protocol.rs`. |
| `ensure_mcp_receipt_authorized` | `ensure_action_receipt_authorized` | Helper in command route handling. |
| DTOs named `McpCall*Dto` | DTOs named `ActionCall*Dto` | Server decoding compatibility can accept old tags temporarily. |

Compatibility rule:

- Runtime command decoding should accept legacy payload tags during one migration window.
- New code should emit only action names.
- Tests should be converted to action names after compatibility tests are added.

### Service Layer

| Current | Target | Notes |
| --- | --- | --- |
| `process/mod.rs` | `service/mod.rs` | Top-level `process` disappears. |
| `process/agent/*` | `service/agent/*` | Agent loop service. |
| `process/dispatch/*` | `service/dispatch/*` | Task runner and task client service. |
| `process/recovery/*` | `service/recovery/*` | Recovery event-loop service. |
| `process/scheduler/*` | `service/scheduler/*` | Scheduler service over domain plan state. |
| `process/supervisor/process.rs` | `service/supervisor/worker_runtime.rs` | Long-lived worker daemon lifecycle. |
| `process/supervisor/runtime.rs` | `service/supervisor/runtime.rs` | Supervisor service runtime. |
| `process/supervisor/state.rs` | `service/supervisor/state.rs` | Host state and action host implementation. |
| `process/supervisor/workspace.rs` | `service/supervisor/workspace.rs` | Workspace config for supervisor service. |
| `process/endpoints.rs` | `service/endpoints.rs` | Environment endpoint helpers. |

Service naming rule:

- Use `worker_runtime` for long-lived worker child process lifecycle.
- Use `execution/process` only for bounded command execution that produces receipts.

## Type Rename Map

### Action Call Types

| Current | Target |
| --- | --- |
| `McpCallRequest` | `ActionCallRequest` |
| `McpCallReceipt` | `ActionReceipt` |
| `McpToolHost` | `ActionHost` |
| `NativeToolHost` | `ActionHost` |
| `NativeMcpState` | `ActionServiceState` or `ActionHostState` |
| `NativeMcpSession` | `ActionSession` |
| `AiMcpDispatchPlan` | `ActionDispatchPlan` |
| `dispatch_ai_mcp_plan` | `dispatch_action_plan` |
| `dispatch_ai_mcp` | `dispatch_action_request` |
| `execute_recorded_ai_mcp_tool` | `execute_recorded_action` |
| `execute_recorded_native_ai_mcp_tool` | `execute_recorded_local_action` |
| `execute_recorded_gateway_tool` | `execute_recorded_gateway_action` |
| `LiveMcpCallExecutor` | `LiveActionExecutor` |

### Command Variants

| Current | Target |
| --- | --- |
| `AuthorizeMcpCall` | `AuthorizeActionCall` |
| `SubmitMcpCallReceipt` | `SubmitActionReceipt` |
| `decode_authorize_mcp_call_command` | `decode_authorize_action_call_command` |
| `decode_submit_mcp_call_receipt_command` | `decode_submit_action_receipt_command` |

### Runtime Transcript Types

| Current | Target |
| --- | --- |
| `McpTranscriptRecord` | `ActionTranscriptRecord` |
| `append_mcp_transcript` | `append_action_transcript` |
| `replay_mcp_transcripts` | `replay_action_transcripts` |
| `mcp_transcript_path` | `action_transcript_path` |

## Migration Phases

### Phase 0: Freeze Contracts and Add Compatibility Tests

Objective: make behavior explicit before moving modules.

Tasks:

- Add tests proving old command tags still decode.
- Add tests proving receipt authorization requires a prior authorization event.
- Add tests proving transcript replay works for the current path.
- Add tests proving the external `/ai/mcp` route still works after internal renames.

Exit criteria:

- Existing tests pass.
- Compatibility expectations are captured before renames.

### Phase 1: Introduce New Modules as Re-exports

Objective: create the target module names without moving implementation yet.

Tasks:

- Add `capability/execution/mod.rs` that re-exports from `capability/tooling`.
- Add `api/action/mod.rs` that re-exports from `api/mcp`.
- Add `runtime/action_transcript.rs` that re-exports from `runtime/mcp_transcript`.
- Add `service/mod.rs` that re-exports from `process`.
- Update `lib.rs` public exports to expose new names alongside old names.

Exit criteria:

- New imports compile.
- Old imports compile.
- No file moves yet.

### Phase 2: Rename Core Action Records

Objective: remove MCP names from the kernel command protocol and record vocabulary.

Tasks:

- Move `capability/tooling/record/mcp.rs` to `capability/execution/action/record.rs`.
- Rename `McpCallRequest` to `ActionCallRequest`.
- Rename `McpCallReceipt` to `ActionReceipt`.
- Rename `LiveMcpCallExecutor` to `LiveActionExecutor`.
- Rename command variants:
  - `AuthorizeMcpCall` to `AuthorizeActionCall`
  - `SubmitMcpCallReceipt` to `SubmitActionReceipt`
- Add temporary type aliases for old names:
  - `pub type McpCallRequest = ActionCallRequest`
  - `pub type McpCallReceipt = ActionReceipt`
  - `pub type LiveMcpCallExecutor = LiveActionExecutor`
- Add temporary decode aliases for old command payload tags.

Exit criteria:

- New type names are used by runtime and protocol code.
- Old external tags still decode.
- Tests pass.

### Phase 3: Rename Runtime Transcript

Objective: make runtime persistence protocol-neutral.

Tasks:

- Rename `runtime/mcp_transcript.rs` to `runtime/action_transcript.rs`.
- Rename APIs to action terminology.
- Change the canonical transcript path to:
  - `state/agent_state/action/action-transcript.tlog.ndjson`
- Keep read support for:
  - `state/agent_state/mcp/mcp-transcript.tlog.ndjson`
- During the migration window, optionally mirror writes to both paths.
- Rename raw result path to:
  - `state/agent_state/action/results.ndjson`

Exit criteria:

- New transcript path is written.
- Legacy transcript path is replayed.
- Transcript tests cover both new and legacy paths.

### Phase 4: Extract Action Runner

Objective: separate external request dispatch from live execution lifecycle.

Tasks:

- Create `capability/execution/action/runner.rs`.
- Define `ActionRunner` or `ActionHost` around:
  - workspace access
  - session recording
  - active generation
  - kernel command submission
  - host-bound service actions
- Move execution orchestration out of current `mcp_tools/dispatch.rs`.
- Keep `api/action/dispatch.rs` pure and effect-free.
- Keep JSON-RPC response shaping at the API edge.

Target lifecycle:

```text
external request
  -> api/action dispatch plan
  -> capability/execution/action runner
  -> runtime authorization command
  -> bounded live execution
  -> runtime transcript append
  -> runtime receipt command
  -> external protocol response
```

Exit criteria:

- `api/action` contains no live execution.
- `capability/execution/action` owns live action execution.
- Supervisor state implements `ActionHost`.

### Phase 5: Move Bounded Execution Capabilities

Objective: dissolve `capability/tooling`.

Tasks:

- Move shell implementation to `capability/execution/shell`.
- Move patch implementation to `capability/execution/patch`.
- Move graph implementation to `capability/execution/graph`.
- Move sandbox process records and runners to `capability/execution/process`.
- Move search and rustc analysis to `capability/execution`.
- Update module exports.
- Leave `capability/tooling` as a deprecated compatibility shim temporarily.

Exit criteria:

- New code imports from `capability::execution`.
- `capability::tooling` contains only re-exports or is deleted after callers migrate.

### Phase 6: Rename Top-level `process` to `service`

Objective: remove overloaded top-level lifecycle naming.

Tasks:

- Create `service/*` matching current `process/*`.
- Move `process/agent` to `service/agent`.
- Move `process/dispatch` to `service/dispatch`.
- Move `process/recovery` to `service/recovery`.
- Move `process/scheduler` to `service/scheduler`.
- Move `process/supervisor/process.rs` to `service/supervisor/worker_runtime.rs`.
- Update imports from `crate::process::...` to `crate::service::...`.
- Keep `process/mod.rs` as a deprecated re-export while migrating if needed.

Exit criteria:

- No new imports use `crate::process`.
- Long-lived lifecycle code lives under `service`.
- Bounded command execution lives under `capability/execution/process`.

### Phase 7: Remove Compatibility Shims

Objective: complete the vocabulary cleanup.

Tasks:

- Delete `api/mcp` after all internal imports are gone.
- Delete `capability/tooling` after all internal imports are gone.
- Delete `process` after all internal imports are gone.
- Remove old type aliases:
  - `McpCallRequest`
  - `McpCallReceipt`
  - `LiveMcpCallExecutor`
  - `McpTranscriptRecord`
- Remove old command tags once external compatibility is no longer required.
- Update docs under `service/agent/doc`.

Exit criteria:

- `rg -n "Mcp|mcp|tooling|crate::process|process/" ai/src` returns only legitimate external compatibility references or nothing.
- Public docs describe actions, execution, service runtime, and capability receipts.

## Layering Rules After Migration

### `kernel`

Allowed:

- State machine vocabulary.
- Stable event, gate, phase, evidence, hash, and packet types.

Not allowed:

- Live execution.
- HTTP.
- JSON-RPC.
- Action runner logic.
- Supervisor or worker lifecycle.

### `runtime`

Allowed:

- Reducer.
- Command ledger.
- Durable replay.
- Transcript persistence.
- Verification.
- Runtime event bus.

Not allowed:

- Running shell commands.
- Calling external tools.
- Owning supervisor child processes.

### `capability`

Allowed:

- Capability contracts.
- Evidence records.
- Execution receipts.
- Bounded live effects through explicit runners.

Not allowed:

- Long-lived daemon supervision.
- HTTP route ownership.
- Kernel reducer decisions.

### `service`

Allowed:

- Agent loop lifecycle.
- Supervisor lifecycle.
- Task runner lifecycle.
- Scheduler lifecycle.
- Recovery event loops.
- Host implementations for capability runners.

Not allowed:

- Defining kernel vocabulary.
- Duplicating runtime reducer rules.

### `api`

Allowed:

- HTTP routes.
- JSON-RPC wire compatibility.
- DTO decoding.
- Command envelope ingestion.

Not allowed:

- Live action execution.
- Direct TLog mutation outside runtime command paths.

## Compatibility Policy

The migration should preserve working behavior between phases.

Short-term compatibility:

- `/ai/mcp` can remain as the external route.
- JSON-RPC method names can remain externally if clients require them.
- Legacy command payload tags can decode.
- Legacy transcript paths can replay.

Internal code should use only:

- `action`
- `execution`
- `service`
- `runtime`
- `kernel`
- `capability`

No new internal APIs should introduce `mcp`, `tooling`, or top-level `process`.

## Suggested Verification Commands

Run after each phase:

```bash
rtk cargo check --manifest-path ai/Cargo.toml
rtk cargo test --manifest-path ai/Cargo.toml
rtk rg -n "Mcp|mcp|tooling|crate::process|process/" ai/src
```

For intermediate phases, the `rg` command should show only expected compatibility shims.

## Recommended First Pull Request

Scope:

- Add `api/action` re-exports.
- Add `capability/execution` re-exports.
- Add `runtime/action_transcript` re-exports.
- Add `service` re-exports.
- Add this plan.
- Do not move implementation yet.

Reason:

This establishes the target vocabulary in code without creating a large risky move-only diff. After that, type renames and file moves can happen in smaller reviewable steps.

## Migration Progress

### Completed

- Added `api::action` as the new protocol-neutral API facade.
- Added nested `api::action::{dispatch,jsonrpc,proxy,schema}` facades.
- Added `api::routes::supervisor::action` with `ai_action_*` route aliases.
- Added `capability::execution` as the new bounded-effect execution facade.
- Added initial `capability::execution::{action,graph,patch,process,shell}` facades.
- Added `runtime::action_transcript` as the action transcript facade.
- Added `service` as the service-layer facade for the old top-level `process` module.
- Added initial `service::{agent,dispatch,recovery,scheduler,supervisor}` facades.
- Added public crate-level aliases for action dispatch, action receipts, and action transcripts.
- Added `capability::execution::action::record` as the action record facade.
- Added protocol helper `action_authorization_submission`.
- Migrated API/lib boundary imports from `crate::process` to `crate::service`.
- Migrated selected action request/receipt consumers to import through `capability::execution`.
- Moved the action request/receipt implementation into `capability::execution::action::record`.
- Replaced `capability::tooling::record::mcp` with a legacy compatibility shim.
- Added first-class `AuthorizeActionCall` and `SubmitActionReceipt` command variants.
- Added worker API decode/proxy support for `AuthorizeActionCall` and `SubmitActionReceipt`.
- Switched live action execution to submit action command variants internally.
- Added `capability::execution::record` with action-owned shared receipt helpers.
- Moved the transcript implementation into `runtime::action_transcript`.
- Replaced `runtime::mcp_transcript` with a legacy compatibility facade.
- Changed the canonical transcript path to `state/agent_state/action/action-transcript.tlog.ndjson`.
- Kept legacy transcript replay fallback for `state/agent_state/mcp/mcp-transcript.tlog.ndjson`.
- Migrated live action dispatch transcript writes/reads to `runtime::action_transcript`.
- Changed raw action result sink to `state/agent_state/action/results.ndjson`.
- Moved live action dispatch implementation into `capability::execution::action::dispatch`.
- Replaced `capability::tooling::mcp_tools::dispatch` with a legacy compatibility facade.
- Switched supervisor route/state imports to `dispatch_action_request` and `ActionHost`.
- Removed production runtime dependencies on capability action receipt structs by introducing `ActionTranscriptReceiptFacts`.
- Removed runtime introspection dependency on capability eval decoding by parsing the stable eval verdict field locally.
- Removed runtime mailbox dependency on capability ids, registry ownership, and evidence submission construction.
- Moved mailbox evidence submission construction to `api::protocol` helpers.
- Made mailbox request construction accept a caller-supplied registry policy hash.
- Migrated selected internal API edge callers from `crate::api::mcp` to `crate::api::action` facade aliases while preserving external MCP compatibility.
- Migrated selected sandbox process request/receipt imports from `capability::tooling` to the `capability::execution` facade.
- Migrated selected graph public exports and graph tool callers from `capability::tooling` to `capability::execution::graph` / execution action facades.
- Migrated native workspace shell/patch callers from `capability::tooling::mcp_tools` to `capability::execution::{shell,patch}` facades.
- Migrated native project/agent action-tool callers from `capability::tooling::mcp_tools` to the `capability::execution::action` facade.
- Migrated action landmark/schema callers from `capability::tooling::mcp_tools::landmarks` to `capability::execution::action::landmarks`.
- Added `capability::execution::action::host` facade and migrated action dispatch off direct `capability::tooling::native` imports.
- Migrated graph editor internals from `capability::tooling::graph_patch_contract` paths to the `capability::execution::graph` facade.
- Migrated domain plan file I/O helpers from direct `crate::process::scheduler::plan_store` paths to the `crate::service::scheduler::plan_store` facade.

Verified with:

```bash
rtk env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER CARGO_TARGET_DIR=/tmp/canon-ai-cargo-check cargo check --manifest-path ai/Cargo.toml
rtk env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER CARGO_TARGET_DIR=/tmp/canon-ai-cargo-check cargo test --manifest-path ai/Cargo.toml runtime::action_transcript --lib
rtk env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER CARGO_TARGET_DIR=/tmp/canon-ai-cargo-check cargo test --manifest-path ai/Cargo.toml capability::tooling::mcp_tools::dispatch::transcript_tests --lib
rtk env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER CARGO_TARGET_DIR=/tmp/canon-ai-cargo-check cargo test --manifest-path ai/Cargo.toml capability::execution::action::dispatch::transcript_tests --lib
```

### Next Step

Move the gateway/action helper modules from `capability/tooling/mcp_tools`
into `capability/execution/action` in small groups:

- `landmarks.rs`
- `common.rs`
- `canon_spawn_agent.rs`
- `canon_send_agent_message.rs`
- `canon_read_mailbox.rs`
- `canon_plan.rs`
- `canon_score.rs`

Keep `capability/tooling/mcp_tools/*` as compatibility facades until all
internal imports use `capability::execution::action`.

## Handoff: How to Complete the Migration

This section is the actionable handoff for the remaining work. Keep changes
small and verify after each group. Always run Rust verification with wrappers
unset:

```bash
rtk env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER CARGO_TARGET_DIR=/tmp/canon-ai-cargo-check cargo check --manifest-path ai/Cargo.toml
```

### Current State

Already completed:

- `api::action` exists as the new protocol-neutral API facade.
- `capability::execution` exists as the new bounded execution facade.
- `capability::execution::action::record` owns `ActionCallRequest`, `ActionReceipt`, and `LiveActionExecutor`.
- `Command::AuthorizeActionCall` and `Command::SubmitActionReceipt` exist and are used by live action execution.
- `runtime::action_transcript` owns transcript implementation and writes to `state/agent_state/action/action-transcript.tlog.ndjson`.
- `runtime::mcp_transcript` is only a compatibility shim.
- `capability::execution::action::dispatch` owns live action dispatch.
- `capability::tooling::mcp_tools::dispatch` is only a compatibility shim.
- API/lib boundary imports have been moved from `crate::process` to `crate::service`.

Still intentionally present as compatibility:

- External `/ai/mcp` route and MCP JSON-RPC method names.
- `api::mcp` module.
- `capability::tooling` module.
- `capability::tooling::mcp_tools` module.
- `process` module.
- Legacy `Mcp*` type aliases and command tags.

### Required Verification

Run these frequently:

```bash
rtk env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER CARGO_TARGET_DIR=/tmp/canon-ai-cargo-check cargo check --manifest-path ai/Cargo.toml
rtk env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER CARGO_TARGET_DIR=/tmp/canon-ai-cargo-check cargo test --manifest-path ai/Cargo.toml runtime::action_transcript --lib
rtk env -u RUSTC_WRAPPER -u RUSTC_WORKSPACE_WRAPPER CARGO_TARGET_DIR=/tmp/canon-ai-cargo-check cargo test --manifest-path ai/Cargo.toml capability::execution::action::dispatch::transcript_tests --lib
```

Use these scans to check migration progress:

```bash
rtk rg -n "crate::process::|pub use crate::process" ai/src
rtk rg -n "capability::tooling::mcp_tools|capability::tooling::record|runtime::mcp_transcript" ai/src
rtk rg -n "Mcp|mcp|tooling|crate::process|process/" ai/src
```

For intermediate phases, the final scan may still show compatibility shims,
external protocol code, docs, and `std::process` usage. Do not treat those as
bugs unless they are internal imports that should have moved.

### Remaining Work Order

#### 1. Move action helper modules

Move these implementation files from:

```text
capability/tooling/mcp_tools/
```

to:

```text
capability/execution/action/
```

Recommended order:

1. `landmarks.rs`
2. `common.rs`
3. `canon_spawn_agent.rs`
4. `canon_send_agent_message.rs`
5. `canon_read_mailbox.rs`
6. `canon_plan.rs`
7. `canon_score.rs`

For each file:

1. Copy or move the implementation to `capability/execution/action/<name>.rs`.
2. Add `pub mod <name>;` in `capability/execution/action/mod.rs`.
3. Replace the old file with a shim:

```rust
//! Legacy MCP-named action compatibility facade.

pub use crate::capability::execution::action::<name>::*;
```

4. Update internal imports to prefer `crate::capability::execution::action::<name>`.
5. Run `cargo check`.

Special notes:

- `dispatch.rs` currently imports `crate::capability::tooling::mcp_tools::landmarks`; after moving `landmarks.rs`, update it to `crate::capability::execution::action::landmarks`.
- `canon_plan.rs` already imports scheduler through `crate::service::scheduler`; keep it that way.
- `canon_spawn_agent.rs` docs mention `McpToolHost`; update docs/comments to `ActionHost`.

#### 2. Move shell, patch, graph, and workspace implementations

Target structure:

```text
capability/execution/patch/
capability/execution/shell/
capability/execution/graph/
capability/execution/workspace.rs
capability/execution/project.rs
capability/execution/utility.rs
```

Move map:

| Current | Target |
| --- | --- |
| `capability/tooling/mcp_tools/apply_patch/*` | `capability/execution/patch/*` |
| `capability/tooling/mcp_tools/shell/*` | `capability/execution/shell/*` |
| `capability/tooling/patch.rs` | `capability/execution/patch/mod.rs` or `patch/contract.rs` |
| `capability/tooling/graph_artifacts.rs` | `capability/execution/graph/artifacts.rs` |
| `capability/tooling/graph_editor.rs` | `capability/execution/graph/editor.rs` |
| `capability/tooling/graph_patch_contract.rs` | `capability/execution/graph/patch_contract.rs` |
| `capability/tooling/native/workspace.rs` | `capability/execution/workspace.rs` |
| `capability/tooling/native/project.rs` | `capability/execution/project.rs` |
| `capability/tooling/native/utility.rs` | `capability/execution/utility.rs` |

Keep old files as shims until all imports are moved.

#### 3. Move remaining execution record modules

The action receipt implementation already moved. Remaining record modules still
under `capability/tooling/record` should move to `capability/execution/record`
or execution-specific folders:

| Current | Target |
| --- | --- |
| `record/artifact.rs` | `execution/record/artifact.rs` |
| `record/process.rs` | `execution/process/record.rs` |
| `record/receipt.rs` | `execution/record/receipt.rs` |
| `record/request.rs` | `execution/record/request.rs` |
| `record/hash.rs` | `execution/record/hash.rs` |
| `record/types.rs` | `execution/record/types.rs` |

Important:

- `execution/record/hash.rs` currently contains only action-needed helpers.
  Expand it carefully when moving process/artifact receipt code.
- Preserve old public exports from `capability::tooling` during transition.
- Keep `ActionReceipt` and `SandboxProcessReceipt` hashes stable. Do not change
  constants unless intentionally bumping a schema.

#### 4. Move `native` host implementation

Current:

```text
capability/tooling/native/
```

Target options:

```text
capability/execution/action/host.rs
capability/execution/workspace.rs
capability/execution/graph/mod.rs
capability/execution/project.rs
capability/execution/utility.rs
service/supervisor/action_host.rs
```

Guideline:

- Keep the `ActionHost` trait under capability execution.
- Put supervisor-specific host actions near `service/supervisor` if they need
  supervisor state, worker reloads, browser router calls, or process lifecycle.
- Keep bounded workspace, graph, patch, shell, and project effects under
  `capability/execution`.

#### 5. Rename `process` to `service` physically

Currently `service/*` is a facade over `process/*`. Complete this after the
capability execution moves are stable.

Move map:

| Current | Target |
| --- | --- |
| `process/agent/*` | `service/agent/*` |
| `process/dispatch/*` | `service/dispatch/*` |
| `process/recovery/*` | `service/recovery/*` |
| `process/scheduler/*` | `service/scheduler/*` |
| `process/supervisor/process.rs` | `service/supervisor/worker_runtime.rs` |
| `process/supervisor/runtime.rs` | `service/supervisor/runtime.rs` |
| `process/supervisor/state.rs` | `service/supervisor/state.rs` |
| `process/supervisor/workspace.rs` | `service/supervisor/workspace.rs` |
| `process/endpoints.rs` | `service/endpoints.rs` |

Then make `process/mod.rs` a compatibility shim:

```rust
//! Legacy process compatibility facade.

pub use crate::service::*;
```

Remove it only after all internal imports use `crate::service`.

#### 6. Rename API edge from `mcp` to `action`

This should be late because external clients may still depend on `/ai/mcp` and
MCP JSON-RPC method names.

Goal:

- `api/action/*` owns implementation.
- `api/mcp/*` is compatibility only.
- `api/routes/supervisor/action.rs` owns handler implementation.
- `api/routes/supervisor/mcp.rs` is compatibility only, or route alias only.

Keep the external `/ai/mcp` route unless explicitly removed from product scope.
The internal handler can still be action-named.

#### 7. Remove compatibility shims

Only after scans show no internal usage:

```bash
rtk rg -n "crate::process::|capability::tooling::|api::mcp::|runtime::mcp_transcript" ai/src
```

Remove in this order:

1. `capability/tooling/mcp_tools/*` shims.
2. `capability/tooling/record/*` shims.
3. `capability/tooling` module.
4. `process` module.
5. Internal `api/mcp` imports. Keep external route compatibility if required.
6. Legacy `Mcp*` aliases and command tags if external compatibility is no longer needed.

### Non-goals

- Do not change hash algorithms or schema versions during file moves.
- Do not remove `/ai/mcp` unless explicitly approved.
- Do not rewrite kernel state-machine semantics.
- Do not collapse `service` into `capability`; service owns long-lived lifecycle,
  capability owns bounded effects and receipts.

- Migrated agent config endpoint imports from `crate::process::endpoints` to `crate::service::endpoints`.
- Migrated loop HTTP config imports to `crate::service::agent::config`.
- Migrated prompt builder config imports to the service agent facade.
- Migrated loop receipt router imports to the service agent facade.
- Migrated DAG scheduler agent/scheduler imports to service facades.
- Migrated loop driver agent/scheduler references to service facades.
- Migrated task client loop HTTP imports to the service agent facade.
- Migrated agent worker task-client imports to the service dispatch facade.
- Migrated task runner agent/dispatch/scheduler imports to service facades.
### Quick Acceptance Criteria

The migration is complete when:

- New internal imports use `api::action`, `capability::execution`, `runtime::action_transcript`, and `service`.
- Live execution submits `AuthorizeActionCall` and `SubmitActionReceipt`.
- Transcript writes go to `state/agent_state/action/action-transcript.tlog.ndjson`.
- Raw action results go to `state/agent_state/action/results.ndjson`.
- `capability/tooling` and `process` are either deleted or pure compatibility facades.
- The only remaining `mcp` references are external compatibility route/protocol names and explicitly documented shims.
