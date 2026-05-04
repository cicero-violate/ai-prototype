# Implementation Plan

## Variables

```text
B = 07ad58b4bf0e41e087a4584ecc97cd778a416a29
H = current HEAD after PLAN-stage commit
G = GOAL.md authority
S = score.md evidence authority
Q = implementable score lift
V = validation confidence
D = safe git delta discipline
```

## Equations

```text
next_work = argmax(Q) = argmax((Δreproducibility + Δevidence - Δrisk) / changed_surface)
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10
```

One-line explanation: choose the smallest change that converts known passing checks into a durable repository-level validation path.

## Source Authority

`GOAL.md` defines the target system as a deterministic agent runtime with a frozen kernel, append-only TLog, typed capabilities, bounded recovery, semantic verification, learning, policy promotion, and local Ollama judgment receipts.

`score.md` is the current evidence authority:

```text
K = 8.4 / 10
C = 7.8 / 10
V = 7.7 / 10
P = 6.1 / 10
B = 3.0 / 10
E = 6.4 / 10
N = 6.6 / 10
D = 6.9 / 10
S = 6.36 / 10
```

Current hard blockers from `score.md`:

```text
cargo_available = false
rustc_available = false
rustc_wrapper_path_exists = false
state_rustc_graph_json_present = false
ollama_judgment_example = skipped_env_missing
AI_delta_receipts_validation_commands = []
AI_delta_receipts_validation_test_count = 0
```

Root Rust validation remains the highest theoretical need, but it is not implementable in this sandbox without adding an external toolchain and the configured `/workspace/ai_sandbox/canon-rustc-v2/...` wrapper. The highest-impact implementable repository change is therefore the router validation wrapper.

## Highest-Impact Target For EXECUTE

Fix `ai-chromium/router-server/run_tests.sh` so it no longer assumes missing npm package metadata and instead runs the direct Node checks that already pass.

Current reproduced evidence:

```text
node --version = v22.16.0
node --check ai-chromium/router-server/src/server.mjs = pass
node --test test/openai-contract.test.mjs = pass, 7/7
node --test test/mock-cdp-integration.test.mjs = pass, 3/3
node --test test/artifact-quality.test.mjs = pass, 5/5
ai-chromium/router-server/package.json_present = false
ai-chromium/router-server/run_tests.sh = fail, npm ENOENT package.json missing
```

Planned source mutation for EXECUTE stage:

```text
file = ai-chromium/router-server/run_tests.sh
change = replace npm script assumptions with direct node commands
required_checks = syntax + openai-contract + mock-cdp-integration + artifact-quality
optional_live_check = gated by LIVE_ROUTER_URL or explicit live flag, not default
```

Do not modify kernel, codec, runtime, API, capability, test expectation logic, or fixture semantics for this target. The goal is reproducibility of already-passing evidence, not broad behavior change.

## Validation Commands

Run before source mutation:

```bash
git status --short
node --version
node --check ai-chromium/router-server/src/server.mjs
(cd ai-chromium/router-server && node --test test/openai-contract.test.mjs)
(cd ai-chromium/router-server && node --test test/mock-cdp-integration.test.mjs)
(cd ai-chromium/router-server && node --test test/artifact-quality.test.mjs)
(cd ai-chromium/router-server && ./run_tests.sh) # expected fail before fix
```

Run after source mutation:

```bash
git status --short
node --version
node --check ai-chromium/router-server/src/server.mjs
(cd ai-chromium/router-server && ./run_tests.sh)
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

Rust validation remains required when the Rust toolchain and wrapper are available:

```bash
RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check
RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets
RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings
CANON_OLLAMA_BASE_URL=http://127.0.0.1:11434/v1 CANON_OLLAMA_MODEL=qwen2.5-coder:7b cargo run --example ollama_judgment
```

The `RUSTC_WORKSPACE_WRAPPER=""` override is validation-only. It avoids the absent configured wrapper without weakening `.cargo/config.toml`.

## Expected Score Movement

```text
B: build/test reproducibility improves because a broken wrapper becomes a runnable gate
N: nested router-server evidence improves because direct passing checks become script-backed
D: documentation/plan alignment improves because validation commands match available metadata
K/C/V/P/E: unchanged; root Rust, graph, Ollama, learning, and current-head receipt gaps remain open
```

Estimated movement if validated:

```text
B: 3.0 -> 3.3
N: 6.6 -> 6.9
D: 6.9 -> 7.0
S: 6.36 -> about 6.45
```

This plan does not claim root Rust validation, graph telemetry, live CDP validation, live Ollama validation, current-head runtime archive proof, or policy-learning replay closure.

## Risks And Constraints

```text
risk_source_mutation = low, one shell wrapper only
risk_behavior_change = low, tests already pass when invoked directly
risk_false_confidence = medium, mitigated by keeping root Rust/Ollama/graph gaps explicit
risk_live_test_flake = medium, mitigated by leaving live CDP opt-in rather than default
risk_scope_creep = medium, mitigated by forbidding unrelated source edits
```

Constraints:

- Do not modify source code in PLAN stage.
- In EXECUTE stage, modify only `ai-chromium/router-server/run_tests.sh` unless validation reveals a directly blocking wrapper-only issue.
- Do not relax tests or redaction/artifact quality expectations.
- Do not hide absent root Rust toolchain, wrapper, graph, Ollama, learning, or receipt evidence.
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

Manifest fields to preserve in final delta output:

```text
base_commit = 07ad58b4bf0e41e087a4584ecc97cd778a416a29
head_commit = <current HEAD>
changed_files = cumulative files from B..H
validation_commands = exact commands run
validation_status = pass/fail with missing-signal notes
```

## Plan-Stage Result

```text
stage = PLAN
source_code_changed = false
plan_file_changed = IMPLEMENTATION_PLAN.md
implementation_target = fix router run_tests.sh to use direct Node validation
validation_scope = documentation_plan_only + reproduced current router wrapper failure
safe_delta_base = origin/main = 07ad58b4bf0e41e087a4584ecc97cd778a416a29
```