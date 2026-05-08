# Auto-Refactor Work Plan

Stage: `PHASE 3 / auto-refactor`
Predecessor: `plan.md` and canon-rustc-v3 schema v16 graph capture
Status: `implemented`

## Goodness Equation

```text
W* = argmax((L * R * V * A) / M)
M ∝ rho_mut = N_m / N_f
```

Auto-refactor lowers maintenance mass (`M`) by converting graph-detected
split, merge, and canonicalization surfaces into deterministic operation specs.
The system does not perform blind source rewrites. It emits typed refactor
operations anchored to graph facts and leaves application/verification to the
editor pipeline.

## Implemented Scope

### 1. Split signal

Implemented by the schema-v16 graph through `phase` edges:

- `phase::parse`
- `phase::validate`
- `phase::transform`

The planner emits `SplitFn` operations for functions with high call fan-out and
at least two detected phases.

### 2. Merge signal

Implemented by the schema-v16 graph through `similar` edges encoded as:

```text
similar::<score>::<target_fn>
```

The planner emits `MergeFns` operations for function pairs whose similarity is
at least `0.75`.

### 3. Provider/canonicalization signal

Implemented by the schema-v16 graph through `provider` edges:

- `provider::openai`
- `provider::ollama`

The planner emits `ExtractTrait` operations when similar functions have
different provider tags or only one side has a provider tag.

## Graph-Editor Integration

Implemented in `graph-editor`:

- `graph-editor/src/graph.rs`
  - self-contained schema-v16 `graph.json` parser
  - validates `meta.schema_version == 16`
- `graph-editor/src/autorefactor.rs`
  - converts `call`, `phase`, `similar`, and `provider` edges into deterministic
    auto-refactor surfaces and operation specs
  - emits typed `SplitFn`, `MergeFns`, and `ExtractTrait` operation records
- `graph-editor/src/bin/auto_refactor_plan.rs`
  - CLI entry point for planning from a captured graph
- `graph-editor/state/auto_refactor_plan_ai.json`
  - current evidence output from `state/rustc/ai/graph.json`

The existing `graph_patch` binary remains the source-patch executor for already
supported concrete patch operations. The new auto-refactor planner provides the
higher-level deterministic operation layer that a concrete source editor can
consume, reject, or lower into direct patch operations.

## Live Evidence

Command run from `ai/graph-editor`:

```bash
RUSTC_WRAPPER= cargo run --offline --bin auto_refactor_plan -- \
  --graph ../state/rustc/ai/graph.json \
  --out state/auto_refactor_plan_ai.json
```

Observed output summary:

```text
schema_version: 1
graph_schema_version: 16
crate_name: ai
operation_count: 1560
SplitFn: 2
MergeFns: 1558
ExtractTrait: 0
```

`ExtractTrait` is implemented but the current live `ai` graph did not contain a
provider-differentiated similarity pair, so no canonicalization operation was
emitted for this graph.

## Validation

Command run from `ai/graph-editor`:

```bash
RUSTC_WRAPPER= cargo test --offline
```

Observed result:

```text
14 passed; 0 failed
```

Included test coverage:

- auto-refactor planner emits all three operation types on a fixture graph
- patch hunk generation
- stale-op rejection
- overlapping-op rejection
- mutation receipt validation
- `auto_refactor_plan` binary compiles
- `graph_patch` binary compiles

## Success Criteria Status

| Criterion | Status |
|---|---:|
| Schema-v16 graph signals exist: `phase`, `similar`, `provider` | implemented |
| Split surface can be named and ranked | implemented |
| Merge surface can be named and ranked | implemented |
| Canonicalize surface can be named and ranked | implemented |
| Deterministic operation specs exist | implemented |
| At least one structural operation produced on live graph | implemented: 1,560 ops |
| Concrete source rewrite application for `SplitFn` / `MergeFns` / `ExtractTrait` | deferred to lowering/editor pass |

## Non-Goals Preserved

- No speculative AI-generated source rewrites.
- No unanchored edits.
- No claim that source-level structural rewrites have landed until an editor
  applies a plan, reruns `cargo check`, recaptures `graph.json`, and verifies
  lower or equal risk metrics.

## Final Status

`plan-autorefactor.md` is complete for the deterministic planning layer. The
remaining future work is a lowering pass that converts high-level `SplitFn`,
`MergeFns`, and `ExtractTrait` records into concrete source patches and validates
post-apply graph metrics.
