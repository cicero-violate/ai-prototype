# Canon Agent Implementation Plan

This plan records the current implementation state after implementation step 4 of the current agent loop.

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, external CLI mode evidence, and evidence-surface index coverage.

## Latest Completed Slice

The latest completed implementation slice is deterministic **policy reuse evidence retrieval-result-use-summary-manifest-readiness** evidence in the validation-harness/root-validator layer. It composes retrieval-result-use-summary-manifest-admission and retrieval-result-use-summary-manifest evidence into one result-use-summary-manifest-ready-for-use/not-ready-for-use retrieval boundary using admitted summary-manifest evidence only, without reading or writing retrieval storage, executing retrieval queries, approving runtime results, promoting policy, or training a model.

This slice answers:

```text
Can an evaluator inspect one deterministic receipt that confirms admitted summary-manifest retrieval result-use evidence is ready for later retrieval gates without reading or writing retrieval storage, executing a retrieval query, approving runtime results, promoting policy, training a model, executing batches, or changing kernel authority?
```

Implemented surfaces:

```text
PolicyReuseEvidenceRetrievalResultUseSummaryManifestReadinessReceipt
policy_reuse_evidence_retrieval_result_use_summary_manifest_readiness_smoke_receipt()
policy_reuse_evidence_retrieval_result_use_summary_manifest_readiness_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-result-use-summary-manifest-readiness-smoke
--policy-reuse-evidence-retrieval-result-use-summary-manifest-readiness-regression-smoke
```

Implemented fields:

```text
schema
record_type
retrieval_result_use_summary_manifest_readiness_version
source_retrieval_result_use_summary_manifest_admission_hash
source_retrieval_result_use_summary_manifest_hash
retrieval_result_use_summary_manifest_admitted
retrieval_result_use_summary_manifest_ready
retrieval_read_performed
retrieval_write_performed
retrieval_query_executed
runtime_result_approval_performed
policy_promotion_performed
student_training_performed
external_result_evidence_present
summary_manifest_readiness_policy_reuse_examples
summary_manifest_readiness_llm_fallback_examples
retrieval_result_use_summary_manifest_ready_for_use
result_use_summary_manifest_readiness_status
not_ready_reason
result_use_summary_manifest_readiness_hash
receipt_hash
```

Completed semantics:

- `retrieval_result_use_summary_manifest_ready_for_use = true` only when retrieval-result-use-summary-manifest-admission passed, retrieval-result-use-summary-manifest passed, external result evidence is present, retrieval storage was not read or written, retrieval query execution did not happen, runtime result approval did not happen, policy was not promoted, student training was not performed, summary-manifest-readiness policy-reuse examples are positive, and `not_ready_reason = "none"`.
- Healthy evidence binds to retrieval-result-use-summary-manifest-admission and retrieval-result-use-summary-manifest receipt hashes, reports result use summary manifest readiness status `result_use_summary_manifest_ready_for_use`, and records `not_ready_reason = "none"`.
- Regression evidence remains structurally valid while exposing `retrieval_result_use_summary_manifest_admitted = false`, `retrieval_result_use_summary_manifest_ready = false`, result use summary manifest readiness status `result_use_summary_manifest_not_ready_for_use`, and `not_ready_reason = "summary_manifest_not_admitted"`.
- The receipt is evidence-only and does not execute batches, change kernel authority, promote policy, read/write retrieval storage, execute retrieval queries, approve runtime retrieval results, train models, or alter runtime behavior.
- The receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence For Latest Baseline

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract retrieval_result_use_summary_manifest_readiness --no-run --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test planning_contract --test score_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract retrieval_result_use_summary_manifest_readiness --quiet

cargo fmt --check: pass
validation_harness_contract retrieval_result_use_summary_manifest_readiness --no-run: pass
cargo check --quiet: pass
planning_contract and score_contract: pass
retrieval-result-use-summary-manifest-readiness focused executable tests: attempted, but connector returned 502 before a Rust result was available
```

## Current Planning Decision

Keep the next implementation turn focused on **Learning**, moving from retrieval-result-use-summary-manifest-readiness evidence toward deterministic retrieval-result-use-summary-manifest-approval evidence.

Current gap:

```text
Retrieval-result-use-summary-manifest-readiness evidence is now explicit, but the stack still lacks one deterministic retrieval-result-use-summary-manifest-approval receipt that approves ready summary-manifest evidence for later retrieval gates without reading or writing retrieval storage or approving runtime results.
```

Recommended next slice:

```text
Add a deterministic policy reuse evidence retrieval-result-use-summary-manifest-approval receipt that composes retrieval-result-use-summary-manifest-readiness and retrieval-result-use-summary-manifest-admission evidence into result-use-summary-manifest-approved/not-approved evidence using ready admitted summary-manifest evidence only.
```

Recommended concrete surfaces:

```text
PolicyReuseEvidenceRetrievalResultUseSummaryManifestApprovalReceipt
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_smoke_receipt()
policy_reuse_evidence_retrieval_result_use_summary_manifest_approval_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-smoke
--policy-reuse-evidence-retrieval-result-use-summary-manifest-approval-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse retrieval-result-use-summary-manifest-readiness and retrieval-result-use-summary-manifest-admission receipt hashes.
4. Keep the receipt evidence-only: no retrieval reads/writes, query execution, policy promotion, student training, or batch execution.
5. Include healthy and controlled summary-manifest-not-ready-for-use regression cases.
6. Keep actual retrieval storage and student-model training deferred.
7. Update external CLI mode fixtures and guarded validation counts only if new public modes or tests are added.
8. Keep unrelated `canon-rustc-v3/` working-tree changes out of this slice unless explicitly selected in a separate turn.

## Implementation Turn Commit Scope

This turn implemented retrieval-result-use-summary-manifest-readiness. It should update and commit:

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

1. A deterministic healthy retrieval-result-use-summary-manifest-approval receipt exists and validates successfully.
2. A deterministic regression retrieval-result-use-summary-manifest-approval receipt exists and exposes a concrete summary-manifest-not-ready-for-use reason.
3. Retrieval-result-use-summary-manifest-approval evidence references retrieval-result-use-summary-manifest-readiness and retrieval-result-use-summary-manifest-admission receipts instead of adding policy authority.
4. Root validator compact modes expose healthy and regression retrieval-result-use-summary-manifest-approval receipts.
5. Validation harness contract tests assert retrieval-result-use-summary-manifest-approval semantics, source binding, compact output, and controlled failing evidence.
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
