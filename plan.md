# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation plan after planning step 6 of the current agent loop.

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, external CLI mode evidence, and evidence-surface index coverage.

## Current Completed Implementation Baseline

The current working tree contains deterministic **policy reuse evidence compact-validation** evidence in the validation-harness/root-validator layer, while retaining evaluator-savings, scaling-projection, distillation-readiness, evidence-surface index, evidence-bundle, evidence-quickcheck, evidence-maturity, evidence-summary, evidence-manifest, validation-budget, rollout-readiness, learning-admission, and retrieval-readiness evidence already present in this loop. This planning turn does not change implementation code and intentionally preserves the existing source modifications in `canon-rustc-v3/src/graph.rs` and `canon-rustc-v3/src/hir.rs` for the implementation turn.

This completed slice answers:

```text
Can an evaluator inspect one deterministic receipt that summarizes the compact targeted validation footprint for the retrieval/learning admission chain without executing validation or promoting policy?
```

Implemented surfaces:

```text
PolicyReuseEvidenceCompactValidationReceipt
policy_reuse_evidence_compact_validation_smoke_receipt()
policy_reuse_evidence_compact_validation_regression_smoke_receipt()
--policy-reuse-evidence-compact-validation-smoke
--policy-reuse-evidence-compact-validation-regression-smoke
```

Implemented fields:

```text
schema
record_type
compact_validation_version
source_retrieval_readiness_hash
source_learning_admission_hash
source_validation_budget_hash
retrieval_ready
learning_data_admissible
validation_budget_passed
targeted_command_count
targeted_test_count
max_targeted_test_count
full_harness_test_count
avoided_full_harness_tests
compact_validation_passed
compact_validation_status
failure_reason
compact_hash
receipt_hash
```

Completed implementation tasks:

1. Added `PolicyReuseEvidenceCompactValidationReceipt` with deterministic validation, JSON output, compact hash, and receipt hash.
2. Added healthy and controlled retrieval-not-ready regression smoke constructors.
3. Bound compact-validation evidence to retrieval-readiness, learning-admission, and validation-budget receipt hashes.
4. Added root validator compact modes for healthy and regression compact-validation receipts.
5. Added validation harness contracts for compact-validation semantics, source binding, compact output, and controlled failing evidence.
6. Updated external CLI mode fixture for the two compact-validation public modes.
7. Updated validation harness expected test count from 196 to 200.
8. Updated retained guarded-test fixture values from 206 to 210 and refreshed dependent validation-duration/performance-cost estimates.
9. Kept kernel authority unchanged and did not execute validation, write retrieval storage, train a student model, or promote policy.

Implemented deterministic semantics:

- `compact_validation_passed = true` only when retrieval-readiness passed, learning-admission passed, validation-budget passed, targeted tests are within budget, and `failure_reason = "none"`.
- Healthy evidence records three targeted commands, six targeted tests, a maximum targeted-test budget of six, full harness count of 200, 194 avoided full-harness tests, status `pass`, and `failure_reason = "none"`.
- Regression evidence remains structurally valid while exposing `retrieval_ready = false`, `learning_data_admissible = false`, `validation_budget_passed = false`, targeted tests above budget, status `fail`, and `failure_reason = "retrieval_not_ready"`.
- Compact-validation source evidence reuses retrieval-readiness, learning-admission, and validation-budget receipt hashes instead of adding policy authority.
- The receipt is evidence-only; it summarizes targeted validation footprint without changing kernel authority, policy promotion, storage writes, model training, validation execution, or runtime behavior.
- The new receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence Recorded For This Baseline

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract policy_reuse_evidence_compact_validation --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet

cargo fmt --check: pass
validation_harness_contract policy_reuse_evidence_compact_validation filter: 4 passed, 0 failed, 196 filtered out
validation_harness_contract: 200 passed, 0 failed
planning_contract: 2 passed, 0 failed
score_contract: 5 passed, 0 failed
```

Planning and score contract validation passed for the previous compact-validation document update. This planning-only turn updates the next-slice handoff and score without changing implementation code.

## Planned Next Implementation Slice

Planning decision for the next implementation turn: keep the slice focused on the weakest remaining axis, **Scalability**.

Current gap:

```text
Compact validation is now explicit, but the evidence stack still lacks one deterministic batch-readiness receipt that summarizes whether the compact-validated retrieval/learning chain is ready for larger batch reuse evaluation.
```

Recommended next slice:

```text
Add a deterministic policy reuse evidence batch-readiness receipt that composes compact-validation, retrieval-readiness, and scaling-projection evidence into a batch-ready/not-ready verdict.
```

Recommended concrete surfaces:

```text
PolicyReuseEvidenceBatchReadinessReceipt
policy_reuse_evidence_batch_readiness_smoke_receipt()
policy_reuse_evidence_batch_readiness_regression_smoke_receipt()
--policy-reuse-evidence-batch-readiness-smoke
--policy-reuse-evidence-batch-readiness-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse compact-validation, retrieval-readiness, and scaling-projection receipt hashes.
4. Keep the receipt evidence-only: it may state batch readiness, but it must not execute batches or promote policy.
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
- Parallel orchestration execution should wait until larger-batch reuse, validation health, retained validation/runtime cost, catalog completeness, evaluator savings, scaling projection, distillation readiness, evidence-surface indexing, bundled evidence inspection, quickcheck validation, maturity staging, stable summary evidence, manifest coverage, validation-budget evidence, rollout-readiness evidence, learning-admission evidence, retrieval-readiness evidence, compact-validation evidence, and batch-readiness evidence are proven together.
- Full-suite validation should run after the batch-readiness slice if fixture or CLI mode churn is broader than expected.

## Planning Step 6 Decision

```text
turn_type = planning_step_6
mode = planning_and_scoring_only
selected_axis = Scalability
selected_slice = deterministic policy reuse evidence batch-readiness receipt
implementation_files_changed_this_turn = none
existing_uncommitted_source_changes_observed = canon-rustc-v3/src/graph.rs, canon-rustc-v3/src/hir.rs
commit_scope = plan.md and score.md only
```

The next implementation turn should add batch-readiness evidence only after reconciling the existing uncommitted `canon-rustc-v3` source changes. Those source changes are not part of this planning commit and should not be overwritten by documentation-only work.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Keep planning commits scoped to `plan.md` and `score.md` unless implementation work is explicitly requested.
4. During implementation, work in validation-harness/root-validator evidence only unless a stricter dependency is discovered.
5. Add deterministic contract tests.
6. Run targeted validation and record results.
7. Update `plan.md` and `score.md`.
8. Commit the turn.
