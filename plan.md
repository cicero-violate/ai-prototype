# Canon Agent Implementation Plan

## Current State

The `ai` project is a Rust prototype for a deterministic, auditable, self-improving agent runtime. The core architecture is already broad and mostly organized:

- Kernel/state-machine surface under `src/kernel`.
- Runtime reducer, transition table, durable state, command ledger, writer, diff, verification, and recovery support under `src/runtime`.
- Capability records for observation, context, memory, LLM, judgment, planning, tooling, verification, eval, policy, learning, and orchestration.
- API protocol, server routes, and transport receipt contracts under `src/api`.
- Loop-mode agent driver under `src/agent`, plus `agent`, `worker`, `supervisor`, graph mutation, and root validation binaries.
- Contract tests for API, transport, planning, scoring, supervisor/worker behavior, graph mutation CLI, MCP receipts, validation harness, panic surface, policy-learning traces, and helper scripts.
- Documentation for architecture, evolution loop, graph source-of-truth, Ollama judgment, runtime usage, and domain strategy.
- Domain intelligence docs intentionally kept outside the compiled Rust API until contracts stabilize.

Recent repository state shows existing non-planning implementation changes outside this turn:

- Modified: `USAGE.md`
- Modified: `canon-rustc-v3/src/graph.rs`

Planning/scoring turns should avoid overwriting or mixing those implementation edits unless explicitly instructed.

## Implementation Priority

### P0 — Restore clean validation baseline

1. **Resolve filesystem/quota write failures**
   - Investigate why library tests still hit OS error 122 (`Disk quota exceeded`) even after generated artifacts were removed and normal free-space checks appeared healthy.
   - Determine whether failures come from user/project quota, too many small files, test temp paths, runtime archive paths, or nested project artifacts.
   - Prefer redirecting test temp/runtime writes to a known writable, cleaned directory if the problem is environmental rather than semantic.

2. **Run baseline validation with wrappers disabled**
   - Required commands from `ai/`:
     - `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check`
     - `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib -- --test-threads=1`
     - `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`
     - `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings`
   - Record exact pass/fail output in `score.md`; do not infer success from partial connector output.

3. **Keep repository hygiene tight**
   - Keep generated runtime artifacts ignored: `target/`, `state/`, `tlog/`, `log/`, runtime archives, SSE chunks, validation scratch output, token caches, and signed URL caches.
   - Keep source-owned docs, tests, JSON contracts, and `tests/fixtures/**/*.ndjson` visible.
   - Do not commit bulky generated artifacts or nested runtime logs.

### P1 — Make validation evidence first-class

1. **Strengthen observe-validation reporting**
   - Ensure `scripts/observe_validation.sh` is executable and produces `target/observe/validation-report.ndjson`.
   - Report git hygiene, validation command results, graph telemetry presence/absence, runtime archive counts, ignored artifact counts, and missing-signal flags.
   - Keep missing graph telemetry as an explicit signal, not a silent pass.

2. **Improve score update discipline**
   - Every execute turn should update `score.md` with commands run, final status, failure class, and next action.
   - Planning turns should update `plan.md` and `score.md` only, then commit those two files.
   - Execute turns should align to the highest-priority incomplete item in this plan.

3. **Protect concurrent work**
   - Before committing, inspect `git status --short`.
   - Stage only files intentionally modified in the current turn.
   - Do not amend or commit unrelated implementation files left by another turn unless explicitly requested.

### P2 — Agent loop reliability

1. **Harden loop-mode execution**
   - Confirm retry behavior preserves conversation context and does not replay incorrectly after a pinned tab/target URL is established.
   - Add fixtures for truncated streams, missing `[DONE]`, missing `message_stream_complete`, and length-finished output.
   - Ensure SSE chunk logs diagnose router/browser failures without persisting sensitive tokens.

2. **Clarify agent/worker/supervisor boundaries**
   - Document when `agent` uses project-loop mode versus phase-driven worker mode.
   - Add coverage for missing `GOAL.md`, unreachable MCP connector, unreachable router-server, malformed worker responses, and supervisor reload behavior.

3. **Keep coordination files authoritative**
   - `plan.md` is the current prioritized implementation roadmap.
   - `score.md` is the current evidence-backed progress snapshot.
   - Avoid parallel status sources unless they are generated from these files or explicitly linked.

### P3 — Runtime and receipt correctness

1. **Expand replay and receipt invariants**
   - Add/extend tests for hash-chain continuity, event ordering, schema version mismatches, and replay failure modes.
   - Confirm every external effect has a typed receipt and replay verifier.
   - Add negative tests for forged, duplicated, reordered, stale, and missing receipts.

2. **Consolidate policy-learning contracts**
   - Keep policy promotion externally verified; never let the LLM approve itself.
   - Add fixtures showing candidate proposal, sandbox execution, external evaluator score, distillation export, and policy-store insertion.
   - Distinguish reusable policy from one-off retrieval examples.

3. **Tighten API/worker compatibility**
   - Keep transport schemas versioned.
   - Add compatibility tests for batch command limits, invalid command envelopes, durable resume behavior, and supervisor reload.

### P4 — Graph source-of-truth integration

1. **Make graph telemetry observable but optional**
   - Keep graph capture disabled unless `CANON_RUSTC_WRAPPER`/`RUSTC_WRAPPER` are explicitly configured.
   - Treat absence of `state/rustc/*/graph.json` as a clear missing signal.

2. **Unify graph mutation evidence**
   - Verify graph mutation ops, graph patch receipts, snapshot contracts, and mutation landing receipts as one end-to-end flow.
   - Add contract examples for graph changes that should be rejected.

3. **Decide subproject boundaries**
   - Clarify whether `canon-rustc-v3` and `graph-editor` are vendored subprojects, submodules, or future external dependencies.
   - Align their plans/scores with the root project only if they remain inside this repo.

### P5 — Domain intelligence layer

1. **Keep domain docs unwired until contracts are stable**
   - `src/domain` should remain documentation/specification-only unless a specific implementation turn promotes records into Rust types.
   - Do not leak business/finance/trading semantics into kernel or runtime.

2. **Define domain record contracts before behavior**
   - Prioritize schemas for `DomainSignal`, `DomainContext`, `DomainJudgment`, `DomainPlan`, `DomainRiskEnvelope`, `DomainEval`, and `DomainPromotionCandidate`.
   - Include provenance, uncertainty, source quality, stale-signal handling, and policy boundaries.

3. **Use trading only as sandbox research**
   - Keep trading separate from production business automation.
   - Require explicit risk and simulation contracts before any execution-oriented behavior.

## Next Execute-Turn Recommendation

Focus on quota-safe validation and the baseline test result:

1. Inspect failed write paths from the latest `cargo test --lib -- --test-threads=1` run.
2. Check project/user quota indicators beyond `df`, including inode exhaustion and per-user/project limits if available.
3. Remove or relocate any unnecessary generated runtime files still under the repository.
4. Configure tests to write temp/runtime artifacts to a cleaned, known-writable path when appropriate.
5. Re-run:
   - `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib -- --test-threads=1`
6. If library tests pass, continue with:
   - `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`
   - `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings`
7. Update `score.md` with exact command outcomes and commit only intentional changes.

## Current Non-Goals

- Do not wire `src/domain` into `lib.rs` yet.
- Do not redesign the kernel.
- Do not add live trading behavior.
- Do not rely on external LLM/Ollama/OpenAI availability for baseline correctness.
- Do not commit token caches, runtime archives, target outputs, or SSE chunk logs.
- Do not mix planning commits with unrelated implementation changes.
