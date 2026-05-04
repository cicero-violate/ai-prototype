# Implementation Plan

## Variables

```text
G = GOAL.md authority
S = score.md evidence authority
B = build / test reproducibility score
V = replay / verification proof score
E = runtime archive evidence score
D = documentation / delta discipline score
N = nested router-server evidence score
Q = implementable score lift
R = implementation risk
BASE = 44945bf71389366f1566abf48f70a81b924fab96
HEAD = 418f6722ddb02b15a1b09dd1306c3733ada4a9c1
```

## Equations

```text
next_work = argmax(Q) = argmax((ΔB + ΔV + ΔE + ΔD - R) / changed_surface)
S = (K · C · V · P · B · E · N · D)^(1/8) = 6.84 / 10
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10 = good
```

One-line explanation: the next change should turn the largest blocker, missing
root validation, into a portable validation path without weakening the frozen
kernel or claiming unavailable proof.

## Source Authority

`GOAL.md` defines the target as a deterministic agent runtime with a frozen
kernel, append-only TLog, typed capability layer, bounded recovery, semantic
verification, policy learning, and replayable evidence for every decision,
recovery, and outcome.

`score.md` is the current evidence authority:

```text
K = 8.4 / 10
C = 7.8 / 10
V = 7.6 / 10
P = 6.2 / 10
B = 4.0 / 10
E = 7.5 / 10
N = 7.4 / 10
D = 7.0 / 10
S = 6.84 / 10
```

Current critical blockers from `score.md`:

```text
cargo_available = false
rustc_available = false
rustc_wrapper_path_exists = false
state_rustc_graph_json_present = false
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
ollama_judgment_example = skipped_env_missing
policy_learning_replay_trace = missing
live_cdp_router_test_at_current_head = missing
runtime_archive_conversation_snapshots = 0
```

## Highest-Impact Implementable Target

Implement a portable root-validation launcher and wrapper override path.

Target files for the EXECUTE stage:

```text
scripts/observe_validation.sh
scripts/write_delta_manifest.py
IMPLEMENTATION_PLAN.md / score.md only if evidence changes
```

Required behavior:

```text
1. Detect usable Rust tooling from PATH first.
2. If PATH lacks cargo/rustc, detect /mnt/data/rust-sandbox/bin and prepend it.
3. If .cargo/config.toml points to an absent rustc-wrapper, run root Rust checks
   with RUSTC_WORKSPACE_WRAPPER="" and record wrapper_override_used = true.
4. Run current-head root checks when cargo becomes available:
   - cargo fmt --check
   - cargo test --all-targets
   - cargo clippy --all-targets -- -D warnings
5. Preserve strict missing-signal flags when checks remain unavailable.
6. Keep graph telemetry separate:
   - wrapper present + graph emitted = graph evidence
   - wrapper absent + root tests pass = build evidence only
7. Generate one current-head validation receipt from the observe report.
8. Generate DELTA_MANIFEST.md from the same receipt fields.
9. Do not commit generated target/observe reports, bundles, or /mnt/data artifacts.
```

This is higher impact than another documentation or router-only change because
`score.md` identifies build/test reproducibility as the lowest scored axis
(`B = 4.0`) and names missing root Rust validation plus absent wrapper as the
main blocker. The change is implementable now because it requires only validation
script/tooling changes; it does not mutate kernel, codec, runtime, capability, or
application logic.

## Explicit Non-Targets

```text
no_kernel_change = true
no_codec_change = true
no_runtime_state_machine_change = true
no_capability_logic_change = true
no_test_relaxation = true
no_fake_graph_telemetry = true
no_claim_of_live_ollama_without_env = true
no_claim_of_live_cdp_without_cdp_run = true
no_generated_artifacts_committed = true
```

Root Rust tests without the wrapper may raise `B`, but they do not close graph
telemetry. Graph scoring requires a valid wrapper and generated
`state/rustc/*/graph.json` evidence.

## Validation Commands

Plan-stage checks:

```bash
git status --short
rg -n "S = 6.84|BASE = 44945bf|Highest-Impact Implementable Target|wrapper_override_used|RUSTC_WORKSPACE_WRAPPER" IMPLEMENTATION_PLAN.md
git diff --check
```

Execute-stage checks after implementing the launcher:

```bash
git status --short
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz \
CANON_OBSERVE_REPORT=target/observe/validation-report-execute-turn.ndjson \
bash scripts/observe_validation.sh
(cd ai-chromium/router-server && ./run_tests.sh)
python3 scripts/write_delta_manifest.py \
  --base "$(git rev-parse origin/main)" \
  --head "$(git rev-parse HEAD)" \
  --report target/observe/validation-report-execute-turn.ndjson \
  --out /mnt/data/DELTA_MANIFEST.md \
  --receipt-out target/observe/delta-validation-receipt.json
python3 - <<'PY'
import json
from pathlib import Path
receipt = json.loads(Path('target/observe/delta-validation-receipt.json').read_text())
required = [
    'base_commit',
    'head_commit',
    'changed_files',
    'validation_status',
    'validation_commands',
    'validation_command_count',
    'validation_test_count',
    'missing_signal_flags',
]
missing = [key for key in required if key not in receipt]
assert not missing, missing
assert receipt['validation_command_count'] >= 6
assert receipt['validation_test_count'] >= 20
print(json.dumps({key: receipt[key] for key in required if key != 'validation_commands'}, sort_keys=True))
PY
git diff --check
```

Rust validation commands expected when toolchain detection succeeds:

```bash
RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check
RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets
RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings
```

Graph validation commands expected only when the configured wrapper exists:

```bash
cargo test --all-targets
find state/rustc -path '*/graph.json' -print
python3 - <<'PY'
import json
from pathlib import Path
paths = sorted(Path('state/rustc').glob('*/graph.json'))
assert paths, 'missing graph.json'
for path in paths:
    data = json.loads(path.read_text())
    print(path, len(data.get('nodes', [])), len(data.get('edges', [])))
PY
```

## Expected Score Movement

```text
B: 4.0 -> 4.8  if root fmt/test/clippy run with wrapper override and pass
D: 7.0 -> 7.2  validation receipt and manifest remain current-head aligned
E: 7.5 -> 7.6  runtime archive evidence binds to current validation receipt
V: 7.6 -> 7.7  replay/verification claims gain cleaner validation gating
S: 6.84 -> about 6.98
```

No increase should be assigned to graph telemetry, policy learning, live CDP, or
live Ollama until those checks run successfully with concrete evidence.

## Risks And Constraints

| Risk | Severity | Mitigation |
|---|---:|---|
| False confidence from wrapper override | High | Record `wrapper_override_used`; do not count graph telemetry unless wrapper emits graph. |
| Hidden dependency on `/mnt/data/rust-sandbox` | Medium | Prefer PATH first; record detected tool paths. |
| Test relaxation by environment mutation | High | Keep strict rustflags; only disable absent wrapper. |
| Generated artifact drift | Medium | Manifest and receipt derive from the same observe report. |
| Stale delta base | Medium | Use `git rev-parse origin/main` at artifact generation time. |
| Scope creep into runtime/kernel | High | EXECUTE changes limited to validation and manifest tooling. |

## Safe Git Delta Procedure

Use the uploaded restore base preserved by `origin/main`:

```bash
B="$(git rev-parse origin/main)"
H="$(git rev-parse HEAD)"
git status --short
git bundle create /mnt/data/repo-delta-0001.bundle "$B..$H"
git bundle verify /mnt/data/repo-delta-0001.bundle
```

Receiver apply command:

```bash
git fetch ./repo-delta-0001.bundle HEAD && git merge --ff-only FETCH_HEAD
```

Required final manifest fields:

```text
base_commit = <git rev-parse origin/main>
head_commit = <git rev-parse HEAD>
changed_files = cumulative files from base_commit..head_commit
validation_commands = exact commands run
validation_status = pass/fail/partial
validation_command_count = nonzero
validation_test_count = nonzero when router tests pass
missing_signal_flags = preserved from observe report
wrapper_override_used = true/false when Rust checks run
bundle_verify = pass
receiver_apply_command = git fetch ./repo-delta-0001.bundle HEAD && git merge --ff-only FETCH_HEAD
```

## Plan-Stage Result

```text
stage = PLAN
plan_schema = 2
source_code_changed = false
plan_file_changed = IMPLEMENTATION_PLAN.md
implementation_target = portable root-validation launcher + wrapper override path
safe_delta_base = origin/main = 44945bf71389366f1566abf48f70a81b924fab96
current_observed_head = 418f6722ddb02b15a1b09dd1306c3733ada4a9c1
```
