# Canon Agent Implementation Plan

This plan records the current planning state after the latest planning/scoring turn of the current agent loop.

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, external CLI mode evidence, and evidence-surface index coverage.

## Latest Completed Slice

The latest completed implementation slice is deterministic **policy reuse evidence retrieval-result-manifest** evidence in the validation-harness/root-validator layer. It composes retrieval-result-admission and retrieval-query-approval evidence into one result-manifest-ready/not-ready retrieval boundary using admitted external result evidence only, without reading or writing retrieval storage, executing retrieval queries, or approving runtime results.

This slice answers:

```text
Can an evaluator inspect one deterministic receipt that summarizes admitted retrieval result evidence into a manifest-ready boundary without reading or writing retrieval storage, executing a retrieval query, approving runtime results, promoting policy, training a model, executing batches, or changing kernel authority?
```

Implemented surfaces:

```text
PolicyReuseEvidenceRetrievalResultManifestReceipt
policy_reuse_evidence_retrieval_result_manifest_smoke_receipt()
policy_reuse_evidence_retrieval_result_manifest_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-result-manifest-smoke
--policy-reuse-evidence-retrieval-result-manifest-regression-smoke
```

Implemented fields:

```text
schema
record_type
retrieval_result_manifest_version
source_retrieval_result_admission_hash
source_retrieval_query_approval_hash
retrieval_result_admitted
retrieval_query_approved
retrieval_read_performed
retrieval_write_performed
retrieval_query_executed
runtime_result_approval_performed
policy_promotion_performed
student_training_performed
external_result_evidence_present
manifest_policy_reuse_examples
manifest_llm_fallback_examples
retrieval_result_manifest_ready
result_manifest_status
not_ready_reason
result_manifest_hash
receipt_hash
```

Completed semantics:

- `retrieval_result_manifest_ready = true` only when retrieval-result-admission passed, retrieval-query-approval passed, external result evidence is present, retrieval storage was not read or written, retrieval query execution did not happen, runtime result approval did not happen, policy was not promoted, student training was not performed, manifest policy-reuse examples are positive, and `not_ready_reason = "none"`.
- Healthy evidence binds to retrieval-result-admission and retrieval-query-approval receipt hashes, reports result manifest status `result_manifest_ready`, and records `not_ready_reason = "none"`.
- Regression evidence remains structurally valid while exposing `retrieval_result_admitted = false`, `retrieval_query_approved = false`, result manifest status `result_manifest_not_ready`, and `not_ready_reason = "result_not_admitted"`.
- The receipt is evidence-only and does not execute batches, change kernel authority, promote policy, read/write retrieval storage, execute retrieval queries, approve runtime retrieval results, train models, or alter runtime behavior.
- The receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence For Latest Baseline

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract policy_reuse_evidence_retrieval_result_manifest --no-run --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract root_validate_policy_reuse_evidence_retrieval_result_manifest --no-run --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test planning_contract --test score_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract policy_reuse_evidence_retrieval_result_manifest_smoke_summarizes_admitted_result --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract policy_reuse_evidence_retrieval_result_manifest_regression_smoke_is_valid_not_ready_evidence --quiet

cargo fmt --check: pass
validation_harness_contract policy_reuse_evidence_retrieval_result_manifest --no-run: pass
validation_harness_contract root_validate_policy_reuse_evidence_retrieval_result_manifest --no-run: pass
cargo check --quiet: pass
planning_contract and score_contract: pass
retrieval-result-manifest focused executable tests: attempted, but connector returned 502 before a Rust result was available
```

## Current Planning Decision

Keep the next implementation turn focused on **Learning**, moving from retrieval-result-manifest evidence toward deterministic retrieval-result-use-admission evidence. This planning turn does not claim additional implementation progress; it refreshes the handoff target and scoring only.

Current gap:

```text
Retrieval-result-manifest evidence is now explicit, but the stack still lacks one deterministic retrieval-result-use-admission receipt that admits a ready result manifest for later retrieval use without reading or writing retrieval storage or approving runtime results.
```

Recommended next slice:

```text
Add a deterministic policy reuse evidence retrieval-result-use-admission receipt that composes retrieval-result-manifest and retrieval-result-admission evidence into result-use-admitted/not-admitted evidence using manifest-ready external result evidence only.
```

Recommended concrete surfaces:

```text
PolicyReuseEvidenceRetrievalResultUseAdmissionReceipt
policy_reuse_evidence_retrieval_result_use_admission_smoke_receipt()
policy_reuse_evidence_retrieval_result_use_admission_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-result-use-admission-smoke
--policy-reuse-evidence-retrieval-result-use-admission-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse retrieval-result-manifest and retrieval-result-admission receipt hashes.
4. Keep the receipt evidence-only: no retrieval reads/writes, query execution, policy promotion, student training, or batch execution.
5. Include healthy and controlled manifest-not-ready regression cases.
6. Keep actual retrieval storage and student-model training deferred.
7. Update external CLI mode fixtures and guarded validation counts only if new public modes or tests are added.
8. Keep unrelated `canon-rustc-v3/` working-tree changes out of this slice unless explicitly selected in a separate turn.

## Planning Turn Commit Scope

This turn is planning/scoring only. It should update and commit only:

```text
plan.md
score.md
```

Observed non-planning working-tree changes remain outside this turn unless a later implementation turn explicitly selects them.

## Acceptance Criteria For Next Implementation Turn

1. A deterministic healthy retrieval-result-use-admission receipt exists and validates successfully.
2. A deterministic regression retrieval-result-use-admission receipt exists and exposes a concrete manifest-not-ready reason.
3. Retrieval-result-use-admission evidence references retrieval-result-manifest and retrieval-result-admission receipts instead of adding policy authority.
4. Root validator compact modes expose healthy and regression retrieval-result-use-admission receipts.
5. Validation harness contract tests assert retrieval-result-use-admission semantics, source binding, compact output, and controlled failing evidence.
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
