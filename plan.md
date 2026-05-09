# Canon Agent Implementation Plan

## Current State

The `ai` project is a Rust prototype for a deterministic, auditable, self-improving agent runtime. The core architecture is organized around a state-machine kernel, typed capability records, durable runtime evidence, and agent-loop orchestration.

Current implementation surfaces include:

- Kernel/state-machine modules under `src/kernel`.
- Runtime reducer, transition table, durable state, command ledger, writer, diff, verification, and recovery support under `src/runtime`.
- Capability records for observation, context, memory, LLM, judgment, planning, tooling, verification, eval, policy, learning, and orchestration.
- API protocol, routes, server, and transport receipt contracts under `src/api`.
- Loop-mode agent driver under `src/agent`, plus `agent`, `worker`, `supervisor`, graph mutation, and root validation binaries.
- Contract tests for API, transport, planning, scoring, supervisor/worker behavior, graph mutation CLI, MCP receipts, validation harness, panic surface, policy-learning traces, and helper scripts.
- Architecture/evolution/graph/Ollama/runtime/domain documentation.
- Domain intelligence docs intentionally kept outside the compiled Rust API until contracts stabilize.

Current working tree contains pre-existing non-planning changes. Planning/scoring turns must not overwrite or stage those implementation edits unless explicitly instructed.

Observed dirty state before this planning refresh:

```text
 M README.md
 M USAGE.md
 M build_test.sh
 M canon-rustc-v3/src/graph.rs
 M plan.md
 M src/agent/loop_driver.rs
?? USAGE_1.md
?? txt.txt
```

This turn should stage and commit only `plan.md` and `score.md`.

## Implementation Priority

### P0 — Re-establish clean validation baseline

1. **Use quota-safe validation paths**
   - Status: library-test filesystem failures were previously isolated to `/tmp`-style write paths.
   - Finding: `/tmp` writes can return OS error 122 (`Disk quota exceeded`) even while ordinary free-space checks appear healthy.
   - Current convention: create `target/test-tmp` and run Rust checks with `TMPDIR="$PWD/target/test-tmp"`.
   - Keep `RUSTC_WRAPPER=""` and `RUSTC_WORKSPACE_WRAPPER=""` unless graph telemetry is explicitly being validated.

2. **Run and record baseline validation**
   - Required commands from repository root:
     - `mkdir -p target/test-tmp`
     - `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check`
     - `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib -- --test-threads=1`
     - `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets`
     - `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings`
   - If connector output limits interfere, capture logs to `target/` and report the exact exit status plus the final relevant failure lines.
   - Do not infer pass/fail from truncated connector output.

3. **Separate planning from implementation work**
   - Planning turns: update `plan.md` and `score.md` only; commit only those files.
   - Execute turns: work the highest-priority incomplete plan item; update `score.md` with exact evidence.
   - Before every commit: inspect `git status --short`, stage only intentional files, then commit.

### P1 — Make validation evidence first-class

1. **Strengthen observe-validation reporting**
   - Ensure `scripts/observe_validation.sh` is executable and writes `target/observe/validation-report.ndjson`.
   - Report git hygiene, validation commands, graph telemetry presence/absence, runtime archive counts, ignored artifact counts, and missing-signal flags.
   - Treat missing graph telemetry as an explicit signal when wrappers are not configured, not as a hidden pass.

2. **Keep score updates evidence-backed**
   - `score.md` must record commands run, status, failure class, and next action.
   - Scores should rise only after fresh passing evidence.
   - Scores should fall when fresh failures reveal semantic defects rather than environment-only defects.

3. **Keep generated artifacts contained**
   - Keep generated runtime artifacts ignored: `target/`, `state/`, `tlog/`, `log/`, runtime archives, SSE chunks, validation scratch output, token caches, and signed URL caches.
   - Keep source-owned docs, tests, JSON contracts, and `tests/fixtures/**/*.ndjson` visible.
   - Do not commit bulky generated artifacts or nested runtime logs.

### P2 — Agent loop reliability

1. **Harden loop-mode execution**
   - Confirm retry behavior preserves conversation context and does not replay incorrectly after a pinned tab/target URL is established.
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
   - Add or extend tests for hash-chain continuity, event ordering, schema version mismatches, and replay failure modes.
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
   - Treat absence of `state/rustc/*/graph.json` as a clear missing signal in validation reports.

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

Complete validation baseline first:

1. Preserve unrelated working-tree changes.
2. Create `target/test-tmp`.
3. Run `cargo fmt --check`, library tests, all-target tests, and clippy using the wrapper-disabled, quota-safe commands above.
4. If any command fails, classify the failure as environment, semantic test failure, compile failure, lint failure, or connector/output failure.
5. Update `score.md` with exact command outcomes and commit only intentional implementation/scoring changes.

## Current Non-Goals

- Do not wire `src/domain` into `lib.rs` yet.
- Do not redesign the kernel.
- Do not add live trading behavior.
- Do not rely on external LLM/Ollama/OpenAI availability for baseline correctness.
- Do not commit token caches, runtime archives, target outputs, or SSE chunk logs.
- Do not mix planning commits with unrelated implementation changes.
