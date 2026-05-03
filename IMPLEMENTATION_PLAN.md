# Implementation Plan

## Stage

```text
stage = PLAN
base_commit = dc1f3f8227ed7f67f6adbf728315160cdd71c920
source_inputs = GOAL.md + score.md
source_code_changes_in_this_stage = none
```

## Variables

```text
EV = evaluation capability score
R  = runtime/replay confidence
VF = verification confidence
G  = graph/telemetry evidence availability
V  = reproduced validation freshness
M  = missing validation signal count
```

## Equation

```text
highest_impact = argmax(score_gain / implementation_risk)
target = validation_report_harness
expected_gain = max(EV, R, VF, G, V) with M reduced
```

One-line explanation: the fastest score improvement is not new capability breadth; it is making the existing deterministic runtime claims reproducible, inspectable, and machine-readable.

## Evidence From GOAL.md

`GOAL.md` defines the repository as a deterministic, self-improving agent runtime whose core value depends on replayable evidence, bounded recovery, append-only learning, and trustworthy outcomes. Its current-status section claims that the kernel, codec, runtime replay, local tooling/process effects, semantic verification, policy persistence, learning promotion, local Ollama judgment path, and bounded observation ingress are implemented as deterministic source surfaces.

The plan therefore targets validation and evidence production rather than speculative new architecture. A stronger proof surface directly supports the stated goal that every decision, recovery, and outcome is replayable and auditable.

## Evidence From score.md

`score.md` reports:

```text
CORE = 7.85 / 10
CAP  = 6.30 / 10
ARCH = 6.66 / 10
GOOD = VF = 8.4 / 10
```

The scorecard identifies validation freshness and missing artifact evidence as the primary blockers:

```text
missing_cargo_fmt = true
missing_cargo_test = true
missing_cargo_run_ollama_judgment = true
missing_clippy = true
missing_generated_graph_json = true
missing_rustc_wrapper_telemetry = true
missing_runtime_download_history = true
missing_conversation_snapshot = true
missing_artifact_apply_worktree = true
missing_external_observation_stream_test = true
missing_external_api_action_test = true
missing_semantic_artifact_verification_test = true
missing_policy_learning_replay_trace = true
```

The highest-leverage next work listed in `score.md` is an OBSERVE-stage validation script that emits one machine-readable report containing build status, test count, graph metrics, runtime archive metrics, and missing-signal flags.

## Target Work Item

Implement a repository validation harness that produces a deterministic machine-readable evidence report.

Recommended tracked output path:

```text
scripts/observe_validation.sh
```

Recommended generated output path, ignored or ephemeral unless explicitly requested:

```text
target/observe/validation-report.ndjson
```

The harness should collect, at minimum:

```text
git_head
git_status_clean
cargo_available
cargo_fmt_check_result
cargo_test_result
cargo_test_count_when_available
ollama_example_result_when_available
clippy_result_when_available
rustc_wrapper_configured
rustc_wrapper_path_exists
state_graph_present
graph_node_count_when_present
graph_edge_count_when_present
graph_intent_coverage_when_present
runtime_archive_present
runtime_archive_log_counts
runtime_archive_download_counts
missing_signal_flags
validation_summary
```

## Required Source Changes For EXECUTE Stage

1. Add `scripts/observe_validation.sh` as a portable shell harness.
2. Make it degrade gracefully when `cargo`, `rustc`, Ollama, graph artifacts, or runtime archives are unavailable.
3. Emit line-delimited JSON records so future stages can append, diff, and replay evidence.
4. Add a small README-style usage block inside the script comments or a short tracked doc section if needed.
5. Do not mutate core runtime behavior in this work item.
6. Do not claim cargo or graph validation passed unless the command actually ran successfully.

## Validation Commands

Run these after implementation in the EXECUTE stage:

```bash
git status --short
bash scripts/observe_validation.sh
test -s target/observe/validation-report.ndjson
python3 - <<'PY'
import json
from pathlib import Path
path = Path('target/observe/validation-report.ndjson')
for line_no, line in enumerate(path.read_text().splitlines(), 1):
    if line.strip():
        json.loads(line)
print(f'valid_ndjson_lines={line_no if path.read_text().splitlines() else 0}')
PY
cargo fmt --check || true
cargo test --all-targets || true
git diff --check
```

If Rust is available, `cargo fmt --check` and `cargo test --all-targets` should be treated as required pass/fail signals. If Rust is unavailable, the harness must record that absence explicitly rather than silently omitting it.

Optional local validation when Ollama is installed and running:

```bash
CANON_OLLAMA_BASE_URL=http://127.0.0.1:11434/v1 \
CANON_OLLAMA_MODEL=qwen2.5-coder:7b \
cargo run --example ollama_judgment
```

## Expected Score Movement

```text
EV: 5.9 -> 6.6+   because eval evidence becomes reproducible and machine-readable
R:  8.4 -> 8.5+   if replay/test status is emitted consistently
VF: 8.4 -> 8.6+   if proof/semantic checks are captured by the report
ARCH confidence: medium/low -> medium if validation freshness improves
```

This does not make the system autonomous by itself. It closes the largest evidence gap blocking credible scoring and future execution stages.

## Risks And Constraints

```text
risk_source_mutation = low, if limited to scripts and generated report paths
risk_false_claims = medium, mitigated by explicit unavailable/fail/pass records
risk_environment_dependency = medium, mitigated by graceful command detection
risk_score_overfit = low, because this improves evidence collection rather than hand-editing scores
```

Constraints:

- Preserve the frozen-kernel boundary.
- Preserve the existing runtime/capability source behavior.
- Do not require Ollama for baseline validation.
- Do not require graph artifacts to exist; detect and report absence.
- Keep generated validation output outside committed source unless explicitly requested.

## Safe Git Delta Procedure

After EXECUTE-stage implementation and validation:

```bash
git status --short
git add scripts/observe_validation.sh IMPLEMENTATION_PLAN.md
git commit -m "Add observe validation plan and harness"
git bundle create /mnt/data/repo-delta-0001.bundle dc1f3f8227ed7f67f6adbf728315160cdd71c920..HEAD
git bundle verify /mnt/data/repo-delta-0001.bundle
```

Receiver apply command:

```bash
git fetch ./repo-delta-0001.bundle HEAD && git merge --ff-only FETCH_HEAD
```

## Plan-Stage Validation

```text
source_code_changed = false
plan_file_changed = IMPLEMENTATION_PLAN.md
implementation_target = scripts/observe_validation.sh
validation_scope = documentation_plan_only
```

## Execute-Stage Result

```text
implemented = scripts/observe_validation.sh
generated_report = target/observe/validation-report.ndjson
generated_report_committed = false
validation_report_ndjson_lines = 10
cargo_available = false
rustc_available = false
runtime_archive_present = true
runtime_archive_log_total = 2
runtime_archive_download_total = 0
state_graph_present = false
missing_signal_count = 13
source_runtime_behavior_changed = false
```

One-line explanation: the implementation creates the promised evidence harness and records the remaining validation gaps without mutating the kernel or capability runtime behavior.