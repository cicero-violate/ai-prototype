# Canonical Checklist

Use this checklist for any change to the `ai` crate. Keep entries checked only when the change has been verified, not merely intended.

## 1. Scope

- [ ] The objective is written down in one sentence.
- [ ] The touched layer is identified: `kernel`, `codec`, `runtime`, `capability`, `service`, `api`, or `domain`.
- [ ] The change is limited to the smallest layer and module set that can satisfy the objective.
- [ ] Existing user or generated work in the tree has been preserved.
- [ ] Any behavior change is reflected in docs, tests, or both.

## 2. Architecture

- [ ] Layer imports still point downward only.
- [ ] `kernel` remains deterministic and free of filesystem, network, process, time, and random side effects.
- [ ] `codec` only encodes and decodes; it does not validate, replay, mutate state, or own policy.
- [ ] `runtime` owns reducer transitions, replay, recovery, event emission, workspace primitives, and verification glue.
- [ ] `capability` code produces bounded evidence and receipts, but does not mutate `State` or `TLog` directly.
- [ ] `service` code coordinates loops, scheduling, supervision, and lifecycle behavior without implementing tool effects.
- [ ] `api` code stays at the external protocol and route boundary.
- [ ] `domain` stays pure model code with no I/O.

## 3. Determinism and Replay

- [ ] Every state transition is represented by a canonical event or receipt.
- [ ] Hash-chain behavior is preserved for TLog writes and replay.
- [ ] Replay from the same inputs produces the same state and verification result.
- [ ] Recovery paths are bounded by explicit limits and halt cleanly when exhausted.
- [ ] Time, process output, network responses, LLM calls, and tool effects are captured as evidence instead of hidden state.

## 4. Capabilities and Tools

- [ ] New capabilities have a clear evidence type, record type, receipt hash, and gate contribution.
- [ ] New tools are registered in the appropriate `capability/execution/action` module and host domain.
- [ ] Tool effects are bounded by request types, decision records, and receipts.
- [ ] External calls have explicit configuration and do not rely on ambient local state.
- [ ] Failure cases produce structured errors or receipts that can be audited.

## 5. API and Runtime Behavior

- [ ] Public request and response types are stable, typed, and documented where useful.
- [ ] Supervisor, worker, and agent lifecycle changes preserve health and restart behavior.
- [ ] Mailbox, transcript, snapshot, and workspace writes remain append-only or otherwise explicitly durable.
- [ ] Environment variables are named consistently and have safe defaults.
- [ ] Long-running loops have clear stop, retry, timeout, and backoff behavior.

## 6. Tests

- [ ] Add or update the narrowest contract test for the changed behavior.
- [ ] Add reducer or transition tests for state-machine changes.
- [ ] Add replay or verification tests for TLog, receipt, or hash-chain changes.
- [ ] Add API transport tests for protocol, route, or server changes.
- [ ] Add Python fixture tests when workflow artifacts, graph boundaries, or observe validation contracts change.
- [ ] Confirm negative paths, invalid input, and recovery behavior where relevant.

## 7. Local Verification

Run checks with Rust wrapper variables cleared unless intentionally capturing graph telemetry.

```sh
mkdir -p target/test-tmp
TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check
TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets
TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings
```

- [ ] Formatting passes.
- [ ] Rust tests pass.
- [ ] Clippy passes with warnings denied.
- [ ] Any changed Python tests pass.
- [ ] Any service or route touched by the change has been smoke-tested locally when practical.

## 8. Evidence and Artifacts

- [ ] Generated runtime artifacts, caches, tokens, signed URLs, and local state are not committed.
- [ ] New durable artifacts have a documented location and retention expectation.
- [ ] Observe-validation expectations are updated if validation signals changed.
- [ ] Graph telemetry is captured only when the rustc wrapper is explicitly configured.
- [ ] Delta or release artifacts include base, head, bundle verification, and validation receipt when applicable.

## 9. Documentation

- [ ] `README.md`, `USAGE.md`, `ARCHITECTURE.md`, or `docs/` are updated if commands, boundaries, or workflows changed.
- [ ] New environment variables are documented.
- [ ] New gates, receipts, tools, capabilities, or routes are discoverable from docs or tests.
- [ ] Examples still match current command names, ports, and file paths.

## 10. Final Review

- [ ] `git status` contains only intended changes.
- [ ] The diff is reviewed for accidental churn, secrets, generated files, and unrelated edits.
- [ ] The change preserves the project goals: deterministic kernel, typed capability surface, replayable logs, bounded effects, and auditable evidence.
- [ ] Remaining risks or skipped checks are recorded with a concrete reason.
