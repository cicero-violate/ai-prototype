# Implementation Plan

## Variables

```text
I,E,C,A,R,P,S,D,T,Co,Em,B,L,Si,F = scorecard dimensions
G = (I·E·C·A·R·P·S·D·T·Co·Em·B·L·Si·F)^(1/15)
H0 = cf2f814e32f9b30966fbbd0c71f17bb52106e556
B = base commit supplied by the next EXECUTE artifact contract
H = committed execution head after the next EXECUTE stage
κ = implementation risk
```

One-line explanation: maximize `ΔG/κ` by removing the largest locally fixable
correctness/robustness drag while preserving the frozen kernel boundary.

## Source Evidence From `GOAL.md`

```text
goal_kernel = frozen deterministic reducer, hash, typed state
goal_audit = append-only replayable TLog with typed evidence
goal_recovery = bounded recovery with clean halt on exhausted budget
goal_growth = capability-layer learning, policy promotion, local LLM receipts
goal_boundary = kernel remains pure; intelligence grows above it
```

`GOAL.md` makes correctness, bounded recovery, and auditability non-negotiable.
A production panic surface conflicts with that goal even when unsafe code is
forbidden.

## Score Evidence From `score.md`

```text
current_G = 6.96 / 10
validation_status = partial
python_unit_tests = pass, 9 tests
router_offline_tests = pass, 45 tests
cargo_available = false
rustc_available = false
cargo_fmt_check = unavailable
cargo_test_all_targets = unavailable
cargo_clippy_all_targets = unavailable
wrapper_graph_validation = skipped_env_missing
ollama_judgment_example = skipped_env_missing
state_graph_present = false
missing_signal_count = 13
unsafe_token_count_src_examples = 0
panic_call_count_src_examples = 0
unwrap_call_count_src_examples = 316
expect_call_count_src_examples = 9
```

Weakest dimensions are `P=5.5`, `C=6.0`, `Si=6.0`, `S=6.4`, `L=6.4`,
`E=6.6`, and `R=6.6`. Some blockers are environmental in this stage: Rust
build tools, wrapper graph capture, Ollama, and external observation/API tests
cannot be made true by editing repository files alone. The highest-impact local
change is therefore to reduce the panic/precondition surface and make that
reduction visible in validation evidence.

## Highest-Impact Target

```text
next_work = argmax(ΔG / κ)
          = production panic-surface reduction + static validation evidence
```

Target dimensions: `C`, `R`, `Si`, `T`, `Em`, and secondarily `E` and `F`.

Rationale: `score.md` explicitly flags `316` `unwrap()` calls and `9`
`expect()` calls as a large panic surface for a bounded-recovery system. Most
counts are in tests or broad source surfaces, but the current non-test hotspots
include deterministic runtime/policy paths that can be made total with typed
fallbacks or error propagation. This is implementable without installing
external services and can be checked by Python/static validation in the current
environment.

## Implementation Scope For Next EXECUTE Stage

1. Add a small static validation tool or test that reports `unwrap()`/`expect()`
   counts separately for:
   - production Rust code outside `#[cfg(test)]` modules;
   - test-only Rust code;
   - examples.
2. Replace the currently visible production `expect()` hotspots with typed or
   total logic:
   - `src/runtime/reducer.rs`: avoid assuming every non-escalation repair action
     has a gate/evidence pair; route invalid repair state to bounded halt or an
     explicit failure path.
   - `src/runtime/recovery_policy.rs`: replace policy-table `expect()` with a
     total `match` or explicit fallback that cannot panic.
   - `src/capability/policy/store.rs`: replace validated-key `expect()` calls
     with `Result` propagation or a deterministic fallback that preserves store
     integrity.
3. Keep all kernel semantics deterministic: no hidden I/O, no random fallback,
   no upward dependency, and no relaxation of `#![forbid(unsafe_code)]`.
4. Update observe/reporting metadata to include production-vs-test panic counts
   so future scoring does not treat test fixtures as runtime risk.
5. Update `score.md` only after validation evidence exists in the EXECUTE stage.
6. Do not rewrite the architecture, capability model, learning model, or
   Ollama path in this turn.

## Out Of Scope

```text
kernel_rewrite = false
capability_rewrite = false
external_toolchain_install = false
external_ollama_install = false
wrapper_graph_capture_without_CANON_RUSTC_WRAPPER = false
claiming_rust_validation_without_cargo = false
runtime_archive_cleanup = defer
conversation_snapshot_recovery = defer
benchmark_suite = defer
```

## Validation Commands

Run all validation available in the environment and record unavailable checks as
explicit missing signals rather than pass.

```bash
export B="${CANON_DELTA_BASE:?set to artifact-contract base commit}"

# Repository hygiene
git status --short
git diff --check
git diff --check "$B..HEAD"

# Existing executable validation in this environment
python3 -m unittest tests/test_write_delta_manifest.py
python3 -m unittest tests/test_observe_validation_contract.py
bash ai-chromium/router-server_bak/run_tests.sh

# New static panic-surface validation expected from this plan
python3 scripts/validate_rust_panic_surface.py \
  --root . \
  --fail-production-unwrap \
  --report target/observe/panic-surface.json

# Root Rust validation when a Rust toolchain is present
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings

# Optional graph validation only when wrapper exists
CANON_RUSTC_WRAPPER=/path/to/canon-rustc-v3 \
CANON_RUSTC_V2_ARTIFACT_DIR=state/rustc \
cargo test --all-targets

# Optional local LLM proof replay only when endpoint/model exist
CANON_OLLAMA_BASE_URL=http://127.0.0.1:11434/v1 \
CANON_OLLAMA_MODEL=qwen2.5-coder:7b \
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" \
cargo run --example ollama_judgment

# Full observe report
CANON_DELTA_BASE="$B" \
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz \
CANON_OBSERVE_REPORT=target/observe/validation-report.ndjson \
bash scripts/observe_validation.sh
```

If `cargo`, wrapper graph capture, or Ollama are unavailable, the observe report
must remain `partial` and must preserve the missing-signal flags.

## Safe Git Delta Output

The next EXECUTE stage must produce a cumulative bundle from the artifact
contract base `B` to the new committed head `H`.

```bash
export B="${CANON_DELTA_BASE:?set to artifact-contract base commit}"
export H="$(git rev-parse HEAD)"
export OUT=/mnt/data
export TURN="${CANON_TURN:-001}"

# Pre-bundle safety gates
git merge-base --is-ancestor "$B" "$H"
git diff --check
git diff --check "$B..$H"

# Bundle and verify cumulative delta
rm -f "$OUT/repo-delta-${TURN}.bundle" "$OUT/DELTA_MANIFEST.md"
git bundle create "$OUT/repo-delta-${TURN}.bundle" "$B..$H"
git bundle verify "$OUT/repo-delta-${TURN}.bundle"
git bundle list-heads "$OUT/repo-delta-${TURN}.bundle" | rg "$H"

# Manifest with validation receipt
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
git fetch ./repo-delta-${TURN}.bundle HEAD
git merge --ff-only FETCH_HEAD
```

## Expected Score Movement

```text
C ↑ if production panic paths are removed and validation records the reduction
R ↑ because recovery/policy paths become total instead of panic-dependent
Si ↑ because hidden precondition assumptions become explicit branches
T ↑ because panic counts are separated into production/test/example buckets
Em ↑ because reviewers can reproduce the risk metric without Cargo
E ↑ because static validation gives a local signal even when Rust tools are absent
F ↑ because bounded recovery semantics align better with future proof layers
G improves only when weak dimensions receive executable evidence
max(G) = good
```

## Execute Turn 004 Result

```text
implemented = production panic-surface reduction + static validation evidence
production_unwrap_expect_panic_total = 0
panic_surface_validation = pass
python_unit_tests = pass
router_offline_tests = pass
rust_toolchain_validation = unavailable in current container
```

Remaining unwrap/expect calls are classified as test-only fixture code. The next
score ceiling is still root Rust validation, wrapper graph telemetry, and local
Ollama proof replay.
