# Architecture

The system is organized into five dependency layers. Each layer may only import from layers below it. No upward imports are permitted.

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
│   ├── policy/      ← versioned, append-only, read by all caps
│   ├── eval/        ← scores outcomes, produces EvalRecord
│   ├── learning/    ← reads TLog, promotes patterns into policy
│   └── orchestration/ ← parallel runs, prioritization, routing
└── api/             ← external surface, HTTP / gRPC, command intake
```

Policy exists before learning as a minimal read-only / append-only store:

```text
policy skeleton → eval signal → judgment reads policy → learning promotion
```

Learning may append verified entries into policy, but policy must not depend on learning. This keeps the policy surface available to judgment, LLM prompting, and evaluation before any adaptive learning loop exists.

## Core Properties

**Correctness.** The kernel is a pure deterministic function. Same input, same output, always. It has no I/O, no side effects, and no dependency on any layer above it. It is frozen by architectural commitment.

**Auditability.** Every state transition is recorded in a hash-chained transaction log. The log is append-only, replayable, and fully verifiable. Nothing the system does is unrecorded.

**Bounded recovery.** The system cannot loop forever. Recovery attempts are counted and capped. If the budget is exhausted the system halts cleanly with a full failure record. Convergence is guaranteed.

**Self-improvement.** The learning capability reads completed run history and promotes confident patterns into policy. Policy is versioned and append-only. No run is wasted — every outcome is a training signal for the next.

**LLM promotion.** The LLM begins as the primary reasoning engine inside capabilities. As policy grows, the LLM is called less for common cases and more for novel ones. Over time the LLM is promoted to specialist and eventually to architectural advisor — flagging where new capabilities are needed rather than doing routine work.

## Capability Build Order

Each capability depends on the ones below it being stable first.

1. tooling       — without this nothing real executes
2. planning      — without this objectives cannot be decomposed
3. observation   — without this the system cannot perceive the world
4. context       — without this every run starts blind
5. memory        — without this learning has nothing to query
6. policy        — minimal read-only / append-only store, empty at first
7. eval          — without this learning has no signal
8. llm           — adapter must enforce structured output
9. judgment      — reads policy and llm, produces JudgmentRecord
10. verification — semantic checking beyond hash validation
11. learning     — reads TLog, promotes patterns into policy
12. orchestration — parallel runs, only after single-thread is proven
