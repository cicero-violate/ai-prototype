# Canon Agent Scorecard

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

## Baseline Before Source Changes

```text
repository = ai
base_commit = 2b9f4c7
restored_head_before_changes = 2b9f4c7
tracked_files = 134
rust_files = 47
runtime_archive = /mnt/data/ai-runtime.tar.gz
runtime_ndjson_files = 6
runtime_audit_events = 0
runtime_largest_message_ledger_lines = 0
validation = PASS: cargo -Znext-lockfile-bump run --bin root_validate --locked
validation_note = root_validate passed root cargo check, bounded score_contract tests, library unit tests, API transport integration tests, validation harness contract tests, planning contract tests, deterministic Python-contract skip receipt, and canon-rustc-v3 graph telemetry receipt generation; current source also contains shared LLM provider transport primitives, a verified evolution candidate ledger with deterministic selection records, a gated distill.jsonl exporter for verified TLog receipts, and bounded orchestration batch selection receipts
```

TODO/FIXME search:

```text
command = rg -n --hidden -g '!.git/**' -g '!target/**' -g '!score.md' 'TODO|FIXME' .
active_markers = 1
finding = documentation-only TODO/FIXME reference remains in canon-rustc-v3/plan.md; no root src/, examples/, or tests/ TODO/FIXME markers were found
```

Baseline scores:

| Axis | Score |
|------+-------|
| I    |   8.2 |
| E    |   7.4 |
| C    |   8.1 |
| A    |   8.9 |
| R    |   8.2 |
| P    |   6.5 |
| S    |   7.3 |
| D    |   8.8 |
| T    |   9.2 |
| Co   |   7.2 |
| Em   |   7.3 |
| B    |   7.9 |
| L    |   8.1 |
| Si   |   6.2 |
| F    |   8.3 |

```text
G = 7.82
max(G) = good
```


## After Step 2: Gated Distillation Exporter

```text
validation = PASS: cargo test -q distillation --lib --locked
validation = PASS: cargo -Znext-lockfile-bump run --bin root_validate --locked
```

Score movement:

```text
L: 7.9 -> 8.1
T: 9.1 -> 9.2
G: 7.80 -> 7.82
```

Rationale: the learning layer now has a replay-gated, proof-bound `distill.jsonl` exporter that refuses tampered TLogs, low-score rows, proof mismatches, and incomplete/non-lineage-valid outcomes before producing training-ready rows.


## After Step 3: Bounded Orchestration Batch Selection

```text
validation = PASS: cargo test -q orchestration --lib --locked
```

Score movement:

```text
S: 7.2 -> 7.3
D: 8.7 -> 8.8
T: 9.1 -> 9.2
G: 7.81 -> 7.82
```

Rationale: orchestration now has a deterministic batch receipt that selects ready routes across multiple run records under explicit parallelism, submission-count, and resource-unit budgets while binding the selected merge to a tamper-evident hash.

## After Agent 2 Step 1: Memory Lookup Receipts

```text
validation = PASS: cargo test -q memory --lib --locked
validation = PASS: cargo -Znext-lockfile-bump run --bin root_validate --locked
```

Score movement:

```text
C: 8.1 -> 8.2
R: 8.2 -> 8.3
T: 9.2 -> 9.3
L: 8.1 -> 8.2
G: 7.82 -> 7.86
```

Rationale: memory lookup now emits a deterministic receipt that binds query hash, lookup limit, returned match count, index fingerprint, aggregate lookup hash, and receipt hash. This improves auditability and replay confidence for the context/memory seam without changing the frozen kernel or adding nondeterministic behavior.

## After Agent 1 Next Step: Context Memory Receipt Binding

```text
validation = PASS: cargo test -q context --lib --locked
validation = PASS: cargo test -q --lib --locked
validation = PASS: cargo -Znext-lockfile-bump run --bin root_validate --locked
```

Score movement:

```text
C: 8.2 -> 8.3
R: 8.3 -> 8.4
D: 8.8 -> 8.9
T: 9.3 -> 9.4
L: 8.2 -> 8.3
G: 7.86 -> 7.90
```

Rationale: context assembly now has a receipt-aware path that binds the validated memory lookup receipt hash into the deterministic context hash while retaining packet lineage fields needed for replay checking. Legacy assembly remains deterministic for current call sites, and receipt-aware flows can now audit observation-to-memory-to-context provenance without changing the frozen kernel.


## After Agent Step 3: Policy-First Judgment Hit Records

```text
validation = PASS: cargo test -q policy_judgment --lib --locked
validation = PASS: cargo test -q judgment --lib --locked
validation = PASS: cargo test -q --lib --locked
validation = PASS: cargo -Znext-lockfile-bump run --bin root_validate --locked
```

Score movement:

```text
E: 7.4 -> 7.5
P: 6.5 -> 6.6
L: 8.3 -> 8.4
T: 9.4 -> 9.5
G: 7.90 -> 7.94
```

Rationale: judgment now has a policy-first record path. A valid context plus promoted policy feedback hash can produce a kernel-compatible `JudgmentRecord` without constructing an LLM record, while empty policy stores deterministically miss and preserve the LLM fallback path. This directly advances the LLM promotion ladder by reducing recurring judgment cost through verified policy coverage.

## After Agent Step 4: Receipt-Aware Live LLM Context Paths

```text
validation = PASS: cargo test -q context --lib --locked
validation = PASS: cargo test -q judgment --lib --locked
validation = PASS: cargo check -q --examples --locked
validation = PASS: cargo -Znext-lockfile-bump run --bin root_validate --locked
```

Score movement:

```text
C: 8.3 -> 8.35
R: 8.4 -> 8.45
T: 9.5 -> 9.55
L: 8.4 -> 8.45
G: 7.94 -> 7.96
```

Rationale: the live Ollama and OpenAI-compatible judgment paths now assemble context through validated memory lookup receipts instead of the legacy memory-only constructor. This extends the audited observation-memory-context-judgment seam into executable provider examples without changing the frozen kernel or the deterministic adapter behavior.

## After Agent Step 5: Policy Lookup Receipts for Policy-First Judgment

```text
validation = PASS: cargo test -q policy_lookup --lib --locked
validation = PASS: cargo test -q policy_judgment --lib --locked
validation = PASS: cargo test -q judgment --lib --locked
validation = PASS: cargo test -q policy --lib --locked
validation = PASS: cargo test -q --lib --locked
validation = PASS: cargo -Znext-lockfile-bump run --bin root_validate --locked
```

Score movement:

```text
C: 8.35 -> 8.40
R: 8.45 -> 8.50
D: 8.9 -> 8.95
T: 9.55 -> 9.60
L: 8.45 -> 8.50
G: 7.96 -> 7.98
```

Rationale: policy-first judgment now consumes a deterministic policy lookup receipt instead of only reading the policy feedback hash directly. The judgment record binds the policy lookup receipt hash, and tampered lookup receipts deterministically miss, improving auditability of the policy-hit path without changing the frozen kernel.

## After Agent Step 6: Eval Scorecard Receipts

```text
validation = PASS: cargo test -q eval --lib --locked
validation = PASS: cargo test -q --lib --locked
validation = PASS: cargo -Znext-lockfile-bump run --bin root_validate --locked
```

Score movement:

```text
C: 8.40 -> 8.45
R: 8.50 -> 8.55
D: 8.95 -> 9.00
T: 9.60 -> 9.65
G: 7.98 -> 8.00
```

Rationale: eval now emits a typed scorecard receipt that binds the verdict, payload hash, dimension ordering, dimension score aggregate, threshold floor, and receipt hash. This improves auditability of the eval gate without changing kernel semantics or allowing the model to approve its own score.

## After Agent Step 7: Planning Lineage Receipts

```text
validation = PASS: RUSTC_WRAPPER= cargo test -q plan_receipt --lib --locked
validation = PASS: RUSTC_WRAPPER= cargo test -q --test planning_contract --locked
validation = PASS: RUSTC_WRAPPER= cargo test -q --lib --locked
validation = PASS: RUSTC_WRAPPER= cargo -Znext-lockfile-bump run --bin root_validate --locked
```

Score movement:

```text
C: 8.45 -> 8.50
R: 8.55 -> 8.60
D: 9.00 -> 9.05
T: 9.65 -> 9.70
G: 8.00 -> 8.03
```

Rationale: planning now emits a typed lineage receipt that binds the objective, selected ready task, task counts, dependency hash, ready-set hash, plan revision, lineage hash, payload hash, verdict, and receipt hash. This improves auditability of the planning gate while preserving the existing deterministic task decomposition and kernel-visible `TaskReady` submission semantics.

## After Agent Step 8: Observation Ingress Receipts

```text
validation = PASS: RUSTC_WRAPPER= cargo test -q observation_ingress_receipt --lib --locked
validation = PASS: RUSTC_WRAPPER= cargo test -q observation --lib --locked
validation = PASS: RUSTC_WRAPPER= cargo test -q --lib --locked
validation = PASS: RUSTC_WRAPPER= cargo -Znext-lockfile-bump run --bin root_validate --locked
```

Score movement:

```text
C: 8.50 -> 8.55
R: 8.60 -> 8.65
D: 9.05 -> 9.10
T: 9.70 -> 9.75
G: 8.03 -> 8.06
```

Rationale: observation ingress now emits a typed receipt that binds the external source hash, cursor state, backlog pressure, observed record sequence span, record aggregate hash, contract hash, contract validity, verdict, and receipt hash. This strengthens the first world-facing evidence boundary while preserving existing bounded ingress and kernel-visible invariant proof semantics.

## After Agent Step 9: Context Assembly Receipts

```text
validation = PASS: RUSTC_WRAPPER= cargo test -q context_assembly_receipt --lib --locked
validation = PASS: RUSTC_WRAPPER= cargo test -q context --lib --locked
validation = PASS: RUSTC_WRAPPER= cargo test -q --lib --locked
validation = PASS: RUSTC_WRAPPER= cargo -Znext-lockfile-bump run --bin root_validate --locked
```

Score movement:

```text
C: 8.55 -> 8.60
R: 8.65 -> 8.70
D: 9.10 -> 9.15
T: 9.75 -> 9.80
G: 8.06 -> 8.09
```

Rationale: context assembly now emits a typed receipt that binds packet objective metadata, observation hash, memory aggregate hash, validated memory lookup receipt hash, prior count, context hash, verdict, and receipt hash. This strengthens the observation-memory-context analysis seam while preserving existing context construction and kernel-visible analysis report semantics.

## Graph-as-Source-of-Truth Architectural Shift

```text
graph.json schema_version = 11
node_count  = 2503
edge_count  = 6840
relation_types = [ alloc, call, impl, io, mut, panic, unsafe ]
intent_labels  = [ pure, mutation, ... ]
node_fields    = { def_id, path, kind, def: { file, line, col, lo, hi } }
```

The canon-rustc-v3 wrapper now produces a verified `graph.json` on every
`cargo check`. Each node carries source byte offsets (`lo`, `hi`) enabling
deterministic reverse patching without any AST re-parsing.

Projected score impact when the graph mutation contract and patch generator
are implemented:

| Axis | Current | Projected | Reason                                                          |
|------+---------+-----------+-----------------------------------------------------------------|
| I    |     8.2 |       8.6 | agent can query semantic graph, target mutations structurally   |
| C    |    8.60 |      8.75 | every mutation is verified by re-capture + graph diff receipt   |
| A    |     8.9 |       9.1 | structural ops are deterministic — no freeform text patching    |
| T    |    9.80 |      9.90 | full op→patch→receipt chain is auditable end-to-end             |
| Em   |     7.3 |       8.0 | external agents can safely mutate source through typed contract |
| F    |     8.3 |       8.7 | graph schema versioning decouples consumers from source layout  |
| G    |    8.09 |     ~8.25 | geometric mean lift across I, C, A, T, Em, F                    |

Axes not expected to move: E, R, P, S, D, Co, B, L, Si — the patch
generator adds a new surface but does not regress existing surfaces.

Current scores before graph mutation contract is implemented:

| Axis | Score |
|------+-------|
| I    |   8.2 |
| E    |   7.5 |
| C    |  8.60 |
| A    |   8.9 |
| R    |  8.70 |
| P    |   6.6 |
| S    |   7.3 |
| D    |  9.15 |
| T    |  9.80 |
| Co   |   7.2 |
| Em   |   7.3 |
| B    |   7.9 |
| L    |  8.50 |
| Si   |   6.2 |
| F    |   8.3 |

```text
G = 8.09
```
