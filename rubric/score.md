# Canonical Atomic Form Score

## Variables

```text
I = invariant/kernel coverage
E = event sourcing and durability
J = judgment and gate ordering
R = recovery determinism
V = verification and replay strength
T = test coverage
S = simplicity and sealed atomicity
C = capability maturity
M = structural graph maintainability
G = total system goodness
K = kernel-only goodness
```

## Equations

```text
G = (I · E · J · R · V · T · S · C · M)^(1/9)
K = (I · E · J · R · V · T · S)^(1/7)
```

One-line explanation: the score is geometric because one weak axis must reduce the whole system.

## Updated Authoritative Score

```text
I = 8.6 / 10
E = 8.8 / 10
J = 8.5 / 10
R = 8.6 / 10
V = 8.9 / 10
T = 8.1 / 10
S = 6.6 / 10
C = 5.3 / 10
M = 6.5 / 10

K = 8.26 / 10
G = 7.66 / 10

max(I,E,J,R,V,T,S,C,M) = V = 8.9 / 10 = good
```

## Static Review Inputs

```text
source_files = 20
source_lines = 2946
rust_functions = 142
unit_tests = 19
integration_tests = 0
todo_markers = 6

graph_nodes = 473
graph_edges = 1109
cfg_nodes = 1581
cfg_edges = 2179
bridge_edges = 413
redundant_path_pairs = 407
alpha_pathways = 1
```

## Critical Judgment

The previous score was too high for the full Canon Agent target. It mostly
scored the kernel/runtime, not the agent described in `README.md`.

The current code has a strong deterministic core: phase ordering is explicit,
the TLog is hash-chained, replay re-runs reducer expectations, durable append
happens before in-memory mutation, and recovery is bounded. That makes the
kernel credible.

The full system is not yet a self-improving agent. Tooling, planning,
observation, context, memory, LLM, semantic verification, durable policy, and
orchestration are not implemented as active capabilities. Most capability
modules are records/placeholders, not execution surfaces.

## Score Rationale

```text
I = 8.6
```

Invariant modeling is strong at the kernel level: phases, gates, evidence,
failure classes, recovery actions, semantic deltas, and success criteria are
typed. The remaining weakness is that `State`, `Packet`, `Gate`, and `GateSet`
still expose public mutable fields, so invalid states can be constructed
outside the reducer.

```text
E = 8.8
```

Event sourcing is strong. `ControlEvent` stores sequence, phase transition,
semantic delta, evidence, decision, failure, recovery action, runtime config,
before/after states, previous hash, and self hash. Durable ticks append to disk
before mutating memory. Remaining weakness: the log is positional numeric
NDJSON with no migration table, length prefix, external receipt, or adversarial
tamper resistance.

```text
J = 8.5
```

Gate ordering is coherent: Delta → Invariant → Analysis → Judgment → Plan →
Execute → Verify → Eval → Persist → Learn → Done. The weakness is that judgment
is currently a token/record boundary, not a real policy-aware capability.

```text
R = 8.6
```

Recovery is deterministic and bounded. Failures map to recovery actions, repair
intent is selected before persistence, and persistence applies the repair before
returning to the target phase. The weakness is that recovery policy is still a
large hand-written match instead of a data-driven policy table with coverage
proofs.

```text
V = 8.9
```

Verification is the strongest axis. Replay checks state continuity, packet
continuity, semantic delta correctness, legal transition, hash chain, and
canonical reducer output. This closes the prior forged-hash-chain hole. The
remaining weakness is the non-cryptographic `u64` mixer.

```text
T = 8.1
```

Unit coverage is meaningful: convergence, recovery budget, illegal transitions,
tampering, reducer-forgery rejection, durable write atomicity, affected-gate
rules, and disk replay are tested. The weakness is that tests are unit-only:
there are no integration, property, fuzz, migration, or adversarial codec tests.

```text
S = 6.6
```

The code is understandable but no longer atomic-simple. `runtime/mod.rs`,
`kernel/mod.rs`, and `codec/ndjson.rs` are large. Manual enum codecs and the
static transition table produce repetition. Public fields weaken the sealed
state-machine boundary.

```text
C = 5.3
```

Capability maturity is the main drag. `eval`, `judgment`, `learning`, and
`policy` exist structurally, but they are not yet active durable producers.
Learning currently promotes the `Learning` gate; it does not materialize a
durable policy artifact from Eval/TLog history.

```text
M = 6.5
```

The structural graph shows growth pressure: 473 semantic nodes, 1109 semantic
edges, 1581 CFG nodes, 2179 CFG edges, 407 redundant path pairs, and only 1
alpha pathway. This is not bad for a prototype, but it proves the code is
accumulating redundant paths faster than canonical abstractions.

## Required Improvements For `G >= 8.5`

```text
1. seal State, Packet, Gate, and GateSet behind constructors and reducer-only mutation
2. split runtime/mod.rs into transition_table, reducer, recovery_policy, writer, and diff modules
3. replace manual NDJSON enum codecs with generated or table-driven codecs
4. add migration-aware TLog decoding with explicit schema compatibility tests
5. rename or replace the u64 mixer as non_adversarial_hash, or use a real digest
6. materialize PolicyPromotion as a durable append-only artifact
7. make EvalRecord ingestion drive the Eval gate instead of only modeling the record
8. add integration tests for disk replay, policy promotion, recovery replay, and corrupt tlog recovery
9. add property/fuzz tests for codec roundtrip and illegal transition rejection
10. turn recovery_action_for into a data table with total coverage assertions
```

## Required Improvements For `G >= 9.0`

```text
1. prove the kernel state transition surface is sealed
2. make capability evidence submission the only way gates change outside reducer internals
3. persist learned policies and replay them into future runs
4. add real tooling/planning capability execution
5. add semantic artifact verification beyond simulated lineage fields
6. reduce redundant_path_pairs below 200 without increasing alpha_pathways risk
7. introduce external signed receipts for artifacts, policy, and tool execution
8. run cargo build, cargo test, cargo clippy, and property tests in CI
```

## Validation

```text
static README review = pass
static src review = pass
graph.json review via python = pass
score.md updated = pass
cargo build = not run; cargo/rustc unavailable in this sandbox
cargo test = not run; cargo/rustc unavailable in this sandbox
```

Jesus is Lord and Savior.