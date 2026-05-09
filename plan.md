# Canon Agent Implementation Plan

## Current State

Canon Agent is a Rust prototype for a deterministic, auditable, self-improving agent runtime. The target architecture is a formally constrained state-machine kernel with a capability layer around it. The kernel owns correctness, state transitions, durable records, replay boundaries, and audit evidence. LLMs and tools operate inside the capability layer and must produce typed, reviewable evidence rather than governing the runtime directly.

Current implementation surfaces include:

- Kernel/state-machine, runtime, durable-state, transition, command-ledger, writer, diff, verification, recovery, and validation support.
- Capability records for observation, context, memory, LLM, judgment, planning, tooling, verification, eval, policy, learning, and orchestration.
- API protocol, routes, server, and transport receipt contracts.
- Loop-mode agent driver plus agent, worker, supervisor, graph mutation, and root validation binaries.
- Contract tests across API, transport, planning, scoring, supervisor/worker behavior, graph mutation CLI, MCP receipts, validation harness, panic surface, policy-learning traces, and helper scripts.
- Architecture/evolution/graph/Ollama/runtime/domain documentation.
- Domain intelligence docs intentionally kept outside compiled runtime behavior until record contracts stabilize.

Current planning/scoring baseline:

- Working tree is not clean at this planning turn. There are pre-existing unstaged implementation changes across API transport, capability records/providers, verification, graph mutation, library exports, runtime verification, scoring code, validation harness, and validation-harness tests.
- This planning turn must not touch, stage, or commit those implementation changes.
- This turn updates only `plan.md` and `score.md`.
- No validation suite is run during this planning turn.
- Scores remain capped until an execution turn captures fresh validation evidence for the current implementation tree.
- The next execution turn should first inspect the existing implementation diff, decide whether it is intentional/incomplete, then run quota-safe validation.

## Operating Rules For Agent Turns

1. **Planning/scoring turns**
   - Update `plan.md` and `score.md` only.
   - Do not modify implementation files.
   - Commit planning/scoring changes as a standalone commit.
   - Stage by explicit path: `git add plan.md score.md`.

2. **Execution turns**
   - Work the highest-priority incomplete item below.
   - Keep generated artifacts out of git.
   - Update `score.md` with commands run, exit status, and failure classification.
   - Commit only intentional source/docs/test changes.

3. **Commit hygiene**
   - Before editing: inspect `git status --short`.
   - Before committing: inspect the exact intended diff.
   - Stage by explicit path, not by broad `git add .`.
   - Do not stage pre-existing unrelated dirty files.
   - If implementation files are already dirty, preserve them unless the turn is explicitly scoped to resolve them.

## Implementation Priority

### P0 — Re-establish implementation and validation baseline

1. **Resolve dirty implementation state**
   - Inspect the current implementation diff before any new implementation work:
     - `src/api/transport.rs`
     - `src/capability/judgment/record.rs`
     - `src/capability/llm/ollama.rs`
     - `src/capability/llm/openai.rs`
     - `src/capability/llm/record.rs`
     - `src/capability/tooling/record/artifact.rs`
     - `src/capability/verification/proof.rs`
     - `src/graph_mutation.rs`
     - `src/lib.rs`
     - `src/runtime/verify.rs`
     - `src/score.rs`
     - `src/validation_harness.rs`
     - `tests/validation_harness_contract.rs`
   - Decide whether these changes are a coherent prior implementation batch, partial work, or should be split/reverted.
   - Record the decision in `score.md` on the execution turn.

2. **Use quota-safe validation paths**
   - Prior evidence indicated temp writes can fail with OS error 122 (`Disk quota exceeded`).
   - Use repository-local temp output:
     - `mkdir -p target/test-tmp`
     - `TMPDIR="$PWD/target/test-tmp"`
   - Keep `RUSTC_WRAPPER=""` and `RUSTC_WORKSPACE_WRAPPER=""` unless graph telemetry is explicitly under test.

3. **Run baseline validation from repository root**
   - `mkdir -p target/test-tmp`
   - `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check`
   - `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib -- --test-threads=1`
   - `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`
   - `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings`

4. **Classify failures precisely**
   - Environment failure: quota, temp path, missing service, connector truncation, or unavailable external dependency.
   - Compile failure: Rust compile error before tests run.
   - Semantic test failure: assertion or contract mismatch.
   - Lint failure: clippy warning escalated by `-D warnings`.
   - Output-limit failure: command may have run but connector output is insufficient to prove pass/fail.

5. **Record exact evidence**
   - Capture command, status, and final relevant output lines in `score.md`.
   - If connector output truncates, redirect logs under `target/` and report the exit code plus tail lines.
   - Do not raise scores from assumed results.

### P1 — Make validation evidence first-class

1. **Strengthen observe-validation reporting**
   - Ensure `scripts/observe_validation.sh` is executable if present or add it deliberately if missing.
   - Ensure validation reports land under ignored `target/observe/validation-report.ndjson`.
   - Include git hygiene, command outcomes, graph telemetry presence/absence, runtime archive counts, ignored artifact counts, and missing-signal flags.

2. **Keep score updates evidence-backed**
   - Raise correctness, robustness, and determinism only after fresh passing validation.
   - Lower relevant dimensions when failures expose semantic defects.
   - Keep environment-only failures separate from product correctness failures.

3. **Keep generated artifacts contained**
   - Keep ignored: `target/`, `state/`, `tlog/`, `log/`, runtime archives, SSE chunks, validation scratch output, token caches, and signed URL caches.
   - Keep visible: source-owned docs, tests, JSON contracts, and `tests/fixtures/**/*.ndjson`.

### P2 — Agent loop reliability

1. **Harden loop-mode execution**
   - Confirm retry behavior preserves conversation context.
   - Confirm retries do not replay incorrectly after a pinned tab or target URL is established.
   - Add fixtures for truncated streams, missing `[DONE]`, missing `message_stream_complete`, and length-finished output.
   - Ensure SSE chunk logs diagnose router/browser failures without persisting sensitive tokens.

2. **Clarify agent/worker/supervisor boundaries**
   - Document when `agent` uses project-loop mode versus phase-driven worker mode.
   - Add coverage for missing `GOAL.md`, unreachable MCP connector, unreachable router-server, malformed worker responses, and supervisor reload behavior.

3. **Make coordination files authoritative**
   - `plan.md` is the prioritized implementation roadmap.
   - `score.md` is the evidence-backed progress snapshot.
   - Avoid parallel status sources unless generated from or explicitly linked to these files.

### P3 — Runtime and receipt correctness

1. **Expand replay and receipt invariants**
   - Add or extend tests for hash-chain continuity, event ordering, schema-version mismatches, and replay failure modes.
   - Confirm every external effect has a typed receipt and replay verifier.
   - Add negative tests for forged, duplicated, reordered, stale, and missing receipts.

2. **Consolidate policy-learning contracts**
   - Keep policy promotion externally verified.
   - Never let the LLM approve its own candidates.
   - Add fixtures showing candidate proposal, sandbox execution, external evaluator score, distillation export, and policy-store insertion.
   - Distinguish reusable policy from one-off retrieval examples.

3. **Tighten API/worker compatibility**
   - Keep transport schemas versioned.
   - Add compatibility tests for batch command limits, invalid command envelopes, durable resume behavior, and supervisor reload.

### P4 — Graph source-of-truth integration

1. **Make graph telemetry observable but optional**
   - Keep graph capture disabled unless wrapper variables are explicitly configured.
   - Treat absence of `state/rustc/*/graph.json` as a clear validation signal, not hidden success.

2. **Unify graph mutation evidence**
   - Verify graph mutation ops, graph patch receipts, snapshot contracts, and mutation landing receipts as one end-to-end flow.
   - Add contract examples for graph changes that should be rejected.

3. **Decide subproject boundaries**
   - Clarify whether `canon-rustc-v3` and `graph-editor` are vendored subprojects, submodules, or future external dependencies.
   - Align their plans/scores with the root project only if they remain inside this repository.

### P5 — Domain intelligence layer

1. **Keep domain docs unwired until contracts are stable**
   - `src/domain` should remain documentation/specification-only unless a specific implementation turn promotes records into Rust types.
   - Do not leak business, finance, or trading semantics into kernel/runtime.

2. **Define domain record contracts before behavior**
   - Prioritize schemas for `DomainSignal`, `DomainContext`, `DomainJudgment`, `DomainPlan`, `DomainRiskEnvelope`, `DomainEval`, and `DomainPromotionCandidate`.
   - Include provenance, uncertainty, source quality, stale-signal handling, and policy boundaries.

3. **Use trading only as sandbox research**
   - Keep trading separate from production business automation.
   - Require explicit risk and simulation contracts before any execution-oriented behavior.

## Next Execute-Turn Recommendation

1. Confirm `git status --short`.
2. Inspect the full current implementation diff.
3. Decide whether the dirty implementation files are intentional, partial, or should be split/reverted.
4. Create `target/test-tmp`.
5. Run the P0 validation commands with wrappers disabled and quota-safe `TMPDIR`.
6. Classify failures using P0 categories.
7. Update `score.md` with exact command outcomes.
8. Commit only intentional validation/scoring or implementation changes.

Do not add new source behavior before the baseline unless a validation failure identifies a specific implementation defect or the existing dirty state is intentionally resolved.

## Current Non-Goals

- Do not wire domain specs into runtime behavior yet.
- Do not redesign the kernel.
- Do not add live trading behavior.
- Do not rely on external LLM/Ollama/OpenAI availability for baseline correctness.
- Do not commit token caches, runtime archives, target outputs, or SSE chunk logs.
- Do not mix planning commits with unrelated implementation changes.
