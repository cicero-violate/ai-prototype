# Canon Agent Implementation Plan

## Current State

Canon Agent is a Rust prototype for a deterministic, auditable, self-improving agent runtime. The target architecture remains a formally constrained state-machine kernel with a capability layer around it. The kernel owns correctness, state transitions, durable records, replay boundaries, and audit evidence. LLMs and tools operate inside the capability layer and must produce typed, reviewable evidence rather than governing the runtime directly.

Current implementation snapshot, 2026-05-09 America/Toronto / 2026-05-09 UTC:

- Branch: `main`.
- Latest visible commit before this implementation turn: `4783cbf Refresh planning and scoring handoff`.
- Working directory: `/workspace/ai_sandbox/canon-mini-agent/prototype/ai`.
- P0 validation baseline is complete.
- P1 validation evidence reporting is complete.
- P2 agent loop reliability is complete.
- P3 runtime and receipt correctness is complete for the current scope.
- P4 graph source-of-truth integration now includes persisted graph evidence classification, deterministic fixture-backed positive evidence for landed graph mutations with receipt snapshots, command-sequenced workflow fixture validation, a compact graph-only report mode, documented/tested root runtime, wrapper, and editor subproject boundaries, a standalone graph workflow fixture validator consumed by observe-validation, full observe-validation command evidence for that validator, explicit wrapper telemetry configuration classification, and normal-path observe-validation summary evidence for graph fixture, wrapper configuration, receipt replay classification, and missing-signal coherence.

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

### P0 — Finish validation baseline — complete

- All-target validation has final exit evidence and final `test result:` summaries from prior implementation work.
- Format, lib tests, and clippy have passed in the current baseline.
- Correctness, robustness, and determinism score ceilings may remain raised while the validation baseline is preserved.

### P1 — Make validation evidence first-class — complete

- Observe-validation reporting emits source-backed validation summary evidence under ignored `target/observe/validation-report.ndjson`.
- The compact report includes git hygiene, command outcomes, graph telemetry presence/absence, runtime archive counts, ignored artifact counts, missing-signal flags, connector-failure classification fields, receipt replay classification inventory, graph evidence state, and graph fixture report-only mode.
- Helper validators and manifest writer scripts are tracked source rather than stale generated artifacts.

### P2 — Agent loop reliability — complete

- Streaming retry behavior preserves evidence continuity across incomplete SSE/router cases.
- Retry decisions are constrained by target URL presence, missing `[DONE]`, missing `message_stream_complete`, and `finish_reason=length` classifications.
- Loop-driver mode classification distinguishes project planning, project execution, and worker certification prompts.

### P3 — Runtime and receipt correctness — complete for current scope

- Receipt-chain invariant verification covers valid replay and compact classifications for forged, duplicated, reordered, stale, and missing receipts.
- API transport consumers expose compact replay reports and expected-count missing-tail checks.
- Worker API route coverage exists for oversized batches, malformed batch payloads, tampered envelope hashes, durable resume replay, and supervisor reload replay continuity.
- Observe-validation summary evidence persists compact receipt-chain classification inventory and missing-signal state.
- Remaining optional P3 extension: add a durable runtime ledger format for failed replay attempts if future consumers require failed-replay rows beyond source-derived validation summary evidence.

### P4 — Graph source-of-truth integration — in progress

1. **Persist graph evidence classification in validation/report rows — complete**
   - Added source-derived graph evidence inventory fields to `scripts/observe_validation.sh`.
   - Added deterministic graph evidence status classifications:
     - `graph_wrapper_absent_by_configuration`
     - `graph_wrapper_configured_missing`
     - `graph_wrapper_configured_no_telemetry`
     - `graph_mutation_evidence_contract_missing`
     - `graph_mutation_evidence_emitted_not_landed`
     - `graph_mutation_landed_without_receipt_ledger`
     - `graph_mutation_landed_with_receipt_snapshot`
   - Added summary fields for graph source contract and graph workflow contract evidence files/tokens.
   - Added missing-signal flags for graph source/workflow contract report absence.
   - Added observe-validation contract coverage for the graph evidence report schema.

2. **Add deterministic positive graph evidence fixture/report path — complete**
   - Added `inspect_graph_workflow_fixture()` to validate the tracked graph workflow fixture manifest, file inventory, SHA-256 integrity rows, required workflow commands, landing command, ledger command, and receipt-snapshot evidence.
   - The observe-validation summary emits `graph_workflow_fixture_*` fields and clears `missing_graph_workflow_fixture_receipt_snapshot` when the deterministic fixture proves a landed graph mutation with receipt snapshot evidence.
   - The positive graph status `graph_mutation_landed_with_receipt_snapshot` is reachable without live wrapper telemetry when fixture evidence is complete.
   - Fixed the observe-validation `state_graph_present` assignment so graph status and missing-signal fields are defined consistently.

3. **Add compact graph fixture report-only mode — complete**
   - Added `--graph-fixture-report` to `scripts/observe_validation.sh`.
   - The report-only mode emits a single `graph_fixture_report` row and exits without running cargo validation, panic validation, policy replay validation, or wrapper telemetry.
   - The row includes `graph_fixture_report_only=true`, `graph_fixture_report_command=--graph-fixture-report`, `graph_evidence_status`, `graph_workflow_fixture_*` fields, and `missing_graph_workflow_fixture_receipt_snapshot`.
   - Current evidence: `CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-step3.ndjson python3 scripts/observe_validation.sh --graph-fixture-report` passed and emitted `graph_evidence_status=graph_mutation_landed_with_receipt_snapshot`.

4. **Verify graph mutation ops, patch receipts, snapshot contracts, and landing receipts end-to-end — complete for deterministic fixture scope**
   - Existing graph CLI contract tests cover verify-ops, generate-patch, verify-landing, verify-receipts, stable usage, workflow fixture integrity, copyable workflow fixtures, and executable manifest flow.
   - The standalone graph workflow fixture validator now requires the manifest command sequence to match verify-ops → generate-patch → verify-landing → verify-receipts.
   - The validator also requires generated patch/receipt outputs to be referenced and verifies the receipt ledger flow connects `patch-receipt.ndjson` and `mutation-receipt.ndjson`.
   - Current evidence: `cargo test --test graph_mutation_cli_contract -- --test-threads=1` passed; 10 tests; `--graph-fixture-report` emitted `graph_workflow_fixture_command_sequence_valid=true`, `graph_workflow_fixture_generated_outputs_present=true`, `graph_workflow_fixture_receipt_ledger_flow_valid=true`, and `graph_workflow_fixture_receipt_snapshot_present=true`.

5. **Clarify graph subproject boundaries — complete**
   - Added a Subproject Boundary Contract to `docs/03-graph-source-of-truth.md`.
   - The contract assigns ownership across `ai/` root runtime, `canon-rustc-v3/`, and `graph-editor/`.
   - Runtime owns graph schema constants, typed mutation operations, deterministic patch generation, receipt verification, validation/report evidence, and TLog admission rules.
   - `canon-rustc-v3/` owns compiler-wrapper capture, rustc integration, graph emission, wrapper telemetry, and wrapper validation probes.
   - `graph-editor/` owns human-facing graph inspection/editing workflows and editor-local UX.
   - Added `tests/test_graph_source_boundary_contract.py` so future boundary changes require explicit documentation and contract coverage.

6. **Add standalone graph workflow fixture validator — complete**
   - Added `scripts/validate_graph_workflow_fixture.py` to validate graph workflow fixture manifests, required files, required commands, SHA-256 integrity rows, landing/ledger commands, and receipt-snapshot evidence.
   - The validator emits a JSON `graph_fixture_report` with `graph_fixture_validator`, `graph_workflow_fixture_*` fields, `graph_evidence_status`, and missing-signal flags.
   - `scripts/observe_validation.sh` now delegates graph fixture inspection and graph-only report rows to the standalone validator while preserving the existing NDJSON report contract.
   - Added `tests/test_graph_workflow_fixture_validator.py` for positive current-fixture evidence and negative missing-fixture evidence.
   - Updated observe-validation contract coverage to treat graph fixture detail fields as validator-owned evidence consumed by observe-validation.

7. **Record graph workflow fixture validation as full observe-validation command evidence — complete**
   - Full observe-validation now runs `scripts/validate_graph_workflow_fixture.py` as the tracked `graph_workflow_fixture_validation` command.
   - The validation summary reads `target/observe/graph-workflow-fixture.json` and records `graph_workflow_fixture_validation_result`, `graph_workflow_fixture_report_path`, `graph_fixture_validator`, and `graph_workflow_fixture_*` fields.
   - The lightweight `--graph-fixture-report` path remains available and still emits a single graph-only report row without broad validation.
   - Current evidence: a forced-short-timeout full observe-validation smoke exited fail due unrelated timeout/fail status, but the summary recorded `graph_workflow_fixture_validation=pass`, `graph_workflow_fixture_validation_result=pass`, `graph_fixture_validator=scripts/validate_graph_workflow_fixture.py`, and `graph_workflow_fixture_receipt_snapshot_present=true`.

8. **Strengthen deterministic graph workflow fixture flow validation — complete**
   - Added validator fields for `graph_workflow_fixture_command_sequence_valid`, `graph_workflow_fixture_generated_outputs_present`, and `graph_workflow_fixture_receipt_ledger_flow_valid`.
   - Positive graph receipt snapshot evidence now requires command order, generated output references, and receipt ledger flow in addition to manifest integrity and required commands.
   - Direct validator, observe-validation contract, graph-only report, Rust formatting, and graph CLI contract checks passed.

9. **Classify optional wrapper telemetry configuration — complete**
   - Added `wrapper_graph_configuration_status`, `wrapper_graph_configuration_reason`, and `wrapper_graph_configuration_status_options` to the observe-validation summary.
   - Status options distinguish `not_configured`, `artifact_dir_configured_without_wrapper`, `wrapper_configured_missing`, and `wrapper_configured_available`.
   - The classification preserves optional wrapper telemetry while making absent/misconfigured wrapper state auditable separately from graph mutation fixture evidence.
   - Current forced-short-timeout observe smoke recorded `wrapper_graph_configuration_status=not_configured`, reason `CANON_RUSTC_WRAPPER and CANON_RUSTC_V3_ARTIFACT_DIR are unset`, `wrapper_graph_validation_requested=false`, `wrapper_graph_validation_available=false`, and `graph_workflow_fixture_validation=pass`.

10. **Capture normal-path observe-validation summary evidence — complete for unconfigured-wrapper environment**
   - Ran normal full observe-validation with `CANON_OBSERVE_REPORT=target/observe/full-observe-step1-normal.ndjson python3 scripts/observe_validation.sh`.
   - The connector returned 502 during the long command, but the script produced ignored exit/report artifacts.
   - Exit file recorded `1`; summary row recorded `validation_status=fail` because known unrelated missing signals remain (`missing_router_offline_tests`, runtime prior/download/conversation/performance signals, optional wrapper telemetry, and wrapper graph validation).
   - The same summary coherently recorded the P4 evidence needed for this slice:
     - `graph_workflow_fixture_validation_result=pass`
     - `graph_evidence_status=graph_mutation_landed_with_receipt_snapshot`
     - `graph_workflow_fixture_receipt_snapshot_present=true`
     - `graph_workflow_fixture_command_sequence_valid=true`
     - `graph_workflow_fixture_generated_outputs_present=true`
     - `graph_workflow_fixture_receipt_ledger_flow_valid=true`
     - `wrapper_graph_configuration_status=not_configured`
     - `wrapper_graph_configuration_reason=CANON_RUSTC_WRAPPER and CANON_RUSTC_V3_ARTIFACT_DIR are unset`
     - `wrapper_graph_validation_requested=false`
     - `wrapper_graph_validation_available=false`
     - `receipt_replay_classification_present=true`
     - `receipt_replay_classifications=[duplicated_receipt, forged_receipt, missing_receipt, reordered_receipt, stale_receipt]`
   - Focused checks also passed for observe-validation contract coverage, graph workflow fixture validator coverage, and graph fixture report-only evidence.

11. **Next P4 slice**
   - Keep wrapper telemetry optional unless `CANON_RUSTC_WRAPPER` or `CANON_RUSTC_V3_ARTIFACT_DIR` is configured.
   - If future graph implementation crosses subproject boundaries, update the boundary contract and test before changing behavior.
   - Prefer either a wrapper-configured observe-validation run when environment support exists, or a narrow executable contract that verifies configured-but-missing wrapper status without requiring live wrapper telemetry.

### P5 — Domain intelligence layer

1. Keep domain docs unwired until contracts are stable.
2. Define record contracts before behavior.
3. Keep trading separate from production business automation.

## Validation Evidence From Latest Implementation Step

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 18 passed; 0 failed
log: target/validation-logs/observe-validation-contract-p4-impl-step8.log
```

```text
python3 -m unittest tests/test_graph_workflow_fixture_validator.py
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/graph-workflow-fixture-validator-p4-impl-step8.log
```

```text
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-step8.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
exit: 0
result: validation_status=pass, graph_evidence_status=graph_mutation_landed_with_receipt_snapshot, graph_workflow_fixture_receipt_snapshot_present=true
log: target/validation-logs/graph-fixture-report-p4-impl-step8.log
```

```text
CANON_TEST_TIMEOUT_SECONDS=1 CANON_OBSERVE_REPORT=target/observe/full-observe-step8-timeout-smoke.ndjson python3 scripts/observe_validation.sh
exit: 1
result: validation_status=fail from forced short timeout/fail status; wrapper_graph_configuration_status=not_configured; wrapper_graph_configuration_reason="CANON_RUSTC_WRAPPER and CANON_RUSTC_V3_ARTIFACT_DIR are unset"; wrapper_graph_validation_requested=false; wrapper_graph_validation_available=false; graph_workflow_fixture_validation=pass
log: target/validation-logs/full-observe-step8-timeout-smoke.log
```

```text
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo fmt --check
exit: 0
log: target/validation-logs/fmt-p4-impl-step8.log
```

```text
TMPDIR="$PWD/target/test-tmp" RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test graph_mutation_cli_contract -- --test-threads=1
exit: 0
result: 10 passed; 0 failed
log: target/validation-logs/graph-mutation-cli-contract-p4-impl-step8.log
```

## Current Implementation-Turn Result

This implementation turn executed the planned normal-path P4 evidence slice. Normal full observe-validation produced ignored report/exit artifacts despite connector transport failure. The report confirmed graph fixture command evidence, wrapper configuration classification, receipt replay classification inventory, and missing-signal fields together in one summary row. Overall validation remains `fail` because unrelated known missing signals are still present, not because the P4 graph fixture or wrapper configuration evidence failed.

## Execution-Turn Handoff

P0, P1, P2, and current P3 scope are complete. P4 has source-derived graph evidence classification, deterministic fixture-backed positive landed-with-receipt-snapshot evidence with command-sequenced workflow validation, a compact graph-only report path that avoids broad cargo validation, a tested subproject boundary contract for root runtime/wrapper/editor responsibilities, a standalone graph workflow fixture validator consumed by observe-validation, full observe-validation command evidence for that validator, explicit optional wrapper telemetry configuration classification, and normal-path observe-validation summary evidence for graph fixture, wrapper configuration, receipt replay classification, and missing-signal coherence. The next implementation value is either configured-wrapper evidence when the environment supports it or a narrow executable contract for configured-but-missing wrapper classification.

## Validation Evidence From Implementation Step 1

```text
CANON_OBSERVE_REPORT=target/observe/full-observe-step1-normal.ndjson python3 scripts/observe_validation.sh
connector result: 502 transport error during long command; ignored artifacts were produced
exit file: target/validation-logs/full-observe-step1-normal.exit = 1
result: validation_status=fail from known unrelated missing signals; P4 graph/wrapper/receipt evidence coherent
report: target/observe/full-observe-step1-normal.ndjson
```

```text
normal observe summary fields:
graph_workflow_fixture_validation_result=pass
graph_evidence_status=graph_mutation_landed_with_receipt_snapshot
graph_workflow_fixture_receipt_snapshot_present=true
graph_workflow_fixture_command_sequence_valid=true
graph_workflow_fixture_generated_outputs_present=true
graph_workflow_fixture_receipt_ledger_flow_valid=true
wrapper_graph_configuration_status=not_configured
wrapper_graph_configuration_reason=CANON_RUSTC_WRAPPER and CANON_RUSTC_V3_ARTIFACT_DIR are unset
wrapper_graph_validation_requested=false
wrapper_graph_validation_available=false
receipt_replay_classification_present=true
receipt_replay_classifications=[duplicated_receipt, forged_receipt, missing_receipt, reordered_receipt, stale_receipt]
missing_signal_count=8
```

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 18 passed; 0 failed
log: target/validation-logs/observe-validation-contract-step1.log
```

```text
python3 -m unittest tests/test_graph_workflow_fixture_validator.py
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/graph-workflow-fixture-validator-step1.log
```

```text
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-step1.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
exit: 0
result: validation_status=pass, graph_evidence_status=graph_mutation_landed_with_receipt_snapshot, graph_workflow_fixture_receipt_snapshot_present=true
log: target/validation-logs/graph-fixture-report-step1.log
```

## Current Non-Goals

- Do not wire domain specs into runtime behavior yet.
- Do not redesign the kernel.
- Do not add live trading behavior.
- Do not rely on external LLM/Ollama/OpenAI availability for baseline correctness.
- Do not commit token caches, runtime archives, target outputs, validation logs, or SSE chunk logs.
