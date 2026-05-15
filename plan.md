# Plan File Instructions

Use this file as the source of truth for selecting and executing the next unit of project work.

## Required sections

Keep the operational portion of the file in this order when practical:

1. `## Goal` — the intended outcome or product state.
2. `## Current State` — concise facts about what is already true.
3. `## Active Priorities` — the ordered checklist used for execution.
4. `## Suggested Validation` — commands or checks that prove work is complete.
5. Additional design notes or background sections after the operational sections.

## Active Priorities format

Use one checklist item per independently executable unit of work:

```md
N. [ ] `<path or component>`: <specific action and expected result>.
   - Scope: <files, functions, records, tests, or artifacts allowed to change>.
   - Done when: <observable completion condition>.
   - Validation: `<targeted command or check>`.
```

Rules:

- Select the first unchecked implementation item matching `N. [ ]` under `## Active Priorities`.
- Implement exactly that item unless it is explicitly labeled as validation, evidence refresh, documentation, cleanup, or blocker handling.
- Keep each checklist item small enough for one execution turn.
- Avoid broad items such as “finish module,” “clean up project,” or “improve quality” unless they are decomposed into file-level or test-level tasks.
- Do not reorder, renumber, delete, or broadly rewrite checklist items during an implementation turn unless the selected item is explicitly a planning or cleanup item.

## Checkbox semantics

- `[ ]` means the item remains selectable by the next execution turn.
- `[x]` means the item’s named work exists and its targeted completion check has passed.
- Full-project validation, release gates, evidence refreshes, or deployment checks should be separate checklist items.
- Do not keep completed implementation work unchecked only because a later full-project gate is unavailable or blocked.
- Infrastructure, connector, network, permission, or tool failures are blockers; document them separately from source or test failures.

## Validation and commit rules

- After changing files, run the most specific validation for the selected item first.
- Then run any required broader validation listed in this file.
- Mark the implementation item `[x]` only when the source/artifact change exists and its targeted validation passes.
- Commit only when the required validation for the intended commit scope is green.
- If validation cannot run or cannot be made green, document the blocker, leave the relevant validation or blocker item unchecked, and do not claim completion for that gate.

## Delegation rule

- If multiple unchecked implementation items are independent, the primary agent should still make progress on the first item before delegating another.
- Delegated work must name the exact scope and the validation metric that proves completion.

---

# Canon Agent Plan

Current date: 2026-05-15.
Workspace: `/workspace/ai_sandbox/canon-mini-agent/prototype/ai`.

## Goal

Canon Agent is a deterministic, auditable, self-improving agent runtime. The
kernel owns correctness, state transitions, replay, receipts, and durable
evidence. Capabilities own observation, context, tools, LLM calls, planning,
judgment, verification, evaluation, learning, graph editing, and policy
promotion. The LLM proposes and explains; it does not govern the state machine
or approve itself.

The near-term goal is to turn `src/domain` from a documentation-only sketch into
real, testable Rust code while preserving the runtime boundary:

```text
objective or world signal
  -> domain signal/context/judgment/plan records
  -> deterministic score and risk envelope
  -> future capability bridge record
  -> typed capability execution
  -> receipt and replay verification
  -> eval score
  -> learning or policy promotion candidate
```

## Current State

- P0 validation baseline: complete.
- P1 validation evidence reporting: complete.
- P2 agent loop reliability: complete.
- P3 runtime and receipt correctness: complete for current scope.
- P4 graph source-of-truth integration: mostly complete for deterministic fixture/report evidence.
- P5 domain intelligence layer: active. `src/domain` contains compiled Rust modules for the shared contract/identity/scoring/risk/bridge surface, subdomain Rust modules, fixture contracts, full validation evidence, and graph refresh evidence for the current scope.

## Active Priorities

Planning reconnaissance refresh on 2026-05-15 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; confirmed Active Priorities items 122 through 124 are complete and that no unchecked executable item remained before this update; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json` with Python; inspected current `SCORE_REPORT.md`; inspected candidate source and tests in `src/capability/eval/record.rs`; and checked the working tree. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.93 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `6.9`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and currently contains 1,564 planned operations. Current planning skipped previously completed or stale high-ranked rows for `agent::cycle`, `agent::loop_driver`, `agent::objective`, `agent::prompt`, `agent::worker_client`, `api::server`, `api::transport`, `capability::judgment::record`, `capability::llm::ollama`, `capability::llm::openai`, and already-covered verification helper constructors. Current planning selected graph operation `fabb83c558fc5ecd`, covering `capability::eval::record::{dimension_hash, eval_payload_hash}`, as the next safe non-`root_validate` consolidation area. The safe subset is private eval scorecard hash folding only: keep all public record and receipt names and signatures, preserve `EvalRecord::submission(...)`, preserve `EvalScorecardReceipt::from_record(...)`, preserve receipt validity and NDJSON behavior, preserve dimension order versus score/threshold hash separation, and leave `root_validate` unchanged.

125. [x] `src/capability/eval/record.rs`: route `eval_payload_hash(...)` through a private ordered eval hash-fold helper shared with `dimension_hash(...)` without changing scorecard semantics.
   - Scope: `src/capability/eval/record.rs` only; allowed production functions are `eval_payload_hash(...)`, `dimension_hash(...)`, and at most one new private helper or enum adjacent to those definitions. Do not change `EvalRecord`, `EvalScorecardReceipt`, `dimension_order_hash(...)`, `dimension_score_hash(...)`, `dimension_id_hash(...)`, NDJSON encode/decode/load/append functions, evidence submission logic, public APIs, tests, or `root_validate` in this item.
   - Done when: `eval_payload_hash(...)` still includes record score, threshold used, ordered dimension id, dimension score, and dimension threshold; `dimension_order_hash(...)` still uses only dimension count and ordered dimension ids; `dimension_score_hash(...)` still uses dimension count, ordered dimension ids, dimension scores, and dimension thresholds; all three hashes remain non-zero through `h.max(1)`; and receipt validity, payload binding, order binding, score/threshold sensitivity, and empty-dimension behavior are unchanged.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

126. [x] `src/capability/eval/record.rs` test `eval_hash_fold_helper_preserves_payload_and_dimension_boundaries`: add focused regression coverage for the shared eval hash-fold boundary.
   - Scope: `src/capability/eval/record.rs` test module only; use the existing `passing_record()` helper and in-memory record variants. Do not change production code in this item.
   - Done when: the named test proves `eval_payload_hash(...)`, `dimension_order_hash(...)`, and `dimension_score_hash(...)` are all non-zero and pairwise distinct for the same record; proves payload hash changes when record score or threshold changes; proves order hash changes on dimension reordering but not on score/threshold-only changes; proves score hash changes on dimension score or threshold changes; proves `EvalScorecardReceipt::from_record(...)` stores the exact three helper outputs; and performs no file I/O, network I/O, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test eval_hash_fold_helper_preserves_payload_and_dimension_boundaries -- --test-threads=1`.

127. [x] `SCORE_REPORT.md`: after items 125 and 126 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.

Planning reconnaissance refresh on 2026-05-15 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; confirmed Active Priorities items 125 through 127 are complete and no unchecked executable item remained before this update; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json` with Python; inspected current `SCORE_REPORT.md`; inspected candidate source and tests in `src/capability/memory/store.rs`; and checked the working tree. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.93 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `6.9`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and currently contains 1,563 planned operations. Current planning skipped previously completed or stale high-ranked rows for `agent::cycle`, `agent::loop_driver`, `agent::objective`, `agent::prompt`, `agent::worker_client`, `api::server`, `api::transport`, `capability::judgment::record`, `capability::llm::ollama`, `capability::llm::openai`, `capability::llm::record`, `capability::llm::transport`, and `capability::eval::record`. Current planning selected graph operation `c57778031a3fd233`, covering `capability::memory::store::{aggregate_index_hash, aggregate_memory_hash}`, as the next safe non-`root_validate` consolidation area. The safe subset is private memory hash folding only: keep `MemoryIndex::fingerprint()`, `MemoryIndex::lookup(...)`, `MemoryLookupRecord::is_valid()`, `MemoryLookupReceipt::from_lookup(...)`, and receipt validation behavior unchanged; preserve distinct query-result aggregate and full-index fingerprint domains; preserve fact field order; and leave `root_validate` unchanged.

128. [x] `src/capability/memory/store.rs`: route `aggregate_memory_hash(...)` and `aggregate_index_hash(...)` through one private ordered memory-fact hash-fold helper while preserving distinct aggregate domains.
   - Scope: `src/capability/memory/store.rs` only; allowed production functions are `aggregate_memory_hash(...)`, `aggregate_index_hash(...)`, and at most one new private helper adjacent to those definitions. Do not change `MemoryFact`, `MemoryLookupRecord`, `MemoryLookupReceipt`, `MemoryIndex`, insertion/sorting/lookup behavior, public APIs, tests, or `root_validate` in this item.
   - Done when: `aggregate_memory_hash(query_hash, matches)` still seeds with `0x510e527fade682d1u64 ^ query_hash`; `aggregate_index_hash(facts)` still seeds with `0x4d45_4d49_4e44_4558u64`; both still fold each fact as key, value hash, weight, and source sequence in that order; both still return `h.max(1)`; and lookup receipt validity, index fingerprinting, duplicate rejection, and result ordering are unchanged.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

129. [x] `src/capability/memory/store.rs` test `memory_hash_fold_helper_preserves_lookup_and_index_boundaries`: add focused regression coverage for the shared memory hash-fold boundary.
   - Scope: `src/capability/memory/store.rs` test module only; use in-memory `MemoryIndex` and `MemoryFact` values. Do not change production code in this item.
   - Done when: the named test proves `aggregate_memory_hash(...)` and `aggregate_index_hash(...)` are non-zero and distinct for the same matching facts; proves changing the query hash changes only the lookup aggregate, not the index fingerprint; proves changing fact key, value hash, weight, or source sequence changes the affected aggregate; proves reordering facts changes the raw aggregate helper output while `MemoryIndex` fingerprint remains sorted-order deterministic after insertion; and proves `MemoryLookupReceipt::from_lookup(...)` stores the exact aggregate and index fingerprint expected from the helpers.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test memory_hash_fold_helper_preserves_lookup_and_index_boundaries -- --test-threads=1`.

130. [x] `SCORE_REPORT.md`: after items 128 and 129 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.


## Additional Validation Notes

- Targeted validation for the selected checklist item, as listed under `## Active Priorities`.
- Required broader gate when available: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`.
- Fixture validation when fixture or fixture-test items are selected: `python3 -m unittest tests/test_domain_fixture_contract.py`.

## Domain Implementation Target

The domain directory currently contains specifications plus the first compiled Rust modules:

```text
src/domain/
  README.md
  bridge.rs
  business.md
  contracts.rs
  contracts.md
  finance.md
  global_intelligence.md
  identity.rs
  integration.md
  mod.rs
  risk.rs
  roadmap.md
  scoring.md
  scoring.rs
  trading.md
```

Convert this into a compiled, still-safe module surface:

```text
src/domain/
  mod.rs                    # public module surface and boundary docs
  contracts.rs              # shared records, ids, enums, schema versions
  identity.rs               # deterministic hash/id helpers
  scoring.rs                # bounded score math and verdict thresholds
  risk.rs                   # risk envelopes and live-effect constraints
  bridge.rs                 # typed future capability mapping descriptors only
  global_intelligence.rs    # global signal classification records
  business.rs               # business opportunity and workflow records
  finance.rs                # finance hypothesis and risk records
  trading.rs                # sandbox-only simulation records
```

The existing `.md` files remain as design notes until the Rust records supersede
them. Do not delete them in the first implementation pass.

## P5 - Domain Intelligence Implementation

P5 starts with pure, deterministic Rust records. Domain code may interpret,
score, classify, and describe future capability requirements. It must not mutate
`State`, `Packet`, `GateSet`, runtime events, command ledgers, or TLog directly.

### P5.1 - Compile the Domain Module

Required work:

- Add `pub mod domain;` to `src/lib.rs` only after records and tests compile.
- Replace `src/domain/mod.rs` sketch-only content with declared Rust modules.
- Keep all domain modules free of I/O, process spawning, network calls, and runtime writes.
- Use existing dependencies only: `serde` and `serde_json` are available.
- Add local unit tests under `src/domain` or integration tests under `tests/`.

Done when:

- `cargo test --all-targets` compiles with `domain` wired into the crate.
- A test proves domain modules do not expose kernel/runtime mutation APIs.
- The markdown specs still explain intent, but compiled Rust owns the contract.

### P5.2 - Shared Contracts

Implement these first in `src/domain/contracts.rs`:

```text
DomainId
DomainSchemaVersion
DomainSourceKind
DomainHorizon
DomainSignalClass
DomainRiskClass
DomainVerdict
DomainPlanKind
DomainLiveEffectLevel

DomainSignal
DomainContext
DomainJudgment
DomainPlan
DomainRiskEnvelope
DomainEval
DomainPromotionCandidate
```

Rules:

- Records are plain data with `Clone`, `Debug`, `PartialEq`, `Eq` where practical.
- Records derive `Serialize` and `Deserialize`.
- Floating-point fields are avoided in core records. Use bounded integer scores,
  preferably `u16` in the range `0..=1000`, to make equality and hashing stable.
- Each record stores its schema version explicitly.
- Each record has enough provenance and hash fields to be replay-auditable.
- Trading records default to sandbox-only and no live execution.

Done when:

- Every conceptual record family from `contracts.md` has a Rust struct or enum.
- Constructors validate score ranges and basic invariants.
- Invalid score ranges, missing hashes, and unsafe trading flags fail tests.

### P5.3 - Deterministic Identity

Implement in `src/domain/identity.rs`:

```text
DomainHash
DomainHashInput
domain_hash_json(record)
domain_hash_parts(parts)
canonical_json_bytes(record)
```

Rules:

- Hash identity must be deterministic across runs.
- Hash input must include schema version and record kind.
- Hash input must not include non-deterministic fields unless they are explicit record data.
- Prefer a small internal stable hash helper if the crate already has one; otherwise use a
  deterministic, documented local hash implementation without adding dependencies.

Done when:

- Serializing and hashing the same record twice produces the same identity.
- Changing one material field changes the identity.
- Schema-version changes alter identity.

### P5.4 - Scoring and Verdicts

Implement in `src/domain/scoring.rs`:

```text
BoundedScore         # 0..=1000
ScoreInputs
ScoreBreakdown
domain_value_score
actionability_score
source_quality_score
confidence_score
uncertainty_score
risk_score
promotion_score
verdict_for_scores
```

Use integer math with saturation. Preserve the intent from `scoring.md`:

```text
domain_value = opportunity * confidence * policy_fit * verification_readiness
             - risk * uncertainty * staleness_penalty

actionability = domain_value * source_quality * context_quality
```

Rules:

- Missing evidence lowers confidence and raises uncertainty.
- Contradiction raises uncertainty and may force `Research` or `Block`.
- Risk envelope violations force `Block`.
- High actionability still requires capability receipts later; domain code cannot execute.

Done when:

- Fixture tests cover `Ignore`, `Watch`, `Research`, `ActBusiness`, `ActFinanceResearch`,
  `SimulateTrading`, and `Block`.
- Scoring is deterministic and does not use floats.
- Boundary values at `0`, threshold edges, and `1000` are tested.

### P5.5 - Risk Envelope

Implement in `src/domain/risk.rs`:

```text
DomainRiskEnvelope
RiskEnvelopeViolation
check_risk_envelope(plan, envelope)
```

Rules:

- Live-effect level must be explicit.
- Domain plans that imply external effects require verification.
- Trading plans must be sandbox-only.
- Finance plans may describe research or allocation hypotheses, but execution remains blocked.
- Business plans may describe workflows and value hypotheses, but tool execution must still be routed through capabilities.

Done when:

- A live trading request always produces a blocking violation.
- A finance execution plan is blocked unless a future explicit policy changes the contract.
- A business workflow plan can pass only when verification and rollback/invalidation fields are present.

### P5.6 - Subdomain Records

Implement subdomain modules after shared contracts compile:

```text
src/domain/global_intelligence.rs
  SignalClass
  GlobalSignalProfile
  stale_for_horizon(...)
  actionability_hint(...)

src/domain/business.rs
  BusinessOpportunity
  WorkflowAutomationCandidate
  CustomerFeedbackSignal
  monetization_score(...)

src/domain/finance.rs
  AssetUniverse
  FinanceHypothesis
  FinanceRiskDimensions
  finance_research_allowed(...)

src/domain/trading.rs
  TradingSimulationPlan
  BacktestReceiptRequirements
  TradingRiskLimit
  enforce_sandbox_only(...)
```

Rules:

- Business is the first production-oriented domain.
- Finance remains research and allocation intelligence.
- Trading remains sandbox-only.
- Subdomain modules depend on shared domain contracts, not on kernel/runtime internals.

Done when:

- Each subdomain has at least one fixture-driven test.
- Business monetization scoring is implemented with deterministic bounded scores.
- Finance hypotheses explicitly carry `execution_allowed = false`.
- Trading simulation plans cannot be constructed with live execution enabled.

### P5.7 - Fixtures

Add deterministic fixtures before bridge behavior:

```text
tests/fixtures/domain/global_signal_macro.json
tests/fixtures/domain/business_workflow_opportunity.json
tests/fixtures/domain/finance_hypothesis_research.json
tests/fixtures/domain/trading_simulation_sandbox.json
tests/fixtures/domain/trading_live_blocked.json
```

Each fixture must include:

- schema version,
- domain id,
- source/provenance hash,
- horizon,
- score inputs,
- expected verdict,
- expected risk-envelope result,
- expected bridge target descriptor.

Done when:

- Fixtures deserialize into Rust records.
- Expected identities and verdicts are asserted in tests.
- Fixture failures produce clear assertion messages.

### P5.8 - Capability Bridge Descriptor, Not Execution

Implement `src/domain/bridge.rs` as descriptors only:

```text
DomainBridgeTarget
DomainBridgeDescriptor
bridge_target_for_signal(...)
bridge_target_for_context(...)
bridge_target_for_judgment(...)
bridge_target_for_plan(...)
bridge_target_for_eval(...)
```

Bridge targets map to future capability families:

```text
DomainSignal    -> ObservationRecord
DomainContext   -> ContextRecord / MemoryLookupRecord
DomainJudgment  -> JudgmentRecord / PolicyJudgmentRecord
DomainPlan      -> PlanRecord
Tool action     -> ToolExecutionRecord
Proof check     -> VerificationRecord
Result score    -> EvalRecord
Repeatable win  -> PolicyPromotion candidate
```

Rules:

- The bridge returns typed descriptors and required receipt families only.
- It does not submit commands.
- It does not call API routes.
- It does not append evidence.
- It does not mutate runtime state.

Done when:

- Tests prove bridge outputs are descriptors, not effects.
- Every domain record family has a future capability target.
- Trading execution targets remain absent or explicitly blocked.

## P4 - Graph Editing

Graph editing is the receipt-backed source mutation path. Keep it separate from
domain implementation unless a domain plan later describes graph-edit intent
through a capability bridge descriptor.

Required flow:

```text
graph.json
  -> agent selects target nodes
  -> GraphMutationOp set
  -> graph-editor patch generation
  -> patch application
  -> wrapper re-capture
  -> old/new graph diff
  -> GraphMutationReceipt
  -> TLog evidence
```

Next work:

- Add a reusable Python graph-analysis script or command that reads `state/rustc/ai/graph.json`, summarizes schema version, graph hash, node kinds, edge relations, intent labels, and candidate mutation targets.
- Prove one end-to-end agent-driven graph edit against a small fixture.
- Keep stale-op, overlap, receipt hash, and re-capture checks mandatory.
- Admit graph mutation receipts through the same evidence path as other capabilities.
- Keep `canon-rustc-v3`, `graph-editor`, and root runtime boundaries documented and tested.
- Do not loosen graph safety guards to make an edit easier.

Done when:

- Python graph analysis is part of the normal graph-edit planning workflow.
- A graph edit can be planned, patched, re-captured, verified, and recorded as evidence.
- The receipt proves the intended graph change landed.
- Failed, stale, or partial mutations produce explicit failure receipts.

## Self-Modification

Self-modification belongs inside verified evolution, after capability planning and
before policy promotion. It is not a kernel feature and not a domain shortcut.

Allowed path:

```text
LLM/capability proposes code change
  -> plan identifies files, graph targets, risks, and acceptance tests
  -> graph edit or normal patch runs in sandbox
  -> tests/evaluator score the result
  -> receipts enter TLog
  -> verified winners become learning data
  -> repeated verified patterns become policy candidates
```

Rules:

- The LLM may propose, never approve.
- External evaluator evidence is required before learning.
- Kernel invariants cannot be bypassed for self-modification.
- Policy promotion requires replay-verifiable support, not a single successful run.

## Product Direction

Primary path:

```text
AI global intelligence
  -> business workflow automation
  -> verified cashflow/value evidence
  -> finance intelligence
  -> capital allocation research
```

Secondary path:

```text
trading sandbox
  -> prediction/risk/eval discipline
  -> paper simulation only
```

Trading is not the first production business path.

## Non-Goals

- Do not redesign the kernel.
- Do not use domain code to bypass the capability registry.
- Do not let domain code mutate `State`, `Packet`, `GateSet`, runtime events, command ledgers, or TLog.
- Do not add live trading execution.
- Do not add external dependencies for the first domain implementation pass unless absolutely necessary.
- Do not treat unconfigured wrapper telemetry as required evidence.
- Do not depend on external model/provider/router availability for baseline correctness.
- Do not commit generated logs, target output, runtime archives, tokens, SSE chunks, or local session artifacts.

## Execution Rules

- Inspect `git status --short` before editing.
- Keep implementation changes scoped to the active priority.
- Stage explicit paths only.
- Update `score.md` only when validation evidence or scores change.
- Prefer focused contract tests and deterministic fixtures over long live-service validation.
- Commit docs/planning changes separately from implementation changes when committing.


Current planning turn selected graph operation `3531e5dc63009037`, covering `agent::cycle::{evidence_u64_value, gate_id_u64}`, as the next safe non-`root_validate` consolidation area. The safe subset is private typed route-table lookup delegation only: keep both wrapper names and signatures, preserve the distinct `GATE_ID_ROUTES` and `EVIDENCE_U64_ROUTES` tables, preserve unknown-input `None` behavior, preserve submit-evidence hash vectors, command-envelope JSON, receipt/evidence routing, runtime behavior, and all recovery-route logic. Do not touch `root_validate`, `src/agent/loop_driver.rs`, `compute_evidence_contract_hash(...)`, `compute_submit_evidence_command_hash(...)`, `compute_envelope_hash(...)`, or the already-reconciled recovery route helper.

77. [x] `src/agent/cycle.rs`: route `gate_id_u64(...)` and `evidence_u64_value(...)` through one private typed route-table lookup helper while preserving distinct gate and evidence domains.
   - Scope: `src/agent/cycle.rs` only; allowed functions are `gate_id_u64(...)`, `evidence_u64_value(...)`, `lookup_u64_route(...)`, `U64Route`, and at most one new private helper or enum adjacent to those definitions. Do not change `GATE_ID_ROUTES`, `EVIDENCE_U64_ROUTES`, `EFFECT_ROUTES`, `effect_for_gate_evidence(...)`, submit-evidence hash functions, `compute_structural_payload_hash(...)`, recovery-route functions, runtime command submission, receipt construction, or tests in this item.
   - Done when: `gate_id_u64(gate)` still looks up only `GATE_ID_ROUTES`; `evidence_u64_value(evidence)` still looks up only `EVIDENCE_U64_ROUTES`; unknown names still return `None`; both wrappers delegate through the same private typed helper boundary; and no submit-evidence packet, envelope, receipt, or runtime behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

78. [x] `src/agent/cycle.rs` test `u64_route_lookup_helpers_preserve_gate_and_evidence_boundaries`: add focused regression coverage for the shared typed route-table lookup helper.
   - Scope: `src/agent/cycle.rs` test module only; use existing private test access to `gate_id_u64(...)`, `evidence_u64_value(...)`, `effect_for_gate_evidence(...)`, and existing submit-evidence hash helper patterns. Do not change production code in this item.
   - Done when: the named test asserts representative gate names map to their existing numeric ids, representative evidence names map to their existing numeric values, gate names are not accepted by `evidence_u64_value(...)`, evidence names are not accepted by `gate_id_u64(...)`, unknown values return `None` for both wrappers, and a representative `effect_for_gate_evidence(...)` route remains unchanged. The test must not perform network I/O, filesystem I/O outside normal cargo test execution, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test u64_route_lookup_helpers_preserve_gate_and_evidence_boundaries -- --test-threads=1`.

79. [x] `SCORE_REPORT.md`: after items 77 and 78 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.

80. [x] `src/capability/llm/ollama.rs`: route `OllamaJudgmentProofEvent::{expected_proof_hash, verifier_context_hash}` through one private ordered hash-fold helper while preserving distinct Ollama proof-event hash domains.
   - Scope: `src/capability/llm/ollama.rs` only; allowed functions are `OllamaJudgmentProofEvent::expected_proof_hash`, `OllamaJudgmentProofEvent::verifier_context_hash`, and one private helper near the existing `fold_ordered_ollama_hash(...)`. Do not change receipt finalization, proof binding, replay verification, NDJSON encode/decode/load functions, HTTP/client behavior, OpenAI code, Ollama request execution, tests, or `root_validate`.
   - Done when: both public methods keep their names and signatures, keep distinct proof versus verifier-context seed constants, keep exact current field order and boolean casts, preserve non-zero `max(1)` hash behavior through the existing fold primitive, and share only the private field-folding path.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

81. [x] `src/capability/llm/ollama.rs` test `ollama_proof_event_hash_helpers_preserve_distinct_domains`: add focused regression coverage for the shared `OllamaJudgmentProofEvent` proof/verifier hash helper.
   - Scope: `src/capability/llm/ollama.rs` test module only; use existing `valid_test_receipt()` and `OllamaJudgmentProofEvent::finalize_receipt(...)` test helpers. Do not change production code in this item.
   - Done when: the named test proves valid proof-event construction, non-zero distinct `expected_proof_hash()` and `verifier_context_hash()` values, proof hash binding to the finalized event, canonical proof projection using the verifier-context hash, proof-only tampering rejection, and verifier-context-only tampering changing the verifier hash without changing the bound proof hash. The test must not perform network I/O, filesystem I/O outside normal cargo test execution, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test ollama_proof_event_hash_helpers_preserve_distinct_domains -- --test-threads=1`.

82. [x] `SCORE_REPORT.md`: after items 80 and 81 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.

Current planning turn selected graph operation `9225db29fcc89658`, covering `api::server::{decode_mcp_call_request, decode_mcp_call_receipt}`, as the next safe non-`root_validate` consolidation area. The safe subset is private DTO decode and registry-policy validation helper delegation only: keep both public decoder names and signatures, preserve request admissibility checks, preserve receipt contract checks, preserve `ToolEffectKind::Process` effect-kind validation, preserve `CapabilityId::Tooling`, preserve all request/receipt field mappings, preserve error classification between `InvalidPayload` and `InvalidCommand`, and do not touch route handlers, API DTO shapes, sandbox process receipt decoding, MCP execution, capability registry semantics, TLog/runtime code, network behavior, or `root_validate`.

83. [x] `src/api/server.rs`: route `decode_mcp_call_request(...)` and `decode_mcp_call_receipt(...)` through private DTO/registry validation helpers while preserving distinct request and receipt contracts.
   - Scope: `src/api/server.rs` only; allowed functions are `decode_mcp_call_request(...)`, `decode_mcp_call_receipt(...)`, `McpCallRequestDto`, `McpCallReceiptDto`, and at most two new private helpers adjacent to the existing decoders. Do not change API route handlers, `decode_sandbox_process_receipt(...)`, `sandbox_process_receipt_from_dto(...)`, process/MCP execution, DTO field names, serde behavior, capability registry construction, TLog/runtime code, tests, or `root_validate`.
   - Done when: both decoders still parse their exact DTOs with `serde_json::from_value(...)` and map malformed DTOs to `ServerError::InvalidPayload`; `decode_mcp_call_request(...)` still constructs `McpCallRequest` with `CapabilityId::Tooling`, preserves all request fields, rejects registry-policy mismatch or failed `is_admissible()` with `ServerError::InvalidCommand`, and returns the unchanged request otherwise; `decode_mcp_call_receipt(...)` still accepts only `effect_kind == 2` as `ToolEffectKind::Process`, preserves all receipt/effect fields, rejects registry-policy mismatch or failed `is_contract_valid()` with `ServerError::InvalidCommand`, and returns the unchanged receipt otherwise; and shared mechanics are confined to private helper boundaries.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

84. [x] `src/api/server.rs` test `mcp_call_decoders_preserve_request_receipt_boundaries`: add focused regression coverage for shared MCP-call decoder helpers.
   - Scope: `src/api/server.rs` test module only; use existing API DTO and receipt/request helper patterns. Do not change production code in this item.
   - Done when: the named test proves a valid MCP call request decodes to `CapabilityId::Tooling` with the expected registry policy, worker URL, tool name, args, timeout, and output-limit fields; a valid MCP call receipt decodes with `ToolEffectKind::Process`, expected request/registry/worker/tool/args/effect/response/status/timing/hash fields, and valid contract behavior; malformed DTO payloads return `ServerError::InvalidPayload`; registry-policy mismatches return `ServerError::InvalidCommand`; and invalid receipt `effect_kind` returns `ServerError::InvalidPayload`. The test must not perform network I/O, filesystem I/O outside normal cargo test execution, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test mcp_call_decoders_preserve_request_receipt_boundaries -- --test-threads=1`.

85. [x] `SCORE_REPORT.md`: after items 83 and 84 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.


Current planning turn selected graph operation `9c529c749f8588e3`, covering `capability::judgment::record::{policy_decision_id, policy_rationale_hash}`, as the next safe non-`root_validate` consolidation area. The safe subset is private ordered judgment-hash folding only: keep both function names and signatures, preserve distinct decision/rationale domain seeds, preserve all current validity guards, preserve field order and non-zero `max(1)` behavior, preserve `PolicyJudgmentRecord::from_context_policy_lookup_receipt(...)`, record hash behavior, submission behavior, policy-reuse receipt behavior, and all policy-store semantics. Do not touch `root_validate`, policy promotion, policy lookup receipts, policy-reuse ledger/scale-trace helpers already consolidated, API routes, LLM clients, runtime state, TLog code, or filesystem/network behavior.

86. [x] `src/capability/judgment/record.rs`: route `policy_decision_id(...)` and `policy_rationale_hash(...)` through one private ordered judgment hash-fold helper while preserving distinct decision and rationale domains.
   - Scope: `src/capability/judgment/record.rs` only; allowed functions are `policy_decision_id(...)`, `policy_rationale_hash(...)`, and at most one new private helper adjacent to those functions. Do not change `PolicyJudgmentRecord::from_context_policy_lookup_receipt(...)`, `policy_judgment_record_hash(...)`, `policy_reuse_record_set_hash(...)`, policy-reuse receipt hash helpers, policy store code, tests, API code, LLM code, runtime/TLog code, or `root_validate`.
   - Done when: both hash functions keep their exact names, arguments, and return type; `policy_decision_id(...)` still rejects invalid context or zero policy version/hash/feedback/lookup receipt values; `policy_rationale_hash(...)` still rejects invalid context or zero decision/feedback/lookup receipt values; decision hashing still uses seed `0x4a55_4447_504f_4c48` and fields `context.objective_id`, `context.context_hash`, `context.memory_aggregate_hash`, `policy_version`, `policy_hash`, `policy_feedback_hash`, `policy_lookup_receipt_hash`; rationale hashing still uses seed `0x5241_544c_504f_4c48` and fields `context.observation_hash`, `context.memory_receipt_hash`, `decision_id`, `policy_feedback_hash`, `policy_lookup_receipt_hash`; both still return `h.max(1)` through the shared private helper; and no judgment record, submission, policy reuse, or policy-store behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

87. [x] `src/capability/judgment/record.rs` test `policy_judgment_hash_helpers_preserve_decision_rationale_boundaries`: add focused regression coverage for the shared decision/rationale hash helper.
   - Scope: `src/capability/judgment/record.rs` test module only; use existing policy judgment test helpers and private access to `policy_decision_id(...)`, `policy_rationale_hash(...)`, `PolicyJudgmentRecord::from_context_policy_lookup_receipt(...)`, and `policy_judgment_record_hash(...)`. Do not change production code in this item.
   - Done when: the named test proves valid policy judgment construction still yields non-zero distinct decision and rationale hashes, direct helper calls match the record fields, invalid/zero inputs still return `0` for each helper, decision-only material changes alter only the expected decision-dependent boundary, rationale-only material changes alter the rationale boundary, and `policy_judgment_record_hash(...)` still binds the resulting record. The test must not perform network I/O, filesystem I/O outside normal cargo test execution, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test policy_judgment_hash_helpers_preserve_decision_rationale_boundaries -- --test-threads=1`.

88. [x] `SCORE_REPORT.md`: after items 86 and 87 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.


Planning reconnaissance refresh on 2026-05-15 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; confirmed Active Priorities are complete through item 88 with no first incomplete executable item; inspected the current `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json` with Python; inspected candidate source and test surfaces in `src/capability/llm/transport.rs`; and checked the working tree through the project connector. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.92 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `6.9`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and currently contains 1,571 planned operations. Current planning selected graph operation `f26c507839cfaf73`, covering `capability::llm::transport::{request_identity_hash, retry_policy_hash}`, as the next safe non-root consolidation area. The safe subset is private ordered hash-fold helper delegation only: keep both public function names and signatures, preserve distinct caller-provided seeds, preserve field order for retry policy and request identity hashes, preserve non-zero `max(1)` behavior, and preserve provider config hashing, endpoint parsing, chat-completions path building, OpenAI/Ollama client behavior, receipt construction, proof construction, NDJSON behavior, retry-budget validation, network behavior, and `root_validate` untouched.

89. [x] `src/capability/llm/transport.rs`: route `retry_policy_hash(...)` and `request_identity_hash(...)` through one private ordered hash-fold helper while preserving distinct transport hash domains.
   - Scope: `src/capability/llm/transport.rs` only; allowed production functions are `retry_policy_hash(...)`, `request_identity_hash(...)`, and at most one new private helper adjacent to them. Do not change `parse_local_http_endpoint(...)`, `chat_completions_path(...)`, `provider_text_hash(...)`, `provider_config_hash(...)`, LLM provider modules, receipt/proof code, retry-budget validation, NDJSON code, tests, network code, or `root_validate`.
   - Done when: both public functions keep their exact names, arguments, visibility, and return type; `retry_policy_hash(seed, timeout_ms, max_retries, attempt_budget)` still folds `timeout_ms`, `max_retries as u64`, and `attempt_budget as u64` after the caller-provided seed; `request_identity_hash(seed, provider_hash, base_url_hash, model_id, request_hash)` still folds `provider_hash`, `base_url_hash`, `model_id`, and `request_hash` after the caller-provided seed; both functions delegate to the same private ordered hash-fold helper; and both still return `h.max(1)` with no provider, endpoint, retry, receipt, proof, or network behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

90. [x] `src/capability/llm/transport.rs` test `transport_hash_helpers_preserve_retry_and_request_identity_boundaries`: add focused regression coverage for the shared transport hash helper.
   - Scope: `src/capability/llm/transport.rs` test module only; use existing public `retry_policy_hash(...)`, `request_identity_hash(...)`, and `provider_config_hash(...)`. Do not change production code in this item.
   - Done when: the named test proves retry-policy hashes are non-zero, deterministic, and sensitive to timeout, max-retry, attempt-budget, and seed changes; request-identity hashes are non-zero, deterministic, and sensitive to provider hash, base-url hash, model id, request hash, and seed changes; retry-policy and request-identity hashes remain distinct for representative inputs; and the test performs no network I/O, filesystem I/O outside normal cargo test execution, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test transport_hash_helpers_preserve_retry_and_request_identity_boundaries -- --test-threads=1`.

91. [x] `SCORE_REPORT.md`: after items 89 and 90 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.


Planning reconnaissance refresh on 2026-05-15 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; confirmed Active Priorities items 95 through 97 are complete and that there was no first incomplete executable item before this update; inspected current `../state/rustc/auto-refactor/*.graph-editor-plan.json`; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json` with Python; inspected candidate source and test surfaces in `src/agent/cycle.rs`, `src/agent/prompt.rs`, `src/agent/objective.rs`, `src/agent/worker_client.rs`, `src/capability/eval/record.rs`, and `src/api/transport.rs`; and checked the working tree through the project connector. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.93 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `6.9`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and currently contains 1,570 planned operations. Current planning first inspected the top-ranked `agent::cycle` hash/route entries, the `agent::prompt` certification prompt cluster, and the `agent::{objective,worker_client}` constructor/setter entries, but rejected those as stale/no-op because the current source already routes them through helper boundaries. Current planning selected graph operation `api::transport::{api_transport_receipt_hash,transport_frame_hash}` from the selected plan as the next safe non-`root_validate` consolidation area. The safe subset is private ordered FNV-style hash-fold delegation only: keep both private function names, constants, seeds, field order, non-zero `max(1)` behavior, receipt validation, frame validation, replay verification, NDJSON format, API route behavior, TLog behavior, filesystem behavior, and `root_validate` unchanged.

98. [x] `src/api/transport.rs`: route `transport_frame_hash(...)` and `api_transport_receipt_hash(...)` through one private ordered FNV-style hash-fold helper while preserving distinct API transport hash domains.
   - Scope: `src/api/transport.rs` only; allowed production functions are `transport_frame_hash(...)`, `api_transport_receipt_hash(...)`, and at most one new private helper adjacent to them. Do not change `ApiTransportFrame`, `ApiTransportReceipt`, `ApiTransportLedger`, `ApiTransportSession`, `handle_transport_frame_once(...)`, receipt-chain verification, NDJSON encode/decode/load/append functions, API server code, command ledger code, TLog code, tests, filesystem behavior, network behavior, or `root_validate`.
   - Done when: `transport_frame_hash(...)` still starts from seed `0x7472_616e_7370_6f72u64`, folds `schema_version`, `route_id`, `request_id`, `envelope.command_id`, and `envelope.command_hash` in the existing order, and returns non-zero; `api_transport_receipt_hash(...)` still starts from seed `0x6170_695f_7472_6374u64`, folds `API_TRANSPORT_RECEIPT_SCHEMA_VERSION`, `API_TRANSPORT_RECEIPT_RECORD`, `request_id`, `payload_hash`, `command_id`, `command_hash`, and `event_hash` in the existing order, and returns non-zero; both hash functions delegate through the same private ordered fold helper; and no public API, receipt, replay, NDJSON, filesystem, or TLog behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

99. [x] `src/api/transport.rs` test `api_transport_hash_helpers_preserve_frame_and_receipt_boundaries`: add focused regression coverage for the shared API transport hash helper.
   - Scope: `src/api/transport.rs` test module only; use existing private access to `transport_frame_hash(...)`, `api_transport_receipt_hash(...)`, `ApiTransportFrame`, `ApiTransportReceipt`, and existing command-envelope test setup patterns. Do not change production code in this item.
   - Done when: the named test proves frame and receipt hashes are non-zero, deterministic, and distinct for representative inputs; frame hashes remain sensitive to schema version, route id, request id, command id, and command hash changes; receipt hashes remain sensitive to request id, payload hash, command id, command hash, and event hash changes; `ApiTransportFrame::is_contract_valid()` and `ApiTransportReceipt::is_contract_valid()` still bind those hashes; and the test performs no network I/O, environment mutation, subprocess spawning, or filesystem I/O outside normal cargo test execution.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test api_transport_hash_helpers_preserve_frame_and_receipt_boundaries -- --test-threads=1`.

100. [x] `SCORE_REPORT.md`: after items 98 and 99 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.

## Suggested Validation

For planning/doc-only changes:

```bash
TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test planning_contract -- --test-threads=1
```

For domain implementation work:

```bash
TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets
```

For domain fixture validation after fixtures exist:

```bash
python3 -m unittest tests/test_domain_fixture_contract.py
TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test domain_contract -- --test-threads=1
```

For graph-editing work:

```bash
python3 scripts/analyze_graph_json.py state/rustc/ai/graph.json
TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test graph_mutation_cli_contract -- --test-threads=1
python3 -m unittest tests/test_graph_workflow_fixture_validator.py
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
```
