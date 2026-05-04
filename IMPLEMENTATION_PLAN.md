# Implementation Plan

## Variables

```text
I,E,C,A,R,P,S,D,T,Co,Em,B,L,Si,F = scorecard dimensions
G = (I·E·C·A·R·P·S·D·T·Co·Em·B·L·Si·F)^(1/15)
B = 3d0c73cd5770487808d9b644a384697bc249fcae
H = committed execution head after the next EXECUTE stage
κ = implementation risk
```

One-line explanation: raise `G` by attacking the lowest reproducibility blocker
that can be fixed locally without changing the frozen kernel architecture.

## EXECUTE Turn 004 Result

```text
implemented = true
default_rustc_wrapper_removed = true
root_rust_validation_env = RUSTC_WRAPPER="" + RUSTC_WORKSPACE_WRAPPER=""
wrapper_graph_validation_env = explicit CANON_RUSTC_WRAPPER only
python_unit_tests = 9 passed
router_offline_tests = 45 passed
root_rust_validation = unavailable in this environment because cargo/rustc are missing
graph_validation = skipped_env_missing without CANON_RUSTC_WRAPPER
ollama_validation = skipped_env_missing without local endpoint/model
updated_G = 6.89 / 10
```

## Evidence From `GOAL.md` And `score.md`

```text
current_G = 6.72 / 10
goal_kernel = frozen deterministic reducer with hash-chained replayable TLog
goal_growth = capability-layer learning, policy promotion, local LLM receipts
current_head = 3d0c73cd5770487808d9b644a384697bc249fcae
validation_status = partial
python_delta_manifest_unit_tests = pass / 5 tests
router_offline_tests = pass / 45 checks
cargo_available = false
rustc_available = false
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
configured_rustc_wrapper_path_exists = false
state_graph_present = false
runtime_archive_present = true
runtime_archive_sha256 = b2578606919ef545e5bda7a87ddef2b362d5913ba1f796a9e138b4f17ecbbae5
runtime_archive_member_count = 94
runtime_download_history_records = 174
runtime_delta_apply_receipts = 16
runtime_conversation_snapshots = 0
missing_signal_count = 12
```

The architecture is coherent, but the weakest score dimensions are constrained
by unreproducible validation: root Rust checks cannot run, the configured
wrapper path is absolute and missing, graph telemetry is absent, and Ollama
proof replay is not reproduced.

## Highest-Impact Target

```text
next_work = argmax(ΔG / κ)
          = portable Rust validation and optional wrapper/graph closure
```

Target dimensions: `C`, `E`, `R`, `D`, `P`, `S`, `Em`, `Si`, and `F`.

Rationale: `score.md` lists Rust validation, wrapper recovery, and graph
telemetry as the top required work. The fastest durable gain is to remove the
absolute-wrapper failure mode, make Cargo checks runnable with wrapper variables
cleared, and make graph generation an explicit optional validation branch rather
than a hidden dependency on `/mnt/data/.../canon-rustc-v3`.

## Implementation Scope For Next EXECUTE Stage

1. Replace the hard-coded `.cargo/config.toml` `rustc-wrapper` path with a
   portable default that lets `cargo fmt`, `cargo test`, and `cargo clippy` run
   on a normal checkout.
2. Move wrapper-enabled graph capture behind an explicit environment contract,
   for example `CANON_RUSTC_WRAPPER=/path/to/canon-rustc-v3`, so missing wrapper
   state is reported as missing evidence rather than blocking all Rust checks.
3. Update `scripts/observe_validation.sh` to record three separate states:
   root Rust validation, wrapper graph validation, and Ollama proof replay.
4. Require root Rust validation commands to run when `cargo` and `rustc` are
   present, with `RUSTC_WRAPPER=` and `RUSTC_WORKSPACE_WRAPPER=` cleared unless
   wrapper validation is explicitly requested.
5. Add focused tests or fixture checks for the validation report semantics:
   - Cargo unavailable is reported as unavailable, not pass.
   - Cargo available with missing wrapper still runs root checks.
   - Wrapper requested with missing path records a missing graph signal.
   - Graph present records node/edge/function/intent metrics.
   - Delta manifest still rejects stale heads, empty commands, and zero tests
     without a reason.
6. Keep kernel/runtime source architecture unchanged unless a minimal validation
   fixture requires a small test-only adjustment.

## Out Of Scope

```text
kernel_rewrite = false
capability_rewrite = false
learning_policy_redesign = false
external_ollama_install = false
runtime_archive_cleanup = defer
conversation_snapshot_recovery = defer
duplicate_artifact_alias_cleanup = defer
```

Do not claim Rust, graph, or Ollama validation passed unless those commands
actually execute and pass in the current environment.

## Validation Commands

```bash
export B=3d0c73cd5770487808d9b644a384697bc249fcae

git status --short
git diff --check
git diff --check "$B..HEAD"

python3 -m unittest tests/test_write_delta_manifest.py

bash ai-chromium/router-server_bak/run_tests.sh

RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= cargo fmt --check
RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= cargo test --all-targets
RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= cargo clippy --all-targets -- -D warnings

CANON_RUSTC_WRAPPER=/path/to/canon-rustc-v3 \
CANON_RUSTC_V2_ARTIFACT_DIR=state/rustc \
cargo test --all-targets

CANON_OLLAMA_BASE_URL=http://127.0.0.1:11434/v1 \
CANON_OLLAMA_MODEL=qwen2.5-coder:7b \
RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= \
cargo run --example ollama_judgment

CANON_DELTA_BASE="$B" \
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz \
CANON_OBSERVE_REPORT=target/observe/validation-report.ndjson \
bash scripts/observe_validation.sh
```

If Cargo, wrapper graph capture, or Ollama are unavailable, the validation
report must record explicit missing signals and preserve partial status.

## Safe Git Delta Output

```bash
export B=3d0c73cd5770487808d9b644a384697bc249fcae
export H="$(git rev-parse HEAD)"
export OUT=/mnt/data
export TURN=001

git merge-base --is-ancestor "$B" "$H"
git diff --check
git diff --check "$B..$H"

rm -f "$OUT/repo-delta-${TURN}.bundle" "$OUT/DELTA_MANIFEST.md"
git bundle create "$OUT/repo-delta-${TURN}.bundle" "$B..$H"
git bundle verify "$OUT/repo-delta-${TURN}.bundle"
git bundle list-heads "$OUT/repo-delta-${TURN}.bundle" | rg "$H"

python3 scripts/write_delta_manifest.py \
  --base "$B" \
  --head "$H" \
  --report target/observe/validation-report.ndjson \
  --bundle "$OUT/repo-delta-${TURN}.bundle" \
  --out "$OUT/DELTA_MANIFEST.md" \
  --receipt-out target/observe/delta-validation-receipt.json
```

Receiver:

```bash
git fetch ./repo-delta-001.bundle HEAD
git merge --ff-only FETCH_HEAD
```

## Expected Score Movement

```text
C ↑ because root Rust checks become runnable and separately reported
E ↑ because validation no longer depends on one missing absolute path
R ↑ because wrapper and graph absence becomes explicit evidence
D ↑ because validation branches are deterministic and replayable
P ↑ because Cargo command timing can be captured when available
S ↑ because normal clones can validate without local /mnt/data coupling
Em ↑ because receivers can reproduce checks with fewer hidden assumptions
Si ↑ because one failing wrapper default is replaced by an explicit contract
F ↑ because the proof pipeline becomes portable across environments
G improves only when weak dimensions receive executable evidence
max(G) = good
```