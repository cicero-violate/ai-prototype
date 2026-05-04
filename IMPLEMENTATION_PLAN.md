# Implementation Plan

## Variables

```text
B  = b9830281da5618db55c12371ec1f17b3abdd0b00
H0 = eae2039591b3df4a8bf525eac37782a4f072d373
G  = GOAL.md authority
S  = score.md evidence state
F  = failing reproducible gate count
M  = missing validation signal count
Q  = score lift per implementation risk
```

## Equation

```text
next_work = argmax(Q) = argmax((Δreproducible_validation + Δevidence - Δrisk) / changed_surface)
GOOD = max(K,C,V,P,B,E,N,D)
```

One-line explanation: the best immediate score improvement is to make the already-committed router artifact-quality gate reproducible by adding the missing deterministic fixtures, not by changing runtime behavior.

## Source Authority

`GOAL.md` defines the repository as a deterministic, self-improving agent runtime whose value depends on a frozen kernel, replayable state, durable receipts, verification, eval, recovery, and learning. It also records prior workstation claims such as `cargo build = passed`, `cargo test = 103 passed / 0 failed`, and graph intent coverage.

`score.md` is the current controlling evidence. It classifies the repository as a serious deterministic runtime prototype, but identifies these unresolved blockers:

```text
root_rust_validation = unavailable here
rustc_wrapper_path_exists = false
state_rustc_graph_json_present = false
runtime_archive_evidence = shallow preflight metadata only
router_artifact_quality = fail, 0/5, missing fixtures
```

The root Rust/toolchain blockers are higher severity, but not implementable in this environment without adding or changing toolchains. The router artifact-quality failure is implementable now because Node is available, the validator exists, and the failure is fixture absence rather than a behavioral ambiguity.

## Highest-Impact Target For EXECUTE

Close the reproducible router artifact-quality gate.

Target files to add only:

```text
ai-chromium/router-server/test/fixtures/artifacts/valid/turn-001/manifest.json
ai-chromium/router-server/test/fixtures/artifacts/valid/turn-001/replay.json
ai-chromium/router-server/test/fixtures/artifacts/valid/turn-001/evaluation.json
ai-chromium/router-server/test/fixtures/artifacts/valid/turn-001/evidence.ndjson
ai-chromium/router-server/test/fixtures/artifacts/missing-manifest/turn-001/replay.json
ai-chromium/router-server/test/fixtures/artifacts/missing-manifest/turn-001/evaluation.json
ai-chromium/router-server/test/fixtures/artifacts/replay-mismatch/turn-001/manifest.json
ai-chromium/router-server/test/fixtures/artifacts/replay-mismatch/turn-001/replay.json
ai-chromium/router-server/test/fixtures/artifacts/replay-mismatch/turn-001/evaluation.json
ai-chromium/router-server/test/fixtures/artifacts/redaction-fail/turn-001/manifest.json
ai-chromium/router-server/test/fixtures/artifacts/redaction-fail/turn-001/replay.json
ai-chromium/router-server/test/fixtures/artifacts/redaction-fail/turn-001/evaluation.json
ai-chromium/router-server/test/fixtures/artifacts/malformed-evidence/turn-001/manifest.json
ai-chromium/router-server/test/fixtures/artifacts/malformed-evidence/turn-001/replay.json
ai-chromium/router-server/test/fixtures/artifacts/malformed-evidence/turn-001/evaluation.json
ai-chromium/router-server/test/fixtures/artifacts/malformed-evidence/turn-001/evidence.ndjson
```

Router source change should stay limited to simplifying the artifact validator/test harness. The validator already checks required JSON files, replay match, evaluation replay match, redaction pass, turn ID consistency, and parseable NDJSON.

## Required Fixture Semantics

```text
valid:
  one turn directory
  manifest.turn_id = replay.turn_id = evaluation.turn_id
  replay.replay_match = true
  evaluation.replay_match = true
  evaluation.redaction_pass = true
  evidence.ndjson = parseable JSON lines

missing-manifest:
  one turn directory without manifest.json
  expected error includes "missing manifest.json"

replay-mismatch:
  replay.replay_match = false
  expected error includes "replay.replay_match must be true"

redaction-fail:
  evaluation.redaction_pass = false
  expected error includes "evaluation.redaction_pass must be true"

malformed-evidence:
  evidence.ndjson contains one malformed JSON line
  expected error includes "malformed JSON"
```

## Validation Commands

Run after the EXECUTE-stage fixture additions:

```bash
git status --short
node --check ai-chromium/router-server/src/server.mjs
cd ai-chromium/router-server && node --test test/openai-contract.test.mjs
cd ai-chromium/router-server && node --test test/mock-cdp-integration.test.mjs
cd ai-chromium/router-server && node --test test/artifact-quality.test.mjs
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

Rust validation remains required when a Rust toolchain and the configured wrapper are available:

```bash
RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check
RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets
RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings
```

The `RUSTC_WORKSPACE_WRAPPER=""` override is validation-only. It avoids the currently absent `/workspace/ai_sandbox/canon-rustc-v2/target/debug/canon-rustc-v2` wrapper without changing `.cargo/config.toml`.

## Expected Score Movement

```text
N: router evidence improves from 5.6 if artifact-quality moves 0/5 fail -> 5/5 pass
E: evidence quality improves because a committed negative/positive fixture set becomes executable
M: missing_router_artifact_quality_fixtures becomes false
K/C/R: unchanged; root Rust and graph blockers remain open
```

This plan does not claim root runtime validation. It only targets one concrete, reproducible failure that can be closed now.

## Risks And Constraints

```text
risk_source_mutation = low, fixtures only
risk_false_positive = low, test asserts both accept and reject paths
risk_scope_creep = medium, mitigated by forbidding router source edits
risk_root_score_overclaim = medium, mitigated by leaving Rust/toolchain blockers explicit
```

Constraints:

- Do not modify kernel, runtime, capability, API, or browser/provider source in this stage.
- Do not alter `score.md` unless the EXECUTE-stage validation result changes evidence.
- Do not hide root Rust/toolchain, graph, runtime archive, or live CDP gaps.
- Keep generated validation output outside committed source unless explicitly requested.

## Safe Git Delta Procedure

Use the uploaded bundle base preserved by `origin/main`:

```bash
git status --short
git bundle create /mnt/data/repo-delta-0001.bundle b9830281da5618db55c12371ec1f17b3abdd0b00..HEAD
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
implementation_target = router artifact-quality fixtures
validation_scope = documentation_plan_only
delta_base = b9830281da5618db55c12371ec1f17b3abdd0b00
```

## Execute-Stage Result

```text
artifact_quality_before = fail, 0/5, missing fixture roots
artifact_quality_after = pass, 5/5
source_simplification = table-driven artifact-quality tests + shorter validator path
root_rust_validation = still unavailable here because cargo/rustc are absent
```