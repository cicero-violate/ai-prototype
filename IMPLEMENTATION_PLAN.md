# Implementation Plan

## Variables

```text
I,E,C,A,R,P,S,D,T,Co,Em,B,L,Si,F = scorecard dimensions
G = (I·E·C·A·R·P·S·D·T·Co·Em·B·L·Si·F)^(1/15)
B = ebf09b8ad9d4b850404b2003f49d65c04af0b772
H = committed execution head after the next EXECUTE stage
κ = implementation risk
```

One-line explanation: maximize score movement by converting accepted-but-unvalidated
delta activity into mandatory, replayable validation evidence.

## Current Evidence From `GOAL.md` And `score.md`

```text
current_G = 6.66 / 10
goal_kernel = frozen deterministic reducer with hash-chained replayable TLog
goal_growth = capability-layer learning, policy promotion, local LLM receipts
current_head = ebf09b8ad9d4b850404b2003f49d65c04af0b772
validation_status_current = partial
router_offline_tests = pass / 45 checks
git_diff_check_current_no_base = pass
cargo_available = false
rustc_available = false
configured_rustc_wrapper_path_exists = false
state_graph_present = false
runtime_archive_present = true
runtime_archive_sha256 = 9439e3b0ca82ffd9f8e743948cd1f8d7fd9334d7e882f79f2db6118afad384f5
runtime_download_history_records = 160
runtime_duplicate_artifact_aliases = 8
runtime_delta_apply_receipts = 15 accepted / 15 verified
runtime_delta_apply_receipt_test_count = 0
runtime_loop_stop_receipts = 1 turn_timeout
```

The repo has a strong architectural target, but the weak score dimensions are
not primarily from missing ideas. They are from missing reproducible proof:
Cargo/Rust validation is unavailable, graph telemetry is absent, and prior
accepted delta receipts recorded no substantive validation commands or tests.

## Highest-Impact Target

```text
next_work = argmax(ΔG / κ)
          = strict validation-receipt closure for delta output
```

Target dimensions: `C`, `R`, `D`, `T`, `Em`, `F`, and `Si`.

Rationale: restoring a local Rust toolchain would improve validation, but it is
environment-dependent. The repository can be improved now by making safe delta
output reject empty validation receipts unless an explicit, machine-readable
zero-test reason is supplied. This directly attacks the strongest concrete risk
in `score.md`: accepted artifacts with `testCount=0` and no command evidence.

## Implementation Scope For Next EXECUTE Stage

1. Update delta-manifest/receipt generation so a delta cannot be emitted from a
   stale report, an empty command list, or an unverified bundle head.
2. Require `validation_command_count > 0` and persisted command/status rows.
3. Require `validation_test_count > 0`, or require an explicit field such as
   `zero_test_reason` when tests are impossible in the current environment.
4. Preserve partial validation as honest evidence: unavailable Cargo/Rust,
   missing graph telemetry, and skipped Ollama must remain visible instead of
   being promoted to pass.
5. Add or update focused tests for manifest generation using synthetic NDJSON
   reports that cover:
   - pass with commands and tests,
   - reject empty commands,
   - reject stale report head,
   - reject zero tests without reason,
   - allow zero tests only with explicit reason.
6. Keep source changes small and localized. Prefer `scripts/write_delta_manifest.py`
   and tests/fixtures over runtime architecture changes.

## Out Of Scope

```text
kernel_rewrite = false
architecture_rewrite = false
runtime_artifact_cleanup = defer
external_ollama_requirement = defer
external_rust_toolchain_install = defer
```

Do not hide missing toolchain, graph, or Ollama signals. Do not claim root Rust
validation passed unless `cargo fmt`, `cargo test`, and `cargo clippy` actually
run and pass.

## Validation Commands

```bash
export B=ebf09b8ad9d4b850404b2003f49d65c04af0b772

git status --short
git diff --check

CANON_DELTA_BASE="$B" \
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz \
CANON_OBSERVE_REPORT=target/observe/validation-report.ndjson \
bash scripts/observe_validation.sh

python3 scripts/write_delta_manifest.py \
  --base "$B" \
  --head "$(git rev-parse HEAD)" \
  --report target/observe/validation-report.ndjson \
  --bundle /mnt/data/repo-delta-XXX.bundle \
  --out /mnt/data/DELTA_MANIFEST.md \
  --receipt-out target/observe/delta-validation-receipt.json

RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= cargo fmt --check
RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= cargo test --all-targets
RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= cargo clippy --all-targets -- -D warnings

CANON_OLLAMA_BASE_URL=http://127.0.0.1:11434/v1 \
CANON_OLLAMA_MODEL=qwen2.5-coder:7b \
RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= \
cargo run --example ollama_judgment
```

If Cargo, Rust, graph output, or Ollama are unavailable, the validation report
must record that condition. Unavailability is evidence, not success.

## Safe Git Delta Output

```bash
export B=ebf09b8ad9d4b850404b2003f49d65c04af0b772
export H="$(git rev-parse HEAD)"

git merge-base --is-ancestor "$B" "$H"
git diff --check
git diff --check "$B..$H"

rm -f /mnt/data/repo-delta-XXX.bundle /mnt/data/DELTA_MANIFEST.md
git bundle create /mnt/data/repo-delta-XXX.bundle "$B..$H"
git bundle verify /mnt/data/repo-delta-XXX.bundle
git bundle list-heads /mnt/data/repo-delta-XXX.bundle | rg "$H"

python3 scripts/write_delta_manifest.py \
  --base "$B" \
  --head "$H" \
  --report target/observe/validation-report.ndjson \
  --bundle /mnt/data/repo-delta-XXX.bundle \
  --out /mnt/data/DELTA_MANIFEST.md \
  --receipt-out target/observe/delta-validation-receipt.json
```

Receiver:

```bash
git fetch ./repo-delta-XXX.bundle HEAD
git merge --ff-only FETCH_HEAD
```

## Expected Score Movement

```text
C ↑ because artifact acceptance gains executable validation evidence
R ↑ because empty/stale receipts become impossible or explicitly justified
D ↑ because bundle/head/report consistency is enforced
T ↑ because validation commands, tests, skips, and reasons are visible
Em ↑ because receivers get safer apply evidence
Si ↑ if duplicate artifact paths collapse into one manifest/receipt path
G improves only when weak dimensions receive real evidence
max(G) = good
```
