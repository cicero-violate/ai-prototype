# Implementation Plan

## Variables

```text
I,E,C,A,R,P,S,D,T,Co,Em,B,L,Si,F = scorecard dimensions
G = (I·E·C·A·R·P·S·D·T·Co·Em·B·L·Si·F)^(1/15)
H0 = ad35d4a71e71d02da4e28e16067cdfef5bb85baf
B = base commit supplied by the next EXECUTE artifact contract
H = committed execution head after the next EXECUTE stage
κ = implementation risk
```

One-line explanation: maximize `ΔG/κ` by turning the weakest current score
surface, performance evidence, into a reproducible validation contract without
relaxing correctness or claiming unavailable Rust/Ollama/graph checks.

## Source Evidence From `GOAL.md`

```text
goal_kernel = frozen deterministic reducer, hash, typed state
goal_audit = append-only replayable TLog with typed evidence
goal_recovery = bounded recovery with clean halt on exhausted budget
goal_growth = capability-layer learning, policy promotion, local LLM receipts
goal_cost = cost per objective completed falls monotonically over time
goal_boundary = kernel remains pure; intelligence grows above it
```

`GOAL.md` makes falling cost per objective an end-state property, not a side
metric. The repository currently records many runtime/archive events, but the
observe report does not promote latency, turn duration, download timing, or
throughput into scored validation evidence.

## Score Evidence From `score.md`

```text
current_G = 6.87 / 10
validation_status = partial
weakest_dimension = P = 5.3 / 10
next_weak_dimensions = Si=6.0, C=6.1, S=6.2, L=6.2, E=6.5
python_unit_tests = pass, 10 tests
router_offline_tests = pass, 45 tests
panic_surface_validation = pass, production_total 0
observe_validation_report_rows = 20
runtime_archive_log_total = 13728
runtime_archive_download_total = 237
runtime_download_history_records = 194
runtime_duplicate_artifact_aliases = 8
cargo_available = false
rustc_available = false
wrapper_graph_validation = skipped_env_missing
ollama_judgment_example = skipped_env_missing
missing_signal_count = 13
```

The prior highest-impact plan, production panic-surface reduction, is already
complete: OBSERVE records `production_total=0`. The remaining largest local score
ceiling is `P=5.3`, because no sustained-load, latency-budget, Rust timing,
benchmark, LLM latency, or memory signal is currently emitted. Rust, graph, and
Ollama validation remain environmental blockers in this container; they must stay
missing signals until the tools/endpoints exist.

## Runtime Archive Timing Evidence

Current runtime evidence can already support a performance contract if parsed by
`observe_validation.sh`.

```text
runtime_timing_records = 116
project_agent_elapsed_ms_records = 921
project_agent_elapsed_ms_median = 182049.489
project_agent_elapsed_ms_max = 1778293.765
download_initial_get_ms_records = 78
download_initial_get_ms_median = 12075.163
download_follow_get_ms_records = 78
download_follow_get_ms_median = 9909.547
download_resolved_get_ms_records = 38
download_resolved_get_ms_median = 6293.140
download_write_ms_records = 116
download_write_ms_median = 0.148
```

These values came from the uploaded runtime archive's NDJSON timing fields. They
are evidence, not a pass/fail contract yet. The next implementation should make
this data first-class and regression-checked.

## Highest-Impact Target

```text
next_work = argmax(ΔG / κ)
          = runtime performance evidence contract + observe-report regression gates
```

Target dimensions: `P`, `T`, `E`, `C`, `Em`, and secondarily `F`.

Rationale: `P=5.3` is the weakest dimension, and performance is currently scored
low because it lacks reproducible validation evidence. A portable parser and
budget gate can be implemented now using Python/Bash tests already available in
this environment. This improves the score basis without pretending that missing
Cargo, wrapper graph telemetry, or local Ollama replay passed.

## Implementation Scope For Next EXECUTE Stage

1. Extend `scripts/observe_validation.sh` with a `runtime_performance_metrics()`
   parser that reads the selected runtime archive and extracts:
   - project-agent turn `elapsedMs` count/min/median/p95/max;
   - artifact download `initialGetMs`, `followGetMs`, `resolvedGetMs`, and
     `writeMs` count/min/median/p95/max;
   - candidate count, download-event count, and duplicate-alias count;
   - command-duration summary from emitted validation commands.
2. Emit a dedicated `runtime_performance_metrics` event before
   `validation_summary`.
3. Add `validation_summary` fields for:
   - `runtime_performance_signal_present`;
   - `project_agent_elapsed_ms_median/max`;
   - `download_initial_get_ms_median/max`;
   - `download_follow_get_ms_median/max`;
   - `download_write_ms_median/max`;
   - `runtime_performance_budget_status`.
4. Add environment-configurable performance budgets with conservative defaults:
   - `CANON_MAX_PROJECT_AGENT_ELAPSED_MS_P95`;
   - `CANON_MAX_DOWNLOAD_INITIAL_GET_MS_P95`;
   - `CANON_MAX_DOWNLOAD_FOLLOW_GET_MS_P95`;
   - `CANON_MAX_DOWNLOAD_WRITE_MS_P95`.
5. Make the performance gate fail only when enough timing records exist and the
   budget is exceeded. If records are absent, keep status `partial` and emit an
   explicit missing signal instead of inventing a pass.
6. Extend `tests/test_observe_validation_contract.py` to require the new parser,
   event name, summary fields, budget variables, and missing-signal behavior.
7. Keep all changes outside the kernel/capability source path unless a minimal
   test fixture is needed. Do not alter runtime semantics, policy logic, LLM
   adapter behavior, or graph generation.
8. Update `score.md` only after the EXECUTE validation proves the new performance
   contract is emitted and reproducible.

## Out Of Scope

```text
kernel_rewrite = false
capability_rewrite = false
external_toolchain_install = false
claiming_rust_validation_without_cargo = false
claiming_graph_validation_without_CANON_RUSTC_WRAPPER = false
claiming_ollama_replay_without_endpoint_and_model = false
runtime_archive_deletion = false
conversation_snapshot_recovery = defer
semantic_artifact_verification = defer
policy_learning_replay = defer
external_api_action_test = defer
```

## Validation Commands

Run all available validation and preserve unavailable checks as explicit missing
signals.

```bash
export B="${CANON_DELTA_BASE:?set to artifact-contract base commit}"

# Repository hygiene
git status --short
git diff --check
git diff --check "$B..HEAD"

# Python contract validation
python3 -m unittest discover -s tests -p 'test_*.py' -v
python3 -m unittest tests/test_observe_validation_contract.py -v
python3 -m unittest tests/test_write_delta_manifest.py -v

# Panic-surface validation remains required
python3 scripts/validate_rust_panic_surface.py \
  --root . \
  --fail-production-unwrap \
  --report target/observe/panic-surface.json

# Router offline validation
bash ai-chromium/router-server_bak/run_tests.sh

# Full observe report with runtime archive performance extraction
CANON_DELTA_BASE="$B" \
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz \
CANON_OBSERVE_REPORT=target/observe/validation-report.ndjson \
bash scripts/observe_validation.sh

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
```

Expected local pass set in this container: Python unit tests, panic-surface
validation, router offline validation, git diff checks, and observe report
emission. Expected local missing set: Cargo, rustc, wrapper graph, generated
graph JSON, and Ollama proof replay.

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
P ↑ because runtime latency and download timing become measured and budgeted
T ↑ because performance evidence becomes explicit in the observe report
E ↑ because local validation gains a portable non-Rust signal
C ↑ slightly because regression gates reduce unchecked operational drift
Em ↑ because reviewers can reproduce performance evidence from the archive
F ↑ because future runs can compare timing distributions deterministically
G improves only when weak dimensions receive executable evidence
max(G) = good
```

The score must remain critical. Do not raise Cargo, graph, Ollama, semantic
artifact, policy-learning, external observation, or external API scores until
those checks are actually reproduced.
