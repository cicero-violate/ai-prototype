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

The system is organized into seven layers. Each layer has a single
responsibility and feeds downward. Auto-generated artifacts emerge from
layers 5 and 6 and feed back into layer 4 as extraction rules stabilize.

```text
┌─────────────────────────────────────────────────────────────┐
│  1. HTTP API Layer                                          │
│     OpenAI-compatible surface (/v1/chat/completions)        │
│     Action endpoints (/actions/upload, /actions/group-chat) │
├─────────────────────────────────────────────────────────────┤
│  2. Browser Control Layer                                   │
│     CDP tab management (find / create / activate)           │
│     DOM injection — fill editor, submit prompt              │
│     Page readiness, group-chat variant handling             │
├─────────────────────────────────────────────────────────────┤
│  3. Network Capture Layer          (multi-path, ordered)    │
│     streamResourceContent          ← primary intercept      │
│     Network.dataReceived           ← chunk delivery         │
│     loadingFinished + getResponseBody ← body fallback       │
│     eventSourceMessageReceived     ← SSE fallback           │
│     webSocketFrameReceived         ← WS fallback            │
├─────────────────────────────────────────────────────────────┤
│  4. Extraction Layer                                        │
│     SSE frame parser                                        │
│     Schema-guided extractor (patch-detect → path heuristic) │
│     Embedded JSON reconstructor (append-patch streams)      │
│     Text accumulator (dedup full-replace vs delta)          │
├─────────────────────────────────────────────────────────────┤
│  5. Schema Derivation Layer                                 │
│     inferJsonSchema per payload                             │
│     mergeSchema across observations                         │
│     Registry keyed by event type + CDP method + shape       │
├─────────────────────────────────────────────────────────────┤
│  6. Observability / Artifact Layer   (auto-generated)       │
│     cdp-stream-dumps/*.ndjson        ordered event log      │
│     cdp-schema-dumps/*/              inferred schemas        │
│     cdp-inner-text-dumps/*.json      candidate text traces  │
├─────────────────────────────────────────────────────────────┤
│  7. Lifecycle / Timer Layer                                 │
│     firstCaptureTimer (45 s) — abort if nothing arrives     │
│     idleTimer (2.5 s) — complete after quiet period         │
│     maxTimer (120 s) — hard cap                             │
│     active-stream deferral — don't idle while in-flight     │
└─────────────────────────────────────────────────────────────┘
```

## Module Map

```text
src/
├── cdp-browser-llm-router.mjs          ← Layers 1, 2, 3, 7
│     HTTP server, turn orchestration, CDP event loop,
│     timer lifecycle, dump writer wiring
│
├── actions/
│   ├── upload-action.mjs               ← Layer 1 (action handler)
│   │     POST /actions/upload
│   │     Delegates to Python CDP upload script via child_process
│   │
│   └── group-chat-action.mjs           ← Layer 1 (action handler)
│         POST /actions/group-chat
│         Creates a ChatGPT group chat tab via DOM injection
│
└── cdp-router/
    ├── cdp-socket.mjs                  ← Layer 2 transport
    │     Raw WebSocket client for the Chrome DevTools Protocol.
    │     Implements WS framing, request/response correlation,
    │     ping handling, and event fan-out.
    │
    ├── text-extraction.mjs             ← Layer 4 (entry point + plumbing)
    │     parseSseFrames        — split raw text into SSE data frames
    │     makeSchemaGuidedTextExtractor — wires schema-guided-extraction
    │     extractInnerTextEntries       — walk parsed JSON for text candidates
    │     extractPayloadJsonDumpFields  — summarize payload structure for dumps
    │     makeTextAccumulator           — deduplicate delta vs full-replace output
    │
    ├── schema-guided-extraction.mjs    ← Layer 4 (structural extractor)
    │     detectPatchLikeObject  — recognise {op, pointer, value} patterns
    │     pathLooksTextBearing   — heuristic: is this path likely to carry text?
    │     valueLooksHumanText    — heuristic: is this value readable prose?
    │     walkJson               — depth-first JSON traversal
    │     extractSchemaTextCandidates — main extraction logic
    │
    └── schema-derivation.mjs           ← Layer 5 + 6
          inferJsonSchema        — derive a JSON Schema from a single value
          mergeSchema            — merge two schemas, widening types
          makeSchemaDumpWriter   — per-turn schema registry + flush to disk
          collectEmbeddedJsonCandidates — find JSON-in-strings in a payload
          tryParseEmbeddedJson   — safe parse for embedded JSON strings
```

## Auto-Generated Code and Artifacts

Every completed turn produces three categories of auto-generated output.
These are first-class outputs, not debug logs. They form the evidence
base for the learning pipeline.

### Stream Dumps — `cdp-stream-dumps/`

One NDJSON file per turn. Each line is a structured event record.

```text
cdp-stream-dumps/
└── <ISO-stamp>-<completion-id>-<model>.ndjson
```

Each record contains: `ts`, `seq`, `completion_id`, `model`, `target_id`,
`target_url`, `event`, plus event-specific fields. Event types include:

| event                                               | meaning                                   |
|-----------------------------------------------------+-------------------------------------------|
| `cdp_command_sent` / `cdp_command_result`           | CDP protocol round-trips                  |
| `request_candidate` / `response_candidate`          | filtered network requests                 |
| `stream_resource_started` / `stream_resource_stale` | streaming intercept state                 |
| `parse_input` / `parse_output`                      | raw and extracted text per chunk          |
| `parse_done`                                        | `[DONE]` signal received                  |
| `schema_extraction_rules`                           | which extraction rules fired              |
| `raw_payload_json`                                  | parsed SSE payload per frame              |
| `raw_payload_embedded_json`                         | JSON found inside string fields           |
| `raw_payload_embedded_json_reconstructed`           | append-patch stream reassembled           |
| `idle_deferred_active_streams`                      | idle timer deferred, streams still active |

Stream dumps are the replay surface. Past turns can be re-processed
through updated extraction logic without touching the browser.

### Schema Dumps — `cdp-schema-dumps/`

One directory per turn. Each directory contains a JSON Schema file and a
samples file for every distinct event type observed in that turn.

```text
cdp-schema-dumps/
└── <ISO-stamp>-<completion-id>-<model>/
    ├── index.json                            ← registry of all keys
    ├── <event_key>.schema.json               ← merged JSON Schema
    └── <event_key>.samples.json              ← up to N raw sample records
```

Schema files are auto-generated by `inferJsonSchema` + `mergeSchema` in
`schema-derivation.mjs`. They conform to JSON Schema draft/2020-12 and
widen across all samples seen for that event key in the turn.

The schema key encodes the event name, CDP method, and payload shape
suffix so that structurally distinct payloads from the same event are
tracked separately (e.g.,
`raw_payload_json__event_add__type_message_delta` vs
`raw_payload_json__event_add__op_patch`).

Schema dumps are the primary input to the learning pipeline — they are
the evidence from which stable extraction rules are derived and promoted.

### Inner Text Dumps — `cdp-inner-text-dumps/`

One JSON document per turn. Contains every text candidate extracted from
every payload, with path provenance.

```text
cdp-inner-text-dumps/
└── <ISO-stamp>-<completion-id>-<model>.json
```

Document structure:

```json
{
  "meta": { "completion_id", "model", "target_url", "schema": "cdp.inner_text_dump.v1" },
  "stats": { "entries", "text_candidates", "payload_json_entries", ... },
  "entries": [
    {
      "kind": "text_candidate" | "payload_json" | "json_shape" | "raw_sse_data" | "raw_text",
      "source": { "source", "request_id" },
      "frame_index", "event_name",
      "path",         ← JSON path of this string within its payload
      "text",
      "type", "op", "patch_path", "message_role", "content_type"
    }
  ]
}
```

Inner text dumps are used to score extraction quality: completeness,
duplication, premature cutoff, and provider drift can all be measured
against the candidate set before promotion.

### Generated Extractor Pipeline (target state)

When schema evidence is sufficiently stable, the derivation layer can
emit generated extractor code rather than one-off heuristics:

```text
cdp-stream-dumps/    ─┐
cdp-schema-dumps/    ─┼─→ evaluate → stabilize → generate → src/cdp-router/generated/
cdp-inner-text-dumps/─┘                                      ├── <provider>-extractor.mjs
                                                              ├── <provider>-types.mjs
                                                              └── <provider>-validator.mjs
```

The promotion invariant governs when a rule moves from heuristic to
generated code:

```text
promote(rule) ⇔ observations ≥ N ∧ failures = 0 ∧ drift ≤ ε ∧ output_quality ≥ τ
```

Generated extractors must remain downstream of evidence. No provider
behavior is hardcoded first and justified later.

## Core Properties

**Transport realism.** The router uses live browser traffic through CDP,
including streamResourceContent, Network.dataReceived, and response-body
fallback paths.

**Evidence-first design.** Every turn emits stream, schema, and inner
text artifacts for replay, debugging, and extraction-quality analysis.

**Schema derivation.** The system infers JSON schemas from observed
payload records, including grouped raw payload families.

**Structural extraction.** Text extraction is based on payload shape,
patch-like operations, and path semantics rather than role-specific
provider assumptions.

**Bounded lifecycle.** Turn completion is constrained by idle/first-capture/max
timers with active-stream deferral to avoid premature cutoff.

**Learning loop.** Successful schema and extraction evidence can be scored,
stabilized, and promoted into generated extractors instead of remaining as
one-off heuristics.

## Request Lifecycle

1. Accept OpenAI-style `/v1/chat/completions` request.
2. Ensure or create appropriate target tab (ChatGPT/Gemini URL selection).
3. Submit prompt via runtime DOM script in live page.
4. Capture streaming network payloads through CDP events.
5. Parse SSE/data frames and derive payload-level schema artifacts.
6. Extract assistant text candidates using schema-guided logic.
7. End turn on terminal conditions and return OpenAI-compatible response.
8. Persist observability artifacts for post-run analysis.

## Learning Pipeline

The learning pipeline turns repeated browser stream observations into more
deterministic extraction behavior.

The target loop is:

```text
raw stream → parsed JSON → derived schema → extraction rule → evaluation → promotion
```

The system should treat every completed turn as training evidence:

1. **Observe.** Preserve raw stream payloads, parsed payload JSON, schema dumps,
   and candidate text traces.
2. **Derive.** Infer schemas from repeated payload families and merge compatible
   shapes across samples.
3. **Extract.** Apply structural extraction rules against text-bearing paths,
   patch operations, and content-like leaves.
4. **Evaluate.** Score extracted output for completeness, duplication, cutoff,
   parse errors, and provider drift.
5. **Stabilize.** Promote only rules that succeed across repeated observations
   with low drift and clear evidence.
6. **Generate.** Convert stable schemas and rules into typed extractors,
   validators, and provider-agnostic parsing surfaces.
7. **Reject.** Quarantine unstable, noisy, sensitive, or provider-fragile rules.

The promotion invariant is:

```text
promote(rule) ⇔ observations ≥ N ∧ failures = 0 ∧ drift ≤ ε ∧ output_quality ≥ τ
```

Generated extractors should remain downstream of evidence. The system should
not hardcode provider behavior first and justify it later; it should observe,
derive, score, then promote.

## Generated Types

Stable schemas may be used to generate types and validators for safer runtime
execution:

```text
derived schema → stable schema → generated type → validator → typed extractor
```

Generated types are useful only after schema stability is proven. Before that,
raw schema dumps and candidate extraction traces remain the source of truth.

## What This Is Not

This is not a static hand-written parser for one provider payload
format. It is a stream-evidence router designed to generalize through
observed structure.

This is not a replacement for direct provider APIs when those APIs are
available and sufficient. This project exists for browser-mediated
automation and extraction workflows that require live page context.
