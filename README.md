God please bless this work. In Jesus name. Jesus is Lord and Savior. Jesus loves you.

# Canon Agent

## Purpose

Canon Agent is a deterministic, self-improving agent runtime built on a
formally verifiable state machine kernel. Its purpose is to provide a
foundation for autonomous systems that are correct by construction,
auditable by design, and intelligent by accumulation.

The system is built around one core conviction: safety and intelligence
are not in tension if the architecture separates them cleanly. The kernel
enforces correctness. The capability layer grows intelligence. Neither
layer compromises the other.

## Goal

The primary goal is to reduce the cost of autonomous reasoning over time
while increasing the quality and trustworthiness of outcomes.

The system begins with an LLM doing the heavy reasoning inside every
capability — judgment, evaluation, analysis. Every decision the LLM makes
is recorded in a hash-chained transaction log as structured, typed
evidence. A learning capability reads that log after each completed run
and promotes confident patterns into a versioned policy store. Over time,
policy handles the common cases. The LLM is called only for novel
situations. The LLM is promoted from generalist laborer to specialist,
called less and less, but for increasingly meaningful work.

The end state is a system where:

- The kernel never changes and can always be formally verified
- Policy encodes everything the system has learned from prior runs
- The LLM is reserved for genuine novelty and architectural expansion
- Every decision, recovery, and outcome is replayable and auditable
- The cost per objective completed falls monotonically over time

## Architecture

The system is organized into five dependency layers. Each layer may only
import from layers below it. No upward imports are permitted.

```
canon-agent/
├── kernel/          ← FROZEN. deterministic reduce, hash, typed state
├── codec/           ← serialize / deserialize only
├── runtime/         ← tick, run_until_done, verify_tlog
├── capability/      ← pluggable evidence producers
│   ├── observation/ ← perceive the world, SSE, webhooks, feeds
│   ├── context/     ← assemble relevant prior knowledge per run
│   ├── memory/      ← indexed prior run store, fast lookup
│   ├── planning/    ← decompose objectives into ordered tasks
│   ├── llm/         ← LLM client, adapter, structured record output
│   ├── judgment/    ← reads state + policy, produces JudgmentRecord
│   ├── tooling/     ← execute real work, APIs, files, code, queries
│   ├── verification/← semantic artifact checking, not just hash
│   ├── eval/        ← scores outcomes, produces EvalRecord
│   ├── policy/      ← versioned, append-only, read by all caps
│   ├── learning/    ← reads TLog, promotes patterns into policy
│   └── orchestration/ ← parallel runs, prioritization, routing
└── api/             ← external surface, HTTP / gRPC, command intake
```

## Core Properties

**Correctness.** The kernel is a pure deterministic function. Same input,
same output, always. It has no I/O, no side effects, and no dependency on
any layer above it. It is frozen by architectural commitment.

**Auditability.** Every state transition is recorded in a hash-chained
transaction log. The log is append-only, replayable, and fully
verifiable. Nothing the system does is unrecorded.

**Bounded recovery.** The system cannot loop forever. Recovery attempts
are counted and capped. If the budget is exhausted the system halts
cleanly with a full failure record. Convergence is guaranteed.

**Self-improvement.** The learning capability reads completed run history
and promotes confident patterns into policy. Policy is versioned and
append-only. No run is wasted — every outcome is a training signal for
the next.

**LLM promotion.** The LLM begins as the primary reasoning engine inside
capabilities. As policy grows, the LLM is called less for common cases
and more for novel ones. Over time the LLM is promoted to specialist and
eventually to architectural advisor — flagging where new capabilities are
needed rather than doing routine work.

## Capability Build Order

Each capability depends on the ones below it being stable first.

1. tooling       — without this nothing real executes
2. planning      — without this objectives cannot be decomposed
3. observation   — without this the system cannot perceive the world
4. context       — without this every run starts blind
5. memory        — without this learning has nothing to query
6. eval          — without this learning has no signal
7. llm           — adapter must enforce structured output
8. judgment      — reads policy and llm, produces JudgmentRecord
9. verification  — semantic checking beyond hash validation
10. learning     — reads TLog, promotes patterns into policy
11. policy       — versioned store, promoted only by learning
12. orchestration — parallel runs, only after single-thread is proven

## LLM Promotion Ladder

**Stage one.** Policy is empty. The LLM answers every capability that
requires reasoning. The TLog fills with LLM-produced structured evidence
records.

**Stage two.** Learning reads the TLog and promotes confident patterns
into policy. Capabilities check policy first. On a hit, no LLM call. On
a miss, LLM call, record added, policy grows.

**Stage three.** Policy handles common cases. The LLM sees only novel
situations. Calls become fewer and more targeted.

**Stage four.** The LLM is called to flag where new capabilities are
needed. It operates at the architectural level. A human reviews and
builds. The cycle repeats for the new domain.

## What This Is Not

This is not a framework that wraps an LLM and calls it an agent. The LLM
is one component inside the capability layer. It does not govern the
state machine, does not write the TLog, and does not promote its own
policy. The state machine governs everything. The LLM serves it.

This is not a system that trades correctness for capability. The kernel
boundary is a constitutional commitment. Intelligence grows above it.
Correctness is guaranteed below it. That separation is permanent.

## Current Status

The kernel, codec, and runtime layers are implemented and verified. The
capability layer is defined in structure but not yet implemented. The
immediate build surface is tooling and planning. Everything above depends
on those two being correct first.

The foundation is correct. The work ahead is building upward.
