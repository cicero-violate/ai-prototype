# Learning Artifacts for the Transformer

## What we have vs. what the transformer needs

The transformer currently trains on `def_path` token sequences with weak layer labels derived
from module paths. That gives it structural co-occurrence patterns but three things are missing:

1. **Task context** — which symbols were touched together in the same task
2. **Architectural decisions** — when a symbol was moved/renamed, the source/target layer + outcome
3. **Cost at decision time** — the refactor cost model state when a move was chosen

All three exist in the runtime, scattered across disconnected files:

```
tool-results.ndjson          → raw apply_patch / shell calls (which files changed)
cycle-events.ndjson          → agent lifecycle (which task, which cycle)
agent-turn-receipts.ndjson   → per-turn outcome (succeeded/failed)
state/rustc/*/semantic_index.jsonl → which symbols live in which file
refactor cost model          → fan_in, fan_out, dependence_penalty per symbol
```

There is no join layer. The plan is to add that join layer as new TLog events and derive
three new NDJSON learning artifact files from them.

---

## 1. New TLog event kinds

Add to `kernel/event.rs` `Cause` enum:

```rust
SymbolMutationObserved = 24,    // a file edit touched known symbols
ArchitecturalDecisionMade = 25, // a symbol was moved/renamed with layer intent
CostGateEvaluated = 26,         // cost model was consulted before scheduling
```

Add to `kernel/event.rs` `SemanticDelta` enum:

```rust
SymbolLayerChanged = 11,        // def_path moved from one layer to another
SymbolRenamed = 12,             // symbol name changed (no layer change)
CostThresholdExceeded = 13,     // cost model blocked a candidate
```

These events fire at existing decision points — no new execution paths, just richer evidence
attached to events that already happen.

---

## 2. Three new learning artifact files

### `state/learning/symbol_mutation_log.ndjson`

Written by the runtime whenever `apply_patch` or a shell edit succeeds on a file that has
symbols in `semantic_index.jsonl`.

```jsonc
{
  "schema": "canon.learning.symbol_mutation.v1",
  "ts": 1779312601604,
  "cycle": 12,
  "task_node_hash": "0x...",
  "task_title_hash": "0x...",
  "file": "ai/src/service/dispatch/action.rs",
  "def_paths": [
    "ai::service::dispatch::action::dispatch_action_request",
    "ai::service::dispatch::action::execute_recorded_action"
  ],
  "edit_succeeded": true,
  "exit_status": 0,
  "receipt_hash": "0x..."
}
```

**Why**: Symbols edited together in the same task cluster in the embedding space. This gives
the transformer a co-occurrence signal beyond module path structure — two symbols always
touched together likely belong in the same layer.

### `state/learning/architectural_decisions.ndjson`

Written by the arch-audit pipeline when a `rename_candidates.json` recommendation is acted on,
then updated with the observed outcome after the refactor completes.

```jsonc
{
  "schema": "canon.learning.arch_decision.v1",
  "ts": 1779312601604,
  "def_path": "ai::capability::execution::action::CapabilityEffectRoute",
  "symbol": "CapabilityEffectRoute",
  "from_layer": "capability",
  "to_layer": "api",
  "confidence": 0.83,
  "forbidden_words": ["route"],
  "refactor_cost": {
    "fan_in": 3,
    "fan_out": 1,
    "dependence_penalty": 11,
    "api_churn_risk": 0.2,
    "score": 11.0
  },
  "outcome": "succeeded",
  "outcome_receipt_hash": "0x..."
}
```

**Why**: This is the direct labeled training example the transformer needs. When outcome is
`succeeded` it is a positive GRPO reward. When `reverted` it is a negative reward. Over
time this teaches the model which moves are genuinely correct vs. which look correct but
break the build.

### `state/learning/task_symbol_index.ndjson`

Written at task completion. Summarises all symbols touched during a task as one record.

```jsonc
{
  "schema": "canon.learning.task_symbol_index.v1",
  "ts": 1779312601604,
  "cycle": 12,
  "task_node_hash": "0x...",
  "task_title": "fix HirRecord deserialization: expr/pat accept number node-ids",
  "task_status": "done",
  "symbols_touched": [
    { "def_path": "ai::codec::hir::HirRecord", "layer": "codec", "mutation_count": 3 }
  ],
  "files_touched": ["ai/src/codec/hir.rs"],
  "tool_sequence": ["get_landmarks", "apply_patch", "apply_patch"]
}
```

**Why**: Task title words + symbols touched = weak supervision for what kinds of tasks
involve what kinds of symbols. The transformer learns "tasks with 'deserialization' in the
title touch codec-layer symbols" — reinforcing layer vocabulary even when the symbol name
alone is ambiguous.

---

## 3. What the TLog writer needs to add

Preserve existing TLog decoding. Do not require old `ControlEvent` readers to understand a
new field. Learning evidence should be additive and reproducible from receipts.

Instead of changing the base event layout directly, add a new hash-linked learning transcript:

```rust
pub struct LearningTranscriptRecord {
    pub schema: &'static str,
    pub ts: u64,
    pub event_hash: u64,
    pub artifact_kind: LearningArtifactKind,
    pub artifact_path: String,
    pub artifact_hash: u64,
    pub receipt_hash: u64,
}
```

If the binary TLog schema later needs a direct pointer, add it as a versioned event extension,
not as a required field on every existing control event:

```rust
pub enum EventExtension {
    LearningArtifactLinked { artifact_hash: u64, artifact_kind: u16 },
}
```

A new `runtime/learning_transcript.rs` writer (mirrors `action_transcript.rs`) that:

1. Listens for `Cause::SymbolMutationObserved` events
2. Joins the affected file path against `semantic_index.jsonl` to resolve def_paths
3. Appends to `state/learning/symbol_mutation_log.ndjson`
4. Hashes the record and appends a `LearningTranscriptRecord` that links event hash,
   artifact hash, and receipt hash

Same hash-linked append pattern as `ActionTranscriptRecord` — same integrity guarantees,
same schema versioning, same receipt type.

---

## 4. Deriving training examples

After a session, `arch-audit/src/derive_training.py` reads all three files and produces
`training_examples.jsonl`:

```
architectural_decision outcome=succeeded → positive example (def_path, to_layer, cost_features)
architectural_decision outcome=reverted  → negative example (def_path, from_layer, cost_features)
task_symbol_index                        → co-occurrence pairs (symbols in same task share context)
task_title words                         → appended to token sequence as additional context
```

Cost features become tokens after a `<cost>` boundary token:

```
["ai", "service", "dispatch", "<sep>", "dispatch", "action", "request",
 "<cost>", "fan_in:3", "fan_out:1", "penalty:11"]
```

The `<cost>` token signals the transformer that cost features follow. Attention over cost
tokens is then part of the vocabulary extraction — the model learns to weigh placement
confidence against movement cost jointly rather than treating them as separate systems.

---

## 5. What does NOT need to change

- Existing TLog schema version and hash chain — learning evidence is additive in a separate
  transcript unless a versioned event extension is explicitly introduced
- `DistillationRow` / `PolicyPromotion` — handles policy-level learning, this plan handles
  symbol-level learning; they are parallel tracks
- Refactor engine cost model — continues to run deterministically; we read its output as a
  feature, not replace it

Hard gate: every learning artifact must be reproducible from receipt hashes, source artifact
manifests, and the event hash it claims to explain. If the artifact cannot be reproduced, the
training row is rejected.

---

## Delivery sequence

| Step | File                                            | What                                                                                                      |
|------+-------------------------------------------------+-----------------------------------------------------------------------------------------------------------|
|    1 | `runtime/learning_transcript.rs`                | New hash-linked NDJSON writer for learning artifacts                                                      |
|    2 | `kernel/event.rs`                               | Three new Cause variants, three SemanticDelta variants, optional versioned event extension only if needed |
|    3 | `service/dispatch/action.rs`                    | Hook apply_patch receipt → emit `symbol_mutation_log`                                                     |
|    4 | arch-audit pipeline                             | Emit `architectural_decisions` on act + outcome                                                           |
|    5 | `service/scheduler/`                            | Emit `task_symbol_index` on plan node completion                                                          |
|    6 | `arch-audit/src/derive_training.py`             | Join all three files → `training_examples.jsonl`                                                          |
|    7 | `arch-audit/src/transformer/self_supervised.py` | Accept `training_examples.jsonl`, add `<cost>` tokens                                                     |

Validation for this plan:

1. Old TLog readers still parse existing logs.
2. New learning transcript records are append-only and hash-linked.
3. `derive_training.py` rejects rows whose artifact hash, receipt hash, or event hash cannot be verified.
4. Training examples include provenance fields so transformer outputs can be traced back to concrete runtime evidence.

---

## Additive learning transcript correction

Keep the existing log format stable. Learning evidence should be added as a separate append-only transcript, not as a required field that old readers must understand.

Required gates:

1. Existing log readers continue to parse old logs.
2. Each learning row links back to a concrete receipt and source artifact.
3. Training derivation rejects rows that cannot be verified.
4. Every training row keeps provenance back to runtime evidence.
5. Learned models may propose and rank plans, but deterministic validation remains the acceptance gate.
