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

Current date: 2026-05-14.
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

Planning reconnaissance refresh on 2026-05-14 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; inspected current `../state/rustc/auto-refactor/*.graph-editor-plan.json`; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json`; inspected candidate source and test surfaces in `src/capability/llm/openai.rs`; and checked the working tree. There was no first incomplete executable item before this planning turn: Active Priorities items 13 through 15 were complete and `status.md` confirmed checklist exhaustion. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan. Existing uncommitted non-planning changes in `GOAL.md` and `src/agent/loop_driver.rs`, plus pre-existing `status.md` execution-blocker ledger additions, are outside this planning scope and must not be overwritten by execution turns.

Current graph-derived aggregate evidence remains `G = 7.99 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.4`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan remains schema version 1, graph schema version 16, and contains 1,614 planned operations. Recently reconciled non-`root_validate` surfaces include `agent::config::{env_u32, env_u64}`, `agent::cycle::{recovery_gate, recovery_target_phase}`, `agent::objective::{with_risk_envelope, with_stop_condition}`, `capability::llm::ollama::OllamaConfig::{base_url_id, model_id}`, `capability::llm::ollama::OllamaMessage::{system, user}`, `capability::llm::openai::OpenAiFunctionTool::{new, with_description}`, `capability::llm::openai::OpenAiConfig::{base_url_id, model_id}`, and related graph-backed helper candidates. `agent::loop_driver` candidates remain rejected for this turn because `src/agent/loop_driver.rs` has unrelated uncommitted edits. Graph candidates that merge semantically distinct hash, route, bridge, score, receipt, proof, loader, encoder, or record-family functions remain rejected unless the execute turn preserves their public semantic APIs and consolidates only private construction mechanics.

Graph operation `fefb1237c6a1ab0a` identified `capability::llm::openai::{json_string_field, message_content_field}` and has now been reconciled by completed items 16 through 18. Current planning turn selected graph-backed prompt candidates `agent::prompt::{analysis_prompt, judgment_prompt, plan_prompt, eval_prompt, recovery_prompt}` as the next safe non-root consolidation area. The safe subset is private helper delegation only: preserve the five public phase-specific prompt functions, their phase labels, context labels, instruction text, final-verdict/no-tool rules, and all call sites while sharing only the repeated `CertificationPrompt` construction mechanics. Do not merge prompt identities, do not change `planning_prompt(...)`, and do not touch `src/agent/loop_driver.rs` because it has unrelated uncommitted edits.

Previously, graph operation `fefb1237c6a1ab0a` identified `capability::llm::openai::{json_string_field, message_content_field}` as a safe non-root consolidation area. The safe subset is private lookup-helper delegation only: keep the two named parser helpers private and semantically distinct, while sharing the common string-field lookup mechanics. Preserve parse scope: `message_content_field(...)` must continue to start from the `"message"` object and then select `"content"`, while `json_string_field(...)` must continue to search from the full body for the named field. Preserve `parse_chat_response_body(...)`, `json_string_at(...)`, `json_u32_field(...)`, JSON escaping/decoding behavior, HTTP/client code, request serialization, receipt/proof behavior, NDJSON behavior, and all network surfaces.

13. [x] `src/capability/llm/openai.rs`: route `OpenAiMessage::{system, user, assistant, assistant_tool_call, tool}` through one private constructor helper.
   - Scope: `src/capability/llm/openai.rs` only; allowed functions are `OpenAiMessage::system(...)`, `OpenAiMessage::user(...)`, `OpenAiMessage::assistant(...)`, `OpenAiMessage::assistant_tool_call(...)`, `OpenAiMessage::tool(...)`, and at most one new private helper near `OpenAiMessage`. Do not change `OpenAiConfig`, `OpenAiRetryBudgetPolicy`, `OpenAiFunctionTool`, `OpenAiTool`, `OpenAiFunctionCall`, `OpenAiToolCall`, `OpenAiChatRequest`, `message_contract_valid(...)`, `push_message_json(...)`, `messages_from_context(...)`, HTTP/client code, receipt/proof structs, NDJSON code, parser helpers, or tests in this item.
   - Done when: `system(...)` still produces role `system` with content and no tool metadata; `user(...)` still produces role `user` with content and no tool metadata; `assistant(...)` still produces role `assistant` with content and no tool metadata; `assistant_tool_call(...)` still produces role `assistant` with no content, no tool-call id, and exactly one tool call; `tool(...)` still produces role `tool` with content, tool-call id, and no tool calls; and all five public constructors delegate to the same private helper boundary.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

14. [x] `src/capability/llm/openai.rs` test `openai_message_constructors_preserve_role_boundaries`: add regression coverage for the shared message constructor helper.
   - Scope: `src/capability/llm/openai.rs` test module only; use existing `OpenAiMessage` and `OpenAiToolCall` public APIs plus existing private test access to `message_contract_valid(...)`. Do not change production code in this item.
   - Done when: the named test proves `system(...)`, `user(...)`, `assistant(...)`, `assistant_tool_call(...)`, and `tool(...)` preserve their exact role strings, content presence, tool-call-id presence, tool-call vector cardinality, and `message_contract_valid(...)` behavior. The test must not perform network I/O, filesystem I/O outside normal cargo test execution, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test openai_message_constructors_preserve_role_boundaries -- --test-threads=1`.

15. [x] `SCORE_REPORT.md`: after items 13 and 14 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.

16. [x] `src/capability/llm/openai.rs`: route `json_string_field(...)` and `message_content_field(...)` through one private scoped string-field lookup helper.
   - Scope: `src/capability/llm/openai.rs` only; allowed functions are `json_string_field(...)`, `message_content_field(...)`, and at most one new private helper adjacent to them. Do not change `parse_chat_response_body(...)`, `json_string_at(...)`, `json_u32_field(...)`, `decode_json_string(...)`, message constructors, request serialization, HTTP/client code, receipt/proof structs, NDJSON code, or tests in this item.
   - Done when: `json_string_field(body, field)` still finds `field` from the full response body and decodes the JSON string value through `json_string_at(...)`; `message_content_field(body)` still finds `"message"` first, then finds `"content"` only from that message-scoped suffix; both helpers share the same private lookup helper; and no public API, wire format, receipt/proof, or network behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

17. [x] `src/capability/llm/openai.rs` test `openai_chat_response_parser_preserves_message_scoped_content_fields`: add regression coverage for scoped content extraction after the shared parser helper.
   - Scope: `src/capability/llm/openai.rs` test module only; use existing private test access to `parse_chat_response_body(...)`. Do not change production code in this item.
   - Done when: the named test builds a response body containing a non-message `"content"` value before `choices[0].message.content`, a valid `"id"`, token fields, and a browser `"target_url"`; asserts that `parse_chat_response_body(...)` returns the message-scoped content rather than the earlier non-message content; and asserts `id`, `target_url`, `prompt_tokens`, `completion_tokens`, `total_tokens`, `response_hash`, and `raw_hash` remain deterministic. The test must not perform network I/O, filesystem I/O outside normal cargo test execution, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test openai_chat_response_parser_preserves_message_scoped_content_fields -- --test-threads=1`.

18. [x] `SCORE_REPORT.md`: after items 16 and 17 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.


19. [ ] `src/agent/prompt.rs`: route certification phase prompt wrappers through one private phase helper while preserving distinct prompt identities.
   - Scope: `src/agent/prompt.rs` only; allowed functions are `analysis_prompt(...)`, `judgment_prompt(...)`, `plan_prompt(...)`, `eval_prompt(...)`, `recovery_prompt(...)`, `certification_prompt(...)`, `CertificationPrompt`, and at most one new private helper adjacent to `certification_prompt(...)`. Do not change `system_prompt(...)`, `planning_prompt(...)`, `CERTIFICATION_OUTPUT_RULE`, call sites in `src/agent/cycle.rs`, loop-driver prompt code, runtime phases, LLM transport code, or tests in this item.
   - Done when: all five certification prompt wrappers still return the same phase names, context labels, instruction text, domain interpolation, human-review line, no-tool rule, and final-verdict rule; each wrapper delegates through the same private helper boundary instead of repeating the `CertificationPrompt` construction; and public function signatures remain unchanged.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

20. [ ] `src/agent/prompt.rs` test `certification_phase_prompt_helper_preserves_distinct_phase_payloads`: add focused regression coverage for the shared certification phase helper.
   - Scope: `src/agent/prompt.rs` test module only; use existing public prompt functions. Do not change production code in this item.
   - Done when: the named test asserts that `analysis_prompt(...)`, `judgment_prompt(...)`, `plan_prompt(...)`, `eval_prompt(...)`, and `recovery_prompt(...)` retain their exact phase labels, phase-specific context labels, phase-specific instruction fragments, shared domain line, `HUMAN_REVIEW_REQUIRED`, `Do not call tools`, `Return plain text only`, `VERDICT: pass`, and `VERDICT: fail` behavior. The test must not perform network I/O, filesystem I/O outside normal cargo test execution, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test certification_phase_prompt_helper_preserves_distinct_phase_payloads -- --test-threads=1`.

21. [ ] `SCORE_REPORT.md`: after items 19 and 20 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows, and `score.md` changes only if refreshed evidence differs from the current rationale.
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
