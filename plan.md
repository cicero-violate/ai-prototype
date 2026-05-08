# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation plan after implementation step 5 of the current agent loop.

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

## Current Completed Implementation Baseline

The current working tree contains deterministic **policy reuse evidence learning-data-admission** evidence in the validation-harness/root-validator layer, while retaining evaluator-savings, scaling-projection, distillation-readiness, evidence-surface index, evidence-bundle, evidence-quickcheck, evidence-maturity, evidence-summary, evidence-manifest, validation-budget, rollout-readiness, learning-admission, retrieval-readiness, compact-validation, batch-readiness, batch-execution-plan, batch-evaluation-admission, batch-run-request, external-evaluator-result, and learning-candidate evidence already present in this loop. This implementation step 5 adds a deterministic clean-dataset admission boundary that composes learning-candidate and external-evaluator-result evidence without promoting policy, writing retrieval storage, or training a student model.

This completed slice answers:

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

Completed implementation tasks:

1. Added `PolicyReuseEvidenceLearningDataAdmissionReceipt` with deterministic validation, JSON output, admission hash, and receipt hash.
2. Added healthy and controlled candidate-not-ready regression smoke constructors.
3. Bound learning-data-admission evidence to learning-candidate and external-evaluator-result receipt hashes.
4. Added root validator compact modes for healthy and regression learning-data-admission receipts.
5. Added validation harness contracts for learning-data-admission semantics, source binding, compact output, and controlled failing evidence.
6. Updated external CLI mode fixture for the two learning-data-admission public modes and raised mode count from 89 to 91.
7. Updated validation harness expected test count from 224 to 228.
8. Kept kernel authority unchanged and did not execute batches, promote policy, write retrieval storage, train a student model, or alter live runtime behavior.

Implemented deterministic semantics:

- `learning_data_admitted = true` only when learning-candidate passed, external-evaluator-result passed, policy promotion was not performed, retrieval storage was not written, student training was not performed, admitted policy-reuse cases are positive, and `not_admitted_reason = "none"`.
- Healthy evidence reuses learning-candidate and external-evaluator-result receipt hashes, reports admission status `admitted`, and records `not_admitted_reason = "none"`.
- Regression evidence remains structurally valid while exposing `learning_candidate_ready = false`, `evaluator_result_passed = false`, admission status `not_admitted`, and `not_admitted_reason = "candidate_not_ready"`.
- Learning-data-admission source evidence reuses existing evidence receipt hashes instead of adding policy authority.
- The receipt is evidence-only; it records admitted/not-admitted clean dataset evidence without executing batches, changing kernel authority, promoting policy, writing retrieval storage, training models, or altering runtime behavior.
- The new receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence Recorded For This Baseline

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

Learning-data-admission compile/check validation passed. Direct root-mode execution was attempted, but the connector returned 502 before reporting Rust results.

## Planned Next Implementation Slice

Planning decision for the next implementation turn: keep the target on **Learning**, now from clean dataset admission evidence toward deterministic retrieval-example-admission evidence.

Current gap:

```text
Learning-data admission is now explicit, but the evidence stack still lacks one deterministic retrieval-example-admission receipt that decides whether admitted learning data may become retrieval examples without writing retrieval storage.
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

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse learning-data-admission and learning-candidate receipt hashes.
4. Keep the receipt evidence-only: it may state retrieval-example admission, but it must not write retrieval storage, promote policy, or train a student model.
5. Include healthy and controlled data-not-admitted regression cases.
6. Keep student-model training deferred.

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

## Deferred Work

- Full live Ollama validation remains environment-dependent.
- Graph telemetry still requires explicit wrapper capture.
- Student-model training is intentionally deferred until verified distillation data is large and clean.
- Parallel orchestration execution should wait until larger-batch reuse, validation health, retained validation/runtime cost, catalog completeness, evaluator savings, scaling projection, distillation readiness, evidence-surface indexing, bundled evidence inspection, quickcheck validation, maturity staging, stable summary evidence, manifest coverage, validation-budget evidence, rollout-readiness evidence, learning-admission evidence, retrieval-readiness evidence, compact-validation evidence, batch-readiness evidence, batch-execution-plan evidence, batch-evaluation-admission evidence, batch-run-request evidence, external-evaluator-result evidence, learning-candidate evidence, and learning-data-admission evidence are proven together.
- Full-suite validation should run after the retrieval-example-admission slice if connector stability allows or if fixture/CLI mode churn is broader than expected.

## Implementation Step 5 Decision

```text
turn_type = implementation_step_5
mode = implementation
selected_axis = Learning
selected_slice = deterministic policy reuse evidence learning-data-admission receipt completed
implementation_files_changed_this_turn = src/validation_harness.rs, src/bin/root_validate.rs, tests/validation_harness_contract.rs, tests/fixtures/external_agent_cli_modes.txt
existing_uncommitted_source_changes_observed = canon-rustc-v3/plan-autorefactor.md remains untracked and out of scope
commit_scope = learning-data-admission implementation, tests, fixture, plan.md, score.md
```

This implementation turn completed learning-data-admission evidence in the validation-harness/root-validator layer and did not change kernel authority or runtime execution behavior.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Keep planning commits scoped to `plan.md` and `score.md` unless implementation work is explicitly requested.
4. During implementation, work in validation-harness/root-validator evidence only unless a stricter dependency is discovered.
5. Add deterministic contract tests.
6. Run targeted validation and record results.
7. Update `plan.md` and `score.md`.
8. Commit the turn.
