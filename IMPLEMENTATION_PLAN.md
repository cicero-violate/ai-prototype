# Implementation Plan

## Variables

```text
B  = ca0537f311aa35d538addc0236fb696acf5b8629
H0 = current observed HEAD before this plan commit
S  = score.md evidence state
G  = GOAL.md target architecture
M  = missing validation signal count
V  = validation freshness
E  = executable evidence coverage
R  = runtime/replay confidence
Q  = score improvement per implementation risk
```

## Equation

```text
next_work = argmax(Q) = argmax((ΔV + ΔE + ΔR - ΔM) / implementation_risk)
GOOD = max(K,C,A,R,OB,CX,ME,PL,LL,JG,TO,VF,EV,PO,LE,OR)
```

One-line explanation: the highest-impact next change is to convert the existing observe harness from passive evidence collection into a Rust-validation evidence gate that produces reproducible pass/fail records without touching kernel behavior.

## Source Authority

`GOAL.md` defines the repository as a deterministic, self-improving agent runtime whose value depends on replayable evidence, bounded recovery, append-only learning, and trustworthy outcomes. The current status claims kernel, codec, runtime replay, local tooling/process effects, semantic verification, policy persistence, learning promotion, local Ollama judgment, and bounded observation ingress are implemented.

`score.md` remains critical: the observe harness exists, but this environment still lacks reproduced Rust validation. The scorecard records `cargo_fmt_check`, `cargo_test_all_targets`, and `cargo_clippy_all_targets` as unavailable because `cargo`/`rustc` were missing here. It also keeps graph generation, rustc-wrapper telemetry, external observation/API tests, semantic artifact verification, and policy-learning replay trace as missing signals.

## Highest-Impact Target For EXECUTE

Strengthen the validation surface, not the core runtime.

Target file:

```text
scripts/observe_validation.sh
```

Target generated evidence:

```text
target/observe/validation-report.ndjson
```

Required EXECUTE-stage behavior:

1. Keep the script portable and deterministic.
2. Add explicit Rust validation records for:
   - `cargo fmt --check`
   - `cargo test --all-targets`
   - `cargo clippy --all-targets -- -D warnings`
3. Capture command availability, exit code, elapsed milliseconds, and output path for each validation command.
4. Preserve graceful degradation when `cargo`, `rustc`, clippy, Ollama, graph artifacts, or runtime archives are unavailable.
5. Emit NDJSON only; every non-empty line must parse as JSON.
6. Do not mutate kernel, runtime, capability, API, or application source behavior.
7. Do not claim any validation passed unless the command actually ran and exited successfully.

## Validation Commands

Run these during EXECUTE after implementation:

```bash
git status --short
bash scripts/observe_validation.sh
test -s target/observe/validation-report.ndjson
python3 - <<'PY'
import json
from pathlib import Path
path = Path('target/observe/validation-report.ndjson')
count = 0
for line in path.read_text().splitlines():
    if line.strip():
        json.loads(line)
        count += 1
print(f'valid_ndjson_lines={count}')
PY
cargo fmt --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
git diff --check
```

If Rust tooling is unavailable, the standalone commands may fail, but the harness must still record that absence explicitly. In a Rust-capable environment, the three cargo commands become required pass/fail score evidence.

Optional local validation when Ollama is installed and running:

```bash
CANON_OLLAMA_BASE_URL=http://127.0.0.1:11434/v1 \
CANON_OLLAMA_MODEL=qwen2.5-coder:7b \
cargo run --example ollama_judgment
```

## Expected Score Movement

```text
EV: improves if validation report becomes command-level reproducible evidence
R:  improves if runtime/replay tests are captured as command records
VF: improves if semantic/proof validation commands are recorded with exits
M:  decreases only when commands actually pass or artifacts are present
```

This plan does not raise scores by assertion. It creates the next execution target needed to replace stale workstation claims with fresh local evidence.

## Risks And Constraints

```text
risk_source_mutation = low, if limited to scripts/observe_validation.sh
risk_false_positive = medium, mitigated by explicit exit-code records
risk_environment_dependency = medium, mitigated by unavailable-tool records
risk_kernel_regression = low, because kernel/runtime source behavior is not targeted
```

Constraints:

- Preserve the frozen-kernel boundary.
- Preserve runtime/capability behavior.
- Keep generated validation output outside committed source unless explicitly requested.
- Preserve safe delta output from `B..H`.

## Safe Git Delta Procedure

After this PLAN-stage commit and any later EXECUTE-stage commit:

```bash
git status --short
git bundle create /mnt/data/repo-delta-0001.bundle ca0537f311aa35d538addc0236fb696acf5b8629..HEAD
git bundle verify /mnt/data/repo-delta-0001.bundle
```

Receiver apply command:

```bash
git fetch ./repo-delta-0001.bundle HEAD && git merge --ff-only FETCH_HEAD
```

## Plan-Stage Result

```text
source_code_changed = false
plan_file_changed = IMPLEMENTATION_PLAN.md
implementation_target = scripts/observe_validation.sh
validation_scope = documentation_plan_only
delta_base = ca0537f311aa35d538addc0236fb696acf5b8629
```
