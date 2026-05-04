# Implementation Plan — Router Server

## Variables

```text
B = 01ec13fce5482e2d53d4097ec7a65d74fe19c11f
H = committed HEAD after this PLAN stage
O = OpenAI endpoint/envelope compatibility
Q = request schema compatibility
S = streaming SSE compatibility
K = SDK/drop-in compatibility
E = evidence/provenance quality
V = replay/verification correctness
X = privacy/redaction correctness
T = testability
R = operational realism
L = policy learning quality
G = geometric_mean(O, Q, S, K, E, V, X, T, R, L)
```

## Equation

```text
next_fix = argmax(score_gain × validation_power × implementation_feasibility ÷ source_risk)
Good = max(O, Q, S, K, E, V, X, T, R, L)
```

One-line explanation: choose the smallest executable change that raises the weakest proven production signal without expanding the browser-control trust boundary.

## Evidence Source

This plan is derived from `GOAL.md` and the current `score.md` evidence at observed HEAD `346128db7569637c95c378d93158c9987ee300c8`.

```text
GOAL.md target = provider-capability router over authenticated browser sessions
GOAL.md current_status = architecture_defined ∧ implementation_unproven
score.md offline_validated = syntax_pass ∧ unit_pass ∧ mock_CDP_pass ∧ smoke_pass
score.md live_unvalidated = CDP_9221_unreachable
score.md artifact_quality_gap = replay_redaction_joint_pass_count / evaluation_files = 2 / 80
score.md weakest_axes = L, K, V, M/R class signals
score.md strongest_current_gain = mocked route/artifact pipeline now passes offline
```

Concrete validation evidence already available:

```text
npm run check = pass; syntax_ok files=47
npm run test:unit = pass; 7/7 node:test cases passed
npm run test:mock = pass; 3/3 node:test cases passed
npm run smoke = pass; openai_contract_smoke_ok
npm test = pass; unit + mock integration
npm run test:live = fail_environment; CDP unavailable at 127.0.0.1:9221
```

## Highest-Impact EXECUTE Target

Target the artifact-quality and replay/redaction gate, not another broad architecture rewrite.

```text
target = artifact_quality_gate_for_completed_turns
primary_axes = V, X, E, T
secondary_axes = R, K, Q, S
blocked_axis = live_CDP_behavior; cannot be proven without authenticated CDP endpoint
```

Reasoning:

```text
mocked_CDP_route_now_exists = true
live_CDP_unavailable_in_current_environment = true
historical_artifact_joint_pass = 2 / 80
current_gap = completed turns can exist without a hard replay∧redaction quality threshold
highest_feasible_gain = make offline completed-turn artifacts fail validation when replay/redaction evidence is missing or false
```

The next source change should add a deterministic artifact-quality validation tool and test fixture. This directly addresses the weakest evidence gap in `score.md`: historical artifacts show poor replay/redaction joint quality, while current mocked turns prove the route can generate better evidence. The project now needs a gate that prevents low-quality completed-turn artifacts from being silently accepted.

## Planned Source Changes for EXECUTE Stage

Do not modify source code in this PLAN stage. The next EXECUTE stage should stay within this scope:

1. Add an artifact-quality validator.
   - Validate completed turn directories under `artifacts/turns/` or a supplied fixture directory.
   - Require manifest presence for completed turns.
   - Require replay evidence with `replay_match=true`.
   - Require redaction evidence with `redaction_pass=true` or equivalent explicit pass signal.
   - Fail closed on malformed JSON/NDJSON.

2. Add a focused test fixture for the validator.
   - Include one passing completed turn fixture.
   - Include failing fixtures for missing manifest, replay mismatch, redaction failure, and malformed evidence.
   - Keep fixtures minimal; do not depend on live browser/CDP.

3. Wire validation into offline commands.
   - Add a dedicated script such as `npm run test:artifacts` or include it in `npm test` only if deterministic and fast.
   - Avoid making historical committed artifacts a required all-pass gate until legacy failures are either quarantined or documented as historical.

4. Preserve the live-browser boundary.
   - Keep `npm run test:live` manual/environment-gated.
   - Do not claim live ChatGPT/Gemini behavior is validated unless a reachable authenticated CDP browser passes the live test.

5. Update `score.md` after EXECUTE validation.
   - Record exact commands and pass/fail results.
   - Raise only axes supported by new evidence.
   - Continue listing live CDP and strict OpenAI semantic parity as unproven if not validated.

## Expected Files to Change in EXECUTE Stage

```text
src/tools/validate-turn-artifacts.mjs
test/artifact-quality.test.mjs
test/fixtures/artifacts/*
package.json
score.md
```

Acceptable alternative:

```text
src/artifacts/* validator module + test wrapper
```

Avoid changes to provider adapters, CDP target management, or OpenAI envelope helpers unless the validator needs exported metadata constants.

## Validation Commands

Required validation for the EXECUTE stage:

```bash
node --version
npm --version
npm run check
npm run test:unit
npm run test:mock
npm run smoke
node --test test/artifact-quality.test.mjs
```

If the artifact validator is added to `npm test`, also run:

```bash
npm test
```

Optional live validation when an authenticated browser is running on CDP port 9221:

```bash
npm run test:live
```

Delta validation after commit:

```bash
git status --short
git log --oneline -n 5
git bundle create /mnt/data/repo-delta-0001.bundle 01ec13fce5482e2d53d4097ec7a65d74fe19c11f..HEAD
git bundle verify /mnt/data/repo-delta-0001.bundle
```

Receiver apply commands:

```bash
git fetch ./repo-delta-0001.bundle HEAD && git merge --ff-only FETCH_HEAD
npm run check
npm run test:unit
npm run test:mock
npm run smoke
```

## Acceptance Criteria

```text
A1: validator rejects completed turns without manifest evidence
A2: validator rejects replay_match=false or missing replay result
A3: validator rejects redaction_pass=false or missing redaction result
A4: validator rejects malformed JSON/NDJSON deterministically
A5: validator accepts a minimal valid completed-turn fixture
A6: validator is covered by node:test and included in documented validation commands
A7: score.md update is evidence-backed and does not overclaim live CDP validation
A8: delta artifacts are created from B..H and verify successfully
```

## Risk Controls

```text
risk_legacy_artifacts_fail_gate ⇒ test fixture path first; document historical corpus separately
risk_false_privacy_confidence ⇒ require explicit redaction pass signal, not absence-only heuristics
risk_replay_overfit ⇒ compare declared expected/extracted content fields, not filenames alone
risk_source_churn ⇒ keep validator standalone and CLI-friendly
risk_live_overclaim ⇒ preserve test:live as separate environment-gated signal
```

## Stage Boundary

This PLAN stage intentionally changes only:

```text
IMPLEMENTATION_PLAN.md
```

No source code is modified in this stage.
