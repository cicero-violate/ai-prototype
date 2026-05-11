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

Current date: 2026-05-10.
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
- P5 domain intelligence layer: active. `src/domain` contains compiled Rust modules for the shared contract/identity/scoring/risk/bridge surface, but subdomain Rust modules, fixture contracts, full validation, and graph refresh remain incomplete.

## Active Priorities

Reconnaissance on 2026-05-10 reconfirmed that `src/domain/identity.rs::DomainHashInput<'a>`, `src/domain/identity.rs::canonical_json_bytes(record)`, `src/domain/identity.rs::domain_hash_json(record)`, `src/domain/identity.rs::domain_hash_parts(parts)`, `src/domain/identity.rs::domain_hash_is_stable`, and `src/domain/identity.rs::domain_hash_changes_when_material_field_changes` are already implemented in local source and have passing targeted identity validation. Full-suite validation is tracked separately as the validation blocker item below. Execute turns should pick up exactly one unchecked implementation item at a time; the first incomplete implementation item remains item 8, `src/domain/identity.rs::domain_hash_changes_when_schema_version_changes`.

Current source inventory from `find src/domain -type f | sort`:

```text
src/domain/bridge.rs
src/domain/business.md
src/domain/contracts.md
src/domain/contracts.rs
src/domain/finance.md
src/domain/global_intelligence.md
src/domain/identity.rs
src/domain/integration.md
src/domain/mod.rs
src/domain/README.md
src/domain/risk.rs
src/domain/roadmap.md
src/domain/scoring.md
src/domain/scoring.rs
src/domain/trading.md
```

Graph evidence from `state/rustc/ai/graph.json`: schema version 16, graph hash `ab2202a8d8ec371b0c462aecc41e28d059920f53e2179dd40ebb6ebb3127fc33`, receipt hash `0a44845e35b656d3d31b481b5e43c456d92994ca8e3179b0fd0f6213954cd4df`, risk hash `c28e55e09a0259a6697a67971c16be23a02b59581c77e6973b2e2a44588e665e`, 4,473 node entries, 31,082 edges, and 2,976 intents. The top-level graph keys are `edges`, `intents`, `meta`, and `nodes`; the `nodes` object is keyed by canonical symbol name. Node-kind counts are `fn 2976`, `impl 1283`, `struct 159`, `enum 53`, `trait 1`, `ty_alias 1`. No compiled `domain::` nodes or `src/domain/*` file hits are present, so graph evidence does not yet prove the P5 domain surface.

Ordered execute-turn checklist, one file or one test per item. Implementation items are marked complete when the source change exists and targeted validation has passed; full-suite validation and commit gating are tracked by the validation item instead of keeping completed implementation work unchecked:

1. [x] `src/domain/identity.rs`: implement `DomainHash` newtype with `as_str(&self) -> &str`, `Display`, `AsRef<str>`, `TryFrom<String>`, and invariant validation for non-empty `domain:`-prefixed hash strings.
   - Local source contains this implementation and unit test `domain_hash_newtype_validates_prefix_and_non_empty_suffix`. Prior targeted validation passed, but full validation remains pending because connector HTTP 502 transport errors returned before Rust output.
2. [x] `src/domain/identity.rs`: implement `DomainHashInput<'a>` enum with `Json(&'a serde_json::Value)` and `Parts(&'a [&'a str])` variants, plus helper conversion paths used by the following identity hash helpers.
   - Implementation is present in local source with helper constructors, `From` conversions, and unit test `domain_hash_input_converts_json_and_parts`.
   - Targeted validation passed again on 2026-05-10 during implementation step 5: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test identity::tests -- --test-threads=1` ran 3 identity tests successfully.
   - Required full-suite validation remains blocked on 2026-05-10: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` returned connector HTTP 502 twice before Rust output in implementation step 5. The blocker is tracked in the validation item below; do not re-select this item for implementation.
3. [x] `src/domain/identity.rs`: implement `canonical_json_bytes(record: &serde_json::Value) -> Vec<u8>` with deterministic object-key ordering, stable array ordering, and explicit null/bool/number/string encoding.
   - Implementation is present in local source with recursive canonical encoding for null, bool, number, string, array, and object values; object entries are sorted by key before encoding.
   - Targeted validation passed on 2026-05-10 during implementation step 1: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test identity::tests -- --test-threads=1` ran 3 identity tests successfully after one connector HTTP 502 retry.
   - Required full-suite validation remains blocked on 2026-05-10: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` returned connector HTTP 502 twice before Rust output. The blocker is tracked in the validation item below; no commit was made in this turn.
4. [x] `src/domain/identity.rs`: implement `domain_hash_json(record: &serde_json::Value) -> DomainHash` using `canonical_json_bytes(record)` and the same deterministic hash namespace as `stable_domain_id(parts)`.
   - Implementation is present in local source and uses `canonical_json_bytes(record)` with the `domain:` hash namespace.
   - Targeted validation passed on 2026-05-10 during implementation step 1: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test identity::tests -- --test-threads=1` ran 3 identity tests successfully after one connector HTTP 502 retry.
   - Required full-suite validation remains blocked on 2026-05-10: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` returned connector HTTP 502 twice before Rust output. The blocker is tracked in the validation item below; no commit was made in this turn.
5. [x] `src/domain/identity.rs`: implement `domain_hash_parts(parts: &[&str]) -> DomainHash` and update `stable_domain_id(parts)` to delegate to `domain_hash_parts(parts).to_string()` without changing existing behavior.
   - Implementation is present in local source; `stable_domain_id(parts)` delegates to `domain_hash_parts(parts).to_string()` and preserves the previous deterministic FNV-style separator algorithm.
   - Targeted validation passed on 2026-05-10 during implementation step 2: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test identity::tests -- --test-threads=1` ran 3 identity tests successfully after one connector HTTP 502 retry.
   - Required full-suite validation remains blocked on 2026-05-10: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` returned connector HTTP 502 twice before Rust output. The blocker is tracked in the validation item below; no commit was made in this turn.
6. [x] `src/domain/identity.rs`: add unit test `domain_hash_is_stable`.
   - Implementation is present in local source and asserts repeatable `domain_hash_parts`, `stable_domain_id`, and `domain_hash_json` outputs for identical material inputs.
   - Targeted validation passed on 2026-05-10 during implementation step 4: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test identity::tests -- --test-threads=1` ran 4 identity tests successfully.
   - Required full-suite validation remains blocked on 2026-05-10: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` returned connector HTTP 502 twice before Rust output. The blocker is tracked in the validation item below; no commit was made in this turn.
7. [x] `src/domain/identity.rs`: add unit test `domain_hash_changes_when_material_field_changes`.
   - Implementation is present in local source and asserts `domain_hash_json` changes when the material `source_hash` field changes.
   - Targeted validation passed on 2026-05-10 during implementation step 5: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test identity::tests -- --test-threads=1` ran 5 identity tests successfully after one connector HTTP 502 retry.
   - Required full-suite validation remains blocked on 2026-05-10: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` returned connector HTTP 502 twice before Rust output. The blocker is tracked in the validation item below; no commit was made in this turn.
8. [ ] `src/domain/identity.rs`: add unit test `domain_hash_changes_when_schema_version_changes`.
9. [ ] `src/domain/scoring.rs`: implement `BoundedScore` with `new(value: u16)`, `get(&self) -> u16`, `zero()`, `max()`, and `saturating_weighted_average(...)` using integer-only math.
10. [ ] `src/domain/scoring.rs`: implement `ScoreInputs`, `ScoreBreakdown`, `source_quality_score(...)`, `confidence_score(...)`, `uncertainty_score(...)`, `risk_score(...)`, `promotion_score(...)`, and `verdict_for_scores(...)` using integer-only saturating math.
11. [ ] `src/domain/scoring.rs`: add unit test `verdict_ignore_thresholds`.
12. [ ] `src/domain/scoring.rs`: add unit test `verdict_watch_thresholds`.
13. [ ] `src/domain/scoring.rs`: add unit test `verdict_research_thresholds`.
14. [ ] `src/domain/scoring.rs`: add unit test `verdict_act_business_thresholds`.
15. [ ] `src/domain/scoring.rs`: add unit test `verdict_act_finance_research_thresholds`.
16. [ ] `src/domain/scoring.rs`: add unit test `verdict_simulate_trading_thresholds`.
17. [ ] `src/domain/scoring.rs`: add unit test `verdict_block_thresholds`.
18. [ ] `src/domain/risk.rs`: implement `RiskEnvelopeViolation` and `check_risk_envelope(plan: &DomainPlan, envelope: &DomainRiskEnvelope) -> Result<(), RiskEnvelopeViolation>` covering rollback requirements, invalidation requirements, risk bounds, and live-effect constraints.
19. [ ] `src/domain/risk.rs`: add unit test `risk_blocks_live_trading`.
20. [ ] `src/domain/risk.rs`: add unit test `risk_blocks_finance_execution`.
21. [ ] `src/domain/risk.rs`: add unit test `risk_allows_verified_business_plan_with_rollback_and_invalidation`.
22. [ ] `src/domain/bridge.rs`: implement `DomainBridgeDescriptor`, `bridge_target_for_signal(...)`, `bridge_target_for_context(...)`, `bridge_target_for_judgment(...)`, `bridge_target_for_plan(...)`, and `bridge_target_for_eval(...)` as descriptor-only functions with no state, command-ledger, runtime, network, process, or TLog mutation authority.
23. [ ] `src/domain/bridge.rs`: add unit test `bridge_maps_each_domain_record_family`.
24. [ ] `src/domain/bridge.rs`: add unit test `bridge_never_targets_live_trading_execution`.
25. [ ] `src/domain/global_intelligence.rs`: create Rust module with `SignalClass`, `GlobalSignalProfile`, `stale_for_horizon(...)`, and `actionability_hint(...)` depending only on shared domain contracts.
26. [ ] `src/domain/global_intelligence.rs`: add unit test `global_signal_profile_staleness_is_deterministic`.
27. [ ] `src/domain/global_intelligence.rs`: add unit test `global_signal_actionability_hint_is_deterministic`.
28. [ ] `src/domain/business.rs`: create Rust module with `BusinessOpportunity`, `WorkflowAutomationCandidate`, `CustomerFeedbackSignal`, and `monetization_score(...)`.
29. [ ] `src/domain/business.rs`: add unit test `business_monetization_score_is_deterministic`.
30. [ ] `src/domain/business.rs`: add unit test `business_monetization_score_is_bounded`.
31. [ ] `src/domain/finance.rs`: create Rust module with `AssetUniverse`, `FinanceHypothesis`, `FinanceRiskDimensions`, and `finance_research_allowed(...)`.
32. [ ] `src/domain/finance.rs`: add unit test `finance_hypothesis_execution_allowed_is_false`.
33. [ ] `src/domain/finance.rs`: add unit test `finance_research_plan_passes_research_only_risk_check`.
34. [ ] `src/domain/trading.rs`: create Rust module with `TradingSimulationPlan`, `BacktestReceiptRequirements`, `TradingRiskLimit`, and `enforce_sandbox_only(...)`.
35. [ ] `src/domain/trading.rs`: add unit test `trading_simulation_plan_rejects_live_execution`.
36. [ ] `src/domain/mod.rs`: declare `global_intelligence`, `business`, `finance`, and `trading` after their Rust files compile, and keep module docs explicit that domain code has no I/O, process, network, runtime, command-ledger, or TLog mutation authority.
37. [ ] `tests/domain_contract.rs`: add integration test `domain_records_deserialize_from_json`.
38. [ ] `tests/domain_contract.rs`: add integration test `domain_identity_is_deterministic`.
39. [ ] `tests/domain_contract.rs`: add integration test `domain_verdicts_are_deterministic`.
40. [ ] `tests/domain_contract.rs`: add integration test `domain_surface_exposes_no_runtime_mutation_api`.
41. [ ] `tests/fixtures/domain/global_signal_macro.json`: add fixture data with schema version, domain id, source/provenance hash, horizon, score inputs, expected verdict, expected risk result, and expected bridge target descriptor.
42. [ ] `tests/fixtures/domain/business_workflow_opportunity.json`: add fixture data for a verified business workflow opportunity.
43. [ ] `tests/fixtures/domain/finance_hypothesis_research.json`: add fixture data for a research-only finance hypothesis.
44. [ ] `tests/fixtures/domain/trading_simulation_sandbox.json`: add fixture data for a sandbox-only trading simulation plan.
45. [ ] `tests/fixtures/domain/trading_live_blocked.json`: add fixture data for a live trading request that must block.
46. [ ] `tests/test_domain_fixture_contract.py`: add fixture validation covering all domain fixture files and clear assertion failures for missing required fields.
47. [ ] Validation blocker / full-suite gate: run `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` and record the result in `score.md`. This remains unchecked until full-suite Rust output is available and green; connector HTTP 502 transport failures should be documented here without causing completed implementation items to remain unchecked. Latest attempt on 2026-05-10 after adding `domain_hash_changes_when_material_field_changes` returned connector HTTP 502 twice before Rust output.
48. [ ] Graph evidence refresh: after domain tests pass, re-capture or regenerate `state/rustc/ai/graph.json` so `domain::` nodes appear, then update `score.md` with the new schema/hash/node evidence.
49. [ ] Keep P4 graph editing separate: do not start graph mutation implementation until P5 domain contracts, identity, scoring, risk, bridge descriptors, subdomain modules, and fixtures are validated.


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
