# Canon Agent Implementation Plan

This plan records the current implementation state after implementation step 1 of the current agent loop.

## North Star

Canon Agent is a deterministic, self-improving runtime where:

- the state-machine kernel governs all control flow;
- capabilities produce typed evidence, never unchecked authority;
- every decision is replayable through hash-chained logs and receipts;
- learning promotes only externally verified wins into append-only policy;
- LLM calls shrink over time as policy handles repeated cases.

The architecture must keep safety and intelligence separated: the kernel enforces correctness, the capability layer accumulates intelligence, and policy learning never grants itself authority.

## Current Implementation Baseline

The repository currently exposes these meaningful surfaces:

- frozen-style Rust crate with `#![forbid(unsafe_code)]` at the public root;
- kernel/runtime/codec/API/capability separation through `src/lib.rs`;
- deterministic planning, scoring, timing, recovery, reducer, transition-table, durable-log, and verification contracts;
- API transport idempotence and replay receipts;
- graph-as-source-of-truth mutation contracts and CLI workflow tests;
- local/Ollama and OpenAI-compatible LLM effect/proof receipt surfaces;
- validation harness constants and observe-validation script contracts;
- policy, learning, eval, judgment, verification, tooling, memory, observation, and orchestration modules exported as crate surfaces;
- deterministic policy reuse ledger summary receipt in the judgment layer;
- validation-harness/root-validate smoke exposure for policy reuse ledger summary evidence, including a controlled validation-regression case;
- deterministic policy reuse scale trace receipt in the judgment layer;
- validation-harness/root-validate smoke exposure for larger-batch policy reuse scale evidence, including a controlled validation-regression case;
- deterministic policy reuse performance-cost trend receipt in the judgment layer;
- validation-harness/root-validate smoke exposure for reuse/cost trend evidence, including a controlled cost-regression case;
- deterministic policy reuse cost catalog receipt in the judgment layer;
- validation-harness/root-validate smoke exposure for complete and incomplete reuse/cost catalog evidence;
- deterministic policy reuse evaluator savings receipt in the judgment layer;
- validation-harness/root-validate smoke exposure for healthy and controlled regression evaluator-savings evidence;
- deterministic policy reuse scaling projection receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled regression scaling-projection evidence;
- deterministic policy reuse distillation-readiness receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled regression distillation-readiness evidence;
- deterministic policy reuse evidence-surface index receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled regression evidence-surface index evidence;
- deterministic policy reuse evidence-bundle receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled regression evidence-bundle evidence;
- deterministic policy reuse evidence-quickcheck receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled missing-command evidence-quickcheck evidence;
- deterministic policy reuse evidence-maturity receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled quickcheck-failed evidence-maturity evidence;
- deterministic policy reuse evidence-summary receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled immature-maturity evidence-summary evidence;
- deterministic policy reuse evidence-manifest receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled missing-summary-mode evidence-manifest evidence;
- deterministic policy reuse evidence validation-budget receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled budget-exceeded validation-budget evidence;
- deterministic policy reuse evidence rollout-readiness receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled validation-budget-failed rollout-readiness evidence;
- deterministic policy reuse evidence learning-admission receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled rollout-not-ready learning-admission evidence;
- deterministic policy reuse evidence retrieval-readiness receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled learning-not-admissible retrieval-readiness evidence;
- deterministic policy reuse evidence compact-validation receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled retrieval-not-ready compact-validation evidence;
- deterministic policy reuse evidence batch-readiness receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled compact-validation-failed batch-readiness evidence;
- deterministic policy reuse evidence batch-execution-plan receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled batch-not-ready batch-execution-plan evidence;
- deterministic policy reuse evidence batch-evaluation-admission receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled plan-not-ready batch-evaluation-admission evidence;
- deterministic policy reuse evidence batch-run-request receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled admission-not-granted batch-run-request evidence;
- deterministic policy reuse evidence external-evaluator-result receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled evaluator-failed external-evaluator-result evidence;
- deterministic policy reuse evidence learning-candidate receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled evaluator-not-passed learning-candidate evidence;
- deterministic policy reuse evidence learning-data-admission receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled candidate-not-ready learning-data-admission evidence;
- deterministic policy reuse evidence retrieval-example-admission receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled data-not-admitted retrieval-example-admission evidence;
- deterministic policy reuse evidence retrieval-example-index receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled example-not-admitted retrieval-example-index evidence;
- deterministic policy reuse evidence retrieval-corpus-readiness receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled index-not-ready retrieval-corpus-readiness evidence;
- deterministic policy reuse evidence retrieval-corpus-admission receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled corpus-not-ready retrieval-corpus-admission evidence;
- deterministic policy reuse evidence retrieval-use-approval receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled corpus-not-admitted retrieval-use-approval evidence;
- deterministic policy reuse evidence retrieval-use-manifest receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled use-not-approved retrieval-use-manifest evidence;
- deterministic policy reuse evidence retrieval-query-plan receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled manifest-not-ready retrieval-query-plan evidence;
- deterministic policy reuse evidence retrieval-query-approval receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled query-plan-not-ready retrieval-query-approval evidence;
- deterministic policy reuse evidence retrieval-result-admission receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled query-not-approved retrieval-result-admission evidence;
- deterministic policy reuse evidence retrieval-result-manifest receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled result-not-admitted retrieval-result-manifest evidence;
- deterministic policy reuse evidence retrieval-result-use-admission receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled manifest-not-ready retrieval-result-use-admission evidence;
- deterministic policy reuse evidence retrieval-result-use-manifest receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled use-not-admitted retrieval-result-use-manifest evidence;
- deterministic policy reuse evidence retrieval-result-use-readiness receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled manifest-not-ready retrieval-result-use-readiness evidence;
- deterministic policy reuse evidence retrieval-result-use-approval receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled readiness-not-ready retrieval-result-use-approval evidence;
- deterministic policy reuse evidence retrieval-result-use-manifest-admission receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled approval-not-granted retrieval-result-use-manifest-admission evidence;
- deterministic policy reuse evidence retrieval-result-use-summary receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled manifest-not-admitted retrieval-result-use-summary evidence;
- deterministic policy reuse evidence retrieval-result-use-summary-manifest receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled summary-not-ready retrieval-result-use-summary-manifest evidence;
- deterministic policy reuse evidence retrieval-result-use-summary-manifest-admission receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled summary-manifest-not-ready retrieval-result-use-summary-manifest-admission evidence;
- deterministic policy reuse evidence retrieval-result-use-summary-manifest-readiness receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled summary-manifest-not-admitted retrieval-result-use-summary-manifest-readiness evidence;
- deterministic policy reuse evidence retrieval-result-use-summary-manifest-approval receipt in the validation harness;
- validation-harness/root-validate smoke exposure for healthy and controlled summary-manifest-not-ready-for-use retrieval-result-use-summary-manifest-approval evidence;
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, external CLI mode evidence, and evidence-surface index coverage.

## Latest Completed Slice

The latest completed implementation slice is deterministic **policy reuse evidence retrieval-result-use-summary-manifest-approval** evidence in the validation-harness/root-validator layer. It composes retrieval-result-use-summary-manifest-readiness and retrieval-result-use-summary-manifest-admission evidence into one result-use-summary-manifest-approved/not-approved retrieval boundary using ready admitted summary-manifest evidence only, without reading or writing retrieval storage, executing retrieval queries, approving runtime results, promoting policy, or training a model.

This slice answers:

```text
Can an evaluator inspect one deterministic receipt that approves ready admitted summary-manifest retrieval result-use evidence for later retrieval gates without reading or writing retrieval storage, executing a retrieval query, approving runtime results, promoting policy, training a model, executing batches, or changing kernel authority?
```

Implemented surfaces:

```text
PolicyReuseEvidenceRetrievalResultUseSummaryManifestApprovalReceipt
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_smoke_receipt()
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-smoke
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-regression-smoke
```

Implemented fields:

```text
schema
record_type
retrieval_result_use_summary_manifest_approval_version
source_retrieval_result_use_summary_manifest_readiness_hash
source_retrieval_result_use_summary_manifest_admission_hash
retrieval_result_use_summary_manifest_ready_for_use
retrieval_result_use_summary_manifest_admitted
retrieval_read_performed
retrieval_write_performed
retrieval_query_executed
runtime_result_approval_performed
policy_promotion_performed
student_training_performed
external_result_evidence_present
summary_manifest_approval_policy_reuse_examples
summary_manifest_approval_llm_fallback_examples
retrieval_result_use_summary_manifest_approved
result_use_summary_manifest_approval_status
not_approved_reason
result_use_summary_manifest_approval_hash
receipt_hash
```

Completed semantics:

- `retrieval_result_use_summary_manifest_approved = true` only when retrieval-result-use-summary-manifest-readiness passed, retrieval-result-use-summary-manifest-admission passed, external result evidence is present, retrieval storage was not read or written, retrieval query execution did not happen, runtime result approval did not happen, policy was not promoted, student training was not performed, summary-manifest-approval policy-reuse examples are positive, and `not_approved_reason = "none"`.
- Healthy evidence binds to retrieval-result-use-summary-manifest-readiness and retrieval-result-use-summary-manifest-admission receipt hashes, reports result use summary manifest approval status `result_use_summary_manifest_approved`, and records `not_approved_reason = "none"`.
- Regression evidence remains structurally valid while exposing `retrieval_result_use_summary_manifest_ready_for_use = false`, `retrieval_result_use_summary_manifest_admitted = false`, result use summary manifest approval status `result_use_summary_manifest_not_approved`, and `not_approved_reason = "summary_manifest_not_ready_for_use"`.
- The receipt is evidence-only and does not execute batches, change kernel authority, promote policy, read/write retrieval storage, execute retrieval queries, approve runtime retrieval results, train models, or alter runtime behavior.
- The receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence For Latest Baseline

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract retrieval_result_use_summary_manifest_approval --no-run --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test planning_contract --test score_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract retrieval_result_use_summary_manifest_approval --quiet

cargo fmt --check: pass
validation_harness_contract retrieval_result_use_summary_manifest_approval --no-run: pass
cargo check --quiet: pass
planning_contract and score_contract: pass
retrieval-result-use-summary-manifest-approval focused executable tests: attempted, but connector returned 502 before a Rust result was available
```

## Current Planning Decision

Keep the next implementation turn focused on **Learning**, moving from retrieval-result-use-summary-manifest-approval evidence toward deterministic retrieval-result-use-summary-manifest-approval-admission evidence.

Current gap:

```text
Retrieval-result-use-summary-manifest-approval evidence is now explicit, but the stack still lacks one deterministic retrieval-result-use-summary-manifest-approval-admission receipt that admits approved summary-manifest evidence for later retrieval gates without reading or writing retrieval storage or approving runtime results.
```

Recommended next slice:

```text
Add a deterministic policy reuse evidence retrieval-result-use-summary-manifest-approval-admission receipt that composes retrieval-result-use-summary-manifest-approval and retrieval-result-use-summary-manifest-readiness evidence into result-use-summary-manifest-approval-admitted/not-admitted evidence using approved summary-manifest evidence only.
```

Recommended concrete surfaces:

```text
PolicyReuseEvidenceRetrievalResultUseSummaryManifestApprovalAdmissionReceipt
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_smoke_receipt()
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-smoke
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse retrieval-result-use-summary-manifest-approval and retrieval-result-use-summary-manifest-readiness receipt hashes.
4. Keep the receipt evidence-only: no retrieval reads/writes, query execution, policy promotion, student training, or batch execution.
5. Include healthy and controlled summary-manifest-not-approved regression cases.
6. Keep actual retrieval storage and student-model training deferred.
7. Update external CLI mode fixtures and guarded validation counts only if new public modes or tests are added.
8. Keep unrelated `canon-rustc-v3/` working-tree changes out of this slice unless explicitly selected in a separate turn.

## Implementation Turn Commit Scope

This turn implemented retrieval-result-use-summary-manifest-approval. It should update and commit:

```text
plan.md
score.md
src/validation_harness.rs
src/bin/root_validate.rs
tests/validation_harness_contract.rs
tests/fixtures/external_agent_cli_modes.txt
```

Observed `canon-rustc-v3/` working-tree changes remain outside this turn unless a later implementation turn explicitly selects them.

## Acceptance Criteria For Next Implementation Turn

1. A deterministic healthy retrieval-result-use-summary-manifest-approval-admission receipt exists and validates successfully.
2. A deterministic regression retrieval-result-use-summary-manifest-approval-admission receipt exists and exposes a concrete summary-manifest-not-approved reason.
3. Retrieval-result-use-summary-manifest-approval-admission evidence references retrieval-result-use-summary-manifest-approval and retrieval-result-use-summary-manifest-readiness receipts instead of adding policy authority.
4. Root validator compact modes expose healthy and regression retrieval-result-use-summary-manifest-approval-admission receipts.
5. Validation harness contract tests assert retrieval-result-use-summary-manifest-approval-admission semantics, source binding, compact output, and controlled failing evidence.
6. Planning and score contract tests pass after documentation updates.
7. The receipt remains evidence-only and does not expand kernel authority, promote policy, execute batches, read/write retrieval storage, execute retrieval queries, or train a student model.

## Evaluation Axes

```text
I  = Intelligence
E  = Efficiency
C  = Correctness
A  = Alignment
R  = Robustness
P  = Performance
S  = Scalability
D  = Determinism
T  = Transparency
Co = Collaboration
Em = Empowerment
B  = Benefit
L  = Learning
St = Structure
Si = Simplicity
F  = Future-Proofing
```

```text
G = (I*E*C*A*R*P*S*D*T*Co*Em*B*L*St*Si*F)^(1/16)
arg max(G) = good
```


## Planning Turn Handoff - Retrieval Result Use Summary Manifest Approval Admission

```text
turn_type = planning_only
selected_axis = Learning
implementation_baseline = retrieval-result-use-summary-manifest-approval evidence is present at commit 6beaaea
current_gap = approved result-use summary-manifest evidence is not yet admitted for later retrieval gates
next_action = implement retrieval-result-use-summary-manifest-approval-admission evidence in the validation harness and root validator
commit_scope = plan.md and score.md only
out_of_scope_changes_preserved = existing canon-rustc-v3 working-tree changes remain unowned by this planning turn
```

Recommended next implementation slice:

```text
Add deterministic policy reuse evidence retrieval-result-use-summary-manifest-approval-admission receipts that admit approved summary-manifest evidence for later gates while remaining evidence-only.
```

Concrete implementation targets:

```text
PolicyReuseEvidenceRetrievalResultUseSummaryManifestApprovalAdmissionReceipt
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_smoke_receipt()
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-smoke
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-regression-smoke
```

Required semantics:

1. Healthy evidence reports `retrieval_result_use_summary_manifest_approval_admitted = true` only when retrieval-result-use-summary-manifest-approval is approved and the source readiness/admission evidence remains valid.
2. Regression evidence remains structurally valid but reports not admitted when the approval evidence is not approved, with a concrete reason such as `summary_manifest_not_approved`.
3. Evidence binds to retrieval-result-use-summary-manifest-approval, retrieval-result-use-summary-manifest-readiness, and retrieval-result-use-summary-manifest-admission receipt hashes.
4. The receipt must not execute retrieval queries, read or write retrieval storage, approve runtime retrieval results, promote policy, execute batches, call an LLM, use network/wall-clock/environment measurements, or train a student model.
5. Root validator compact modes and validation harness contracts must cover healthy and regression evidence.
6. Update guarded CLI-mode fixtures only if public root-validate modes are added.
7. Keep unrelated `canon-rustc-v3/` modifications out of the implementation slice unless explicitly selected later.

Acceptance criteria for the next implementation turn:

1. Healthy approval-admission receipt exists and passes focused validation.
2. Regression approval-admission receipt exists and exposes `summary_manifest_not_approved` or an equally explicit non-admission reason.
3. Contract tests assert source-hash binding, booleans, status strings, compact output, and controlled failing evidence.
4. `cargo fmt --check`, `cargo check --quiet`, planning/score contracts, and focused validation harness tests pass or any connector failure is recorded precisely.
5. Kernel authority and runtime retrieval behavior remain unchanged.


## Implementation Step 1 - Retrieval Result Use Summary Manifest Approval Admission

Completed this turn:

```text
PolicyReuseEvidenceRetrievalResultUseSummaryManifestApprovalAdmissionReceipt
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_smoke_receipt()
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-smoke
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-regression-smoke
```

Completed semantics:

- `retrieval_result_use_summary_manifest_approval_admitted = true` only when retrieval-result-use-summary-manifest-approval passed, retrieval-result-use-summary-manifest-readiness passed, retrieval-result-use-summary-manifest-admission passed, external result evidence is present, retrieval storage was not read or written, retrieval query execution did not happen, runtime result approval did not happen, policy was not promoted, student training was not performed, approval-admission policy-reuse examples are positive, and `not_admitted_reason = "none"`.
- Healthy evidence binds to retrieval-result-use-summary-manifest-approval, retrieval-result-use-summary-manifest-readiness, and retrieval-result-use-summary-manifest-admission receipt hashes, reports approval-admission status `result_use_summary_manifest_approval_admitted`, and records `not_admitted_reason = "none"`.
- Regression evidence remains structurally valid while exposing `retrieval_result_use_summary_manifest_approved = false`, `retrieval_result_use_summary_manifest_ready_for_use = false`, `retrieval_result_use_summary_manifest_admitted = false`, approval-admission status `result_use_summary_manifest_approval_not_admitted`, and `not_admitted_reason = "summary_manifest_not_approved"`.
- The receipt is evidence-only and does not execute batches, change kernel authority, promote policy, read/write retrieval storage, execute retrieval queries, approve runtime retrieval results, train models, or alter runtime behavior.
- The receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence For Implementation Step 1

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract retrieval_result_use_summary_manifest_approval_admission --no-run --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test planning_contract --test score_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract retrieval_result_use_summary_manifest_approval_admission --quiet

cargo fmt --check: pass
validation_harness_contract retrieval_result_use_summary_manifest_approval_admission --no-run: pass
cargo check --quiet: pass after retry
planning_contract and score_contract: pass
retrieval-result-use-summary-manifest-approval-admission focused executable tests: attempted twice, but connector returned 502 before a Rust result was available
```

## Current Planning Decision After Implementation Step 1

Keep the next implementation turn focused on **Learning**, moving from admitted approval-summary-manifest evidence toward a deterministic downstream retrieval gate that can consume approval-admission evidence without reading or writing retrieval storage or approving runtime results.

Current gap:

```text
Retrieval-result-use-summary-manifest-approval-admission evidence is now explicit, but the stack still lacks a downstream evidence gate that consumes admitted approval-summary-manifest evidence for later retrieval/model-learning decisions while remaining evidence-only.
```

Recommended next slice:

```text
Add the next deterministic policy reuse evidence receipt that consumes retrieval-result-use-summary-manifest-approval-admission evidence and prepares a later retrieval/model-learning boundary without performing retrieval storage operations, query execution, policy promotion, runtime result approval, or student training.
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse retrieval-result-use-summary-manifest-approval-admission receipt hashes rather than creating authority.
4. Keep the receipt evidence-only: no retrieval reads/writes, query execution, policy promotion, student training, runtime result approval, or batch execution.
5. Include healthy and controlled regression cases.
6. Update external CLI mode fixtures and guarded validation counts only if new public modes or tests are added.
7. Keep unrelated `canon-rustc-v3/` working-tree changes out of this slice unless explicitly selected in a separate turn.

## Implementation Step 1 Commit Scope

This turn should update and commit:

```text
plan.md
score.md
src/validation_harness.rs
src/bin/root_validate.rs
tests/validation_harness_contract.rs
tests/fixtures/external_agent_cli_modes.txt
```

Observed `canon-rustc-v3/` working-tree changes remain outside this turn unless a later implementation turn explicitly selects them.


## Planning Turn - Downstream Approval Admission Consumption Gate

This is a planning-only turn. It does not claim new implementation evidence beyond the staged retrieval-result-use-summary-manifest-approval-admission work already present in the working tree.

Current implementation baseline:

```text
Latest completed owned slice: retrieval-result-use-summary-manifest-approval-admission evidence.
Current strongest Learning boundary: approved, ready-for-use, admitted summary-manifest evidence can be admitted into a deterministic approval-admission receipt.
Observed staged implementation files: src/validation_harness.rs, src/bin/root_validate.rs, tests/validation_harness_contract.rs, tests/fixtures/external_agent_cli_modes.txt, plan.md, score.md.
Observed unrelated working-tree files: canon-rustc-v3/* modified/untracked files remain outside this plan.
```

Next implementation objective:

```text
Add a deterministic policy reuse evidence receipt that consumes retrieval-result-use-summary-manifest-approval-admission evidence and emits a downstream approval-admission consumption gate for later retrieval/model-learning decisions.
```

Recommended name shape:

```text
PolicyReuseEvidenceRetrievalResultUseSummaryManifestApprovalAdmissionConsumptionReceipt
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_smoke_receipt()
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-smoke
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-regression-smoke
```

Required semantics for the next slice:

1. Bind the new receipt to `result_use_summary_manifest_approval_admission_hash` and source approval/readiness/admission hashes from the approval-admission receipt.
2. Set the healthy consumption gate true only when approval-admission is admitted, upstream approval/readiness/admission booleans are true, external result evidence is present, and forbidden side-effect booleans remain false.
3. Preserve evidence-only behavior: no retrieval storage reads/writes, no retrieval query execution, no runtime result approval, no policy promotion, no batch execution, no live LLM call, no network call, no wall-clock dependency, and no student training.
4. Provide a controlled regression path where approval-admission is not admitted and the new receipt remains structurally valid with a deterministic not-consumed status and reason.
5. Keep the kernel, transition table, runtime reducer, durable writer, and command ledger untouched.
6. Add compact `root_validate` modes only for the two new receipts, and update `tests/fixtures/external_agent_cli_modes.txt` if public modes are added.
7. Add focused contract tests for hash binding, booleans, status strings, reason strings, compact output, and forbidden side effects.
8. Run validation in this order where possible: `cargo fmt --check`, focused no-run test compile, `cargo check --quiet`, planning/score contracts, focused executable validation harness tests.

Acceptance criteria:

```text
Healthy status: result_use_summary_manifest_approval_admission_consumed
Regression status: result_use_summary_manifest_approval_admission_not_consumed
Healthy reason: none
Regression reason: summary_manifest_approval_admission_not_admitted
No kernel authority drift.
No runtime retrieval behavior change.
No ownership of canon-rustc-v3 changes.
```

Commit discipline for the next implementation turn:

```text
Commit implementation/test/fixture files only if they belong to the new consumption gate.
Do not include canon-rustc-v3 modified or untracked files.
Do not include planning-only edits from this turn unless they are intentionally refreshed after implementation evidence is produced.
```


## Implementation Step 1 - Retrieval Result Use Summary Manifest Approval Admission Consumption

Completed this turn:

```text
PolicyReuseEvidenceRetrievalResultUseSummaryManifestApprovalAdmissionConsumptionReceipt
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_smoke_receipt()
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_admission_consumption_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-smoke
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-admission-consumption-regression-smoke
```

Completed semantics:

- `retrieval_result_use_summary_manifest_approval_admission_consumed = true` only when approval-admission is admitted, upstream approval/readiness/admission booleans are true, external result evidence is present, retrieval storage was not read or written, retrieval query execution did not happen, runtime result approval did not happen, policy was not promoted, batch execution did not happen, student training was not performed, consumption policy-reuse examples are positive, and `not_consumed_reason = "none"`.
- Healthy evidence binds to retrieval-result-use-summary-manifest-approval-admission receipt hash plus upstream approval, readiness, and admission source hashes, reports status `result_use_summary_manifest_approval_admission_consumed`, and records `not_consumed_reason = "none"`.
- Regression evidence remains structurally valid while exposing `retrieval_result_use_summary_manifest_approval_admitted = false`, upstream approval/readiness/admission booleans false, consumption status `result_use_summary_manifest_approval_admission_not_consumed`, and `not_consumed_reason = "summary_manifest_approval_admission_not_admitted"`.
- The receipt remains evidence-only: no retrieval reads/writes, no query execution, no runtime result approval, no policy promotion, no batch execution, no live LLM, no network call, no wall-clock dependency, and no student training.
- The kernel, transition table, runtime reducer, durable writer, and command ledger were not changed.

## Validation Evidence For Approval Admission Consumption

```text
cargo fmt --check: initially failed on formatting-only drift, then passed after cargo fmt
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract approval_admission_consumption --no-run --quiet: pass
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --quiet: pass
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test planning_contract --test score_contract --quiet: pass
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract approval_admission_consumption --quiet: attempted twice, but connector returned 502 before Rust test output was available
```

## Current Planning Decision After Approval Admission Consumption

Keep the next implementation turn focused on **Learning**, moving from consumed approval-admission summary-manifest evidence toward the next deterministic retrieval/model-learning boundary that can evaluate whether consumed evidence should become retrieval example material without performing storage operations or training.

Current gap:

```text
Approval-admission evidence can now be consumed by a deterministic evidence-only gate, but the stack still lacks the next admission boundary that decides whether consumed evidence is eligible to become retrieval-example learning material.
```

Recommended next slice:

```text
Add a deterministic policy reuse evidence receipt that consumes retrieval-result-use-summary-manifest-approval-admission-consumption evidence and emits an evidence-only retrieval-example learning eligibility gate.
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse approval-admission-consumption receipt hashes rather than creating authority.
4. Keep the receipt evidence-only: no retrieval storage reads/writes, no query execution, no policy promotion, no runtime result approval, no batch execution, and no student training.
5. Include healthy and controlled regression cases.
6. Update external CLI mode fixtures and guarded validation counts only if new public modes or tests are added.
7. Keep unrelated `canon-rustc-v3/` working-tree changes out of this slice unless explicitly selected in a separate turn.

## Implementation Step 1 Commit Scope - Approval Admission Consumption

This turn should update and commit:

```text
plan.md
score.md
src/validation_harness.rs
src/bin/root_validate.rs
tests/validation_harness_contract.rs
tests/fixtures/external_agent_cli_modes.txt
```

Observed `canon-rustc-v3/` working-tree changes remain outside this turn.
