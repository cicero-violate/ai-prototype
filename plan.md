# Phase 2 Turn 2 Plan

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

- `GOAL.md` requires externally verified candidates, typed receipts, replayable TLog evidence, kernel-gated deployment, and versioned policy/protocol authority.
- `score.md` identifies API/transport proof, deterministic auditability, validation/toolchain, and simplicity risks. `score.md` is preserved and must not be edited in this phase.
- Turn 1 added duplicate process receipt batch rejection, changing the API command contract surface.
- Active TODO/FIXME source scan remains clean outside plan/score text and generated build output.

## Boundary

- Do not edit `score.md`.
- Do not create final delta bundle or manifest during this intermediate turn.
- Make a bounded cumulative source change on top of Turn 1.
- Prefer correctness/auditability closure over broad architecture churn.
- Keep policy-learning ownership unchanged: learning promotes; policy stores append-only entries.

## Tasks

1. Refresh `plan.md` from `GOAL.md` and `score.md`.
2. Preserve the Turn 1 duplicate process receipt batch rejection.
3. Close the schema-versioning gap created by the changed API command contract.
4. Update protocol tests so stale envelopes remain rejected under the new schema version.
5. Run Rust bootstrap before validation.
6. Run bounded validation and keep `score.md` untouched.
7. Commit the turn change without producing final bundle artifacts.

## Completed This Turn

- Bumped `API_PROTOCOL_SCHEMA_VERSION` from `4` to `5` after the duplicate process receipt batch contract change.
- Renamed and updated the schema binding test from `api_protocol_schema_v4_binds_command_hash_to_payload` to `api_protocol_schema_v5_binds_command_hash_to_payload`.
- Preserved stale-envelope rejection by continuing to assert that `API_PROTOCOL_SCHEMA_VERSION - 1` is invalid.
- Preserved Turn 1 atomic duplicate process receipt batch rejection.

## Validation Result

```text
python3 /mnt/data/bootstrap_rustc_session.py --skip-library-probe --no-install-launchers: pass
cargo test api_protocol_schema_v5_binds_command_hash_to_payload --offline -- --nocapture: pass
cargo test api_rejects_duplicate_process_receipt_batch_atomically --offline -- --nocapture: pass
cargo check --offline: pass
git diff --check: pass
git diff --exit-code -- score.md: pass, untouched
```

## Remaining Risk

- Full `cargo test --all-targets --no-fail-fast` remains too large for this bounded intermediate turn environment.
- `cargo fmt --check` still cannot run because the supplied bootstrap toolchain does not include `cargo-fmt`.
- This turn improves API protocol determinism but does not implement external HTTP/gRPC transport, wrapper graph telemetry, live Ollama proof, or full observe-validation completion.
