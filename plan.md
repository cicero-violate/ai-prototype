# Canon Agent Implementation Plan

This plan records the current planning turn after implementation step 5. This turn does not implement runtime code; it updates the handoff plan and score for the next implementation slice.

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, external CLI mode evidence, and evidence-surface index coverage.

## Latest Completed Slice

The latest completed implementation slice is deterministic **policy reuse evidence learning-data-admission** evidence in the validation-harness/root-validator layer. It composes learning-candidate and external-evaluator-result evidence into one clean-dataset admission boundary.

This slice answers:

```text
Can an evaluator inspect one deterministic receipt that decides whether learning-candidate evidence may enter a clean dataset without promoting policy, writing retrieval storage, training a model, executing batches, or changing kernel authority?
```

Implemented surfaces:

```text
PolicyReuseEvidenceLearningDataAdmissionReceipt
policy_reuse_evidence_learning_data_admission_smoke_receipt()
policy_reuse_evidence_learning_data_admission_regression_smoke_receipt()
--policy-reuse-evidence-learning-data-admission-smoke
--policy-reuse-evidence-learning-data-admission-regression-smoke
```

Implemented fields:

```text
schema
record_type
data_admission_version
source_learning_candidate_hash
source_external_evaluator_result_hash
learning_candidate_ready
evaluator_result_passed
policy_promotion_performed
retrieval_write_performed
student_training_performed
admitted_batch_capacity
admitted_policy_reuse_cases
admitted_llm_fallback_cases
learning_data_admitted
admission_status
not_admitted_reason
admission_hash
receipt_hash
```

Completed semantics:

- `learning_data_admitted = true` only when learning-candidate passed, external-evaluator-result passed, policy promotion was not performed, retrieval storage was not written, student training was not performed, admitted policy-reuse cases are positive, and `not_admitted_reason = "none"`.
- Healthy evidence reuses learning-candidate and external-evaluator-result receipt hashes, reports admission status `admitted`, and records `not_admitted_reason = "none"`.
- Regression evidence remains structurally valid while exposing `learning_candidate_ready = false`, `evaluator_result_passed = false`, admission status `not_admitted`, and `not_admitted_reason = "candidate_not_ready"`.
- The receipt is evidence-only and does not execute batches, change kernel authority, promote policy, write retrieval storage, train models, or alter runtime behavior.
- The receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence For Latest Baseline

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract policy_reuse_evidence_learning_data_admission --no-run --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo run --quiet --bin root_validate -- --policy-reuse-evidence-learning-data-admission-smoke
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo run --quiet --bin root_validate -- --policy-reuse-evidence-learning-data-admission-regression-smoke

cargo fmt --check: pass
cargo check --quiet: pass
validation_harness_contract policy_reuse_evidence_learning_data_admission --no-run: pass
root_validate learning-data-admission smoke mode: attempted, but connector returned 502 before a Rust result was available
root_validate learning-data-admission regression mode: attempted, but connector returned 502 before a Rust result was available
```

## Current Planning Decision

Keep the next implementation turn focused on **Learning**, moving from clean dataset admission evidence to deterministic retrieval-example-admission evidence.

Current gap:

```text
Learning-data admission is explicit, but the evidence stack still lacks one deterministic retrieval-example-admission receipt that decides whether admitted learning data may become retrieval examples without writing retrieval storage.
```

Recommended next slice:

```text
Add a deterministic policy reuse evidence retrieval-example-admission receipt that composes learning-data-admission and learning-candidate evidence into example-admitted/not-admitted evidence.
```

Recommended concrete surfaces:

```text
PolicyReuseEvidenceRetrievalExampleAdmissionReceipt
policy_reuse_evidence_retrieval_example_admission_smoke_receipt()
policy_reuse_evidence_retrieval_example_admission_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-example-admission-smoke
--policy-reuse-evidence-retrieval-example-admission-regression-smoke
```

Recommended receipt fields:

```text
schema
record_type
retrieval_example_admission_version
source_learning_data_admission_hash
source_learning_candidate_hash
learning_data_admitted
learning_candidate_ready
retrieval_write_performed
policy_promotion_performed
student_training_performed
example_policy_reuse_cases
example_llm_fallback_cases
retrieval_example_admitted
admission_status
not_admitted_reason
admission_hash
receipt_hash
```

Recommended deterministic semantics:

- `retrieval_example_admitted = true` only when learning data is admitted, the source learning candidate is ready, retrieval storage was not written, policy was not promoted, student training was not performed, example policy-reuse cases are positive, and `not_admitted_reason = "none"`.
- Healthy evidence should bind to the learning-data-admission and learning-candidate receipt hashes.
- Regression evidence should remain structurally valid while exposing `learning_data_admitted = false`, admission status `not_admitted`, and `not_admitted_reason = "data_not_admitted"`.
- The receipt may decide retrieval-example admission, but must not write retrieval storage.

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse learning-data-admission and learning-candidate receipt hashes.
4. Keep the receipt evidence-only: no retrieval writes, policy promotion, student training, or batch execution.
5. Include healthy and controlled data-not-admitted regression cases.
6. Keep student-model training deferred.
7. Update external CLI mode fixtures and guarded validation counts only if new public modes or tests are added.
8. Keep unrelated `canon-rustc-v3/` working-tree changes out of this slice unless explicitly selected in a separate turn.

## Acceptance Criteria For Next Implementation Turn

1. A deterministic healthy retrieval-example-admission receipt exists and validates successfully.
2. A deterministic regression retrieval-example-admission receipt exists and exposes a concrete data-not-admitted reason.
3. Retrieval-example-admission evidence references learning-data-admission and learning-candidate receipts instead of adding policy authority.
4. Root validator compact modes expose healthy and regression retrieval-example-admission receipts.
5. Validation harness contract tests assert retrieval-example-admission semantics, source binding, compact output, and controlled failing evidence.
6. Planning and score contract tests pass after documentation updates.
7. The receipt remains evidence-only and does not expand kernel authority, promote policy, execute batches, write retrieval storage, or train a student model.

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
