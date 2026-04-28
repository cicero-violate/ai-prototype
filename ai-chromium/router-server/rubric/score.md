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
ARCH = (TR · ST · SD · SG · RC · OB · MD · SF · TS · DX)^(1/10)
GOOD = max(TR,ST,SD,SG,RC,OB,MD,SF,TS,DX)
```

One-line explanation: this project’s goal is deterministic recovery of assistant output from browser-streamed raw payloads; the score rewards robust capture and schema-derived extraction over provider-specific hardcoding.

## Score Summary

```text
TR = 8.5 / 10
ST = 8.3 / 10
SD = 7.8 / 10
SG = 7.4 / 10
RC = 8.2 / 10
OB = 8.9 / 10
MD = 8.4 / 10
SF = 7.2 / 10
TS = 5.8 / 10
DX = 8.0 / 10

CORE = 8.04 / 10
DATA = 8.00 / 10
IMPL = 7.79 / 10
ARCH = 7.79 / 10

max(TR,ST,SD,SG,RC,OB,MD,SF,TS,DX) = OB = 8.9 / 10 = good
```

## Static Review Inputs

```text
review_scope = src/cdp-browser-llm-router.mjs + src/cdp-router/*.mjs + Goal.md + cdp-*dump outputs
runtime_validation = manual curl turn runs performed
syntax_validation = node --check passed for router and split modules
automated_tests = absent

goal = schema-derived extractor generator over raw streamed payloads
invariant = avoid provider hardcoding unless promoted by observed schema evidence

stream_capture_paths = Network.streamResourceContent + Network.dataReceived + Network.getResponseBody fallback + websocket fallback
schema_outputs = event schemas + raw_payload_json grouped schemas + embedded JSON reconstruction
extraction_strategy = structural patch/text heuristics with minimal semantic assumptions
```

## Critical Judgment

The project is now materially aligned with the stated goal. Raw payload capture is strong, schema dumping is high signal, and extraction logic is being pushed toward structural inference rather than role/provider assumptions.

Major improvements include: cleaner file-purpose split, stronger lifecycle handling to avoid premature stop while streams are still active, first-class raw payload schema families, and reconstruction of append-fragmented embedded JSON before schema derivation.

Current weak point is verification depth. There is still no automated regression suite proving extraction correctness across archived stream dumps, model variants, and edge lifecycle cases. The system is operationally effective but not yet test-hardened.

## Module Rating Table

| Module/Concern              | Score | Reason                                                                                    |
|-----------------------------+-------+-------------------------------------------------------------------------------------------|
| CDP transport (`CdpSocket`) |   8.5 | Solid low-level ws handling, command/reply mapping, stale stream handling.                |
| Stream acquisition          |   8.3 | Multiple capture paths and fallback logic; still exposed to provider event quirks.        |
| Schema derivation           |   7.8 | Good inferred schemas and grouping; some cases still depend on reconstruction success.    |
| Schema-guided extraction    |   7.4 | Structural and less hardcoded now; still heuristic-heavy and not fully learned/generated. |
| Lifecycle robustness        |   8.2 | Idle deferral with active streams fixed early cutoff class of failures.                   |
| Observability/dumps         |   8.9 | Excellent ndjson + schema + inner-text artifacts for debugging and analysis.              |
| Modularity                  |   8.4 | Strong improvement with dedicated transport/schema/extraction files.                      |
| Safety/filtering            |   7.2 | Reasonable text filtering and truncation; sensitive-data exposure risk remains in dumps.  |
| Testability                 |   5.8 | No formal unit/integration fixtures for extraction and lifecycle invariants.              |
| Operability                 |   8.0 | Simple OpenAI-compatible surface and useful runtime stats in responses.                   |

## Highest Leverage Next Work

1. Add golden-stream fixtures (`input ndjson -> expected content/finish_reason`) and run them in CI.
2. Build schema regression checks for `raw_payload_json*` and `raw_payload_embedded_json_reconstructed*` outputs.
3. Add deterministic turn-end policy tests (idle, done-signal, loadingFinished, max timeout interactions).
4. Reduce dump sensitivity risk (token/cookie redaction controls and optional secure mode defaults).
5. Introduce explicit extractor quality metrics (precision/recall over curated transcripts).

## Updated Verdict

```text
objective_rating = ARCH = 7.79 / 10
current_system = robust streamed-payload capture + schema-derived extraction prototype
best_property = observability and debuggability under real browser traffic
main_gap = missing automated correctness harness for lifecycle/extraction invariants
next_unlock = fixture-driven extraction + lifecycle regression suite
```

## Current Status

Transport, streaming capture, schema dumps, and extraction plumbing are
implemented and actively used. The codebase has been split by purpose
across transport, schema derivation, and extraction modules.

Current strengths:

- Robust CDP capture and detailed diagnostics
- Strong artifact pipeline (stream/schema/inner-text)
- Reduced hardcoded role assumptions in extraction

Current gaps:

- No dedicated automated regression suite for extraction/lifecycle invariants
- Schema-guided extraction is still heuristic, not yet generated end-to-end
- Embedded JSON reconstruction quality depends on streamed fragment integrity

The foundation is operational. The next stage is fixture-driven
validation and tighter schema-to-extractor generation.
