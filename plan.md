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

- `GOAL.md` requires a deterministic runtime with replayable TLog evidence, bounded recovery, kernel-gated deployment, evaluator-scored evolution, and verified receipts before policy promotion or learning data reuse.
- `score.md` is evidence input only and is not edited or staged in this phase.
- The scorecard risk register emphasizes missing wrapper graph telemetry, unavailable formatter/linter tools, live-provider proof gaps, transport API proof gaps, and large-module simplicity debt.
- `/mnt/data/ai-runtime.tar.gz` was inspected before source work. It contains `RUNTIME_MANIFEST.json`, `.repo-agent-runtime/current-run-summary.json`, `.repo-agent-runtime/audit.ndjson`, `.repo-agent-runtime/69f9c035-5ad8-83ea-bc15-f2c90e024a82.messages.ndjson`, `.repo-agent-runtime/69f9c035-5ad8-83ea-bc15-f2c90e024a82.downloads.ndjson`, `.repo-agent-runtime/delta-apply-receipts/be6f4ff9181c78ba71fca7f28bbbced862bf019b.json`, and loop-stop receipts.
- Runtime archive evidence: manifest base commit matches `be6f4ff9181c78ba71fca7f28bbbced862bf019b`; prior delta application accepted the transport-session proof commit; loop-stop receipts record turn timeouts near `1800000 ms`, including one blocking conversation API failure.

## Boundary

- Base commit: `be6f4ff9181c78ba71fca7f28bbbced862bf019b`.
- Worktree: `/mnt/data/ai-phase2-turn1/ai`.
- Do not edit or stage `score.md`.
- Remove stale `/mnt/data/repo-delta-002.bundle` and `/mnt/data/DELTA_MANIFEST.md` before final artifact creation.
- Run `/mnt/data/bootstrap_rustc_session.py` before Rust validation.
- Use `timeout 300s` for Rust validation and tests.
- Final bundle must be cumulative for `B..H`.

## Tasks

1. Replace stale plan content with the current Phase 2 turn 1 boundary, runtime evidence, task selection, completed work, validation, and remaining risks.
2. Strengthen the deterministic API transport restart invariant without adding network or provider dependencies.
3. Require `ApiTransportSession` restart verification to prove that the in-memory command ledger is exactly reconstructable from the TLog.
4. Add a regression test proving stale command-ledger restart state is rejected instead of allowing duplicate API command mutation after restart.
5. Run bounded validation under the bootstrapped Rust toolchain.
6. Commit the result and create `/mnt/data/repo-delta-002.bundle` plus `/mnt/data/DELTA_MANIFEST.md` for `B..H`.

## Completed This Turn

- Inspected `/mnt/data/ai-runtime.tar.gz` and recorded manifest, audit, current-run summary, message ledger, download ledger, delta-apply receipt, and loop-stop timeout evidence.
- Updated `ApiTransportSession::from_parts` to verify that `CommandLedger` matches `CommandLedger::reconstruct_from_tlog(&tlog)` before accepting restored runtime state.
- Updated `ApiTransportSession::verify` to check the same command-ledger/TLog equality invariant during live session verification.
- Added a private `verify_command_ledger_matches_tlog` helper in `src/api/transport.rs`.
- Added `transport_session_rejects_stale_command_ledger_on_restart` in `tests/api_transport_contract.rs`.
- Proved that a restarted session with the session-derived command ledger is accepted, while the same TLog plus a stale empty command ledger is rejected with `CanonError::InvalidReplay`.
- Kept `score.md` untouched.

## Validation Result

```text
timeout 300s python3 /mnt/data/bootstrap_rustc_session.py: pass
rustc: rustc 1.75.0 (82e1608df 2023-12-21)
cargo: cargo 1.75.0 (1d8b05cdd 2023-11-20)

CARGO_INCREMENTAL=0 timeout 300s cargo check --offline: pass
CARGO_INCREMENTAL=0 timeout 300s cargo test --test api_transport_contract --offline: pass, 11 passed
CARGO_INCREMENTAL=0 timeout 300s cargo test --all-targets --offline: pass, 110 lib tests + 11 integration tests + 0-test binary/examples
timeout 300s python3 -m unittest discover -s tests -p 'test_*.py' -v: pass, 27 passed
timeout 300s python3 scripts/validate_policy_learning_trace.py --root . --report /mnt/data/ai-phase2-turn1-policy-learning-trace.json: pass, missing_count=0
timeout 300s python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report /mnt/data/ai-phase2-turn1-panic-surface.json: pass, production_total=0, test_total=335, example_total=1, finding_count=336
CARGO_INCREMENTAL=0 timeout 300s cargo run --example loop_trace --offline: pass, 31 events, success=true
timeout 300s cargo fmt --check: unavailable, cargo-fmt not installed
timeout 300s cargo clippy --all-targets --offline -- -D warnings: unavailable, clippy not installed
git diff --check: pass
```

## Remaining Risk

- Wrapper graph telemetry still requires an explicit root wrapper capture path and generated `state/rustc/*/graph.json` evidence.
- `rustfmt` and `clippy` remain unavailable in the supplied bootstrapped toolchain.
- Live Ollama/OpenAI provider paths remain endpoint-dependent and were not exercised in this turn.
- Transport proof now rejects stale session command ledgers on restart, but still does not include a full HTTP/gRPC server implementation.
- Large-module simplicity debt remains in `src/lib.rs` and provider adapters.
