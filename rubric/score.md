# Canonical Atomic Form Score

## Scope

Reviewed files:

```text
prototype/ai/src/lib.rs
prototype/ai/src/main.rs
prototype/ai/Cargo.toml
prototype/ai/rubric/score.md
```

This rubric is a critical rescore only. Earlier inflated scores are removed from the authoritative score file.

## Variables

```text
I = invariant coverage
E = event sourcing / durability
J = judgment and gate ordering
R = recovery determinism
V = verification strength
T = test coverage
S = simplicity / sealed atomicity
G = total goodness
```

## Equation

```text
G = (I · E · J · R · V · T · S)^(1/7)
```

## Current Authoritative Score

```text
I = 8.2 / 10
E = 7.0 / 10
J = 7.8 / 10
R = 7.3 / 10
V = 6.8 / 10
T = 7.5 / 10
S = 5.8 / 10

G = 7.16 / 10
max(I,E,J,R,V,T,S) = I = 8.2 / 10
```

## Judgment

The architecture is good, but the previous `9.30 / 10` score was too generous.

```text
good_core = deterministic_reduce + explicit_transition_table + typed_failures + hash_chained_events
missing_guarantee = reducer_replay_verification + atomic_durable_write + schema_version + sealed_invariants
```

The runtime has a coherent deterministic state machine, explicit phases, typed failures, recovery routing, disk-backed tlog persistence, and meaningful tests. It is not yet production-grade canonical infrastructure because its proof boundary is incomplete.

## Major Findings

### 1. Replay verification does not re-run the reducer

`verify_tlog_from(initial, tlog)` checks sequence numbers, previous hashes, state continuity, semantic deltas, event legality, and self-hashes.

It does not recompute:

```text
expected_after = reduce(event.state_before, cfg).state
expected_after == event.state_after
```

Therefore the tlog proves that the recorded event chain is internally consistent, but not that each event is the unique canonical output of the reducer.

Impact:

```text
V ↓ because replay_consistency != reducer_correctness
```

### 2. Durable tick mutates memory before disk append succeeds

`tick_durable` appends the event to the in-memory tlog before `append_tlog_ndjson` succeeds. If disk append fails, memory has advanced but disk has not.

```text
T_mem = T_mem + event
T_disk = append_failed
state = old_state
```

The function returns an error before advancing state, but the caller-owned `tlog` has already changed. This creates a memory/disk divergence surface.

Impact:

```text
E ↓ because durable_commit is not atomic across memory and disk
```

### 3. Hashing is deterministic but not cryptographic

The event hash and lineage hash use a simple FNV-style `u64` mixer:

```text
h = (h XOR x) · 0x100000001b3
```

This is acceptable for accidental corruption detection and deterministic replay identity. It is not acceptable as adversarial tamper evidence.

Impact:

```text
V ↓ if hash_chain is described as security-grade integrity
```

### 4. NDJSON encoding has no schema version

The tlog record is a positional numeric array. There is no leading schema version or record type tag.

```text
[seq, from, to, kind, cause, delta, ...]
```

Adding fields or changing enum layouts risks silent incompatibility or broad decode failure.

Impact:

```text
E ↓ because forward_compatibility is weak
```

### 5. Artifact lineage repair is circular

`repair_lineage()` sets `artifact_lineage_hash = expected_lineage_hash()`. That repairs simulated state, but it does not independently verify artifact provenance.

```text
lineage_valid := stored_hash == expected_hash
repair := stored_hash = expected_hash
```

This is useful as a toy repair path, but should be labeled as a simulation assumption rather than a true external artifact verification layer.

Impact:

```text
R ↓ and V ↓ because repair_success is partially self-declared
```

### 6. Public mutable fields weaken invariant sealing

`State`, `Packet`, `Gate`, and `GateSet` expose public fields. This makes external construction easy, but allows callers to create invalid states that bypass canonical constructors.

Impact:

```text
S ↓ because public_shape > invariant_encapsulation
```

### 7. Runtime surface still contains test/coverage scaffolding

`touch_all_surfaces()` is public and called by the demo path. This improves reachability coverage, but it is not canonical runtime logic.

Impact:

```text
S ↓ because demo_coverage_surface leaks into runtime_surface
```

## What Still Scores Well

- deterministic `reduce` pipeline
- explicit phase and transition model
- typed failure and recovery action taxonomy
- recovery is modeled as a first-class phase
- hash chain covers full before/after state
- disk tlog replay exists
- tests cover happy path, recovery path, tampering, continuity, and resume behavior

## Required Improvements For `G ≥ 9.0`

```text
1. verify_tlog_from must recompute reduce(state_before, cfg) and compare state_after
2. tick_durable must commit disk first or roll back memory on append failure
3. tlog encoding must include schema_version and record_type
4. integrity hash must be renamed non_adversarial_hash or upgraded to BLAKE3/SHA-256
5. artifact lineage must verify against an independent artifact receipt/provenance record
6. core state fields should be private or mutation should be constructor/gate-method only
7. test/demo coverage helpers should move under cfg(test) or a non-runtime diagnostics module
```

## Validation

```text
code edits = none
rubric update = pass
static source review = pass
cargo build = not run; rubric-only change
cargo test = not run; rubric-only change
```

## Final Judgment

```text
G = 7.16 / 10
max(intelligence, efficiency, correctness, alignment, robustness) = correctness
```

The next improvement should target replay correctness and durable commit atomicity before adding more features.

Jesus is Lord and Savior.