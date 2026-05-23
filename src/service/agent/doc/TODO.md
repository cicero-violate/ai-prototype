# Agent TODO

## Stage 0: Specification Only

- [x] Create `src/agent/` directory.
- [x] Keep it unwired from `src/lib.rs`.
- [x] Document relationship to generic external agent loops.
- [x] Document relationship to `router-server`.
- [ ] Decide whether the first agent implementation is local-only, worker-API-only, or hybrid.
- [ ] Decide whether the Rust agent loop owns HTTP client code or uses existing API transport directly.

## Stage 1: Records Before Behavior

- [ ] Define conceptual `AgentObjective`.
- [ ] Define conceptual `AgentStep`.
- [ ] Define conceptual `AgentDecision`.
- [ ] Define conceptual `AgentAction`.
- [ ] Define conceptual `AgentReceipt`.
- [ ] Define conceptual `AgentRunSummary`.
- [ ] Define deterministic hash identity for each record.
- [ ] Define schema versions before serialization.

## Stage 2: Read-Only Driver

- [ ] Build a read-only driver that observes `State` and `TLog`.
- [ ] It must not submit commands.
- [ ] It must produce a proposed next step only.
- [ ] It must explain why a gate or capability is selected.
- [ ] It must detect terminal states.
- [ ] It must detect convergence risk.

## Stage 3: Command-Producing Driver

- [ ] Convert proposed steps into `CommandEnvelope` values.
- [ ] Submit through `ApiTransportSession` or worker HTTP API.
- [ ] Verify idempotent replay through `CommandLedger`.
- [ ] Verify TLog after every submitted command.
- [ ] Reject any effect without a capability receipt.

## Stage 4: Router-Server / External Loop Bridge

- [ ] Define how Rust agent requests an LLM turn through router-server.
- [ ] Define how an external agent loop can hand off state to Rust.
- [ ] Define how SSE/chunk logs map to `ObservationRecord`.
- [ ] Define how ChatGPT/MCP tool approvals become receipts.
- [ ] Define retry policy for incomplete router turns.

## Stage 5: MCP Tool Bridge

- [ ] Use `McpCallReceipt` for MCP tool effects.
- [ ] Require allowlisted tools.
- [ ] Require max output bytes.
- [ ] Require timeout budgets.
- [ ] Require replay verification against TLog.
- [ ] Block unreceipted side effects.

## Stage 6: Learning Bridge

- [ ] Feed verified completed runs into eval.
- [ ] Require eval pass before policy promotion.
- [ ] Require proof hash before distillation.
- [ ] Export distillation rows only from verified TLogs.
- [ ] Separate reusable policy from raw chat transcripts.

## Stage 7: Supervisor Integration

- [ ] Decide whether agent loop runs inside worker, supervisor, or separate binary.
- [ ] Prefer separate binary if the loop can self-modify worker code.
- [ ] Worker should remain command-driven.
- [ ] Supervisor should remain lifecycle-driven.
- [ ] Agent should remain decision/driver-driven.

## Safety

- [ ] No direct kernel mutation.
- [ ] No direct TLog append outside canonical writer paths.
- [ ] No tool execution without receipt.
- [ ] No LLM output accepted without structured record or receipt.
- [ ] No live finance/trading execution.
- [ ] No self-modification without supervisor reload boundary.
- [ ] No policy promotion without verified eval.
