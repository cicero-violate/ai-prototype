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

## Graph Schema  (schema_version = 12)

```
graph.json
├── meta
│   ├── crate_name         string
│   ├── schema_version     u32
│   ├── node_count / edge_count
│   ├── captured_at_ms     u64
│   ├── graph_hash         sha256   — hash of all node + edge content
│   ├── intent_hash        sha256   — hash of intents map
│   └── risk_hash          sha256   — hash of risk-relation edges
│
├── nodes                  map<path, GraphNode>
│   └── GraphNode
│       ├── def_id         "crate_num:index"
│       ├── path           fully-qualified Rust path
│       ├── kind           "fn" | "impl" | "trait" | "struct" | "enum" | "ty_alias"
│       ├── def?           SourceSpan — span including outer attributes + body
│       │   ├── file       relative to workspace root
│       │   ├── line / col
│       │   ├── lo         byte offset of first attribute or item keyword
│       │   └── hi         byte offset of closing }
│       ├── source_text?   verbatim UTF-8 bytes [lo .. hi] including attributes
│       ├── sig?           FnSig — structured signature for "fn" nodes
│       │   ├── params     [{name, ty}]
│       │   └── return_ty  source text of return type, "()" if default
│       └── fields         [{name, ty}] — struct fields or enum variant names
│
├── edges                  list<GraphEdge>
│   └── GraphEdge
│       ├── relation       "call" | "impl" | "mut" | "io" | "panic"
│       │                  | "unsafe" | "alloc" | "use"
│       ├── from           caller / implementor / module path
│       ├── to             callee / trait path  OR  fact::* sentinel
│       └── span?          SourceSpan — call-site location for "call" edges
│
└── intents                map<path, "pure" | "mutation">
```

### Edge relation vocabulary

| relation | source    | meaning                                                |
|----------+-----------+--------------------------------------------------------|
| `call`   | HIR + MIR | `from` calls `to`                                      |
| `impl`   | HIR       | `from` implements trait `to`                           |
| `mut`    | HIR + MIR | `from` performs mutation (assignment, SetDiscriminant) |
| `io`     | MIR       | `from` calls a known I/O callee                        |
| `panic`  | MIR       | `from` can panic (Assert, unwrap, expect)              |
| `unsafe` | HIR + MIR | `from` contains an unsafe block or inline asm          |
| `alloc`  | MIR       | `from` performs heap allocation                        |

`fact::mut`, `fact::io`, etc. are virtual sentinel targets.  They appear only
as edge targets, never as keys in `nodes`.

### Captured node kinds

Only locally-defined, non-automatically-derived, non-synthetic items:

- **fn** — top-level functions and associated functions / methods
- **impl** — impl blocks (inherent and trait)
- **trait** — trait definitions

Not captured: structs, enums, type aliases, constants, statics, use
statements, mod declarations, attributes, doc comments, extern crate.

### span coverage

`def.lo` / `def.hi` are byte offsets of the full HIR item span.
For `fn` items this covers the `pub`/`fn` keyword through the closing `}` of
the function body.  For `impl` items this covers the full block including all
methods — method spans are therefore nested inside their parent impl span.
The graph-editor's overlap guard rejects ops that target both a container and
a member simultaneously.

~44 % of nodes have no `def` / `source_text` (macro-generated, compiler-
synthesised, or external-crate items).  This is correct.

## Usage

```bash
cd canon-rustc-v3 && cargo build   # requires nightly + rustc_private

# ad-hoc
CANON_RUSTC_V3_ARTIFACT_DIR=state/rustc \
RUSTC_WRAPPER="$PWD/canon-rustc-v3/target/debug/canon-rustc-v3" \
cargo check

# permanent via .cargo/config.toml
[build]
rustc-wrapper = "canon-rustc-v3/target/debug/canon-rustc-v3"
[env]
CANON_RUSTC_V3_ARTIFACT_DIR = "state/rustc"
```

Output: `$CANON_RUSTC_V3_ARTIFACT_DIR/<crate_name>/graph.json`

## What Is Still Missing for Automatic Refactoring

The graph enables patch-based structural edits today.  Full automatic
refactoring requires the additions below, ranked by impact.

### Gap 1 — Call-site source locations  ★ highest impact  ✓ IMPLEMENTED

`GraphEdge` now carries `span: Option<SourceSpan>` populated from `expr.span`
for `ExprKind::Call` and `ExprKind::MethodCall` in `hir.rs`.  Every `call`
edge now records the byte location of the call expression.

**Unlocks**: safe mechanical rename of any fn within the crate.

### Gap 2 — Struct / enum / type alias nodes  ✓ IMPLEMENTED

`allowed_node_kind` now returns `"struct"`, `"enum"`, `"ty_alias"` for the
corresponding `DefKind` variants.  `visit_item` arms capture:
- struct field names and types in `GraphNode.fields`
- enum variant names in `GraphNode.fields` (ty left empty for variants)
- type aliases: span only

**Unlocks**: type-level refactoring, derive management.

### Gap 3 — Attributes are outside item spans  ✓ IMPLEMENTED

`span_with_attrs(hir_id, item_span)` is called before every `upgrade_span`.
It reads `tcx.hir().attrs(hir_id)` and extends `item_span.lo()` backwards to
the earliest non-expansion attribute span.  `source_text` now covers
`#[must_use]`, `#[cfg(test)]`, `#[derive(...)]` etc.

**Unlocks**: safe removal that includes attributes, conditional compilation
analysis.

### Gap 4 — Use / import statement tracking  ✓ IMPLEMENTED

`visit_item` now handles `ItemKind::Use`.  For each resolved `Res::Def` in
`use_path.res`, a `"use"` edge is emitted: `from = enclosing module`,
`to = imported path`, `span = use-statement span`.  `"use"` is added to
`EDGE_RELATIONS` in `facts.rs`.

**Unlocks**: safe rename of public symbols, move-item refactoring.

### Gap 5 — Structured function signatures  ✓ IMPLEMENTED

`GraphNode` now has `sig: Option<FnSig>` with `{params: [{name, ty}], return_ty}`.
Populated for all `fn` nodes: top-level fns via `ItemKind::Fn { sig, body, .. }`,
impl methods via `ImplItemKind::Fn(fn_sig, body_id)`, and trait methods (both
provided and abstract) via `TraitItemKind::Fn`.  Types are verbatim source
snippets; parameter names come from the function body's `Param::pat`.

**Unlocks**: signature-aware refactoring, type-directed search.

## Capability Matrix

All 5 gaps are now implemented in schema_version 12.

| Refactoring                      | schema 11 | schema 12 (now) |
|----------------------------------+-----------+-----------------|
| Remove dead function             | ✓         | ✓               |
| Add / remove attribute           | ✓         | ✓               |
| Reclassify intent                | ✓         | ✓               |
| Detect risk boundary             | ✓         | ✓               |
| Rename function (crate-internal) | —         | ✓               |
| Change function signature        | —         | partial         |
| Rename a type                    | —         | ✓               |
| Add / remove derive              | —         | ✓               |
| Move item between modules        | —         | ✓               |
| Inline a function                | —         | partial         |
| Extract a function               | —         | —               |

## Reference

- https://chatgpt.com/s/t_69fd62a20ad08191814b4f1f53359714
