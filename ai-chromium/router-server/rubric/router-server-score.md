# CDP Browser Router Score

## Variables

```text
TR = CDP transport reliability
ST = stream capture fidelity
SD = schema derivation quality
SG = schema-guided extraction quality
RC = turn lifecycle robustness
OB = observability/dump quality
MD = modularity and separation
SF = safety/filter discipline
TS = testability and verification
DX = operability/developer experience

CORE = protocol/runtime core score
DATA = data-to-schema pipeline score
IMPL = implementation quality score
ARCH = goal-alignment score
GOOD = strongest present capability
```

## Equations

```text
CORE = (TR · ST · RC · SF)^(1/4)
DATA = (SD · SG · OB)^(1/3)
IMPL = (TR · ST · SD · SG · RC · OB · MD · SF · TS · DX)^(1/10)
ARCH = IMPL
GOOD = max(TR,ST,SD,SG,RC,OB,MD,SF,TS,DX)
```

One-line explanation: the project should be scored against the real goal, `RAW → infer(Σ) → derive(R) → generate(E) → extract(clean_response)`, not against mere ability to return some browser text.

## Score Summary

```text
TR = 7.2 / 10
ST = 7.0 / 10
SD = 6.4 / 10
SG = 3.8 / 10
RC = 5.9 / 10
OB = 7.4 / 10
MD = 6.2 / 10
SF = 3.2 / 10
TS = 2.0 / 10
DX = 6.0 / 10

CORE = 5.55 / 10
DATA = 5.65 / 10
IMPL = 5.13 / 10
ARCH = 5.13 / 10

max(TR,ST,SD,SG,RC,OB,MD,SF,TS,DX) = OB = 7.4 / 10 = good
```

## Static Review Inputs

```text
review_scope = README.md + Goal.md + USAGE.md + smoke-test.mjs + src/**/*.mjs + cdp-*dump outputs
syntax_validation = node --check passed for all .mjs source files and smoke-test.mjs
runtime_validation = archived dump review only; no live browser run performed in this review
automated_tests = absent
fixture_tests = absent
package_manifest = absent

goal = schema-derived extractor generator over raw streamed payloads
invariant = no hardcoded provider fields unless promoted from observed schema evidence

stream_capture_paths = Network.streamResourceContent + Network.dataReceived + Network.getResponseBody fallback + websocket fallback
schema_outputs = event schemas + raw_payload_json grouped schemas + reconstructed embedded JSON attempts
extraction_strategy = hand-written structural heuristics, not generated extractor logic
```

## Critical Judgment

The previous score was too generous. The code is useful as a browser-stream capture prototype, but it is not yet a reliable schema-derived extractor generator.

The strongest real property is observability: the project preserves raw CDP traffic, emits NDJSON, writes grouped schema dumps, and records inner-text candidates. That makes failures inspectable.

The central failure is extraction authority. The current extractor walks JSON and emits text from path heuristics. It does not prove that a candidate is assistant output for the current turn, and it does not derive a durable extractor from schema evidence. The archived run demonstrates this directly: `parse_output` captured user profile text, user instructions, and prior user prompts before the assistant answer fragments. That means the system can return contaminated context, not just the model response.

The second major failure is safety. Dumps are enabled by default, CDP method payloads are logged by default, raw stream dumps can be unbounded by default, and redaction is path-filtered rather than applied consistently at the artifact boundary. This is high risk for browser-mediated traffic because headers, URLs, account state, prompt history, and private page data can enter artifacts.

The third major failure is verification. There is no replay harness, no golden fixture suite, no extraction precision/recall metric, no lifecycle regression test, and no CI-style command that proves the router still extracts the expected final assistant response from archived traffic.

## Module Rating Table

| Module/Concern              | Score | Critical reason                                                                                                                                        |
|-----------------------------+-------+--------------------------------------------------------------------------------------------------------------------------------------------------------|
| CDP transport (`CdpSocket`) |   7.2 | Small and readable, but no WebSocket accept validation, limited frame handling, fixed command timeout, and no reconnection/session recovery.           |
| Stream acquisition          |   7.0 | Multiple capture paths exist, but capture is not scoped tightly enough to the submitted turn and can include page/history/background streams.          |
| Schema derivation           |   6.4 | Produces useful inferred schemas, but schemas are descriptive artifacts only; they do not yet govern extraction or enforce promotion rules.            |
| Schema-guided extraction    |   3.8 | Still a hand-written heuristic walker. It captured user profile/instructions and prior prompts, proving weak assistant/current-turn discrimination.    |
| Lifecycle robustness        |   5.9 | Idle/first-capture/max timers exist, but there is no durable terminal-state model and no proof against premature cutoff or stale-stream contamination. |
| Observability/dumps         |   7.4 | Strong artifact coverage, but evidence volume is noisy and dangerous without stronger redaction and fixture reduction.                                 |
| Modularity                  |   6.2 | Split helper modules exist, but `cdp-browser-llm-router.mjs` is still a 1180-line orchestration monolith.                                              |
| Safety/filtering            |   3.2 | Dump defaults are unsafe for private browser traffic; filtering is candidate-level, not artifact-level, and raw CDP payloads remain exposed.           |
| Testability                 |   2.0 | `smoke-test.mjs` is a live manual probe, not a deterministic test. No archived replay assertions exist.                                                |
| Operability                 |   6.0 | OpenAI-compatible surface is practical, but there is no package manifest, structured config validation, fixture CLI, or clear production-safe mode.    |

## Evidence From Reviewed Artifacts

```text
source_lines = 1180-line src/cdp-browser-llm-router.mjs + 4 helper modules
archived_stream_lines = 3634 ndjson records
schema_dump_keys = 63 grouped schema keys
inner_text_entries = 713 entries
inner_text_candidates = 627 candidates
observed_parse_outputs = included user_profile + user_instructions + prior user prompts + answer fragments
node_check = passed
```

## Highest Leverage Next Work

1. Add a deterministic replay command: `ndjson -> extracted_response -> expected_response`.
2. Gate extraction by turn identity: only current submitted prompt’s response stream may update the accumulator.
3. Split extractor phases: `observe schema -> promote rule -> compile extractor -> execute extractor -> record rule id`.
4. Add artifact-boundary redaction before any dump write, not only candidate filtering.
5. Replace `smoke-test.mjs` with golden fixtures for ChatGPT, Gemini, partial chunks, `[DONE]`, idle cutoff, stale streams, and prompt-history contamination.
6. Split `cdp-browser-llm-router.mjs` into HTTP API, target/session control, capture, lifecycle, extraction, artifact writing, and OpenAI response codec.
7. Add extractor metrics: `precision`, `recall`, `contamination_rate`, `cutoff_rate`, `empty_response_rate`, and `schema_rule_coverage`.

## Updated Verdict

```text
objective_rating = ARCH = 5.13 / 10
current_system = useful CDP stream evidence collector with heuristic response extraction
best_property = observability over real browser traffic
main_gap = extractor cannot yet prove clean current-turn assistant output
next_unlock = fixture-driven replay harness + turn-scoped generated extraction rules
```

## Current Status

The project has a real and useful foundation: CDP capture works, artifacts are rich, schema dumps are generated, and the server can expose an OpenAI-shaped local API.

The score must stay constrained because the core promise is not merely capture. The goal is a schema-derived extractor generator. Today, the system mostly has schema observation plus heuristic extraction. Until extraction is generated from promoted schema evidence and verified against replay fixtures, the architecture remains a prototype rather than a dependable router.
