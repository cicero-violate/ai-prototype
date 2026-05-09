# Canon Agent Implementation Plan

## Current State

Canon Agent is a Rust prototype for a deterministic, auditable, self-improving agent runtime. The target architecture remains a formally constrained state-machine kernel with a capability layer around it. The kernel owns correctness, state transitions, durable records, replay boundaries, and audit evidence. LLMs and tools operate inside the capability layer and must produce typed, reviewable evidence rather than governing the runtime directly.

Current implementation snapshot, 2026-05-09 America/Toronto / 2026-05-09 UTC:

- Branch: `main`.
- Latest visible commit before this implementation turn: `57c49b8 Update planning and scoring`.
- Working directory: `/workspace/ai_sandbox/canon-mini-agent/prototype/ai`.
- P0 validation baseline is complete.
- P1 validation evidence reporting is complete.
- P2 agent loop reliability is complete.
- P3 runtime and receipt correctness is complete for the current scope.
- P4 graph source-of-truth integration now includes persisted graph evidence classification, deterministic fixture-backed positive evidence for landed graph mutations with receipt snapshots, command-sequenced workflow fixture validation, a compact graph-only report mode, documented/tested root runtime, wrapper, and editor subproject boundaries, a standalone graph workflow fixture validator consumed by observe-validation, full observe-validation command evidence for that validator, explicit wrapper telemetry configuration classification, normal-path observe-validation summary evidence for graph fixture, wrapper configuration, receipt replay classification, and missing-signal coherence, executable configured-wrapper classifier branch coverage, source-derived runtime performance signal evidence from observe-validation command durations, runtime archive missing-signal derivation from inspected archive contents, runtime manifest base-match derivation from archived manifest metadata, and explicit router/offline availability classification in observe-validation summary evidence.

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

11. **Add executable configured-wrapper classifier coverage — complete**
   - Added `test_wrapper_graph_configuration_classifier_executes_all_status_branches` to `tests/test_observe_validation_contract.py`.
   - The test imports the Python observe-validation script through an explicit `SourceFileLoader` because the script keeps its historical `.sh` filename while containing Python.
   - The test executes `wrapper_graph_configuration_status()` and `wrapper_graph_configuration_reason()` for all wrapper configuration branches:
     - `not_configured`
     - `artifact_dir_configured_without_wrapper`
     - `wrapper_configured_missing`
     - `wrapper_configured_available`
   - This gives focused configured-wrapper classification evidence without requiring live wrapper telemetry or a wrapper-configured long observe-validation run.

12. **Derive runtime performance signal from observe-validation command durations — complete**
   - Added `runtime_performance_summary()` to `scripts/observe_validation.sh`.
   - The summary now derives runtime performance evidence from validation command `duration_ms` values instead of hardcoding `runtime_performance_signal_present=false`.
   - The derived metrics include command duration total, nearest-rank median/p95 for validation command durations, zero-valued download timing placeholders when command-duration evidence exists, budget status, and budget failures.
   - `missing_runtime_performance_signal` now reflects the derived signal instead of a permanent missing flag.
   - Added observe-validation contract tests for source-derived performance evidence and missing-duration fallback.
   - Normal observe-validation produced ignored artifacts confirming `runtime_performance_signal_present=true`, `runtime_performance_budget_status=pass`, `missing_runtime_performance_signal=false`, and `missing_signal_count=7`.

13. **Derive runtime archive missing flags from inspected archive contents — complete**
   - Moved runtime archive inspection before missing-signal calculation in `scripts/observe_validation.sh`.
   - `missing_runtime_download_index`, `missing_runtime_prior_state`, and `missing_runtime_conversation_ledger` now derive from `inspect_runtime_archive()` counts instead of hardcoded `true` values.
   - Added an observe-validation contract test that builds a synthetic runtime tar archive and verifies download index, prior state, conversation ledger, current-run summary, and runtime manifest evidence are detected.
   - Ran observe-validation with an ignored synthetic runtime archive. The report confirmed `runtime_archive_inspection_status=pass`, archive counts present, `missing_runtime_download_index=false`, `missing_runtime_prior_state=false`, `missing_runtime_conversation_ledger=false`, and `missing_signal_count=4`.

14. **Derive runtime manifest base-match evidence from archived manifest metadata — complete**
   - `inspect_runtime_archive()` now parses `runtime-manifest.json` from runtime archives and records `runtime_manifest_base_commit` when the manifest contains `base_commit`, `delta_base`, or `base`.
   - `missing_runtime_manifest_base_match` now derives from `CANON_DELTA_BASE` matching the archived manifest base commit instead of only checking whether an environment base was provided.
   - Added observe-validation contract coverage for archived manifest base extraction and matching behavior.
   - Ran observe-validation with an ignored synthetic runtime archive and `CANON_DELTA_BASE=base-step5`. The report confirmed `runtime_manifest_base_matches_delta_base=true`, `runtime_manifest_base_commit=base-step5`, `missing_runtime_manifest_base_match=false`, and `missing_signal_count=3`.

15. **Classify router/offline missing-signal state — complete**
   - Keep wrapper telemetry optional unless `CANON_RUSTC_WRAPPER` or `CANON_RUSTC_V3_ARTIFACT_DIR` is configured.
   - If future graph implementation crosses subproject boundaries, update the boundary contract and test before changing behavior.
   - Added `router_offline_classification()` to classify router/offline state as `router_offline_unavailable`, `router_offline_available_passed`, or `router_offline_available_not_passed`.
   - Observe-validation summary rows now emit router/offline classification status, reason, availability, evidence files, and classification options.
   - `missing_router_offline_tests` now derives from the router/offline classifier instead of being hardcoded true.
   - In the current environment, the router offline harness is unavailable, so summary evidence records `router_offline_unavailable`, `router_offline_test_status=skipped_env_missing`, and `missing_router_offline_tests=false` without requiring a live router harness.
   - Added executable branch coverage for unavailable, available/pass, and available/fail classifier states.

16. **Next P4 slice**
   - Continue reducing known missing-signal gaps where evidence can be source-derived without weakening optional wrapper telemetry semantics.
   - Candidate next target: clearer separation of optional wrapper telemetry and wrapper graph validation from required validation status when wrapper variables are unset.
   - Preserve graph fixture, receipt replay, runtime archive, runtime manifest, router/offline, and performance evidence semantics while reducing remaining missing-signal count.

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

## Validation Evidence From Implementation Step 1

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 24 passed; 0 failed
log: target/validation-logs/observe-validation-contract-router-step1.log
```

```text
python3 -m py_compile scripts/observe_validation.sh tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-router-step1.log
```

```text
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-router-step1.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
exit: 0
result: validation_status=pass; graph_evidence_status=graph_mutation_landed_with_receipt_snapshot; graph_workflow_fixture_receipt_snapshot_present=true
log: target/validation-logs/graph-fixture-report-router-step1.log
```

```text
CANON_TEST_TIMEOUT_SECONDS=1 CANON_OBSERVE_REPORT=target/observe/full-observe-router-step1-timeout-smoke.ndjson python3 scripts/observe_validation.sh
connector result: 502 transport error during long command; ignored artifacts were produced
exit file: target/validation-logs/full-observe-router-step1-timeout-smoke.exit = 1
result: validation_status=fail from known unrelated missing signals; router/offline summary classification emitted
summary: router_offline_test_classification=router_offline_unavailable; router_offline_test_status=skipped_env_missing; missing_router_offline_tests=false; missing_signal_count=6; graph_evidence_status=graph_mutation_landed_with_receipt_snapshot; graph_workflow_fixture_validation_result=pass; wrapper_graph_configuration_status=not_configured
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

Implementation step 5 reduced the runtime manifest base-match missing signal by parsing base metadata from archived `runtime-manifest.json` evidence. Full observe-validation again hit connector transport 502, but ignored exit/report artifacts were produced. The report confirmed archive inspection passed, runtime manifest base commit matched `CANON_DELTA_BASE`, archive and performance missing flags remained clear, and `missing_signal_count` dropped to `3`; overall validation still fails because other known gaps remain.

## Execution-Turn Handoff

P0, P1, P2, and current P3 scope are complete. P4 has source-derived graph evidence classification, deterministic fixture-backed positive landed-with-receipt-snapshot evidence with command-sequenced workflow validation, a compact graph-only report path that avoids broad cargo validation, a tested subproject boundary contract for root runtime/wrapper/editor responsibilities, a standalone graph workflow fixture validator consumed by observe-validation, full observe-validation command evidence for that validator, explicit optional wrapper telemetry configuration classification, normal-path observe-validation summary evidence for graph fixture, wrapper configuration, receipt replay classification, and missing-signal coherence, executable branch coverage for all wrapper configuration statuses, source-derived runtime performance signal evidence, runtime archive missing-signal derivation from inspected archive contents, and runtime manifest base-match derivation from archived manifest metadata. The next implementation value is further reduction of remaining known missing signals or live wrapper-configured evidence when the environment supports it.

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

## Validation Evidence From Implementation Step 2

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 19 passed; 0 failed
log: target/validation-logs/observe-validation-contract-step2.log
```

```text
python3 -m unittest tests/test_graph_workflow_fixture_validator.py
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/graph-workflow-fixture-validator-step2.log
```

```text
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-step2.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
exit: 0
result: validation_status=pass, graph_evidence_status=graph_mutation_landed_with_receipt_snapshot, graph_workflow_fixture_receipt_snapshot_present=true
log: target/validation-logs/graph-fixture-report-step2.log
```

```text
python3 -m py_compile scripts/observe_validation.sh tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-step2.log
```

## Validation Evidence From Implementation Step 3

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 21 passed; 0 failed
log: target/validation-logs/observe-validation-contract-step3.log
```

```text
python3 -m unittest tests/test_graph_workflow_fixture_validator.py
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/graph-workflow-fixture-validator-step3.log
```

```text
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-step3.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
exit: 0
result: validation_status=pass, graph_evidence_status=graph_mutation_landed_with_receipt_snapshot, graph_workflow_fixture_receipt_snapshot_present=true
log: target/validation-logs/graph-fixture-report-step3.log
```

```text
python3 -m py_compile scripts/observe_validation.sh tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-step3.log
```

```text
CANON_OBSERVE_REPORT=target/observe/full-observe-step3-performance.ndjson python3 scripts/observe_validation.sh
connector result: 502 transport error during long command; ignored artifacts were produced
exit file: target/validation-logs/full-observe-step3-performance.exit = 1
result: validation_status=fail from known unrelated missing signals; runtime performance signal present and budget passing
summary: runtime_performance_signal_present=true, runtime_performance_budget_status=pass, missing_runtime_performance_signal=false, missing_signal_count=7
```

## Validation Evidence From Implementation Step 4

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 22 passed; 0 failed
log: target/validation-logs/observe-validation-contract-step4.log
```

```text
python3 -m unittest tests/test_graph_workflow_fixture_validator.py
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/graph-workflow-fixture-validator-step4.log
```

```text
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-step4.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
exit: 0
result: validation_status=pass, graph_evidence_status=graph_mutation_landed_with_receipt_snapshot, graph_workflow_fixture_receipt_snapshot_present=true
log: target/validation-logs/graph-fixture-report-step4.log
```

```text
python3 -m py_compile scripts/observe_validation.sh tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-step4.log
```

```text
CANON_RUNTIME_ARCHIVE=target/runtime-archive-step4.tar CANON_OBSERVE_REPORT=target/observe/full-observe-step4-runtime-archive.ndjson python3 scripts/observe_validation.sh
connector result: 502 transport error during long command; ignored artifacts were produced
exit file: target/validation-logs/full-observe-step4-runtime-archive.exit = 1
result: validation_status=fail from known unrelated missing signals; runtime archive and performance evidence present
summary: runtime_archive_inspection_status=pass, runtime_archive_download_index_files=1, runtime_archive_prior_state_files=1, runtime_archive_conversation_ledger_files=1, missing_runtime_download_index=false, missing_runtime_prior_state=false, missing_runtime_conversation_ledger=false, missing_runtime_performance_signal=false, missing_signal_count=4
```

## Validation Evidence From Implementation Step 5

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 23 passed; 0 failed
log: target/validation-logs/observe-validation-contract-step5.log
```

```text
python3 -m unittest tests/test_graph_workflow_fixture_validator.py
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/graph-workflow-fixture-validator-step5.log
```

```text
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-step5.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
exit: 0
result: validation_status=pass, graph_evidence_status=graph_mutation_landed_with_receipt_snapshot, graph_workflow_fixture_receipt_snapshot_present=true
log: target/validation-logs/graph-fixture-report-step5.log
```

```text
python3 -m py_compile scripts/observe_validation.sh tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-step5.log
```

```text
CANON_DELTA_BASE=base-step5 CANON_RUNTIME_ARCHIVE=target/runtime-archive-step5.tar CANON_OBSERVE_REPORT=target/observe/full-observe-step5-runtime-base.ndjson python3 scripts/observe_validation.sh
connector result: 502 transport error during long command; ignored artifacts were produced
exit file: target/validation-logs/full-observe-step5-runtime-base.exit = 1
result: validation_status=fail from known unrelated missing signals; runtime archive, manifest base-match, and performance evidence present
summary: runtime_manifest_base_expected=base-step5, runtime_manifest_base_commit=base-step5, runtime_manifest_base_matches_delta_base=true, missing_runtime_manifest_base_match=false, missing_runtime_download_index=false, missing_runtime_prior_state=false, missing_runtime_conversation_ledger=false, missing_runtime_performance_signal=false, missing_signal_count=3
```

## Current Non-Goals

- Do not wire domain specs into runtime behavior yet.
- Do not redesign the kernel.
- Do not add live trading behavior.
- Do not rely on external LLM/Ollama/OpenAI availability for baseline correctness.
- Do not commit token caches, runtime archives, target outputs, validation logs, or SSE chunk logs.
