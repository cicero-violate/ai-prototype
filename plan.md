# Canon Agent Implementation Plan

## Current State

Canon Agent is a Rust prototype for a deterministic, auditable, self-improving agent runtime. The target architecture remains a formally constrained state-machine kernel with a capability layer around it. The kernel owns correctness, state transitions, durable records, replay boundaries, and audit evidence. LLMs and tools operate inside the capability layer and must produce typed, reviewable evidence rather than governing the runtime directly.

The current working tree contains a broad, pre-existing implementation batch plus this planning/scoring refresh. This planning turn intentionally updates only `plan.md` and `score.md`; all source, test, example, and runtime implementation changes remain unowned by this turn and should be handled by the next execution turn.

Current planning snapshot, 2026-05-08 America/Toronto / 2026-05-09T03:15:51Z UTC:

- Branch: `main`.
- Latest visible planning commit before this turn: `390769c Update Canon Agent planning and scoring`.
- `plan.md` and `score.md` had no local diff at the start of this refresh.
- Dirty implementation files are still present and must not be conflated with this planning/scoring commit.
- The next execution turn should begin with diff review, not new feature work.

Latest known validation evidence from the prior implementation step:

- `RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check` passed.
- `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib -- --test-threads=1` passed: 191 tests.
- `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo clippy --all-targets -- -D warnings` passed.
- `cargo test --all-targets` was attempted and progressed through many suites successfully, but connector/output-window failures prevented final exit-file evidence. Treat this as incomplete evidence, not a semantic product failure.

## Operating Rules For Agent Turns

1. **Planning/scoring turns**
   - Update `plan.md` and `score.md` only.
   - Do not modify implementation files.
   - Commit planning/scoring changes as a standalone commit when possible.
   - If unrelated implementation files are already dirty, stage only `plan.md` and `score.md`.

2. **Execution turns**
   - Work the highest-priority incomplete item below.
   - Keep generated artifacts out of git.
   - Update `score.md` with commands run, exit status, and failure classification.
   - Commit only intentional source/docs/test changes.

3. **Commit hygiene**
   - Before editing: inspect `git status --short`.
   - Before committing: inspect the exact intended diff.
   - Stage by explicit path, not broad `git add .` unless the turn intentionally owns all modified paths.
   - Keep generated logs, target output, runtime archives, tokens, and SSE chunks out of git.

## Implementation Priority

### P0 — Finish validation baseline

1. **Preserve current implementation batch for execution review**
   - Current dirty implementation paths should be reviewed by an execution turn, not by this planning turn.
   - Do not overwrite or revert existing implementation changes unless the execution turn explicitly owns that decision.
   - Treat the batch as likely intentional until exact diffs and tests prove otherwise.
   - Keep the current implementation batch separate from this planning/scoring commit.

2. **Remaining validation gate**
   - Rerun only:
     - `mkdir -p target/test-tmp target/validation-logs`
     - `TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets > target/validation-logs/test-all-rerun.log 2>&1; echo $? > target/validation-logs/test-all-rerun.exit`
   - If the connector disconnects, inspect the exit file and process table before rerunning.
   - Avoid duplicate all-target invocations.
   - Capture final exit status and final `test result:` lines in `score.md`.

3. **Then refresh the complete baseline**
   - After all-target tests have final evidence, rerun if necessary:
     - `cargo fmt --check`
     - `cargo test --lib -- --test-threads=1`
     - `cargo clippy --all-targets -- -D warnings`
   - Only raise correctness, robustness, and determinism from fresh, complete evidence.

4. **Failure classification rules**
   - Environment failure: quota, temp path, missing service, connector truncation, or unavailable external dependency.
   - Compile failure: Rust compile error before tests run.
   - Semantic test failure: assertion or contract mismatch.
   - Lint failure: clippy warning escalated by `-D warnings`.
   - Output-limit failure: command may have run but connector output is insufficient to prove pass/fail.

### P1 — Make validation evidence first-class

1. Strengthen observe-validation reporting under ignored `target/observe/validation-report.ndjson`.
2. Include git hygiene, command outcomes, graph telemetry presence/absence, runtime archive counts, ignored artifact counts, and missing-signal flags.
3. Keep score updates evidence-backed; raise correctness/robustness/determinism only from fresh passing gates.

### P2 — Agent loop reliability

1. Harden loop-mode retry behavior and evidence preservation.
2. Add fixtures for truncated streams, missing `[DONE]`, missing `message_stream_complete`, and length-finished output.
3. Clarify project-loop mode versus phase-driven worker mode.

### P3 — Runtime and receipt correctness

1. Expand replay and receipt invariants for forged, duplicated, reordered, stale, and missing receipts.
2. Keep policy promotion externally verified; never let the LLM approve its own candidates.
3. Tighten API/worker compatibility for batch command limits, invalid envelopes, durable resume behavior, and supervisor reload.

### P4 — Graph source-of-truth integration

1. Keep graph telemetry observable but optional unless wrapper variables are configured.
2. Verify graph mutation ops, patch receipts, snapshot contracts, and landing receipts as one end-to-end flow.
3. Clarify `canon-rustc-v3` and `graph-editor` repository/subproject boundaries.

### P5 — Domain intelligence layer

1. Keep domain docs unwired until contracts are stable.
2. Define record contracts before behavior.
3. Keep trading separate from production business automation.

## Next Execute-Turn Recommendation

1. Confirm `git status --short` and identify unrelated dirty files.
2. Inspect the current implementation diff before changing source.
3. Rerun only the remaining all-target validation gate with wrapper-disabled, quota-safe settings.
4. Record exact exit status and final output lines in `score.md`.
5. If the all-target command passes, commit the reviewed implementation batch separately from planning/scoring.
6. Move to P1 only after all-target tests have final pass/fail evidence.

## Planning-Turn Handoff

This turn does not change implementation priority or score ceilings. The correct next move is still validation closure, not additional architecture expansion. Treat any existing source/test/example modifications as an execution-turn batch requiring review, validation, scoring, and a separate commit.

## Current Non-Goals

- Do not wire domain specs into runtime behavior yet.
- Do not redesign the kernel.
- Do not add live trading behavior.
- Do not rely on external LLM/Ollama/OpenAI availability for baseline correctness.
- Do not commit token caches, runtime archives, target outputs, or SSE chunk logs.
