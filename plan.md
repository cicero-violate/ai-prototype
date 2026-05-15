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


Planning reconnaissance refresh on 2026-05-15 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; confirmed Active Priorities items 89 through 91 are complete and that there was no first incomplete executable item before this update; inspected current `../state/rustc/auto-refactor/*.graph-editor-plan.json`; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json`; inspected candidate source and test surfaces in `src/agent/loop_driver.rs`; and checked the working tree through the project connector. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.93 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `6.9`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.2`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and currently contains 1,572 planned operations. Current planning selected graph operation `59df2500a9eb2620`, covering `agent::loop_driver::{agent_identity, agent_tag}`, as the next safe non-`root_validate` consolidation area. The safe subset is private agent-label helper delegation only: keep both private helper names and signatures, preserve exact single-agent prompt identity text, preserve exact multi-agent prompt identity text, preserve exact single-agent log tag `agent`, preserve exact multi-agent log tag `agent-{agent_id}`, preserve all planning/execution prompt call sites, preserve cycle labels and retry labels, and do not touch router/client code, workspace sync, run-cycle scheduling, spawned-agent behavior, graph mutation code, or `root_validate`.

92. [ ] `src/agent/loop_driver.rs`: route `agent_identity(...)` and `agent_tag(...)` through one private agent-label formatting helper while preserving prompt identity and log-tag semantics.
   - Scope: `src/agent/loop_driver.rs` only; allowed production functions are `agent_identity(...)`, `agent_tag(...)`, and at most one new private helper adjacent to them. Do not change `retry_attempt_label(...)`, `project_turn_mode(...)`, `planning_prompt(...)`, `execute_prompt(...)`, `spawn_agent_prompt(...)`, `LoopDriver::run_agent_loop(...)`, workspace sync, router/client code, sleep scheduling, cycle execution, mailbox logic, graph mutation code, or tests in this item.
   - Done when: `agent_identity(agent_id, agent_count)` still returns `You are the agent for this project.` for single-agent mode and `You are **Agent {agent_id}** (one of {agent_count} parallel agents).` for multi-agent mode; `agent_tag(agent_id, agent_count)` still returns `agent` for single-agent mode and `agent-{agent_id}` for multi-agent mode; both wrappers delegate through the same private helper boundary; and no prompt body, log label, retry-label, scheduling, transport, filesystem, or network behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

93. [ ] `src/agent/loop_driver.rs` test `agent_label_helpers_preserve_identity_and_tag_boundaries`: add focused regression coverage for the shared agent-label helper.
   - Scope: `src/agent/loop_driver.rs` test module only; use private test access to `agent_identity(...)`, `agent_tag(...)`, `planning_prompt(...)`, `execute_prompt(...)`, and existing prompt-test patterns. Do not change production code in this item.
   - Done when: the named test asserts single-agent identity text, multi-agent identity text for at least two agent ids, single-agent tag text, multi-agent tag text, planning-prompt inclusion of the identity line, execution-prompt inclusion of the identity line, and separation between prompt identity strings and log tag strings. The test must not perform network I/O, filesystem I/O outside normal cargo test execution, environment mutation, thread spawning, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test agent_label_helpers_preserve_identity_and_tag_boundaries -- --test-threads=1`.

94. [ ] `SCORE_REPORT.md`: after items 92 and 93 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.


Planning reconnaissance refresh on 2026-05-15 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; confirmed Active Priorities items 13 through 66 are complete with no first incomplete executable item; inspected current `../state/rustc/auto-refactor/*.graph-editor-plan.json`; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json`; inspected candidate source and test surfaces in `src/capability/llm/openai.rs` and `src/lib.rs`; and checked the working tree through the project connector. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.99 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.4`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and currently contains 1,582 planned operations. Current planning selected graph operation `e9030dda9e8b24ae`, covering `capability::llm::openai::OpenAiJudgmentProofEvent::{expected_proof_hash, verifier_context_hash}`, as the next safe non-root consolidation area. The safe subset is private ordered hash-fold helper delegation only: keep the two public hash method names and their distinct proof versus verifier-context domain seeds, exact field order, non-zero `h.max(1)` behavior, receipt/proof projection, replay, NDJSON, and network semantics unchanged. Do not touch OpenAI request serialization, HTTP/client code, retry budget logic, receipt creation, proof finalization, OpenAI NDJSON loaders/encoders, Ollama code, loop-driver call sites, or `root_validate`.
Planning reconnaissance refresh on 2026-05-14 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; inspected current `../state/rustc/auto-refactor/*.graph-editor-plan.json`; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json`; inspected candidate source and test surfaces in `src/capability/llm/openai.rs`; and checked the working tree. There was no first incomplete executable item before this planning turn: Active Priorities items 13 through 15 were complete and `status.md` confirmed checklist exhaustion. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan. Existing uncommitted non-planning changes in `GOAL.md` and `src/agent/loop_driver.rs`, plus pre-existing `status.md` execution-blocker ledger additions, are outside this planning scope and must not be overwritten by execution turns.

Current graph-derived aggregate evidence remains `G = 7.99 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.4`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan remains schema version 1, graph schema version 16, and contains 1,614 planned operations. Recently reconciled non-`root_validate` surfaces include `agent::config::{env_u32, env_u64}`, `agent::cycle::{recovery_gate, recovery_target_phase}`, `agent::objective::{with_risk_envelope, with_stop_condition}`, `capability::llm::ollama::OllamaConfig::{base_url_id, model_id}`, `capability::llm::ollama::OllamaMessage::{system, user}`, `capability::llm::openai::OpenAiFunctionTool::{new, with_description}`, `capability::llm::openai::OpenAiConfig::{base_url_id, model_id}`, and related graph-backed helper candidates. `agent::loop_driver` candidates remain rejected for this turn because `src/agent/loop_driver.rs` has unrelated uncommitted edits. Graph candidates that merge semantically distinct hash, route, bridge, score, receipt, proof, loader, encoder, or record-family functions remain rejected unless the execute turn preserves their public semantic APIs and consolidates only private construction mechanics.

Graph operation `fefb1237c6a1ab0a` identified `capability::llm::openai::{json_string_field, message_content_field}` and has now been reconciled by completed items 16 through 18. Graph-backed prompt candidates `agent::prompt::{analysis_prompt, judgment_prompt, plan_prompt, eval_prompt, recovery_prompt}` have now been reconciled by completed items 19 through 21. Current planning turn selected graph operations `857e0d881591a854`, `9738a57e57cccc3a`, and `8c69ad27455480ac`, covering `capability::observation::source::ObservationIngressBatch::{empty, backpressure, rejected}`, as the next safe non-root consolidation area. The safe subset is private constructor-helper delegation only: keep the three public constructor names and their distinct `ObservationIngressDecision` values, source-hash behavior, backlog behavior, cursor behavior, empty-record behavior, receipt/contract semantics, and caller behavior unchanged. Do not touch `ObservationIngressBatch::accepted(...)`, receipt hashing, source ingestion, API DTO conversion, loop-driver call sites, or `src/agent/loop_driver.rs` because it has unrelated uncommitted edits.

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


19. [x] `src/agent/prompt.rs`: route certification phase prompt wrappers through one private phase helper while preserving distinct prompt identities.
   - Scope: `src/agent/prompt.rs` only; allowed functions are `analysis_prompt(...)`, `judgment_prompt(...)`, `plan_prompt(...)`, `eval_prompt(...)`, `recovery_prompt(...)`, `certification_prompt(...)`, `CertificationPrompt`, and at most one new private helper adjacent to `certification_prompt(...)`. Do not change `system_prompt(...)`, `planning_prompt(...)`, `CERTIFICATION_OUTPUT_RULE`, call sites in `src/agent/cycle.rs`, loop-driver prompt code, runtime phases, LLM transport code, or tests in this item.
   - Done when: all five certification prompt wrappers still return the same phase names, context labels, instruction text, domain interpolation, human-review line, no-tool rule, and final-verdict rule; each wrapper delegates through the same private helper boundary instead of repeating the `CertificationPrompt` construction; and public function signatures remain unchanged.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

20. [x] `src/agent/prompt.rs` test `certification_phase_prompt_helper_preserves_distinct_phase_payloads`: add focused regression coverage for the shared certification phase helper.
   - Scope: `src/agent/prompt.rs` test module only; use existing public prompt functions. Do not change production code in this item.
   - Done when: the named test asserts that `analysis_prompt(...)`, `judgment_prompt(...)`, `plan_prompt(...)`, `eval_prompt(...)`, and `recovery_prompt(...)` retain their exact phase labels, phase-specific context labels, phase-specific instruction fragments, shared domain line, `HUMAN_REVIEW_REQUIRED`, `Do not call tools`, `Return plain text only`, `VERDICT: pass`, and `VERDICT: fail` behavior. The test must not perform network I/O, filesystem I/O outside normal cargo test execution, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test certification_phase_prompt_helper_preserves_distinct_phase_payloads -- --test-threads=1`.

21. [x] `SCORE_REPORT.md`: after items 19 and 20 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.

22. [x] `src/capability/observation/source.rs`: route `ObservationIngressBatch::{empty, backpressure, rejected}` through one private constructor helper while preserving distinct ingress decisions.
   - Scope: `src/capability/observation/source.rs` only; allowed functions are `ObservationIngressBatch::empty(...)`, `ObservationIngressBatch::backpressure(...)`, `ObservationIngressBatch::rejected(...)`, and at most one new private helper inside `impl ObservationIngressBatch`. Do not change `ObservationIngressBatch::accepted(...)`, `ObservationIngressBatch::is_accepted(...)`, `ObservationIngressBatch::is_contract_valid(...)`, `ObservationIngressBatch::submission(...)`, `ObservationIngressBatch::receipt(...)`, `ObservationIngressReceipt`, source ingestion logic, hash helpers, API DTO conversion, loop-driver call sites, or tests in this item.
   - Done when: `empty(...)` still returns decision `Empty`, preserves the provided `source_id`, `source_hash`, and `cursor`, sets `backlog_len` to `0`, and has no records; `backpressure(...)` still returns decision `Backpressure`, preserves the provided `source_id`, `source_hash`, `cursor`, and `backlog_len`, and has no records; `rejected(...)` still returns decision `Rejected`, preserves the provided `source_id` and `cursor`, sets `source_hash` to `0`, sets `backlog_len` to `0`, and has no records; and all three public constructors delegate to the same private helper boundary.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

23. [x] `src/capability/observation/source.rs` test `observation_ingress_batch_constructors_preserve_decision_boundaries`: add focused regression coverage for the shared non-accepted ingress batch constructor helper.
   - Scope: `src/capability/observation/source.rs` test module only; use existing public `ObservationIngressBatch` constructors and `ObservationCursor`. Do not change production code in this item.
   - Done when: the named test asserts that `empty(...)`, `backpressure(...)`, and `rejected(...)` preserve their exact decisions, source-id behavior, source-hash behavior, cursor fields, backlog lengths, empty record vectors, `is_accepted()` false result, `is_contract_valid()` false result, and receipt decision/backlog/record-count behavior. The test must not perform network I/O, filesystem I/O outside normal cargo test execution, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test observation_ingress_batch_constructors_preserve_decision_boundaries -- --test-threads=1`.

24. [x] `SCORE_REPORT.md`: after items 22 and 23 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.



Current planning turn selected graph operation `a3d596dc34917ca3`, covering `capability::tooling::record::process::LiveSandboxProcessExecutor::{with_allowed_command, with_locked_env}`, as the next safe non-`root_validate` consolidation area. The safe subset is private builder-helper delegation only: keep both public builder names and signatures, preserve insertion order, preserve exact `allowed_commands` string storage, preserve exact `(key, value)` locked-env storage, preserve all defaults from `new(...)`, and do not touch process execution, authorization, request hashing, receipt hashing, NDJSON, replay verification, TLog/API call sites, or filesystem/process-spawning behavior.

25. [x] `src/capability/tooling/record/process.rs`: route `LiveSandboxProcessExecutor::{with_allowed_command, with_locked_env}` through one private builder mutation helper while preserving builder semantics.
   - Scope: `src/capability/tooling/record/process.rs` only; allowed functions are `LiveSandboxProcessExecutor::with_allowed_command(...)`, `LiveSandboxProcessExecutor::with_locked_env(...)`, and at most one new private helper inside `impl LiveSandboxProcessExecutor`. Do not change `LiveSandboxProcessExecutor::new(...)`, `with_timeout_ms(...)`, `with_max_output_bytes(...)`, `with_registry(...)`, `execute_process(...)`, `replay_receipt(...)`, process planning, command validation, environment validation, request hashing, receipt hashing, NDJSON encoding/loading, or tests in this item.
   - Done when: `with_allowed_command(...)` still appends exactly one command string to `allowed_commands` and leaves `locked_env`, timeouts, output limit, root, and registry unchanged; `with_locked_env(...)` still appends exactly one `(key, value)` pair to `locked_env` and leaves `allowed_commands`, timeouts, output limit, root, and registry unchanged; both builders still return `Self`; and both builders delegate through the same private helper boundary.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

26. [x] `src/capability/tooling/record/process.rs` test `live_sandbox_process_executor_builders_preserve_policy_boundaries`: add focused regression coverage for the shared builder mutation helper.
   - Scope: `src/capability/tooling/record/process.rs` test module only; use existing public `LiveSandboxProcessExecutor` builder APIs. Do not change production code in this item.
   - Done when: the named test constructs an executor with a deterministic root, two allowed commands, two locked environment values, a timeout, and an output limit; asserts command insertion order, locked-env insertion order, root preservation, timeout preservation, output-limit preservation, canonical registry preservation, and independence between command and locked-env builders; and performs no process spawning, network I/O, or filesystem I/O outside normal cargo test execution.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test live_sandbox_process_executor_builders_preserve_policy_boundaries -- --test-threads=1`.

27. [x] `SCORE_REPORT.md`: after items 25 and 26 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.




Current planning turn selected graph operation 48 from the selected local ai graph-editor plan, covering agent::worker_client::WorkerClient::{new, new_with_timeout}, as the next safe non-root consolidation area. The safe subset is private constructor-helper delegation only: keep both public constructor names and signatures, preserve the default timeout constant, preserve custom timeout milliseconds, preserve port storage, preserve from_env behavior, and do not touch HTTP request construction, socket connection logic, response parsing, worker API call sites, or network behavior.

28. [x] src/agent/worker_client.rs: route WorkerClient::{new, new_with_timeout} through one private constructor helper while preserving default and custom timeout semantics.
   - Scope: src/agent/worker_client.rs only; allowed functions are WorkerClient::new(...), WorkerClient::new_with_timeout(...), and at most one new private helper inside impl WorkerClient. Do not change WorkerClient::from_env(...), health(...), state(...), submit_command(...), get(...), post(...), send(...), parse_response(...), error formatting, HTTP wire strings, socket behavior, or tests in this item.
   - Done when: new(port) still stores the provided port and uses DEFAULT_TIMEOUT_MS; new_with_timeout(port, timeout_ms) still stores the provided port and uses Duration::from_millis(timeout_ms); both public constructors delegate through the same private helper boundary; and no public API or network behavior changes.
   - Validation: TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib.

29. [x] src/agent/worker_client.rs test worker_client_constructors_preserve_default_and_custom_timeouts: strengthen focused regression coverage for the shared constructor helper.
   - Scope: src/agent/worker_client.rs test module only; use existing private test access to WorkerClient fields and DEFAULT_TIMEOUT_MS. Do not change production code in this item.
   - Done when: the named test asserts that WorkerClient::new(...) preserves its port and default timeout, WorkerClient::new_with_timeout(...) preserves its port and custom timeout, distinct ports remain distinct, default and custom timeout paths remain distinct, and the test performs no network I/O, filesystem I/O, environment mutation, or process spawning.
   - Validation: TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test worker_client_constructors_preserve_default_and_custom_timeouts -- --test-threads=1.

30. [x] SCORE_REPORT.md: after items 28 and 29 land, refresh graph-derived structural evidence and review whether score.md rationale changes without raising project-level scores absent capability evidence.
   - Scope: SCORE_REPORT.md, score.md, plan.md, and status.md only.
   - Done when: scripts/recapture_rustc_graphs.sh --check validates the configured graph root, SCORE_REPORT.md is regenerated from ../state/rustc, status.md records aggregate and affected crate rows, and score.md changes only if refreshed evidence differs from the current rationale.
   - Validation: bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)".


Current planning turn selected graph operation `cef5bdfc17f274a8`, covering `capability::llm::ollama::OllamaJudgmentProofEvent::{expected_proof_hash, verifier_context_hash}`, as the next safe non-`root_validate` consolidation area. The safe subset is private hash-helper delegation only: keep both public method names and signatures, preserve the distinct domain seed constants, preserve exact field order for proof hashes and verifier-context hashes, preserve `h.max(1)` non-zero behavior, preserve `proof_flags(...)`, canonical effect proof projection, receipt matching, NDJSON encoding/loading, replay verification, TLog/API call sites, and all network behavior. Do not touch `root_validate`, `src/agent/loop_driver.rs`, receipt finalization semantics, or OpenAI proof-event code in this item.

31. [x] `src/capability/llm/ollama.rs`: route `OllamaJudgmentProofEvent::{expected_proof_hash, verifier_context_hash}` through one private ordered hash-fold helper while preserving distinct hash domains.
   - Scope: `src/capability/llm/ollama.rs` only; allowed functions are `OllamaJudgmentProofEvent::expected_proof_hash(...)`, `OllamaJudgmentProofEvent::verifier_context_hash(...)`, and at most one new private helper adjacent to them. Do not change `OllamaJudgmentProofEvent::finalize_receipt(...)`, `finalize_receipt_after_tlog(...)`, `finalize_receipt_at_seq(...)`, `is_valid(...)`, `proof_flags(...)`, `to_canonical_effect_proof(...)`, `to_canonical_verification_proof_record(...)`, `matches_receipt(...)`, receipt hashing, NDJSON encoding/loading, replay verification, API/TLog call sites, HTTP/client code, OpenAI proof-event code, or tests in this item.
   - Done when: `expected_proof_hash(...)` still starts from `0x4f4c_4c41_4d41_5652u64`, mixes proof-line, receipt-core, receipt-event sequence, proof-event sequence, receipt-event hash, endpoint/model/retry/request fields, budget flags, verifier flags in the same order, and returns `h.max(1)`; `verifier_context_hash(...)` still starts from `0x4f4c_4c41_4d41_4354u64`, mixes endpoint/model/retry/request fields and budget flags in the same order, and returns `h.max(1)`; both public hash methods delegate through the same private helper boundary; and no public API, receipt/proof, replay, NDJSON, or network behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

32. [x] `src/lib.rs` test `ollama_proof_event_hash_helpers_preserve_distinct_domains`: add focused regression coverage for the shared proof-event hash helper.
   - Scope: `src/lib.rs` test module only; use existing public `OllamaJudgmentProofEvent` construction/projection helpers and existing proof-event test setup patterns. Do not change production code in this item.
   - Done when: the named test obtains a valid `OllamaJudgmentProofEvent`, asserts `expected_proof_hash()` equals the stored `proof_hash`, asserts `verifier_context_hash()` is non-zero and distinct from `expected_proof_hash()`, asserts `to_canonical_effect_proof(...)` binds the canonical proof verifier-context hash and proof hash to the same values, and asserts tampering a proof-only field changes `expected_proof_hash()` without changing `verifier_context_hash()` while tampering a verifier-context field changes both as appropriate. The test must not perform network I/O, environment mutation, process spawning, or filesystem I/O outside normal cargo test execution.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test ollama_proof_event_hash_helpers_preserve_distinct_domains -- --test-threads=1`.

33. [x] `SCORE_REPORT.md`: after items 31 and 32 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.


Next queued graph-backed work after item 33: selected graph operation `e9030dda9e8b24ae`, covering `capability::llm::openai::OpenAiJudgmentProofEvent::{expected_proof_hash, verifier_context_hash}`, as the next safe non-`root_validate` consolidation area. The safe subset is private hash-helper delegation only: keep both public method names and signatures, preserve the distinct domain seed constants, preserve exact field order for proof hashes and verifier-context hashes, preserve `h.max(1)` non-zero behavior, preserve `proof_flags(...)`, canonical effect proof projection, receipt matching, NDJSON encoding/loading, replay verification, TLog/API call sites, and all network behavior. Do not touch `root_validate`, `src/agent/loop_driver.rs`, receipt finalization semantics, or Ollama proof-event code in this item.

34. [x] `src/capability/llm/openai.rs`: route `OpenAiJudgmentProofEvent::{expected_proof_hash, verifier_context_hash}` through one private ordered hash-fold helper while preserving distinct hash domains.
   - Scope: `src/capability/llm/openai.rs` only; allowed functions are `OpenAiJudgmentProofEvent::expected_proof_hash(...)`, `OpenAiJudgmentProofEvent::verifier_context_hash(...)`, and at most one new private helper adjacent to them. Do not change `OpenAiJudgmentProofEvent::finalize_receipt_after_tlog(...)`, `finalize_receipt_at_seq(...)`, `is_valid(...)`, `proof_flags(...)`, `to_canonical_effect_proof(...)`, `to_canonical_verification_proof_record(...)`, `matches_receipt(...)`, receipt hashing, NDJSON encoding/loading, replay verification, API/TLog call sites, HTTP/client code, Ollama proof-event code, or tests in this item.
   - Done when: `expected_proof_hash(...)` still starts from `0x4f50_454e_4149_5052u64`, mixes proof-line, receipt-core, receipt-event sequence, proof-event sequence, receipt-event hash, endpoint/model/retry/request fields, budget flags, verifier flags in the same order, and returns `h.max(1)`; `verifier_context_hash(...)` still starts from `0x4f50_454e_4149_4354u64`, mixes endpoint/model/retry/request fields and budget flags in the same order, and returns `h.max(1)`; both public hash methods delegate through the same private helper boundary; and no public API, receipt/proof, replay, NDJSON, or network behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

35. [x] `src/lib.rs` test `openai_proof_event_hash_helpers_preserve_distinct_domains`: add focused regression coverage for the shared proof-event hash helper.
   - Scope: `src/lib.rs` test module only; use existing public `OpenAiJudgmentProofEvent` construction/projection helpers and existing proof-event test setup patterns. Do not change production code in this item.
   - Done when: the named test obtains a valid `OpenAiJudgmentProofEvent`, asserts `expected_proof_hash()` equals the stored `proof_hash`, asserts `verifier_context_hash()` is non-zero and distinct from `expected_proof_hash()`, asserts `to_canonical_effect_proof(...)` binds the canonical proof verifier-context hash and proof hash to the same values, and asserts tampering a proof-only field changes `expected_proof_hash()` without changing `verifier_context_hash()` while tampering a verifier-context field changes both as appropriate. The test must not perform network I/O, environment mutation, process spawning, or filesystem I/O outside normal cargo test execution.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test openai_proof_event_hash_helpers_preserve_distinct_domains -- --test-threads=1`.

36. [x] `SCORE_REPORT.md`: after items 34 and 35 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.


Current planning turn selected graph split operation `f83874fb2b4b9aa3`, covering `agent::router::cdp_get`, as the next safe non-`root_validate` structural candidate. The safe subset is private phase extraction only: keep the public/private call signature of `cdp_get(...)` unchanged, preserve local socket resolution, timeout configuration, HTTP request bytes, response read behavior, CRLF header/body splitting, status parsing, and `OpenAiError` mapping. Do not touch `src/agent/loop_driver.rs`, router turn/session adoption, streaming request code, retry policy, CDP target matching, tab-close outcome semantics, or any network behavior outside the helper's internal decomposition.

37. [x] `src/agent/router.rs`: split `cdp_get(...)` into private request/response phase helpers while preserving the original `cdp_get(...)` signature.
   - Scope: `src/agent/router.rs` only; allowed functions are `cdp_get(...)` and at most two new private helpers adjacent to it, such as one helper for building the CDP HTTP GET request bytes and one helper for parsing `(status, body)` from the raw HTTP response string. Do not change `close_browser_tab_for_url(...)`, `close_cdp_target(...)`, `cdp_target_id_for_url(...)`, `devtools_page_target(...)`, retry helpers, streaming helpers, `RouterClient`, `RouterTabCloseOutcome`, or tests in this item.
   - Done when: `cdp_get(host, port, path, timeout_ms)` still resolves `(host, port)`, connects with `TcpStream::connect_timeout`, applies read/write timeouts, writes exactly `GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nAccept: application/json\r\nConnection: close\r\n\r\n`, flushes, reads the response to a string, parses the status code from the first response line, returns the body after the first `\r\n\r\n`, and maps malformed responses to `OpenAiError::InvalidResponse`; the request construction and response parsing phases are delegated to private helpers.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

38. [x] `src/agent/router.rs` test `cdp_get_helpers_preserve_http_request_and_response_parsing`: add focused regression coverage for the extracted `cdp_get(...)` helpers.
   - Scope: `src/agent/router.rs` test module only; use an existing private-test pattern and a local `TcpListener` loopback server inside the test. Do not change production code in this item.
   - Done when: the named test serves one deterministic local HTTP response, calls `cdp_get("127.0.0.1", port, "/json/list", timeout_ms)`, asserts that the server observed the exact GET request line, Host header, Accept header, and Connection header, and asserts that the returned status and body match the fixture. The test must not perform external network I/O, environment mutation, process spawning, or filesystem I/O outside normal cargo test execution.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test cdp_get_helpers_preserve_http_request_and_response_parsing -- --test-threads=1`.

39. [x] `SCORE_REPORT.md`: after items 37 and 38 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.


Planning reconnaissance refresh on 2026-05-14 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; preserved the pre-existing `status.md` implementation-step-4 blocker entry; inspected `../state/rustc/auto-refactor/*.graph-editor-plan.json`; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json`; inspected candidate source and test surfaces in `src/agent/loop_driver.rs`; and checked the working tree. There was no first incomplete executable item before this planning turn: Active Priorities items 13 through 39 were complete. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.99 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.4`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan remains schema version 1, graph schema version 16, and contains 1,582 planned operations. Graph operation `001e821dc83e940a` identifies `agent::loop_driver::LoopDriver::run_cycle` as a high-impact non-`root_validate` split candidate. The safe subset for this plan is private preparation-helper extraction only: keep `run_cycle(...)` private signature, turn retry loop, router streaming call, receipt writing, observation-ingress submission, eval submission, policy-learning call, sleeping behavior, spawned/non-spawned behavior, command URL behavior, and error strings unchanged. Do not touch `run_cycle_attempt(...)`, router/client code, kernel/TLog submission helpers, prompt text helpers beyond calling existing `build_turn_prompt_context(...)`, process spawning, filesystem paths outside existing goal/score reads, or network behavior.

40. [x] `src/agent/loop_driver.rs`: split `LoopDriver::run_cycle(...)` by extracting one private cycle-preparation helper for spawned/project mode turn scheduling.
   - Scope: `src/agent/loop_driver.rs` only; allowed code is `LoopDriver::run_cycle(...)` and at most one new private helper plus a small private return struct near `TurnPromptContext` or adjacent helper definitions. Do not change `run_agent_loop(...)`, `run_cycle_attempt(...)`, `submit_cycle_start_observation_ingress(...)`, `build_turn_prompt_context(...)`, prompt builders, router/client calls, receipt/TLog/eval/learning helpers, sleeps, retry loop, environment reads, or tests in this item.
   - Done when: `run_cycle(...)` delegates computation of `is_spawned`, `command_url`, `total_turns`, `turn_offset`, and `goal` to the new private helper; spawned agents still run exactly `execute_turns` turns with `turn_offset = 1` and an empty goal; non-spawned agents still run `execute_turns + 1` turns with `turn_offset = 0` and read `GOAL.md` through `read_goal_file(...)`; and all existing public/private signatures and error behavior are preserved.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

41. [x] `src/agent/loop_driver.rs` test `run_cycle_preparation_preserves_project_and_spawned_turn_schedules`: add focused regression coverage for the extracted cycle-preparation helper.
   - Scope: `src/agent/loop_driver.rs` test module only; use `minimal_loop_config(...)`, a temporary directory under `../.tmp`, and existing private test access. Do not change production code in this item.
   - Done when: the named test proves project mode reads the fixture `GOAL.md`, sets `is_spawned = false`, produces `total_turns = execute_turns + 1`, `turn_offset = 0`, and preserves the configured command URL; and spawned mode with `domain`/`metric` set does not require or read `GOAL.md`, sets `is_spawned = true`, produces `total_turns = execute_turns`, `turn_offset = 1`, and returns an empty goal. The test must not perform network I/O, environment mutation outside the scoped config value, process spawning, or filesystem I/O outside the temporary fixture directory.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test run_cycle_preparation_preserves_project_and_spawned_turn_schedules -- --test-threads=1`.

42. [x] `SCORE_REPORT.md`: after items 40 and 41 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.




Planning reconnaissance refresh on 2026-05-14 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; inspected current `../state/rustc/auto-refactor/*.graph-editor-plan.json`; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json`; inspected candidate source and test surfaces in `src/capability/llm/ollama.rs` and `src/lib.rs`; and checked the working tree. There was no first incomplete executable item before this planning turn: Active Priorities items 13 through 42 were complete and implementation steps 4 and 5 were blocked by checklist exhaustion. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.99 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.4`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and contains 1,582 planned operations. Graph operation `fc62d48f89386146` identifies `capability::llm::ollama::OllamaLlmEffectReceipt::{canonical_authority_hash, canonical_request_hash}` as the next safe non-`root_validate` consolidation area. The safe subset is private ordered hash-fold delegation only: keep both public method names and signatures, preserve distinct authority/request domain seed constants, preserve exact authority field order and request field order, preserve `h.max(1)` non-zero behavior, preserve `is_valid()` gating, preserve canonical effect receipt binding, verification proof binding, receipt/proof hashes, NDJSON encoding/loading, replay verification, TLog/API call sites, and all network behavior. Do not touch `root_validate`, `src/agent/loop_driver.rs`, OpenAI receipt code, judgment proof-event code, receipt finalization semantics, or HTTP/client code in this item.

43. [x] `src/capability/llm/ollama.rs`: route `OllamaLlmEffectReceipt::{canonical_authority_hash, canonical_request_hash}` through one private ordered hash-fold helper while preserving distinct authority and request domains.
   - Scope: `src/capability/llm/ollama.rs` only; allowed functions are `OllamaLlmEffectReceipt::canonical_authority_hash(...)`, `OllamaLlmEffectReceipt::canonical_request_hash(...)`, and at most one new private helper adjacent to them. Do not change `OllamaLlmEffectReceipt::canonical_effect(...)`, `canonical_effect_receipt(...)`, `verification_proof_binding(...)`, `has_proof_binding(...)`, `expected_receipt_core_hash(...)`, receipt hashing, judgment proof-event hashing, NDJSON encoding/loading, replay verification, API/TLog call sites, HTTP/client code, OpenAI code, or tests in this item.
   - Done when: `canonical_authority_hash(...)` still returns `None` for invalid receipts, starts from `0x4f4c_4c41_4d41_4155u64`, mixes `provider_hash`, `base_url_hash`, `model_id`, `timeout_ms`, `max_retries`, `attempt_budget`, and `retry_budget_hash` in the same order, and returns `Some(h.max(1))`; `canonical_request_hash(...)` still returns `None` for invalid receipts, starts from `0x4f4c_4c41_4d41_5251u64`, mixes `request_hash`, `command_hash`, and `request_identity_hash` in the same order, and returns `Some(h.max(1))`; both public hash methods delegate through the same private helper boundary; and no public API, receipt/proof, replay, NDJSON, or network behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

44. [x] `src/lib.rs` test `ollama_effect_receipt_hash_helpers_preserve_authority_request_domains`: add focused regression coverage for the shared effect-receipt hash helper.
   - Scope: `src/lib.rs` test module only; use existing public `OllamaLlmEffectReceipt` construction/projection helpers and existing effect-receipt test setup patterns. Do not change production code in this item.
   - Done when: the named test obtains a valid `OllamaLlmEffectReceipt`, asserts `canonical_authority_hash()` and `canonical_request_hash()` are non-zero and distinct, asserts `canonical_effect_receipt()` binds `authority_hash` and `request_hash` to those same values, asserts tampering an authority-only field changes `canonical_authority_hash()` without changing `canonical_request_hash()`, asserts tampering a request-only field changes `canonical_request_hash()` without changing `canonical_authority_hash()`, and asserts an invalid receipt returns `None` for both hash helpers and no canonical effect receipt. The test must not perform network I/O, environment mutation, process spawning, or filesystem I/O outside normal cargo test execution.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test ollama_effect_receipt_hash_helpers_preserve_authority_request_domains -- --test-threads=1`.

45. [x] `SCORE_REPORT.md`: after items 43 and 44 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.


Planning reconnaissance refresh on 2026-05-14 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; inspected current `../state/rustc/auto-refactor/*.graph-editor-plan.json`; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json`; inspected candidate source and test surfaces in `src/capability/llm/openai.rs` and `src/lib.rs`; and checked the working tree. There was no first incomplete executable item before this planning turn: Active Priorities items 13 through 45 were complete. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.99 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.4`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and contains 1,582 planned operations. Graph operation `ef272f0c65098854` identifies `capability::llm::openai::OpenAiLlmEffectReceipt::{canonical_authority_hash, canonical_request_hash}` as the next safe non-`root_validate` consolidation area. The safe subset is private ordered hash-fold delegation only: keep both public method names and signatures, preserve distinct authority/request domain seed constants, preserve exact authority field order and request field order, preserve `h.max(1)` non-zero behavior, preserve `is_valid()` gating, preserve canonical effect receipt binding, verification proof binding, receipt/proof hashes, NDJSON encoding/loading, replay verification, TLog/API call sites, and all network behavior. Do not touch `root_validate`, `src/agent/loop_driver.rs`, Ollama receipt code, judgment proof-event code, receipt finalization semantics, or HTTP/client code in this item.

46. [x] `src/capability/llm/openai.rs`: route `OpenAiLlmEffectReceipt::{canonical_authority_hash, canonical_request_hash}` through one private ordered hash-fold helper while preserving distinct authority and request domains.
   - Scope: `src/capability/llm/openai.rs` only; allowed functions are `OpenAiLlmEffectReceipt::canonical_authority_hash(...)`, `OpenAiLlmEffectReceipt::canonical_request_hash(...)`, and at most one new private helper adjacent to them. Do not change `OpenAiLlmEffectReceipt::canonical_effect(...)`, `canonical_effect_receipt(...)`, `verification_proof_binding(...)`, `has_proof_binding(...)`, `expected_receipt_core_hash(...)`, receipt hashing, judgment proof-event hashing, NDJSON encoding/loading, replay verification, API/TLog call sites, HTTP/client code, Ollama code, or tests in this item.
   - Done when: `canonical_authority_hash(...)` still returns `None` for invalid receipts, starts from `0x4f50_454e_4149_4155u64`, mixes `provider_hash`, `base_url_hash`, `model_id`, `timeout_ms`, `max_retries`, `attempt_budget`, and `retry_budget_hash` in the same order, and returns `Some(h.max(1))`; `canonical_request_hash(...)` still returns `None` for invalid receipts, starts from `0x4f50_454e_4149_5251u64`, mixes `request_hash`, `command_hash`, and `request_identity_hash` in the same order, and returns `Some(h.max(1))`; both public hash methods delegate through the same private helper boundary; and no public API, receipt/proof, replay, NDJSON, or network behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

47. [x] `src/lib.rs` test `openai_effect_receipt_hash_helpers_preserve_authority_request_domains`: add focused regression coverage for the shared effect-receipt hash helper.
   - Scope: `src/lib.rs` test module only; use existing public `OpenAiLlmEffectReceipt` construction/projection helpers and the existing Ollama effect-receipt hash test pattern. Do not change production code in this item.
   - Done when: the named test obtains a valid `OpenAiLlmEffectReceipt`, asserts `canonical_authority_hash()` and `canonical_request_hash()` are non-zero and distinct, asserts `canonical_effect_receipt()` binds `authority_hash` and `request_hash` to those same values, asserts tampering an authority-only field changes `canonical_authority_hash()` without changing `canonical_request_hash()`, asserts tampering a request-only field changes `canonical_request_hash()` without changing `canonical_authority_hash()`, and asserts an invalid receipt returns `None` for both hash helpers and no canonical effect receipt. The test must not perform network I/O, environment mutation, process spawning, or filesystem I/O outside normal cargo test execution.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test openai_effect_receipt_hash_helpers_preserve_authority_request_domains -- --test-threads=1`.

48. [x] `SCORE_REPORT.md`: after items 46 and 47 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.

Planning reconnaissance refresh on 2026-05-14 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; inspected current `../state/rustc/auto-refactor/*.graph-editor-plan.json`; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json`; inspected candidate source and test surfaces in `src/capability/llm/ollama.rs`; and checked the working tree. There was no first incomplete executable item before this planning turn: Active Priorities items 46 through 48 were complete. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.99 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.4`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and contains 1,582 planned operations. Graph operation `7cf3b1c32959e16d` identifies `capability::llm::ollama::{encode_ollama_judgment_proof_event_ndjson, encode_ollama_llm_effect_receipt_ndjson}` as the next safe non-`root_validate` consolidation area. The safe subset is private NDJSON field-list encoding delegation only: keep both named encoder functions, preserve exact schema-version fields, record-type fields, field ordering, numeric casts, comma-separated JSON-array format, append/load verification behavior, receipt/proof hashes, replay verification, TLog/API call sites, and all network behavior. Do not touch `root_validate`, `src/agent/loop_driver.rs`, decoder semantics, loader semantics, receipt finalization semantics, OpenAI code, or HTTP/client code in this item.

49. [x] `src/capability/llm/ollama.rs`: route `encode_ollama_llm_effect_receipt_ndjson(...)` and `encode_ollama_judgment_proof_event_ndjson(...)` through one private numeric-field NDJSON encoder helper while preserving distinct record layouts.
   - Scope: `src/capability/llm/ollama.rs` only; allowed functions are `encode_ollama_llm_effect_receipt_ndjson(...)`, `encode_ollama_judgment_proof_event_ndjson(...)`, and at most one new private helper adjacent to them. Do not change append functions, decode functions, load functions, verification functions, receipt/proof structs, receipt/proof hashing, replay verification, API/TLog call sites, OpenAI code, HTTP/client code, or tests in this item.
   - Done when: `encode_ollama_llm_effect_receipt_ndjson(...)` still emits exactly the same JSON array fields in the same order for an `OllamaLlmEffectReceipt`; `encode_ollama_judgment_proof_event_ndjson(...)` still emits exactly the same JSON array fields in the same order for an `OllamaJudgmentProofEvent`; both encoders delegate the shared numeric-field join/array formatting through the same private helper boundary; and no public API, receipt/proof, replay, NDJSON decode/load, or network behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

50. [x] `src/capability/llm/ollama.rs` test `ollama_ndjson_encoders_preserve_record_layouts`: add focused regression coverage for the shared Ollama NDJSON encoder helper.
   - Scope: `src/capability/llm/ollama.rs` test module only; use existing private test access to `encode_ollama_llm_effect_receipt_ndjson(...)`, `encode_ollama_judgment_proof_event_ndjson(...)`, `parse_u64_fields(...)`, and decode helpers or existing public receipt/proof construction patterns. Do not change production code in this item.
   - Done when: the named test constructs one valid `OllamaLlmEffectReceipt` and one valid `OllamaJudgmentProofEvent`, asserts each encoded line parses to the exact expected schema version, record type, field count, and selected sentinel field positions, asserts each line decodes back to the original record through the existing decoder path, and performs no network I/O, environment mutation, process spawning, or filesystem I/O outside normal cargo test execution.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test ollama_ndjson_encoders_preserve_record_layouts -- --test-threads=1`.

51. [x] `SCORE_REPORT.md`: after items 49 and 50 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.

Planning reconnaissance refresh on 2026-05-14 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; inspected current `../state/rustc/auto-refactor/*.graph-editor-plan.json` with Python; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json`; inspected candidate source and test surfaces in `src/capability/verification/proof.rs`; checked existing `CanonicalEffect` call sites; and checked the working tree. There was no first incomplete executable item before this planning turn: Active Priorities items 13 through 51 were complete and implementation had exhausted the checklist. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.99 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.4`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and contains 1,582 planned operations. Graph operations `bd5ec4dd371f30e4`, `0011737595f8ccc9`, `21cd6ffbbc4641bb`, `8710f1bddcfe7b32`, `dd9df99277d8646c`, `6b91c9c1366496ec`, and `ec6c793a184933a0` identify duplicated `capability::verification::proof::CanonicalEffect::{artifact, process, semantic_verification, policy, observation}` wrapper constructors. The safe subset is private named-kind constructor-helper delegation only: keep every public wrapper name and signature, preserve each distinct `CanonicalEffectKind`, preserve `CanonicalEffect::new(...)` validation and `is_valid()` non-zero digest/metadata behavior, preserve `CanonicalEffect::llm(...)` metadata hashing, preserve `contract_hash(...)`, preserve canonical effect receipts/proofs, and do not touch tooling receipts, LLM receipts, policy store receipts, semantic verification repair, process execution, observation ingestion, graph mutation, TLog/API call sites, or `root_validate`.

52. [x] `src/capability/verification/proof.rs`: route `CanonicalEffect::{artifact, process, semantic_verification, policy, observation}` through one private named-kind constructor helper while preserving distinct effect kinds.
   - Scope: `src/capability/verification/proof.rs` only; allowed code is the five public wrapper constructors plus at most one new private helper inside `impl CanonicalEffect`. Do not change `CanonicalEffect::new(...)`, `CanonicalEffect::llm(...)`, `CanonicalEffect::is_valid(...)`, `CanonicalEffect::contract_hash(...)`, `CanonicalEffectKind`, `ProofSubjectKind`, `CanonicalEffectReceipt`, `CanonicalVerificationProofRecord`, encode/decode helpers, tooling receipt call sites, LLM receipt call sites, policy-store call sites, semantic verification code, or tests in this item.
   - Done when: `artifact(...)` still constructs `CanonicalEffectKind::Artifact`; `process(...)` still constructs `CanonicalEffectKind::Process`; `semantic_verification(...)` still constructs `CanonicalEffectKind::SemanticVerification`; `policy(...)` still constructs `CanonicalEffectKind::Policy`; `observation(...)` still constructs `CanonicalEffectKind::Observation`; all five still reject zero digest or metadata via the existing validation path; and all five public wrappers delegate to the same private helper boundary.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

53. [x] `src/lib.rs` test `canonical_effect_named_constructors_preserve_kind_and_validation_boundaries`: add focused regression coverage for the shared `CanonicalEffect` wrapper helper.
   - Scope: `src/lib.rs` test module only; use public `CanonicalEffect` constructors, `CanonicalEffectKind`, and `ProofSubjectKind` conversions that are already re-exported through the crate test surface. Do not change production code in this item.
   - Done when: the named test asserts that `artifact(...)`, `process(...)`, `semantic_verification(...)`, `policy(...)`, and `observation(...)` preserve exact `CanonicalEffectKind`, digest, metadata, non-zero `contract_hash()`, and expected `subject_kind()` mappings; asserts zero digest and zero metadata are rejected for each wrapper; and performs no network I/O, filesystem I/O outside normal cargo test execution, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test canonical_effect_named_constructors_preserve_kind_and_validation_boundaries -- --test-threads=1`.

54. [x] `SCORE_REPORT.md`: after items 52 and 53 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.

Planning reconnaissance refresh on 2026-05-14 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; inspected current `../state/rustc/auto-refactor/*.graph-editor-plan.json` with Python; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json`; inspected candidate source and existing tests in `src/agent/cycle.rs`; and checked the working tree. There was no first incomplete executable item before this planning turn: Active Priorities items 52 through 54 were complete and implementation steps 4 and 5 were blocked by checklist exhaustion. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.99 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.4`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and contains 1,582 planned operations. Graph operations `21edae3cb3c10f1b`, `96af74928d58e034`, and `447858122f87cb5a` identify duplicated `agent::cycle::{compute_evidence_contract_hash, compute_submit_evidence_command_hash, compute_envelope_hash}` domain-hash wrapper mechanics. The safe subset is private domain-hash vector delegation only: keep all three private wrapper names and signatures, preserve `HashDomain::{EvidenceContract, SubmitEvidenceCommand, CommandEnvelope}` seeds and prefix fields, preserve exact field order for evidence contracts, submit-evidence commands, and command envelopes, preserve `h.max(1)` non-zero behavior, preserve `build_submit_evidence_json(...)`, preserve real `EvidenceSubmission`, `Command::SubmitEvidence`, and `CommandEnvelope` hash equality, and do not touch runtime cycle transitions, router calls, TLog/API submission behavior, kernel types, or `root_validate`.

55. [x] `src/agent/cycle.rs`: route `compute_evidence_contract_hash(...)`, `compute_submit_evidence_command_hash(...)`, and `compute_envelope_hash(...)` through one private domain-hash vector helper while preserving hash-domain boundaries.
   - Scope: `src/agent/cycle.rs` only; allowed functions are `compute_evidence_contract_hash(...)`, `compute_submit_evidence_command_hash(...)`, `compute_envelope_hash(...)`, and at most one new private helper adjacent to the existing hash helpers. Do not change `compute_structural_payload_hash(...)`, `HashDomain`, `HashDomain::descriptor(...)`, `compute_domain_contract_hash(...)`, `mix_contract_hash(...)`, `build_submit_evidence_json(...)`, `CycleRunner`, route tables, runtime phase logic, router/client code, kernel/TLog submission helpers, or tests in this item.
   - Done when: `compute_evidence_contract_hash(...)` still hashes `[gate_u64, evidence_u64, passed as u64, effect_u64, payload_hash]` under `HashDomain::EvidenceContract`; `compute_submit_evidence_command_hash(...)` still hashes `[sub_contract_hash]` under `HashDomain::SubmitEvidenceCommand`; `compute_envelope_hash(...)` still hashes `[command_id, cmd_contract_hash]` under `HashDomain::CommandEnvelope`; all three wrappers delegate through the same private helper boundary; and no public API, JSON wire format, kernel hash, runtime behavior, or receipt behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

56. [x] `src/agent/cycle.rs` test `submit_evidence_hash_domain_wrappers_preserve_distinct_vectors`: add focused regression coverage for the shared domain-hash vector helper.
   - Scope: `src/agent/cycle.rs` test module only; use existing private test access to `gate_id_u64(...)`, `evidence_u64_value(...)`, `compute_structural_payload_hash(...)`, `compute_evidence_contract_hash(...)`, `compute_submit_evidence_command_hash(...)`, `compute_envelope_hash(...)`, `compute_domain_contract_hash(...)`, and `build_submit_evidence_json(...)`. Do not change production code in this item.
   - Done when: the named test proves invariant and plan evidence hash chains preserve the existing known contract, command, and envelope hash vectors; proves the three wrapper domains remain non-zero and pairwise distinct for the same scalar where applicable; proves field-order changes would produce different hashes by comparing against explicit alternate `compute_domain_contract_hash(...)` calls; and performs no network I/O, filesystem I/O outside normal cargo test execution, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test submit_evidence_hash_domain_wrappers_preserve_distinct_vectors -- --test-threads=1`.

57. [x] `SCORE_REPORT.md`: after items 55 and 56 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.

Current planning turn selected graph operations `1ac97f37b87d03c1`, `a01a6d0fe4befba7`, and `58e8cc4a0d31fa0e`, covering `capability::tooling::record::hash::{tool_command_hash, tool_input_hash, tool_output_hash}`, as the next safe non-`root_validate` consolidation area. The safe subset is private ordered hash-helper delegation only: keep all three wrapper function names and signatures, preserve each distinct domain seed, preserve exact field order for command, input, and output hashes, preserve `h.max(1)` non-zero behavior, preserve `ToolRequest::from_packet(...)`, `ToolRequest::is_valid_for(...)`, artifact/tool receipt hashing, process/tool replay verification, NDJSON behavior, and all filesystem/process behavior. Do not touch `root_validate`, receipt finalization semantics, artifact materialization, process execution, MCP request hashing, or call sites outside the named source/test scope in these items.

58. [x] `src/capability/tooling/record/hash.rs`: route `tool_command_hash(...)`, `tool_input_hash(...)`, and `tool_output_hash(...)` through one private ordered hash-vector helper while preserving distinct tool hash domains.
   - Scope: `src/capability/tooling/record/hash.rs` only; allowed functions are `tool_command_hash(...)`, `tool_input_hash(...)`, `tool_output_hash(...)`, and at most one new private helper adjacent to them. Do not change `tool_effect_output_hash(...)`, `mix_packet(...)`, `tool_failure_hash(...)`, artifact naming/body helpers, path or sandbox guards, process helpers, NDJSON helpers, `ToolRequest`, `ToolReceipt`, process receipt code, MCP receipt code, or tests in this item.
   - Done when: `tool_command_hash(packet)` still starts from `0xbb67ae8584caa73b`, mixes `packet.objective_id` then `packet.active_task_id`, and returns `h.max(1)`; `tool_input_hash(packet)` still starts from `0x13198a2e03707344`, mixes `packet.objective_id`, `packet.active_task_id`, `packet.ready_tasks as u64`, then `packet.revision`, and returns `h.max(1)`; `tool_output_hash(request)` still starts from `0x243f6a8885a308d3`, mixes `request.objective_id`, `request.task_id`, `request.command_hash`, then `request.input_hash`, and returns `h.max(1)`; all three wrappers delegate through the same private helper boundary; and no public API, receipt, replay, filesystem, or process behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

59. [x] `src/capability/tooling/record/hash.rs` test `tool_hash_wrappers_preserve_distinct_domains_and_vectors`: add focused regression coverage for the shared tool hash helper.
   - Scope: `src/capability/tooling/record/hash.rs` test module only; add a local `#[cfg(test)]` module if none exists and use existing private access to `tool_command_hash(...)`, `tool_input_hash(...)`, `tool_output_hash(...)`, `ToolRequest`, and `Packet`. Do not change production code in this item.
   - Done when: the named test constructs a deterministic `Packet`, derives a `ToolRequest` from it, asserts the existing command/input/output hash values are non-zero and pairwise distinct, asserts `ToolRequest::from_packet(...)` and `ToolRequest::is_valid_for(...)` still bind the command and input hashes to the packet, asserts changing `ready_tasks` or `revision` affects input hash without changing command hash, asserts changing `command_hash` or `input_hash` affects output hash, and performs no network I/O, process spawning, environment mutation, or filesystem I/O outside normal cargo test execution.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test tool_hash_wrappers_preserve_distinct_domains_and_vectors -- --test-threads=1`.

60. [x] `SCORE_REPORT.md`: after items 58 and 59 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.

Planning reconnaissance refresh on 2026-05-14 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; inspected current `../state/rustc/auto-refactor/*.graph-editor-plan.json` with Python; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json`; inspected candidate source and test surfaces in `src/capability/tooling/record/receipt.rs`; checked existing receipt/hash regression coverage; and checked the working tree. There was no first incomplete executable item before this planning turn: Active Priorities items 58 through 60 were complete and implementation steps 4 and 5 were blocked by checklist exhaustion. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.99 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.4`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and contains 1,582 planned operations. Graph operation `729490216f0ede3b` identifies `capability::tooling::record::receipt::ProcessEffectReceipt::{canonical_authority_hash, canonical_request_hash}` as the next safe non-`root_validate` consolidation area. The safe subset is private ordered hash-fold delegation only: keep both public method names and signatures, preserve distinct authority/request domain seed constants, preserve exact authority field order and request field order, preserve `is_valid()` gating, preserve `h.max(1)` non-zero behavior, preserve canonical effect receipt binding, verification proof binding, provider proof hashing, replay verification, NDJSON encoding/loading, TLog/API call sites, and all filesystem/process behavior. Do not touch `root_validate`, artifact receipt hashing, process execution, sandbox authorization, MCP receipt code, LLM receipt code, receipt finalization semantics, or call sites outside the named source/test scope in these items.

61. [x] `src/capability/tooling/record/receipt.rs`: route `ProcessEffectReceipt::{canonical_authority_hash, canonical_request_hash}` through one private ordered hash-vector helper while preserving distinct process receipt hash domains.
   - Scope: `src/capability/tooling/record/receipt.rs` only; allowed functions are `ProcessEffectReceipt::canonical_authority_hash(...)`, `ProcessEffectReceipt::canonical_request_hash(...)`, and at most one new private helper adjacent to them. Do not change `ProcessEffectReceipt::from_persisted_event(...)`, `is_valid(...)`, `replay_verified(...)`, `matches_event(...)`, `receipt_core_hash(...)`, `verifier_context_hash(...)`, `provider_proof_hash(...)`, `canonical_effect(...)`, `canonical_effect_receipt(...)`, `to_canonical_effect_proof(...)`, `proof_line_hash(...)`, `verification_proof_binding(...)`, artifact effect receipt code, NDJSON helpers, process execution, sandbox authorization, or tests in this item.
   - Done when: `canonical_authority_hash(...)` still returns `None` for invalid receipts, starts from `0x5052_4f43_4155_5448u64`, mixes `self.capability as u64` then `self.registry_policy_hash`, and returns `Some(h.max(1))`; `canonical_request_hash(...)` still returns `None` for invalid receipts, starts from `0x5052_4f43_5251_5354u64`, mixes only `self.request_hash`, and returns `Some(h.max(1))`; both public hash methods delegate through the same private helper boundary; and no public API, receipt/proof, replay, NDJSON, filesystem, sandbox, or process behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

62. [x] `src/lib.rs` test `process_effect_receipt_hash_helpers_preserve_authority_request_domains`: add focused regression coverage for the shared `ProcessEffectReceipt` authority/request hash helper.
   - Scope: `src/lib.rs` test module only; use existing public process tooling receipt construction/projection helpers and existing sandbox process receipt test setup patterns. Do not change production code in this item.
   - Done when: the named test obtains a valid `ProcessEffectReceipt`, asserts `canonical_authority_hash()` and `canonical_request_hash()` are non-zero and distinct, asserts `canonical_effect_receipt()` binds `authority_hash` and `request_hash` to those same values, asserts tampering an authority-only field changes `canonical_authority_hash()` without changing `canonical_request_hash()`, asserts tampering a request-only field changes `canonical_request_hash()` without changing `canonical_authority_hash()`, and asserts an invalid receipt returns `None` for both hash helpers and no canonical effect receipt. The test must not perform network I/O, environment mutation, process spawning, or filesystem I/O outside normal cargo test execution.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test process_effect_receipt_hash_helpers_preserve_authority_request_domains -- --test-threads=1`.

63. [x] `SCORE_REPORT.md`: after items 61 and 62 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.


Planning reconnaissance refresh on 2026-05-14 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; inspected current `../state/rustc/auto-refactor/*.graph-editor-plan.json` with Python; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json`; inspected candidate source and existing tests in `src/agent/cycle.rs`; and checked the working tree. There was no first incomplete executable item before this planning turn: Active Priorities items 13 through 63 were complete and implementation had exhausted the checklist. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.99 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.4`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and contains 1,582 planned operations. Graph operation `3531e5dc63009037`, covering `agent::cycle::{gate_id_u64, evidence_u64_value}`, is already reconciled in source by shared `lookup_u64_route(...)` delegation, so the next unreconciled safe non-`root_validate` graph candidate is graph operation `ee47b6f1c6db70eb`, covering `agent::cycle::{recovery_route_for_action, recovery_route_for_failure}`. The safe subset is private route-lookup helper delegation only: keep both private wrapper names and signatures, preserve exact action-name matching, preserve exact failure-list membership matching, preserve unknown-action and unknown-failure `None` behavior, preserve `RecoveryActionSpec` gate/target mappings, preserve recovery phase routing, and do not touch `RECOVERY_ROUTES`, `recovery_action_for_failure(...)`, `recovery_action_spec(...)`, gate/evidence/effect hash helpers, `build_submit_evidence_json(...)`, runtime cycle transitions, router/client code, kernel/TLog submission helpers, or `root_validate`.

64. [x] `src/agent/cycle.rs`: route `recovery_route_for_action(...)` and `recovery_route_for_failure(...)` through one private recovery-route lookup helper while preserving action and failure lookup semantics.
   - Scope: `src/agent/cycle.rs` only; allowed functions are `recovery_route_for_action(...)`, `recovery_route_for_failure(...)`, and at most one new private helper adjacent to the existing recovery route helpers. Do not change `RECOVERY_ROUTES`, `recovery_action_for_failure(...)`, `recovery_action_spec(...)`, `run_recovery_phase(...)`, `RecoveryActionSpec`, `RecoveryRoute`, phase routes, gate/evidence/effect route tables, hash helpers, `build_submit_evidence_json(...)`, `CycleRunner`, router/client code, kernel/TLog submission helpers, or tests in this item.
   - Done when: `recovery_route_for_action(action)` still returns the route whose `action` exactly matches `action`; `recovery_route_for_failure(failure)` still returns the route whose `failures` slice contains `failure`; both wrappers still return `None` for unknown inputs; both wrappers delegate through the same private helper boundary; and no runtime, JSON, receipt, route-table, or recovery-phase behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

65. [x] `src/agent/cycle.rs` test `recovery_route_lookup_helper_preserves_action_failure_boundaries`: add focused regression coverage for the shared recovery-route lookup helper.
   - Scope: `src/agent/cycle.rs` test module only; use existing private test access to `recovery_route_for_action(...)`, `recovery_route_for_failure(...)`, `recovery_action_for_failure(...)`, and `recovery_action_spec(...)`. Do not change production code in this item.
   - Done when: the named test asserts that direct action lookup preserves representative routes for `RecheckInvariant`, `BindReadyTask`, `Reexecute`, `RepairArtifactLineage`, `RecomputeEval`, and `Escalate`; failure lookup preserves representative failures including `InvariantBlocked`, `PlanReadyQueueEmpty`, `TaskReceiptMissing`, `ArtifactLineageBroken`, `EvalFailed`, and `RecoveryExhausted`; `recovery_action_for_failure(...)` and `recovery_action_spec(...)` continue to expose the same action, gate, evidence, and target-phase mappings; unknown action and unknown failure return `None`; and the test performs no network I/O, filesystem I/O outside normal cargo test execution, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test recovery_route_lookup_helper_preserves_action_failure_boundaries -- --test-threads=1`.

66. [x] `SCORE_REPORT.md`: after items 64 and 65 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.


67. [x] `src/capability/llm/openai.rs`: route `OpenAiJudgmentProofEvent::{expected_proof_hash, verifier_context_hash}` through one private ordered hash-fold helper while preserving distinct OpenAI proof-event hash domains.
   - Scope: `src/capability/llm/openai.rs` only; allowed functions are `OpenAiJudgmentProofEvent::expected_proof_hash(...)`, `OpenAiJudgmentProofEvent::verifier_context_hash(...)`, and at most one new private helper adjacent to them. Do not change `OpenAiJudgmentProofEvent::finalize_receipt_after_tlog(...)`, `finalize_receipt_at_seq(...)`, `is_valid(...)`, `proof_flags(...)`, `matches_receipt(...)`, `to_canonical_effect_proof(...)`, `to_canonical_verification_proof_record(...)`, `OpenAiLlmEffectReceipt`, OpenAI request serialization, HTTP/client code, retry budget logic, NDJSON helpers, Ollama code, or tests in this item.
   - Done when: `expected_proof_hash(...)` still starts from `0x4f50_454e_4149_5052u64`, mixes exactly the existing proof fields in the existing order, and returns a non-zero hash; `verifier_context_hash(...)` still starts from `0x4f50_454e_4149_4354u64`, mixes exactly the existing verifier-context fields in the existing order, and returns a non-zero hash; both public hash methods delegate through the same private helper boundary; and no public API, receipt/proof, replay, NDJSON, or network behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

68. [x] `src/lib.rs` test `openai_proof_event_hash_helpers_preserve_distinct_domains`: add focused regression coverage for the shared `OpenAiJudgmentProofEvent` proof/verifier hash helper.
   - Scope: `src/lib.rs` test module only; use existing public OpenAI LLM call, receipt, proof-event, canonical proof, state, context, policy, and TLog APIs plus existing test setup patterns. Do not change production code in this item.
   - Done when: the named test obtains a valid `OpenAiJudgmentProofEvent`, asserts `expected_proof_hash()` equals `proof_hash`, asserts `verifier_context_hash()` is non-zero and distinct from the provider proof hash, asserts `to_canonical_effect_proof(...)` binds the same verifier-context and provider proof hashes, asserts proof-only tampering changes `expected_proof_hash()` without changing `verifier_context_hash()`, and asserts verifier-context tampering changes both relevant hashes. The test must not perform network I/O, environment mutation, process spawning, or filesystem I/O outside normal cargo test execution.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test openai_proof_event_hash_helpers_preserve_distinct_domains -- --test-threads=1`.

69. [x] `SCORE_REPORT.md`: after items 67 and 68 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.

Planning reconnaissance refresh on 2026-05-15 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; confirmed there was no unchecked item under `## Active Priorities`; inspected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json`; inspected candidate source and test surfaces in `src/capability/judgment/record.rs`; and checked the working tree. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan. Existing staged and unstaged changes in `SCORE_REPORT.md`, `plan.md`, `status.md`, and `src/capability/llm/openai.rs` were treated as pre-existing execution artifacts and must not be overwritten by the next implementation turn.

Current graph-derived aggregate evidence remains `G = 7.99 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.4`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and currently contains 1,582 planned operations. Current planning selected graph operations `49c79f21e5fd37ff`, `62b32fc740a94d50`, and `c19be7a443dd01b9`, covering policy-reuse receipt hash helper candidates in `capability::judgment::record`, as the next safe non-root consolidation area. The safe subset is private ordered hash-fold helper delegation only: keep each receipt hash function name, record-type validation, enum/string code mapping, source-hash validation, field order, domain seed, and non-zero `h.max(1)` behavior unchanged. Do not touch `PolicyJudgmentRecord`, `JudgmentRecord`, policy lookup behavior, receipt constructors, JSON encoders, promotion logic, OpenAI code, Ollama code, loop-driver call sites, or `root_validate`.

70. [x] `src/capability/judgment/record.rs`: route `policy_reuse_ledger_summary_receipt_hash(...)` and `policy_reuse_scale_trace_receipt_hash(...)` through one private ordered policy-reuse receipt hash-fold helper while preserving their distinct field domains.
   - Scope: `src/capability/judgment/record.rs` only; allowed functions are `policy_reuse_ledger_summary_receipt_hash(...)`, `policy_reuse_scale_trace_receipt_hash(...)`, and at most one new private helper adjacent to the policy-reuse hash helpers. Do not change `PolicyReuseLedgerSummaryReceipt`, `PolicyReuseScaleTraceReceipt`, `PolicyReusePerformanceCostTrendReceipt`, `PolicyReuseCostCatalogReceipt`, `PolicyReuseEvaluatorSavingsReceipt`, `PolicyReuseReceipt`, `PolicyReuseTrendReceipt`, `PolicyJudgmentRecord`, JSON encoders, receipt constructors, validation methods, OpenAI code, Ollama code, or tests in this item.
   - Done when: `policy_reuse_ledger_summary_receipt_hash(...)` still rejects zero schema/source hashes, invalid record types, and `reuse_rate_bps > 10_000`; still uses seed `0x504f_4c52_4c45_4447u64`; still mixes schema version, record type code, policy hits, policy misses, LLM fallbacks, validation passes, validation failures, reuse rate, regression flag, and source receipt hash in the existing order; `policy_reuse_scale_trace_receipt_hash(...)` still rejects zero schema/source hashes, invalid record types, and `reuse_rate_bps > 10_000`; still uses seed `0x504f_4c52_5343_414cu64`; still mixes schema version, record type code, batch size, policy hits, policy misses, LLM fallbacks, validation passes, validation failures, reuse rate, regression flag, avoided LLM calls per batch, and source receipt hash in the existing order; both functions delegate through the same private ordered fold helper; and no public API, receipt, replay, JSON, or policy behavior changes.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --lib`.

71. [x] `src/capability/judgment/record.rs` test `policy_reuse_receipt_hash_helpers_preserve_distinct_record_boundaries`: add focused regression coverage for the shared policy-reuse receipt hash-fold helper.
   - Scope: `src/capability/judgment/record.rs` test module only; use existing `PolicyReuseReceipt`, `PolicyReuseLedgerSummaryReceipt`, and `PolicyReuseScaleTraceReceipt` constructors and private test access to receipt hash helpers. Do not change production code in this item.
   - Done when: the named test proves ledger-summary and scale-trace receipts remain valid, have non-zero distinct receipt hashes for the same underlying reuse summary, reject record-type tampering, reject `reuse_rate_bps > 10_000`, reject source-hash tampering, and keep scale-trace-only fields bound to the scale-trace receipt hash. The test must not perform network I/O, filesystem I/O outside normal cargo test execution, environment mutation, or process spawning.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test policy_reuse_receipt_hash_helpers_preserve_distinct_record_boundaries -- --test-threads=1`.

72. [x] `SCORE_REPORT.md`: after items 70 and 71 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
   - Scope: `SCORE_REPORT.md`, `score.md`, `plan.md`, and `status.md` only.
   - Done when: `scripts/recapture_rustc_graphs.sh --check` validates the configured graph root, `SCORE_REPORT.md` is regenerated from `../state/rustc`, `status.md` records aggregate and affected crate rows including `ai` and any changed function counts, and `score.md` changes only if refreshed evidence differs from the current rationale.
   - Validation: `bash scripts/recapture_rustc_graphs.sh --check && cargo run --manifest-path ../score/Cargo.toml --quiet -- --artifact-root ../state/rustc --report SCORE_REPORT.md --date "$(date +%Y-%m-%d)"`.

Planning reconnaissance refresh on 2026-05-15 re-read `plan.md`, `status.md`, `score.md`, and `SCORE_REPORT.md`; confirmed Active Priorities items 70 through 72 are complete in the working tree with validation evidence recorded; inspected current `../state/rustc/auto-refactor/*.graph-editor-plan.json`; inspected the selected `../state/rustc/auto-refactor/workspace__ai_sandbox__canon-mini-agent__prototype__state__rustc__ai__graph.graph-editor-plan.json`; inspected commit-hook blocker files `src/api/server.rs` and `src/capability/mod.rs`; inspected the next graph-backed candidate surface in `src/agent/router.rs`; and checked the working tree through the project connector. User direction remains explicit: do not continue `root_validate` work. `root_validate` remains the weakest graph row, but it is intentionally non-selectable for this plan.

Current graph-derived aggregate evidence remains `G = 7.99 / 10`, Architecture `8.9`, Structure `4.9`, Simplicity `7.1`, Maintainability `10.0`, Determinism `10.0`, and Coherency `8.4`; Structure remains the lowest aggregate axis. The selected `ai` graph-editor plan is schema version 1, graph schema version 16, and currently contains 1,582 planned operations. The most immediate blocker to landing validated work was commit-hook `cargo fmt --check` drift in `src/api/server.rs` and `src/capability/mod.rs`; that unblock is tracked as item 73. The next graph-backed non-root candidate is operation `f83874fb2b4b9aa3`, covering `agent::router::cdp_get`, with split boundaries `phase::parse` and `phase::transform`. The safe subset is request/response helper extraction only: preserve `cdp_get(...)` signature, target resolution behavior, TCP timeout configuration, request bytes, response parsing, return type, error mapping, and existing loopback test semantics. Do not touch browser session logic, OpenAI code, Ollama code, loop-driver scheduling, graph mutation code, or `root_validate`.

73. [x] `src/api/server.rs` and `src/capability/mod.rs`: apply the minimal rustfmt-equivalent edits required by the commit hook without changing behavior.
   - Scope: `src/api/server.rs` test formatting around `gate_from_str("UnknownGate")`; `src/capability/mod.rs` test formatting around `EvidenceSubmission::new(...)`; no semantic edits, no production behavior edits, and no other files for this item.
   - Done when: `cargo fmt --check` no longer reports diffs for `src/api/server.rs` or `src/capability/mod.rs`.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check`.

74. [x] `src/agent/router.rs`: split `cdp_get(...)` into private request/transport and response-parse phase helpers while preserving the existing public helper boundary and loopback behavior.
   - Scope: `src/agent/router.rs` only; allowed production functions are `cdp_get(...)`, `build_cdp_get_request(...)`, `parse_cdp_get_response(...)`, and at most two new private helpers adjacent to them. Do not change `close_cdp_target(...)`, `cdp_target_id_for_url(...)`, browser session orchestration, streaming request builders, OpenAI/Ollama code, loop-driver code, or tests in this item.
   - Done when: `cdp_get(...)` keeps the same signature and return type; still resolves `(host, port)` with `ToSocketAddrs`; still uses `TcpStream::connect_timeout`; still sets read and write timeouts from `timeout_ms`; still writes and flushes the exact `build_cdp_get_request(...)` output; still reads the response into a `String`; still delegates final parsing to `parse_cdp_get_response(...)`; and the graph-backed implementation exposes named private phase helpers without changing request bytes, response body extraction, status parsing, or `OpenAiError` mapping.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test cdp_get_helpers_preserve_http_request_and_response_parsing -- --test-threads=1`.

75. [x] `src/agent/router.rs` test `cdp_get_helpers_preserve_http_request_and_response_parsing`: extend the focused loopback regression to cover the new `cdp_get(...)` split boundaries without introducing network dependence beyond local loopback.
   - Scope: `src/agent/router.rs` test module only; use the existing loopback listener test and private helper access. Do not change production code in this item.
   - Done when: the named test still proves the emitted request line and headers, observed status, body extraction, and server-thread completion; it also directly covers the new request/transport or response-parse helper boundary where practical. The test must not contact external network services, mutate environment variables, spawn subprocesses, or write files.
   - Validation: `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test cdp_get_helpers_preserve_http_request_and_response_parsing -- --test-threads=1`.

76. [x] `SCORE_REPORT.md`: after items 74 and 75 land, refresh graph-derived structural evidence and review whether `score.md` rationale changes without raising project-level scores absent capability evidence.
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
