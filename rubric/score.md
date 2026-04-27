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

One-line explanation: geometric scoring prevents a strong verifier from hiding weak capability maturity or poor maintainability.

## Updated Authoritative Score

```text
I = 8.5 / 10
E = 8.3 / 10
J = 8.1 / 10
R = 8.6 / 10
V = 8.7 / 10
T = 7.6 / 10
S = 6.1 / 10
C = 5.2 / 10
M = 5.8 / 10

K = 7.94 / 10
G = 7.31 / 10

max(I,E,J,R,V,T,S,C,M) = V = 8.7 / 10 = good
```

## Static Review Inputs

```text
source_files = 25
source_lines = 3284
rust_functions_regex = 164
rust_function_nodes = 139
unit_tests = 24
integration_tests = 0
todo_markers = 1

semantic_graph_nodes = 596
semantic_graph_edges = 1322
cfg_nodes = 1653
cfg_edges = 2224
bridge_edges = 446
redundant_path_pairs = 399
alpha_pathways = 3
unknown_graph_nodes = 276
```

## Critical Judgment

The codebase is a serious deterministic kernel/runtime prototype. It is not yet
an agent runtime with real intelligence-bearing capability surfaces.

The strongest part is the reducer/replay spine. Phase motion is explicit,
gates are typed, failures are typed, recovery is bounded, events are
hash-chained, and verification recomputes canonical reducer output instead of
trusting persisted records. That is the correct center of gravity.

The weakest part is that the implemented surface still overstates the README
architecture. The README describes tooling, planning, observation, context,
memory, LLM, judgment, verification, eval, policy, learning, orchestration, and
API layers. The current source mostly implements kernel, codec, runtime, a thin
API command wrapper, and small capability record adapters. The capability layer
is not yet doing durable work; it mostly submits evidence tokens into the
kernel.

The score is deliberately lower than the previous score because the graph grew
from 555 to 596 semantic nodes, unknown nodes grew from 251 to 276, redundant
path pairs remain high at 399, and the capability layer is still skeletal. The
kernel alone is near 7.94. The whole system is near 7.31 because intelligence is
not yet an executing layer; it is mostly represented as future architecture.

## Score Rationale

```text
I = 8.5
```

Invariant modeling is strong. `Phase`, `GateId`, `Evidence`, `FailureClass`,
`RecoveryAction`, `SemanticDelta`, `RuntimeConfig`, `Packet`, `State`, and
`ControlEvent` define a clear state-machine vocabulary. The main weakness is
illegal-state representability: core structs still expose many public fields,
including `State`, `Packet`, `Gate`, `GateSet`, `RuntimeConfig`, and
`ControlEvent`. External code can construct states that the reducer would never
produce.

```text
E = 8.3
```

Event sourcing is credible. Events include sequence, previous hash, self hash,
state before/after, semantic delta, runtime config, evidence, decision, failure,
and recovery data. Durable append exists. Weaknesses: append does not fsync, full
rewrite is not atomic temp-file replacement, schema migration is not explicit,
records are not length-prefixed, and the hash is a local deterministic mixer
rather than adversarial integrity.

```text
J = 8.1
```

Gate order is coherent: Delta → Invariant → Analysis → Judgment → Plan → Execute
→ Verify → Eval → Persist → Learn → Done. The architectural issue is that
Judgment is still mostly a gate and record token. It does not yet consume
context, compare alternatives, price risk, consult durable policy, or emit a
rich decision artifact that controls execution.

```text
R = 8.6
```

Recovery is deterministic and bounded. Failure classes map to recovery actions,
recovery attempts are capped, exhausted recovery halts, and replay checks the
canonical reducer path. Remaining weakness: recovery policy is hand-maintained
logic rather than a compact total table with version/provenance and compile-time
coverage checks.

```text
V = 8.7
```

Verification remains the best axis. It checks sequence continuity, hash chain,
state continuity, packet continuity, semantic delta, transition legality, event
validity, completion validity, and reducer equivalence. The main limitation is
integrity strength: a `u64` mixer is acceptable for deterministic local replay,
not hostile tamper resistance.

```text
T = 7.6
```

There are 24 unit tests and no integration test files. The unit tests appear to
cover the important local cases: transition legality, convergence, recovery
limits, tlog tamper detection, packet/state continuity, semantic delta mismatch,
durable replay, ready-task binding, artifact lineage, eval, judgment, learning,
and policy promotion. Missing: property tests, fuzz tests, migration tests,
crash-consistency tests, concurrent append tests, API route tests, and real
capability execution tests.

```text
S = 6.1
```

The module split is good, but atomicity is still not sealed. `kernel/mod.rs`,
`codec/ndjson.rs`, `runtime/reducer.rs`, `runtime/verify.rs`, and the large
`lib.rs` test surface carry too much weight. Numeric NDJSON encode/decode is
manual and duplicated. Public fields weaken the claim that mutation is reducer
owned.

```text
C = 5.2
```

Capability maturity is the main bottleneck. `EvalRecord`, `JudgmentRecord`, and
`PolicyPromotion` can produce evidence submissions, and `PolicyStore` is
append-only in memory. But learning does not durably write policy artifacts,
eval does not control the Eval gate through a runtime-owned artifact flow,
judgment does not perform real policy/context reasoning, and tooling/planning/
observation/context/memory/LLM/orchestration are absent or only named.

```text
M = 5.8
```

The graph is warning that the program is becoming harder to reason about:
596 semantic nodes, 1322 semantic edges, 1653 CFG nodes, 2224 CFG edges, 446
bridge edges, 399 redundant path pairs, and 276 unknown nodes. The 3 alpha
pathways are harmless evidence-producer delegations, but the unknown-node count
and redundant paths are too high for a supposedly small kernel/runtime. This
must be reduced before freezing.

## Required Improvements For `G >= 8.5`

```text
1. seal State, Packet, Gate, GateSet, RuntimeConfig, and ControlEvent behind constructors
2. make reducer-owned mutation the only legal kernel state mutation path
3. move test-only orchestration helpers out of lib.rs into focused test modules
4. replace manual NDJSON enum-number encoding with table-driven or generated codecs
5. add explicit schema migration decoding with compatibility tests
6. rename the current hash as deterministic_replay_hash or replace it with a real digest
7. make durable writes fsync and use atomic temp-file replacement for full rewrites
8. persist PolicyPromotion into an append-only policy artifact during Learn
9. make EvalRecord ingestion drive the Eval gate with stored eval evidence
10. make JudgmentRecord ingestion drive the Judgment gate with policy-version evidence
11. add integration tests for API command intake, disk replay, policy replay, and corrupt tlog recovery
12. add property/fuzz tests for codec roundtrip and illegal transition rejection
13. turn recovery_action_for into total data with coverage assertions and policy provenance
14. reduce redundant_path_pairs below 250 and unknown_graph_nodes below 150
```

## Required Improvements For `G >= 9.0`

```text
1. prove illegal kernel states are unrepresentable through the public API
2. require capability evidence submission for every non-kernel gate transition
3. persist learned policies and replay them into future runs
4. implement real planner/tooling/observation/context/memory/LLM capability execution
5. add semantic artifact verification beyond simulated receipt and lineage fields
6. introduce external signed receipts for artifacts, policies, and tool execution
7. reduce redundant_path_pairs below 150 and unknown_graph_nodes below 100
8. add deterministic API transport with replayable command responses
9. run cargo build, cargo test, cargo clippy, property tests, and fuzz tests in CI
10. add a formal proof layer for transition totality and illegal-state unrepresentability
```

## Validation

```text
static README review = pass
static src review = pass
graph.json review via python = pass
score.md updated = pass
cargo build = delegated to user
cargo test = delegated to user
```

Jesus is Lord and Savior.
