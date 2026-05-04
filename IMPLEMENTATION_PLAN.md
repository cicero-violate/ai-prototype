# Implementation Plan

## Variables

```text
I  = Intelligence
E  = Efficiency
C  = Correctness
A  = Alignment
R  = Robustness
P  = Performance
S  = Scalability
D  = Determinism
T  = Transparency
Co = Collaboration
Em = Empowerment
B  = Benefit
L  = Learning
Si = Simplicity
F  = Future-Proofing
G  = geometric goodness
Q  = expected score lift
X  = implementation risk
H0 = 4e8a41f7f6a4c31d275e986147d6193c1d1f3895
```

## Equations

```text
G = (I · E · C · A · R · P · S · D · T · Co · Em · B · L · Si · F)^(1/15)
GOOD = max(I,E,C,A,R,P,S,D,T,Co,Em,B,L,Si,F) = T = 8.0 / 10 = good
next_work = argmax(Q - X) = portable current-head validation closure
```

One-line explanation: the fastest honest score lift is to turn missing current-head validation into reproducible evidence without changing kernel semantics or faking unavailable graph/Ollama signals.

## Source Authority

`GOAL.md` defines the repo as a deterministic, self-improving agent runtime with a frozen kernel, append-only TLog, replayable typed evidence, bounded recovery, semantic verification, policy learning, LLM promotion, and monotonically cheaper reasoning.

`score.md` records the current observed score:

```text
I  = 7.6 / 10
E  = 6.4 / 10
C  = 6.2 / 10
A  = 7.8 / 10
R  = 6.1 / 10
P  = 5.8 / 10
S  = 6.2 / 10
D  = 7.4 / 10
T  = 8.0 / 10
Co = 6.6 / 10
Em = 7.0 / 10
B  = 7.5 / 10
L  = 6.3 / 10
Si = 5.4 / 10
F  = 6.8 / 10
G  = 6.70 / 10
```

Critical missing signals from `score.md`:

```text
missing_root_rust_toolchain = true
missing_cargo_fmt = true
missing_cargo_test = true
missing_clippy = true
missing_cargo_run_ollama_judgment = true
missing_generated_graph_json = true
missing_rustc_wrapper_telemetry = true
missing_conversation_snapshot = true
missing_external_observation_stream_test = true
missing_external_api_action_test = true
missing_semantic_artifact_verification_test = true
missing_policy_learning_replay_trace = true
```

## Highest-Impact Implementable Target

Implement a **portable current-head validation closure** that makes the repo able to report, verify, and manifest its validation state from the same evidence source.

This target improves the most score dimensions with the least semantic risk:

```text
C  improves by proving or explicitly preserving fmt/test/clippy failure states.
R  improves by removing hidden dependence on a missing absolute rustc-wrapper.
D  improves by making validation evidence replayable and receipt-backed.
T  remains strong by binding report, manifest, and bundle hashes.
Em improves by giving the receiver exact commands and missing-signal flags.
F  improves by making validation portable across the sandbox and local machines.
```

The target deliberately does **not** claim graph, Ollama, live CDP, policy-learning, or semantic-artifact success unless those commands actually run and produce current-head evidence.

## Execute-Stage Mutation Plan

Turn 004 implementation target: add the smallest router behavioral validation surface that can run without live CDP, then bind the result into the existing validation report, manifest, and bundle flow.

Primary files for the next EXECUTE stage:

```text
scripts/observe_validation.sh
scripts/write_delta_manifest.py
README.md
score.md
IMPLEMENTATION_PLAN.md
```

Turn 004 concrete mutations:

```text
ai-chromium/router-server_bak/test/offline-contract.test.mjs
ai-chromium/router-server_bak/run_tests.sh
README.md
score.md
IMPLEMENTATION_PLAN.md
```

Allowed source-code files only if reproduced validation exposes a concrete defect:

```text
src/**
examples/**
ai-chromium/**
```

Required behavior:

```text
1. Detect cargo/rustc on PATH.
2. If absent, detect /mnt/data/rust-sandbox/bin and prepend it.
3. Detect .cargo/config.toml rustc-wrapper.
4. If the wrapper path is missing, run root validation with:
   RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER=""
5. Record wrapper_override_required and wrapper_override_used explicitly.
6. Run git diff --check, router offline checks, cargo fmt, cargo test, and clippy.
7. Skip Ollama only when CANON_OLLAMA_BASE_URL or CANON_OLLAMA_MODEL is absent.
8. Preserve every unavailable signal as a missing_signal_flag.
9. Generate one validation report, one receipt, one manifest, and one bundle from the same head.
10. Keep generated bundles, runtime archives, target output, token caches, and signed URL caches uncommitted.
```

Secondary source target after validation closure passes:

```text
Add an offline integration test that proves:
observation -> judgment -> eval -> learning -> policy promotion -> replay
semantic artifact verification -> receipt/proof rejection on tamper
```

That secondary target directly attacks `missing_semantic_artifact_verification_test` and `missing_policy_learning_replay_trace`, but it should not precede current-head validation closure.

## Explicit Non-Targets

```text
no_kernel_rewrite = true
no_state_machine_semantic_change_without_reproduced_failure = true
no_test_relaxation = true
no_fake_graph_json = true
no_fake_ollama_receipt = true
no_live_cdp_claim_without_live_cdp_run = true
no_generated_artifacts_committed = true
no_delta_bundle_without_manifest = true
```

## Validation Commands

Plan-stage validation:

```bash
git status --short
rg -n "G = 6.70|portable current-head validation closure|Safe Git Delta Procedure" IMPLEMENTATION_PLAN.md
git diff --check
```

Execute-stage validation:

```bash
git status --short
CANON_DELTA_BASE="${BASE_COMMIT:?set base commit}" \
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz \
CANON_OBSERVE_REPORT=target/observe/validation-report-execute.ndjson \
bash scripts/observe_validation.sh
```

Root Rust validation when cargo exists:

```bash
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings
```

Ollama validation only when local endpoint exists:

```bash
CANON_OLLAMA_BASE_URL=http://127.0.0.1:11434/v1 \
CANON_OLLAMA_MODEL=qwen2.5-coder:7b \
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" \
cargo run --example ollama_judgment
```

Graph validation only when the configured wrapper exists and emits graph output:

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
  --base "$BASE_COMMIT" \
  --head "$(git rev-parse HEAD)" \
  --report target/observe/validation-report-execute.ndjson \
  --bundle /mnt/data/repo-delta-XXX.bundle \
  --bundle-verify pass \
  --out /mnt/data/DELTA_MANIFEST.md \
  --receipt-out target/observe/delta-validation-receipt.json

python3 - <<'PY'
import json
from pathlib import Path
receipt = json.loads(Path('target/observe/delta-validation-receipt.json').read_text())
required = {
    'base_commit', 'head_commit', 'changed_files', 'changed_file_count',
    'validation_status', 'validation_commands', 'validation_command_count',
    'validation_test_count', 'missing_signal_flags', 'wrapper_override_required',
    'wrapper_override_used', 'bundle_sha256', 'bundle_verify',
    'receiver_apply_command',
}
missing = sorted(required - set(receipt))
assert not missing, missing
assert receipt['bundle_verify'] == 'pass'
print(json.dumps({k: receipt[k] for k in sorted(required) if k != 'validation_commands'}, sort_keys=True))
PY
git diff --check
```

## Expected Score Movement

```text
C: 6.2 -> 6.8  if current-head fmt/test/clippy run and pass, or fail with exact evidence.
R: 6.1 -> 6.6  if wrapper absence is detected and safely overridden for root validation.
D: 7.4 -> 7.7  if validation report, receipt, manifest, and bundle all agree.
Em: 7.0 -> 7.3 if receiver apply commands and missing flags are complete.
F: 6.8 -> 7.1  if validation no longer depends silently on /workspace-only paths.
G: 6.70 -> ~6.90 from validation closure alone.
G: >7.05 only after semantic-artifact and learning replay tests also pass.
```

Do not raise graph telemetry, live Ollama, live CDP, external API action, semantic-artifact verification, or learning scores without concrete current-head output.

## Risks And Controls

| Risk | Severity | Control |
|---|---:|---|
| False confidence from wrapper override | High | Report override explicitly; never count graph telemetry without `graph.json`. |
| Missing Rust toolchain | High | Preserve missing flags; do not mark cargo checks as pass. |
| Validation-only work mistaken for source correctness | High | Separate evidence closure from behavioral feature changes. |
| Runtime archive overclaim | Medium | Treat archive logs as historical unless current head reproduces them. |
| Stale base/head delta | Medium | Verify base ancestor before bundle creation and manifest emission. |
| Generated artifact clutter | Medium | Exclude target, bundles, runtime archives, tokens, signed URL caches. |

## Safe Git Delta Procedure

Use the EXECUTE prompt's base commit as authority. If no new base is provided, use observed head `H0`.

```bash
BASE_COMMIT="${BASE_COMMIT:-4e8a41f7f6a4c31d275e986147d6193c1d1f3895}"
TURN="${TURN:-XXX}"
H="$(git rev-parse HEAD)"

git cat-file -e "$BASE_COMMIT^{commit}"
git merge-base --is-ancestor "$BASE_COMMIT" "$H"
git status --short

git bundle create "/mnt/data/repo-delta-${TURN}.bundle" "$BASE_COMMIT..$H"
git bundle verify "/mnt/data/repo-delta-${TURN}.bundle"
```

Receiver apply command:

```bash
git fetch ./repo-delta-XXX.bundle HEAD && git merge --ff-only FETCH_HEAD
```

Required final answer shape for EXECUTE:

```text
[repo-delta-XXX.bundle](sandbox:/mnt/data/repo-delta-XXX.bundle)
[DELTA_MANIFEST.md](sandbox:/mnt/data/DELTA_MANIFEST.md)
```

Required manifest fields:

```text
base_commit
head_commit
changed_files
changed_file_count
validation_status
validation_commands
validation_command_count
validation_test_count
missing_signal_flags
wrapper_override_required
wrapper_override_used
bundle_sha256
bundle_verify
receiver_apply_command
```