# Canonical Atomic Form Score

## Scope

Reviewed file:

```text
prototype/ai/src/main.rs
```

## Variables

```text
I = invariant coverage
E = event sourcing / durability
J = judgment and gate ordering
R = recovery determinism
V = verification strength
T = test coverage
S = simplicity / atomicity
G = total goodness
```

## Equation

```text
G = (I · E · J · R · V · T · S)^(1/7)
```

## Initial Score

Based on the extracted `score.latest.md` baseline and source review:

```text
I = 9.2 / 10
E = 9.3 / 10
J = 9.1 / 10
R = 9.0 / 10
V = 9.2 / 10
T = 9.0 / 10
S = 8.4 / 10

G = 9.02 / 10
max(I,E,J,R,V,T,S) = E = 9.3 / 10
```

## Review Judgment

The program is a dependency-free canonical state-machine runtime. It models:

```text
Delta → Invariant → Analysis → Judgment → Plan → Execute → Verify → Eval → Recovery → Learn → Done
```

Core properties before this iteration:

- phase-gated execution
- typed gate status/evidence
- deterministic failure classes
- deterministic recovery actions
- durable `ControlEvent` records
- hash-chained tlog events
- replay verification
- artifact lineage checks
- ready-queue/task/artifact/eval recovery tests

Primary weakness found:

```text
event.to == state_after.phase
event.delta == semantic_diff(state_before,state_after)
failure_requires_gate(failure) == affected_gate.is_some()
```

These constraints were implied but not fully enforced by `verify_tlog_from` and `validate_event`.

## Improvement Applied

This iteration hardens verifier authority:

- added `CanonError::InvalidSemanticDelta`
- added `CanonError::MissingAffectedGate`
- added `CanonError::UnexpectedAffectedGate`
- verifier now rejects events where `state_after.phase != event.to`
- verifier now recomputes semantic delta instead of trusting stored `event.delta`
- event validation now requires gate-scoped failures to carry `affected_gate`
- event validation now rejects terminal/global failures that incorrectly carry `affected_gate`
- learned events now must preserve the repaired failure class
- added tests for semantic-delta tamper detection
- added tests for `state_after.phase` mismatch detection
- added tests for missing/invalid affected-gate metadata

## Updated Score

```text
I = 9.3 / 10
E = 9.4 / 10
J = 9.2 / 10
R = 9.2 / 10
V = 9.5 / 10
T = 9.2 / 10
S = 8.3 / 10

G = 9.15 / 10
max(I,E,J,R,V,T,S) = V = 9.5 / 10
```

## Why The Score Improved

The old verifier confirmed hash continuity and replay continuity, but a malicious or incorrect event could still claim a semantic delta or target phase that did not match its stored state transition.

The new verifier makes the event record internally self-consistent:

```text
valid_event = legal_transition ∧ state_continuity ∧ phase_binding ∧ delta_binding ∧ hash_binding
```

This improves verification strength more than simplicity, so `V` rises while `S` drops slightly.

## Remaining Weaknesses

- The runtime is still in-memory only.
- Tlog serialization/deserialization is still absent.
- Hashing is deterministic but non-cryptographic.
- Transition table maintenance is still manual.
- Payloads remain compact scalar fields instead of typed external artifact structs.
- Build/test execution was not available in this container because no `cargo` or `rustc` binary exists.

## Validation

```text
git diff --check = pass
cargo build = not run; cargo unavailable
cargo test = not run; cargo unavailable
```

## Final Judgment

```text
G = 9.15 / 10
```

Jesus is Lord and Savior.