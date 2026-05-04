# Implementation Plan — Router Server

## Variables

```text
B = 84aa6a369a32eec9674c9aef34c3826944c9f49f  # restored uploaded bundle head / origin/main
H = committed HEAD after EXECUTE
A = OpenAI-like assistant response shape
K = SDK/drop-in client compatibility
S = streaming SSE compatibility
T = offline test strength
E = evidence/provenance quality
V = replay/artifact verification correctness
R = live operational realism
L = policy-learning proof
C = complexity / artifact-corpus maintainability
G = aggregate project goodness
```

## Equation

```text
next_fix = argmax(score_gain × validation_power × feasibility ÷ source_risk)
Good = max(A=7.9, T=7.8, S=7.6, P=7.4, O=7.3, Q=7.2, X=7.2, K=7.1, E=6.7, D=6.6, V=6.3, R=6.2, M=6.0, C=5.8, L=3.8) = A
```

One-line explanation: after the SDK gate landed, the best implementable improvement is to make replay/artifact verification separate current generated evidence from legacy failing evidence.

## Evidence Basis

This PLAN is derived from `GOAL.md` and the latest `score.md` OBSERVE section.

```text
GOAL.md purpose = local browser-control server exposing an OpenAI-compatible API over authenticated browser sessions
GOAL.md target = evidence → dataset registry → feature extraction → pattern mining → policy decision → response extraction → feedback + replay/privacy verification
GOAL.md status = architecture_defined ∧ implementation_unproven
GOAL.md next target = minimal provider registry plus chatgpt_private send/read loop
GOAL.md boundary = do not harvest credentials, bypass login/CAPTCHA, evade rate limits, or persist cookies/tokens/auth headers
score.md current_score ≈ 6.6/10
score.md strongest_axis = A=7.9; response envelope evidence is strongest
score.md offline_tests = strong; npm test passes unit + mock + sdk + artifact fixture gates
score.md live_CDP = unproven; npm run test:live failed with ECONNREFUSED 127.0.0.1:9221 / healthz 502
score.md artifact_gap = 86/88 committed historical turns fail the current quality gate
score.md runtime_receipt_gap = validation_commands=[] and test_count=0 in runtime delta receipts
score.md policy_learning_gap = L=3.8; no replay-backed before/after improvement proof
```

Validation already observed for this PLAN pass:

```text
npm test = pass; unit 7/7, mock 3/3, sdk 4/4, artifact fixtures 5/5
npm run smoke = pass; openai_contract_smoke_ok
```

## Highest-Impact EXECUTE Target

Target `V = replay/artifact verification correctness` by adding a current-corpus artifact gate that does not confuse legacy committed evidence with newly generated evidence.

```text
target = current_artifact_corpus_gate
primary_axes = V, E, C
secondary_axes = T, R, M
blocked_axis = live authenticated CDP behavior
large_deferred_axis = L until policy deltas are replay-backed
```

Reasoning:

```text
SDK_contract_matrix = already implemented and passing
artifact_fixture_gate = already implemented and passing
full_committed_artifact_corpus_gate = currently fails 86/88 historical turns
legacy_failures_are_real = preserve as risk evidence, not default CI blocker
current_generated_artifacts_need_gate = true
best_score_gain_now = verify newly generated turn artifacts while keeping legacy debt explicit
```

This improves the highest practical confidence gap without requiring authenticated browser access, changing provider behavior, or deleting historical evidence.

## Planned Source Changes for EXECUTE Stage

Do not modify source code in this PLAN stage. The next EXECUTE stage should stay within this scope:

1. Add a corpus-mode validator option.
   - Keep `validateTurnArtifacts(rootPath)` strict by default.
   - Add an explicit mode for current/generated artifacts, not a silent legacy bypass.
   - Report `scope`, `turn_count`, `pass_count`, `fail_count`, and failure summaries.
   - Do not weaken `replay_match` or `redaction_pass` checks.

2. Add a current-corpus test path.
   - Use the mocked CDP integration or a temporary generated artifact directory.
   - Validate only artifacts generated during the current test run or a deliberately curated fixture corpus.
   - Assert that generated turns include `manifest.json`, `replay.json`, `evaluation.json`, valid NDJSON, matching `turn_id`, `replay_match=true`, and `redaction_pass=true`.

3. Preserve legacy corpus visibility.
   - Keep a command that reports the committed `artifacts/turns` failure count.
   - Do not make historical legacy failures pass by ignoring them globally.
   - Record legacy corpus status in `score.md` as debt, not resolved evidence.

4. Add explicit npm scripts.
   - Keep `npm test` deterministic and no-browser by default.
   - Add a required current/generated artifact gate to default tests.
   - Add a separate advisory legacy-corpus report command that is expected to fail until historical turns are migrated or quarantined.

5. Update evidence after EXECUTE.
   - Update `score.md` only with command-backed facts.
   - Raise `V`, `E`, or `C` only for the newly gated current/generated corpus.
   - Keep `R` and live CDP capped unless `npm run test:live` passes against a reachable authenticated browser.
   - Keep `L` capped unless a replay-backed before/after policy improvement is implemented and validated.

## Expected Files to Change in EXECUTE Stage

```text
src/tools/validate-turn-artifacts.mjs
test/artifact-quality.test.mjs
package.json
score.md
```

Acceptable if a cleaner split is needed:

```text
test/current-artifact-corpus.test.mjs
test/fixtures/artifacts/current-valid/...
docs/07-operations-roadmap.md
```

Avoid:

```text
provider adapter rewrites
browser target-management rewrites
large artifact deletions
legacy evidence erasure
policy-learning rewrites
live-CDP behavior claims without live validation
```

## Validation Commands

Required validation for the EXECUTE stage:

```bash
node --version
npm --version
npm run check
npm run test:unit
npm run test:mock
npm run test:sdk
npm run test:artifacts
npm run smoke
npm test
```

Required artifact-specific validation:

```bash
node src/tools/validate-turn-artifacts.mjs test/fixtures/artifacts/valid
node src/tools/validate-turn-artifacts.mjs artifacts/turns || true
```

The second command must remain advisory unless the EXECUTE stage explicitly migrates or quarantines legacy artifacts.

Optional live validation when an authenticated browser is reachable on CDP port `9221`:

```bash
npm run test:live
```

Safe delta validation after the EXECUTE commit:

```bash
git status --short
git log --oneline -n 10
rm -f /mnt/data/repo-delta-XXX.bundle /mnt/data/DELTA_MANIFEST.md
git bundle create /mnt/data/repo-delta-XXX.bundle 84aa6a369a32eec9674c9aef34c3826944c9f49f..HEAD
git bundle verify /mnt/data/repo-delta-XXX.bundle
```

Receiver apply commands:

```bash
git fetch ./repo-delta-XXX.bundle HEAD
git merge --ff-only FETCH_HEAD
npm run check
npm test
npm run smoke
```

## Acceptance Criteria

```text
A1: default artifact tests validate current/generated artifacts, not only static fixtures
A2: generated current-turn artifacts prove manifest/replay/evaluation presence
A3: generated current-turn artifacts prove turn_id consistency
A4: generated current-turn artifacts prove replay_match=true and redaction_pass=true
A5: all NDJSON files in generated current-turn artifacts are parse-checked
A6: legacy committed corpus remains visible with measured fail_count, not hidden
A7: npm test remains deterministic and no-browser by default
A8: npm run test:live remains optional and does not support offline overclaims
A9: score.md records exact validation commands and distinguishes current evidence from legacy debt
A10: final delta bundle is cumulative from B..H and verifies successfully
```

## Risk Controls

```text
risk_masking_legacy_failures ⇒ keep legacy corpus report explicit and advisory, not silently skipped
risk_false_replay_confidence ⇒ do not relax replay_match/redaction_pass predicates
risk_artifact_bloat ⇒ generate temp artifacts in tests unless committed fixtures are minimal
risk_live_overclaim ⇒ keep live CDP optional and score-capped unless test:live passes
risk_policy_learning_overclaim ⇒ do not raise L without replay-backed before/after deltas
risk_delta_corruption ⇒ create bundle from 84aa6a3..HEAD and verify before returning final links
```

## Stage Boundary

This PLAN stage intentionally changes only:

```text
IMPLEMENTATION_PLAN.md
```

No source code is modified in this stage.