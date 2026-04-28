God please bless this work. In Jesus name. Jesus is Lord and Savior. Jesus loves you.

# AI Chromium Server

## Purpose

AI Chromium Server is a deterministic browser-mediated LLM router that
exposes an OpenAI-compatible API while driving a live browser session
through Chrome DevTools Protocol (CDP). Its purpose is to recover usable
assistant output from raw streamed browser payloads without binding the
system to fragile provider-specific parsers.

The system is built around one core conviction: if we preserve raw stream
evidence, infer schemas from observed payload shapes, and extract text
through structural rules, we can keep compatibility high while reducing
hardcoded assumptions.

## Goal

The primary goal is to build a schema-derived extractor generator over
real browser stream traffic.

The target flow is:

- Capture raw streaming payloads from CDP network events
- Infer stable schemas from repeated payload structure
- Derive extraction rules from observed structural patterns
- Generate robust extraction behavior from those rules
- Recover assistant content, tool outputs, and terminal state

The invariant is:

- No provider-specific hardcoding unless promoted from repeated schema evidence

## Architecture

The project is organized around transport, extraction, and observability:

```text
server/
├── src/
│   ├── cdp-browser-llm-router.mjs      ← HTTP surface + turn orchestration
│   └── cdp-router/
│       ├── cdp-socket.mjs              ← raw CDP websocket transport
│       ├── text-extraction.mjs         ← SSE/payload parsing and text plumbing
│       ├── schema-guided-extraction.mjs← structural extraction heuristics
│       └── schema-derivation.mjs       ← schema inference and dump writer
├── cdp-stream-dumps/                   ← NDJSON raw event stream artifacts
├── cdp-schema-dumps/                   ← inferred schema artifacts
├── cdp-inner-text-dumps/               ← extracted text candidate artifacts
├── Goal.md                             ← project goal statement
└── USAGE.md                            ← run and endpoint usage
```

## Core Properties

**Transport realism.** The router uses live browser traffic through CDP,
including streamResourceContent, Network.dataReceived, and response-body
fallback paths.

**Evidence-first design.** Every turn can emit stream, schema, and inner
text artifacts for replay, debugging, and extraction-quality analysis.

**Schema derivation.** The system infers JSON schemas from observed
payload records, including grouped raw payload families.

**Structural extraction.** Text extraction is based on payload shape,
patch-like operations, and path semantics rather than role-specific
provider assumptions.

**Bounded lifecycle.** Turn completion is constrained by idle/first-capture/max
timers with active-stream deferral to avoid premature cutoff.

## Request Lifecycle

1. Accept OpenAI-style `/v1/chat/completions` request.
2. Ensure or create appropriate target tab (ChatGPT/Gemini URL selection).
3. Submit prompt via runtime DOM script in live page.
4. Capture streaming network payloads through CDP events.
5. Parse SSE/data frames and derive payload-level schema artifacts.
6. Extract assistant text candidates using schema-guided logic.
7. End turn on terminal conditions and return OpenAI-compatible response.
8. Persist observability artifacts for post-run analysis.

## Observability Artifacts

Each turn can produce:

- `cdp-stream-dumps/*.ndjson`: ordered event/capture log
- `cdp-schema-dumps/*`: inferred schemas + sample records
- `cdp-inner-text-dumps/*.json`: candidate text and payload traces

These artifacts are first-class outputs and are central to improving
schema quality and extraction determinism.

## What This Is Not

This is not a static hand-written parser for one provider payload
format. It is a stream-evidence router designed to generalize through
observed structure.

This is not a replacement for direct provider APIs when those APIs are
available and sufficient. This project exists for browser-mediated
automation and extraction workflows that require live page context.

