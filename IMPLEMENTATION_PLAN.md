# Implementation Plan

## Variables

```text
G = GOAL.md authority
S = score.md evidence authority
B0 = d47aaa7d3487521df1d0b5d2ee11310934e9a359
H0 = 5853e08e88ceb853b1db31b299a78c81d0e00dae
Q = implementable score lift
R = implementation risk
V = validation evidence strength
D = safe git delta discipline
```

## Equations

```text
next_work = argmax(Q) = argmax((ΔE + ΔB + ΔD + ΔV - R) / changed_surface)
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10
```

One-line explanation: the next change should convert already available checks into current-head, replayable delta evidence without weakening the frozen-kernel boundary.

## Source Authority

`GOAL.md` defines the target as a deterministic agent runtime with a frozen kernel, append-only TLog, typed capability layer, bounded recovery, semantic verification, learning/policy promotion, and replayable evidence for every decision, recovery, and outcome.

`score.md` is the current evidence authority:

```text
K = 8.4 / 10
C = 7.8 / 10
V = 7.8 / 10
P = 6.1 / 10
B = 3.6 / 10
E = 6.7 / 10
N = 7.1 / 10
D = 7.1 / 10
S = 6.64 / 10
```

Current critical blockers from `score.md`:

```text
cargo_available = false
rustc_available = false
rustc_wrapper_path_exists = false
state_rustc_graph_json_present = false
ollama_judgment_example = skipped_env_missing
current_head_delta_receipts_with_nonzero_tests = missing
AI_delta_receipts_validation_status = null
AI_delta_receipts_validation_command_count = null
AI_delta_receipts_validation_test_count = null
runtime_manifest_history = stale_advisory
live_cdp_router_test = not_run
policy_learning_replay_trace = missing
```

## Highest-Impact Implementable Target

Implement a current-head validation receipt and safe-delta manifest generator.

Target files for EXECUTE stage:

```text
scripts/observe_validation.sh
scripts/write_delta_manifest.py or scripts/write_delta_manifest.sh
IMPLEMENTATION_PLAN.md / score.md only if evidence changes
```

Required behavior:

```text
1. Run the existing observe harness.
2. Run the nested router offline validation gate.
3. Run git diff --check.
4. Parse target/observe/validation-report.ndjson with Python.
5. Count validation commands, passed commands, failed commands, skipped commands, and router tests.
6. Emit a current-head receipt under target/observe/ with:
   - base_commit
   - head_commit
   - changed_files
   - validation_status
   - validation_commands
   - validation_command_count
   - validation_test_count
   - missing_signal_flags
   - output hashes or output paths
7. Generate DELTA_MANIFEST.md from the same receipt fields.
8. Keep generated receipts/manifests out of committed source unless explicitly requested.
```

This is higher impact than another scorecard-only edit because it directly closes the documented null validation receipt fields while preserving the existing router validation improvement. It is implementable in this sandbox because it depends on `python3`, `git`, `bash`, and the already-passing Node router checks, not on unavailable root Rust tooling.

## Explicit Non-Targets

```text
no_kernel_change = true
no_codec_change = true
no_runtime_state_machine_change = true
no_test_relaxation = true
no_claim_of_root_rust_validation = true
no_claim_of_graph_telemetry = true
no_claim_of_live_ollama = true
no_claim_of_live_cdp = true
```

Root Rust validation, graph telemetry, and live Ollama remain required for a higher score, but they are not currently executable here because `cargo`, `rustc`, and the configured wrapper path are unavailable.

## Validation Commands

Plan-stage checks:

```bash
git status --short
rg -n "current_head_delta_receipts|validation_status|validation_command_count|validation_test_count|Highest-Leverage" score.md IMPLEMENTATION_PLAN.md
bash scripts/observe_validation.sh
(cd ai-chromium/router-server && ./run_tests.sh)
git diff --check
```

Execute-stage checks after implementing the receipt/manifest generator:

```bash
git status --short
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz bash scripts/observe_validation.sh
(cd ai-chromium/router-server && ./run_tests.sh)
python3 scripts/write_delta_manifest.py \
  --base "$(git rev-parse origin/main)" \
  --head "$(git rev-parse HEAD)" \
  --report target/observe/validation-report.ndjson \
  --out /mnt/data/DELTA_MANIFEST.md \
  --receipt-out target/observe/delta-validation-receipt.json
python3 - <<'PY'
import json
from pathlib import Path
receipt = json.loads(Path('target/observe/delta-validation-receipt.json').read_text())
required = ['base_commit','head_commit','validation_status','validation_commands','validation_command_count','validation_test_count']
missing = [key for key in required if key not in receipt]
assert not missing, missing
assert receipt['validation_command_count'] > 0
assert receipt['validation_test_count'] >= 15
print(json.dumps({key: receipt[key] for key in required}, sort_keys=True))
PY
git diff --check
```

Rust validation remains required when the toolchain and wrapper are available:

```bash
RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check
RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets
RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings
CANON_OLLAMA_BASE_URL=http://127.0.0.1:11434/v1 CANON_OLLAMA_MODEL=qwen2.5-coder:7b cargo run --example ollama_judgment
```

## Expected Score Movement

```text
E: 6.7 -> 7.0  current-head runtime/delta evidence gains validation bindings
B: 3.6 -> 3.8  available checks become one reproducible receipt path
D: 7.1 -> 7.3  manifest, validation, and scorecard evidence align
V: 7.8 -> 7.9  replay/proof discipline improves for delta artifacts
S: 6.64 -> about 6.73
```

No increase should be assigned to kernel, root Rust correctness, graph telemetry, policy learning, live CDP, or live Ollama until those checks run successfully.

## Risks And Constraints

| Risk | Severity | Mitigation |
|---|---:|---|
| False confidence from non-Rust checks | High | Receipt must preserve missing Rust/wrapper/graph/Ollama flags. |
| Generated artifact drift | Medium | Manifest and receipt must derive from the same parsed validation report. |
| Stale base/head authority | Medium | Use `git rev-parse origin/main` and `git rev-parse HEAD` at generation time. |
| Test-count inflation | Medium | Count router tests from command output and keep root Rust count separate. |
| Scope creep into kernel/runtime | High | Change only validation/manifest tooling in EXECUTE stage. |

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
bundle_verify = pass
receiver_apply_command = git fetch ./repo-delta-0001.bundle HEAD && git merge --ff-only FETCH_HEAD
```

## Plan-Stage Result

```text
stage = PLAN
source_code_changed = false
plan_file_changed = IMPLEMENTATION_PLAN.md
implementation_target = current-head validation receipt + safe delta manifest generator
safe_delta_base = origin/main = d47aaa7d3487521df1d0b5d2ee11310934e9a359
current_observed_head = 5853e08e88ceb853b1db31b299a78c81d0e00dae
```