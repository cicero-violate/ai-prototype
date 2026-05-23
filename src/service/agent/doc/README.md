# Agent Architecture

Purpose: define the future in-crate agent loop layer for Canon Agent.

This directory is intentionally **not wired into `lib.rs`**. It is a design and
contract surface for an autonomous driver that can eventually sit above the
existing deterministic runtime, capability system, API transport, worker, and
router-server integration.

## Current Status

```text
status: draft
wiring: none
crate_surface: none
runtime_effect: none
```

The current `ai` crate already has:

```text
kernel      → deterministic state model
runtime     → tick/reduce/replay/recovery
capability  → evidence producers and receipts
api         → command ingress and transport session
worker      → HTTP surface around ApiTransportSession
learning    → verified promotion and distillation
```

The missing piece is:

```text
agent       → autonomous observe/decide/act/verify loop
```

## External Agent Loop Relationship

Any external agent loop can interact with the running router-server. One local
example lives at:

```text
/workspace/ai_sandbox/canon-mini-agent/prototype/chatgpt-agent-loop/
```

External loops drive model/browser turns through the OpenAI-compatible router-server:

```text
/workspace/ai_sandbox/canon-mini-agent/prototype/router-server/
```

The future `src/agent/` layer should not assume a specific external loop
implementation. It should define the Rust-side contract for an internal agent
driver that can either:

- consume commands from any external agent loop,
- call the worker API,
- call an LLM provider through existing capability clients,
- call MCP/tooling through receipt-producing capability records,
- submit evidence through the existing API path,
- verify/replay all results.

## Boundary Rule

```text
Agent
  = loop policy + step selection + objective management + driver coordination

Capability
  = evidence production + rich records + receipts + tool/LLM effects

API / Worker
  = command ingress + idempotent transport + durable state mutation

Runtime
  = deterministic tick/reduce/replay/recovery

Kernel
  = stable phases, gates, evidence tokens, packet, state
```

The agent layer must not directly mutate `State`, `GateSet`, `Packet`, or `TLog`.
It should submit work through `CommandEnvelope`, `ApiTransportFrame`, or future
capability adapters.

## Target Flow

```text
[Objective]
      ↓
[Agent Loop]
  observe state
  decide next step
  select capability/tool/LLM path
  collect receipt
  submit evidence
  verify TLog
  update loop memory/policy intent
      ↓
[Worker API / ApiTransportSession]
      ↓
[Runtime tick]
      ↓
[TLog]
      ↓
[Verification / Eval / Learning]
```

## Proposed Files

```text
src/agent/
  README.md              # this overview
  mod.rs                 # commented Rust-facing sketch; not wired
  TODO.md                # implementation checklist
  architecture.md        # layer and dataflow model
  loop_contract.md       # step lifecycle and invariants
  router_integration.md  # relationship to router-server and external agent loops
  safety.md              # boundaries, no-bypass rules, risk controls
```

## Proposed Future Rust Modules

Not declared yet:

```text
src/agent/
  objective.rs
  loop.rs
  driver.rs
  step.rs
  observation.rs
  planner.rs
  executor.rs
  verifier.rs
  supervisor.rs
```

## Non-Goals For This Draft

- No `pub mod agent` in `src/lib.rs`.
- No changes to `kernel`.
- No changes to `runtime::reduce`.
- No direct tool execution bypassing capability receipts.
- No direct LLM calls that skip `LlmRecord`/receipt paths.
- No hidden mutation outside `ApiTransportSession` or durable runtime paths.
- No live trading or finance execution path.

## Agent Loop Equation

```text
agent_step = observe(state)
           → decide(policy, objective, gates)
           → act(capability/tool/llm)
           → submit(evidence)
           → verify(tlog)
           → evaluate(result)
           → learn(only if verified)
```

## TODO Summary

- [ ] Stabilize the agent loop contract before adding Rust behavior.
- [ ] Define `AgentObjective` and `AgentStep` records.
- [ ] Define how the agent observes worker state.
- [ ] Define how the agent selects capability producers.
- [ ] Define how router-server and external agent loops are represented.
- [ ] Define receipt requirements for all external effects.
- [ ] Define stop conditions and convergence limits.
- [ ] Define human-review boundaries.
- [ ] Define tests that prove the agent cannot bypass runtime verification.
