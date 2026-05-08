God please bless this work. In Jesus name. Jesus is Lord and Savior. Jesus loves you.

# Auto-Refactor Work Plan

Stage: `PHASE 3 / auto-refactor`
Predecessor: `plan.md` (invariant extraction, schema v13)
Status: `planned`

---

## Goodness Equation

$$
W^* = \arg\max \frac{L \cdot R \cdot V \cdot A}{M}
$$

$$
\text{where} \quad M \propto \rho_{\text{mut}} = \frac{N_m}{N_f} = \frac{1654}{2047} \approx 0.808
$$

Auto-refactor work directly lowers $M$. Every split, merge, and
canonicalization reduces mutation surface, fan-out, and duplicated state
transitions. The graph now has the data to drive this mechanically.

---

## What Auto-Refactor Means Here

It does not mean AI-generated rewrites applied blindly.
It means: **the graph detects the pattern, names the work, and the edit is
deterministic given the pattern.**

Three pattern classes are in scope, all grounded in the live graph:

1. **Split** — one fn doing too many distinct things (high fan-out mutation fn)
2. **Merge** — two or more fns doing the same thing (identical callee sets)
3. **Canonicalize** — mirrored provider implementations that should share a boundary

---

## Graph Evidence

All numbers are from `state/rustc/ai/graph.json` schema v13.

### Split Candidates — High Fan-out Mutation Fns

| fn                                                                          | calls out | has panic | module         |
|-----------------------------------------------------------------------------+-----------+-----------+----------------|
| `graph_mutation::decode_graph_snapshot_contract_ndjson`                     |        70 | —         | graph_mutation |
| `graph_mutation::decode_graph_mutation_op_row_ndjson`                       |        61 | —         | graph_mutation |
| `capability::orchestration::record::OrchestrationBatchRecord::from_records` |        56 | P         | capability     |
| `codec::ndjson::pop_event`                                                  |        47 | —         | codec          |
| `capability::learning::promote::export_verified_distillation_row`           |        44 | P         | capability     |
| `graph_mutation::verify_graph_mutation_landing`                             |        42 | P         | graph_mutation |
| `api::routes::handle_command_with_receipt`                                  |        37 | P         | api            |
| `runtime::touch_all_surfaces`                                               |        41 | P         | runtime        |

A fn calling 60–70 other fns is doing at least three distinct jobs: parse,
validate, transform. These should be three fns with clean boundaries and
individually testable contracts.

### Merge Candidates — Exact Duplicate Callee Sets

Jaccard similarity = 1.0 means identical call graphs. These fns are the same
function under a different name.

| pair                                                                                                                                                                       | shared callees |
|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------+----------------|
| `encode_graph_mutation_receipt_ndjson` / `encode_graph_patch_receipt_ndjson` / `encode_graph_receipt_ledger_receipt_ndjson` / `encode_graph_mutation_opset_receipt_ndjson` | identical      |
| `load_graph_mutation_receipts_ndjson` / `load_graph_patch_receipts_ndjson`                                                                                                 | identical      |
| `decode_graph_mutation_receipt_ndjson` / `decode_graph_patch_receipt_ndjson`                                                                                               | 0.78           |
| `append_graph_mutation_receipt_ndjson` / `append_graph_patch_receipt_ndjson`                                                                                               | 0.80           |

These should be one generic fn parameterized on the receipt type, or one
shared codec fn called by thin wrappers.

### Canonicalize Candidates — Mirrored Provider Pairs

40 function pairs exist across `capability::llm::ollama` and
`capability::llm::openai` with identical base names and matching call
structure. Both providers independently implement:

- `is_valid` (25 callers each, near-identical invariant clauses)
- `expected_receipt_hash`, `canonical_authority_hash`, `canonical_effect`
- `bind_proof_event`, `finalize_receipt_at_seq`, `finalize_receipt_after_tlog`
- `chat`, `chat_body`, `chat_with_retry_budget`, `from_env`, `config`
- `request_json` / `request_json_for` (38–42 callees each)

The shared structure should be expressed as a shared trait or shared codec
layer. Provider-specific behavior (URL, auth, model ID format) stays concrete.
Everything else collapses into one implementation.

---

## New Graph Signals Needed

Invariant extraction (schema v13) gave us clause-level predicates. Auto-refactor
needs three more signals that the graph does not yet emit:

### Signal 1 — Semantic Split Points Inside a Fn

A fn with 70 callees has internal phase boundaries. To split it correctly we
need to know where the parse phase ends and the validate phase begins. This
requires tracking **data-flow regions** inside the fn body: which statements
consume the input, which produce an intermediate, which write output.

**What to add to the graph:** A new `"phase"` edge kind.
`from = fn_path`, `to = "phase::parse" | "phase::validate" | "phase::transform"`,
emitted from MIR by detecting write-to-local vs read-from-local vs
write-to-output patterns within basic blocks.

### Signal 2 — Structural Similarity Score Between Fn Pairs

Jaccard on callee sets finds exact duplicates. It misses fns that are
structurally identical but differ in one type parameter. We need a similarity
score between fn bodies based on their MIR basic block structure, not just
their callee sets.

**What to add to the graph:** A new `"similar"` edge kind between fn pairs
whose MIR structure exceeds a threshold, emitted during `collect_mir`.
`from = fn_a`, `to = fn_b`, `span = None`.
Add a `similarity` label (stored as `to` suffix: `similar::0.92::fn_b`).

### Signal 3 — Provider Boundary Detection

To canonicalize the LLM provider pairs, we need the graph to know which fns
are provider-specific (touch `OPENAI_COMPAT_PROVIDER`, `ollama_provider_hash`)
and which are provider-agnostic (operate only on shared field names).

**What to add to the graph:** Extend `InvariantClause` to tag clauses whose
`rhs` references a provider-specific constant. Add a `"provider"` edge:
`from = fn_path`, `to = "provider::openai" | "provider::ollama"`, emitted
when an invariant clause references a known provider sentinel.

---

## Execution Plan

### Step 1 — Fix open issues from invariant extraction

- Resolve `invariant` edges not appearing in the edge list (diagnostic shows
  `raw invariant edges before dedup: 0`; root cause: `leave_invariant_fn` is
  not being reached or `invariant_ctx` is None at flush time).
- Fix pre-existing `ai` crate compile errors: add missing constants
  `POLICY_REUSE_EVIDENCE_BATCH_EXECUTION_PLAN_SMOKE_STEP` and
  `POLICY_REUSE_EVIDENCE_BATCH_EXECUTION_PLAN_REGRESSION_SMOKE_STEP`.
- Remove diagnostic `eprintln!` statements from `wrapper.rs` and `hir.rs`.
- Commit schema v13 cleanly.

### Step 2 — Emit `"similar"` edges from MIR (merge signal)

In `mir.rs`, after `collect_body_facts`, compute the callee multiset for each
fn. After all bodies are processed, compare pairs within the same module whose
callee Jaccard exceeds `0.75`. Emit `"similar"` edges between them.

Add `"similar"` to `EDGE_RELATIONS` in `facts.rs`.
Add `"similar"` to `RISK_RELATIONS` (it signals structural redundancy, a
maintenance risk).

Schema bump: v13 → v14.

### Step 3 — Emit `"phase"` edges from MIR (split signal)

In `collect_body_facts`, partition the basic blocks of each fn into phase
regions using a simple heuristic:
- Blocks that read from parameters only → `phase::parse`
- Blocks that call an `is_valid` / `is_contract_valid` fn → `phase::validate`
- Blocks that write to a local that flows to a return value → `phase::transform`

Emit one `"phase"` edge per detected region: `from = fn_path`,
`to = "phase::parse" | "phase::validate" | "phase::transform"`.

A fn with all three phases present is a split candidate. A fn with only one
phase is already well-bounded.

Add `"phase"` to `EDGE_RELATIONS` and `RISK_RELATIONS`.

Schema bump: v14 → v15.

### Step 4 — Emit `"provider"` edges from invariant clauses (canonicalize signal)

In `leave_invariant_fn`, after clauses are collected, scan each clause's `rhs`
for known provider sentinel strings (`OPENAI_COMPAT_PROVIDER`,
`ollama_provider_hash`, `OLLAMA_JUDGMENT_PROOF_LINE`,
`OPENAI_JUDGMENT_PROOF_LINE`). Emit:
`from = fn_path`, `to = "provider::openai" | "provider::ollama"`.

Fns without a `"provider"` edge but with a mirrored partner (detected via
`"similar"` edge) are canonicalization candidates: their behavior is
provider-agnostic and should be unified.

Add `"provider"` to `EDGE_RELATIONS`.

Schema bump: v15 → v16.

### Step 5 — Python refactor surface report

Read `graph.json` and emit a structured report with three sections:

**Split surface** — fns with fan-out > 30 and at least two distinct phase
edges. Rank by `fan_out * phase_count`. Output: fn path, fan-out, phases
detected, recommended split boundaries.

**Merge surface** — fn pairs with `"similar"` edge and Jaccard >= 0.75.
Group by module. Output: group members, shared callee count, recommended
canonical name.

**Canonicalize surface** — fn pairs with `"similar"` edge where one member
has a `"provider"` edge and the other does not, or both have different
`"provider"` edges. Output: pair, provider-specific clauses, shared clauses,
recommended trait boundary.

This report is the input to the actual edit operations. No edits are made in
this step — only the work surface is named and ranked.

### Step 6 — Graph-editor integration (apply refactor ops)

The graph-editor already consumes `graph.json` and applies patch operations
anchored to `def.lo` / `def.hi` byte offsets. Auto-refactor adds three new
operation types:

- `SplitFn` — given a fn path and a split point (phase boundary basic block),
  produce two new fn definitions with the call from original delegating to both.
- `MergeFns` — given a list of equivalent fn paths, produce one canonical fn
  and replace all others with thin wrappers or direct renames.
- `ExtractTrait` — given a set of fn paths across two types with matching
  signatures, produce a trait definition and impl blocks for both types.

Each operation is verified by re-running `cargo check` through the wrapper
and confirming that the resulting `graph.json` has lower `rho_mut`, lower
average fan-out, and no new panic surfaces.

---

## Success Criteria

$$
\rho_{\text{mut}}^{\text{after}} < \rho_{\text{mut}}^{\text{before}} = 0.808
$$

$$
\overline{\text{fan-out}}_{\text{mut}}^{\text{after}} < \overline{\text{fan-out}}_{\text{mut}}^{\text{before}}
$$

$$
|\mathcal{P}|^{\text{after}} \leq |\mathcal{P}|^{\text{before}} = 169 \quad \text{(no new panic surfaces)}
$$

$$
\text{merge\_count} + \text{split\_count} + \text{canonicalize\_count} > 0
$$

The last criterion is the most important: the system must produce at least one
verified structural improvement, not just a report.

---

## Non-Goals

- Do not generate speculative rewrites that are not anchored to a graph signal.
- Do not merge fns whose callee similarity is structural coincidence rather
  than semantic equivalence — the `"provider"` edge distinguishes these.
- Do not split fns whose fan-out comes entirely from a single repeated callee
  (e.g. `kernel::mix` — that is a hotspot, not a complexity problem).
- Do not create new public API surface without a corresponding invariant.
- Do not auto-refactor across crate boundaries in this phase.
