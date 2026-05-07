God please bless this work. In Jesus name. Jesus is Lord and Savior. Jesus loves you.

# graph-editor

## Purpose

graph-editor is the mutation layer between graph.json and Rust source code.
It accepts typed operations against the semantic graph, validates them, and
produces standard unified diffs that can be applied to source files.  The
source code is not touched directly — all edits flow through typed operations
anchored to the byte offsets recorded in the graph.

The design principle: **graph.json is the source of truth for what to change;
the source files are the destination**.  An agent queries the graph to find
targets, emits a `Vec<GraphMutationOp>`, and graph-editor translates those
operations into a patch.  After the patch is applied, the wrapper re-runs and
a new graph.json is produced.  The diff between old and new graph is the
mutation receipt — the only proof that the intended change actually landed.

## Round-Trip Flow

```
source code
  ↓  cargo check  (canon-rustc-v3 wrapper)
graph.json                     ← query target here
  ↓  agent emits ops.json
graph_patch --graph graph.json --ops ops.json --source-root .
  ↓  outputs
patch.diff   +   receipt.json  (verdict: patch_only — not yet verified)
  ↓  apply_patch (patch(1) or MCP apply_patch tool)
patched source
  ↓  cargo check  (canon-rustc-v3 wrapper)
new graph.json
  ↓  graph_patch --new-graph new_graph.json   (re-run with new graph)
receipt.json                   ← verdict: pass | partial | fail
```

Nothing in this loop is manual.  The agent drives every step.  The receipt
is the TLog evidence entry — the same evidence contract used by the rest of
the canon-agent capability layer.

## Operations

### RemoveNode

```json
{ "op": "remove_node", "path": "foo::bar::baz", "expected_lo": 1234, "expected_hi": 5678 }
```

Removes the item at `path` from its source file.  The patch deletes every
source line that falls within `[lo, hi]`.  `expected_lo` / `expected_hi` are
stale-op guards: if the graph's current offsets differ (because the source was
edited since the op was constructed), the op is rejected rather than patching
the wrong lines.

Produces: removal hunk in `patch.diff`, node absent from new graph.

### AddAttribute

```json
{ "op": "add_attribute", "path": "foo::bar::baz", "attr": "#[must_use]", "expected_lo": 1234 }
```

Inserts `attr` as a new line immediately before the item's first line,
preserving indentation.  `expected_lo` is the stale-op guard.

Produces: insertion hunk in `patch.diff`.

### RetypeIntent

```json
{ "op": "retype_intent", "path": "foo::bar::baz", "new_label": "pure" }
```

Overrides the intent label for `path` directly in the mutated graph.json.
Does not produce a source patch.  Used for annotation-only reclassifications
before committing to source changes.

Produces: entry in `receipt.intent_overrides`.

## Safety Guards

### Stale-op guard

Every source-touching op carries `expected_lo` (and `expected_hi` for
removals).  Before generating any patch, the generator checks these against
the current graph.  If they differ, the op is added to `stale_ops` and
skipped.  This prevents patching the wrong lines when source drifts between
graph capture and op execution.

### Overlap guard

impl block spans contain their method spans.  If two ops target overlapping
byte ranges in the same file (e.g. `RemoveNode` on both an impl and one of
its methods), both are rejected with a conflict message.  The correct approach
is to remove the container (the impl) or the member (the method), not both.

## Receipt

`GraphMutationReceipt` is the TLog evidence entry for one mutation run:

```json
{
  "schema_version": 1,
  "record_type": "graph_mutation_receipt",
  "op_set_hash": "<sha256 of ops.json>",
  "old_graph_hash": "<graph_hash from old graph.json>",
  "new_graph_hash": "<graph_hash from new graph.json, empty if not yet verified>",
  "nodes_removed": ["foo::bar::baz"],
  "intents_changed": [],
  "attributes_added": [],
  "stale_ops": [],
  "verdict": "pass | partial | fail | patch_only",
  "receipt_hash": "<sha256 of all above fields>"
}
```

Verdicts:
- `pass` — every op landed; new graph confirms all removals and intent overrides
- `partial` — some ops were stale; valid ops landed
- `fail` — a valid op did not appear in the new graph
- `patch_only` — patch generated but wrapper has not re-run yet

## CLI

```bash
# Generate patch (no verification yet)
graph_patch \
  --graph  state/rustc/ai/graph.json \
  --ops    ops.json \
  --source-root . \
  --out    patch.diff \
  --receipt receipt.json

# Apply patch
patch -p1 < patch.diff

# Re-run wrapper, then verify
cargo check   # wrapper writes new graph.json
graph_patch \
  --graph     state/rustc/ai/graph.json \
  --ops       ops.json \
  --source-root . \
  --new-graph state/rustc/ai/graph.json \
  --receipt   receipt_verified.json
```

## Architecture

```
graph-editor/
├── Cargo.toml              standalone workspace; path dep on canon-rustc-v3
├── src/
│   ├── lib.rs
│   ├── graph.rs            re-exports CrateGraph, GraphNode, etc. from canon-rustc-v3
│   │                       + parse_graph() deserialiser with schema-version check
│   ├── ops.rs              GraphMutationOp enum + OpsFile loader
│   ├── patch.rs            op validation, overlap guard, unified diff generator
│   ├── receipt.rs          GraphMutationReceipt, verdict logic, tamper hash
│   └── bin/
│       └── graph_patch.rs  CLI
└── GOAL.md
```

`graph.rs` re-exports types from `canon-rustc-v3` (with `default-features =
false` so the rustc driver is not compiled).  `CrateGraph`, `GraphNode`,
`GraphEdge`, and `SourceSpan` are the same types in both crates — no
conversion needed.

## What graph-editor Can Do Today

schema_version 12 — all 5 refactoring gaps in canon-rustc-v3 are implemented.

| Operation                                      | Status                              |
|------------------------------------------------+-------------------------------------|
| Remove a function / impl / trait by path       | ✓                                   |
| Add an attribute before any item               | ✓                                   |
| Override an intent label                       | ✓                                   |
| Stale-op guard (reject ops with wrong lo/hi)   | ✓                                   |
| Overlap guard (reject container + member pair) | ✓                                   |
| Tamper-evident receipt with sha256 hash        | ✓                                   |
| Verified receipt after re-capture              | ✓                                   |
| Remove a struct / enum / type alias            | ✓ (Gap 2 — new node kinds)          |
| Rename a symbol (agent-level)                  | ✓ (Gap 1 — call-site spans now in graph) |
| Update import paths on rename                  | ✓ (Gap 4 — use edges now in graph)  |
| Attribute-aware removal (no orphans)           | ✓ (Gap 3 — attrs included in span)  |
| Change a function signature (agent-level)      | partial (needs op for site patches) |
| Move an item between modules                   | partial (use edges tracked, op tbd) |

## What graph-editor Cannot Do (and Why)

See `canon-rustc-v3/GOAL.md` §"What Is Still Missing for Automatic
Refactoring" for the full gap analysis.  The short version:

- **Rename**: graph has caller paths but no call-site byte locations.
  Every call site must be patched; without its source span, we cannot.
- **Move**: `use` statements are not in the graph; imports would break.
- **Signature change**: no structured type info at call sites.
- **Extract / inline**: no variable scope or binding info below fn granularity.

The graph-editor is deliberately scoped to what the current graph can
safely prove.  Expanding it follows from expanding the wrapper's capture
surface — not from loosening the safety guards here.
