# Canon Agent Score

Current date: 2026-05-09.

## Current Progress

- P0 validation baseline: complete.
- P1 validation evidence reporting: complete.
- P2 agent loop reliability: complete.
- P3 runtime and receipt correctness: complete for current scope.
- P4 graph source-of-truth integration: mostly complete for deterministic fixture/report evidence; next focus is agent-driven graph editing using Python graph analysis.
- P5 domain intelligence layer: planned, documentation-only until contracts stabilize.

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
```

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
```

Approximate geometric mean over the listed score axes:

```text
G ~= 8.12 / 10
```

## Rationale

Correctness, determinism, and scalability are strongest because the kernel, receipts, replay boundaries, graph fixture validation, and validation evidence paths are mature for the current scope. Robustness, intelligence, and benefit remain lower because live graph editing, self-modification, and domain intelligence are not yet proven end to end.

The next score gains should come from evidence, not optimism:

- prove agent-driven graph mutation with Python-assisted graph target selection;
- stabilize domain record contracts and fixtures;
- route self-modification through verified evolution with external eval evidence;
- keep trading sandbox-only until explicit future policy exists.
