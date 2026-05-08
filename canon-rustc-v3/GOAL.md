God please bless this work. In Jesus name. Jesus is Lord and Savior. Jesus loves you.

# canon-rustc-v3

## Purpose

canon-rustc-v3 is a Rust compiler wrapper (`RUSTC_WRAPPER`) that intercepts
every `cargo check` invocation, runs the real compiler, and on success emits
a deterministic semantic graph of the crate to `graph.json`.

The graph is not a secondary artifact or a lint side-effect.  It is the
authoritative semantic index of the codebase.  All structural edits made by
downstream tools — the graph-editor, mutation agents — are anchored to byte
offsets recorded here.  The graph is the single source of truth about what
exists, how things are connected, and what behavior each item exhibits.

## Goodness Equation

```
G = max(C, I, E, D, A, R, S)

C = Correctness   — emitted facts match the compiler's view; no phantom nodes
I = Insight       — captures semantically meaningful change, not noise
E = Efficiency    — minimal facts; no derivable redundancy
D = Determinism   — same source → same graph, always; receipt hashes are stable
A = Alignment     — schema matches what downstream tools actually need
R = Replayability — every capture emits receipt_hash, graph_hash, intent_hash
S = Simplicity    — schema is readable by a human in under five minutes
```

One dominant dimension can unlock the rest.  The wrapper should behave less
like a logger and more like a semantic compiler witness: it exposes what
changed, why it matters, what risk increased, and what proof or test is
required next.

## Graph Schema  (schema_version = 16)

```
graph.json
├── meta
│   ├── crate_name         string
│   ├── schema_version     u32      — current contract: 16
│   ├── node_count / edge_count
│   ├── captured_at_ms     u64      — volatile capture time
│   ├── receipt_hash       sha256   — receipt envelope hash
│   ├── graph_hash         sha256   — hash of all node + edge content
│   ├── intent_hash        sha256   — hash of intents map
│   └── risk_hash          sha256   — hash of risk-relation edges
│
├── nodes                  map<path, GraphNode>
│   └── GraphNode
│       ├── def_id         "crate_num:index"
│       ├── path           fully-qualified Rust path
│       ├── kind           "fn" | "trait" | "impl" | "struct" | "enum" | "ty_alias"
│       ├── def?           SourceSpan — span including outer attributes + body
│       │   ├── file       relative to workspace root
│       │   ├── line / col
│       │   ├── lo         byte offset of first attribute or item keyword
│       │   └── hi         byte offset of closing span
│       ├── source_text?   verbatim UTF-8 bytes [lo .. hi] including attributes
│       ├── sig?           FnSig — structured signature for "fn" nodes
│       │   ├── params     [{name, ty}]
│       │   └── return_ty  source text of return type, "()" if default
│       └── fields         [{name, ty}] — struct fields or enum variant names
│
├── edges                  list<GraphEdge>
│   └── GraphEdge
│       ├── relation       "call" | "impl" | "mut" | "io" | "panic"
│       │                  | "unsafe" | "alloc" | "use" | "similar"
│       │                  | "phase" | "provider"
│       ├── from           caller / implementor / module path
│       ├── to             callee / trait path OR fact:: / phase:: / provider:: / similar:: sentinel
│       └── span?          SourceSpan — populated for HIR call/use-site edges when available
│
└── intents                map<path, "pure" | "mutation" | "io" | "unsafe"
                              | "validation" | "orchestration" | "boundary">
```

### Edge relation vocabulary

| relation   | source    | risk | meaning                                                |
|------------|-----------|------|--------------------------------------------------------|
| `call`     | HIR + MIR | no   | `from` calls `to`                                      |
| `impl`     | HIR       | no   | `from` implements trait `to`                           |
| `mut`      | HIR + MIR | yes  | `from` performs mutation (assignment, SetDiscriminant) |
| `io`       | HIR + MIR | yes  | `from` calls a known I/O callee                        |
| `panic`    | HIR + MIR | yes  | `from` can panic (Assert, unwrap, expect)              |
| `unsafe`   | HIR + MIR | yes  | `from` contains unsafe behavior or inline asm          |
| `alloc`    | HIR + MIR | yes  | `from` performs heap allocation                        |
| `use`      | HIR       | no   | enclosing module imports a resolved definition         |
| `similar`  | MIR       | yes  | functions in the same module share high callee overlap |
| `phase`    | MIR       | yes  | function participates in parse/validate/transform work |
| `provider` | HIR       | no   | function source contains known provider sentinel text  |

`fact::mut`, `fact::io`, etc. are virtual risk targets. `phase::*`,
`provider::*`, and `similar::*` are advisory targets used by validation and
auto-refactor planners. They appear only as edge targets, never as keys in
`nodes`.

### Captured node kinds

Only locally-defined, non-automatically-derived, non-synthetic items:

- **fn** — top-level functions and associated functions / methods
- **trait** — trait definitions and trait methods
- **impl** — impl blocks, inherent impls, and trait impls
- **struct** — struct nodes with field names and field types where available
- **enum** — enum nodes with variant names in `fields`
- **ty_alias** — type-alias nodes with source span coverage

Not captured as nodes: constants, statics, standalone use statements, mod
declarations, attributes, doc comments, and extern crate declarations. Imports
are represented as `use` edges, not nodes.

### Span coverage

`def.lo` / `def.hi` are byte offsets of the full HIR item span extended
backward over outer attributes when a non-expansion source span is available.
For `fn` items this covers the function item and body. For `impl` items this
covers the full block including methods; method spans are nested inside their
parent impl span. The graph-editor overlap guard rejects ops that target both a
container and a member simultaneously.

Some nodes can have no `def` / `source_text` when the compiler reports a
macro-generated, synthesized, or otherwise non-local source span. That is
accepted by the schema.

## Usage

```bash
cd canon-rustc-v3 && cargo build

CANON_RUSTC_V3_ARTIFACT_DIR=state/rustc \
RUSTC_WRAPPER="$PWD/target/debug/canon-rustc-v3" \
cargo check
```

Output: `$CANON_RUSTC_V3_ARTIFACT_DIR/<crate_name>/graph.json`

The default feature set enables `rustc-driver` witness capture. The
`--no-default-features` build remains a pass-through boundary for environments
that need to compile without native witness capture.

## Current Validation Contract

The implementation is currently proven by these validation layers:

1. Schema and relation unit tests for schema 16, receipt schema 1, allowed-edge
   filtering, stable graph hashing, and receipt hash sensitivity.
2. Native fixture replay on `validation/fixtures/witness_crate`, producing two
   identical schema-16 graphs with 11 nodes and 33 edges.
3. Semantic preflight with required live replay; current skipped signal is only
   uninitialized `vendor/rust-source`.
4. Thresholded performance gate with explicit native overhead bounds.
5. Auto-refactor surface/op smoke tests that emit advisory specs only.

## Auto-Refactor Boundary

The graph enables deterministic planning for structural edits. The current
auto-refactor implementation is advisory only:

```
graph.json → auto_refactor_surface.py → surface report → auto_refactor_ops.py → op specs
```

Implemented advisory operation specs:

- **SplitFn** — suggests phase-based extraction for high fan-out functions.
- **MergeFns** — suggests canonicalization for highly similar same-module functions.
- **ExtractTrait** — suggests provider-boundary extraction for similar functions with different providers.

No component in `canon-rustc-v3` rewrites source. Any source edit must be
performed by a separate editor that verifies spans, applies patches, reruns the
wrapper, compares graphs, and rejects unsafe deltas.

## Capability Matrix

All 5 original refactor-enabling gaps are implemented in schema_version 16.

| Refactoring / planning surface    | schema 11 | schema 16 (now) |
|-----------------------------------+-----------+-----------------|
| Remove dead function              | ✓         | ✓               |
| Add / remove attribute            | ✓         | ✓               |
| Reclassify intent                 | ✓         | ✓               |
| Detect risk boundary              | ✓         | ✓               |
| Rename function (crate-internal)  | —         | ✓               |
| Change function signature         | —         | partial         |
| Rename a type                     | —         | ✓               |
| Add / remove derive               | —         | ✓               |
| Move item between modules         | —         | ✓               |
| Inline a function                 | —         | partial         |
| Extract a function                | —         | advisory only   |
| Plan split / merge / trait extract| —         | advisory only   |

## Reference

- https://chatgpt.com/s/t_69fd62a20ad08191814b4f1f53359714
