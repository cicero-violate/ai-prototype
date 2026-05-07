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
- Verified TLog receipts can be distilled into training-ready datasets
  for smaller, faster specialist models
- The expected cost per recurring objective falls over time as verified
  policy coverage increases

## Verified Evolution Loop

Use an AlphaEvolve-style loop: generate code candidates, run them, score
them, keep the winners, and feed the winners back into the next prompt.

```text
seed program
→ prompt sampler
→ LLM mutation/crossover
→ candidate patch
→ sandbox run
→ evaluator
→ fitness score
→ program database
→ winner selection
→ next generation
```

Real names:

- **seed program**		: the current code, plan, policy, or action sequence.
- **candidate patch**	: one proposed mutation of the seed.
- **sandbox run**		: isolated execution of the candidate.
- **evaluator**			: tests, replay, proof checks, benchmarks, and eval rules.
- **fitness score**		: measured quality, not model opinion.
- **program database**	: archive of candidates, scores, receipts, and lineage.
- **selection**			: keep candidates that pass and improve the score.
- **next generation**	: use selected winners as context for more mutations.

Authority rule:

```text
LLM proposes candidates.
Sandbox executes candidates.
Evaluator scores candidates.
TLog records receipts.
Program database stores lineage.
Kernel gates deployment.
```

The LLM never approves itself. A candidate becomes learning data only when
external evidence proves it passed. If the evaluator is subjective or can be
steered by the same model that proposed the candidate, the loop is not verified
evolution; it is only model self-review.

## TLog Distillation Loop

After verified evolution, convert only winning traces into learning data.

```text
winning candidate → receipt/proof → TLog → distill.jsonl → policy/student → gated reuse
```

Dataset rule:

```text
D = { E in TLog |
      E.eval.verdict = pass
  and E.replay.valid = true
  and E.score >= threshold
  and E.inputs are retained
  and E.outputs are semantically inspectable }
```

Hashes, receipts, and proof IDs prove lineage. They are not training signal by
themselves. Training rows must retain the semantic input, selected action,
observable output, measured score, proof hash, and source event.

Each `distill.jsonl` row must keep:

```text
(instruction, input_state, action, output, score, proof_hash, source_event)
```

Use the data in this order:

1. Promote simple repeated wins into policy.
2. Keep hard or novel wins as examples for retrieval.
3. Train a small student model only after the dataset is large and clean.
4. Serve the student through Ollama or another host.
5. Keep the same verifier, proof, replay, and kernel gates.

Ollama is not the student. Ollama is only a local model host. The student
is the trained small model or adapter.

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

Learning may append verified entries into policy, but policy must not depend on
learning. This keeps the policy surface available to judgment, LLM prompting,
and evaluation before any adaptive learning loop exists.

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
6. policy        — minimal read-only / append-only store, empty at first
7. eval          — without this learning has no signal
8. llm           — adapter must enforce structured output
9. judgment      — reads policy and llm, produces JudgmentRecord
10. verification — semantic checking beyond hash validation
11. learning     — reads TLog, promotes patterns into policy
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

## Live Ollama Judgment Path

The deterministic LLM adapter remains available for replayable tests, but the
agent also includes a real local Ollama/OpenAI-compatible executable path.

```bash
export CANON_OLLAMA_BASE_URL=http://127.0.0.1:11434/v1
export CANON_OLLAMA_MODEL=qwen2.5-coder:7b
cargo run --example ollama_judgment
```

This path runs:

```text
agent runtime → observation → context → Ollama /v1/chat/completions → LlmRecord → Judgment gate → TLog receipt
```

## Graph-as-Source-of-Truth

`graph.json` is the canonical semantic representation of the codebase. The
canon-rustc-v3 compiler wrapper captures it during every `cargo check`. It
is not a secondary artifact — it is the authoritative index of what exists,
how it is connected, and what behavior it exhibits.

### Schema

```text
meta     — crate name, schema version, node/edge counts, content hashes
nodes    — path → { def_id, kind, def: { file, line, col, lo, hi } }
edges    — [ { relation, from, to } ]   (call | alloc | mut | io | panic | unsafe | impl)
intents  — path → label                 (pure | mutation | ...)
```

Every node carries source byte offsets (`lo`, `hi`) and a file path. These
are the anchors for all round-trip patch generation. No source location is
inferred — it is recorded at capture time by the compiler.

### Mutation Contract

A mutation is a typed operation on the graph that has a deterministic
corresponding source patch:

```text
RemoveNode(path)
  → excise bytes [lo, hi] from def.file
  → remove all edges where from == path or to == path
  → re-run wrapper → assert node absent from next graph

RetypeIntent(path, new_label)
  → update intents[path] in graph
  → optionally annotate source with #[allow(...)] / #[must_use] at def.line
  → re-run wrapper → assert intent updated

AddAttribute(path, attr)
  → insert attr text at def.line - 1 in def.file
  → re-run wrapper → assert attribute visible in next graph

RemoveEdge(from, to, relation)
  → locate the call/use at the from node's source span
  → generate source patch removing that specific use
  → re-run wrapper → assert edge absent from next graph
```

### Round-Trip Flow

```text
source code
  ↓  cargo check (canon-rustc-v3 wrapper)
graph.json                ← source of truth
  ↓  mutation agent (separate project)
mutation set [ { op, path, args } ]
  ↓  patch generator (reads lo/hi from graph.json nodes)
source patch (unified diff)
  ↓  apply_patch (chatgpt-mcp-connector)
patched source code
  ↓  cargo check (canon-rustc-v3 wrapper)
new graph.json            ← verify mutation landed
  ↓  diff old graph ↔ new graph
mutation receipt          ← TLog entry
```

The external mutation/query agent never touches source directly. It emits
typed operations against the graph. The patch generator translates those
operations into source patches using the byte offsets already in the graph.
The wrapper re-runs and produces a new graph. The diff between the two
graphs is the mutation receipt.

### Separation of Concerns

```text
this project (ai/)
  — produces graph.json via canon-rustc-v3 wrapper
  — defines graph.json schema and version contract
  — defines the mutation operation set and their source-patch semantics
  — defines the verification contract (re-capture + graph diff = receipt)

mutation/query project (separate)
  — queries graph.json to identify targets
  — emits typed mutation operations
  — drives the patch-apply → re-capture → verify loop
  — records mutation receipts into TLog
```

The only shared contract between the two projects is the graph.json schema
version and the typed mutation operation set defined here.
