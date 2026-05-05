# Phase 2 Turn 1 Plan

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
G  = Goodness
```

## Equation

```text
G = (I*E*C*A*R*P*S*D*T*Co*Em*B*L*Si*F)^(1/15)
max(G) = good
```

One-line explanation: Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers the whole system.

## Source Inputs

- `GOAL.md` requires deterministic command intake, replayable TLog evidence, receipt lineage, evaluator-gated evolution, and policy learning from verified traces.
- `score.md` is evidence input only and is not edited or staged in this phase.
- Current score risks emphasize external transport proof, missing wrapper graph telemetry, live-provider proof gaps, unavailable fmt/clippy tooling, and large-module simplicity debt.
- `ai-runtime.tar.gz` was inspected. It contains a runtime manifest, audit/process NDJSON, message/download/candidate ledgers, bad-candidate state, current-run summary, and an older accepted delta-apply receipt. The manifest base matches this turn base `d5cb6f0055d29acae9b05b38d4027349f689d402`; older receipt evidence is advisory only.

## Boundary

- Base commit: `d5cb6f0055d29acae9b05b38d4027349f689d402`.
- Worktree: `/mnt/data/phase2-turn1-ai/ai`.
- Do not edit or stage `score.md`.
- Remove stale `/mnt/data/repo-delta-002.bundle` and `/mnt/data/DELTA_MANIFEST.md` before final artifact creation.
- Run `/mnt/data/bootstrap_rustc_session.py` before Rust validation.
- Run `nproc` before every Cargo command.
- Use `timeout 300s` for tests and Rust validation commands.

## Tasks

1. Replace stale `plan.md` with the current Phase 2 turn 1 plan derived from `GOAL.md` and `score.md`.
2. Preserve cumulative committed API transport receipt persistence and replay proof from base `d5cb6f0055d29acae9b05b38d4027349f689d402`.
3. Add a deterministic way to reconstruct `ApiTransportLedger` from persisted transport receipt NDJSON.
4. Reject duplicate persisted transport request ids during ledger reconstruction.
5. Add tests proving process-restart replay from persisted transport receipts and duplicate request-id rejection.
6. Run bootstrap and bounded validation.
7. Update this plan with completed work and validation evidence.
8. Commit the result and create `/mnt/data/repo-delta-002.bundle` plus `/mnt/data/DELTA_MANIFEST.md` for `B..H`.

## Completed This Turn

- Replaced the stale Phase 2 Turn 3 plan with a current Phase 2 Turn 1 plan and artifact boundary.
- Inspected `ai-runtime.tar.gz` and recorded its runtime manifest, audit/process logs, conversation/download/candidate ledgers, current run summary, bad-candidate state, and prior delta receipt evidence.
- Added `ApiTransportLedger::from_receipts` so persisted API transport receipts can rebuild the in-memory replay ledger after process restart.
- Added `load_api_transport_ledger_ndjson` and re-exported it through `src/lib.rs`.
- Hardened transport ledger insertion so invalid receipts or duplicate request ids are rejected instead of silently admitted.
- Added focused integration tests for restart replay from persisted transport receipts and duplicate persisted request-id rejection.
- Kept `score.md` untouched.

## Validation Result

```text
timeout 300s python3 /mnt/data/bootstrap_rustc_session.py --skip-probe --skip-library-probe: pass, rustc 1.75.0, cargo 1.75.0
nproc_before_cargo_check = 56
CARGO_INCREMENTAL=0 timeout 300s cargo check --offline: pass
nproc_before_cargo_test_api_transport = 56
CARGO_INCREMENTAL=0 timeout 300s cargo test --test api_transport_contract --offline: pass, 8 passed
nproc_before_cargo_test_lib = 56
CARGO_INCREMENTAL=0 timeout 300s cargo test --lib --offline: pass, 110 passed
nproc_before_cargo_test_all_targets = 56
CARGO_INCREMENTAL=0 timeout 300s cargo test --all-targets --offline: pass, 110 lib tests + 8 integration tests + 0-test binary/examples
python module tests split under timeout 300s: pass, 27 passed total
  - tests.test_observe_validation_contract: 13 passed
  - tests.test_panic_surface_contract: 1 passed
  - tests.test_policy_learning_trace_contract: 3 passed
  - tests.test_write_delta_manifest: 10 passed
timeout 300s python3 scripts/validate_policy_learning_trace.py --root . --report /mnt/data/ai-phase2-turn1-policy-learning-trace.json: pass, missing_count=0
timeout 300s python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report /mnt/data/ai-phase2-turn1-panic-surface.json: pass, production_total=0, test_total=335, example_total=1, finding_count=336
nproc_before_cargo_fmt = 56; timeout 300s cargo fmt --check: unavailable, cargo-fmt not installed
nproc_before_cargo_clippy = 56; timeout 300s cargo clippy --all-targets --offline -- -D warnings: unavailable, clippy not installed
git diff --check: pass
```

## Remaining Risk

- Wrapper graph telemetry still requires an explicit root wrapper capture path.
- `rustfmt` and `clippy` remain unavailable in the supplied bootstrapped toolchain.
- Live Ollama/OpenAI provider paths remain endpoint-dependent and were not exercised in this turn.
- Transport proof now covers deterministic command ingress plus persisted replay ledger reconstruction, but not a full HTTP/gRPC server implementation.
- Large-module simplicity debt remains in `src/lib.rs` and provider adapters.
