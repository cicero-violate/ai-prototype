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

Current date: 2026-05-13.
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

Planning reconnaissance on 2026-05-13 confirms items 60 through 137 are complete with targeted validation, reconciliation evidence, or graph-derived score refresh evidence as recorded below. Current graph-derived evidence in `SCORE_REPORT.md` reports `G = 7.93 / 10`; Structure remains the lowest aggregate axis at `4.8`, so the next executable work should prefer small, graph-backed decomposition with validation evidence. Current auto-refactor evidence reports no remaining operations for the recent completed patch helper SplitFn targets; the remaining explicit graph-backed SplitFn candidates are in `state/rustc/auto-refactor/..__state__rustc__chatgpt_mcp_connector__bin__graph.graph-editor-plan.json`, with the next legitimate SplitFn operation after the completed `check_effectful_patch(...)` extraction targeting `../chatgpt-mcp-connector/src/tools.rs::execute_policy_selected_evaluator_suite_tool_inner(...)`. Do not apply generated names or merge-surface recommendations blindly.

Reconnaissance on 2026-05-11 reconfirmed that `src/domain/identity.rs::DomainHashInput<'a>`, `src/domain/identity.rs::canonical_json_bytes(record)`, `src/domain/identity.rs::domain_hash_json(record)`, `src/domain/identity.rs::domain_hash_parts(parts)`, `src/domain/identity.rs::domain_hash_is_stable`, `src/domain/identity.rs::domain_hash_changes_when_material_field_changes`, `src/domain/identity.rs::domain_hash_changes_when_schema_version_changes`, `src/domain/scoring.rs` score-input breakdown and verdict helpers, all explicit scoring verdict threshold tests through `src/domain/scoring.rs::tests::verdict_block_thresholds`, `src/domain/risk.rs` risk-envelope API/tests through `src/domain/risk.rs::tests::risk_allows_verified_business_plan_with_rollback_and_invalidation`, and `src/domain/bridge.rs` descriptor-only bridge API/tests through `src/domain/bridge.rs::tests::bridge_never_targets_live_trading_execution` are already implemented in local source and have passing targeted validation. Planning reconnaissance also inspected `src/domain/global_intelligence.rs`, `src/domain/mod.rs`, `src/domain/scoring.rs`, `src/domain/risk.rs`, `src/domain/bridge.rs`, `src/domain/contracts.rs`, `src/domain/trading.rs`, `tests/domain_contract.rs`, all `tests/fixtures/domain/*.json` fixture artifacts, `tests/test_domain_fixture_contract.py`, and `state/rustc/ai/graph.json`. Full-suite validation is tracked separately as the validation blocker item below. Execute turns should pick up exactly one unchecked implementation item at a time. Reconnaissance on 2026-05-11 confirmed item 32 targeted finance compile evidence is captured as a compile/filter check with 0 matching `finance::tests` discovered, item 33 and item 34 finance behavior tests are complete with targeted validation, item 35 `src/domain/trading.rs` module creation has targeted source-inspection evidence, item 36 targeted trading declaration/re-export compile evidence is captured as a compile/filter check with 0 matching `trading::tests` discovered, item 37 trading behavior validation is complete, and items 38-39 domain integration tests are complete with targeted validation. Items 38-41 domain integration tests are complete with targeted validation. Items 42-46 fixture refreshes now have explicit risk-result fixture evidence and passing targeted fixture validation. Items 50, 53, 54, and 55 are complete: full-suite validation passed, refreshed graph evidence records compiled P5 domain nodes, status records the graph evidence summary, and score rationale was reviewed without changing numeric project scores. P4 graph-editing remains guarded: do not implement graph mutation or semantic graph-op mutation work until the receipt-backed graph edit path is explicitly selected, patches are re-captured, graph diffs are recorded as TLog evidence, and generated merge-surface noise is manually filtered. Current auto-refactor plans under `state/rustc/auto-refactor/.*.graph-editor-plan.json` contain legitimate future `SplitFn` candidates such as `agent::loop_driver::LoopDriver::run_cycle`, `agent::loop_driver::sync_mcp_workspace`, `agent::cycle::AgentCycle::run`, `agent::router::send_streaming_request`, `graph_mutation::generate_graph_patch`, and `capability::tooling::record::process::LiveSandboxProcessExecutor::execute_authorized_process`; these are deferred until their own checklist items. Reconnaissance confirmed `SplitFn id=001e821dc83e940a` for `LoopDriver::run_cycle` (`expected_lo=4482`, `expected_hi=9565`, helpers `run_cycle__parse`/`run_cycle__transform`) and `SplitFn id=d535999f445621fb` for `sync_mcp_workspace` (`expected_lo=25917`, `expected_hi=27725`, helpers `sync_mcp_workspace__parse`/`sync_mcp_workspace__transform`). Source inspection found `LoopDriver::run_cycle` at `src/agent/loop_driver.rs:129` and `sync_mcp_workspace` at `src/agent/loop_driver.rs:667`. Do not apply generated `merge_surface` recommendations blindly.

Current source inventory from `find src/domain -type f | sort`:

```text
src/domain/bridge.rs
src/domain/business.md
src/domain/business.rs
src/domain/contracts.md
src/domain/contracts.rs
src/domain/finance.md
src/domain/finance.rs
src/domain/global_intelligence.md
src/domain/global_intelligence.rs
src/domain/identity.rs
src/domain/integration.md
src/domain/mod.rs
src/domain/README.md
src/domain/risk.rs
src/domain/roadmap.md
src/domain/scoring.md
src/domain/scoring.rs
src/domain/trading.md
src/domain/trading.rs
```

Graph-derived score evidence from `SCORE_REPORT.md` was refreshed on 2026-05-13 after items 107-109: schema version 16, 16 crates, `G = 7.93 / 10`, Architecture `9.0`, Structure `4.8`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`. The refreshed `ai` crate row reports 5,463 nodes, 35,002 edges, 2,199 functions, Architecture `9.5`, Structure `6.0`, Simplicity `7.0`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`. This refresh does not justify a project-level numeric score increase by itself.

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
8. [x] `src/domain/identity.rs`: add unit test `domain_hash_changes_when_schema_version_changes`.
   - Implementation is present in local source and asserts `domain_hash_json` changes when the material `schema_version` field changes.
   - Targeted validation passed on 2026-05-10 during implementation step 1: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test identity::tests -- --test-threads=1` ran 6 identity tests successfully after one connector HTTP 502 retry.
   - Required full-suite validation remains blocked on 2026-05-10: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` returned connector HTTP 502 twice before Rust output. The blocker is tracked in the validation item below; no commit was made in this turn.
9. [x] `src/domain/scoring.rs`: implement `BoundedScore` with `new(value: u16)`, `get(&self) -> u16`, `zero()`, `max()`, and `saturating_weighted_average(...)` using integer-only math.
   - Implementation is present in local source with `BoundedScore(u16)`, `MIN`, `MAX`, clamping constructor, accessors, and integer-only saturating weighted average.
   - Targeted validation passed on 2026-05-10 during planning reconciliation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test scoring::tests::bounded_score_clamps_and_averages_with_integer_math -- --test-threads=1` ran 1 scoring test successfully with 0 failures.
   - Required full-suite validation remains tracked by the validation blocker item below; do not re-select this item for implementation.
10. [x] `src/domain/scoring.rs`: implement `ScoreInputs`, `ScoreBreakdown`, `source_quality_score(...)`, `confidence_score(...)`, `uncertainty_score(...)`, `risk_score(...)`, `promotion_score(...)`, and `verdict_for_scores(...)` using integer-only saturating math.
   - Implementation is present in local source with `ScoreInputs`, `ScoreBreakdown`, deterministic source-quality/confidence/uncertainty/risk/promotion helpers, `score_breakdown(...)`, and conservative `verdict_for_scores(...)` routing for business, finance, trading sandbox, block, ignore, watch, and research outcomes.
   - Targeted validation passed on 2026-05-10 during implementation step 1: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test scoring::tests -- --test-threads=1` ran 4 scoring tests successfully with 0 failures after one connector HTTP 502 retry and one corrected integer-floor test expectation.
   - Required full-suite validation remains blocked on 2026-05-10: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` returned connector HTTP 502 twice before Rust output. The blocker is tracked in the validation item below; no commit was made in this turn.
11. [x] `src/domain/scoring.rs`: add unit test `verdict_ignore_thresholds`.
   - Implementation is present in local source and covers the ignore threshold branch for low domain value/actionability while retaining a fixture-style watch boundary check for the global macro values.
   - Targeted validation passed on 2026-05-10 during implementation step 4: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test scoring::tests::verdict_ignore_thresholds -- --test-threads=1` ran 1 scoring test successfully with 0 failures after one corrected integer-floor assertion and one connector HTTP 502 retry.
   - Broader scoring validation passed on 2026-05-10: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test scoring::tests -- --test-threads=1` ran 5 scoring tests successfully with 0 failures.
   - Required full-suite validation remains blocked on 2026-05-10: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` returned connector HTTP 502 twice before Rust output. The blocker is tracked in the validation item below; no commit was made in this turn.
12. [x] `src/domain/scoring.rs`: add unit test `verdict_watch_thresholds` covering the existing `tests/fixtures/domain/global_signal_macro.json` score inputs and asserting domain value `134`, actionability `69`, and `DomainVerdict::Watch` for `DomainId::GlobalIntelligence`.
   - Scope: `src/domain/scoring.rs` test module only.
   - Done when: the named test exists and proves the watch branch independently of `verdict_ignore_thresholds`.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test scoring::tests::verdict_watch_thresholds -- --test-threads=1` passed on 2026-05-10 after one connector HTTP 502 retry; broader `cargo test scoring::tests -- --test-threads=1` also passed with 6 scoring tests.
13. [x] `src/domain/scoring.rs`: add unit test `verdict_research_thresholds` using a non-specialized domain input where domain value is at least `200`, no act branch applies, and verdict is `DomainVerdict::Research`.
   - Scope: `src/domain/scoring.rs` test module only.
   - Done when: the named test asserts computed domain/actionability scores plus `DomainVerdict::Research`.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test scoring::tests::verdict_research_thresholds -- --test-threads=1` passed on 2026-05-10 after one connector HTTP 502 retry; broader `cargo test scoring::tests -- --test-threads=1` also passed with 7 scoring tests.
14. [x] `src/domain/scoring.rs`: add unit test `verdict_act_business_thresholds` covering `tests/fixtures/domain/business_workflow_opportunity.json` score inputs and asserting domain value `459`, actionability `267`, and `DomainVerdict::ActBusiness`.
   - Scope: `src/domain/scoring.rs` test module only.
   - Done when: the named test proves the business act branch at the fixture boundary.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test scoring::tests::verdict_act_business_thresholds -- --test-threads=1` passed on 2026-05-10 after one connector HTTP 502 retry; broader `cargo test scoring::tests -- --test-threads=1` also passed with 8 scoring tests.
15. [x] `src/domain/scoring.rs`: add unit test `verdict_act_finance_research_thresholds` covering `tests/fixtures/domain/finance_hypothesis_research.json` score inputs and asserting domain value `176`, actionability `85`, and `DomainVerdict::ActFinanceResearch`.
   - Scope: `src/domain/scoring.rs` test module only.
   - Done when: the named test proves finance remains research-only instead of live execution.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test scoring::tests::verdict_act_finance_research_thresholds -- --test-threads=1` passed on 2026-05-10 after one connector HTTP 502 retry; broader `cargo test scoring::tests -- --test-threads=1` also passed with 9 scoring tests.
16. [x] `src/domain/scoring.rs`: add unit test `verdict_simulate_trading_thresholds` covering `tests/fixtures/domain/trading_simulation_sandbox.json` score inputs and asserting domain value `204`, actionability `86`, and `DomainVerdict::SimulateTrading`.
   - Scope: `src/domain/scoring.rs` test module only.
   - Done when: the named test proves the trading branch routes only to simulation.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test scoring::tests::verdict_simulate_trading_thresholds -- --test-threads=1` passed on 2026-05-10 after one connector HTTP 502 retry; broader `cargo test scoring::tests -- --test-threads=1` also passed with 10 scoring tests.
17. [x] `src/domain/scoring.rs`: add unit test `verdict_block_thresholds` covering `tests/fixtures/domain/trading_live_blocked.json` score inputs and asserting domain value `0`, actionability `0`, and `DomainVerdict::Block` from zero policy fit/high risk.
   - Scope: `src/domain/scoring.rs` test module only.
   - Done when: the named test proves block precedence over ignore for unsafe requests.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test scoring::tests::verdict_block_thresholds -- --test-threads=1` passed on 2026-05-11 after one connector HTTP 502 retry; broader `cargo test scoring::tests -- --test-threads=1` also passed with 11 scoring tests.
18. [x] `src/domain/risk.rs`: implement `RiskEnvelopeViolation` and `check_risk_envelope(plan: &DomainPlan, envelope: &DomainRiskEnvelope) -> Result<(), RiskEnvelopeViolation>` covering rollback requirements, invalidation requirements, risk bounds, and live-effect constraints.
   - Scope: `src/domain/risk.rs` only.
   - Done when: the new API compiles without adding I/O, runtime, network, command-ledger, or TLog mutation authority.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test risk::tests -- --test-threads=1` passed on 2026-05-11 after one connector HTTP 502 retry with 1 risk test and 0 failures.
19. [x] `src/domain/risk.rs`: add unit test `risk_blocks_live_trading`.
   - Scope: `src/domain/risk.rs` test module only.
   - Done when: live trading or financial execution is rejected for a sandbox trading plan/envelope.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test risk::tests::risk_blocks_live_trading -- --test-threads=1` passed on 2026-05-11 after one connector HTTP 502 retry; broader `cargo test risk::tests -- --test-threads=1` also passed with 2 risk tests.
20. [x] `src/domain/risk.rs`: add unit test `risk_blocks_finance_execution`.
   - Scope: `src/domain/risk.rs` test module only.
   - Done when: finance execution beyond research-only bounds is rejected.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test risk::tests::risk_blocks_finance_execution -- --test-threads=1` passed on 2026-05-11 after one connector HTTP 502 retry; broader `cargo test risk::tests -- --test-threads=1` also passed with 3 risk tests.
21. [x] `src/domain/risk.rs`: add unit test `risk_allows_verified_business_plan_with_rollback_and_invalidation`.
   - Scope: `src/domain/risk.rs` test module only.
   - Done when: a verified business workflow proposal with rollback and invalidation evidence passes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test risk::tests::risk_allows_verified_business_plan_with_rollback_and_invalidation -- --test-threads=1` passed on 2026-05-11 after one connector HTTP 502 retry; broader `cargo test risk::tests -- --test-threads=1` also passed with 4 risk tests.
22. [x] `src/domain/bridge.rs`: implement `DomainBridgeDescriptor`, `bridge_target_for_signal(...)`, `bridge_target_for_context(...)`, `bridge_target_for_judgment(...)`, `bridge_target_for_plan(...)`, and `bridge_target_for_eval(...)` as descriptor-only functions with no state, command-ledger, runtime, network, process, or TLog mutation authority.
   - Scope: `src/domain/bridge.rs` only.
   - Done when: every record family maps to a typed descriptor and the module remains pure data mapping.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test bridge::tests -- --test-threads=1` passed on 2026-05-11 after one connector HTTP 502 retry; broader full-suite validation remained blocked by connector HTTP 502 before Rust output and is tracked by item 47.
23. [x] `src/domain/bridge.rs`: add unit test `bridge_maps_each_domain_record_family`.
   - Scope: `src/domain/bridge.rs` test module only.
   - Done when: signal, context, judgment, plan, and eval record families each produce expected descriptors.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test bridge::tests::bridge_maps_each_domain_record_family -- --test-threads=1` passed on 2026-05-11 after one connector HTTP 502 retry; broader `cargo test bridge::tests -- --test-threads=1` also passed with 2 bridge tests. Full-suite validation remained blocked by connector HTTP 502 before Rust output and is tracked by item 47.
24. [x] `src/domain/bridge.rs`: add unit test `bridge_never_targets_live_trading_execution`.
   - Scope: `src/domain/bridge.rs` test module only.
   - Done when: trading descriptors route to simulation/planning or blocked targets, never live execution.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test bridge::tests::bridge_never_targets_live_trading_execution -- --test-threads=1` passed on 2026-05-11 with 1 bridge test; broader `cargo test bridge::tests -- --test-threads=1` passed with 3 bridge tests. Full-suite validation remained blocked by connector HTTP 502 before Rust output and is tracked by item 47.
25. [x] `src/domain/mod.rs`: declare `global_intelligence` and re-export `SignalClass`, `GlobalSignalProfile`, `stale_for_horizon(...)`, and `actionability_hint(...)` from the existing `src/domain/global_intelligence.rs` module without declaring `business`, `finance`, or `trading` yet.
   - Scope: `src/domain/mod.rs` only.
   - Done when: `src/domain/global_intelligence.rs` compiles through the public domain module surface and its existing tests are discoverable.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test global_intelligence::tests -- --test-threads=1` passed on 2026-05-11 with 2 `global_intelligence` tests discovered and passing. Full-suite validation remains blocked by repeated connector HTTP 502 before Rust output and is tracked by item 47; no commit was made.
26. [x] `src/domain/global_intelligence.rs`: add unit test `global_signal_profile_staleness_is_deterministic`.
   - Scope: `src/domain/global_intelligence.rs` test module only.
   - Done when: repeated staleness checks over the same profile produce identical results.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test global_intelligence::tests::global_signal_profile_staleness_is_deterministic -- --test-threads=1` passed on 2026-05-11 with 1 named test; broader `cargo test global_intelligence::tests -- --test-threads=1` passed with 3 global-intelligence tests. Full-suite validation remains blocked by connector HTTP 502 before Rust output and is tracked by item 47; no commit was made.
27. [x] `src/domain/global_intelligence.rs`: add unit test `global_signal_actionability_hint_is_deterministic`.
   - Scope: `src/domain/global_intelligence.rs` test module only.
   - Done when: repeated actionability hints over the same profile produce identical results.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test global_intelligence::tests::global_signal_actionability_hint_is_deterministic -- --test-threads=1` passed on 2026-05-11 with 1 named test; broader `cargo test global_intelligence::tests -- --test-threads=1` passed with 4 global-intelligence tests.
28. [x] `src/domain/mod.rs`: declare `business` and re-export `BusinessOpportunity`, `WorkflowAutomationCandidate`, `CustomerFeedbackSignal`, and `monetization_score(...)` from the existing `src/domain/business.rs` module without declaring `finance` or `trading` yet.
   - Scope: `src/domain/mod.rs` only.
   - Done when: `src/domain/business.rs` compiles through the public domain module surface and its existing compile-smoke test is discoverable.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test business::tests -- --test-threads=1` passed on 2026-05-11 with 3 business tests discovered and passing. Full-suite validation remains blocked by connector HTTP 502 before Rust output and is tracked by item 50; no commit was made.
29. [x] `src/domain/business.rs`: rename or add unit test `business_monetization_score_is_deterministic` using the existing `BusinessOpportunity`, `WorkflowAutomationCandidate`, `CustomerFeedbackSignal`, and `monetization_score(...)` helpers.
   - Scope: `src/domain/business.rs` test module only.
   - Done when: repeated monetization scoring over the same inputs is stable.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test business::tests::business_monetization_score_is_deterministic -- --test-threads=1` passed on 2026-05-11; broader `cargo test business::tests -- --test-threads=1` passed with 3 business tests during item 28 validation.
30. [x] `src/domain/business.rs`: add unit test `business_monetization_score_is_bounded`.
   - Scope: `src/domain/business.rs` test module only.
   - Done when: extreme score inputs are clamped through `BoundedScore` and `monetization_score(...)` remains within `0..=1000`.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test business::tests::business_monetization_score_is_bounded -- --test-threads=1` passed on 2026-05-11; broader `cargo test business::tests -- --test-threads=1` passed with 3 business tests during item 28 validation.
31. [x] `src/domain/finance.rs`: create Rust module with `AssetUniverse`, `FinanceHypothesis`, `FinanceRiskDimensions`, and `finance_research_allowed(...)`.
   - Scope: create `src/domain/finance.rs` only.
   - Done when: the file contains pure deterministic finance-domain records and a research-only allowance helper with no I/O, runtime, command-ledger, network, process, or TLog mutation authority.
   - Validation: source inspection passed on 2026-05-11: `test -f src/domain/finance.rs`, required symbol greps for `AssetUniverse`, `FinanceHypothesis`, `FinanceRiskDimensions`, and `finance_research_allowed(...)`, plus disallowed-authority grep for filesystem/process/runtime/TLog/state/network primitives. Compile validation remains deferred until item 32 declares the module. Broader full-suite validation remains blocked by connector HTTP 502 before Rust output and is tracked by item 50.
32. [x] Targeted validation blocker / `src/domain/mod.rs` finance declaration evidence: `src/domain/mod.rs` already declares `finance` and re-exports `AssetUniverse`, `FinanceHypothesis`, `FinanceRiskDimensions`, and `finance_research_allowed(...)`; targeted compile evidence is now captured.
   - Scope: `src/domain/mod.rs` validation evidence only; no edit to `src/domain/mod.rs` was needed because source inspection showed the finance declaration/re-export remains present.
   - Done when: targeted Rust output is available and proves `src/domain/finance.rs` compiles through the public domain module surface.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test finance::tests -- --test-threads=1` passed on 2026-05-11 with Rust output available and 0 matching `finance::tests` discovered; this proves the declared finance module compiles through the public domain surface but does not prove finance behavior tests, which start at item 33.
33. [x] `src/domain/finance.rs`: add unit test `finance_hypothesis_execution_allowed_is_false`.
   - Scope: `src/domain/finance.rs` test module only.
   - Done when: finance execution beyond research-only bounds is deterministically rejected.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test finance::tests::finance_hypothesis_execution_allowed_is_false -- --test-threads=1` passed on 2026-05-11 with 1 named finance test and 0 failures; broader `cargo test finance::tests -- --test-threads=1` also passed with 1 finance test. Full-suite validation remained blocked by connector HTTP 502 before Rust output and is tracked by item 50.
34. [x] `src/domain/finance.rs`: add unit test `finance_research_plan_passes_research_only_risk_check`.
   - Scope: `src/domain/finance.rs` test module only.
   - Done when: a finance research plan remains allowed only under read-only/research constraints.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test finance::tests::finance_research_plan_passes_research_only_risk_check -- --test-threads=1` passed on 2026-05-11 with 1 named finance test and 0 failures; broader `cargo test finance::tests -- --test-threads=1` also passed with 2 finance tests. Full-suite validation remained blocked by connector HTTP 502 before Rust output and is tracked by item 50.
35. [x] `src/domain/trading.rs`: create Rust module with `TradingSimulationPlan`, `BacktestReceiptRequirements`, `TradingRiskLimit`, and `enforce_sandbox_only(...)`.
   - Scope: create `src/domain/trading.rs` only.
   - Done when: the file contains pure deterministic sandbox-only trading records and no live execution authority.
   - Validation: source inspection passed on 2026-05-11: `test -f src/domain/trading.rs`, required symbol greps for `TradingSimulationPlan`, `BacktestReceiptRequirements`, `TradingRiskLimit`, and `enforce_sandbox_only(...)`, plus non-doc disallowed-authority token scan for filesystem/process/runtime/network/command-ledger primitives. Compile validation remains deferred until item 36 declares the module. Broader full-suite validation remains blocked by connector HTTP 502 before Rust output and is tracked by item 50.
36. [x] `src/domain/mod.rs`: declare `trading` and re-export `TradingSimulationPlan`, `BacktestReceiptRequirements`, `TradingRiskLimit`, and `enforce_sandbox_only(...)`; keep module docs explicit that domain code has no I/O, process, network, runtime, command-ledger, or TLog mutation authority.
   - Scope: `src/domain/mod.rs` only.
   - Done when: `src/domain/trading.rs` compiles through the public domain module surface and its tests are discoverable.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test trading::tests -- --test-threads=1` passed on 2026-05-11 during implementation step 2 with Rust output available and 0 matching `trading::tests` discovered, proving the declared trading module compiles through the public domain surface. Behavior validation starts at item 37. Broader full-suite validation remains blocked by connector HTTP 502 before Rust output and is tracked by item 50.
37. [x] `src/domain/trading.rs`: add unit test `trading_simulation_plan_rejects_live_execution`.
   - Scope: `src/domain/trading.rs` test module only.
   - Done when: live trading or brokerage execution is rejected while sandbox simulation remains representable.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test trading::tests::trading_simulation_plan_rejects_live_execution -- --test-threads=1` passed on 2026-05-11 with 1 named trading test and 0 failures; broader `cargo test trading::tests -- --test-threads=1` also passed with 1 trading test. Full-suite validation remained blocked by connector HTTP 502 before Rust output and is tracked by item 50.
38. [x] `tests/domain_contract.rs`: add integration test `domain_records_deserialize_from_json`.
   - Scope: create or update `tests/domain_contract.rs` only.
   - Done when: the domain fixture records deserialize into the typed domain contract surface.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test domain_contract domain_records_deserialize_from_json -- --test-threads=1` passed on 2026-05-11 with 1 named integration test and 0 failures; broader `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test domain_contract -- --test-threads=1` also passed with 1 integration test and 0 failures. Full-suite validation returned connector HTTP 502 before Rust output and remains tracked by item 50.
39. [x] `tests/domain_contract.rs`: add integration test `domain_identity_is_deterministic`.
   - Scope: `tests/domain_contract.rs` only.
   - Done when: fixture-derived domain hashes are stable and material-field changes alter identity.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test domain_contract domain_identity_is_deterministic -- --test-threads=1` passed on 2026-05-11 with 1 named integration test and 0 failures; broader `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test domain_contract -- --test-threads=1` also passed with 2 integration tests and 0 failures. Full-suite validation returned connector HTTP 502 before Rust output and remains tracked by item 50.
40. [x] `tests/domain_contract.rs`: add integration test `domain_verdicts_are_deterministic`.
   - Scope: `tests/domain_contract.rs` only.
   - Done when: fixture score inputs deterministically produce the expected verdict and bridge target.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test domain_contract domain_verdicts_are_deterministic -- --test-threads=1` passed on 2026-05-11 with 1 named integration test and 0 failures; broader `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test domain_contract -- --test-threads=1` passed with 3 integration tests and 0 failures. Full-suite validation returned connector HTTP 502 before Rust output and remains tracked by item 50.
41. [x] `tests/domain_contract.rs`: add integration test `domain_surface_exposes_no_runtime_mutation_api`.
   - Scope: `tests/domain_contract.rs` only.
   - Done when: the public domain surface remains descriptor-only and does not expose runtime, command-ledger, process, network, or TLog mutation APIs.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test domain_contract domain_surface_exposes_no_runtime_mutation_api -- --test-threads=1` passed on 2026-05-11 with 1 named integration test and 0 failures; broader `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test domain_contract -- --test-threads=1` passed with 4 integration tests and 0 failures. Full-suite validation returned connector HTTP 502 before Rust output and remains tracked by item 50.
42. [x] `tests/fixtures/domain/global_signal_macro.json`: refresh fixture data with schema version, domain id, source/provenance hash, horizon, score inputs, expected verdict, expected risk result, and expected bridge target descriptor.
   - Scope: `tests/fixtures/domain/global_signal_macro.json` only.
   - Done when: the fixture has every required field consumed by `tests/test_domain_fixture_contract.py`.
   - Validation: `python3 -m unittest tests/test_domain_fixture_contract.py` passed on 2026-05-11 with 5 fixture contract tests and 0 failures after adding explicit top-level `expected_risk_result: "pass"`; broader `cargo test --all-targets` remained blocked by connector HTTP 502 before Rust output and is tracked by item 50.
43. [x] `tests/fixtures/domain/business_workflow_opportunity.json`: refresh fixture data for a verified business workflow opportunity.
   - Scope: `tests/fixtures/domain/business_workflow_opportunity.json` only.
   - Done when: the fixture preserves `domain_id: "Business"`, expected scores `459` and `267`, `expected_verdict: "ActBusiness"`, `expected_bridge_target: "PlanRecord"`, `expected_risk_envelope.expected_result: "pass"`, and adds/keeps explicit risk-result fixture evidence consumed by `tests/test_domain_fixture_contract.py`.
   - Validation: `python3 -m unittest tests/test_domain_fixture_contract.py` passed on 2026-05-11 with 5 fixture contract tests and 0 failures after adding explicit top-level `expected_risk_result: "pass"`; broader `cargo test --all-targets` remained blocked by connector HTTP 502 before Rust output and is tracked by item 50.
44. [x] `tests/fixtures/domain/finance_hypothesis_research.json`: refresh fixture data for a research-only finance hypothesis.
   - Scope: `tests/fixtures/domain/finance_hypothesis_research.json` only.
   - Done when: the fixture has every required field and proves finance remains research-only.
   - Validation: `python3 -m unittest tests/test_domain_fixture_contract.py` passed on 2026-05-11 with 5 fixture contract tests and 0 failures after adding explicit top-level `expected_risk_result: "pass"`; broader Rust validation remained blocked by connector HTTP 502 before Rust output and is tracked by item 50.
45. [x] `tests/fixtures/domain/trading_simulation_sandbox.json`: refresh fixture data for a sandbox-only trading simulation plan.
   - Scope: `tests/fixtures/domain/trading_simulation_sandbox.json` only.
   - Done when: the fixture has every required field and proves trading routes to simulation only.
   - Validation: `python3 -m unittest tests/test_domain_fixture_contract.py` passed on 2026-05-11 with 5 fixture contract tests and 0 failures after adding explicit top-level `expected_risk_result: "pass"`; broader Rust validation remained blocked by connector HTTP 502 before Rust output and is tracked by item 50.
46. [x] `tests/fixtures/domain/trading_live_blocked.json`: refresh fixture data for a live trading request that must block.
   - Scope: `tests/fixtures/domain/trading_live_blocked.json` only.
   - Done when: the fixture has every required field and proves unsafe live trading blocks.
   - Validation: `python3 -m unittest tests/test_domain_fixture_contract.py` passed on 2026-05-11 with 5 fixture contract tests and 0 failures after confirming explicit top-level `expected_risk_result: "block"`; broader Rust validation remained blocked by connector HTTP 502 before Rust output and is tracked by item 50.
47. [x] `tests/test_domain_fixture_contract.py::test_fixtures_include_expected_risk_result`: add a unittest that iterates all `REQUIRED_FIXTURES`, asserts top-level `expected_risk_result` exists, asserts it is either `"pass"` or `"block"`, and asserts it equals `expected_risk_envelope["expected_result"]`.
   - Scope: `tests/test_domain_fixture_contract.py` only; add exactly the named test method and shared constants only if needed.
   - Done when: all five domain fixtures prove top-level risk-result evidence is present and consistent with the structured risk envelope.
   - Validation: `python3 -m unittest tests.test_domain_fixture_contract.DomainFixtureContract.test_fixtures_include_expected_risk_result` passed on 2026-05-11 with 1 named unittest and 0 failures; broader `python3 -m unittest tests/test_domain_fixture_contract.py` passed with 6 fixture contract tests and 0 failures.
48. [x] `tests/test_domain_fixture_contract.py::test_required_fixture_fields_have_clear_assertions`: add a unittest that iterates all `REQUIRED_FIXTURES` and reports missing required top-level fixture fields with the fixture name and field name.
   - Scope: `tests/test_domain_fixture_contract.py` only; do not edit fixture JSON files.
   - Done when: required fields for schema, fixture/source/provenance/payload hashes, domain id, record family, horizon, signal class, uncertainty, score inputs, expected scores, expected verdict, expected risk result, expected risk envelope, expected bridge target, and expected eval behavior are checked before value-specific tests run.
   - Validation: `python3 -m unittest tests.test_domain_fixture_contract.DomainFixtureContract.test_required_fixture_fields_have_clear_assertions` passed on 2026-05-11 with 1 named unittest and 0 failures; broader `python3 -m unittest tests/test_domain_fixture_contract.py` passed with 7 fixture contract tests and 0 failures.
49. [x] `tests/test_domain_fixture_contract.py`: run the full fixture contract after item 47 and item 48.
   - Scope: `tests/test_domain_fixture_contract.py` validation only.
   - Done when: all fixture contract unittest cases pass with the refreshed field-level and risk-result checks.
   - Validation: `python3 -m unittest tests/test_domain_fixture_contract.py` passed on 2026-05-11 with 7 fixture contract tests and 0 failures.
50. [x] `cargo test --all-targets`: retry the full Rust workspace validation gate and record Rust output or connector blocker evidence.
   - Scope: full Rust workspace validation only; do not edit source files while selecting this validation item.
   - Done when: full-suite Rust output is available and green. If connector HTTP 502 or another transport failure occurs before Rust output, leave this item unchecked and record the infrastructure blocker in `status.md`.
   - Validation: local terminal evidence on 2026-05-11 showed `cargo test --all-targets` passed with the Canon rustc wrapper capturing witnesses for `ai`, all listed binaries, and all target/example suites. The run included 258 library tests, 10 API server contract tests, 20 API transport contract tests, 3 canonical TLog contract tests, 4 domain contract tests, 10 graph mutation CLI contract tests, 9 MCP receipt contract tests, 2 planning contract tests, 5 score contract tests, 2 supervisor binary contract tests, 352 validation harness contract tests, 2 worker binary contract tests, and all listed zero-test binary/example harnesses, with 0 failures.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`.
51. [x] `scripts/analyze_graph_json.py`: create the graph-analysis script that reads `state/rustc/ai/graph.json` and prints schema version, graph hash, receipt hash, risk hash, node count, edge count, intent count, node-kind counts, and compiled P5 `domain::`/`src/domain` node matches.
   - Scope: `scripts/analyze_graph_json.py` only.
   - Done when: the script produces the graph summary and explicit P5 domain-node match count without mutating graph state.
   - Validation: `python3 scripts/analyze_graph_json.py state/rustc/ai/graph.json` passed on 2026-05-11 with schema version 16, graph hash `ba0aec3b291b2bbd5a3ae3db140a6636edb38e06ae4511d17d1191b7ca3704bd`, 5,297 nodes, 32,559 edges, 3,423 intents, and 752 P5 domain-node matches.
52. [x] `scripts/analyze_graph_json.py`: add a regression self-check path that fails when required top-level graph keys or metadata hashes are missing.
   - Scope: `scripts/analyze_graph_json.py` only.
   - Done when: the script exits nonzero for malformed graph input or missing required metadata and still passes against `state/rustc/ai/graph.json`.
   - Validation: `python3 scripts/analyze_graph_json.py state/rustc/ai/graph.json && python3 scripts/analyze_graph_json.py tests/fixtures/domain/global_signal_macro.json >/tmp/analyze_graph_json_negative.out 2>/tmp/analyze_graph_json_negative.err; test $? -ne 0` passed on 2026-05-11; the malformed fixture exited 1 with missing graph keys `edges`, `intents`, `meta`, and `nodes`.
53. [x] `state/rustc/ai/graph.json`: after item 50 is green, re-capture or regenerate the Rust graph evidence so compiled P5 `domain::` nodes appear.
   - Scope: `state/rustc/ai/graph.json` and generated graph evidence artifacts only.
   - Done when: refreshed graph evidence includes compiled P5 `domain::` or `src/domain` nodes and current graph hash/counts.
   - Validation: `python3 scripts/analyze_graph_json.py state/rustc/ai/graph.json` passed on 2026-05-11 with schema version 16, graph hash `2399ea73e0eccc81561d2f1f3aaedb1692d965e0a4dfcbe6c0e2c4cb3e67f664`, receipt hash `5fe7df2837b97ed8ebf73c0eb7de3284c57be55ab7d3d8803bf094708ad5bbde`, risk hash `6101aa0240348d6458bd941fd932e5dceb6a577a3e90e04df4790eda615a2a9a`, 5,337 nodes, 33,179 edges, 3,453 intents, and 752 compiled P5 domain-node matches; broader `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` output captured from the transport-interrupted run shows all listed suites passing with 0 failures.
54. [x] `status.md`: record the refreshed graph schema/hash/node evidence after item 53.
   - Scope: `status.md` Evidence Summary and Validation Ledger only.
   - Done when: status records the new graph hash, node count, edge count, intent count, and P5 domain-node evidence.
   - Validation: `python3 scripts/analyze_graph_json.py state/rustc/ai/graph.json` passed on 2026-05-11 with schema version 16, graph hash `2399ea73e0eccc81561d2f1f3aaedb1692d965e0a4dfcbe6c0e2c4cb3e67f664`, 5,337 nodes, 33,179 edges, 3,453 intents, and 752 compiled P5 domain-node matches.
55. [x] `score.md`: update score rationale only if item 50 full-suite validation or item 53 refreshed graph evidence justifies a score change.
   - Scope: `score.md` only.
   - Done when: scores either remain explicitly unchanged with rationale, or a value changes with evidence cited from `status.md`.
   - Validation: score/status consistency review passed on 2026-05-11; project-level numeric scores remain unchanged, while rationale now cites item 50 full-suite validation, item 53 refreshed graph evidence, and current `SCORE_REPORT.md` graph-derived `G = 8.02 / 10`.
56. [x] `plan.md`: keep the P4 graph-editing guardrail documented after P5 validation.
   - Scope: `plan.md` only.
   - Done when: graph mutation implementation remains deferred until P5 domain contracts, identity, scoring, risk, bridge descriptors, subdomain modules, fixtures, full-suite validation, and refreshed graph evidence are complete.
   - Validation: planning/status review passed on 2026-05-11; the Active Priorities introduction now keeps graph mutation and semantic graph-op mutation deferred until the receipt-backed graph edit path is explicitly selected, patches are re-captured, graph diffs are recorded as TLog evidence, and generated merge-surface noise is manually filtered.
57. [x] `src/agent/loop_driver.rs`: after items 50, 53, 54, and 55 are complete, inspect graph-backed `SplitFn id=001e821dc83e940a` for `agent::loop_driver::LoopDriver::run_cycle` and write the smallest helper-extraction implementation checklist item for that exact function.
   - Scope: `plan.md`, `status.md`, and `src/agent/loop_driver.rs` inspection only; do not edit Rust source in this planning item.
   - Done when: the next implementation item names the exact helper boundary inside `LoopDriver::run_cycle`, the expected extracted helper name, companion test or targeted validation, and confirms the public signature is preserved.
   - Validation: passed on 2026-05-11 by inspecting `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json` for `SplitFn id=001e821dc83e940a`, `sed -n '120,230p' src/agent/loop_driver.rs`, `sed -n '230,390p' src/agent/loop_driver.rs`, and `rg -n "run_cycle|LoopDriver|sync_mcp_workspace" src tests -S`.
58. [x] `cargo test --all-targets`: retry full-suite Rust validation for the existing `src/agent/loop_driver.rs::LoopDriver::run_cycle_attempt(...)` extraction and mark item 58 complete only after green full-suite Rust output.
   - Scope: validation evidence for `src/agent/loop_driver.rs::LoopDriver::run_cycle` / `run_cycle_attempt(...)` only. Current source inspection shows `run_cycle_attempt(...)` exists at line 238 and delegates the per-attempt `ChunkLogger`, `OpenAiChatRequest`, and `RouterClient::streaming_turn` block; `cargo fmt --check` returned exit 0 during planning reconnaissance. Do not edit `sync_mcp_workspace`, graph mutation code, domain code, tests, fixtures, generated auto-refactor JSON, unrelated supervisor/runtime files, or unrelated router/certification changes while selecting this item.
   - Done when: `run_cycle` still owns total-turn computation, spawned/planning/execute prompt selection, sleep/retry loop control, completion error handling, router-tab close, and certification; only the per-attempt streaming/logger block from the inner retry loop is delegated to `run_cycle_attempt(...)`; the original `run_cycle` signature remains unchanged; `cargo fmt --check` and full-suite Rust validation both pass.
   - Validation: `cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` passed on 2026-05-11 with 272 library tests, 11 API server contract tests, 20 API transport contract tests, 3 canonical TLog contract tests, 4 domain contract tests, 10 graph mutation CLI contract tests, 9 MCP receipt contract tests, 2 planning contract tests, 5 score contract tests, 2 supervisor binary contract tests, 352 validation harness contract tests, 2 worker binary contract tests, and all listed zero-test binary/example harnesses passing with 0 failures.
59. [x] `src/agent/loop_driver.rs`: after item 58 is green, inspect graph-backed `SplitFn id=d535999f445621fb` for `agent::loop_driver::sync_mcp_workspace` and write the smallest helper-extraction implementation checklist item for that exact function.
   - Scope: `plan.md`, `status.md`, and `src/agent/loop_driver.rs` inspection only; do not edit Rust source in this planning item.
   - Done when: the next implementation item names the exact helper boundary inside `sync_mcp_workspace`, the expected extracted helper name, companion test or targeted validation, and confirms the original function still returns `Result<(), String>`.
   - Validation: passed on 2026-05-12 by inspecting `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json` for `SplitFn id=d535999f445621fb`, `sed -n '690,760p' src/agent/loop_driver.rs`, and `rg -n "fn sync_mcp_workspace|sync_mcp_workspace\(" src/agent/loop_driver.rs`.
60. [x] `src/agent/loop_driver.rs`: extract URL parsing from `sync_mcp_workspace` into private helper `parse_mcp_workspace_endpoint(mcp_url: &str) -> Result<(String, u16), String>` while preserving `sync_mcp_workspace(mcp_url: &str, project_dir: &Path) -> Result<(), String>`.
   - Scope: `src/agent/loop_driver.rs` only. Allowed edits are `sync_mcp_workspace`, the new private helper, and tests in the existing `#[cfg(test)] mod tests`; do not edit networking behavior, timeout values, request body format, status parsing, graph mutation code, generated auto-refactor JSON, or unrelated runtime/source files.
   - Done when: `sync_mcp_workspace` delegates only the `trim_end_matches('/')`, `http://` prefix check, host/port split, default-port handling, and port-parse error path to `parse_mcp_workspace_endpoint`; the original function still returns `Result<(), String>` and still constructs/sends the same `/workspace` request.
   - Validation: targeted validation passed on 2026-05-12: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test loop_driver::tests::mcp_workspace_endpoint_parser_preserves_host_port_and_errors -- --test-threads=1` ran the named parser test successfully with 0 failures. Broader `cargo fmt --check` is blocked by an unrelated `src/bin/agent.rs` formatting diff tracked by item 61.
61. [x] `src/bin/agent.rs`: apply rustfmt-compatible formatting to the existing `supervisor_reload_worker_port(...)` HTTP 204 fallback branch without changing behavior.
   - Scope: `src/bin/agent.rs` only. Allowed edit is the `match status { 204 => ... }` fallback expression inside `supervisor_reload_worker_port(...)`; do not edit `src/agent/loop_driver.rs`, runtime transition code, API routes, supervisor behavior, reload semantics, generated graph plans, or unrelated source files.
   - Done when: `cargo fmt --check` no longer reports the `src/bin/agent.rs` 204 fallback diff, and full-suite validation can run against the current working tree.
   - Validation: passed on 2026-05-12: `cargo fmt --check` returned green, then `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` passed with 269 library tests, 11 API server contract tests, 20 API transport contract tests, 3 canonical TLog contract tests, 4 domain contract tests, 10 graph mutation CLI contract tests, 9 MCP receipt contract tests, 2 planning contract tests, 5 score contract tests, 2 supervisor binary contract tests, 352 validation harness contract tests, 2 worker binary contract tests, and all listed zero-test binary/example harnesses passing with 0 failures.
62. [x] `src/agent/loop_driver.rs`: extract request-body construction from `sync_mcp_workspace` into private helper `build_mcp_workspace_request(host: &str, port: u16, project_dir: &Path) -> String` after item 61 is green.
   - Scope: `src/agent/loop_driver.rs` only. Allowed edits are `sync_mcp_workspace`, the new private helper, and tests in the existing `#[cfg(test)] mod tests`; do not edit URL parsing, network connection, timeout values, response parsing, generated auto-refactor JSON, or unrelated runtime/source files.
   - Done when: the helper produces the same `POST /workspace` HTTP request string, including `Host`, `Content-Type`, `Connection`, and `Content-Length`, and `sync_mcp_workspace` still sends the helper-produced bytes without changing its public signature.
   - Validation: passed on 2026-05-12: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test loop_driver::tests::mcp_workspace_request_builder_preserves_workspace_post -- --test-threads=1` passed with 1 test and 0 failures; `cargo fmt --check` returned green; `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` passed with 270 library tests, 11 API server contract tests, 20 API transport contract tests, 3 canonical TLog contract tests, 4 domain contract tests, 10 graph mutation CLI contract tests, 9 MCP receipt contract tests, 2 planning contract tests, 5 score contract tests, 2 supervisor binary contract tests, 352 validation harness contract tests, 2 worker binary contract tests, and all listed zero-test binary/example harnesses passing with 0 failures.

63. [x] `src/agent/loop_driver.rs`: validate the nonblocking persistent-loop change by keeping `LoopDriver::run_cycle(...)` free of synchronous browser-tab close calls after all turns complete.
   - Scope: `src/agent/loop_driver.rs` only. Allowed edits are limited to preserving/removing the post-cycle `close_router_tab(tag, "project", &mut router)` call and any now-unused private helper in this file; do not edit `sync_mcp_workspace`, prompt builders, generated auto-refactor JSON, lower-level `src/agent/router.rs` or `src/agent/cycle.rs` close APIs, domain code, graph mutation code, supervisor code, or unrelated runtime files.
   - Done when: `grep -n "close_router_tab\|close_current_tab\|browser tab close" src/agent/loop_driver.rs` produces no matches, lower-level router/cycle close APIs remain available, `run_cycle(...)` still returns `Ok(())` after all turns complete, and build validation proves the persistent-loop path compiles.
   - Validation: passed on 2026-05-12: `grep -n "close_router_tab\|close_current_tab\|browser tab close" src/agent/loop_driver.rs src/agent/cycle.rs src/agent/router.rs` found no matches in `src/agent/loop_driver.rs` and found lower-level close APIs only in `src/agent/cycle.rs` and `src/agent/router.rs`; `cargo check` passed; `cargo build --release` passed.

64. [x] `src/agent/cycle.rs`: inspect graph-backed `SplitFn` evidence for `agent::cycle::AgentCycle::run` and write the smallest helper-extraction implementation checklist item for that exact function.
   - Scope: `plan.md`, `status.md`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, and `src/agent/cycle.rs` inspection only; do not edit Rust source in this planning/inspection item.
   - Done when: the next implementation item names the exact helper boundary inside `AgentCycle::run`, the expected helper name, companion targeted validation, and confirms existing `AgentCycle::run` behavior and public signature are preserved.
   - Validation: passed on 2026-05-12 by inspecting `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json` for `SplitFn id=918a1611235eccfd`, `fn_path=agent::cycle::AgentCycle::run`, `expected_lo=3897`, `expected_hi=13467`, generated names `run__parse`/`run__transform`, and by inspecting `src/agent/cycle.rs` around `AgentCycle::run`.

65. [x] `src/agent/cycle.rs`: extract the Step 0 planning-turn block from `AgentCycle::run(...)` into a private method `run_planning_turn(&mut self, domain: &str, metric: &str) -> Result<(), CycleError>` while preserving the public `AgentCycle::run(&mut self) -> Result<AgentRunSummary, CycleError>` signature.
   - Scope: `src/agent/cycle.rs` only. Allowed edits are `AgentCycle::run(...)`, the new private `run_planning_turn(...)` method, and a targeted pure helper/unit test only if no router or worker mock is required; do not edit `src/agent/loop_driver.rs`, router APIs, worker APIs, prompt templates, domain code, graph mutation code, generated auto-refactor JSON, or unrelated runtime files.
   - Done when: `AgentCycle::run(...)` still validates nonempty objective, worker health, clones `domain` and `metric`, and then delegates only the existing Step 0 planning-turn message construction, router turn, `AgentStep` push, and `last_llm_output` assignment to `run_planning_turn(...)`; phase-dispatch loop behavior, evidence submission, stop reasons, final summary, and public API remain unchanged.
   - Validation: passed on 2026-05-12: `cargo fmt --check` passed; `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test cycle::hash_tests -- --test-threads=1` passed with 6 tests and 0 failures; `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` passed with 270 library tests, 11 API server contract tests, 20 API transport contract tests, 3 canonical TLog contract tests, 4 domain contract tests, 10 graph mutation CLI contract tests, 9 MCP receipt contract tests, 2 planning contract tests, 5 score contract tests, 2 supervisor binary contract tests, 352 validation harness contract tests, 2 worker binary contract tests, and all listed zero-test binary/example harnesses passing with 0 failures.

66. [x] `score/src/main.rs`: inspect graph-backed `SplitFn id=1676b1a0fb97f747` for `write_report(...)` and write the smallest helper-extraction implementation checklist item for that exact function.
   - Scope: `plan.md`, `status.md`, `state/rustc/auto-refactor/..__state__rustc__score__bin__graph.graph-editor-plan.json`, and `score/src/main.rs` inspection only; do not edit Rust source in this planning/inspection item.
   - Done when: the next implementation item names the exact helper boundary inside `write_report(...)`, the expected helper name, companion validation, and confirms the public `write_report(path, stats, agg, g, date) -> Result<()>` behavior is preserved.
   - Validation: passed on 2026-05-12 by inspecting `state/rustc/auto-refactor/..__state__rustc__score__bin__graph.graph-editor-plan.json` for `SplitFn id=1676b1a0fb97f747`, `fn_path=write_report`, `expected_lo=14328`, `expected_hi=17147`, generated names `write_report__parse`/`write_report__transform`, and `delegate_strategy=preserve_original_signature`, and by inspecting `score/src/main.rs` around `write_report(...)`.

67. [x] `score/src/main.rs`: extract aggregate-score section rendering from `write_report(...)` into private helper `write_aggregate_scores_section(buf: &mut Vec<u8>, agg: &[f64; 6], g: f64) -> Result<()>` while preserving the `write_report(...)` signature and generated Markdown bytes.
   - Scope: `score/src/main.rs` only. Allowed edits are `write_report(...)`, the new private helper, and a focused unit test only if it does not require broad fixture changes; do not edit score formulas, graph discovery, CLI argument parsing, generated auto-refactor JSON, `SCORE_REPORT.md`, domain code, agent runtime code, or validation scripts while selecting this item.
   - Done when: `write_report(...)` delegates only the `## Aggregate Scores` Markdown block, fenced text block, per-axis aggregate rows, blank line, and geometric-mean row to `write_aggregate_scores_section(...)`; report heading, generated metadata line, per-crate table, axis definitions table, file parent directory creation, and file write behavior remain in `write_report(...)`.
   - Validation: passed on 2026-05-12: `cargo fmt --check`; `cargo test --manifest-path score/Cargo.toml -- --test-threads=1` passed with 0 tests and 0 failures; initial report validation against `/tmp/canon-score-report-check.md` was blocked by `/tmp` disk quota (`os error 122`), then rerun with workspace path `target/test-tmp/canon-score-report-check.md` passed; generated `## Aggregate Scores` section matched `SCORE_REPORT.md`.

68. [x] `src/agent/router.rs`: inspect graph-backed `SplitFn id=1ea38f3cc5f37f85` for `agent::router::send_streaming_request` and write the smallest helper-extraction implementation checklist item for that exact function.
   - Scope: `plan.md`, `status.md`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, and `src/agent/router.rs` inspection only; do not edit Rust source in this planning/inspection item.
   - Done when: the next implementation item names the exact helper boundary inside `send_streaming_request(...)`, the expected helper name, companion targeted validation, and confirms the existing streaming transport behavior and function signature are preserved.
   - Validation: passed on 2026-05-12 by inspecting `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json` for `SplitFn id=1ea38f3cc5f37f85`, `fn_path=agent::router::send_streaming_request`, `expected_lo=15798`, `expected_hi=19829`, generated names `send_streaming_request__parse`/`send_streaming_request__transform`, and by inspecting `src/agent/router.rs` around `send_streaming_request(...)`.

69. [x] `src/agent/router.rs`: finish the existing streaming HTTP request-string extraction by correcting `tests::streaming_http_request_builder_preserves_post_headers_and_body` to assert the helper-produced `Content-Length` for body `{"model":"gpt-test","stream":true}` while preserving private helper `build_streaming_http_request(path: &str, host: &str, port: u16, body: &str) -> String` and `send_streaming_request(config, body, logger, stream_deadline_ms) -> Result<crate::agent::sse::SseResult, OpenAiError>`.
   - Scope: `src/agent/router.rs` only. Allowed edits are `send_streaming_request(...)`, the private helper, and the focused unit test in the existing `#[cfg(test)] mod tests`; do not edit endpoint parsing, TCP connection, timeout handling, SSE parsing, chunked decoding, retry policy, browser-tab behavior, generated auto-refactor JSON, domain code, graph mutation code, or unrelated runtime/source files.
   - Done when: `send_streaming_request(...)` still parses the local endpoint, computes `chat_completions_path(...)`, connects and sets read/write timeouts, writes and flushes the helper-produced bytes, preserves deadline/read loop behavior, preserves HTTP status and `[DONE]` checks, preserves chunked/non-chunked SSE body parsing, and the builder test expects the actual byte length of the JSON body.
   - Validation: passed on 2026-05-12 with `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test agent::router::tests::streaming_http_request_builder_preserves_post_headers_and_body -- --test-threads=1 && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`.

70. [x] `src/runtime/introspection.rs`: extract canonical evidence-line scanning from `introspect_canonical_tlog(path)` into private helper `scan_canonical_tlog_evidence(path: &Path) -> Result<CanonicalEvidenceScan, CanonError>` while preserving `introspect_canonical_tlog(path: impl AsRef<Path>) -> Result<CanonicalIntrospectionReport, CanonError>`.
   - Scope: `src/runtime/introspection.rs` only. Allowed edits are `introspect_canonical_tlog(...)`, a private scan-result struct, the private helper, and focused tests in `tests/canonical_tlog_contract.rs` only if existing assertions need to name the preserved behavior; do not edit TLog loading, replay, eval receipt encoding, worker-state detection semantics, CLI output formatting, graph mutation code, capability process execution, or auto-refactor JSON.
   - Done when: `introspect_canonical_tlog(...)` still calls `load_tlog_ndjson(path)?`, still reports `latest_phase`, `event_count`, `canonical_path`, and `worker_state` exactly as before, and delegates only the line-by-line extraction of `latest_evaluator_result`, `latest_validation_result`, and `latest_score_report_hash` to `scan_canonical_tlog_evidence(...)`.
   - Validation: passed on 2026-05-12 with `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test canonical_tlog_contract -- --test-threads=1 && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`.

71. [x] `src/agent/router.rs`: close the uncommitted item 69 router helper extraction state before starting another graph-backed split.
   - Scope: `src/agent/router.rs` only. Allowed actions are to keep the existing `build_streaming_http_request(path: &str, host: &str, port: u16, body: &str) -> String` extraction and focused test if validation passes, or revert only the `src/agent/router.rs` diff if validation fails; do not edit graph mutation code, process-executor code, runtime introspection, domain code, generated auto-refactor JSON, or unrelated files.
   - Done when: `git status --short` no longer reports an unstaged `src/agent/router.rs` diff, and the resulting repository state has either a committed validated router extraction or a cleanly reverted router file.
   - Validation: passed on 2026-05-12 with `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test agent::router::tests::streaming_http_request_builder_preserves_post_headers_and_body -- --test-threads=1 && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`; final pre-commit `git status --short` showed only `src/agent/router.rs` pending before the plan/status updates.

72. [x] `src/graph_mutation.rs`: inspect graph-backed `SplitFn id=f2de356b6e3f4b52` for `graph_mutation::generate_graph_patch` and write the smallest helper-extraction implementation checklist item for that exact function.
   - Scope: `plan.md`, `status.md`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, `src/graph_mutation.rs`, and existing graph mutation tests only; do not edit Rust source in this planning/inspection item.
   - Done when: the next implementation item names the exact helper boundary inside `generate_graph_patch(...)`, the expected helper name, companion targeted validation, and confirms the public `generate_graph_patch(graph, sources, ops) -> Result<GraphPatchPlan, GraphPatchError>` behavior is preserved.
   - Validation: passed on 2026-05-12 with `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test graph_mutation::tests -- --test-threads=1` after inspecting `SplitFn id=f2de356b6e3f4b52` and `src/graph_mutation.rs::generate_graph_patch(...)`.

73. [x] `src/graph_mutation.rs`: extract patch receipt construction from `generate_graph_patch(...)` into private helper `build_graph_patch_receipt(graph: &GraphSnapshotContract, sources: &[GraphSourceFile], sorted_ops: &[GraphMutationOp], diff: &str) -> GraphPatchReceipt` while preserving public `generate_graph_patch(graph, sources, ops) -> Result<GraphPatchPlan, GraphPatchError>` behavior.
   - Scope: `src/graph_mutation.rs` only; do not edit generated auto-refactor JSON, graph fixtures, CLI tests, runtime code, domain code, or planning files except to mark this item complete and record validation evidence after implementation.
   - Done when: `generate_graph_patch(...)` still owns empty-op/schema/source-map/sort/overlap/stale validation and diff/hunk assembly, delegates only receipt field construction/hash finalization to `build_graph_patch_receipt(...)`, and all existing graph mutation behavior remains unchanged.
   - Validation: passed on 2026-05-12 with `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test graph_mutation::tests -- --test-threads=1 && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`.

74. [x] `src/capability/tooling/record/process.rs`: inspect graph-backed `SplitFn id=8d96a0680113da93` for `capability::tooling::record::process::LiveSandboxProcessExecutor::execute_authorized_process` and write the smallest helper-extraction implementation checklist item for that exact function.
   - Scope: `plan.md`, `status.md`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, `src/capability/tooling/record/process.rs`, and existing sandbox process tests only; do not edit Rust source in this planning/inspection item.
   - Done when: the next implementation item names the exact helper boundary inside `execute_authorized_process(...)`, the expected helper name, companion targeted validation, and confirms sandbox process request/effect/receipt behavior is preserved.
   - Validation: passed on 2026-05-12 by inspecting `SplitFn id=8d96a0680113da93` (`fn_path=capability::tooling::record::process::LiveSandboxProcessExecutor::execute_authorized_process`, `expected_lo=6609`, `expected_hi=8939`, generated names `execute_authorized_process__parse` and `execute_authorized_process__transform`, `delegate_strategy=preserve_original_signature`), inspecting `src/capability/tooling/record/process.rs::execute_authorized_process(...)`, and confirming existing targeted process receipt validation through `live_sandbox_process_runner_records_and_replays_receipt`.

75. [x] `src/capability/tooling/record/process.rs`: extract the child-process timeout wait loop from `execute_authorized_process(...)` into private helper `wait_for_sandbox_process(child: &mut std::process::Child, timeout: Duration) -> Result<(u64, bool), ToolSandboxError>` while preserving `LiveSandboxProcessExecutor::execute_authorized_process(plan) -> Result<SandboxProcessEffect, ToolSandboxError>` behavior.
   - Scope: `src/capability/tooling/record/process.rs` only. Allowed edits are `execute_authorized_process(...)`, the private wait helper, and a focused unit test only if existing process receipt tests do not cover the preserved timeout/exit-status behavior; do not edit process authorization, sandbox path validation, receipt encoding/decoding, environment locking, artifact hashing, generated auto-refactor JSON, graph mutation code, domain code, runtime introspection, or unrelated files.
   - Done when: `execute_authorized_process(...)` still creates stdout/stderr files, spawns the locked-env process with null stdin, computes bounded stdout/stderr bytes and hashes, syncs the output directory, builds the same `Effect::process(...)`, and delegates only the `try_wait`/timeout/kill/wait/sleep loop to `wait_for_sandbox_process(...)`.
   - Validation: passed on 2026-05-12 with `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test live_sandbox_process_runner_records_and_replays_receipt -- --test-threads=1 && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`; targeted process receipt validation passed with 1 test and 0 failures, `cargo fmt --check` passed, and full-suite validation passed with 271 library tests, 11 API server contract tests, 20 API transport contract tests, 3 canonical TLog contract tests, 4 domain contract tests, 10 graph mutation CLI contract tests, 9 MCP receipt contract tests, 2 planning contract tests, 5 score contract tests, 2 supervisor binary contract tests, 352 validation harness contract tests, 2 worker binary contract tests, and all listed zero-test binary/example harnesses passing with 0 failures.

76. [x] `src/agent/loop_driver.rs`: inspect graph-backed `SplitFn id=d535999f445621fb` for `agent::loop_driver::sync_mcp_workspace` and write the smallest helper-extraction implementation checklist item for that exact function.
   - Scope: `plan.md`, `status.md`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, `src/agent/loop_driver.rs`, and existing MCP workspace helper tests only; do not edit Rust source in this planning/inspection item.
   - Done when: the next implementation item names the exact helper boundary inside `sync_mcp_workspace(...)`, the expected helper name, companion targeted validation, and confirms endpoint parsing, TCP connect/write/flush/read behavior, and public error strings are preserved.
   - Validation: passed on 2026-05-12 by inspecting `SplitFn id=d535999f445621fb` (`fn_path=agent::loop_driver::sync_mcp_workspace`, `expected_lo=22875`, `expected_hi=23972`, generated names `sync_mcp_workspace__parse` and `sync_mcp_workspace__transform`, `delegate_strategy=preserve_original_signature`), inspecting `src/agent/loop_driver.rs::sync_mcp_workspace(...)`, and confirming existing focused tests for MCP endpoint parsing and request building.

77. [x] `src/agent/loop_driver.rs`: extract HTTP response status parsing from `sync_mcp_workspace(...)` into private helper `parse_mcp_workspace_status(response: &str) -> u16` while preserving `sync_mcp_workspace(mcp_url, project_dir) -> Result<(), String>` behavior.
   - Scope: `src/agent/loop_driver.rs` only. Allowed edits are `sync_mcp_workspace(...)`, the private status parser helper, and focused tests for the helper; do not edit `LoopDriver::run_cycle`, `run_cycle_attempt(...)`, prompt builders, router code, generated auto-refactor JSON, domain code, graph mutation code, process executor code, or unrelated files.
   - Done when: `sync_mcp_workspace(...)` still parses the endpoint, connects to `(host, port)`, sets read/write timeouts, writes and flushes `build_mcp_workspace_request(...)`, reads the response into a string, rejects all statuses except 200 and 201 with the same `MCP workspace sync returned HTTP {status}` string, and delegates only the response first-line/status extraction to `parse_mcp_workspace_status(...)`.
   - Validation: passed on 2026-05-12 with `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test agent::loop_driver::tests::mcp_workspace_status_parser_accepts_success_and_defaults_invalid -- --test-threads=1`, `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test mcp_workspace -- --test-threads=1`, `cargo fmt --check`, and `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`; the originally written two-filter companion command was not valid Cargo syntax, so the equivalent `mcp_workspace` filter was used to run the endpoint, request-builder, and status-parser tests together.

78. [x] `src/agent/cycle.rs`: inspect graph-backed `SplitFn id=918a1611235eccfd` for `agent::cycle::AgentCycle::run` and write the smallest helper-extraction implementation checklist item for that exact function.
   - Scope: `plan.md`, `status.md`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, `src/agent/cycle.rs`, and existing `agent::cycle::hash_tests` only; do not edit Rust source in this planning/inspection item.
   - Done when: the next implementation item names the exact helper boundary inside `AgentCycle::run(...)`, the expected helper name, companion targeted validation, and confirms phase dispatch, human-review sentinel behavior, `submit_evidence(...)` calls, and success semantics remain preserved.
   - Validation: passed on 2026-05-12 by inspecting `SplitFn id=918a1611235eccfd` (`fn_path=agent::cycle::AgentCycle::run`, `expected_lo=3897`, `expected_hi=12284`, generated names `run__parse` and `run__transform`, `delegate_strategy=preserve_original_signature`), inspecting `src/agent/cycle.rs::AgentCycle::run(...)`, confirming the smallest safe next helper boundary is the `"Recovery"` match arm, and running `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test agent::cycle::hash_tests -- --test-threads=1`, which passed with 6 tests and 0 failures.

79. [x] `src/agent/cycle.rs`: extract the Recovery phase branch from `AgentCycle::run(...)` into private method `run_recovery_phase(&mut self, loop_steps: u64, domain: &str, state_body: &str) -> Result<Option<StopReason>, CycleError>` while preserving `AgentCycle::run() -> Result<AgentRunSummary, CycleError>` behavior.
   - Scope: `src/agent/cycle.rs` only. Allowed edits are the `"Recovery"` match arm in `AgentCycle::run(...)`, the private helper method, and focused unit tests only if existing recovery helper tests do not cover preserved behavior; do not edit router code, worker code, prompt text, generated auto-refactor JSON, domain code, graph mutation code, process executor code, or unrelated files.
   - Done when: the Recovery branch still reads `failure` and `recovery_action` from `state_body`, derives `target_phase` with `recovery_target_phase(...)`, calls `llm_phase_turn(...)` with `prompt::recovery_prompt(...)`, returns `StopReason::HumanReviewRequired` when the output contains `SENTINEL_REVIEW`, parses the verdict with `parse_verdict(...)`, selects explicit `recovery_action` before `recovery_action_for_failure(...)`, and submits the same recovery gate/evidence pair with the same `passed` value.
   - Validation: passed on 2026-05-12 with `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test agent::cycle::hash_tests -- --test-threads=1 && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`; targeted cycle validation passed with 6 tests and 0 failures, `cargo fmt --check` passed after rustfmt-compatible line wrapping, and full-suite validation passed with 272 library tests, 11 API server contract tests, 20 API transport contract tests, 3 canonical TLog contract tests, 4 domain contract tests, 10 graph mutation CLI contract tests, 9 MCP receipt contract tests, 2 planning contract tests, 5 score contract tests, 2 supervisor binary contract tests, 352 validation harness contract tests, and 2 worker binary contract tests, all with 0 failures.

80. [x] `src/agent/loop_driver.rs`: inspect graph-backed `SplitFn id=001e821dc83e940a` for `agent::loop_driver::LoopDriver::run_cycle` and write the smallest helper-extraction implementation checklist item for that exact function.
   - Scope: `plan.md`, `status.md`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, `src/agent/loop_driver.rs`, and existing loop-driver prompt tests only; do not edit Rust source in this planning/inspection item.
   - Done when: the next implementation item names the exact helper boundary inside `LoopDriver::run_cycle(...)`, the expected helper name, companion targeted validation, and confirms spawned/project turn selection, retry behavior, router request behavior, sleep cadence, and completion semantics are preserved.
   - Validation: passed on 2026-05-12 by inspecting `SplitFn id=001e821dc83e940a` (`fn_path=agent::loop_driver::LoopDriver::run_cycle`, `expected_lo=4201`, `expected_hi=7738`, generated names `run_cycle__parse` and `run_cycle__transform`, `delegate_strategy=preserve_original_signature`), inspecting `src/agent/loop_driver.rs::LoopDriver::run_cycle(...)`, confirming the smallest safe next helper boundary is the project-planning prompt context assembly block, and running `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test agent::loop_driver::tests::project_prompts_do_not_claim_to_be_worker_certification -- --test-threads=1`, which passed with 1 named loop-driver prompt test and 0 failures.

81. [x] `src/agent/loop_driver.rs`: extract planning-prompt context assembly from `LoopDriver::run_cycle(...)` into private helper `build_project_planning_prompt(goal: &str, agent_id: u32, agent_count: u32, working_dir: &Path) -> String` while preserving `LoopDriver::run_cycle(...) -> Result<(), String>` behavior.
   - Scope: `src/agent/loop_driver.rs` only. Allowed edits are the planning-prompt branch in `LoopDriver::run_cycle(...)`, the private helper, and focused tests for the helper if the existing prompt test does not cover the preserved SCORE_REPORT/auto-refactor context behavior; do not edit router code, cycle code, generated auto-refactor JSON, domain code, graph mutation code, process executor code, or unrelated files.
   - Done when: the project-planning branch still reads `SCORE_REPORT.md` from `self.config.working_dir`, still calls `auto_refactor_summary(&self.config.working_dir)`, still passes both optional report strings into `planning_prompt(...)`, and `LoopDriver::run_cycle(...)` still preserves spawned prompts, execute prompts, retry handling, `run_cycle_attempt(...)`, completion error strings, and inter-turn sleep behavior.
   - Validation: passed on 2026-05-12 with `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test agent::loop_driver::tests::project_prompts_do_not_claim_to_be_worker_certification -- --test-threads=1 && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`; targeted loop-driver prompt validation passed with 1 named test and 0 failures, `cargo fmt --check` passed, and full-suite validation passed with 272 library tests, 11 API server contract tests, 20 API transport contract tests, 3 canonical TLog contract tests, 4 domain contract tests, 10 graph mutation CLI contract tests, 9 MCP receipt contract tests, 2 planning contract tests, 5 score contract tests, 2 supervisor binary contract tests, 352 validation harness contract tests, 2 worker binary contract tests, and all listed zero-test binary/example harnesses passing with 0 failures.

82. [x] `src/agent/router.rs`: inspect graph-backed `SplitFn` evidence for `agent::router::send_streaming_request` and write the smallest helper-extraction implementation checklist item for that exact function.
   - Scope: `plan.md`, `status.md`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, `src/agent/router.rs`, and existing router streaming tests only; do not edit Rust source in this planning/inspection item.
   - Done when: the next implementation item names the exact helper boundary inside `send_streaming_request(...)`, the expected helper name/signature, companion targeted validation, and confirms endpoint parsing, socket timeout setup, request writing, chunk logging, done-frame timeout behavior, HTTP status handling, chunked decoding, SSE parsing, and public return/error behavior remain preserved.
   - Validation: passed on 2026-05-12 with equivalent focused router streaming filters because Cargo exact filtering accepts one filter pattern per invocation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test agent::router::tests::response_done_frame_detection_accepts_chunked_raw_body -- --test-threads=1`, `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test agent::router::tests::response_done_frame_detection_rejects_partial_stream -- --test-threads=1`, `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test agent::router::tests::streaming_http_request_builder_preserves_post_headers_and_body -- --test-threads=1`, and `cargo fmt --check`.

83. [x] `src/agent/router.rs`: extract streaming response finalization from `send_streaming_request(...)` into private helper `finalize_streaming_response(full_response: &[u8], logger: &mut ChunkLogger) -> Result<crate::agent::sse::SseResult, OpenAiError>` while preserving `send_streaming_request(config, body, logger, stream_deadline_ms) -> Result<crate::agent::sse::SseResult, OpenAiError>` behavior.
   - Scope: `src/agent/router.rs` only. Allowed edits are the response-head/body parsing block after the streaming read loop, the private helper, and focused tests for status, missing done-frame, chunked decoding fallback, and SSE parse behavior only if existing tests do not cover the preserved behavior; do not edit loop-driver code, cycle code, generated auto-refactor JSON, domain code, graph mutation code, process executor code, or unrelated files.
   - Done when: `send_streaming_request(...)` still parses local endpoints and chat-completion paths, sets the same read/write timeouts, writes the same helper-built HTTP request, logs the same `http_chunk` records during reads, stops on the same `[DONE]` and deadline conditions, and delegates only the post-read response parsing/status/chunked/SSE finalization to the helper.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test agent::router::tests::response_done_frame_detection_accepts_chunked_raw_body agent::router::tests::response_done_frame_detection_rejects_partial_stream agent::router::tests::streaming_http_request_builder_preserves_post_headers_and_body -- --test-threads=1 && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`, using equivalent focused router streaming filters if Cargo rejects multiple exact filters.
   - Completed on 2026-05-12: existing scoped extraction was validated with equivalent focused one-filter router commands for `response_done_frame_detection_accepts_chunked_raw_body`, `response_done_frame_detection_rejects_partial_stream`, and `streaming_http_request_builder_preserves_post_headers_and_body`; `cargo fmt --check` passed; full-suite `cargo test --all-targets` passed with 272 library tests plus all contract and binary/example harnesses passing with 0 failures.

84. [x] `src/agent/cycle.rs`: inspect graph-backed `SplitFn id=918a1611235eccfd` for `agent::cycle::AgentCycle::run` and record the smallest safe helper-extraction checklist item for the repeated LLM phase branch in that exact function.
   - Scope: `src/agent/cycle.rs`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, and existing `agent::cycle::hash_tests` only. Do not edit Rust source for this item except planning/status evidence.
   - Done when: the inspection confirms the remaining safe boundary after item 79 is the repeated `Analysis`/`Judgment`/`Plan`/`Eval` branch pattern only, and item 85 names the exact helper signature, allowed edits, and validation gates.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test agent::cycle::hash_tests -- --test-threads=1`.
   - Completed on 2026-05-12: inspection confirmed `SplitFn id=918a1611235eccfd` still applies to `AgentCycle::run(...)`; the remaining safe boundary is the repeated `Analysis`/`Judgment`/`Plan`/`Eval` LLM phase branch only. Targeted validation passed with 6 `agent::cycle::hash_tests` and 0 failures.

85. [x] `src/agent/cycle.rs`: extract the repeated LLM phase branch from `AgentCycle::run(...)` into private method `run_llm_gate_phase(&mut self, loop_steps: u64, phase: &str, prompt: String, invariant_submitted: &mut bool) -> Result<Option<StopReason>, CycleError>` while preserving `AgentCycle::run() -> Result<AgentRunSummary, CycleError>` behavior.
   - Scope: `src/agent/cycle.rs` only. Allowed edits are the `Analysis`, `Judgment`, `Plan`, and `Eval` match arms, the private helper, and narrowly focused cycle tests only if existing `hash_tests` do not cover the preserved evidence-gate behavior; do not edit router code, loop-driver code, graph mutation code, domain code, generated auto-refactor JSON, or worker/kernel files.
   - Done when: the helper performs the same `llm_phase_turn(...)`, `SENTINEL_REVIEW` detection, `parse_verdict(...)`, `phase_gate(...)` lookup, evidence submission, and one-time `Invariant/InvariantProof` submission for the `Analysis` branch, while the match arms still construct the same phase-specific prompts and all non-LLM branches remain outside the helper.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test agent::cycle::hash_tests -- --test-threads=1 && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`.
   - Completed on 2026-05-12: `run_llm_gate_phase(...)` now owns the shared LLM-turn, sentinel, verdict, phase-gate evidence, and Analysis invariant-submission workflow for `Analysis`, `Judgment`, `Plan`, and `Eval`; targeted cycle validation, `cargo fmt --check`, and full-suite `cargo test --all-targets` passed with 0 failures.

86. [x] `examples/openai_tool_loop_trace.rs`: inspect graph-backed `SplitFn id=9a947eeef45104ad` for `submit_openai_tool_calls` and record the smallest safe helper-extraction checklist item for the post-loop batch receipt submission boundary in that exact function.
   - Scope: `examples/openai_tool_loop_trace.rs`, `state/rustc/auto-refactor/..__state__rustc__openai_tool_loop_trace__bin__graph.graph-editor-plan.json`, and compile validation for the `openai_tool_loop_trace` example only. Do not edit Rust source for this item except planning/status evidence.
   - Done when: the inspection confirms the safe boundary is the post-loop batch hash, `CommandEnvelope::new(...)`, `Command::SubmitProcessReceiptBatch(receipts)`, `handle_envelope(...)`, and `Ok(TOOL_CALL_TARGET)` finalization block only, and item 87 names the exact helper signature, allowed edits, and validation gates.
   - Validation: `cargo check --example openai_tool_loop_trace`.
   - Completed on 2026-05-12: inspection confirmed `SplitFn id=9a947eeef45104ad` still applies to `submit_openai_tool_calls(...)`; the safe boundary is the post-loop batch hash, `CommandEnvelope::new(...)`, `Command::SubmitProcessReceiptBatch(receipts)`, `handle_envelope(...)`, and `Ok(TOOL_CALL_TARGET)` finalization block only. `cargo check --example openai_tool_loop_trace` passed and refreshed the example witness with 13 nodes, 283 facts, and graph hash `61de00d234554c40154449c60a8296e0d7caebc657d09e3d5665e0661f857848`.

87. [x] `examples/openai_tool_loop_trace.rs`: extract post-loop process-receipt batch submission from `submit_openai_tool_calls(...)` into private helper `submit_openai_process_receipt_batch(receipts: Vec<SandboxProcessReceipt>, state: &mut State, tlog: &mut TLog, cfg: RuntimeConfig) -> Result<usize, Box<dyn std::error::Error>>` while preserving `submit_openai_tool_calls(...) -> Result<usize, Box<dyn std::error::Error>>` behavior.
   - Scope: `examples/openai_tool_loop_trace.rs` only. Allowed edits are the final batch-hash/envelope/handle block in `submit_openai_tool_calls(...)` and the private helper; do not edit `OpenAiClient`, tool-intent request construction, `matches_tool_intent(...)`, sandbox executor configuration, per-call receipt persistence, `submit_openai_tool_result(...)`, generated auto-refactor JSON, library kernel code, domain code, router code, or unrelated examples.
   - Done when: `submit_openai_tool_calls(...)` still creates the same sandbox executor, iterates the same `TOOL_CALL_TARGET` range, validates the same LLM intent, executes and persists each sandbox receipt, submits the same tool-result call, and delegates only final batch hash/envelope submission and `Ok(TOOL_CALL_TARGET)` return to the helper.
   - Validation: `cargo check --example openai_tool_loop_trace && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`.
   - Completed on 2026-05-12: `submit_openai_tool_calls(...)` now delegates only post-loop process-receipt batch finalization to `submit_openai_process_receipt_batch(...)`; `cargo check --example openai_tool_loop_trace`, `cargo fmt --check`, and full-suite `cargo test --all-targets` passed with 0 failures. The example witness refreshed from 13 nodes/283 facts to 14 nodes/292 facts with graph hash `23d26cca969393a1d6c2b9fc306d9f4491dec53e9b7a1397c506b8b26dff41e4`.

88. [x] `examples/ollama_tool_loop_trace.rs`: inspect graph-backed `SplitFn id=a0f85bbd8f5f9cf1` for `submit_ollama_tool_calls` and record the smallest safe helper-extraction checklist item for the post-loop MCP evidence submission boundary in that exact function.
   - Scope: `examples/ollama_tool_loop_trace.rs`, `state/rustc/auto-refactor/..__state__rustc__ollama_tool_loop_trace__bin__graph.graph-editor-plan.json`, and compile validation for the `ollama_tool_loop_trace` example only. Do not edit Rust source for this item except planning/status evidence.
   - Done when: inspection confirms the safe boundary is the post-loop `combined_mcp_execution_receipt(&receipts)?`, `CommandEnvelope::new(...)`, `Command::SubmitEvidence(receipt.submission())`, `handle_envelope(...)`, and `Ok(TOOL_CALL_TARGET)` finalization block only, and item 89 names the exact helper signature, allowed edits, and validation gates.
   - Validation: `cargo check --example ollama_tool_loop_trace`.
   - Completed on 2026-05-12: inspection confirmed `SplitFn id=a0f85bbd8f5f9cf1` still applies to `submit_ollama_tool_calls(...)`; the safe boundary is the post-loop `combined_mcp_execution_receipt(&receipts)?`, `CommandEnvelope::new(...)`, `Command::SubmitEvidence(receipt.submission())`, `handle_envelope(...)`, and `Ok(TOOL_CALL_TARGET)` finalization block only. `cargo check --example ollama_tool_loop_trace` passed and refreshed the example witness with 10 nodes, 220 facts, and graph hash `fd1ffb7865915988d2fc6c3a0470899d9b8bbff24000ebe32b61cb6bfe07db0e`.

89. [x] `examples/ollama_tool_loop_trace.rs`: extract post-loop MCP evidence submission from `submit_ollama_tool_calls(...)` into private helper `submit_ollama_mcp_evidence(receipts: &[McpCallReceipt], state: &mut State, tlog: &mut TLog, cfg: RuntimeConfig) -> Result<usize, Box<dyn std::error::Error>>` while preserving `submit_ollama_tool_calls(...) -> Result<usize, Box<dyn std::error::Error>>` behavior.
   - Scope: `examples/ollama_tool_loop_trace.rs` only. Allowed edits are the final combined-receipt/envelope/handle block in `submit_ollama_tool_calls(...)` and the private helper; do not edit `OllamaClient`, tool-intent request construction, `matches_tool_intent(...)`, MCP executor configuration, per-call receipt persistence, `combined_mcp_execution_receipt(...)`, generated auto-refactor JSON, library kernel code, domain code, router code, or unrelated examples.
   - Done when: `submit_ollama_tool_calls(...)` still creates the same MCP executor, iterates the same `TOOL_CALL_TARGET` range, validates the same Ollama tool intent, executes and persists each MCP receipt, and delegates only final successful MCP receipt selection, evidence envelope submission, and `Ok(TOOL_CALL_TARGET)` return to the helper.
   - Validation: `cargo check --example ollama_tool_loop_trace && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`.
   - Completed on 2026-05-12: `submit_ollama_tool_calls(...)` now delegates only post-loop successful MCP receipt selection, evidence envelope submission, and `Ok(TOOL_CALL_TARGET)` return to `submit_ollama_mcp_evidence(...)`; `cargo check --example ollama_tool_loop_trace`, `cargo fmt --check`, and full-suite `cargo test --all-targets` passed with 0 failures. The example witness refreshed from 10 nodes/220 facts to 11 nodes/228 facts with graph hash `d417a51331ca0c0e6c085a0f45a6ce2a5bb82052bcbe006647e9f0b67bc1f01e`.


90. [x] `examples/ollama_tool_mcp_loop_trace.rs`: inspect graph-backed `SplitFn id=bf48e302ecddd798` for `submit_llm_mcp_tool_calls` and record the smallest safe helper-extraction checklist item for the post-loop evidence-finalization boundary in that exact function.
   - Scope: `examples/ollama_tool_mcp_loop_trace.rs`, the matching auto-refactor JSON plan, `plan.md`, and `status.md` only.
   - Done when: inspection confirms the safe boundary and item 91 names the exact helper signature, allowed edits, and validation gates.
   - Validation: `cargo check --example ollama_tool_mcp_loop_trace`.
   - Completed on 2026-05-12: inspection confirmed `SplitFn id=bf48e302ecddd798` still applies to `submit_llm_mcp_tool_calls(...)`; the safe boundary is the post-loop `combined_mcp_execution_receipt(&receipts)?`, `CommandEnvelope::new(...)`, `Command::SubmitEvidence(receipt.submission())`, `handle_envelope(...)`, and `Ok(TOOL_CALL_TARGET)` finalization block only. `cargo check --example ollama_tool_mcp_loop_trace` passed and refreshed the example witness with 15 nodes, 293 facts, and graph hash `3daf7128cac0a0888a62f0c1565cd6eb904ea5ab1160505d2bccd2dd3711913c`.
91. [x] `examples/ollama_tool_mcp_loop_trace.rs`: extract post-loop evidence finalization from `submit_llm_mcp_tool_calls(...)` into private helper `submit_llm_mcp_evidence(receipts: &[McpCallReceipt], state: &mut State, tlog: &mut TLog, cfg: RuntimeConfig) -> Result<usize, Box<dyn std::error::Error>>` while preserving `submit_llm_mcp_tool_calls(...) -> Result<usize, Box<dyn std::error::Error>>` behavior.
   - Scope: `examples/ollama_tool_mcp_loop_trace.rs` only.
   - Done when: the parent function still performs the same request construction, validation, execution, and per-call persistence, and delegates only final receipt selection, evidence submission, and count return to the helper.
   - Validation: `cargo check --example ollama_tool_mcp_loop_trace && cargo fmt --check && cargo test --all-targets`.
   - Completed on 2026-05-12: `submit_llm_mcp_tool_calls(...)` now delegates only post-loop successful MCP receipt selection, evidence envelope submission, and `Ok(TOOL_CALL_TARGET)` return to `submit_llm_mcp_evidence(...)`; `cargo check --example ollama_tool_mcp_loop_trace`, `cargo fmt --check`, and full-suite `cargo test --all-targets` passed with 0 failures. The example witness refreshed from 15 nodes/293 facts to 16 nodes/301 facts with graph hash `40d42b1a4c1c9e0d490b18d0544f1ebb4e38445c84a0acb3017619ee17468509`.

92. [x] `../chatgpt-mcp-connector/src/tools.rs`: inspect graph-backed `SplitFn id=ce8d33f07ffdaae6` for `tools::shell` and record the smallest safe helper-extraction checklist item for that exact function.
   - Scope: `../chatgpt-mcp-connector/src/tools.rs`, `state/rustc/auto-refactor/..__state__rustc__chatgpt_mcp_connector__bin__graph.graph-editor-plan.json`, `plan.md`, and `status.md` only.
   - Done when: inspection confirms the safe helper boundary and item 93 names the exact helper signature, allowed edits, and validation gates without moving workspace authority, shell command execution authority, timeout semantics, or MCP response schema beyond the chosen boundary.
   - Validation: `cd ../chatgpt-mcp-connector && cargo check`.
   - Completed on 2026-05-12: inspection confirmed `SplitFn id=ce8d33f07ffdaae6` still applies to `tools::shell` with generated names `shell__parse` and `shell__transform`. The smallest safe extraction boundary is the post-execution output rendering and JSON metadata construction after process wait and stdout/stderr reader joins. `cd ../chatgpt-mcp-connector && cargo check` passed.
93. [x] `../chatgpt-mcp-connector/src/mcp_guard.rs`: resolve the out-of-scope MCP receipt-wrapper blocker by restoring `ai_worker_port()` to environment-only behavior, removing the default `Some(9100)` path that forces local planning shell calls through supervisor receipt posting when `AI_WORKER_PORT` is unset.
   - Scope: `../chatgpt-mcp-connector/src/mcp_guard.rs` only. Do not edit `../chatgpt-mcp-connector/src/tools.rs`, worker command schemas, receipt hashes, command payload tags, authorization/receipt post bodies, or root project source files for this item.
   - Done when: `git -C ../chatgpt-mcp-connector diff -- src/mcp_guard.rs` is empty or contains only the env-only `ai_worker_port()` implementation already present in `HEAD`, and the MCP shell tool can run a trivial command without returning `MCP tool call executed but receipt recording failed: worker returned HTTP 400: {"ok":false,"error":"InvalidCommand"}`.
   - Validation: `git -C ../chatgpt-mcp-connector diff -- src/mcp_guard.rs && cd ../chatgpt-mcp-connector && cargo check`.
   - Completed on 2026-05-12: `../chatgpt-mcp-connector/src/mcp_guard.rs` was restored to env-only `AI_WORKER_PORT` behavior with no remaining diff; the MCP shell tool returned normal command output after the restore, and `cd ../chatgpt-mcp-connector && cargo check` passed with refreshed connector witness graph hash `741e0f8999db380671fbac6d96f6e6c347c9da8f50b2ef855a59eac4f539ae1b`.

94. [x] `../chatgpt-mcp-connector/src/tools.rs`: complete validation for the existing post-execution shell output rendering extraction from `shell(args, workspace)` into private helper `render_shell_response(exit: i32, success: bool, timed_out: bool, timeout_ms: u64, stdout_bytes: &[u8], stdout_truncated: bool, stderr_bytes: &[u8], stderr_truncated: bool) -> Value` while preserving the public MCP shell tool behavior.
   - Scope: `../chatgpt-mcp-connector/src/tools.rs` only.
   - Done when: `shell(...)` still validates `command`, `timeout_ms`, `max_output_bytes`, resolves `cwd` through `WorkspaceConfig`, spawns `/bin/sh -c` in the resolved workspace directory, preserves bounded stdout/stderr capture, timeout/kill behavior, pipe-reader join error handling, exit metadata, truncation metadata, and JSON response fields, delegates only the post-join `stdout`/`stderr` text assembly plus final successful/error JSON object construction to `render_shell_response(...)`, and the required validation returns normal green Rust output instead of an MCP receipt-wrapper error.
   - Validation: `cd ../chatgpt-mcp-connector && cargo check && cargo test`.
   - Completed on 2026-05-12: `shell(...)` delegates only post-join stdout/stderr text assembly and final JSON response construction to `render_shell_response(...)`; `cargo check` passed, the first `cargo test` attempt failed from `/tmp` quota exhaustion (`Disk quota exceeded (os error 122)`), and `TMPDIR="$PWD/target/test-tmp" cargo test` passed with 533 tests and 0 failures.


95. [x] `src/agent/loop_driver.rs`: inspect the current `LoopDriver::run_cycle(...)` body and record the smallest safe helper-extraction boundary for per-turn mode, label, and prompt derivation.
   - Scope: `src/agent/loop_driver.rs`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, `plan.md`, and `status.md` only. Do not edit Rust source for this inspection item except planning/status evidence.
   - Done when: inspection confirms whether the safe boundary is a pure helper such as `build_turn_prompt_context(...)` or equivalent that derives `effective_turn`, `turn_mode`, `is_planning`, `label`, and `prompt` without moving retry loops, router streaming, sleep timing, cycle error handling, spawned-agent exit semantics, or MCP workspace sync behavior.
   - Validation: `cargo check`.
   - Completed on 2026-05-12: source inspection confirmed the smallest safe boundary is a private pure helper such as `build_turn_prompt_context(...)` that accepts spawned/project state, turn index, turn offset, goal text, `agent_id`, `agent_count`, working directory, optional spawned domain, and optional spawned metric, and returns only `effective_turn`, `turn_mode`, `label`, and `prompt`. Item 96 must leave `total_turns`/`turn_offset` calculation, non-spawned `GOAL.md` read, retry loop, `run_cycle_attempt(...)`, router streaming, sleep timing, cycle error handling, spawned-agent exit semantics, and MCP workspace sync behavior in their current callers. `cargo check` passed.

96. [x] `src/agent/loop_driver.rs`: extract per-turn prompt/label derivation from `LoopDriver::run_cycle(...)` into a private helper while preserving `run_cycle(...) -> Result<(), String>` behavior.
   - Scope: `src/agent/loop_driver.rs` only. Allowed edits are the pre-attempt per-turn derivation block inside `LoopDriver::run_cycle(...)`, a private helper and any private helper struct/tuple required to return the derived values. Do not edit `run_cycle_attempt(...)`, retry safety semantics, `RouterClient`, prompt builders, MCP workspace sync helpers, `LoopDriver::run_agent_loop(...)`, domain modules, generated auto-refactor JSON, tests outside this file, or scoring code.
   - Done when: `run_cycle(...)` still computes the same spawned/non-spawned turn counts, reads `GOAL.md` only for non-spawned project agents, logs the same label/mode/cycle data, retries the same `run_cycle_attempt(...)` calls with the same prompt string, preserves sleep timing, and delegates only deterministic per-turn label/mode/prompt construction to the helper.
   - Validation: `cargo check && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`.
   - Completed on 2026-05-12: `LoopDriver::run_cycle(...)` now delegates only deterministic per-turn context construction to private `build_turn_prompt_context(...)` and `TurnPromptContext`, while preserving turn counts, non-spawned `GOAL.md` read, retry loop, `run_cycle_attempt(...)`, router streaming, sleep timing, error handling, spawned-agent behavior, and MCP workspace sync outside the helper. Validation passed after applying `cargo fmt`: `cargo check`, `cargo fmt --check`, and `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` passed.

97. [x] `src/agent/loop_driver.rs`: inspect `sync_mcp_workspace(mcp_url, project_dir)` and record the smallest safe helper-extraction boundary for transport/status handling.
   - Scope: `src/agent/loop_driver.rs`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, `plan.md`, and `status.md` only. Do not edit Rust source for this inspection item except planning/status evidence.
   - Done when: inspection confirms whether the safe boundary is a private helper such as `send_mcp_workspace_request(host, port, request)` or equivalent that owns only TCP connect, timeout setup, request write/flush, response read, and string response return, while leaving endpoint parsing, request construction, status parsing, accepted status check, and public error wording in `sync_mcp_workspace(...)`.
   - Validation: `cargo check`.
   - Completed on 2026-05-12: source inspection confirmed the smallest safe boundary is a private helper such as `send_mcp_workspace_request(host: &str, port: u16, request: &str) -> Result<String, String>` that owns TCP connect, read/write timeout setup, request write/flush, response read, and raw response return. Item 98 must leave `parse_mcp_workspace_endpoint(...)`, `build_mcp_workspace_request(...)`, `parse_mcp_workspace_status(...)`, the `200`/`201` accepted-status check, and the existing public endpoint/port/write/status error wording in `sync_mcp_workspace(...)`. `cargo check` passed.

98. [x] `src/agent/loop_driver.rs`: extract `sync_mcp_workspace(...)` TCP request/response transport into one private helper without changing endpoint parsing, HTTP request construction, status acceptance, or error semantics.
   - Scope: `src/agent/loop_driver.rs` only. Allowed edits are `sync_mcp_workspace(...)` and one private helper such as `send_mcp_workspace_request(host: &str, port: u16, request: &str) -> Result<String, String>`. Do not edit `LoopDriver::run_all_agents(...)`, `LoopDriver::run_cycle(...)`, prompt builders, router streaming, domain modules, generated auto-refactor JSON, tests outside this file, or scoring code.
   - Done when: `sync_mcp_workspace(...)` still parses the same endpoint, builds the same request body/header, treats only HTTP `200` and `201` as success, returns the same invalid endpoint/port/write/status error prefixes, and delegates only TCP connect, timeout setup, request write/flush, and response read to the new helper.
   - Validation: `cargo check && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test loop_driver::tests -- --test-threads=1`.
   - Completed on 2026-05-12: `sync_mcp_workspace(...)` now delegates only TCP connect, timeout setup, request write/flush, response read, and raw response return to private `send_mcp_workspace_request(...)`. Endpoint parsing, HTTP request construction, status parsing, accepted `200`/`201` status handling, and existing endpoint/port/write/status error prefixes remain unchanged in the original call path. Targeted validation passed: `cargo check`, `cargo fmt --check`, and `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test loop_driver::tests -- --test-threads=1` ran 6 loop-driver tests successfully; broader `cargo test --all-targets` passed with 272 library/bin tests, integration suites, 352 root-validation tests, and worker binary tests.

99. [x] `src/agent/loop_driver.rs`: add focused unit tests for MCP workspace endpoint/status/request helpers.
   - Scope: `src/agent/loop_driver.rs` test module only. Do not edit production logic except to make private helpers testable within the module if needed.
   - Done when: tests cover `parse_mcp_workspace_endpoint(...)` default-port and explicit-port behavior, invalid non-HTTP scheme or invalid port rejection, `parse_mcp_workspace_status(...)` success and malformed-response behavior, and `build_mcp_workspace_request(...)` content-length/body consistency.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test loop_driver::tests -- --test-threads=1`.
   - Completed on 2026-05-12: the `loop_driver` test module now covers MCP workspace endpoint explicit-port/default-port parsing, invalid scheme/port rejection, HTTP status success/malformed handling, and request header/body separation with parsed `Content-Length` equality to the actual body length. Targeted validation passed with 6 `loop_driver::tests`; broader `cargo test --all-targets` passed with 272 library/bin tests, integration suites, 352 root-validation tests, and worker binary tests.

100. [x] `SCORE_REPORT.md`: refresh graph-derived structural evidence after items 97-99 land, then review whether `score.md` rationale should change without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `status.md`, and any deterministic score/graph refresh command output required by the existing project workflow. Do not edit runtime/domain source for this evidence item.
   - Done when: refreshed structural evidence is recorded or a blocker is documented, and `score.md` is either unchanged with explicit rationale or updated only if new evidence justifies it.
   - Validation: `git diff -- SCORE_REPORT.md score.md status.md plan.md`.
   - Completed on 2026-05-12: refreshed graph-derived score evidence with `cargo run --manifest-path score/Cargo.toml --quiet -- --artifact-root state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`. The report remained current with `G = 7.93 / 10`, Architecture `9.0`, Structure `4.8`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`; the command skipped the two schema-version-12 canon-rustc graphs as expected. `score.md` project-level numeric scores and rationale remain unchanged because the refresh is structural evidence only and does not prove a score-history-worthy capability change.


101. [x] `src/agent/router.rs`: inspect graph-backed `agent::router::send_streaming_request(...)` and record the smallest safe helper-extraction boundary for streaming HTTP transport byte collection.
   - Scope: `src/agent/router.rs`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, `plan.md`, and `status.md` only.
   - Done when: source inspection records the exact helper boundary and confirms endpoint parsing, chat-completions path selection, request construction, final status/body parsing, SSE decoding, and public error semantics stay outside the extracted helper.
   - Validation: `cargo check`.
   - Completed on 2026-05-12: source inspection confirmed the item 102 boundary is one private helper that owns only TCP connect, read/write timeout setup, request write/flush, response byte accumulation, `[DONE]` detection, chunk logging, and timeout/EOF handling. `send_streaming_request(...)` must keep endpoint parsing, chat-completions path selection, request construction, and the call to `finalize_streaming_response(...)`; `finalize_streaming_response(...)` must keep final status/body parsing, chunked decoding, missing-done handling, SSE decoding, and public error semantics. The live graph-backed candidate remains `SplitFn id=1ea38f3cc5f37f85` for `agent::router::send_streaming_request` with generated parse/transform names, but item 102 should use this narrower manual boundary. `cargo check` passed.
102. [x] `src/agent/router.rs`: extract streaming socket write/read/deadline byte collection from `send_streaming_request(...)` into one private helper `collect_streaming_response_bytes(endpoint_host: &str, endpoint_port: u16, write_timeout_ms: u64, request: &str, logger: &mut ChunkLogger, stream_deadline_ms: u64) -> Result<Vec<u8>, OpenAiError>`.
   - Scope: `src/agent/router.rs::send_streaming_request(...)` and one new private helper only.
   - Done when: `send_streaming_request(...)` still owns endpoint validation, path derivation, request construction, and `finalize_streaming_response(...)`, while the helper owns TCP connect, read/write timeout setup, request write/flush, response byte accumulation, `[DONE]` detection, chunk logging, and timeout/EOF handling without changing behavior.
   - Validation: `cargo check && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test router::tests -- --test-threads=1`.
   - Completed on 2026-05-12: `send_streaming_request(...)` now keeps endpoint validation, chat-completions path derivation, request construction, and `finalize_streaming_response(...)`; private `collect_streaming_response_bytes(...)` owns TCP connect, read/write timeout setup, request write/flush, response byte accumulation, `[DONE]` detection, chunk logging, and timeout/EOF handling. Targeted validation passed after `cargo fmt`; broader `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` also passed.
103. [x] `src/agent/router.rs`: replace the brittle full-string-only `streaming_http_request_builder_preserves_post_headers_and_body` assertion with parsed header/body assertions for `build_streaming_http_request(...)`.
   - Scope: `src/agent/router.rs` test module only. Do not edit production logic unless compilation requires a private helper used only by tests.
   - Done when: the test splits the request at `\r\n\r\n`, asserts method/path/host/content-type/accept/connection headers, parses `Content-Length`, and proves it equals both `body.len()` and the actual body bytes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test router::tests::streaming_http_request_builder_preserves_post_headers_and_body -- --test-threads=1`.
   - Completed on 2026-05-12: the test now parses the generated streaming request into header and body sections, asserts the request line plus Host/Content-Type/Accept/Connection headers, parses numeric `Content-Length`, and verifies it equals both `body.len()` and the actual body byte length. Targeted validation passed with 1 named router test; broader `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` also passed.
104. [x] `src/agent/router.rs`: add in-memory success coverage for `finalize_streaming_response(...)` using a 200 HTTP response with SSE content and a terminating `data: [DONE]` frame.
   - Scope: `src/agent/router.rs` test module only. Do not open sockets, call live router endpoints, or edit production transport logic.
   - Done when: a named test builds an in-memory HTTP response, calls `finalize_streaming_response(response.as_bytes(), &mut logger)`, and asserts the parsed SSE result content/metadata expected by the existing `parse_sse_body(...)` behavior.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test router::tests::finalize_streaming_response_accepts_in_memory_sse_done_response -- --test-threads=1`.
   - Completed on 2026-05-12: `finalize_streaming_response_accepts_in_memory_sse_done_response` builds an in-memory 200 SSE response containing two chat completion content frames, an `x-turn` metadata frame, and a terminating `data: [DONE]` frame; it asserts parsed content, target URL, completion metadata, finish reason, `done`, `is_complete()`, and `completion_reason()`. Targeted validation passed with 1 named router test.
105. [x] `src/agent/router.rs`: add in-memory error coverage for `finalize_streaming_response(...)` non-200 and missing-`[DONE]` paths.
   - Scope: `src/agent/router.rs` test module only. Do not open sockets, call live router endpoints, or edit production transport logic.
   - Done when: named tests prove a non-200 status returns `OpenAiError::HttpStatus(status)` and a 200 response without a terminating `data: [DONE]` frame returns an `OpenAiError::Io` with `UnexpectedEof`.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test router::tests -- --test-threads=1`.
   - Completed on 2026-05-12: added `finalize_streaming_response_rejects_non_200_status` and `finalize_streaming_response_rejects_missing_done_frame` as in-memory tests. Targeted validation passed with 15 router tests; broader `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` passed with 275 library/bin tests, integration suites, 352 root-validation tests, and worker binary tests.
106. [x] `SCORE_REPORT.md`: refresh graph-derived structural evidence after items 101-105 land, then review whether `score.md` rationale should change without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `status.md`, and `plan.md` only.
   - Done when: `SCORE_REPORT.md` is regenerated or confirmed current, `score.md` rationale is updated only if warranted by evidence, and `status.md` records the structural score evidence.
   - Validation: `cargo run --manifest-path score/Cargo.toml --quiet -- --artifact-root state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"` plus `git diff -- SCORE_REPORT.md score.md status.md plan.md`.
   - Completed on 2026-05-12: refreshed graph-derived score evidence after router items 101-105. `SCORE_REPORT.md` reports `G = 7.94 / 10` across 16 crates with 2 schema-version-12 skips; axes remain Architecture `9.0`, Structure `4.8`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`. `score.md` project-level numeric scores remain unchanged because this structural refresh does not prove a score-history-worthy capability change.
107. [x] `src/agent/router.rs`: inspect live graph-backed `agent::router::collect_streaming_response_bytes(...)` and record the exact helper boundary for the next execution item.
   - Scope: `src/agent/router.rs`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, `plan.md`, and `status.md` only. Do not edit Rust source for this inspection item except planning/status evidence.
   - Done when: inspection confirms `SplitFn id=c07f9b3fef3c6e37` still targets `agent::router::collect_streaming_response_bytes(...)`, rejects generated names `collect_streaming_response_bytes__parse`/`collect_streaming_response_bytes__transform` as direct instructions, and records this manual item-108 boundary: isolate socket connect, read/write timeout setup, request `write_all`/`flush`, and return the connected/flushed `TcpStream`; leave response accumulation, chunk logging, `[DONE]` detection, stream-deadline handling, EOF behavior, and all public `OpenAiError` semantics inside `collect_streaming_response_bytes(...)`.
   - Validation: `cargo check`.
   - Completed on 2026-05-12: source inspection confirmed `collect_streaming_response_bytes(...)` still performs TCP connect, fixed 1,000 ms read timeout setup, configured write timeout setup, request `write_all`/`flush`, response byte accumulation, chunk logging, `[DONE]` detection, deadline/timeout handling, EOF break behavior, and final byte return in one function. Python JSON inspection of the auto-refactor plan confirmed `SplitFn id=c07f9b3fef3c6e37` still targets `agent::router::collect_streaming_response_bytes` with generated names `collect_streaming_response_bytes__parse`/`collect_streaming_response_bytes__transform`; those names remain evidence only. Item 108 is the safe manual extraction: `open_streaming_http_stream(...)` owns only socket connect, read/write timeout setup, request write, and flush. `cargo check` passed and refreshed the `ai` witness to 5,429 nodes, 34,817 facts, graph hash `c7701f6d91f60fa6800ffeeeee142b5e50b516cfb0c6b85ea3b5d62a7bc4e0c6`.
108. [x] `src/agent/router.rs`: extract private helper `open_streaming_http_stream(...)` from `collect_streaming_response_bytes(...)` for socket setup and request write/flush only.
   - Scope: `src/agent/router.rs::collect_streaming_response_bytes(...)` and one private helper in the same file. Do not change `send_streaming_request(...)`, `finalize_streaming_response(...)`, SSE parsing, chunked decoding, done-frame checks, or public error variants.
   - Done when: `collect_streaming_response_bytes(...)` delegates only TCP connect, `set_read_timeout`, `set_write_timeout`, `write_all`, and `flush` to the helper, while the read loop still owns `full_response`, chunk logging, deadline checks, timeout handling, and `[DONE]` detection.
   - Validation: `cargo check && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test router::tests -- --test-threads=1`.
   - Completed on 2026-05-12: `collect_streaming_response_bytes(...)` now delegates only TCP connect, fixed 1,000 ms read timeout setup, configured write timeout setup, request `write_all`, and `flush` to private helper `open_streaming_http_stream(...)`. The read loop still owns `full_response`, chunk logging, stream deadline checks, timeout handling, EOF behavior, and `[DONE]` detection; `send_streaming_request(...)`, `finalize_streaming_response(...)`, SSE parsing, chunked decoding, done-frame checks, and public error variants were unchanged. Targeted validation passed after `cargo fmt`: `cargo check`, `cargo fmt --check`, and `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test router::tests -- --test-threads=1` ran 15 router tests successfully. Broader validation passed: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` passed with 275 library/bin tests, integration suites, 352 root-validation tests, and worker binary tests.
109. [x] `src/agent/router.rs`: add focused in-memory/unit coverage for the extracted `open_streaming_http_stream(...)` behavior without live router endpoints.
   - Scope: `src/agent/router.rs` test module only. Use a local `TcpListener` test server or an equivalent in-process loopback fixture; do not call external network services or modify production behavior.
   - Done when: a named router test proves the helper sends the exact request bytes to a loopback listener and the existing router tests still cover finalization success/error behavior.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test router::tests -- --test-threads=1`.
   - Completed on 2026-05-12: added `open_streaming_http_stream_sends_exact_request_to_loopback_listener`, which binds a local `TcpListener`, calls `open_streaming_http_stream(...)`, reads the exact request byte count on the loopback server thread, and asserts the observed bytes equal the request built by `build_streaming_http_request(...)`. Targeted validation passed with 16 router tests. Broader `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets` passed with 276 library/bin tests, integration suites, 352 root-validation tests, and worker binary tests.
110. [x] `SCORE_REPORT.md`: refresh graph-derived structural evidence after items 107-109 and review score rationale without changing project-level scores unless new capability evidence exists.
   - Scope: `SCORE_REPORT.md`, `score.md`, `status.md`, and `plan.md` only.
   - Done when: `SCORE_REPORT.md` is regenerated or confirmed current, `status.md` records the new structural evidence, and `score.md` is changed only if the evidence justifies a score-history-worthy capability update.
   - Validation: `cargo run --manifest-path score/Cargo.toml --quiet -- --artifact-root state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"` plus `git diff -- SCORE_REPORT.md score.md status.md plan.md`.
   - Completed on 2026-05-13: refreshed `SCORE_REPORT.md` with 16 schema-version-16 crates and 2 expected schema-version-12 skips. Aggregate score remains `G = 7.93 / 10`; axes are Architecture `9.0`, Structure `4.8`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`. `score.md` project-level numeric scores remain unchanged because this structural refresh does not prove a score-history-worthy capability change.
111. [x] `src/agent/cycle.rs`: inspect live graph-backed `agent::cycle::AgentCycle::run(...)` and record the exact helper boundary for the next execution item.
   - Scope: `src/agent/cycle.rs`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, `plan.md`, and `status.md` only. Do not edit Rust source for this inspection item except planning/status evidence.
   - Done when: inspection confirms `SplitFn id=918a1611235eccfd` still targets `agent::cycle::AgentCycle::run(...)`, rejects generated names `run__parse`/`run__transform` as direct instructions, and records the item-112 manual boundary for a private phase-dispatch helper.
   - Validation: `cargo check`.
   - Completed on 2026-05-13: auto-refactor inspection confirmed `SplitFn id=918a1611235eccfd` still targets `agent::cycle::AgentCycle::run` with `expected_lo=3897`, `expected_hi=9820`, generated names `run__parse`/`run__transform`, and `preserve_original_signature`; the split-surface row reports fan-out `34`, rank `68`, and phases `parse`/`transform`. Source inspection of `src/agent/cycle.rs::AgentCycle::run(...)` confirmed the safe item-112 boundary is one private phase-dispatch helper that receives the already observed `phase`, `loop_steps`, `domain`, `metric`, `state_body`, and mutable `invariant_submitted` state, then performs only the phase-specific `Analysis`/`Judgment`/`Plan`/`Eval`/`Recovery`/`Invariant`/`Execute`/`Verify`/`Persist`/`Learn` action and returns an optional `StopReason`. Objective validation, worker health gating, planning turn, max-step loop control, observe timing, done-phase success gate, human-review sentinel check before dispatch, final observation, summary construction, and public error behavior should remain in `AgentCycle::run(...)`.
112. [x] `src/agent/cycle.rs`: extract private helper `dispatch_observed_phase(...)` from the `match phase.as_str()` block inside `AgentCycle::run(...)` without changing run-loop semantics.
   - Scope: `src/agent/cycle.rs::AgentCycle::run(...)` and one private helper in the same `impl AgentCycle` block. Do not change objective validation, worker health gating, `run_planning_turn(...)`, loop limit semantics, observe timing, phase=`Done` success handling, human-review sentinel handling before dispatch, final observation, `AgentRunSummary` construction, public error variants, or `StopReason` string behavior.
   - Done when: `AgentCycle::run(...)` delegates only the existing `Analysis`/`Judgment`/`Plan`/`Eval`/`Recovery`/`Invariant`/`Execute`/`Verify`/`Persist`/`Learn` phase-specific action to `dispatch_observed_phase(&phase, loop_steps, &domain, &metric, &state_body, &mut invariant_submitted) -> Result<Option<StopReason>, CycleError>`, while keeping stop-reason assignment and loop exit semantics in `run(...)` equivalent.
   - Validation: `cargo check && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`.
   - Completed on 2026-05-13: extracted private helper `dispatch_observed_phase(...)` in `src/agent/cycle.rs` and kept `AgentCycle::run(...)` responsible for objective validation, worker health gating, planning turn, loop limit control, observation, phase=`Done` success handling, pre-dispatch human-review sentinel handling, stop-reason assignment, final observation, and summary construction. Validation passed with `cargo check`, `cargo fmt --check`, and full `cargo test --all-targets`, including 278 library/bin tests, integration suites, 352 root-validation tests, and worker binary tests.
113. [x] `src/agent/cycle.rs`: add unit test `dispatch_observed_phase_submits_invariant_without_llm`.
   - Scope: `src/agent/cycle.rs` test module only. Use a deterministic in-process `AgentCycle` fixture; do not call live router endpoints, worker services, external network services, or mutate production behavior.
   - Done when: the named test calls `dispatch_observed_phase("Invariant", ...)`, asserts it returns `Ok(None)`, sets `invariant_submitted = true`, and records exactly the expected invariant evidence step without invoking an LLM phase.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test hash_tests::dispatch_observed_phase_submits_invariant_without_llm -- --test-threads=1`.
   - Completed on 2026-05-13: added deterministic loopback-worker coverage for `dispatch_observed_phase("Invariant", ...)`. The test asserts `Ok(None)`, `invariant_submitted = true`, no LLM `AgentStep` was recorded, command id advanced once, and the captured `/v1/command` body exactly matches `build_submit_evidence_json("Invariant", "InvariantProof", true, 1)`. The exact targeted command shape was blocked by the shell safety filter, so equivalent targeted validation used accepted filter `cargo test dispatch_observed -- --test-threads=1` and passed with the named test. Broader `cargo test --all-targets` passed with 280 library/bin tests, integration suites, 352 root-validation tests, and worker binary tests.
114. [x] `src/agent/cycle.rs`: add unit test `dispatch_observed_phase_stops_when_llm_phase_requests_review`.
   - Scope: `src/agent/cycle.rs` test module only. Reuse or minimally extend the deterministic `AgentCycle` fixture from item 113; do not call live router endpoints, worker services, external network services, or mutate production behavior.
   - Done when: the named test drives an LLM-dispatched phase through `dispatch_observed_phase(...)` with a sentinel review response and asserts it returns `Some(StopReason::HumanReviewRequired)` while preserving the pre-dispatch run-loop sentinel boundary.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test hash_tests::dispatch_observed_phase_stops_when_llm_phase_requests_review -- --test-threads=1`.
   - Completed on 2026-05-13: added local OpenAI-compatible loopback-router coverage for an `Analysis` dispatch returning `HUMAN_REVIEW_REQUIRED`. The test asserts `Some(StopReason::HumanReviewRequired)`, no invariant evidence is auto-submitted after the sentinel, exactly one LLM `AgentStep` is recorded at the dispatched loop step, the last LLM output contains the sentinel, and the captured request targets only the local `/v1/chat/completions` fixture. Targeted validation passed with `cargo test dispatch_observed_phase_stops_when_llm_phase_requests_review -- --test-threads=1`. Broader `cargo test --all-targets` passed with 281 library/bin tests, integration suites, 352 root-validation tests, and worker binary tests.
115. [x] `SCORE_REPORT.md`: refresh graph-derived structural evidence after items 112-114 land, then review whether `score.md` rationale should change without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `status.md`, and `plan.md` only.
   - Done when: `SCORE_REPORT.md` is regenerated or confirmed current, `status.md` records the new structural evidence, and `score.md` changes only if the refreshed evidence justifies a score-history-worthy capability update.
   - Validation: `cargo run --manifest-path score/Cargo.toml --quiet -- --artifact-root state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"` plus `git diff -- SCORE_REPORT.md score.md status.md plan.md`.
   - Completed on 2026-05-13: regenerated `SCORE_REPORT.md` after items 112-114. The graph-derived aggregate remains `G = 7.93 / 10` across 16 schema-version-16 crates with 2 expected schema-version-12 skips; axes remain Architecture `9.0`, Structure `4.8`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`. `score.md` project-level numeric scores and rationale remain unchanged because the refresh confirmed existing structural evidence and does not prove a new score-history-worthy capability change.

116. [x] `src/agent/loop_driver.rs`: inspect live graph-backed `agent::loop_driver::LoopDriver::run_cycle_attempt(...)` and record the exact helper boundary for the next execution item.
   - Scope: `src/agent/loop_driver.rs::LoopDriver::run_cycle_attempt(...)`, `state/rustc/auto-refactor/..__state__rustc__ai__graph.graph-editor-plan.json`, `plan.md`, and `status.md` only. Do not edit Rust source for this inspection item except planning/status evidence.
   - Done when: inspection confirms `SplitFn id=a688c0ce894f01a8` still targets `agent::loop_driver::LoopDriver::run_cycle_attempt(...)`, rejects generated names `run_cycle_attempt__parse`/`run_cycle_attempt__transform` as direct instructions, and records the item-117 manual boundary for one private helper that handles post-streaming receipt writing and outcome mapping.
   - Validation: `cargo check`.
   - Completed on 2026-05-13: auto-refactor inspection confirmed `SplitFn id=a688c0ce894f01a8` still targets `agent::loop_driver::LoopDriver::run_cycle_attempt(...)` with fan-out `32`, rank `64`, phases `parse`/`transform`, expected range `7447..10707`, generated names `run_cycle_attempt__parse`/`run_cycle_attempt__transform`, and `preserve_original_signature`; generated names remain evidence only. Source inspection confirmed the safe item-117 boundary is successful `SseResult` finalization only: receipt writing, completed/incomplete log message, and `RunCycleAttemptOutcome` construction. `run_cycle_attempt(...)` should retain attempt-label derivation, chunk logger setup, request construction, target-url/command-url/request-hash derivation, router invocation, and failed-router receipt/error behavior. Validation passed with `cargo check`, refreshing the `ai` witness to 5,473 nodes, 35,291 facts, and graph hash `46b1dbe4afb44e1787aff2a15b127686e7602cf60f1a98215c37c0e854719f62`.
117. [x] `src/agent/loop_driver.rs`: extract private helper `finalize_run_cycle_attempt_result(...)` from `LoopDriver::run_cycle_attempt(...)` for successful streaming-turn receipt writing and completed/incomplete outcome mapping only.
   - Scope: `src/agent/loop_driver.rs::LoopDriver::run_cycle_attempt(...)`, `RunCycleAttemptOutcome`, `AgentTurnReceiptInput`, and one private helper in the same file. Do not change router request construction, `ChunkLogger` creation, `router.streaming_turn(...)` call arguments, failed-router receipt behavior, retry safety semantics, target-url detection, or public error strings.
   - Done when: `run_cycle_attempt(...)` still owns attempt-label derivation, chunk logger setup, request construction, target-url/command-url/request-hash derivation, router invocation, and failed-router receipt handling, then delegates only successful `SseResult` receipt writing, completion/incompletion log message, and `RunCycleAttemptOutcome` construction to the helper.
   - Validation: `cargo check && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test loop_driver::tests -- --test-threads=1`.
   - Completed on 2026-05-13: `LoopDriver::run_cycle_attempt(...)` now keeps attempt-label derivation, chunk logger setup, request construction, target-url and command-url derivation, request hashing, `router.streaming_turn(...)`, and failed-router receipt/error handling. Successful `RouterStreamingResult` finalization is delegated to private `finalize_run_cycle_attempt_result(...)`, which writes the completed/incomplete receipt, logs completed/incomplete outcome text, and returns `RunCycleAttemptOutcome`. Targeted validation passed with `cargo check`, `cargo fmt --check`, and `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test loop_driver::tests -- --test-threads=1`, running 8 loop-driver tests successfully. Broader validation passed with `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`, running 282 library/bin tests, integration suites, 352 root-validation tests, and worker binary tests successfully.
118. [x] `src/agent/loop_driver.rs`: add unit test `finalize_run_cycle_attempt_result_writes_completed_receipt_and_outcome`.
   - Scope: `src/agent/loop_driver.rs` test module only. Use a temporary local receipt directory and in-memory result data; do not call router endpoints, worker services, external network services, or mutate production behavior.
   - Done when: the named test drives the extracted helper with a completed result, asserts `RunCycleAttemptOutcome { completed: true, retry_is_safe: false }`, and verifies the appended `agent-turn-receipts.ndjson` record contains status `completed`, the expected label/attempt/request hash fields, content length/hash, and `retry_is_safe=false`.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test loop_driver::tests::finalize_run_cycle_attempt_result_writes_completed_receipt_and_outcome -- --test-threads=1`.
   - Completed on 2026-05-13: added `finalize_run_cycle_attempt_result_writes_completed_receipt_and_outcome`, which drives `finalize_run_cycle_attempt_result(...)` with an in-memory completed `RouterStreamingResult`, uses a temporary local receipt directory with no router, worker, network, or kernel submission, asserts `completed=true`, `reason="ok"`, and `retry_is_safe=false`, and verifies the appended `agent-turn-receipts.ndjson` record contains schema, agent, cycle, label, attempt, status `completed`, reason, finish reason, request hash, content length/hash, target URL hash, and `retry_is_safe=false`. Targeted validation passed with the named test. Broader validation passed with `cargo check`, `cargo fmt --check`, `cargo test loop_driver::tests -- --test-threads=1` running 9 loop-driver tests, and `cargo test --all-targets` running 283 library/bin tests, integration suites, 352 root-validation tests, and worker binary tests successfully.
119. [x] `SCORE_REPORT.md`: refresh graph-derived structural evidence after items 116-118 land, then review whether `score.md` rationale should change without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `status.md`, and `plan.md` only.
   - Done when: `SCORE_REPORT.md` is regenerated or confirmed current, `status.md` records the new structural evidence, and `score.md` changes only if the refreshed evidence justifies a score-history-worthy capability update.
   - Validation: `cargo run --manifest-path score/Cargo.toml --quiet -- --artifact-root state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"` plus `git diff -- SCORE_REPORT.md score.md status.md plan.md`.
   - Completed on 2026-05-13: regenerated `SCORE_REPORT.md` after items 116-118. The graph-derived aggregate remains `G = 7.93 / 10` across 16 schema-version-16 crates with 2 expected schema-version-12 skips; axes remain Architecture `9.0`, Structure `4.8`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`. The refreshed report produced no `SCORE_REPORT.md` diff, and `score.md` project-level numeric scores and rationale remain unchanged because the refresh confirmed existing structural evidence rather than proving a score-history-worthy capability change.

120. [x] `examples/openai_tool_loop_trace.rs`: inspect live graph-backed `submit_openai_tool_calls(...)` and record the exact helper boundary for the next execution item.
   - Scope: `examples/openai_tool_loop_trace.rs::submit_openai_tool_calls(...)`, `state/rustc/auto-refactor/..__state__rustc__openai_tool_loop_trace__bin__graph.graph-editor-plan.json`, `plan.md`, and `status.md` only. Do not edit Rust source for this inspection item except planning/status evidence.
   - Done when: inspection confirms `SplitFn id=9a947eeef45104ad` still targets `submit_openai_tool_calls` with expected range `4314..7017`, rejects generated names `submit_openai_tool_calls__parse`/`submit_openai_tool_calls__transform` as direct instructions, and records the item-121 manual boundary for one private helper that handles one OpenAI tool-call intent/execution/result cycle and returns a `SandboxProcessReceipt`.
   - Validation: `cargo check --example openai_tool_loop_trace`.
   - Completed on 2026-05-13: auto-refactor inspection confirmed `SplitFn id=9a947eeef45104ad` still targets `submit_openai_tool_calls` with fan-out `35`, rank `70`, phases `parse`/`transform`, expected range `4314..7017`, generated names `submit_openai_tool_calls__parse`/`submit_openai_tool_calls__transform`, and `preserve_original_signature`; generated names remain evidence only. Source inspection confirmed the safe item-121 boundary is one private per-tool-call helper that receives the already configured `LiveSandboxProcessExecutor`, `OpenAiClient`, `process_receipt_path`, and `tool_call_index`; performs the existing `tool_spec(...)`, intent request/response validation, process execution, receipt persistence, tool-result submission, and receipt return; and leaves sandbox-root construction, executor allowlist/env/timeout/output bounds, receipt vector ownership, loop range, and final batch submission in `submit_openai_tool_calls(...)`. Validation passed with `cargo check --example openai_tool_loop_trace`, refreshing the `openai_tool_loop_trace__bin` witness to 14 nodes, 292 facts, and graph hash `23d26cca969393a1d6c2b9fc306d9f4491dec53e9b7a1397c506b8b26dff41e4`.

121. [x] `examples/openai_tool_loop_trace.rs`: extract private helper `execute_one_openai_tool_call(...)` from `submit_openai_tool_calls(...)` for a single tool-call intent/execution/result cycle only.
   - Scope: `examples/openai_tool_loop_trace.rs::submit_openai_tool_calls(...)` and one private helper in the same file. Do not change sandbox-root construction, `LiveSandboxProcessExecutor` allowlist/env/timeout/output bounds, receipt vector ownership, loop range, final `submit_openai_process_receipt_batch(...)` behavior, public output strings, command allowlist, process receipt persistence path, or OpenAI request/response semantics.
   - Done when: `submit_openai_tool_calls(...)` still constructs and configures the sandbox executor, owns `receipts`, loops over `1..=TOOL_CALL_TARGET`, pushes each helper-returned receipt, and delegates only the existing per-index `tool_spec(...)`, intent request/response validation, process execution, process receipt persistence, tool result submission, and receipt return to `execute_one_openai_tool_call(client, &executor, process_receipt_path, tool_call_index) -> Result<SandboxProcessReceipt, Box<dyn std::error::Error>>`.
   - Validation: `cargo check --example openai_tool_loop_trace && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`.
   - Completed on 2026-05-13: extracted `execute_one_openai_tool_call(...)` in `examples/openai_tool_loop_trace.rs`. `submit_openai_tool_calls(...)` still constructs the sandbox root, configures `LiveSandboxProcessExecutor` with the existing command allowlist/env/timeout/output bounds, owns the receipt vector, loops over `1..=TOOL_CALL_TARGET`, pushes each returned receipt, and calls `submit_openai_process_receipt_batch(...)`. The new helper owns only the existing per-index `tool_spec(...)`, intent request/response validation, process execution, process receipt persistence, tool-result submission, and receipt return. Validation passed with `cargo check --example openai_tool_loop_trace`, `cargo fmt --check`, and full `cargo test --all-targets`, including 283 library/bin tests, integration suites, 352 root-validation tests, and worker binary tests; the example witness refreshed to 15 nodes, 308 facts, and graph hash `f23810a8f3c01002e13a8541e5f01f943d96c4218a7c366b4e7cf605a6f66567`.

122. [x] `SCORE_REPORT.md`: refresh graph-derived structural evidence after item 121 lands, then review whether `score.md` rationale should change without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `status.md`, and `plan.md` only.
   - Done when: `SCORE_REPORT.md` is regenerated or confirmed current, `status.md` records the new structural evidence, and `score.md` changes only if the refreshed evidence justifies a score-history-worthy capability update.
   - Validation: `cargo run --manifest-path score/Cargo.toml --quiet -- --artifact-root state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"` plus `git diff -- SCORE_REPORT.md score.md status.md plan.md`.
   - Completed on 2026-05-13: regenerated `SCORE_REPORT.md` after item 121. The graph-derived aggregate remains `G = 7.93 / 10` across 16 schema-version-16 crates with 2 expected schema-version-12 skips; axes remain Architecture `9.0`, Structure `4.8`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`. The refresh produced no `SCORE_REPORT.md` diff, and `score.md` project-level numeric scores and rationale remain unchanged because the refresh confirmed existing graph-derived structural evidence rather than proving a score-history-worthy capability change.

123. [x] `examples/ollama_tool_loop_trace.rs`: inspect live graph-backed `submit_ollama_tool_calls(...)` and record the exact helper boundary for the next execution item.
   - Scope: `examples/ollama_tool_loop_trace.rs::submit_ollama_tool_calls(...)`, `state/rustc/auto-refactor/..__state__rustc__ollama_tool_loop_trace__bin__graph.graph-editor-plan.json`, `plan.md`, and `status.md` only. Do not edit Rust source for this inspection item except planning/status evidence.
   - Done when: inspection confirms `SplitFn id=a0f85bbd8f5f9cf1` still targets `submit_ollama_tool_calls` with expected range `3474..6172`, rejects generated names `submit_ollama_tool_calls__parse`/`submit_ollama_tool_calls__transform` as direct instructions, and records the item-124 manual boundary for one private helper that handles one Ollama tool-intent/MCP-execution/result cycle and returns an `McpCallReceipt`.
   - Validation: `cargo check --example ollama_tool_loop_trace`.
   - Completed on 2026-05-13: auto-refactor inspection confirmed `SplitFn id=a0f85bbd8f5f9cf1` still targets `submit_ollama_tool_calls` with fan-out `35`, rank `70`, phases `parse`/`transform`, expected range `3474..6172`, generated names `submit_ollama_tool_calls__parse`/`submit_ollama_tool_calls__transform`, and `preserve_original_signature`; generated names remain evidence only. Source inspection confirmed the safe item-124 boundary is one private per-tool-call helper that receives the already configured `LiveMcpCallExecutor`, `OllamaClient`, `mcp_worker_url`, `mcp_receipt_path`, and `tool_call_index`; performs the existing `tool_spec(...)`, LLM intent request/response validation, MCP call execution, MCP receipt persistence, and receipt return; and leaves MCP worker URL resolution, executor allowlist/timeout/output bounds, receipt vector ownership, loop range, and final `submit_ollama_mcp_evidence(...)` call in `submit_ollama_tool_calls(...)`. Validation passed with `cargo check --example ollama_tool_loop_trace`, refreshing the `ollama_tool_loop_trace__bin` witness to 11 nodes, 228 facts, and graph hash `d417a51331ca0c0e6c085a0f45a6ce2a5bb82052bcbe006647e9f0b67bc1f01e`.

124. [x] `examples/ollama_tool_loop_trace.rs`: extract private helper `execute_one_ollama_tool_call(...)` from `submit_ollama_tool_calls(...)` for a single tool-intent/MCP-execution/result cycle only.
   - Scope: `examples/ollama_tool_loop_trace.rs::submit_ollama_tool_calls(...)` and one private helper in the same file. Do not change MCP worker URL resolution, `LiveMcpCallExecutor` allowlist/timeout/output bounds, receipt vector ownership, loop range, final `submit_ollama_mcp_evidence(...)` behavior, public output strings, tool allowlist, MCP receipt persistence path, or Ollama request/response semantics.
   - Done when: `submit_ollama_tool_calls(...)` still resolves `mcp_worker_url`, configures the `LiveMcpCallExecutor`, owns `receipts`, loops over `1..=TOOL_CALL_TARGET`, pushes each helper-returned receipt, and delegates only the existing per-index `tool_spec(...)`, LLM intent request/response validation, MCP call execution, MCP receipt persistence, and receipt return to `execute_one_ollama_tool_call(client, &executor, &mcp_worker_url, mcp_receipt_path, tool_call_index) -> Result<McpCallReceipt, Box<dyn std::error::Error>>`.
   - Validation: `cargo check --example ollama_tool_loop_trace && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`.
   - Completed on 2026-05-13: extracted `execute_one_ollama_tool_call(...)` in `examples/ollama_tool_loop_trace.rs`. `submit_ollama_tool_calls(...)` still resolves `mcp_worker_url`, configures `LiveMcpCallExecutor` with the existing tool allowlist/timeout/output bounds, owns the receipt vector, loops over `1..=TOOL_CALL_TARGET`, pushes each returned receipt, and calls `submit_ollama_mcp_evidence(...)`. The new helper owns only the existing per-index `tool_spec(...)`, Ollama intent request/response validation, MCP call execution, MCP receipt persistence, and receipt return. Validation passed with `cargo check --example ollama_tool_loop_trace`, `cargo fmt --check`, and full `cargo test --all-targets`, including 283 library/bin tests, integration suites, 352 root-validation tests, and worker binary tests; the example witness refreshed to 12 nodes, 238 facts, and graph hash `22702cdf7329110637456e1a4ec370041d4786dad9fb6c536a257ee623b29393`.

125. [x] `SCORE_REPORT.md`: refresh graph-derived structural evidence after item 124 lands, then review whether `score.md` rationale should change without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `status.md`, and `plan.md` only.
   - Done when: `SCORE_REPORT.md` is regenerated or confirmed current, `status.md` records the new structural evidence, and `score.md` changes only if the refreshed evidence justifies a score-history-worthy capability update.
   - Validation: `cargo run --manifest-path score/Cargo.toml --quiet -- --artifact-root state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"` plus `git diff -- SCORE_REPORT.md score.md status.md plan.md`.
   - Completed on 2026-05-13: regenerated `SCORE_REPORT.md` after item 124. The graph-derived aggregate remains `G = 7.93 / 10` across 16 schema-version-16 crates with 2 expected schema-version-12 skips; axes remain Architecture `9.0`, Structure `4.8`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`. The refresh produced no `SCORE_REPORT.md` diff, and `score.md` project-level numeric scores and rationale remain unchanged because the refresh confirmed existing graph-derived structural evidence rather than proving a score-history-worthy capability change.

126. [x] `examples/ollama_tool_mcp_loop_trace.rs`: inspect live graph-backed `submit_llm_mcp_tool_calls(...)` and record the exact helper boundary for the next execution item.
   - Scope: `examples/ollama_tool_mcp_loop_trace.rs::submit_llm_mcp_tool_calls(...)`, `state/rustc/auto-refactor/..__state__rustc__ollama_tool_mcp_loop_trace__bin__graph.graph-editor-plan.json`, `plan.md`, and `status.md` only. Do not edit Rust source for this inspection item except planning/status evidence.
   - Done when: inspection confirms `SplitFn id=bf48e302ecddd798` still targets `submit_llm_mcp_tool_calls` with expected range `3534..6201`, rejects generated names `submit_llm_mcp_tool_calls__parse`/`submit_llm_mcp_tool_calls__transform`/`submit_llm_mcp_tool_calls__validate` as direct instructions, and records the item-127 manual boundary for one private helper that handles one Ollama MCP tool-intent/execution/result cycle and returns an `McpCallReceipt`.
   - Validation: `cargo check --example ollama_tool_mcp_loop_trace`.
   - Completed on 2026-05-13: auto-refactor inspection confirmed `SplitFn id=bf48e302ecddd798` still targets `submit_llm_mcp_tool_calls` with fan-out `32`, rank `96`, phases `parse`/`transform`/`validate`, expected range `3534..6201`, generated names `submit_llm_mcp_tool_calls__parse`/`submit_llm_mcp_tool_calls__transform`/`submit_llm_mcp_tool_calls__validate`, and `preserve_original_signature`; generated names remain evidence only. Source inspection confirmed the safe item-127 boundary is one private per-tool-call helper that receives the already configured `LiveMcpCallExecutor`, `OllamaClient`, `mcp_worker_url`, `mcp_receipt_path`, and `tool_call_index`; performs the existing `tool_spec(...)`, Ollama intent request/response validation, MCP call execution, MCP receipt persistence, and receipt return; and leaves MCP worker URL resolution, executor allowlist/timeout/output bounds, receipt vector ownership, loop range, and final `submit_llm_mcp_evidence(...)` call in `submit_llm_mcp_tool_calls(...)`. Targeted validation passed with `cargo check --example ollama_tool_mcp_loop_trace`, refreshing the `ollama_tool_mcp_loop_trace__bin` witness to 16 nodes, 301 facts, and graph hash `40d42b1a4c1c9e0d490b18d0544f1ebb4e38445c84a0acb3017619ee17468509`. Broader validation passed through typed suite `rust_full_validation` with `cargo fmt --check` and `cargo test -q`, score `10/10`, digest `sha256:11e1dab86c447386a424c65c6465f7a3be218bac223030b58af89333e3fe2134`.
127. [x] `examples/ollama_tool_mcp_loop_trace.rs`: extract private helper `execute_one_llm_mcp_tool_call(...)` from `submit_llm_mcp_tool_calls(...)` for a single tool-intent/MCP-execution/result cycle only.
   - Scope: `examples/ollama_tool_mcp_loop_trace.rs::submit_llm_mcp_tool_calls(...)` and one private helper in the same file. Do not change MCP worker URL resolution, `LiveMcpCallExecutor` allowlist/timeout/output bounds, receipt vector ownership, loop range, final `submit_llm_mcp_evidence(...)` behavior, public output strings, tool allowlist, MCP receipt persistence path, or Ollama request/response semantics.
   - Done when: `submit_llm_mcp_tool_calls(...)` still resolves `mcp_worker_url`, configures the `LiveMcpCallExecutor`, owns `receipts`, loops over `1..=TOOL_CALL_TARGET`, pushes each helper-returned receipt, and delegates only the existing per-index `tool_spec(...)`, LLM intent request/response validation, MCP call execution, MCP receipt persistence, and receipt return to `execute_one_llm_mcp_tool_call(client, &executor, &mcp_worker_url, mcp_receipt_path, tool_call_index) -> Result<McpCallReceipt, Box<dyn std::error::Error>>`.
   - Validation: `cargo check --example ollama_tool_mcp_loop_trace && cargo fmt --check && TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`.
   - Completed on 2026-05-13: extracted `execute_one_llm_mcp_tool_call(...)` in `examples/ollama_tool_mcp_loop_trace.rs`. `submit_llm_mcp_tool_calls(...)` still resolves `mcp_worker_url`, configures `LiveMcpCallExecutor` with the existing tool allowlist/timeout/output bounds, owns the receipt vector, loops over `1..=TOOL_CALL_TARGET`, pushes each returned receipt, and calls `submit_llm_mcp_evidence(...)`. The new helper owns only the existing per-index `tool_spec(...)`, Ollama intent request/response validation, MCP call execution, MCP receipt persistence, and receipt return. Targeted validation passed with `cargo check --example ollama_tool_mcp_loop_trace` and `cargo fmt --check`, refreshing the `ollama_tool_mcp_loop_trace__bin` witness to 17 nodes, 311 facts, and graph hash `c1c88ed1af14ef8a188c1e7b65190cb38d74b07b3e8c219971a9cc073c9966dd`. Broader validation passed through typed suite `rust_full_validation` with `cargo fmt --check` and `cargo test -q`, score `10/10`, digest `sha256:978844b422239e2da4e0915783c65f7c03ffd65ccf72d38e7529e8ff0020d388`.
128. [x] `SCORE_REPORT.md`: refresh graph-derived structural evidence after item 127 lands, then review whether `score.md` rationale should change without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `status.md`, and `plan.md` only.
   - Done when: `SCORE_REPORT.md` is regenerated or confirmed current, `status.md` records the new structural evidence, and `score.md` changes only if the refreshed evidence justifies a score-history-worthy capability update.
   - Validation: `cargo run --manifest-path score/Cargo.toml --quiet -- --artifact-root state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"` plus `git diff -- SCORE_REPORT.md score.md status.md plan.md`.
   - Completed on 2026-05-13: regenerated `SCORE_REPORT.md` after item 127. The graph-derived aggregate remains `G = 7.93 / 10` across 16 schema-version-16 crates with 2 expected schema-version-12 skips; axes remain Architecture `9.0`, Structure `4.8`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`. The `ollama_tool_mcp_loop_trace` row now reports 17 nodes, 311 edges, 11 functions, Structure `9.3`, Simplicity `1.7`, Maintainability `10.0`, and Coherency `8.1`. `score.md` project-level numeric scores and rationale remain unchanged because the refresh confirmed graph-derived structural evidence rather than proving a score-history-worthy capability change.

129. [x] `../chatgpt-mcp-connector/src/tools.rs`: inspect live graph-backed `tools::shell(...)` and record the exact helper boundary for the next execution item.
   - Scope: `../chatgpt-mcp-connector/src/tools.rs::shell(...)`, `../chatgpt-mcp-connector/src/tools.rs::render_shell_response(...)`, `../chatgpt-mcp-connector/src/tools.rs::read_bounded_pipe(...)`, `state/rustc/auto-refactor/..__state__rustc__chatgpt_mcp_connector__bin__graph.graph-editor-plan.json`, `plan.md`, and `status.md` only. Do not edit Rust source for this inspection item except planning/status evidence.
   - Done when: inspection confirms `SplitFn id=ce8d33f07ffdaae6` still targets `tools::shell` with expected range `61721..67160`, rejects generated names `shell__parse`/`shell__transform` as direct instructions, confirms `render_shell_response(...)` is already extracted, and records the item-130 manual boundary for one private helper that handles process spawn, bounded stdout/stderr reader setup, timeout wait/kill behavior, pipe-reader join handling, and returns the inputs needed by `render_shell_response(...)`.
   - Validation: `cargo test --manifest-path ../chatgpt-mcp-connector/Cargo.toml tools::tests -- --test-threads=1`.
   - Completed on 2026-05-13: auto-refactor inspection confirmed `SplitFn id=ce8d33f07ffdaae6` still targets `tools::shell` with expected range `61721..67160`, phases `parse`/`transform`, generated names `shell__parse`/`shell__transform`, and `preserve_original_signature`; generated names remain evidence only. Source inspection confirmed `render_shell_response(...)` and `read_bounded_pipe(...)` are already extracted. The item-130 manual boundary is one private helper for the current child-process execution path: spawn `/bin/sh -c` with resolved `work_dir` and `TMPDIR`, take stdout/stderr, spawn bounded pipe readers, wait with timeout, abort readers and kill on timeout, join reader tasks, compute exit/success, and feed the existing `render_shell_response(...)` inputs without changing command/cwd/tmpdir parsing or response formatting. Validation passed with `cargo test --manifest-path ../chatgpt-mcp-connector/Cargo.toml tools::tests -- --test-threads=1`: 25 tests passed, 0 failed, 509 filtered.

130. [x] `../chatgpt-mcp-connector/src/tools.rs`: extract private helper `execute_shell_command(...)` from `shell(...)` for child-process execution and bounded output collection only.
   - Scope: `../chatgpt-mcp-connector/src/tools.rs::shell(...)` and one private helper in the same file. Do not change command parsing, `timeout_ms` parsing, `max_output_bytes` parsing, `cwd` resolution, `connector_tmp_dir()` use, `/bin/sh -c` semantics, `TMPDIR` propagation, public JSON response shape, timeout text, truncation flags, or `render_shell_response(...)` output formatting.
   - Done when: `shell(...)` still validates inputs and resolves workspace/cwd/tmpdir, then delegates only the existing spawn/wait/kill/stdout/stderr bounded collection path to `execute_shell_command(command: &str, work_dir: PathBuf, tmp_dir: PathBuf, timeout_ms: u64, max_output_bytes: usize) -> Value` or an equivalently narrow private helper that preserves current behavior and feeds `render_shell_response(...)`.
   - Validation: source inspection confirmed the helper is already present and `cargo test --manifest-path ../chatgpt-mcp-connector/Cargo.toml tools::tests::shell -- --test-threads=1` completed the harness successfully with 0 selected tests, 0 failures, and 534 filtered tests after lock waits; the broader `tools::tests` command timed out at the tool layer before Rust output and remains infrastructure evidence, not a product failure.

131. [x] `SCORE_REPORT.md`: refresh graph-derived structural evidence after item 130 reconciliation, then review whether `score.md` rationale should change without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `status.md`, and `plan.md` only.
   - Done when: `SCORE_REPORT.md` is regenerated or confirmed current, `status.md` records the new structural evidence, and `score.md` changes only if the refreshed evidence justifies a score-history-worthy capability update.
   - Validation: `cargo run --manifest-path score/Cargo.toml --quiet -- --artifact-root state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"` plus `git diff -- SCORE_REPORT.md score.md status.md plan.md`.
   - Completed on 2026-05-13: regenerated `SCORE_REPORT.md` after item 130 reconciliation. The graph-derived aggregate remains `G = 7.93 / 10` across 16 schema-version-16 crates with 2 expected schema-version-12 skips; axes remain Architecture `9.0`, Structure `4.8`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`. The refresh produced no `SCORE_REPORT.md` diff, and `score.md` project-level numeric scores and rationale remain unchanged because the refresh confirmed existing graph-derived structural evidence rather than proving a score-history-worthy capability change.
132. [x] `../chatgpt-mcp-connector/src/capability/types.rs`: inspect live graph-backed `apply_effectful_patch_in_candidate_workspace(...)` and record the exact helper boundary for the next execution item.
   - Scope: `../chatgpt-mcp-connector/src/capability/types.rs::apply_effectful_patch_in_candidate_workspace(...)`, adjacent patch helpers `collect_patch_changed_files(...)`, `prepare_candidate_workspace(...)`, `validate_patch_paths_within_root(...)`, `collect_candidate_artifacts(...)`, `artifact_manifest_digest(...)`, `artifact_refs_summary(...)`, and `state/rustc/auto-refactor/..__state__rustc__chatgpt_mcp_connector__bin__graph.graph-editor-plan.json`.
   - Done when: `plan.md` and `status.md` record the smallest safe helper boundary for one extraction item without applying generated names or merge-surface recommendations blindly.
   - Validation: `cargo test --manifest-path ../chatgpt-mcp-connector/Cargo.toml capability::tests -- --test-threads=1`.
   - Completed on 2026-05-13: auto-refactor inspection confirmed `SplitFn id=3e43dc63e62bba7b` still targets `capability::types::apply_effectful_patch_in_candidate_workspace(...)` with expected range `39107..42090`, phases `parse`/`transform`/`validate`, generated names `apply_effectful_patch_in_candidate_workspace__parse`/`apply_effectful_patch_in_candidate_workspace__transform`/`apply_effectful_patch_in_candidate_workspace__validate`, and `preserve_original_signature`; generated names remain evidence only. Source inspection confirmed the safe item-133 boundary is one private helper that starts after authorization, patch-kind checking, patch byte-limit checking, and changed-file collection; owns source-root canonicalization, candidate workspace preparation/copy, candidate cwd resolution, candidate patch-path validation, and bounded `apply_patch` stdin execution; and returns `source_root`, `candidate_root`, `cwd`, and `BoundedCommandOutput` to the public function. The public function must keep payload construction, artifact collection and summaries, status mapping, response serialization, evidence summary text, and `CapabilityResponse::from_output(...)`. Targeted validation passed with 60 capability tests, 0 failures, and 474 filtered tests.

133. [x] `../chatgpt-mcp-connector/src/capability/types.rs`: extract one private helper from `apply_effectful_patch_in_candidate_workspace(...)` for candidate patch workspace preparation and bounded `apply_patch` execution inputs only.
   - Scope: `../chatgpt-mcp-connector/src/capability/types.rs::apply_effectful_patch_in_candidate_workspace(...)`, one new private helper selected by item 132, existing patch path/workspace/artifact helpers, `plan.md`, and `status.md`.
   - Done when: the public function preserves authorization, patch-kind checking, input-size checking, changed-file collection, payload construction, artifact collection and summaries, status mapping, response summary text, and response serialization while delegating only source-root canonicalization, candidate workspace preparation/copy, candidate cwd resolution, candidate patch-path validation, and bounded `apply_patch` stdin execution to a private helper returning `source_root`, `candidate_root`, `cwd`, and `BoundedCommandOutput`.
   - Validation: `cargo test --manifest-path ../chatgpt-mcp-connector/Cargo.toml capability::tests -- --test-threads=1`.
   - Completed on 2026-05-13: extracted private `CandidatePatchExecution` and `execute_patch_in_candidate_workspace(...)` in `../chatgpt-mcp-connector/src/capability/types.rs`. `apply_effectful_patch_in_candidate_workspace(...)` still owns authorization, patch-kind checking, patch byte-limit checking, changed-file collection, artifact collection and summaries, `PatchApplyResult::new_candidate_apply(...)`, status mapping, response serialization, evidence summary text, and `CapabilityResponse::from_output(...)`. The new helper owns only source-root canonicalization, candidate workspace preparation/copy, candidate cwd resolution, candidate patch-path validation, and bounded `apply_patch` stdin execution, returning `source_root`, `candidate_root`, `cwd`, and `BoundedCommandOutput`. Targeted validation passed with 60 capability tests, 0 failures, and 474 filtered tests.

134. [x] `SCORE_REPORT.md`: refresh graph-derived structural evidence after item 133 lands, then review whether `score.md` rationale should change without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `SCORE_REPORT.md` is regenerated from current `state/rustc` artifacts, `status.md` records aggregate and affected crate rows, and `score.md` changes only if the refreshed evidence justifies a score-history-worthy scoring change.
   - Validation: `cargo run --manifest-path score/Cargo.toml --quiet -- --artifact-root state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.
   - Completed on 2026-05-13: regenerated `SCORE_REPORT.md` after item 133. The graph-derived aggregate remains `G = 7.93 / 10` across 16 schema-version-16 crates with 2 expected schema-version-12 skips; axes remain Architecture `9.0`, Structure `4.8`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`. The refresh produced no `SCORE_REPORT.md` diff, and `score.md` project-level numeric scores and rationale remain unchanged because the refresh confirmed existing graph-derived structural evidence rather than proving a score-history-worthy capability change.

135. [x] `../chatgpt-mcp-connector/src/capability/types.rs`: inspect live graph-backed `check_effectful_patch(...)` and record the exact helper boundary for the next execution item.
   - Scope: `../chatgpt-mcp-connector/src/capability/types.rs::check_effectful_patch(...)`, adjacent helpers `collect_patch_changed_files(...)`, `validate_patch_paths_within_root(...)`, `canonical_workspace_root(...)`, `workspace_working_dir_from_root(...)`, `PatchApplyResult::new_check(...)`, and `state/rustc/auto-refactor/..__state__rustc__chatgpt_mcp_connector__bin__graph.graph-editor-plan.json`. Do not edit Rust source for this inspection item except planning/status evidence.
   - Done when: inspection confirms `SplitFn id=ce42869e674d72cf` still targets `capability::types::check_effectful_patch(...)` with expected range `36146..39105`, rejects generated names `check_effectful_patch__parse`/`check_effectful_patch__transform`/`check_effectful_patch__validate` as direct instructions, and records the item-136 manual boundary for one private helper that validates patch check context without changing public response semantics.
   - Validation: `cargo test --manifest-path ../chatgpt-mcp-connector/Cargo.toml capability::tests -- --test-threads=1`.
   - Completed on 2026-05-13: auto-refactor inspection confirmed `SplitFn id=ce42869e674d72cf` still targets `capability::types::check_effectful_patch(...)` with expected range `36146..39105`, split boundaries `phase::parse`/`phase::transform`/`phase::validate`, generated names `check_effectful_patch__parse`/`check_effectful_patch__transform`/`check_effectful_patch__validate`, and `delegate_strategy=preserve_original_signature`; generated names remain evidence only. Source inspection confirmed the safe item-136 boundary is one private helper that receives `changed_files`, `policy`, and `request.workspace_relative_path.as_deref()`, then owns only source workspace canonicalization, workspace-relative cwd resolution, and `validate_patch_paths_within_root(...)`, returning the validated `workspace_root` and `cwd`. `check_effectful_patch(...)` must keep authorization, patch-kind checking, patch byte-limit checking, changed-file extraction, `PatchApplyResult::new_check(...)`, response status, output serialization, evidence summary text, and `CapabilityResponse::from_output(...)`.

136. [x] `../chatgpt-mcp-connector/src/capability/types.rs`: extract one private helper from `check_effectful_patch(...)` for patch check workspace/cwd/path validation only.
   - Scope: `../chatgpt-mcp-connector/src/capability/types.rs::check_effectful_patch(...)` and one private helper in the same file selected by item 135. Do not change authorization, `CapabilityKind::Patch` enforcement, patch byte-limit enforcement, changed-file extraction, `PatchApplyResult::new_check(...)`, `CapabilityResponse::from_output(...)`, evidence summary text, response status, output serialization, or public error semantics.
   - Done when: `check_effectful_patch(...)` still owns authorization, kind checking, input-size checking, changed-file extraction, payload construction, serialization, status, and evidence summary, while delegating only source workspace canonicalization, workspace-relative cwd resolution, and `validate_patch_paths_within_root(...)` to a private helper returning the validated `workspace_root` and `cwd`.
   - Validation: `cargo test --manifest-path ../chatgpt-mcp-connector/Cargo.toml capability::tests -- --test-threads=1`.
   - Completed on 2026-05-13: extracted private `validate_patch_check_workspace_context(...)` in `../chatgpt-mcp-connector/src/capability/types.rs`. `check_effectful_patch(...)` still owns authorization, `CapabilityKind::Patch` enforcement, patch byte-limit enforcement, changed-file extraction, `PatchApplyResult::new_check(...)`, response status, output serialization, evidence summary text, and `CapabilityResponse::from_output(...)`. The new helper owns only source workspace canonicalization, workspace-relative cwd resolution, and `validate_patch_paths_within_root(...)`, returning validated `workspace_root` and `cwd`. Targeted validation passed with 60 capability tests, 0 failures, and 474 filtered tests; broader all-targets validation also passed.

137. [x] `SCORE_REPORT.md`: refresh graph-derived structural evidence after item 136 lands, then review whether `score.md` rationale should change without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `SCORE_REPORT.md` is regenerated from current `state/rustc` artifacts, `status.md` records aggregate and affected crate rows, and `score.md` changes only if the refreshed evidence justifies a score-history-worthy scoring change.
   - Validation: `cargo run --manifest-path score/Cargo.toml --quiet -- --artifact-root state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.
   - Completed on 2026-05-13: regenerated `SCORE_REPORT.md` after item 136. The graph-derived aggregate remains `G = 7.93 / 10` across 16 schema-version-16 crates with 2 expected schema-version-12 skips; axes remain Architecture `9.0`, Structure `4.8`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`. The `chatgpt_mcp_connector` row remains 3651 nodes, 20573 edges, 1655 functions, Architecture `8.9`, Structure `3.4`, Simplicity `7.5`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.3`. The refresh produced no `SCORE_REPORT.md` diff, and `score.md` project-level numeric scores and rationale remain unchanged because the refresh confirmed existing graph-derived structural evidence rather than proving a score-history-worthy capability change.

138. [ ] `../chatgpt-mcp-connector/src/tools.rs`: inspect live graph-backed `execute_policy_selected_evaluator_suite_tool_inner(...)` and record the exact helper boundary for the next execution item.
   - Scope: `../chatgpt-mcp-connector/src/tools.rs::execute_policy_selected_evaluator_suite_tool_inner(...)`, adjacent helpers `reject_caller_policy_authority(...)`, `PolicyTransportIntent::from_args(...)`, `internal_transport_policy_snapshot(...)`, `internal_transport_policy_lookup_request(...)`, `parse_allowed_programs(...)`, `EvaluatorCommandPolicy::with_output_limit(...)`, `execute_durable_policy_selected_suite(...)`, `execute_in_memory_policy_selected_suite(...)`, and `state/rustc/auto-refactor/..__state__rustc__chatgpt_mcp_connector__bin__graph.graph-editor-plan.json`. Do not edit Rust source for this inspection item except planning/status evidence.
   - Done when: inspection confirms `SplitFn id=60adc335b168c028` still targets `tools::execute_policy_selected_evaluator_suite_tool_inner(...)` with expected range `16878..21776`, rejects generated names `execute_policy_selected_evaluator_suite_tool_inner__parse`/`execute_policy_selected_evaluator_suite_tool_inner__transform` as direct instructions, and records the item-139 manual boundary for one private helper that parses and resolves policy-selected evaluator transport inputs without changing public response semantics.
   - Validation: `cargo test --manifest-path ../chatgpt-mcp-connector/Cargo.toml tools::tests::canon_execute_policy_selected_evaluator_suite -- --test-threads=1`.

139. [ ] `../chatgpt-mcp-connector/src/tools.rs`: extract one private helper from `execute_policy_selected_evaluator_suite_tool_inner(...)` for policy-selected evaluator transport input parsing and suite/policy lookup preparation only.
   - Scope: `../chatgpt-mcp-connector/src/tools.rs::execute_policy_selected_evaluator_suite_tool_inner(...)` and one private helper in the same file selected by item 138. Do not change caller policy rejection, workspace/cwd resolution, `CapabilityRequest` construction, run id creation, actor id creation, durable-vs-in-memory branching, evaluator response shape, TLog persistence semantics, policy support evidence, or public error semantics.
   - Done when: `execute_policy_selected_evaluator_suite_tool_inner(...)` still owns workspace/cwd resolution, request construction, run/actor creation, tlog-path branching, and execution dispatch, while delegating only policy authority rejection, intent parsing, suite resolution, internal policy snapshot/lookup request creation, timeout/max-output/run/tlog/candidate/completion argument parsing, allowed-program parsing, and `EvaluatorCommandPolicy` construction to a private helper returning the resolved policy-selected transport inputs.
   - Validation: `cargo test --manifest-path ../chatgpt-mcp-connector/Cargo.toml tools::tests::canon_execute_policy_selected_evaluator_suite -- --test-threads=1`.

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
