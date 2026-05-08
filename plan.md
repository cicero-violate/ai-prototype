# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation plan after implementation step 3 of the current agent loop.

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, external CLI mode evidence, and evidence-surface index coverage.

## Current Completed Implementation Baseline

The current working tree contains deterministic **policy reuse evidence learning-admission** evidence in the validation-harness/root-validator layer, while retaining evaluator-savings, scaling-projection, distillation-readiness, evidence-surface index, evidence-bundle, evidence-quickcheck, evidence-maturity, evidence-summary, evidence-manifest, validation-budget, and rollout-readiness evidence already present in this loop.

This slice answers:

```text
Can an evaluator inspect one deterministic receipt that admits or rejects a rollout-ready policy-reuse trace as learning data without promoting policy or training a model?
```

Implemented surfaces:

```text
PolicyReuseEvidenceLearningAdmissionReceipt
policy_reuse_evidence_learning_admission_smoke_receipt()
policy_reuse_evidence_learning_admission_regression_smoke_receipt()
--policy-reuse-evidence-learning-admission-smoke
--policy-reuse-evidence-learning-admission-regression-smoke
```

Implemented fields:

```text
schema
record_type
admission_version
source_rollout_readiness_hash
source_validation_budget_hash
source_summary_hash
rollout_ready
validation_budget_passed
summary_status
external_evidence_required
learning_data_admissible
admission_status
not_admissible_reason
admission_hash
receipt_hash
```

Completed implementation tasks:

1. Added `PolicyReuseEvidenceLearningAdmissionReceipt` with deterministic validation, JSON output, admission hash, and receipt hash.
2. Added healthy and controlled rollout-not-ready regression smoke constructors.
3. Bound learning-admission evidence to rollout-readiness, validation-budget, and summary receipt hashes.
4. Added root validator compact modes for healthy and regression learning-admission receipts.
5. Added validation harness contracts for learning-admission semantics, source binding, compact output, and controlled not-admissible evidence.
6. Updated external CLI mode fixture for the two learning-admission public modes.
7. Updated validation harness expected test count from 188 to 192.
8. Updated retained guarded-test fixture values from 198 to 202 and refreshed dependent validation-duration/performance-cost estimates.
9. Kept kernel authority unchanged and did not train a student model or promote policy.

Implemented deterministic semantics:

- `learning_data_admissible = true` only when rollout readiness passed, validation budget passed, summary status is `pass`, no external evidence is required, and `not_admissible_reason = "none"`.
- Healthy evidence records admission status `admissible` and `not_admissible_reason = "none"`.
- Regression evidence remains structurally valid while exposing `rollout_ready = false`, `validation_budget_passed = false`, `learning_data_admissible = false`, `admission_status = "not_admissible"`, and `not_admissible_reason = "rollout_not_ready"`.
- Learning-admission source evidence reuses rollout-readiness, validation-budget, and summary receipt hashes instead of adding policy authority.
- The receipt is evidence-only; it summarizes admissibility without changing kernel authority, policy promotion, model training, or runtime behavior.
- The new receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence Recorded For This Baseline

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract policy_reuse_evidence_learning_admission --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet

cargo fmt --check: pass
validation_harness_contract policy_reuse_evidence_learning_admission filter: 4 passed, 0 failed, 188 filtered out
validation_harness_contract: 192 passed, 0 failed
planning_contract: 2 passed, 0 failed
score_contract: 5 passed, 0 failed
```

Planning and score contract validation passed after this document update.

## Planned Next Implementation Slice

Planning decision for the next turn: keep the next implementation slice focused on the weakest remaining axis, **Intelligence**.

Current gap:

```text
Learning admission is now explicit, but admitted traces still do not have a deterministic retrieval-example readiness receipt for later case-based reuse.
```

Recommended next slice:

```text
Add a deterministic policy reuse evidence retrieval-readiness receipt that composes learning-admission, rollout-readiness, and summary evidence into a retrieval-example-ready/not-ready verdict.
```

Recommended concrete surfaces:

```text
PolicyReuseEvidenceRetrievalReadinessReceipt
policy_reuse_evidence_retrieval_readiness_smoke_receipt()
policy_reuse_evidence_retrieval_readiness_regression_smoke_receipt()
--policy-reuse-evidence-retrieval-readiness-smoke
--policy-reuse-evidence-retrieval-readiness-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse learning-admission, rollout-readiness, and summary receipt hashes.
4. Keep the receipt evidence-only: it may state retrieval readiness, but it must not write retrieval storage or promote policy.
5. Include healthy and controlled not-ready regression cases.
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
- Parallel orchestration execution should wait until larger-batch reuse, validation health, retained validation/runtime cost, catalog completeness, evaluator savings, scaling projection, distillation readiness, evidence-surface indexing, bundled evidence inspection, quickcheck validation, maturity staging, stable summary evidence, manifest coverage, validation-budget evidence, rollout-readiness evidence, learning-admission evidence, and retrieval-readiness evidence are proven together.
- Full-suite validation should run after the retrieval-readiness slice if fixture or CLI mode churn is broader than expected.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. During implementation, work in validation-harness/root-validator evidence only unless a stricter dependency is discovered.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.
