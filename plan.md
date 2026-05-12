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

Current date: 2026-05-12.
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

Planning reconnaissance on 2026-05-12 confirms items 60 through 65 are complete with targeted and broader validation as recorded below. After item 65, the next selected graph-backed file-scoped improvement is `score/src/main.rs::write_report(...)`, from `state/rustc/auto-refactor/..__state__rustc__score__bin__graph.graph-editor-plan.json` `SplitFn id=1676b1a0fb97f747`. This target is smaller than router, graph mutation, process-executor, or connector shell candidates, and it is directly tied to the score/reporting pipeline while Structure remains the lowest graph-derived axis in `SCORE_REPORT.md`.

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

Graph evidence from `state/rustc/ai/graph.json` was refreshed on 2026-05-11: schema version 16, graph hash `2399ea73e0eccc81561d2f1f3aaedb1692d965e0a4dfcbe6c0e2c4cb3e67f664`, receipt hash `5fe7df2837b97ed8ebf73c0eb7de3284c57be55ab7d3d8803bf094708ad5bbde`, risk hash `6101aa0240348d6458bd941fd932e5dceb6a577a3e90e04df4790eda615a2a9a`, 5,337 nodes, 33,179 edges, 3,453 intents, and 752 compiled P5 domain-node matches. The top-level graph keys are `edges`, `intents`, `meta`, and `nodes`; the `nodes` object is keyed by canonical symbol name. Node-kind counts are `fn 3453`, `impl 1582`, `struct 206`, `enum 90`, `trait 1`, and `ty_alias 5`. This refreshed graph evidence proves the compiled P5 domain surface for the current planning scope.

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

67. [ ] `score/src/main.rs`: extract aggregate-score section rendering from `write_report(...)` into private helper `write_aggregate_scores_section(buf: &mut Vec<u8>, agg: &[f64; 6], g: f64) -> Result<()>` while preserving the `write_report(...)` signature and generated Markdown bytes.
   - Scope: `score/src/main.rs` only. Allowed edits are `write_report(...)`, the new private helper, and a focused unit test only if it does not require broad fixture changes; do not edit score formulas, graph discovery, CLI argument parsing, generated auto-refactor JSON, `SCORE_REPORT.md`, domain code, agent runtime code, or validation scripts while selecting this item.
   - Done when: `write_report(...)` delegates only the `## Aggregate Scores` Markdown block, fenced text block, per-axis aggregate rows, blank line, and geometric-mean row to `write_aggregate_scores_section(...)`; report heading, generated metadata line, per-crate table, axis definitions table, file parent directory creation, and file write behavior remain in `write_report(...)`.
   - Validation: `cargo fmt --check`; `cargo test --manifest-path score/Cargo.toml -- --test-threads=1`; `cargo run --manifest-path score/Cargo.toml --quiet -- --artifact-root state/rustc --report /tmp/canon-score-report-check.md --date 2026-05-12` and compare the generated aggregate-score block with `SCORE_REPORT.md`.


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
