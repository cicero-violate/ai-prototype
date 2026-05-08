# Canon Agent Implementation Plan

This plan records the current implementation state after implementation step 2 of the current agent loop.

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, external CLI mode evidence, and evidence-surface index coverage.

## Latest Completed Slice

The latest completed implementation slice is deterministic **policy reuse evidence retrieval-example-index** evidence in the validation-harness/root-validator layer. It composes retrieval-example-admission and learning-data-admission evidence into one index/no-index boundary without writing retrieval storage.

This slice answers:

```text
Can an evaluator inspect one deterministic receipt that summarizes whether admitted retrieval examples may be indexed without writing retrieval storage, promoting policy, training a model, executing batches, or changing kernel authority?
```

Implemented surfaces:

```text
PolicyReuseEvidenceRetrievalExampleIndexReceipt
policy_reuse_evidence_retrieval_example_index_smoke_receipt()
policy_reuse_evidence_retrieval_example_index_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-example-index-smoke
--policy-reuse-evidence-retrieval-example-index-regression-smoke
```

Implemented fields:

```text
schema
record_type
retrieval_example_index_version
source_retrieval_example_admission_hash
source_learning_data_admission_hash
retrieval_example_admitted
learning_data_admitted
retrieval_write_performed
policy_promotion_performed
student_training_performed
indexed_policy_reuse_examples
indexed_llm_fallback_examples
retrieval_example_indexed
index_status
not_indexed_reason
index_hash
receipt_hash
```

Completed semantics:

- `retrieval_example_indexed = true` only when retrieval-example admission passed, learning-data admission passed, retrieval storage was not written, policy was not promoted, student training was not performed, indexed policy-reuse examples are positive, and `not_indexed_reason = "none"`.
- Healthy evidence binds to retrieval-example-admission and learning-data-admission receipt hashes, reports index status `indexed`, and records `not_indexed_reason = "none"`.
- Regression evidence remains structurally valid while exposing `retrieval_example_admitted = false`, `learning_data_admitted = false`, index status `not_indexed`, and `not_indexed_reason = "example_not_admitted"`.
- The receipt is evidence-only and does not execute batches, change kernel authority, promote policy, write retrieval storage, train models, or alter runtime behavior.
- The receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence For Latest Baseline

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract policy_reuse_evidence_retrieval_example_index --no-run --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test planning_contract --test score_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract root_validate_policy_reuse_evidence_retrieval_example_index --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo run --quiet --bin root_validate -- --policy-reuse-evidence-retrieval-example-index-smoke
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo run --quiet --bin root_validate -- --policy-reuse-evidence-retrieval-example-index-regression-smoke

cargo fmt --check: pass
validation_harness_contract policy_reuse_evidence_retrieval_example_index --no-run: pass
cargo check --quiet: pass
planning_contract and score_contract: pass
root_validate retrieval-example-index focused executable tests: attempted, but connector returned 502 before a Rust result was available
root_validate retrieval-example-index smoke mode: attempted, but connector returned 502 before a Rust result was available
root_validate retrieval-example-index regression mode: attempted, but connector returned 502 before a Rust result was available
```

## Current Planning Decision

Keep the next implementation turn focused on **Learning**, moving from retrieval-example indexing evidence toward deterministic retrieval-corpus-readiness evidence.

Current gap:

```text
Retrieval-example indexing is now summarized as evidence, but the stack still lacks one deterministic retrieval-corpus-readiness receipt that decides whether indexed examples are ready to serve as a retrieval corpus without reading or writing retrieval storage.
```

Recommended next slice:

```text
Add a deterministic policy reuse evidence retrieval-corpus-readiness receipt that composes retrieval-example-index and retrieval-example-admission evidence into ready/not-ready corpus evidence without performing retrieval storage operations.
```

Recommended concrete surfaces:

```text
PolicyReuseEvidenceRetrievalCorpusReadinessReceipt
policy_reuse_evidence_retrieval_corpus_readiness_smoke_receipt()
policy_reuse_evidence_retrieval_corpus_readiness_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-corpus-readiness-smoke
--policy-reuse-evidence-retrieval-corpus-readiness-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse retrieval-example-index and retrieval-example-admission receipt hashes.
4. Keep the receipt evidence-only: no retrieval reads/writes, policy promotion, student training, or batch execution.
5. Include healthy and controlled index-not-ready regression cases.
6. Keep actual retrieval storage and student-model training deferred.
7. Update external CLI mode fixtures and guarded validation counts only if new public modes or tests are added.
8. Keep unrelated `canon-rustc-v3/` working-tree changes out of this slice unless explicitly selected in a separate turn.

## Acceptance Criteria For Next Implementation Turn

1. A deterministic healthy retrieval-corpus-readiness receipt exists and validates successfully.
2. A deterministic regression retrieval-corpus-readiness receipt exists and exposes a concrete index-not-ready reason.
3. Retrieval-corpus-readiness evidence references retrieval-example-index and retrieval-example-admission receipts instead of adding policy authority.
4. Root validator compact modes expose healthy and regression retrieval-corpus-readiness receipts.
5. Validation harness contract tests assert retrieval-corpus-readiness semantics, source binding, compact output, and controlled failing evidence.
6. Planning and score contract tests pass after documentation updates.
7. The receipt remains evidence-only and does not expand kernel authority, promote policy, execute batches, read/write retrieval storage, or train a student model.

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
