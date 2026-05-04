# Implementation Plan

## Variables

```text
I,E,C,A,R,P,S,D,T,Co,Em,B,L,Si,F = scorecard dimensions
G = (I·E·C·A·R·P·S·D·T·Co·Em·B·L·Si·F)^(1/15)
B = dce0d671d547613915f35f00926b45d34bd821c6
H = committed execution head after this implementation stage
κ = implementation risk
```

One-line explanation: maximize score movement by converting missing validation
signals into deterministic proof or explicit, reproducible failure records.

## Current Evidence From `GOAL.md` And `score.md`

```text
current_G = 6.77 / 10
goal_kernel = frozen deterministic reducer with hash-chained replayable TLog
goal_growth = capability-layer learning, policy promotion, local LLM receipts
current_head = dce0d671d547613915f35f00926b45d34bd821c6
runtime_archive_present = true
runtime_archive_members = 84
runtime_manifest_base_commit = dce0d671d547613915f35f00926b45d34bd821c6
router_offline_tests = pass / 45 checks
cargo_available = false
rustc_available = false
configured_rustc_wrapper_path_exists = false
state_graph_present = false
ollama_judgment_example = skipped_env_missing
missing_signal_count = 12
implemented_now = delta manifest bundle/head verification, unique accepted alias reporting
```

## Highest-Impact Target

```text
next_work = argmax(ΔG / κ)
          = portable current-head validation closure
```

This targets `C`, `R`, `D`, `Em`, `F`, and `Si`: the score is capped less by
architecture and more by unavailable Cargo/Rust, missing graph telemetry,
missing Ollama proof replay, and noisy artifact lineage.

## Implemented This Stage

1. Kept the validation entrypoint as the canonical NDJSON evidence path for
   toolchain, wrapper, graph, Ollama env, runtime archive, and git delta state.
2. Preserved portable Cargo validation by clearing stale wrapper variables when
   the configured wrapper path is absent.
3. Preserved graph and Ollama gates as explicit `present/absent` and
   `verified/skipped_env_missing/fail` records.
4. Hardened delta manifest output: it now verifies the bundle and refuses output
   unless the bundle exposes the expected committed head.
5. Reduced artifact-lineage noise in observation output by reporting unique
   accepted download aliases separately from repeated candidate aliases.
6. Updated `score.md` with reproduced evidence only; missing Rust/Ollama/graph
   signals remain missing rather than inferred.

## Do Not Fake

```text
cargo_test = pass_or_unavailable_or_fail
graph_json = present_and_parsed_or_absent
ollama_receipt = verified_or_skipped_env_missing_or_fail
policy_learning_trace = real_replay_trace_or_missing
delta_bundle = verified_or_not_emitted
```

## Validation Commands

```bash
export B=dce0d671d547613915f35f00926b45d34bd821c6

git status --short
git diff --check

CANON_DELTA_BASE="$B" \
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz \
CANON_OBSERVE_REPORT=target/observe/validation-report.ndjson \
bash scripts/observe_validation.sh

RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= cargo fmt --check
RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= cargo test --all-targets
RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= cargo clippy --all-targets -- -D warnings

CANON_OLLAMA_BASE_URL=http://127.0.0.1:11434/v1 \
CANON_OLLAMA_MODEL=qwen2.5-coder:7b \
RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= \
cargo run --example ollama_judgment

find state/rustc -name graph.json -print
```

If Cargo, Rust, graph output, or Ollama are unavailable, the validation entrypoint
must emit that as evidence rather than silently passing.

## Safe Git Delta Output

```bash
export B=dce0d671d547613915f35f00926b45d34bd821c6
export H="$(git rev-parse HEAD)"

git merge-base --is-ancestor "$B" "$H"
git diff --check "$B..$H"

rm -f /mnt/data/repo-delta-004.bundle /mnt/data/DELTA_MANIFEST.md
git bundle create /mnt/data/repo-delta-004.bundle "$B..$H"
git bundle verify /mnt/data/repo-delta-004.bundle

python3 scripts/write_delta_manifest.py \
  --base "$B" \
  --head "$H" \
  --report target/observe/validation-report.ndjson \
  --bundle /mnt/data/repo-delta-004.bundle \
  --bundle-verify pass \
  --out /mnt/data/DELTA_MANIFEST.md \
  --receipt-out target/observe/delta-validation-receipt.json
```

Receiver:

```bash
git fetch ./repo-delta-004.bundle HEAD
git merge --ff-only FETCH_HEAD
```

## Expected Score Movement

```text
C,R,D,Em,F ↑ if Cargo/test/graph/Ollama proof becomes reproducible
T stays high if missing signals remain explicit
Si ↑ if artifact aliases and validation paths collapse
G improves only when weak dimensions receive real evidence
max(G) = good
```