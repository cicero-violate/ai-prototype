# Canon Agent Score

Current date: 2026-05-10.

## Current Progress

- P0 validation baseline: complete.
- P1 validation evidence reporting: complete.
- P2 agent loop reliability: complete.
- P3 runtime and receipt correctness: complete for current scope.
- P4 graph source-of-truth integration: mostly complete for deterministic fixture/report evidence; next focus is agent-driven graph editing using Python graph analysis.
- P5 domain intelligence layer: active. Reconnaissance found `src/domain` wired
  into `src/lib.rs` and Rust files present for `bridge.rs`, `contracts.rs`,
  `identity.rs`, `risk.rs`, and `scoring.rs`; several planned subdomain Rust
  files and fixture tests remain incomplete or absent.

## Graph Analysis Evidence

Python inspection of `state/rustc/ai/graph.json`:

```text
schema_version = 16
crate_name     = ai
graph_hash     = ab2202a8d8ec371b0c462aecc41e28d059920f53e2179dd40ebb6ebb3127fc33
nodes          = 4473
edges          = 31082
intents        = 2976
node_kinds     = fn 2976, impl 1283, struct 159, enum 53, trait 1, ty_alias 1
edge_relations = call 17393, phase 5925, similar 3174, mut 2926, use 1014, panic 212, unsafe 186, alloc 124, io 74, impl 35, provider 19
domain_nodes   = 0
graph_nodes    = graph_mutation:: 81, capability:: 903, kernel:: 59
```

Relevant interpretation: the graph snapshot is valid and rich enough for future
graph-edit planning, but it does not yet contain `domain::` nodes. The graph was
captured before the current uncommitted domain Rust implementation was reflected
in graph evidence, so domain progress should not be scored as verified until
tests pass and graph evidence is refreshed.

## Scores

```text
I  Intelligence      = 7.2
A  Architecture      = 8.7
E  Efficiency        = 8.4
C  Correctness       = 9.1
A  Alignment         = 8.1
R  Robustness        = 7.0
P  Performance       = 7.4
S  Scalability       = 9.3
D  Determinism       = 8.8
T  Transparency      = 8.0
Co Collaboration     = 7.9
Em Empowerment       = 8.2
B  Benefit           = 7.6
L  Learning          = 8.5
St Structure         = 7.7
Si Simplicity        = 8.4
F  Future-Proofing   = 8.1
Ch Coherency         = 8.6
```

Approximate geometric mean over the listed score axes remains about:

```text
G ~= 8.14 / 10
```

The score is not raised yet because only targeted contract validation completed;
full-suite validation was blocked by connector-side HTTP 502 errors.

## Rationale

Correctness, determinism, scalability, and coherency are strongest because the kernel, receipts, replay boundaries, graph fixture validation, validation evidence paths, and current planning direction agree on the same evidence-first architecture. Robustness, intelligence, and benefit remain lower because live graph editing, self-modification, and domain intelligence are not yet proven end to end.

Planning-turn evidence on 2026-05-10:

- First incomplete active priority: item 2, `src/domain/contracts.rs` constructors for `DomainSignal`, `DomainContext`, `DomainJudgment`, `DomainPlan`, `DomainRiskEnvelope`, `DomainEval`, and `DomainPromotionCandidate`.
- Existing domain Rust files from `find src/domain -type f | sort`: `src/domain/bridge.rs`, `src/domain/contracts.rs`, `src/domain/identity.rs`, `src/domain/risk.rs`, `src/domain/scoring.rs`, and `src/domain/mod.rs`.
- Missing planned subdomain Rust files: `src/domain/global_intelligence.rs`, `src/domain/business.rs`, `src/domain/finance.rs`, and `src/domain/trading.rs`.
- Existing design notes remain: `business.md`, `contracts.md`, `finance.md`, `global_intelligence.md`, `integration.md`, `README.md`, `roadmap.md`, `scoring.md`, and `trading.md`.
- Python graph analysis confirmed schema version 16, graph hash `ab2202a8d8ec371b0c462aecc41e28d059920f53e2179dd40ebb6ebb3127fc33`, 4,473 nodes, 31,082 edges, 2,976 intents, and 0 `domain::` prefix nodes. Nineteen nodes contain the text `domain`, but they are older agent/runtime references such as `runtime::reducer::raise_domain_failure`, not compiled `src/domain` module evidence.
- Helper-agent spawn was attempted through `canon_spawn_agent` and failed with `connect worker on port 9100: Connection refused`; planning proceeded in this agent.
- `git status --short` showed multiple pre-existing uncommitted implementation/test changes outside this planning turn. This planning turn stages only `plan.md` and `score.md`.


Implementation step 2 evidence on 2026-05-10:

- Completed Active Priorities item 1 in `src/domain/contracts.rs`.
- Added serde coverage for domain schema primitive enums and core record structs.
- Replaced `DomainSignal.signal_class` string storage with typed `DomainSignalClass`.
- Added unit tests for schema primitive JSON round-trip and live-effect safety ordering.
- Targeted validation passed: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test contracts::tests -- --test-threads=1` ran 2 domain contract tests successfully.
- Full validation command attempted twice: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`; both attempts failed at the connector layer with HTTP 502 upstream/external service errors, so no full-suite Rust failure was observed in this turn.
- Helper-agent spawn was requested during this planning turn but failed with `connect worker on port 9100: Connection refused`; planning proceeded locally.

The next score gains should come from evidence, not optimism:

- prove agent-driven graph mutation with Python-assisted graph target selection;
- stabilize domain record contracts and fixtures;
- route self-modification through verified evolution with external eval evidence;
- keep trading sandbox-only until explicit future policy exists.
