# Implementation Plan

## Variables

```text
B = 12f02b6c1814e873c9c1c8ee80f17cd826131e93
H = current HEAD after plan-stage commit
G = GOAL.md authority
S = score.md evidence state
Q = implementable score lift per changed surface
V = validation confidence
D = safe delta output discipline
```

## Equation

```text
next_work = argmax(Q) = argmax((ΔV + Δevidence + Δreproducibility - Δrisk) / changed_surface)
GOOD = max(K,C,V,P,B,E,N,D)
```

One-line explanation: target the smallest current mutation that turns a reproduced failing gate into a passing gate without weakening the frozen-kernel/runtime boundary.

## Source Authority

`GOAL.md` defines the repository target as a deterministic agent runtime whose value depends on a frozen kernel, replayable state, durable receipts, verification, eval, recovery, learning, and monotonically cheaper autonomous reasoning.

`score.md` is the current evidence authority. It records strong architecture but weak reproducibility:

```text
overall_score = 5.92 / 10
strongest_axis = kernel_boundary_architecture, 8.4 / 10
root_rust_validation = unavailable here
rustc_wrapper_path_exists = false
state_rustc_graph_json_present = false
runtime_archive = advisory, stale base mismatch
router_syntax = pass
router_openai_contract = pass, 7/7
router_mock_cdp = pass, 3/3
router_artifact_quality = fail, 0/5
```

Root Rust validation would be higher impact, but it is not implementable in this environment because `cargo`/`rustc` and the configured workspace wrapper are unavailable. The highest implementable score lift is the nested router artifact-quality gate because Node is available, the failure is reproduced, and the cause is local fixture duplication rather than missing external infrastructure.

## Highest-Impact Target For EXECUTE

Close `ai-chromium/router-server/test/artifact-quality.test.mjs` from `0/5 fail` to `5/5 pass`.

Current reproduced failure:

```text
fixture valid              expected turn_count 1, actual 2
fixture missing-manifest   expected turn_count 1, actual 2
fixture replay-mismatch    expected turn_count 1, actual 2
fixture redaction-fail     expected turn_count 1, actual 2
fixture malformed-evidence expected turn_count 1, actual 2
```

Root cause:

```text
Each fixture has two turn directories:
- canonical turn-001
- stale legacy turn_* directory

validateTurnArtifacts() counts both because both contain turn marker files.
```

Preferred implementation:

```text
remove stale fixture directories only:
  ai-chromium/router-server/test/fixtures/artifacts/valid/turn_pass
  ai-chromium/router-server/test/fixtures/artifacts/missing-manifest/turn_missing_manifest
  ai-chromium/router-server/test/fixtures/artifacts/replay-mismatch/turn_replay_mismatch
  ai-chromium/router-server/test/fixtures/artifacts/redaction-fail/turn_redaction_fail
  ai-chromium/router-server/test/fixtures/artifacts/malformed-evidence/turn_malformed
```

Do not change router runtime behavior. Do not change kernel/runtime/capability/API Rust source. Do not relax the test expectation. The validator should keep detecting malformed JSON, missing manifest, replay mismatch, and redaction failure.

## Validation Commands

Run before committing the EXECUTE-stage fix:

```bash
git status --short
node --version
node --check ai-chromium/router-server/src/server.mjs
(cd ai-chromium/router-server && node --test test/openai-contract.test.mjs)
(cd ai-chromium/router-server && node --test test/mock-cdp-integration.test.mjs)
(cd ai-chromium/router-server && node --test test/artifact-quality.test.mjs)
bash scripts/observe_validation.sh
test -s target/observe/validation-report.ndjson
python3 - <<'PY'
import json
from pathlib import Path
path = Path('target/observe/validation-report.ndjson')
count = 0
for line in path.read_text().splitlines():
    if line.strip():
        json.loads(line)
        count += 1
print(f'valid_ndjson_lines={count}')
PY
git diff --check
```

Rust validation remains required when a Rust toolchain and wrapper are available:

```bash
RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check
RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets
RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings
```

The `RUSTC_WORKSPACE_WRAPPER=""` override is validation-only. It avoids the absent `/workspace/ai_sandbox/canon-rustc-v2/target/debug/canon-rustc-v2` wrapper without changing `.cargo/config.toml`.

## Expected Score Movement

```text
N: nested router-server evidence improves if artifact-quality moves 0/5 fail -> 5/5 pass
B: build/test reproducibility improves slightly because one known failing gate closes
E: evidence quality improves because fixture semantics become canonical and executable
K/C/V: unchanged; frozen-kernel architecture and root runtime claims remain static
```

This plan does not claim root Rust validation, live Ollama validation, graph regeneration, live CDP validation, or policy-learning replay validation.

## Risks And Constraints

```text
risk_source_mutation = low, fixture deletion only
risk_behavior_change = low, no runtime source change required
risk_false_positive = medium, mitigated by preserving all five positive/negative fixture cases
risk_scope_creep = medium, mitigated by forbidding unrelated source edits
risk_overclaim = medium, mitigated by leaving root Rust/toolchain gaps explicit
```

Constraints:

- Do not modify source code in PLAN stage.
- In EXECUTE stage, prefer fixture cleanup over validator/test logic changes.
- Do not alter `score.md` unless validation evidence changes.
- Do not hide root Rust/toolchain, graph, runtime archive, Ollama, or live CDP gaps.
- Keep generated validation output outside committed source unless explicitly requested.

## Safe Git Delta Procedure

Use the uploaded restore base preserved by `origin/main`:

```bash
B=$(git rev-parse origin/main)
H=$(git rev-parse HEAD)
git status --short
git bundle create /mnt/data/repo-delta-0001.bundle "$B..$H"
git bundle verify /mnt/data/repo-delta-0001.bundle
```

Receiver apply command:

```bash
git fetch ./repo-delta-0001.bundle HEAD && git merge --ff-only FETCH_HEAD
```

## Plan-Stage Result

```text
source_code_changed = false
plan_file_changed = IMPLEMENTATION_PLAN.md
implementation_target = deduplicate router artifact-quality fixtures
validation_scope = documentation_plan_only + reproduced current router gate state
safe_delta_base = origin/main = 12f02b6c1814e873c9c1c8ee80f17cd826131e93
```