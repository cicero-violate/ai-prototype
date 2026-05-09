# Graph as Source of Truth

`graph.json` is the canonical semantic representation of the codebase. The canon-rustc-v3 compiler wrapper captures it during every `cargo check`. It is not a secondary artifact — it is the authoritative index of what exists, how it is connected, and what behavior it exhibits.

## Schema

```text
meta     — crate name, schema version, node/edge counts, content hashes
nodes    — path → { def_id, kind, def: { file, line, col, lo, hi } }
edges    — [ { relation, from, to } ]   (call | alloc | mut | io | panic | unsafe | impl)
intents  — path → label                 (pure | mutation | ...)
```

Every node carries source byte offsets (`lo`, `hi`) and a file path. These are the anchors for all round-trip patch generation. No source location is inferred — it is recorded at capture time by the compiler.

## Mutation Contract

A mutation is a typed operation on the graph that has a deterministic corresponding source patch:

```text
RemoveNode(path)
  → excise bytes [lo, hi] from def.file
  → remove all edges where from == path or to == path
  → re-run wrapper → assert node absent from next graph

RetypeIntent(path, new_label)
  → update intents[path] in graph
  → optionally annotate source with #[allow(...)] / #[must_use] at def.line
  → re-run wrapper → assert intent updated

AddAttribute(path, attr)
  → insert attr text at def.line - 1 in def.file
  → re-run wrapper → assert attribute visible in next graph

RemoveEdge(from, to, relation)
  → locate the call/use at the from node's source span
  → generate source patch removing that specific use
  → re-run wrapper → assert edge absent from next graph
```

## Round-Trip Flow

```text
source code
  ↓  cargo check (canon-rustc-v3 wrapper)
graph.json                ← source of truth
  ↓  mutation agent (separate project)
mutation set [ { op, path, args } ]
  ↓  patch generator (reads lo/hi from graph.json nodes)
source patch (unified diff)
  ↓  apply_patch (chatgpt-mcp-connector)
patched source code
  ↓  cargo check (canon-rustc-v3 wrapper)
new graph.json            ← verify mutation landed
  ↓  diff old graph ↔ new graph
mutation receipt          ← TLog entry
```

The external mutation/query agent never touches source directly. It emits typed operations against the graph. The patch generator translates those operations into source patches using the byte offsets already in the graph. The wrapper re-runs and produces a new graph. The diff between the two graphs is the mutation receipt.

## Subproject Boundary Contract

The graph stack spans three repositories/subprojects. Their boundaries are part
of the correctness contract, not a naming convention.

```text
ai/ root runtime
  owns: graph schema constants, typed mutation operations, deterministic patch
        generation, receipt verification, validation/report evidence, and TLog
        admission rules
  may read: graph.json, graph workflow fixtures, graph mutation receipts
  must not own: compiler-wrapper capture internals or interactive graph editing UI

canon-rustc-v3/
  owns: compiler-wrapper capture, rustc integration, graph.json emission, wrapper
        telemetry, and wrapper validation probes
  may read: root graph schema/version contract
  must not own: runtime state-machine transitions, TLog admission, policy
        promotion, or mutation/query UI behavior

graph-editor/
  owns: human-facing graph inspection/editing workflows and editor-local UX
  may read: graph.json, typed graph operation contracts, mutation receipts
  must not own: compiler-wrapper capture, runtime state-machine transitions,
        TLog admission, or policy promotion
```

Cross-boundary changes must preserve directionality: `canon-rustc-v3` emits
semantic graph evidence, `ai/` verifies and records typed evidence, and
`graph-editor` presents or proposes operations against that evidence. No
subproject may silently replace another subproject's authority. If a future
change requires a boundary exception, the exception must be recorded in this
document and covered by a contract test before implementation behavior depends
on it.

## Separation of Concerns

```text
this project (ai/)
  — produces graph.json via canon-rustc-v3 wrapper
  — defines graph.json schema and version contract
  — defines the mutation operation set and their source-patch semantics
  — defines the verification contract (re-capture + graph diff = receipt)

mutation/query project (separate)
  — queries graph.json to identify targets
  — emits typed mutation operations
  — drives the patch-apply → re-capture → verify loop
  — records mutation receipts into TLog
```

The only shared contract between the two projects is the graph.json schema version and the typed mutation operation set defined here.
