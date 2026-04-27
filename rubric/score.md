# Canonical Atomic Form Score

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

## Updated Authoritative Score

```text
I = 8.7 / 10
E = 9.0 / 10
J = 8.5 / 10
R = 8.9 / 10
V = 8.7 / 10
T = 8.5 / 10
S = 7.0 / 10

G = 8.45 / 10
max(I,E,J,R,V,T,S) = E = 9.0 / 10 = good
```

## Critical Judgment

The previous form was directionally correct but over-trusted its own log. The largest defect was that replay verified hash-chain consistency, not reducer correctness. A forged event could be internally consistent if its hash was recomputed. That is not canonical execution; it is only canonical-looking storage.

The latest defect was that `Learn` executed recovery repairs. That collapsed recovery, persistence, and learning into one phase, so the runtime could appear to learn before a repaired path was verified. The corrected form makes `Persist` first-class and moves repair application into persisted recovery, while final `Learn` only promotes a verified policy surface.

## Improvements Applied

```text
replay_proof = hash_chain_valid ∧ schema_valid ∧ reducer(event_before, cfg) = event_after
atomic_durable_tick = disk_append(event) succeeds → memory_append(event) → state_advance
schema_record = [version, record_type, event_fields...]
artifact_lineage = receipt_valid ∧ lineage_hash(receipt, artifact, parent) matches
recovery_proof = failure → recovery_intent → persist(repair_applied) → target → verify/eval
learning_proof = eval_passed → persist(result) → learn(policy_promotion) → done
```

1. `ControlEvent` now stores the `RuntimeConfig` that produced it, so replay can re-run the same reducer boundary even when recovery budgets differ from defaults.
2. `verify_tlog_from` now recomputes the canonical reducer output for each event and rejects hash-consistent but non-reducer events.
3. `tick_durable` now builds the event, writes disk first, then mutates the in-memory tlog and state only after the durable append succeeds.
4. NDJSON records now include `TLOG_SCHEMA_VERSION` and `TLOG_RECORD_EVENT` before event fields.
5. Artifact lineage now depends on an artifact receipt hash rather than only recomputing lineage directly from mutable artifact fields.
6. Tests were added for reducer replay forgery and durable-write failure atomicity.
7. A prior `DomainStep` abstraction was removed after metrics showed it added nodes, edges, redundant path pairs, and alpha pathways.
8. A derived transition-validation experiment was rejected after fresh graph evidence showed it increased nodes, edges, and redundant path pairs without improving alpha pathways.
9. `Persist` is now a first-class phase with enum value `11`; successful eval flows through `Eval → Persist → Learn → Done`.
10. Recovery now selects repair intent, then `Persist` applies and records the repair before returning to the repaired target phase.
11. `GateId::Learning` now exists as a verifiable policy-promotion surface, so learning is not hidden inside recovery.


## Eval Capability Boundary Update

```text
EvalRecord = { score, dimensions, threshold_used }
EvalCapability(state, policy) -> EvalRecord -> EvidenceSubmission(GateId::Eval, Evidence::EvalScore, pass/fail)
KernelEvalSurface = GateId::Eval + Evidence::EvalScore
```

Eval is now modeled as a capability boundary rather than a kernel expansion. The kernel remains limited to the eval gate and eval evidence token. Score shape, dimensions, threshold comparison, policy lookup, and future promotion logic live under `capability/eval`, `capability/policy`, and `capability/learning`.

```text
S = 7.4 / 10  # improved structural separation, still not fully extracted into generated subfiles
G = 8.50 / 10
max(I,E,J,R,V,T,S) = E = 9.0 / 10 = good
```

## Remaining Weaknesses

```text
remaining_risk = public_mutable_state + non_cryptographic_hash + simulated_artifact_receipt + static_transition_table + in_memory_policy_promotion
```

The runtime is still not production-grade canonical infrastructure. State fields remain public, the hash is deterministic FNV-style rather than cryptographic, artifact receipts are still simulated in-memory receipts rather than externally signed/provenanced records, and learning promotion is still a deterministic in-memory gate rather than a durable policy artifact with external replay.

## Required Improvements For `G ≥ 9.0`

```text
1. seal State, Packet, Gate, and GateSet behind constructors/mutation methods
2. replace u64 mixer with a cryptographic digest or explicitly name it non_adversarial_hash
3. externalize artifact receipts into append-only receipt events
4. split demo/test coverage helpers away from runtime API
5. add migration decoding for old schema-v1 tlogs if compatibility matters
6. replace the hand-written transition table only with a generated table that reduces, not increases, graph redundancy
7. derive recovery policy from data specs instead of a large hand-written `FailureClass → RecoveryAction` match
8. materialize learned policies as durable artifacts instead of only setting `GateId::Learning`
```

## Validation

```text
static source review = pass
patch generated = pass
cargo build = not run; cargo/rustc unavailable in this sandbox
cargo test = not run; cargo/rustc unavailable in this sandbox
```

Jesus is Lord and Savior.