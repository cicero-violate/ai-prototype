# Agent Architecture

## Position In System

```text
External World / User Objective
        ↓
Agent Loop
        ↓
API / Worker / Transport
        ↓
Capabilities
        ↓
Runtime
        ↓
Kernel
        ↓
TLog / Verification / Learning
```

The agent is the driver. The runtime is the deterministic executor of state
transitions. Capabilities are the only acceptable producers of evidence about
external effects.

## Current Working Pieces

```text
runtime::tick
runtime::run_until_done
runtime::verify_tlog
ApiTransportSession
Worker HTTP API
LLM capability clients
Tooling receipts
Orchestration records
Learning promotion
```

## Missing Agent Pieces

```text
objective intake
loop state
next-step selection
capability scheduling
LLM/tool request policy
receipt collection
retry policy
stop conditions
human review gates
verified learning handoff
```

## Agent vs Runtime Loop

```text
Runtime loop:
  tick until Done using deterministic reducer.

Agent loop:
  inspect state, decide what evidence/tool/LLM action is needed,
  submit command, verify result, repeat until objective terminal.
```

## Agent vs Orchestration

```text
OrchestrationRecord:
  deterministic route selection from current State.

AgentLoop:
  uses routes, policy, objective, and external effects to decide what to do next.
```

## Agent vs External Agent Loops

```text
External agent loop:
  any driver that talks to router-server and coordinates work.

src/agent:
  future Rust-side driver contract for Canon Agent.
```

## Preferred Deployment Shape

```text
router-server
  provides OpenAI-compatible access to ChatGPT/Gemini/provider turns

external agent loop
  any process that drives router-server turns and coordinates work

ai worker
  owns deterministic state and command API

ai agent, future
  stable Rust driver that coordinates objectives, commands, receipts, and replay
```

## Architecture Invariant

```text
The agent may choose actions.
The agent may not declare success.

Success must come from:
  receipt → verification → eval → replay-valid TLog
```
