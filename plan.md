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
- P5 domain intelligence layer: documentation exists in `src/domain`; compiled Rust records are not implemented yet.

## Active Priorities

First incomplete item: Active Priorities item 4, `src/domain/contracts.rs` unit test `domain_record_constructors_reject_missing_hashes` for `DomainRiskEnvelope::new`, `DomainJudgment::new`, `DomainPlan::new`, `DomainEval::new`, and `DomainPromotionCandidate::new`. Planning refresh on 2026-05-10 reconfirmed that `src/domain` is wired into `src/lib.rs` and Rust files exist for `bridge.rs`, `contracts.rs`, `identity.rs`, `mod.rs`, `risk.rs`, and `scoring.rs`; design-note Markdown files also remain for business, finance, global intelligence, integration, roadmap, scoring, and trading. The planned subdomain Rust files `global_intelligence.rs`, `business.rs`, `finance.rs`, and `trading.rs` are still absent. Python graph inspection of `state/rustc/ai/graph.json` shows schema version 16, graph hash `ab2202a8d8ec371b0c462aecc41e28d059920f53e2179dd40ebb6ebb3127fc33`, 4,473 nodes, 31,082 edges, 2,976 intents, node kinds led by 2,976 functions, 1,283 impls, 159 structs, and 53 enums, and 0 compiled `domain::` nodes; the only node containing `domain` is `runtime::reducer::raise_domain_failure`, so graph refresh must wait until domain code and tests are complete.

Ordered execute-turn checklist, one file or one test per item:

1. [x] `src/domain/contracts.rs`: add or complete `DomainSchemaVersion`, `DomainSourceKind`, `DomainHorizon`, `DomainSignalClass`, `DomainRiskClass`, `DomainPlanKind`, and `DomainLiveEffectLevel` so names align with P5.2.
   - Completed 2026-05-10: schema/source/horizon/signal/risk/plan/live-effect primitives derive serde traits, round-trip through JSON, and preserve live-effect safety ordering.
2. [x] `src/domain/contracts.rs`: implement `DomainRiskEnvelope::new(...)`, `DomainJudgment::new(...)`, `DomainPlan::new(...)`, `DomainEval::new(...)`, and `DomainPromotionCandidate::new(...)` using explicit schema versions, non-empty provenance hashes, and `0..=1000` score validation.
   - Completed 2026-05-10: added constructors for `DomainJudgment`, `DomainPlan`, `DomainEval`, and `DomainPromotionCandidate`; existing `DomainSignal::new(...)`, `DomainContext::new(...)`, and `DomainRiskEnvelope::new(...)` already validated ids, hashes, scores, schema version, and live-effect safety.
3. [x] `src/domain/contracts.rs`: add unit test `domain_record_constructors_validate_invariants` covering success paths for `DomainRiskEnvelope::new`, `DomainJudgment::new`, `DomainPlan::new`, `DomainEval::new`, and `DomainPromotionCandidate::new`.
   - Completed 2026-05-10: added success-path constructor invariant assertions for schema version assignment, bounded score preservation, hash/provenance preservation, bridge target storage, and safe live-effect storage.
4. [ ] `src/domain/contracts.rs`: add unit test `domain_record_constructors_reject_missing_hashes` covering required hash/provenance inputs to `DomainRiskEnvelope::new`, `DomainJudgment::new`, `DomainPlan::new`, `DomainEval::new`, and `DomainPromotionCandidate::new`.
5. [ ] `src/domain/contracts.rs`: add unit test `domain_record_constructors_reject_out_of_range_scores` covering all `0..=1000` score/risk inputs in `DomainRiskEnvelope::new`, `DomainJudgment::new`, `DomainPlan::new`, `DomainEval::new`, and `DomainPromotionCandidate::new`.
6. [ ] `src/domain/contracts.rs`: add unit test `domain_plan_constructor_rejects_unsafe_financial_execution` covering `DomainPlanKind::FinanceResearch`, `DomainPlanKind::TradingSimulation`, and `DomainLiveEffectLevel::LiveExternalEffect`.
7. [ ] `src/domain/identity.rs`: replace `stable_domain_id(parts)` or extend it with `DomainHash`, `DomainHashInput`, `canonical_json_bytes(record)`, `domain_hash_json(record)`, and `domain_hash_parts(parts)`.
8. [ ] `src/domain/identity.rs`: add unit test `domain_hash_is_stable`.
9. [ ] `src/domain/identity.rs`: add unit test `domain_hash_changes_when_material_field_changes`.
10. [ ] `src/domain/identity.rs`: add unit test `domain_hash_changes_when_schema_version_changes`.
11. [ ] `src/domain/scoring.rs`: implement `BoundedScore`, `ScoreInputs`, `ScoreBreakdown`, `source_quality_score(...)`, `confidence_score(...)`, `uncertainty_score(...)`, `risk_score(...)`, `promotion_score(...)`, and `verdict_for_scores(...)` using integer-only saturating math.
12. [ ] `src/domain/scoring.rs`: add unit test `verdict_ignore_thresholds`.
13. [ ] `src/domain/scoring.rs`: add unit test `verdict_watch_thresholds`.
14. [ ] `src/domain/scoring.rs`: add unit test `verdict_research_thresholds`.
15. [ ] `src/domain/scoring.rs`: add unit test `verdict_act_business_thresholds`.
16. [ ] `src/domain/scoring.rs`: add unit test `verdict_act_finance_research_thresholds`.
17. [ ] `src/domain/scoring.rs`: add unit test `verdict_simulate_trading_thresholds`.
18. [ ] `src/domain/scoring.rs`: add unit test `verdict_block_thresholds`.
19. [ ] `src/domain/risk.rs`: replace or extend `evaluate_risk_envelope(...)` with `RiskEnvelopeViolation` and `check_risk_envelope(plan, envelope)` using `DomainRiskEnvelope`, rollback requirements, invalidation requirements, and explicit live-effect constraints.
20. [ ] `src/domain/risk.rs`: add unit test `risk_blocks_live_trading`.
21. [ ] `src/domain/risk.rs`: add unit test `risk_blocks_finance_execution`.
22. [ ] `src/domain/risk.rs`: add unit test `risk_allows_verified_business_plan_with_rollback_and_invalidation`.
23. [ ] `src/domain/bridge.rs`: implement `DomainBridgeDescriptor`, `bridge_target_for_signal(...)`, `bridge_target_for_context(...)`, `bridge_target_for_judgment(...)`, `bridge_target_for_plan(...)`, and `bridge_target_for_eval(...)` as descriptor-only functions with no state or TLog mutation.
24. [ ] `src/domain/bridge.rs`: add unit test `bridge_maps_each_domain_record_family`.
25. [ ] `src/domain/bridge.rs`: add unit test `bridge_never_targets_live_trading_execution`.
26. [ ] `src/domain/global_intelligence.rs`: create `SignalClass`, `GlobalSignalProfile`, `stale_for_horizon(...)`, and `actionability_hint(...)` depending only on shared domain contracts.
27. [ ] `src/domain/global_intelligence.rs`: add unit test `global_signal_profile_staleness_is_deterministic`.
28. [ ] `src/domain/global_intelligence.rs`: add unit test `global_signal_actionability_hint_is_deterministic`.
29. [ ] `src/domain/business.rs`: create `BusinessOpportunity`, `WorkflowAutomationCandidate`, `CustomerFeedbackSignal`, and `monetization_score(...)`.
30. [ ] `src/domain/business.rs`: add unit test `business_monetization_score_is_deterministic`.
31. [ ] `src/domain/business.rs`: add unit test `business_monetization_score_is_bounded`.
32. [ ] `src/domain/finance.rs`: create `AssetUniverse`, `FinanceHypothesis`, `FinanceRiskDimensions`, and `finance_research_allowed(...)`.
33. [ ] `src/domain/finance.rs`: add unit test `finance_hypothesis_execution_allowed_is_false`.
34. [ ] `src/domain/finance.rs`: add unit test `finance_research_plan_passes_research_only_risk_check`.
35. [ ] `src/domain/trading.rs`: create `TradingSimulationPlan`, `BacktestReceiptRequirements`, `TradingRiskLimit`, and `enforce_sandbox_only(...)`.
36. [ ] `src/domain/trading.rs`: add unit test `trading_simulation_plan_rejects_live_execution`.
37. [ ] `src/domain/mod.rs`: declare `global_intelligence`, `business`, `finance`, and `trading` after their Rust files compile, and keep module docs explicit that domain code has no I/O, process, network, runtime, command-ledger, or TLog mutation authority.
38. [ ] `tests/domain_contract.rs`: add integration test `domain_records_deserialize_from_json`.
39. [ ] `tests/domain_contract.rs`: add integration test `domain_identity_is_deterministic`.
40. [ ] `tests/domain_contract.rs`: add integration test `domain_verdicts_are_deterministic`.
41. [ ] `tests/domain_contract.rs`: add integration test `domain_surface_exposes_no_runtime_mutation_api`.
42. [ ] `tests/fixtures/domain/global_signal_macro.json`: add fixture data with schema version, domain id, source/provenance hash, horizon, score inputs, expected verdict, expected risk result, and expected bridge target descriptor.
43. [ ] `tests/fixtures/domain/business_workflow_opportunity.json`: add fixture data for a verified business workflow opportunity.
44. [ ] `tests/fixtures/domain/finance_hypothesis_research.json`: add fixture data for a research-only finance hypothesis.
45. [ ] `tests/fixtures/domain/trading_simulation_sandbox.json`: add fixture data for a sandbox-only trading simulation plan.
46. [ ] `tests/fixtures/domain/trading_live_blocked.json`: add fixture data for a live trading request that must block.
47. [ ] `tests/test_domain_fixture_contract.py`: add fixture validation covering all domain fixture files and clear assertion failures for missing required fields.
48. [ ] Validation: run `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` and record the result in `score.md`.
49. [ ] Graph evidence refresh: after domain tests pass, re-capture or regenerate `state/rustc/ai/graph.json` so `domain::` nodes appear, then update `score.md` with the new schema/hash/node evidence.
50. [ ] Keep P4 graph editing separate: do not start graph mutation implementation until P5 domain contracts, scoring, risk, bridge descriptors, subdomain modules, and fixtures are validated.

## Domain Implementation Target

The domain directory currently contains specifications and a single Rust sketch:

```text
src/domain/
  README.md
  business.md
  contracts.md
  finance.md
  global_intelligence.md
  integration.md
  mod.rs
  roadmap.md
  scoring.md
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
