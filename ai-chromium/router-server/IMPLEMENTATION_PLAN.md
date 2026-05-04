# Implementation Plan — Router Server

## Variables

```text
B = ca3414f2cb61e4d634b5f42d3d0dbcd58632d35b  # restored uploaded bundle head / origin/main
P = IMPLEMENTATION_PLAN.md
S = score.md
G = GOAL.md
H = committed HEAD after EXECUTE
O = OpenAI endpoint/envelope compatibility
Q = request schema compatibility
A = assistant response schema compatibility
K = SDK/drop-in compatibility
E = evidence/provenance quality
V = replay/verification correctness
T = deterministic testability
R = live operational realism
M = documentation accuracy
C = complexity / cognitive load
L = policy learning quality
```

## Equation

```text
next_fix = argmax(score_gain × validation_power × feasibility ÷ source_risk)
Good = max(A=7.9, T=7.9, S_stream=7.6, P_arch=7.4, O=7.3, Q=7.2, K=7.1, E=6.9, V=6.6, R=6.2, M=6.0, C=5.9, L=3.8) = A = T
```

One-line explanation: the strongest current axes are assistant contract shape and offline testability; the best immediate improvement is to make required/offline, advisory/legacy, and optional/live validation boundaries explicit and machine-checkable.

## Evidence Basis

This PLAN is derived from `GOAL.md`, current `score.md`, recent git history, `package.json`, `run_tests.sh`, tests, and validation re-run in this environment.

```text
GOAL.md purpose = local browser-control server exposing an OpenAI-compatible API over authenticated browser sessions
GOAL.md target = evidence → dataset registry → feature extraction → pattern mining → policy decision → response extraction → feedback + replay/privacy verification
GOAL.md boundary = operator-owned authenticated browser only; no credential harvesting, CAPTCHA bypass, rate-limit evasion, or cookie/token persistence
score.md current_rating = 6.8/10
score.md strongest_axes = A=7.9 and T=7.9
score.md key_blockers = live_CDP_unproven ∧ legacy_artifact_quality_low ∧ policy_learning_unproven
score.md legacy_artifact_quality = 2/88 committed turns pass current validator; 86/88 fail expected legacy gate
score.md runtime_risk = stale_advisory download present; runtime receipts/history exist but live browser evidence is incomplete
package_scripts = check, test, test:unit, test:mock, test:sdk, test:artifacts, test:artifacts:legacy, test:live, smoke
run_tests.sh = already treats live CDP as opt-in via RUN_LIVE_TESTS=1 or LIVE_ROUTER_URL
```

Validation observed during this PLAN stage:

```text
npm run check = pass; syntax_ok files=51
npm test = pass; unit 7/7, mock 3/3, SDK 4/4, artifact tests 6/6
npm run smoke = pass; openai_contract_smoke_ok
npm run test:live = fail_environment; healthz 502; connect ECONNREFUSED 127.0.0.1:9221
```

## Highest-Impact EXECUTE Target

Target `T + E + R + M` by adding a deterministic validation profile and release-evidence contract.

```text
target = validation_profile_and_release_evidence_contract
primary_axes = T, E, R, M
secondary_axes = C, V
blocked_axis = live authenticated CDP pass
large_deferred_axis = L until replay-backed policy before/after improvement exists
```

Reasoning:

```text
live_CDP_validation = highest realism gap but unavailable in this environment
policy_learning = lowest score but requires replay-backed feedback design beyond one safe step
current_artifact_gate = already implemented by prior commit ca3414f
best_feasible_step_now = separate required offline gates, advisory legacy gates, and optional live gates with explicit evidence output
```

This should prevent false production claims while improving release discipline: offline CI can pass deterministically, legacy debt remains visible, and live browser proof becomes an explicit opt-in requirement instead of an implicit failed default.

## Planned Changes for EXECUTE Stage

Do not modify source code in this PLAN stage. The next EXECUTE stage should stay within this scope:

1. Add explicit validation profiles.
   - `test:offline` or `validate:offline`: required deterministic gate.
   - `validate:legacy`: advisory committed artifact corpus report; expected to expose legacy failures.
   - `validate:live`: optional authenticated CDP gate; never required without operator-owned browser.
   - `validate:release`: ordered profile that runs offline gates, smoke, legacy advisory, and records status.

2. Emit command-backed validation evidence.
   - Record command names, exit codes, and pass/fail classes.
   - Distinguish `pass`, `fail`, and `fail_environment`.
   - Capture live CDP failure as environment evidence, not source failure.
   - Do not persist secrets, cookies, tokens, prompts, or assistant content.

3. Keep legacy artifact debt explicit.
   - Preserve `npm run test:artifacts:legacy` as advisory unless legacy corpus migration is implemented.
   - Report `turn_count`, `pass_count`, and `fail_count`.
   - Do not delete, rewrite, or silently ignore historical evidence.

4. Preserve safe git delta output.
   - Do not create final `/mnt/data/repo-delta-XXX.bundle` in PLAN or intermediate stages.
   - In final EXECUTE stage, create the cumulative bundle from original base `B..HEAD`.
   - Verify the bundle before returning links.
   - Write a manifest containing base commit, head commit, changed files, validation commands, and receiver apply commands.

5. Update `score.md` only with command-backed facts.
   - Raise `T`, `E`, or `M` only if the validation profile and evidence contract pass.
   - Keep `R` capped unless `npm run test:live` passes against reachable authenticated CDP.
   - Keep `L` capped unless a replay-backed policy-learning delta is implemented and validated.

## Expected Files to Change in EXECUTE Stage

```text
package.json
run_tests.sh
IMPLEMENTATION_PLAN.md
score.md
```

Acceptable if needed:

```text
docs/07-operations-roadmap.md
test/validation-profile.test.mjs
src/tools/report-validation.mjs
```

Avoid:

```text
provider adapter rewrites
browser target-management rewrites
legacy artifact deletion
credential/cookie/token persistence
live-CDP success claims without live validation
policy-learning score increases without replay-backed before/after proof
```

## Validation Commands

Required no-browser validation:

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

Required profile validation after EXECUTE adds the scripts:

```bash
npm run validate:offline
npm run validate:release
```

Required legacy advisory signal:

```bash
npm run test:artifacts:legacy || true
```

Optional live validation when an authenticated browser is reachable:

```bash
RUN_LIVE_TESTS=1 npm run test:live
```

Safe delta validation for final stage only:

```bash
git status --short
git log --oneline -n 10
rm -f /mnt/data/repo-delta-XXX.bundle /mnt/data/DELTA_MANIFEST.md
git bundle create /mnt/data/repo-delta-XXX.bundle B..HEAD
git bundle verify /mnt/data/repo-delta-XXX.bundle
```

Receiver apply commands:

```bash
git fetch ./repo-delta-XXX.bundle HEAD
git merge --ff-only FETCH_HEAD
npm run validate:offline
npm run smoke
```

## Acceptance Criteria

```text
A1: offline validation has one explicit command and passes without browser access
A2: release validation records required gates and advisory legacy/live classes separately
A3: live CDP failure is classified as fail_environment when CDP 9221 is unreachable
A4: legacy artifact corpus remains visible with measured pass/fail counts
A5: default tests remain deterministic and no-browser
A6: no secrets, cookies, tokens, prompts, or assistant content are persisted by validation evidence
A7: score.md records exact commands and does not overclaim live or policy-learning status
A8: final delta bundle is cumulative from B..HEAD and verifies successfully
```

## Risk Controls

```text
risk_false_production_confidence ⇒ keep live CDP optional and explicitly environment-gated
risk_hiding_legacy_debt ⇒ keep legacy corpus report advisory but visible
risk_validation_noise ⇒ classify environment failure separately from source failure
risk_secret_retention ⇒ store command metadata and digests only, not browser/session content
risk_delta_corruption ⇒ create final bundle from B..HEAD and verify before returning links
risk_scope_creep ⇒ do not alter provider behavior during this validation-profile step
```

## Stage Boundary

PLAN stage intentionally changed only:

```text
IMPLEMENTATION_PLAN.md
```

EXECUTE stage may change source, tests, package scripts, and score evidence to implement the selected validation-profile gate.

## EXECUTE Turn 1 Result

```text
implemented = validation_profile_runner ∧ validation_classifier_test ∧ package_scripts ∧ run_tests_update ∧ score_evidence_update
validation_profiles = offline(required), legacy(advisory), live(optional), release(required+advisory+optional)
release_gate = required_offline_pass ∧ legacy_visible ∧ live_skipped_without_operator_browser
```

Validation commands for this turn:

```bash
npm run check
npm test
npm run smoke
npm run validate:offline
npm run validate:legacy
npm run validate:release
npm run validate:live || true
```

Observed results:

```text
npm run check = pass; syntax_ok files=53
npm test = pass; unit + mock + sdk + artifact + validation-profile tests
npm run smoke = pass; openai_contract_smoke_ok
npm run validate:offline = pass; required_pass=true
npm run validate:legacy = pass_advisory; advisory_failures=1; legacy corpus still fails internally
npm run validate:release = pass; required_pass=true; advisory_failures=1; optional_skips=1
npm run validate:live = fail_environment; GET http://127.0.0.1:9221/json/version unreachable
```

Safe delta output remains:

```bash
git bundle create /mnt/data/repo-delta-004.bundle ca3414f2cb61e4d634b5f42d3d0dbcd58632d35b..HEAD
git bundle verify /mnt/data/repo-delta-004.bundle
git fetch ./repo-delta-004.bundle HEAD
git merge --ff-only FETCH_HEAD
```
