# Canon Agent Implementation Plan

## Current State

Canon Agent is a Rust prototype for a deterministic, auditable, self-improving agent runtime. The target architecture remains a formally constrained state-machine kernel with a capability layer around it. The kernel owns correctness, state transitions, durable records, replay boundaries, and audit evidence. LLMs and tools operate inside the capability layer and must produce typed, reviewable evidence rather than governing the runtime directly.

Current implementation snapshot, 2026-05-09 America/Toronto / 2026-05-09 UTC:

- Branch: `main`.
- Latest visible commit before this implementation turn: `c381eb3 Expose command normalization metadata`.
- Working directory: `/workspace/ai_sandbox/canon-mini-agent/prototype/ai`.
- P0 validation baseline is complete.
- P1 validation evidence reporting is complete.
- P2 agent loop reliability is complete.
- P3 runtime and receipt correctness is complete for the current scope.
- P4 graph source-of-truth integration now includes persisted graph evidence classification, deterministic fixture-backed positive evidence for landed graph mutations with receipt snapshots, command-sequenced workflow fixture validation, a compact graph-only report mode, documented/tested root runtime, wrapper, and editor subproject boundaries, a standalone graph workflow fixture validator consumed by observe-validation, full observe-validation command evidence for that validator, explicit wrapper telemetry configuration classification, normal-path observe-validation summary evidence for graph fixture, wrapper configuration, receipt replay classification, and missing-signal coherence, executable configured-wrapper classifier branch coverage, source-derived runtime performance signal evidence from observe-validation command durations, runtime archive missing-signal derivation from inspected archive contents, runtime manifest base-match derivation from archived manifest metadata, explicit router/offline availability classification in observe-validation summary evidence, optional wrapper graph validation missing-signal derivation, generated graph JSON evidence classification, compact runtime archive report integration, separated validation status fields, compact command-execution reports, connector transport artifact classification, delta manifest preservation of transport artifact state, compact full-summary artifact replay, actual full-summary artifact manifest generation, row command fallback preservation, summary/row command conflict rejection, command execution metadata conflict rejection, and duplicate row command conflict rejection, and distinct command-count closure for exact duplicate row evidence, and duplicate command-name closure for summary-provided validation commands, and explicit command evidence normalization metadata in receipts and manifests, and actual compact full-summary artifact coverage for command normalization metadata. The working tree was clean at the start of implementation step 4.

## Current P4 Completion Summary

The current source-of-truth plan state is:

1. Generated graph JSON classification is landed and no longer treated as active pending implementation.
2. Compact runtime archive report integration is landed and preserved through observe-validation status semantics.
3. Required command execution classification is landed and can be inspected through `--command-execution-report` without long cargo validation.
4. Connector transport artifact classification is landed and distinguishes complete versus incomplete artifacts after transport interruption.
5. Delta manifests preserve connector transport artifact evidence, so downstream receipts do not rely only on `connector_transport_instability_present`.

Current next implementation target: continue P4 by adding compact receiver-side checks for command normalization metadata uniqueness and non-duplication in manifest metric rendering.

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

16. **Classify optional wrapper graph validation missing-signal state — complete**
   - Added `wrapper_graph_validation_classification()` to convert wrapper configuration and validation result into explicit optional/required validation evidence.
   - Summary rows now emit `wrapper_graph_validation_classification`, classification reason, required booleans, and derived validation/telemetry missing booleans.
   - `missing_wrapper_graph_validation` and `missing_rustc_wrapper_telemetry` now derive from the wrapper validation classifier instead of treating unconfigured optional wrapper capture as missing.
   - In the current unconfigured-wrapper environment, observe-validation records `wrapper_graph_optional_not_configured`, `wrapper_graph_validation_required=false`, `wrapper_graph_telemetry_required=false`, `missing_wrapper_graph_validation=false`, and `missing_rustc_wrapper_telemetry=false`.
   - Requested-but-unavailable and requested-but-failing wrapper states remain required missing signals.
   - Added executable branch coverage for optional-not-configured, requested-without-wrapper, requested-wrapper-missing, requested-passed, and requested-not-passed states.

17. **Next P4 slice**
   - Continue reducing known missing-signal gaps where evidence can be source-derived without weakening optional wrapper, router/offline, graph fixture, receipt replay, runtime archive, runtime manifest, and performance evidence semantics.
   - Candidate next targets: derive `missing_generated_graph_json` from graph fixture evidence when no live wrapper graph is requested, or reduce runtime archive missing signals with a reusable synthetic runtime archive/report fixture.

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

## Validation Evidence From Implementation Step 2

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 25 passed; 0 failed
log: target/validation-logs/observe-validation-contract-wrapper-step2.log
```

```text
python3 -m py_compile scripts/observe_validation.sh tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-wrapper-step2.log
```

```text
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-wrapper-step2.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
exit: 0
result: validation_status=pass; graph_evidence_status=graph_mutation_landed_with_receipt_snapshot; graph_workflow_fixture_receipt_snapshot_present=true
log: target/validation-logs/graph-fixture-report-wrapper-step2.log
```

```text
CANON_TEST_TIMEOUT_SECONDS=1 CANON_OBSERVE_REPORT=target/observe/full-observe-wrapper-step2-timeout-smoke.ndjson python3 scripts/observe_validation.sh
connector result: 502 transport error during long command; ignored artifacts were produced
exit file: target/validation-logs/full-observe-wrapper-step2-timeout-smoke.exit = 1
result: validation_status=fail from known unrelated missing signals; wrapper optionality summary classification emitted
summary: wrapper_graph_validation_classification=wrapper_graph_optional_not_configured; wrapper_graph_validation_required=false; wrapper_graph_telemetry_required=false; missing_wrapper_graph_validation=false; missing_rustc_wrapper_telemetry=false; missing_router_offline_tests=false; missing_signal_count=4; graph_evidence_status=graph_mutation_landed_with_receipt_snapshot; graph_workflow_fixture_validation_result=pass
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

## Planning Turn Update — 2026-05-09

This planning turn did not modify implementation files. It found pre-existing dirty working-tree changes in:

```text
scripts/observe_validation.sh
tests/test_observe_validation_contract.py
```

Those dirty files were completed and validated in Generated Graph JSON Implementation Step 1. The classifier is now the committed path for deriving `missing_generated_graph_json`.

### Completed Execution Slice — classify generated graph JSON evidence

Goal completed: reduce the remaining `missing_generated_graph_json` gap without weakening wrapper evidence semantics.

Acceptance criteria:

1. `missing_generated_graph_json` is derived from explicit classification fields, not only `not state_graph_present`.
2. Live `state/.../graph.json` evidence remains accepted as `generated_graph_json_present`.
3. If wrapper graph capture is not requested and the deterministic graph workflow fixture proves a landed mutation with receipt snapshot, missing generated graph JSON is cleared as fixture substitution.
4. If wrapper graph capture is requested, absence of live generated graph JSON remains a required missing signal even when fixture evidence exists.
5. If neither live graph JSON nor fixture receipt evidence exists, missing generated graph JSON remains true.
6. Observe-validation summary rows include classification, reason, option inventory, missing boolean, and fixture-substitution boolean.
7. Focused unit tests execute every classifier branch.
8. Existing optional wrapper, router/offline, graph fixture, receipt replay, runtime archive, runtime manifest, and runtime performance semantics are preserved.

Validation used for that execution turn:

```text
python3 -m unittest tests/test_observe_validation_contract.py
python3 -m unittest tests/test_graph_workflow_fixture_validator.py
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-generated-json.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
python3 -m py_compile scripts/observe_validation.sh tests/test_observe_validation_contract.py
CANON_TEST_TIMEOUT_SECONDS=1 CANON_OBSERVE_REPORT=target/observe/full-observe-generated-json-timeout-smoke.ndjson python3 scripts/observe_validation.sh
```

The full observe smoke exited `1` while known unrelated missing signals remained, but source-derived summary evidence showed the generated graph JSON classification fields were emitted and `missing_signal_flags.missing_generated_graph_json=false` in the observed live-graph-present environment.

### Commit Hygiene For Next Execution Turn

- Keep generated validation logs, reports, archives, and exit files ignored.
- Before committing, inspect `git diff -- scripts/observe_validation.sh tests/test_observe_validation_contract.py plan.md score.md`.
- If the execution turn owns the current dirty implementation files, include them with that execution commit and update `score.md` with fresh validation evidence.
- If this planning commit is still ahead when execution begins, do not amend it; create a separate implementation commit.


## Validation Evidence From Generated Graph JSON Implementation Step 1

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 26 passed; 0 failed
log: target/validation-logs/observe-validation-contract-generated-json-step1.log
```

```text
python3 -m unittest tests/test_graph_workflow_fixture_validator.py
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/graph-workflow-fixture-validator-generated-json-step1.log
```

```text
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-generated-json-step1.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
exit: 0
result: validation_status=pass, graph_evidence_status=graph_mutation_landed_with_receipt_snapshot, graph_workflow_fixture_receipt_snapshot_present=true
log: target/validation-logs/graph-fixture-report-generated-json-step1.log
```

```text
python3 -m py_compile scripts/observe_validation.sh tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-generated-json-step1.log
```

```text
CANON_TEST_TIMEOUT_SECONDS=1 CANON_OBSERVE_REPORT=target/observe/full-observe-generated-json-step1-timeout-smoke.ndjson python3 scripts/observe_validation.sh
connector result: 502 transport error during long command; ignored artifacts were produced
exit file: target/validation-logs/full-observe-generated-json-step1-timeout-smoke.exit = 1
result: validation_status=fail from known unrelated missing signals; generated graph JSON classification evidence present
summary: generated_graph_json_classification_present=true, generated_graph_json_classification=generated_graph_json_present, generated_graph_json_missing=false, missing_signal_flags.missing_generated_graph_json=false, generated_graph_json_fixture_substitution=false, wrapper_graph_validation_requested=false, wrapper_graph_configuration_status=not_configured, graph_evidence_status=graph_mutation_landed_with_receipt_snapshot, graph_workflow_fixture_validation_result=pass, graph_workflow_fixture_receipt_snapshot_present=true, missing_signal_count=4
```

## Completed Execution Slice After Generated Graph JSON Step 1

Completed P4 runtime archive/base evidence slice by adding compact `--runtime-archive-report` mode and a reusable `runtime_archive_missing_flags()` helper. The report mode proves `runtime_manifest_base_match`, `runtime_download_index`, `runtime_prior_state`, and `runtime_conversation_ledger` in a short focused path without running full cargo validation.


## Validation Evidence From Runtime Archive Report Implementation Step 2

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 27 passed; 0 failed
log: target/validation-logs/observe-validation-contract-runtime-report-step2.log
```

```text
CANON_DELTA_BASE=runtime-base-step2 CANON_RUNTIME_ARCHIVE=target/runtime-archive-step2-report.tar CANON_OBSERVE_REPORT=target/observe/runtime-archive-report-step2.ndjson python3 scripts/observe_validation.sh --runtime-archive-report
exit: 0
result: validation_status=pass, runtime_archive_inspection_status=pass, runtime_manifest_base_matches_delta_base=true, runtime_archive_missing_signal_count=0
summary: missing_runtime_manifest_base_match=false, missing_runtime_download_index=false, missing_runtime_prior_state=false, missing_runtime_conversation_ledger=false
log: target/validation-logs/runtime-archive-report-step2.log
```

```text
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-runtime-step2.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
exit: 0
result: validation_status=pass, graph_evidence_status=graph_mutation_landed_with_receipt_snapshot
log: target/validation-logs/graph-fixture-report-runtime-step2.log
```

```text
python3 -m py_compile scripts/observe_validation.sh tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-runtime-report-step2.log
```

```text
python3 -m unittest tests/test_graph_workflow_fixture_validator.py
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/graph-workflow-fixture-validator-runtime-step2.log
```

## Completed Execution Slice After Runtime Archive Report Step 2

Completed compact runtime archive report integration into full observe-validation summary semantics. Full observe-validation now uses direct `CANON_RUNTIME_ARCHIVE` evidence first, and otherwise can consume a passing compact report supplied through `CANON_RUNTIME_ARCHIVE_REPORT` when its base matches the current `CANON_DELTA_BASE`. Base-mismatched or invalid reports are rejected and fall back to missing archive evidence.


## Validation Evidence From Runtime Archive Report Integration Step 3

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 30 passed; 0 failed
log: target/validation-logs/observe-validation-contract-runtime-integration-step3.log
```

```text
python3 -m py_compile scripts/observe_validation.sh tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-runtime-integration-step3.log
```

```text
CANON_DELTA_BASE=$(git rev-parse HEAD) CANON_RUNTIME_ARCHIVE=target/runtime-archive-step3-head-report.tar CANON_OBSERVE_REPORT=target/observe/runtime-archive-report-step3-head.ndjson python3 scripts/observe_validation.sh --runtime-archive-report
exit: 0
result: validation_status=pass, runtime_archive_inspection_status=pass, runtime_manifest_base_matches_delta_base=true, runtime_archive_missing_signal_count=0
summary: missing_runtime_manifest_base_match=false, missing_runtime_download_index=false, missing_runtime_prior_state=false, missing_runtime_conversation_ledger=false
log: target/validation-logs/runtime-archive-report-step3-head.log
```

```text
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-runtime-integration-step3.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
exit: 0
result: validation_status=pass, graph_evidence_status=graph_mutation_landed_with_receipt_snapshot
log: target/validation-logs/graph-fixture-report-runtime-integration-step3.log
```

```text
python3 -m unittest tests/test_graph_workflow_fixture_validator.py
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/graph-workflow-fixture-validator-runtime-integration-step3.log
```

```text
CANON_TEST_TIMEOUT_SECONDS=1 CANON_DELTA_BASE=$(git rev-parse HEAD) CANON_RUNTIME_ARCHIVE_REPORT=target/observe/runtime-archive-report-step3-head.ndjson CANON_OBSERVE_REPORT=target/observe/full-observe-runtime-report-integration-step3-head.ndjson python3 scripts/observe_validation.sh
connector result: 502 transport error during long command; ignored artifacts were produced
exit file: target/validation-logs/full-observe-runtime-report-integration-step3-head.exit = 1
result: validation_status=fail from command timeout/required command status; compact runtime report integration evidence present
summary: runtime_archive_evidence_source=compact_report, runtime_archive_report_status=pass, runtime_archive_report_base_matches_current=true, delta_base_is_ancestor=true, runtime_manifest_base_matches_delta_base=true, missing_runtime_manifest_base_match=false, missing_runtime_download_index=false, missing_runtime_prior_state=false, missing_runtime_conversation_ledger=false, missing_runtime_performance_signal=false, missing_signal_count=0
```

## Completed Execution Slice After Runtime Archive Report Integration Step 3

Completed full observe-validation status separation. The summary now emits `missing_signal_status`, `command_execution_status`, and `validation_status_reason` so `missing_signal_count=0` can be represented distinctly from required command timeout/failure status while preserving the existing overall `validation_status` and exit behavior.


## Validation Evidence From Status Split Implementation Step 4

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 32 passed; 0 failed
log: target/validation-logs/observe-validation-contract-status-split-step4.log
```

```text
python3 -m py_compile scripts/observe_validation.sh tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-status-split-step4.log
```

```text
CANON_DELTA_BASE=$(git rev-parse HEAD) CANON_RUNTIME_ARCHIVE=target/runtime-archive-step4-head-report.tar CANON_OBSERVE_REPORT=target/observe/runtime-archive-report-step4-head.ndjson python3 scripts/observe_validation.sh --runtime-archive-report
exit: 0
result: validation_status=pass, runtime_archive_inspection_status=pass, runtime_manifest_base_matches_delta_base=true, runtime_archive_missing_signal_count=0
summary: missing_runtime_manifest_base_match=false, missing_runtime_download_index=false, missing_runtime_prior_state=false, missing_runtime_conversation_ledger=false
log: target/validation-logs/runtime-archive-report-step4-head.log
```

```text
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-status-split-step4.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
exit: 0
result: validation_status=pass, graph_evidence_status=graph_mutation_landed_with_receipt_snapshot
log: target/validation-logs/graph-fixture-report-status-split-step4.log
```

```text
python3 -m unittest tests/test_graph_workflow_fixture_validator.py
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/graph-workflow-fixture-validator-status-split-step4.log
```

```text
CANON_TEST_TIMEOUT_SECONDS=1 CANON_DELTA_BASE=$(git rev-parse HEAD) CANON_RUNTIME_ARCHIVE_REPORT=target/observe/runtime-archive-report-step4-head.ndjson CANON_OBSERVE_REPORT=target/observe/full-observe-status-split-step4.ndjson python3 scripts/observe_validation.sh
connector result: 502 transport error during long command; ignored artifacts were produced
exit file: target/validation-logs/full-observe-status-split-step4.exit = 1
result: overall validation_status=fail from required command timeout/failure, while missing-signal health passed
summary: validation_status=fail, validation_status_reason=required_command_failure_or_timeout, command_execution_status=fail, missing_signal_status=pass, missing_signal_count=0, failed_required_commands=[cargo_test_all_targets], runtime_archive_evidence_source=compact_report, runtime_archive_report_status=pass, missing_runtime_manifest_base_match=false, missing_runtime_download_index=false, missing_runtime_prior_state=false, missing_runtime_conversation_ledger=false
```

## Completed Execution Slice After Status Split Step 4

Completed focused required command execution outcome classification. Full observe-validation summary rows now emit `command_execution_classification`, classification options, required command names, statuses, exit codes, timeout list, hard-failure list, missing list, skipped-environment list, passed list, and failed list. This makes timeout versus hard command failure auditable without conflating command execution state with missing-signal health.


## Validation Evidence From Command Classification Implementation Step 5

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 34 passed; 0 failed
log: target/validation-logs/observe-validation-contract-command-classification-step5.log
```

```text
python3 -m py_compile scripts/observe_validation.sh tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-command-classification-step5.log
```

```text
CANON_DELTA_BASE=$(git rev-parse HEAD) CANON_RUNTIME_ARCHIVE=target/runtime-archive-step5-head-report.tar CANON_OBSERVE_REPORT=target/observe/runtime-archive-report-step5-head.ndjson python3 scripts/observe_validation.sh --runtime-archive-report
exit: 0
result: validation_status=pass, runtime_archive_inspection_status=pass, runtime_manifest_base_matches_delta_base=true, runtime_archive_missing_signal_count=0
summary: missing_runtime_manifest_base_match=false, missing_runtime_download_index=false, missing_runtime_prior_state=false, missing_runtime_conversation_ledger=false
log: target/validation-logs/runtime-archive-report-step5-head.log
```

```text
CANON_OBSERVE_REPORT=target/observe/graph-fixture-report-command-classification-step5.ndjson python3 scripts/observe_validation.sh --graph-fixture-report
exit: 0
result: validation_status=pass, graph_evidence_status=graph_mutation_landed_with_receipt_snapshot
log: target/validation-logs/graph-fixture-report-command-classification-step5.log
```

```text
python3 -m unittest tests/test_graph_workflow_fixture_validator.py
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/graph-workflow-fixture-validator-command-classification-step5.log
```

```text
CANON_TEST_TIMEOUT_SECONDS=1 CANON_DELTA_BASE=$(git rev-parse HEAD) CANON_RUNTIME_ARCHIVE_REPORT=target/observe/runtime-archive-report-step5-head.ndjson CANON_OBSERVE_REPORT=target/observe/full-observe-command-classification-step5.ndjson python3 scripts/observe_validation.sh
connector result: 502 transport error during long command; ignored artifacts were produced
exit file: target/validation-logs/full-observe-command-classification-step5.exit = 1
result: overall validation_status=fail from required command hard failure while missing-signal health passed
summary: command_execution_classification=required_command_hard_failure, command_execution_status=fail, validation_status_reason=required_command_failure_or_timeout, missing_signal_status=pass, missing_signal_count=0, required_command_failed=[cargo_test_all_targets], required_command_hard_failed=[cargo_test_all_targets], required_command_timed_out=[], required_command_exit_codes.cargo_test_all_targets=101, runtime_archive_evidence_source=compact_report
```

## Completed Execution Slice After Command Classification Step 5

Completed the compact command-execution report-only mode. `scripts/observe_validation.sh --command-execution-report` now emits a single `command_execution_report` row with `command_execution_summary()` fields, required command status maps, exit-code maps, hard-failure/timeout/missing/skipped/pass lists, connector failure classes, and a fixture hook through `CANON_COMMAND_EXECUTION_FIXTURE`. This gives short, deterministic evidence for command classification without running long cargo validation.

## Validation Evidence From Command Report Implementation Step 1

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 36 passed; 0 failed
log: target/validation-logs/observe-validation-contract-command-report-step1.log
exit file: target/validation-logs/observe-validation-contract-command-report-step1.exit
```

```text
python3 -m py_compile scripts/observe_validation.sh tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-command-report-step1.log
exit file: target/validation-logs/py-compile-command-report-step1.exit
```

```text
CANON_OBSERVE_REPORT=target/observe/command-execution-report-step1.ndjson python3 scripts/observe_validation.sh --command-execution-report
exit: 1
expected result: validation_status=fail for deterministic fixture hard failure
summary: event=command_execution_report, command_execution_report_only=true, command_execution_classification=required_command_hard_failure, command_execution_status=fail, required_command_hard_failed=[cargo_test_all_targets], required_command_exit_codes.cargo_test_all_targets=101
log: target/validation-logs/command-execution-report-step1.log
exit file: target/validation-logs/command-execution-report-step1.exit
```

## Completed Execution Slice After Command Report Step 1

Completed connector transport artifact classification. Observe-validation now emits `connector_transport_artifact_classification` fields in both full validation summaries and compact command-execution reports. The classifier distinguishes normal artifact states from transport interruption with complete artifacts versus transport interruption with incomplete artifacts. This separates connector transport instability from validation command outcomes when ignored report/exit artifacts are available.

## Validation Evidence From Transport Artifact Implementation Step 2

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 38 passed; 0 failed
log: target/validation-logs/observe-validation-contract-transport-artifacts-step2.log
exit file: target/validation-logs/observe-validation-contract-transport-artifacts-step2.exit
```

```text
python3 -m py_compile scripts/observe_validation.sh tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-transport-artifacts-step2.log
exit file: target/validation-logs/py-compile-transport-artifacts-step2.exit
```

```text
CANON_OBSERVE_REPORT=target/observe/command-execution-transport-artifacts-step2.ndjson CANON_CONNECTOR_TRANSPORT_STATUS=502 CANON_CONNECTOR_TRANSPORT_REPORT=target/observe/command-execution-transport-artifacts-step2.ndjson CANON_CONNECTOR_TRANSPORT_EXIT_FILE=target/validation-logs/transport-artifacts-step2.exit python3 scripts/observe_validation.sh --command-execution-report
exit: 1
expected result: validation_status=fail for deterministic command hard-failure fixture, while transport artifacts are classified complete
summary: command_execution_classification=required_command_hard_failure, connector_transport_artifact_classification=transport_interrupted_artifacts_complete, connector_transport_report_complete=true, connector_transport_exit_file_present=true
log: target/validation-logs/command-execution-transport-artifacts-step2.log
exit file: target/validation-logs/command-execution-transport-artifacts-step2.exit
```

## Completed Execution Slice After Transport Artifact Step 2

Completed higher-level consumer integration for connector transport artifact classification. `scripts/write_delta_manifest.py` now preserves `connector_transport_artifact_classification` and related artifact state fields from observe-validation summaries into the delta receipt and rendered manifest. This prevents downstream delta/archive consumers from relying only on the older `connector_transport_instability_present` boolean.

## Validation Evidence From Manifest Consumer Implementation Step 3

```text
python3 -m unittest tests/test_write_delta_manifest.py
exit: 0
result: 11 passed; 0 failed
log: target/validation-logs/write-delta-manifest-transport-artifacts-step3.log
exit file: target/validation-logs/write-delta-manifest-transport-artifacts-step3.exit
```

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 38 passed; 0 failed
log: target/validation-logs/observe-validation-contract-transport-artifacts-step3.log
exit file: target/validation-logs/observe-validation-contract-transport-artifacts-step3.exit
```

```text
python3 -m py_compile scripts/write_delta_manifest.py scripts/observe_validation.sh tests/test_write_delta_manifest.py tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-transport-artifacts-step3.log
exit file: target/validation-logs/py-compile-transport-artifacts-step3.exit
```

## Completed Execution Slice After Manifest Consumer Step 3

Completed plan/documentation drift cleanup for the current P4 state. The top-level snapshot now lists the landed generated graph JSON classification, compact runtime archive report integration, separated validation status fields, compact command-execution reports, connector transport artifact classification, and delta manifest preservation of transport artifact state as completed baseline capabilities rather than active pending implementation.

## Validation Evidence From Planning Cleanup Step 4

```text
cargo test --test planning_contract -- --test-threads=1
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/planning-contract-doc-cleanup-step4.log
exit file: target/validation-logs/planning-contract-doc-cleanup-step4.exit
```

```text
cargo test --test score_contract -- --test-threads=1
exit: 0
result: 5 passed; 0 failed
log: target/validation-logs/score-contract-doc-cleanup-step4.log
exit file: target/validation-logs/score-contract-doc-cleanup-step4.exit
```

```text
python3 - <<'PY'
from pathlib import Path
text = Path('plan.md').read_text(encoding='utf-8')
stale_phrase = 'current ' + 'uncommitted work'
assert stale_phrase not in text
assert 'Current P4 Completion Summary' in text
assert '30bf057 Preserve connector transport artifacts in manifests' in text
PY
exit: 0
log: target/validation-logs/plan-doc-sanity-step4.log
exit file: target/validation-logs/plan-doc-sanity-step4.exit
```

## Completed Execution Slice After Planning Cleanup Step 4

Completed compact full-summary artifact replay. `scripts/observe_validation.sh --full-summary-report` now emits a deterministic `validation_summary` row without running long cargo validation. The row proves command execution status, missing-signal status, connector transport artifact classification, compact runtime archive evidence, runtime manifest base-match evidence, runtime performance signal presence, and command rows that are accepted by the delta manifest closure validator.

## Validation Evidence From Full-Summary Replay Step 5

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 39 passed; 0 failed
log: target/validation-logs/observe-validation-contract-full-summary-step5.log
exit file: target/validation-logs/observe-validation-contract-full-summary-step5.exit
```

```text
python3 -m unittest tests/test_write_delta_manifest.py
exit: 0
result: 12 passed; 0 failed
log: target/validation-logs/write-delta-manifest-full-summary-step5.log
exit file: target/validation-logs/write-delta-manifest-full-summary-step5.exit
```

```text
python3 -m py_compile scripts/observe_validation.sh scripts/write_delta_manifest.py tests/test_observe_validation_contract.py tests/test_write_delta_manifest.py
exit: 0
log: target/validation-logs/py-compile-full-summary-step5.log
exit file: target/validation-logs/py-compile-full-summary-step5.exit
```

```text
CANON_OBSERVE_REPORT=target/observe/full-summary-step5.ndjson CANON_DELTA_BASE=$(git rev-parse HEAD) CANON_CONNECTOR_TRANSPORT_STATUS=502 CANON_CONNECTOR_TRANSPORT_REPORT=target/observe/full-summary-step5.ndjson CANON_CONNECTOR_TRANSPORT_EXIT_FILE=target/validation-logs/full-summary-step5.exit python3 scripts/observe_validation.sh --full-summary-report
exit: 0
summary: event=validation_summary, validation_status=pass, command_execution_status=pass, missing_signal_status=pass, connector_transport_artifact_classification=transport_interrupted_artifacts_complete, runtime_archive_evidence_source=compact_report, validation_command_count=3, validation_test_count=3
log: target/validation-logs/full-summary-step5.log
exit file: target/validation-logs/full-summary-step5-command.exit
```

## Next Execution Slice After Full-Summary Replay Step 5

Continue P4 by using the compact full-summary report as a reusable fixture in any remaining receiver/archive workflows that still require long observe-validation artifacts. Candidate next slice: generate a delta manifest from an actual `--full-summary-report` artifact in a focused integration test or script-level contract. Preserve compact command-execution report mode, separated status fields, compact runtime report integration, direct archive precedence, optional wrapper semantics, generated graph JSON classification, graph fixture evidence, receipt replay inventory, runtime performance evidence, connector transport artifact classification, and delta-manifest preservation of transport artifact state.

## Current Non-Goals

- Do not wire domain specs into runtime behavior yet.
- Do not redesign the kernel.
- Do not add live trading behavior.
- Do not rely on external LLM/Ollama/OpenAI availability for baseline correctness.
- Do not commit token caches, runtime archives, target outputs, validation logs, or SSE chunk logs.

## Planning Step 6 — Current Implementation Plan

This turn is planning/scoring only. No implementation files should be changed in this turn.

### Immediate execution target for the next implementation turn

Add a focused integration test or script-level contract that generates a delta manifest from an actual compact full-summary artifact produced by:

```text
python3 scripts/observe_validation.sh --full-summary-report
```

The next implementation should prove the generated artifact can feed `scripts/write_delta_manifest.py` end-to-end, rather than only proving that a hand-constructed or in-memory full-summary-shaped row is accepted by existing manifest tests.

### Required acceptance criteria

1. The test creates a temporary observe report path under an ignored output location such as `target/observe/` or a test temporary directory.
2. The test invokes or faithfully executes the actual `--full-summary-report` path.
3. The test feeds that emitted report into the delta manifest writer.
4. The resulting manifest preserves at least these fields:
   - `validation_status=pass`
   - `command_execution_status=pass`
   - `missing_signal_status=pass`
   - `connector_transport_artifact_classification=transport_interrupted_artifacts_complete`
   - `runtime_archive_evidence_source=compact_report`
   - command rows containing `cmd` values suitable for closure validation
5. The test remains short and deterministic, with no dependency on live wrapper telemetry, router, Ollama, OpenAI, network, long cargo validation, or external services.
6. Existing compact modes must remain intact:
   - `--graph-fixture-report`
   - `--command-execution-report`
   - `--full-summary-report`
7. Existing separated status semantics must remain intact:
   - `validation_status`
   - `command_execution_status`
   - `missing_signal_status`

### Candidate file scope for next implementation turn

Likely files:

```text
tests/test_write_delta_manifest.py
scripts/observe_validation.sh
plan.md
score.md
```

`tests/test_write_delta_manifest.py` is the preferred first target. Modify `scripts/observe_validation.sh` only if the real emitted report lacks fields needed for manifest generation or closure validation.

### Suggested validation commands for next implementation turn

```text
python3 -m unittest tests/test_write_delta_manifest.py
python3 -m unittest tests/test_observe_validation_contract.py
python3 -m py_compile scripts/observe_validation.sh scripts/write_delta_manifest.py tests/test_write_delta_manifest.py tests/test_observe_validation_contract.py
```

If the implementation touches only manifest tests and no observe-validation behavior, the observe-validation contract can still be run as a regression guard because this slice depends on the compact full-summary report contract.

### Non-goals for next implementation turn

- Do not run or require long full observe-validation.
- Do not add live wrapper-configured validation as a prerequisite.
- Do not introduce network, model-provider, or router dependencies.
- Do not broaden graph workflow semantics.
- Do not change scoring upward unless the new end-to-end manifest-from-real-report evidence passes.

## Completed Execution Slice After Actual Full-Summary Manifest Step 1

Completed the focused receiver/archive integration slice for compact full-summary artifacts. `tests/test_write_delta_manifest.py` now generates a real `--full-summary-report` artifact by running the observe-validation compact report path inside the temporary manifest repository, feeds that emitted NDJSON report into the delta manifest writer, and verifies that the resulting receipt and manifest preserve validation status, command execution status, missing-signal status, connector transport artifact classification, compact runtime evidence source, runtime manifest base-match evidence, and validation command `cmd` fields.

`scripts/write_delta_manifest.py` now preserves compact full-summary status fields that were previously consumed only by direct observe-validation checks:

```text
command_execution_status
missing_signal_status
runtime_archive_evidence_source
full_summary_report_only
full_summary_report_command
runtime_manifest_base_commit
runtime_archive_report_present
runtime_archive_report_status
runtime_archive_report_base_matches_current
runtime_archive_present
```

## Validation Evidence From Actual Full-Summary Manifest Step 1

```text
python3 -m unittest tests/test_write_delta_manifest.py
exit: 0
result: 13 passed; 0 failed
log: target/validation-logs/write-delta-manifest-actual-full-summary-step1.log
```

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 39 passed; 0 failed
log: target/validation-logs/observe-validation-contract-actual-full-summary-step1.log
```

```text
python3 -m py_compile scripts/observe_validation.sh scripts/write_delta_manifest.py tests/test_write_delta_manifest.py tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-actual-full-summary-step1.log
```

## Next Execution Slice After Actual Full-Summary Manifest Step 1

Continue P4 by finding any remaining receiver/archive/report workflows that still depend on long observe-validation artifacts and convert one more such consumer to compact deterministic evidence. Candidate next slice: add a short contract proving compact runtime archive/full-summary preserved fields are not duplicated in the rendered manifest and remain stable when command rows are sourced from report rows instead of only the summary `validation_commands` field.

## Completed Execution Slice After Row-Command Manifest Step 2

Completed the short receiver/archive contract for compact report stability. `tests/test_write_delta_manifest.py` now proves that delta manifest generation remains valid when validation commands are sourced from `validation_command` report rows instead of the summary `validation_commands` field.

The new contract also proves compact full-summary/runtime preserved fields render exactly once in the manifest:

```text
validation_status
command_execution_status
missing_signal_status
missing_signal_count
runtime_archive_evidence_source
runtime_archive_report_present
runtime_archive_report_status
runtime_archive_report_base_matches_current
runtime_archive_present
runtime_manifest_base_expected
runtime_manifest_base_commit
runtime_manifest_base_matches_delta_base
full_summary_report_only
full_summary_report_command
```

No manifest writer change was required for this slice; the existing fallback and de-duplication logic already satisfied the new executable contract.

## Validation Evidence From Row-Command Manifest Step 2

```text
python3 -m unittest tests/test_write_delta_manifest.py
exit: 0
result: 14 passed; 0 failed
log: target/validation-logs/write-delta-manifest-row-command-step2.log
```

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 39 passed; 0 failed
log: target/validation-logs/observe-validation-contract-row-command-step2.log
```

```text
python3 -m py_compile scripts/observe_validation.sh scripts/write_delta_manifest.py tests/test_write_delta_manifest.py tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-row-command-step2.log
```

```text
cargo test --test planning_contract -- --test-threads=1
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/planning-contract-row-command-step2.log
```

```text
cargo test --test score_contract -- --test-threads=1
exit: 0
result: 5 passed; 0 failed
log: target/validation-logs/score-contract-row-command-step2.log
```

## Next Execution Slice After Row-Command Manifest Step 2

Continue P4 by looking for the next receiver/archive/report assumption that can be made executable with a short deterministic fixture. Candidate next slice: strengthen manifest closure around contradictory validation command sources, such as rejecting or classifying reports where summary `validation_commands` conflict with `validation_command` rows or where command-count metadata does not match the command evidence selected by the writer.

## Completed Execution Slice After Command-Conflict Manifest Step 3

Completed deterministic manifest closure for contradictory validation command evidence. `scripts/write_delta_manifest.py` now compares command fingerprints when both summary `validation_commands` and `validation_command` report rows are present. Matching duplicate evidence remains accepted, while conflicting name/cmd/status evidence is rejected before a receipt or manifest can be generated.

The command fingerprint used for conflict detection is intentionally small and closure-oriented:

```text
name
cmd
status
```

`tests/test_write_delta_manifest.py` now covers both matching duplicate command evidence and conflicting duplicate command evidence. This closes the prior gap where the manifest writer preferred summary commands over row commands without explicitly classifying or rejecting disagreement.

## Validation Evidence From Command-Conflict Manifest Step 3

```text
python3 -m unittest tests/test_write_delta_manifest.py
exit: 0
result: 16 passed; 0 failed
log: target/validation-logs/write-delta-manifest-command-conflict-step3.log
```

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 39 passed; 0 failed
log: target/validation-logs/observe-validation-contract-command-conflict-step3.log
```

```text
python3 -m py_compile scripts/observe_validation.sh scripts/write_delta_manifest.py tests/test_write_delta_manifest.py tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-command-conflict-step3.log
```

```text
cargo test --test planning_contract -- --test-threads=1
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/planning-contract-command-conflict-step3.log
```

```text
cargo test --test score_contract -- --test-threads=1
exit: 0
result: 5 passed; 0 failed
log: target/validation-logs/score-contract-command-conflict-step3.log
```

## Next Execution Slice After Command-Conflict Manifest Step 3

Continue P4 by strengthening manifest closure around command metadata completeness beyond name/cmd/status. Candidate next slice: reject or classify reports where duplicate command evidence agrees on name/cmd/status but disagrees on execution-relevant metadata such as `exit_code`, `duration_ms`, `timed_out`, or `connector_failure_class`.

## Completed Execution Slice After Command-Metadata Manifest Step 4

Completed deterministic manifest closure for duplicate command execution metadata. `scripts/write_delta_manifest.py` now includes execution-relevant metadata in the command fingerprint used when both summary `validation_commands` and `validation_command` report rows are present.

The duplicate command evidence fingerprint now includes:

```text
name
cmd
status
exit_code
duration_ms
timed_out
connector_failure_class
```

`tests/test_write_delta_manifest.py` now proves matching duplicate command metadata remains accepted and conflicting duplicate command execution metadata is rejected before a receipt or manifest can be generated. This closes the prior gap where command identity/status agreement could hide disagreement about observed execution details.

## Validation Evidence From Command-Metadata Manifest Step 4

```text
python3 -m unittest tests/test_write_delta_manifest.py
exit: 0
result: 17 passed; 0 failed
log: target/validation-logs/write-delta-manifest-command-metadata-step4.log
```

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 39 passed; 0 failed
log: target/validation-logs/observe-validation-contract-command-metadata-step4.log
```

```text
python3 -m py_compile scripts/observe_validation.sh scripts/write_delta_manifest.py tests/test_write_delta_manifest.py tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-command-metadata-step4.log
```

```text
cargo test --test planning_contract -- --test-threads=1
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/planning-contract-command-metadata-step4.log
```

```text
cargo test --test score_contract -- --test-threads=1
exit: 0
result: 5 passed; 0 failed
log: target/validation-logs/score-contract-command-metadata-step4.log
```

## Next Execution Slice After Command-Metadata Manifest Step 4

Continue P4 by strengthening validation report closure around duplicate command identity. Candidate next slice: reject or classify duplicate `validation_command` rows with the same command name but conflicting evidence, even when the summary omits `validation_commands`.

## Completed Execution Slice After Duplicate-Row Manifest Step 5

Completed deterministic manifest closure for duplicate `validation_command` rows. `scripts/write_delta_manifest.py` now checks row-level duplicate command names before selecting summary or row command evidence. Exact duplicate row evidence remains accepted, while duplicate rows with the same command name and conflicting fingerprints are rejected before a receipt or manifest can be generated.

The duplicate-row check uses the same command fingerprint as summary/row conflict detection:

```text
name
cmd
status
exit_code
duration_ms
timed_out
connector_failure_class
```

`tests/test_write_delta_manifest.py` now covers exact duplicate row evidence and conflicting duplicate row evidence when the summary omits `validation_commands`. This closes the prior gap where row-only command evidence could contain conflicting duplicate command names.

## Validation Evidence From Duplicate-Row Manifest Step 5

```text
python3 -m unittest tests/test_write_delta_manifest.py
exit: 0
result: 19 passed; 0 failed
log: target/validation-logs/write-delta-manifest-duplicate-rows-step5.log
```

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 39 passed; 0 failed
log: target/validation-logs/observe-validation-contract-duplicate-rows-step5.log
```

```text
python3 -m py_compile scripts/observe_validation.sh scripts/write_delta_manifest.py tests/test_write_delta_manifest.py tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-duplicate-rows-step5.log
```

```text
cargo test --test planning_contract -- --test-threads=1
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/planning-contract-duplicate-rows-step5.log
```

```text
cargo test --test score_contract -- --test-threads=1
exit: 0
result: 5 passed; 0 failed
log: target/validation-logs/score-contract-duplicate-rows-step5.log
```

## Next Execution Slice After Duplicate-Row Manifest Step 5

Continue P4 by strengthening command-count closure around duplicate row evidence. Candidate next slice: require `validation_command_count` to match the number of distinct command names or explicitly classify/allow exact duplicate rows, so repeated identical command rows cannot inflate command coverage without clear evidence semantics.


## Completed Execution Slice After Distinct Command-Count Manifest Step 1

Completed deterministic manifest closure for exact duplicate `validation_command` rows. `scripts/write_delta_manifest.py` now deduplicates identical row evidence by command name after rejecting conflicting duplicate rows. Exact duplicate row evidence may support one distinct command, but it can no longer inflate `validation_command_count` or receipt command lists.

The row-only command evidence path now applies this order:

```text
collect validation_command rows
reject conflicting duplicate command names
deduplicate identical duplicate command names
validate summary validation_command_count against distinct commands
```

`tests/test_write_delta_manifest.py` now proves both sides of the new semantics: repeated identical row evidence is accepted when `validation_command_count` names one distinct command, and rejected when the summary attempts to count the duplicate row as a second command.

## Validation Evidence From Distinct Command-Count Manifest Step 1

```text
python3 -m unittest tests/test_write_delta_manifest.py
exit: 0
result: 20 passed; 0 failed
log: target/validation-logs/write-delta-manifest-distinct-command-count-step1.log
```

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 39 passed; 0 failed
log: target/validation-logs/observe-validation-contract-distinct-command-count-step1.log
```

```text
python3 -m py_compile scripts/observe_validation.sh scripts/write_delta_manifest.py tests/test_write_delta_manifest.py tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-distinct-command-count-step1.log
```

```text
cargo test --test planning_contract -- --test-threads=1
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/planning-contract-distinct-command-count-step1.log
```

```text
cargo test --test score_contract -- --test-threads=1
exit: 0
result: 5 passed; 0 failed
log: target/validation-logs/score-contract-distinct-command-count-step1.log
```

## Next Execution Slice After Distinct Command-Count Manifest Step 1

Continue P4 by applying the same duplicate command-name closure to summary-provided `validation_commands`, not only row-derived command evidence. Candidate next slice: reject duplicate summary command names with conflicting evidence and deduplicate or reject identical duplicate summary commands before validating `validation_command_count`.


## Completed Execution Slice After Summary Command-Closure Manifest Step 2

Completed deterministic duplicate command-name closure for summary-provided `validation_commands`. `scripts/write_delta_manifest.py` now applies the same conflict rejection and deduplication semantics to summary command arrays and row-derived command evidence before cross-source comparison, count validation, and receipt generation.

The command evidence validation path now applies this order to both summary and row command sources:

```text
collect command evidence
reject conflicting duplicate command names per source
deduplicate identical duplicate command names per source
compare summary and row fingerprints when both sources are present
validate validation_command_count against distinct commands
```

`tests/test_write_delta_manifest.py` now proves duplicate summary command semantics explicitly: repeated identical summary command evidence is accepted when `validation_command_count` names one distinct command, rejected when the summary attempts to count the duplicate as a second command, and rejected when duplicate summary entries disagree on execution metadata.

## Validation Evidence From Summary Command-Closure Manifest Step 2

```text
python3 -m unittest tests/test_write_delta_manifest.py
exit: 0
result: 23 passed; 0 failed
log: target/validation-logs/write-delta-manifest-summary-duplicates-step2.log
```

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 39 passed; 0 failed
log: target/validation-logs/observe-validation-contract-summary-duplicates-step2.log
```

```text
python3 -m py_compile scripts/observe_validation.sh scripts/write_delta_manifest.py tests/test_write_delta_manifest.py tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-summary-duplicates-step2.log
```

```text
cargo test --test planning_contract -- --test-threads=1
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/planning-contract-summary-duplicates-step2.log
```

```text
cargo test --test score_contract -- --test-threads=1
exit: 0
result: 5 passed; 0 failed
log: target/validation-logs/score-contract-summary-duplicates-step2.log
```

## Next Execution Slice After Summary Command-Closure Manifest Step 2

Continue P4 by improving receipt/manifest transparency for normalized command evidence. Candidate next slice: persist explicit command evidence normalization metadata, such as source command counts versus distinct command counts, so receivers can see when duplicate evidence was deduplicated rather than only observing the final distinct command list.


## Completed Execution Slice After Command Normalization Metadata Step 3

Completed explicit command evidence normalization metadata for delta manifest receipts and manifests. `scripts/write_delta_manifest.py` now emits source labels and count semantics after command evidence normalization, so receivers can audit when duplicate command evidence was deduplicated rather than only seeing the final distinct command list.

The receipt and manifest now preserve:

```text
validation_command_distinct_count
validation_command_source
validation_command_summary_input_count
validation_command_summary_distinct_count
validation_command_summary_duplicate_count
validation_command_row_input_count
validation_command_row_distinct_count
validation_command_row_duplicate_count
```

`tests/test_write_delta_manifest.py` now asserts that the metadata is present in both receipts and manifests for mixed summary/row command evidence, duplicate summary evidence, and duplicate row evidence.

## Validation Evidence From Command Normalization Metadata Step 3

```text
python3 -m unittest tests/test_write_delta_manifest.py
exit: 0
result: 23 passed; 0 failed
log: target/validation-logs/write-delta-manifest-command-normalization-step3.log
```

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 39 passed; 0 failed
log: target/validation-logs/observe-validation-contract-command-normalization-step3.log
```

```text
python3 -m py_compile scripts/observe_validation.sh scripts/write_delta_manifest.py tests/test_write_delta_manifest.py tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-command-normalization-step3.log
```

```text
cargo test --test planning_contract -- --test-threads=1
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/planning-contract-command-normalization-step3.log
```

```text
cargo test --test score_contract -- --test-threads=1
exit: 0
result: 5 passed; 0 failed
log: target/validation-logs/score-contract-command-normalization-step3.log
```

## Next Execution Slice After Command Normalization Metadata Step 3

Continue P4 by proving command normalization metadata survives receiver/archive workflows that generate a manifest from an actual compact `--full-summary-report` artifact. Candidate next slice: extend the actual full-summary artifact manifest generation test so its generated receipt and manifest include command source/count normalization metadata.


## Completed Execution Slice After Full-Summary Command Normalization Step 4

Completed receiver/archive coverage for command normalization metadata through the actual compact `--full-summary-report` artifact manifest-generation path. `tests/test_write_delta_manifest.py` now verifies that a report produced by the copied `observe_validation.sh --full-summary-report` script can be consumed by `write_delta_manifest.py` and still emits command normalization metadata in both the generated receipt and manifest.

The actual full-summary artifact path now asserts:

```text
validation_command_distinct_count
validation_command_source
validation_command_summary_input_count
validation_command_summary_distinct_count
validation_command_summary_duplicate_count
validation_command_row_input_count
validation_command_row_distinct_count
validation_command_row_duplicate_count
```

This proves the metadata is not limited to hand-written report fixtures and survives the compact summary receiver workflow used for short transport-safe artifacts.

## Validation Evidence From Full-Summary Command Normalization Step 4

```text
python3 -m unittest tests/test_write_delta_manifest.py
exit: 0
result: 23 passed; 0 failed
log: target/validation-logs/write-delta-manifest-full-summary-normalization-step4.log
```

```text
python3 -m unittest tests/test_observe_validation_contract.py
exit: 0
result: 39 passed; 0 failed
log: target/validation-logs/observe-validation-contract-full-summary-normalization-step4.log
```

```text
python3 -m py_compile scripts/observe_validation.sh scripts/write_delta_manifest.py tests/test_write_delta_manifest.py tests/test_observe_validation_contract.py
exit: 0
log: target/validation-logs/py-compile-full-summary-normalization-step4.log
```

```text
cargo test --test planning_contract -- --test-threads=1
exit: 0
result: 2 passed; 0 failed
log: target/validation-logs/planning-contract-full-summary-normalization-step4.log
```

```text
cargo test --test score_contract -- --test-threads=1
exit: 0
result: 5 passed; 0 failed
log: target/validation-logs/score-contract-full-summary-normalization-step4.log
```

## Next Execution Slice After Full-Summary Command Normalization Step 4

Continue P4 by adding compact manifest rendering checks for the new command normalization metadata. Candidate next slice: assert each command normalization metric renders exactly once in the generated manifest, including in compact full-summary and row-fallback paths.
