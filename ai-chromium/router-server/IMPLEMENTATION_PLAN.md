# Implementation Plan — Router Server

## Variables

```text
B = 050b9997b04e83f9ac8d86d8c83e41c7daa035b3
H = committed HEAD after this planning stage
P = provider/capability architecture score
T = testability score
R = operational realism score
V = replay/verification score
L = policy learning score
Q = request schema compatibility score
S = streaming compatibility score
G = score.md weakest-link geometric mean
```

## Equation

```text
next_fix = argmax(score_gain × implementation_feasibility × validation_power ÷ source_risk)
Good = max(P, T, R, V, L, Q, S)
```

One-line explanation: choose the smallest source change that increases confidence in the browser-backed route without requiring an authenticated live browser.

## Source Evidence

Evidence used for this plan:

```text
GOAL.md status = architecture_defined ∧ implementation_unproven
score.md current_score ≈ 6.2 / 10
score.md Good = A, with strongest response-envelope builder and weakest proven policy learning
score.md live_CDP_pass = missing because 127.0.0.1:9221 refused connection
score.md highest_leverage_next_fix[1] = no-browser integration harness that mocks CDP/network capture while exercising handleChatCompletions end-to-end
package scripts = check, test:unit, smoke, test:live
unit evidence = 7/7 node:test cases pass for OpenAI contract helpers
smoke evidence = openai_contract_smoke_ok
live evidence = fail_environment; cdp_unavailable; healthz 502
runtime archive = sanitized, 2 included files, no download history, no leak/integrity/schema findings
artifact corpus = 88 turn dirs, replay/redaction pass quality still weak
```

## Target Highest-Impact Score Improvement

Target:

```text
target = mocked_CDP_integration_tests
primary_axes = T, R, Q, S, V
secondary_axes = O, K, E
```

Why this is the highest-impact next move:

```text
live_CDP_required = true
current_environment_CDP_available = false
unit_tests_cover_helpers_only = true
smoke_test_covers_contract_shape_only = true
mocked_CDP_integration_test_can_cover_router_to_browser_pipeline_without_auth_browser = true
```

The next implementation should add an offline integration harness around the live route path instead of depending on an authenticated browser. That gives deterministic validation for the `/v1/chat/completions` path, request-to-prompt bridging, SSE response behavior, receipt/artifact writes, redaction gates, and CDP failure handling while preserving the separate opt-in live test.

## Planned Source Changes for EXECUTE Stage

Do not implement these in this PLAN stage. The next EXECUTE stage should make source/test changes only within this scope:

1. Add a deterministic no-browser CDP/network fixture harness.
   - Prefer `node:test` fixtures under `test/`.
   - Mock only the browser/CDP boundary, not the OpenAI contract helpers.
   - Exercise the actual router handler/server path as much as possible.

2. Add end-to-end tests for non-streaming `/v1/chat/completions`.
   - Valid user message returns assistant envelope.
   - System/developer/user role bridge is represented in the submitted browser prompt.
   - Unsupported tools fail before browser mutation.
   - Redacted request artifacts do not retain prompt body or file paths.

3. Add end-to-end tests for streaming `/v1/chat/completions`.
   - First SSE chunk includes assistant role.
   - Content deltas are emitted in order.
   - Final finish chunk and `[DONE]` are emitted.
   - Error behavior is explicit and client-parseable.

4. Add artifact-quality fixture assertions.
   - Manifest exists for completed mocked turn.
   - Replay record compares extracted content against emitted content.
   - Redaction pass is asserted before persisted artifacts are accepted.
   - Raw capture persistence remains blocked unless explicit unsafe/discovery mode is enabled.

5. Keep live CDP tests opt-in/manual.
   - Do not make `npm run test:live` a required offline validation gate.
   - Document that live validation still requires an authenticated Chrome CDP endpoint on `127.0.0.1:9221`.

## Files Expected to Change in EXECUTE Stage

Likely files:

```text
test/mock-cdp-integration.test.mjs
src/browser/* or src/provider/* only if dependency injection seams are missing
src/server.mjs only if route handler export/seam is required
package.json if a dedicated offline integration script is added
score.md after validation evidence changes
```

Avoid broad rewrites. If a seam is needed, expose the minimum testable boundary rather than replacing provider architecture.

## Validation Commands

Required offline validation for EXECUTE stage:

```bash
node --version
npm --version
npm run check
npm run test:unit
npm run smoke
node --test test/mock-cdp-integration.test.mjs
```

Optional live validation, only when an authenticated CDP browser is available:

```bash
npm run test:live
```

Artifact validation after commit:

```bash
git status --short
git log --oneline -n 5
git bundle create /mnt/data/repo-delta-0001.bundle 050b9997b04e83f9ac8d86d8c83e41c7daa035b3..HEAD
git bundle verify /mnt/data/repo-delta-0001.bundle
```

Receiver apply validation:

```bash
git fetch ./repo-delta-0001.bundle HEAD && git merge --ff-only FETCH_HEAD
npm run check
npm run test:unit
npm run smoke
```

## Acceptance Criteria

```text
A1: offline integration test proves one non-streaming chat completion through the browser boundary mock
A2: offline integration test proves streaming SSE chunk order and terminal [DONE]
A3: unsupported capabilities fail before mocked browser mutation
A4: completed mocked turn writes/validates manifest + redaction + replay evidence
A5: live CDP test remains available but is documented as environment-gated
A6: score.md is updated with concrete before/after validation evidence
A7: source delta stays minimal and test-driven
```

## Risk Controls

```text
risk_browser_mock_overfit ⇒ fixture should assert boundary calls and final OpenAI envelope, not private implementation details
risk_live_claim_overstatement ⇒ score.md must continue to mark live CDP as unverified unless npm run test:live passes
risk_privacy_regression ⇒ artifact assertions must fail if prompt bodies, file paths, cookies, tokens, or auth headers persist
risk_streaming_false_positive ⇒ SSE parser test must parse actual emitted event stream, not only helper objects
risk_source_churn ⇒ prefer one new test file and smallest dependency injection seam
```

## Stage Boundary

This PLAN stage intentionally changes only:

```text
IMPLEMENTATION_PLAN.md
```

No source code is modified in this stage.