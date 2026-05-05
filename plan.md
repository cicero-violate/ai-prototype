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

- `GOAL.md` requires externally verified candidates, typed receipts, replayable TLog evidence, kernel-gated deployment, and bounded recovery.
- `score.md` identifies API/transport proof, deterministic auditability, validation/toolchain, performance, robustness, and simplicity risks. `score.md` is preserved and must not be edited in this phase.
- Turn 1 added atomic duplicate process receipt batch rejection.
- Turn 2 bumped the external API protocol schema to version 5 for that command contract change.
- Runtime archive inspection found logs, network request traces, download indexes, candidate/audit ledgers, a current run summary, and prior delta receipts in `/mnt/data/ai-runtime.tar.gz`.

## Boundary

- Do not edit `score.md`.
- Preserve all prior committed changes from this conversation.
- Create final delta artifacts only after committing this turn.
- Make a bounded source change that improves correctness, robustness, and performance without broad architecture churn.
- Keep policy-learning ownership unchanged: learning promotes; policy stores append-only entries.

## Tasks

1. Refresh `plan.md` from `GOAL.md` and `score.md`.
2. Preserve Turn 1 duplicate process receipt batch rejection.
3. Preserve Turn 2 schema-version rejection for stale envelopes.
4. Close one bounded API robustness gap by limiting command batch fan-out.
5. Bump the API protocol schema again because the command acceptance contract changed.
6. Add a deterministic test proving oversized process receipt batches are rejected atomically.
7. Run Rust bootstrap before validation.
8. Run bounded validation and keep `score.md` untouched.
9. Commit the cumulative turn result.
10. Create `/mnt/data/repo-delta-004.bundle` and `/mnt/data/DELTA_MANIFEST.md` for `B..H`.

## Completed This Turn

- Added `API_COMMAND_BATCH_LIMIT = 16` to the external API protocol.
- Rejected oversized `SubmitEvidenceBatch`, `SubmitObservationIngress`, and `SubmitProcessReceiptBatch` commands at the protocol contract boundary.
- Bumped `API_PROTOCOL_SCHEMA_VERSION` from `5` to `6` after the batch-boundary contract change.
- Renamed and updated the schema binding test from `api_protocol_schema_v5_binds_command_hash_to_payload` to `api_protocol_schema_v6_binds_command_hash_to_payload`.
- Added `api_rejects_oversized_process_receipt_batch_atomically` to prove oversized process receipt batches do not mutate state or TLog.
- Preserved Turn 1 atomic duplicate process receipt batch rejection.
- Preserved Turn 2 stale-envelope rejection by continuing to assert that `API_PROTOCOL_SCHEMA_VERSION - 1` is invalid.

## Validation Result

```text
python3 /mnt/data/bootstrap_rustc_session.py --skip-library-probe --skip-probe: pass
cargo test api_protocol_schema_v6_binds_command_hash_to_payload --offline -- --nocapture: pass
cargo test api_rejects_oversized_process_receipt_batch_atomically --offline -- --nocapture: pass
cargo test api_rejects_duplicate_process_receipt_batch_atomically --offline -- --nocapture: pass
cargo check --offline: pass
git diff --check: pass
git diff --exit-code -- score.md: pass, untouched
```

## Remaining Risk

- Full `cargo test --all-targets --no-fail-fast` remains too large for this bounded environment.
- `cargo fmt --check` still cannot run because the supplied bootstrap toolchain does not include `cargo-fmt`.
- This turn improves API command boundedness but does not implement external HTTP/gRPC transport, wrapper graph telemetry, live Ollama proof, or full observe-validation completion.
