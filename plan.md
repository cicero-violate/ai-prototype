# Phase 2 Turn 3 Plan

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

- `GOAL.md` requires API command intake, deterministic receipts, replayable TLog evidence, policy learning, and evaluator-gated evolution.
- `score.md` is evidence input only and must not be edited or staged in this phase.
- Current score risks include external transport proof, missing wrapper graph telemetry, unavailable fmt/clippy tooling, live provider paths not exercised, and large-module simplicity debt.
- Runtime archive inspection found sanitized audit/process logs only; there were no download indexes, conversation snapshots, or prior apply worktrees in `ai-runtime.tar.gz`.
- Phase 2 turn 3 must preserve cumulative commits from base commit `7e21d9bf2295430508bf12f3cbfb7e52a639e15a`, validate with bounded commands, commit, and create the final cumulative bundle/manifest.

## Boundary

- Base commit: `7e21d9bf2295430508bf12f3cbfb7e52a639e15a`.
- Worktree: `/mnt/data/ai-work/ai`.
- Do not edit or stage `score.md`.
- Remove stale `/mnt/data/repo-delta-004.bundle` and `/mnt/data/DELTA_MANIFEST.md` before final artifact creation.
- Run `/mnt/data/bootstrap_rustc_session.py` before Rust validation.
- Run `nproc` before every Cargo command.
- Use `timeout 300s` for tests and Rust validation commands.

## Tasks

1. Replace stale `plan.md` content with a current Phase 2 turn 3 plan derived from `GOAL.md` and `score.md`.
2. Preserve the cumulative deterministic API transport frame and request-ledger changes from turns 1 and 2.
3. Add deterministic transport receipt NDJSON encode/decode/load/append functions so accepted API ingress can be audited outside process memory.
4. Add transport receipt verification against TLog event hashes, command ids, and command hashes.
5. Add focused transport tests proving persistence, replay verification, and tamper rejection.
6. Run bootstrap and bounded validation.
7. Update this plan with completed work and validation evidence.
8. Commit cumulative changes from `B..H`, then create `/mnt/data/repo-delta-004.bundle` and `/mnt/data/DELTA_MANIFEST.md`.

## Completed This Turn

- Replaced stale Phase 2 Turn 2 plan content with a current Phase 2 Turn 3 scope and final artifact boundary.
- Preserved cumulative API transport frame, request-id ledger, replay, conflict rejection, and no-mutation tamper tests from prior turns.
- Added deterministic transport receipt persistence: `ApiTransportReceipt::new`, receipt self-hash binding, NDJSON encode/decode/load/append helpers, and TLog-backed receipt verification.
- Added focused transport coverage proving receipt persistence, TLog verification, command-hash tamper rejection, and payload-hash tamper rejection through the receipt hash.
- Re-exported the new transport receipt helpers and constants through `src/lib.rs`.
- Inspected `ai-runtime.tar.gz`; it contains sanitized runtime manifest, audit NDJSON, and process NDJSON, with no download indexes, conversation snapshots, or prior runtime worktrees.
- Kept `score.md` untouched.

## Validation Result

```text
timeout 300s python3 /mnt/data/bootstrap_rustc_session.py --skip-probe --skip-library-probe: pass, rustc 1.75.0, cargo 1.75.0
nproc_before_cargo_check_after_receipt_hash = 56
CARGO_INCREMENTAL=0 timeout 300s cargo check --offline: pass
nproc_before_cargo_test_api_transport_after_receipt_hash = 56
CARGO_INCREMENTAL=0 timeout 300s cargo test --test api_transport_contract --offline: pass, 6 passed
nproc_before_cargo_test_lib_after_receipt_hash = 56
CARGO_INCREMENTAL=0 timeout 300s cargo test --lib --offline: pass, 110 passed
nproc_before_cargo_test_all_targets_after_receipt_hash = 56
CARGO_INCREMENTAL=0 timeout 300s cargo test --all-targets --offline: pass, 110 lib tests + 6 integration tests + 0-test binary/examples
timeout 300s python3 -m unittest discover -s tests -p 'test_*.py' -v: pass, 27 passed
timeout 300s python3 scripts/validate_policy_learning_trace.py --root . --report /mnt/data/ai-phase2-turn3-policy-learning-trace.json: pass, missing_count=0
timeout 300s python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report /mnt/data/ai-phase2-turn3-panic-surface.json: pass, production_total=0, test_total=335, example_total=1, finding_count=336
nproc_before_cargo_fmt = 56; timeout 300s cargo fmt --check --offline: unavailable, cargo-fmt not installed
nproc_before_cargo_clippy = 56; timeout 300s cargo clippy --all-targets --offline -- -D warnings: unavailable, clippy not installed
git diff --check: pass
```

## Remaining Risk

- Wrapper graph telemetry still requires an explicit root wrapper capture path.
- `rustfmt` and `clippy` are unavailable in the supplied bootstrapped toolchain unless a fuller toolchain is provided.
- Live Ollama/OpenAI provider paths remain endpoint-dependent and are not part of this turn.
- The transport surface is deterministic API ingress proof, not a full HTTP/gRPC server implementation.
- Large-module simplicity debt remains in `src/lib.rs` and provider adapters.