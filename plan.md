# Canon Agent Plan

Current date: 2026-05-09.
Workspace: `/workspace/ai_sandbox/canon-mini-agent/prototype/ai`.

## Goal

Canon Agent is a deterministic, auditable, self-improving agent runtime. The kernel owns correctness, state transitions, replay, receipts, and durable evidence. Capabilities own observation, context, tools, LLM calls, planning, judgment, verification, evaluation, learning, graph editing, and policy promotion. The LLM proposes and explains; it does not govern the state machine or approve itself.

The near-term goal is to finish the evidence-backed mutation and learning path:

```text
objective
  -> domain/capability interpretation
  -> plan
  -> typed tool or graph operation
  -> sandbox execution
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
- P5 domain intelligence layer: planned, documentation-only today.

## Active Priorities

1. Close P4 by treating graph editing as an actual mutation path, not only validation/report evidence.
2. Start P5 by stabilizing domain record contracts before wiring behavior.
3. Add an explicit self-modification lane inside verified evolution.
4. Use Python to inspect `state/rustc/ai/graph.json` before designing graph edits.
5. Keep business automation first, finance intelligence second, and trading sandbox-only.

## P4 - Graph Editing

Graph editing is the receipt-backed source mutation path.

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

## P5 - Domain Intelligence

The domain layer remains unwired until contracts are stable. It describes interpretation, strategy, scoring, policy intent, and domain risk. It must not mutate `State`, `Packet`, `GateSet`, runtime events, or TLog directly.

Record families to stabilize first:

- `DomainSignal`
- `DomainContext`
- `DomainJudgment`
- `DomainPlan`
- `DomainRiskEnvelope`
- `DomainEval`
- `DomainPromotionCandidate`

Capability bridge:

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

Next work:

- Define schema versions and deterministic hash identity for domain records.
- Add fixtures before Rust behavior.
- Define source quality, uncertainty, contradiction, actionability, and risk scoring.
- Add tests proving domain records cannot bypass the capability registry.
- Keep trading execution blocked unless a future sandbox policy explicitly allows it.

## Self-Modification

Self-modification belongs inside verified evolution, after capability planning and before policy promotion. It is not a kernel feature and not a domain shortcut.

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
- Do not wire `src/domain` into `lib.rs` until record contracts are stable.
- Do not add live trading execution.
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

For graph-editing work:

```bash
python3 scripts/analyze_graph_json.py state/rustc/ai/graph.json
TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test graph_mutation_cli_contract -- --test-threads=1
python3 -m unittest tests/test_graph_workflow_fixture_validator.py
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
```

For domain work:

```bash
python3 -m unittest tests/test_observe_validation_contract.py
TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --all-targets
```
