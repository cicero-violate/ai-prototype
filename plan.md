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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, external CLI mode evidence, and evidence-surface index coverage.

## Latest Completed Slice

The latest completed implementation slice is deterministic **policy reuse evidence retrieval-corpus-admission** evidence in the validation-harness/root-validator layer. It composes retrieval-corpus-readiness and retrieval-example-index evidence into one admitted/not-admitted corpus boundary without reading or writing retrieval storage.

This slice answers:

```text
Can an evaluator inspect one deterministic receipt that decides whether a ready corpus may be admitted for retrieval use without reading or writing retrieval storage, promoting policy, training a model, executing batches, or changing kernel authority?
```

Implemented surfaces:

```text
PolicyReuseEvidenceRetrievalCorpusAdmissionReceipt
policy_reuse_evidence_retrieval_corpus_admission_smoke_receipt()
policy_reuse_evidence_retrieval_corpus_admission_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-corpus-admission-smoke
--policy-reuse-evidence-retrieval-corpus-admission-regression-smoke
```

Implemented fields:

```text
schema
record_type
retrieval_corpus_admission_version
source_retrieval_corpus_readiness_hash
source_retrieval_example_index_hash
retrieval_corpus_ready
retrieval_example_indexed
retrieval_read_performed
retrieval_write_performed
policy_promotion_performed
student_training_performed
admitted_policy_reuse_examples
admitted_llm_fallback_examples
retrieval_corpus_admitted
admission_status
not_admitted_reason
admission_hash
receipt_hash
```

Completed semantics:

- `retrieval_corpus_admitted = true` only when retrieval-corpus readiness passed, retrieval-example index passed, retrieval storage was not read or written, policy was not promoted, student training was not performed, admitted policy-reuse examples are positive, and `not_admitted_reason = "none"`.
- Healthy evidence binds to retrieval-corpus-readiness and retrieval-example-index receipt hashes, reports admission status `admitted`, and records `not_admitted_reason = "none"`.
- Regression evidence remains structurally valid while exposing `retrieval_corpus_ready = false`, `retrieval_example_indexed = false`, admission status `not_admitted`, and `not_admitted_reason = "corpus_not_ready"`.
- The receipt is evidence-only and does not execute batches, change kernel authority, promote policy, read/write retrieval storage, train models, or alter runtime behavior.
- The receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence For Latest Baseline

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract policy_reuse_evidence_retrieval_corpus_admission --no-run --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test planning_contract --test score_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract root_validate_policy_reuse_evidence_retrieval_corpus_admission --quiet

cargo fmt --check: pass
validation_harness_contract policy_reuse_evidence_retrieval_corpus_admission --no-run: pass
cargo check --quiet: pass
planning_contract and score_contract: pass
root_validate retrieval-corpus-admission focused executable tests: attempted, but connector returned 502 before a Rust result was available
```

## Current Planning Decision

Keep the next implementation turn focused on **Learning**, moving from retrieval-corpus-admission evidence toward deterministic retrieval-use-approval evidence.

Current gap:

```text
Retrieval-corpus admission is now explicit, but the stack still lacks one deterministic retrieval-use-approval receipt that decides whether an admitted corpus may be used for retrieval without reading or writing retrieval storage.
```

Recommended next slice:

```text
Add a deterministic policy reuse evidence retrieval-use-approval receipt that composes retrieval-corpus-admission and retrieval-corpus-readiness evidence into approved/not-approved retrieval-use evidence without performing retrieval storage operations.
```

Recommended concrete surfaces:

```text
PolicyReuseEvidenceRetrievalUseApprovalReceipt
policy_reuse_evidence_retrieval_use_approval_smoke_receipt()
policy_reuse_evidence_retrieval_use_approval_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-use-approval-smoke
--policy-reuse-evidence-retrieval-use-approval-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse retrieval-corpus-admission and retrieval-corpus-readiness receipt hashes.
4. Keep the receipt evidence-only: no retrieval reads/writes, policy promotion, student training, or batch execution.
5. Include healthy and controlled corpus-not-admitted regression cases.
6. Keep actual retrieval storage and student-model training deferred.
7. Update external CLI mode fixtures and guarded validation counts only if new public modes or tests are added.
8. Keep unrelated `canon-rustc-v3/` working-tree changes out of this slice unless explicitly selected in a separate turn.

## Acceptance Criteria For Next Implementation Turn

1. A deterministic healthy retrieval-use-approval receipt exists and validates successfully.
2. A deterministic regression retrieval-use-approval receipt exists and exposes a concrete corpus-not-admitted reason.
3. Retrieval-use-approval evidence references retrieval-corpus-admission and retrieval-corpus-readiness receipts instead of adding policy authority.
4. Root validator compact modes expose healthy and regression retrieval-use-approval receipts.
5. Validation harness contract tests assert retrieval-use-approval semantics, source binding, compact output, and controlled failing evidence.
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
