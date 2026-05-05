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

- `GOAL.md` requires externally verified candidates, typed receipts, replayable TLog evidence, and kernel-gated deployment.
- `score.md` identifies missing end-to-end integration proof, incomplete observe validation, absent wrapper graph telemetry, and simplicity/API transport risk. `score.md` is preserved and must not be edited in this phase.
- Active TODO/FIXME source scan remains clean outside plan/score text and generated build output.

## Boundary

- Do not edit `score.md`.
- Do not create final delta bundle or manifest during this intermediate turn.
- Make a bounded source change that improves correctness/auditability without broad architecture churn.
- Keep policy-learning ownership unchanged: learning promotes; policy stores append-only entries.

## Tasks

1. Refresh `plan.md` from `GOAL.md` and `score.md`.
2. Close one API correctness gap from the score risk surface: duplicate process receipts must not be accepted in one batch.
3. Preserve atomic command behavior: invalid batches must not mutate `State` or `TLog`.
4. Add Rust test coverage for duplicate process receipt batch rejection.
5. Run Rust bootstrap before validation.
6. Run bounded validation and keep `score.md` untouched.
7. Commit the turn change without producing final bundle artifacts.

## Completed This Turn

- Updated `Command::SubmitProcessReceiptBatch` contract validation to reject duplicate `receipt_hash` values.
- Added `process_receipt_hashes_are_unique` in `src/api/protocol.rs`.
- Added `api_rejects_duplicate_process_receipt_batch_atomically` in `src/lib.rs`.
- Verified that a duplicated sandbox process receipt batch returns `CanonError::InvalidApiCommand` and leaves both `State` and `TLog` unchanged.

## Validation Result

```text
python3 /mnt/data/bootstrap_rustc_session.py --skip-library-probe --no-install-launchers: pass
cargo test api_rejects_duplicate_process_receipt_batch_atomically -- --nocapture: pass
cargo check --offline: pass
python3 -m py_compile scripts/write_delta_manifest.py scripts/validate_policy_learning_trace.py scripts/validate_rust_panic_surface.py: pass
timeout 30s python3 -X faulthandler -m unittest tests.test_policy_learning_trace_contract -v: pass, 3 tests
timeout 30s python3 -X faulthandler -m unittest tests.test_observe_validation_contract -v: pass, 12 tests
timeout 30s python3 -X faulthandler -m unittest tests.test_write_delta_manifest -v: pass, 10 tests
timeout 30s python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json: pass, missing_count=0
timeout 30s python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json: pass, production_total=0
git diff --check: pass
git diff --exit-code -- score.md: pass, untouched
```

## Remaining Risk

- Full `cargo test --all-targets --no-fail-fast` was attempted but timed out during compilation in this environment before this bounded change was validated.
- `cargo fmt --check` could not run because the supplied bootstrap toolchain does not include `cargo-fmt`.
- This turn improves API batch correctness but does not implement external HTTP/gRPC transport, wrapper graph telemetry, live Ollama proof, or full observe-validation completion.
