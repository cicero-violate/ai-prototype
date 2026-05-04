# Implementation Plan

## Variables

```text
G = GOAL.md architectural authority
S = score.md reproduced-evidence authority
B = build, test, and toolchain reproducibility axis
V = verification, receipt binding, and proof replay axis
P = capability, policy, learning, and autonomy axis
D = documentation, delta discipline, and operational clarity axis
Q = expected score lift
R = implementation risk
BASE = c2f34ba6de533809414f6b11cccdd6592926d619
```

## Equations

```text
S = (K · C · V · P · B · E · N · D)^(1/8) = 6.64 / 10
GOOD = max(K,C,V,P,B,E,N,D) = K = 8.4 / 10 = good
next_work = argmax(Q - R) = close B first, then bind proof traces to V/P
```

One-line explanation: because `B = 3.0 / 10` is the geometric bottleneck,
the next implementation must convert missing current-head validation into a
portable, receipt-backed validation path without faking unavailable signals.

## Source Authority

`GOAL.md` defines the target system as a deterministic, self-improving agent
runtime with a frozen kernel, append-only TLog, replayable typed evidence,
semantic verification, policy learning, and monotonically cheaper reasoning.

`score.md` records the current reproduced state:

```text
K = 8.4 / 10
C = 7.8 / 10
V = 7.3 / 10
P = 6.0 / 10
B = 3.0 / 10
E = 8.0 / 10
N = 7.6 / 10
D = 7.2 / 10
S = 6.64 / 10
missing_signal_count = 12
```

Critical observed blockers:

```text
missing_root_rust_toolchain = true
missing_cargo_fmt = true
missing_cargo_test = true
missing_clippy = true
missing_cargo_run_ollama_judgment = true
missing_generated_graph_json = true
missing_rustc_wrapper_telemetry = true
missing_external_observation_stream_test = true
missing_external_api_action_test = true
missing_semantic_artifact_verification_test = true
missing_policy_learning_replay_trace = true
```

## Highest-Impact Implementable Target

Implement a **current-head validation closure** focused on build/test
reproducibility first, with no kernel or capability semantics changed unless a
failing reproduced test requires it.

Target files for the EXECUTE stage:

```text
scripts/observe_validation.sh
scripts/write_delta_manifest.py
IMPLEMENTATION_PLAN.md
score.md
```

Conditional target files only if validation exposes a concrete source defect:

```text
src/**
examples/**
```

Required implementation behavior:

```text
1. Detect cargo/rustc from PATH.
2. If absent, detect /mnt/data/rust-sandbox/bin and prepend it.
3. If .cargo/config.toml points to a missing rustc-wrapper, run root Rust
   validation with RUSTC_WRAPPER="" and RUSTC_WORKSPACE_WRAPPER="".
4. Record wrapper_override_required and wrapper_override_used explicitly.
5. Run current-head root checks when cargo is available:
   - cargo fmt --check
   - cargo test --all-targets
   - cargo clippy --all-targets -- -D warnings
6. Keep graph telemetry separate from root Rust validation:
   - root tests passing without wrapper improves B only
   - graph score improves only with state/rustc/*/graph.json evidence
7. Preserve every missing signal that remains unavailable.
8. Generate a validation receipt and DELTA_MANIFEST.md from the same report.
9. Never commit target/, generated bundles, runtime archives, token caches, or
   signed URL caches.
```

If Rust validation becomes available and passes, the next highest-impact source
change is a pure offline integration proof that exercises:

```text
observation -> judgment -> eval -> learning -> policy promotion -> replay
semantic artifact verification -> receipt/proof rejection on tamper
```

This second target directly closes `missing_policy_learning_replay_trace` and
`missing_semantic_artifact_verification_test` while staying aligned with
`GOAL.md`'s self-improvement and semantic-verification requirements.

## Explicit Non-Targets

```text
no_kernel_rewrite = true
no_state_machine_semantic_change_without_test_failure = true
no_test_relaxation = true
no_fake_graph_json = true
no_fake_ollama_proof = true
no_live_cdp_claim_without_live_cdp_run = true
no_generated_artifacts_committed = true
no_delta_bundle_without_manifest = true
```

## Validation Commands

Plan-stage validation:

```bash
git status --short
rg -n "S = 6.64|B = 3.0|Highest-Impact Implementable Target|Safe Git Delta Procedure" IMPLEMENTATION_PLAN.md
git diff --check
```

Execute-stage validation:

```bash
git status --short
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz \
CANON_OBSERVE_REPORT=target/observe/validation-report-execute.ndjson \
bash scripts/observe_validation.sh
```

Run these explicitly when cargo is available:

```bash
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings
```

Run graph checks only when the configured wrapper exists and emits graph data:

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

Manifest validation:

```bash
python3 scripts/write_delta_manifest.py \
  --base "$(git rev-parse origin/main)" \
  --head "$(git rev-parse HEAD)" \
  --report target/observe/validation-report-execute.ndjson \
  --out /mnt/data/DELTA_MANIFEST.md \
  --receipt-out target/observe/delta-validation-receipt.json
python3 - <<'PY'
import json
from pathlib import Path
receipt = json.loads(Path('target/observe/delta-validation-receipt.json').read_text())
required = {
    'base_commit', 'head_commit', 'changed_files', 'validation_status',
    'validation_commands', 'validation_command_count', 'validation_test_count',
    'missing_signal_flags', 'wrapper_override_required', 'wrapper_override_used',
}
missing = sorted(required - set(receipt))
assert not missing, missing
assert receipt['base_commit'] == receipt['base_commit'].strip()
assert receipt['head_commit'] == receipt['head_commit'].strip()
print(json.dumps({k: receipt[k] for k in sorted(required) if k != 'validation_commands'}, sort_keys=True))
PY
git diff --check
```

## Expected Score Movement

```text
B: 3.0 -> 4.0  implemented: deterministic toolchain absence/presence, wrapper override, repo metrics, and delta-base ancestry
B: 3.0 -> 5.0  if fmt/test/clippy run at current head and pass
V: 7.3 -> 7.7  if semantic artifact verification has an offline proof/tamper test
P: 6.0 -> 6.6  if policy-learning replay trace is proven by an offline integration test
D: 7.2 -> 7.4  implemented: manifest receipt can carry bundle hash, changed-file count, and validation metrics
S: 6.64 -> 6.90 from B/D closure; above 7.10 only if V/P traces also close
```

Do not raise graph telemetry, live Ollama, live CDP, or external API action
scores without concrete current-head evidence.

## Risks And Constraints

| Risk | Severity | Required control |
|---|---:|---|
| False confidence from wrapper override | High | Record override fields; do not count graph telemetry without graph output. |
| Missing Rust toolchain | High | Preserve missing flags; do not mark root validation pass. |
| Generated artifact drift | Medium | Build manifest from the same observe report used for scoring. |
| Stale base/head artifact | Medium | Use `origin/main` as base and verify bundle before final links. |
| Runtime archive overclaim | Medium | Treat archive evidence as historical unless current-head validation reproduces it. |
| Scope creep into kernel | High | Keep source changes validation-only unless a reproduced failure requires code repair. |

## Safe Git Delta Procedure

For EXECUTE turn 004, use the explicit stage base and turn-scoped artifact name:

```bash
B="c2f34ba6de533809414f6b11cccdd6592926d619"
H="$(git rev-parse HEAD)"
git status --short
rm -f /mnt/data/repo-delta-004.bundle /mnt/data/DELTA_MANIFEST.md
git bundle create /mnt/data/repo-delta-004.bundle "$B..$H"
git bundle verify /mnt/data/repo-delta-004.bundle
```

Receiver apply command:

```bash
git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD
```

Required final answer shape for an EXECUTE stage:

```text
[repo-delta-004.bundle](sandbox:/mnt/data/repo-delta-004.bundle)
[DELTA_MANIFEST.md](sandbox:/mnt/data/DELTA_MANIFEST.md)
```

Required manifest fields:

```text
base_commit
head_commit
changed_files
validation_commands
validation_status
validation_command_count
validation_test_count
missing_signal_flags
wrapper_override_required
wrapper_override_used
bundle_verify
receiver_apply_command
```