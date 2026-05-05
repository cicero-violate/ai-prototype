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

Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers the whole system.

## Source Inputs

- `GOAL.md` requires deterministic command intake, replayable TLog evidence, kernel-gated deployment, evaluator-scored evolution, and verified receipts before learning or policy promotion.
- `score.md` is evidence input only and is not edited or staged in this phase.
- The scorecard risk register emphasizes incomplete external transport proof, missing wrapper graph telemetry, unavailable formatter/linter tools, live-provider proof gaps, and large-module simplicity debt.
- `/mnt/data/ai-runtime.tar.gz` was inspected before source work. It contains `RUNTIME_MANIFEST.json`, `.repo-agent-runtime/audit.ndjson`, `.repo-agent-runtime/loop-stop-receipts/turn_timeout-turn-2-bda1c8239c78cc96.json`, and `log/chatgpt_project_agent.ndjson`.
- Runtime archive evidence: manifest base commit matches `c9bf0f31aed64cf4bfe8ccfa22b99447fe3b501e`; included runtime files passed leak, integrity, and schema scans; download history was empty; no conversation snapshots or download indexes were present; the loop-stop receipt records a turn 2 timeout after `1800019.973 ms` with one blocking conversation API failure.

## Boundary

- Base commit: `c9bf0f31aed64cf4bfe8ccfa22b99447fe3b501e`.
- Worktree: `/mnt/data/ai-phase2-turn1/ai`.
- Do not edit or stage `score.md`.
- Remove stale `/mnt/data/repo-delta-002.bundle` and `/mnt/data/DELTA_MANIFEST.md` before final artifact creation.
- Run `/mnt/data/bootstrap_rustc_session.py` before Rust validation.
- Use `timeout 300s` for Rust validation and tests.
- Final bundle must be cumulative for `B..H`, not only the last patch.

## Tasks

1. Replace stale `plan.md` with the current plan derived from `GOAL.md`, `score.md`, and runtime archive evidence.
2. Improve the transport proof surface without introducing networking or provider dependencies.
3. Add a deterministic `ApiTransportSession` that owns runtime state, TLog, command ledger, and transport ledger as one replay-verifiable ingress unit.
4. Require session reconstruction from parts to verify both the TLog and persisted transport receipts before accepting frames.
5. Add integration tests proving session replay, no duplicate mutation on replay, and rejection of receipts that are not backed by TLog events.
6. Run bounded validation under the bootstrapped Rust toolchain.
7. Update this plan with completed work and validation evidence.
8. Commit the result and create `/mnt/data/repo-delta-002.bundle` plus `/mnt/data/DELTA_MANIFEST.md` for `B..H`.

## Completed This Turn

- Replaced stale plan content with the current Phase 2 turn 1 boundary, runtime evidence, tasks, completed work, validation, and remaining risks.
- Inspected `/mnt/data/ai-runtime.tar.gz` and recorded the available manifest, audit/process NDJSON, loop-stop receipt, empty download history, and absence of conversation snapshots/download indexes.
- Added `ApiTransportSession` in `src/api/transport.rs`.
- Added `ApiTransportSession::from_parts`, which rejects invalid restart state unless `verify_tlog` and `verify_api_transport_receipts` both pass.
- Added `ApiTransportSession::handle_frame`, `verify`, accessors, and `into_parts` for deterministic transport ingress/replay composition.
- Re-exported `ApiTransportSession` from `src/lib.rs`.
- Added integration tests for session replay verification and rejection of receipt-ledger state that is not backed by TLog events.
- Kept `score.md` untouched.

## Validation Result

```text
timeout 300s python3 /mnt/data/bootstrap_rustc_session.py: pass
rustc: rustc 1.75.0 (82e1608df 2023-12-21)
cargo: cargo 1.75.0 (1d8b05cdd 2023-11-20)

CARGO_INCREMENTAL=0 timeout 300s cargo check --offline: pass
CARGO_INCREMENTAL=0 timeout 300s cargo test --test api_transport_contract --offline: pass, 10 passed
CARGO_INCREMENTAL=0 timeout 300s cargo test --lib --offline: pass, 110 passed
CARGO_INCREMENTAL=0 timeout 300s cargo test --all-targets --offline: pass, 110 lib tests + 10 integration tests + 0-test binary/examples
timeout 300s python3 -m unittest discover -s tests -p 'test_*.py' -v: pass, 27 passed
timeout 300s python3 scripts/validate_policy_learning_trace.py --root . --report /mnt/data/ai-phase2-turn1-policy-learning-trace.json: pass, missing_count=0
timeout 300s python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report /mnt/data/ai-phase2-turn1-panic-surface.json: pass, production_total=0, test_total=335, example_total=1, finding_count=336
CARGO_INCREMENTAL=0 timeout 300s cargo run --example loop_trace --offline: pass, 31 events, success=true
timeout 300s cargo fmt --check: unavailable, cargo-fmt not installed
timeout 300s cargo clippy --all-targets --offline -- -D warnings: unavailable, clippy not installed
git diff --check: pass
```

## Remaining Risk

- Wrapper graph telemetry still requires an explicit root wrapper capture path.
- `rustfmt` and `clippy` remain unavailable in the supplied bootstrapped toolchain.
- Live Ollama/OpenAI provider paths remain endpoint-dependent and were not exercised in this turn.
- Transport proof now has a deterministic session object with restart verification, but still does not include a full HTTP/gRPC server implementation.
- Large-module simplicity debt remains in `src/lib.rs` and provider adapters.