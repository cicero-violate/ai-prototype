# Canon Agent Implementation Plan

## Current State

The `ai` project is a Rust prototype for a deterministic, auditable agent runtime. It already contains:

- A frozen-ish kernel/state-machine surface under `src/kernel`.
- Runtime reducer, replay, durable state, command ledger, verification, and recovery support under `src/runtime`.
- Capability records for observation, context, memory, LLM, judgment, planning, tooling, verification, eval, policy, learning, and orchestration.
- API protocol, server routes, and transport receipts.
- `agent`, `worker`, and `supervisor` binaries.
- Loop-mode agent behavior that reads a project `GOAL.md`, drives planning/execute turns through router-server, logs SSE chunks, and coordinates through `plan.md`/`score.md`.
- Contract tests for API, transport, planning, scoring, supervisor/worker binaries, graph mutation, MCP receipts, validation harness, panic surface, policy learning traces, and helper scripts.
- Documentation for architecture, evolution loop, graph source of truth, Ollama judgment, runtime usage, and domain strategy.
- Domain intelligence docs intentionally kept outside `lib.rs` until contracts stabilize.

The repo also contains embedded/subproject material (`canon-rustc-v3`, `graph-editor`, runtime/download/log artifacts). Future implementation should avoid accidentally mixing generated artifacts, temporary logs, or nested project state into commits.

## Implementation Priority

### P0 — Stabilize repository and validation baseline

1. **Establish clean commit hygiene**
   - Keep root `ai/plan.md` and `ai/score.md` as the coordination source for agent loops.
   - Audit `.gitignore` coverage for runtime logs, target outputs, SSE chunks, token caches, signed URL caches, archives, and generated bundles.
   - Prevent accidental commits of credentials or transient runtime state.

2. **Run and document baseline checks**
   - Preferred root checks:
     - `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check`
     - `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`
     - `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings`
   - If the local toolchain or wrapper prevents execution, record the exact blocker in `score.md` rather than treating it as pass/fail ambiguity.

3. **Make validation evidence explicit**
   - Ensure `scripts/observe_validation.sh` is present, executable, and produces a useful `target/observe/validation-report.ndjson`.
   - Ensure the observe report covers: git hygiene, API/worker/supervisor tests, Rust command results, graph telemetry presence, runtime archive counts, and missing-signal flags.

### P1 — Agent loop reliability

1. **Harden loop-mode turn execution**
   - Confirm retry behavior preserves conversation context and does not replay after a pinned tab/target URL is established.
   - Add tests or fixtures for truncated streams, missing `[DONE]`, missing `message_stream_complete`, and length-finished output.
   - Ensure SSE chunk logs are enough to diagnose router/browser failures without storing sensitive tokens.

2. **Improve coordination file discipline**
   - Keep planning turns restricted to `plan.md` and `score.md` unless explicitly requested.
   - Keep execute turns aligned to the highest-priority incomplete item in this plan.
   - Require each execute turn to update `score.md` with checks run, pass/fail state, and next action.

3. **Clarify single-cycle vs loop-mode boundaries**
   - Document when `agent` uses project-loop mode versus phase-driven worker mode.
   - Add coverage for missing `GOAL.md`, unreachable MCP connector, unreachable router-server, and malformed worker responses.

### P2 — Runtime and receipt correctness

1. **Strengthen replay and receipt invariants**
   - Expand tests for hash-chain continuity, event ordering, schema version mismatches, and receipt replay failure modes.
   - Confirm every external effect has a typed receipt and replay verifier.
   - Add negative tests for forged, duplicated, reordered, and missing receipts.

2. **Consolidate policy learning contracts**
   - Keep policy promotion externally verified; never let the LLM approve itself.
   - Add fixtures showing candidate proposal, sandbox execution, external evaluator score, distillation export, and policy-store insertion.
   - Distinguish reusable policy from one-off retrieval examples.

3. **Tighten API/worker compatibility**
   - Keep transport schemas versioned.
   - Add compatibility tests for batch command limits, invalid command envelopes, durable resume behavior, and supervisor reload.

### P3 — Graph source-of-truth integration

1. **Make graph telemetry observable but optional**
   - Keep graph capture disabled unless `CANON_RUSTC_WRAPPER`/`RUSTC_WRAPPER` are explicitly configured.
   - Make absence of `state/rustc/*/graph.json` a clear missing signal, not a silent failure.

2. **Unify graph mutation evidence**
   - Verify graph mutation ops, graph patch receipts, snapshot contracts, and mutation landing receipts as one end-to-end flow.
   - Add contract examples for graph changes that should be rejected.

3. **Decide subproject boundary**
   - Clarify whether `canon-rustc-v3` and `graph-editor` are vendored subprojects, submodules, or future external dependencies.
   - Align their plans/scores with the root project only if they remain inside this repo.

### P4 — Domain intelligence layer

1. **Keep domain docs unwired until contracts are stable**
   - `src/domain` should remain documentation/specification-only unless a specific implementation turn promotes records into Rust types.
   - Do not leak business/finance/trading semantics into kernel or runtime.

2. **Define domain record contracts before behavior**
   - Prioritize `DomainSignal`, `DomainContext`, `DomainJudgment`, `DomainPlan`, `DomainRiskEnvelope`, `DomainEval`, and `DomainPromotionCandidate` schemas.
   - Include provenance, uncertainty, source quality, stale-signal handling, and policy boundaries.

3. **Use trading only as sandbox research**
   - Keep trading separate from production business automation.
   - Require explicit risk and simulation contracts before any execution-oriented behavior.

## Next Execute-Turn Recommendation

Run validation baseline and tighten repository hygiene:

1. Inspect `ai/.gitignore` and root status to identify generated/transient files.
2. Run formatting/tests/clippy with Rust wrappers disabled.
3. Update `score.md` with exact command outcomes.
4. If checks fail, fix the smallest deterministic issue first.
5. Commit only intentional source/doc/test changes.

## Non-Goals For The Next Execute Turn

- Do not wire `src/domain` into `lib.rs` yet.
- Do not redesign the kernel.
- Do not add live trading behavior.
- Do not rely on external LLM/Ollama/OpenAI availability for baseline correctness.
- Do not commit token caches, runtime archives, target outputs, or SSE chunk logs.
