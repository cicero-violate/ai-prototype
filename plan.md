# Canon Agent Implementation Plan

## Current State

Canon Agent is a Rust prototype for a deterministic, auditable, self-improving agent runtime. The intended architecture is a formally constrained state-machine kernel with an expanding capability layer. The kernel owns correctness, transitions, durable records, and replay boundaries. LLMs and other tools operate inside the capability layer and must produce typed, reviewable evidence rather than governing the system directly.

Current implementation surfaces include:

- Kernel/state-machine modules under `src/kernel`.
- Runtime reducer, transition table, durable state, command ledger, writer, diff, verification, and recovery support under `src/runtime`.
- Capability records for observation, context, memory, LLM, judgment, planning, tooling, verification, eval, policy, learning, and orchestration.
- API protocol, routes, server, and transport receipt contracts under `src/api`.
- Loop-mode agent driver under `src/agent`, plus `agent`, `worker`, `supervisor`, graph mutation, and root validation binaries.
- Contract tests for API, transport, planning, scoring, supervisor/worker behavior, graph mutation CLI, MCP receipts, validation harness, panic surface, policy-learning traces, and helper scripts.
- Architecture/evolution/graph/Ollama/runtime/domain documentation.
- Domain intelligence docs intentionally kept outside compiled runtime behavior until record contracts stabilize.

Current planning/scoring baseline:

- Working tree was clean at the start of this planning refresh.
- This turn updates only `plan.md` and `score.md`.
- No fresh validation suite was run in this planning turn.
- The planning artifacts already matched the current roadmap; this turn keeps the implementation plan stable and records that no implementation work was attempted.
- Next execution turn should prioritize a full quota-safe validation baseline and then update `score.md` with exact evidence.

## Operating Rules For Agent Turns

1. **Planning/scoring turns**
   - Update `plan.md` and `score.md` only.
   - Do not modify implementation files.
   - Commit planning/scoring changes as a standalone commit.

2. **Execution turns**
   - Work the highest-priority incomplete item below.
   - Keep generated artifacts out of git.
   - Update `score.md` with commands run, exit status, and failure classification.
   - Commit only intentional source/docs/test changes.

3. **Commit hygiene**
   - Before editing: inspect `git status --short`.
   - Before committing: inspect `git diff -- plan.md score.md` or the exact intended file set.
   - Stage by path, never with broad `git add .`, unless the task explicitly requires staging all changed files.

## Implementation Priority

### P0 — Re-establish clean validation baseline

1. **Use quota-safe validation paths**
   - Previous validation evidence indicated `/tmp`-style temporary writes can fail with OS error 122 (`Disk quota exceeded`).
   - Use repository-local temp output for Rust validation:
     - `mkdir -p target/test-tmp`
     - `TMPDIR="$PWD/target/test-tmp"`
   - Keep `RUSTC_WRAPPER=""` and `RUSTC_WORKSPACE_WRAPPER=""` unless graph telemetry is explicitly being validated.

2. **Run baseline validation from repository root**
   - `mkdir -p target/test-tmp`
   - `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check`
   - `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib -- --test-threads=1`
   - `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`
   - `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings`

3. **Classify any failure precisely**
   - Environment failure: quota, temp path, missing service, connector truncation, or unavailable external dependency.
   - Compile failure: Rust compile error before tests run.
   - Semantic test failure: assertion or contract mismatch.
   - Lint failure: clippy warning escalated by `-D warnings`.
   - Output-limit failure: command may have run but connector output is insufficient to prove pass/fail.

4. **Record exact evidence**
   - Capture command, status, and final relevant output lines in `score.md`.
   - If connector output truncates, redirect logs under `target/` and report the exit code plus tail lines.
   - Do not raise scores from assumed results.

### P1 — Make validation evidence first-class

1. **Strengthen observe-validation reporting**
   - Ensure `scripts/observe_validation.sh` is executable.
   - Ensure it writes `target/observe/validation-report.ndjson`.
   - Include git hygiene, command outcomes, graph telemetry presence/absence, runtime archive counts, ignored artifact counts, and missing-signal flags.

2. **Keep score updates evidence-backed**
   - Raise correctness/robustness/determinism only after fresh passing validation.
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
   - Keep graph capture disabled unless `CANON_RUSTC_WRAPPER`/`RUSTC_WRAPPER` are explicitly configured.
   - Treat absence of `state/rustc/*/graph.json` as a clear validation signal, not as hidden success.

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

Run and record the validation baseline:

1. Confirm `git status --short` before editing.
2. Create `target/test-tmp`.
3. Run `cargo fmt --check`, library tests, all-target tests, and clippy using wrapper-disabled, quota-safe commands.
4. Classify any failures using the categories in P0.
5. Update `score.md` with exact command outcomes.
6. Commit only intentional validation/scoring or implementation changes.

Do not change source code before the baseline unless a validation failure identifies a specific implementation defect.

## Current Non-Goals

- Do not wire `src/domain` into runtime behavior yet.
- Do not redesign the kernel.
- Do not add live trading behavior.
- Do not rely on external LLM/Ollama/OpenAI availability for baseline correctness.
- Do not commit token caches, runtime archives, target outputs, or SSE chunk logs.
- Do not mix planning commits with unrelated implementation changes.
