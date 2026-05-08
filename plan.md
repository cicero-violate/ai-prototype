# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation plan after implementation step 2 of the current agent loop.

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, external CLI mode evidence, and evidence-surface index coverage.

## Current Completed Implementation Baseline

The current working tree contains deterministic **policy reuse evidence batch-run-request** evidence in the validation-harness/root-validator layer, while retaining evaluator-savings, scaling-projection, distillation-readiness, evidence-surface index, evidence-bundle, evidence-quickcheck, evidence-maturity, evidence-summary, evidence-manifest, validation-budget, rollout-readiness, learning-admission, retrieval-readiness, compact-validation, batch-readiness, batch-execution-plan, and batch-evaluation-admission evidence already present in this loop. This implementation step 2 adds a deterministic no-execute request boundary that composes batch-evaluation-admission and batch-execution-plan evidence.

This completed slice answers:

```text
Can an evaluator inspect one deterministic receipt that records the next external batch-run request boundary after admission without executing the run, promoting policy, writing retrieval storage, training a model, or changing kernel authority?
```

Implemented surfaces:

```text
PolicyReuseEvidenceBatchRunRequestReceipt
policy_reuse_evidence_batch_run_request_smoke_receipt()
policy_reuse_evidence_batch_run_request_regression_smoke_receipt()
--policy-reuse-evidence-batch-run-request-smoke
--policy-reuse-evidence-batch-run-request-regression-smoke
```

Implemented fields:

```text
schema
record_type
request_version
source_batch_evaluation_admission_hash
source_batch_execution_plan_hash
batch_evaluation_admitted
plan_ready
no_execute_request
execution_performed
requested_batch_capacity
requested_policy_reuse_cases
requested_llm_fallback_cases
batch_request_ready
request_status
not_requestable_reason
request_hash
receipt_hash
```

Completed implementation tasks:

1. Added `PolicyReuseEvidenceBatchRunRequestReceipt` with deterministic validation, JSON output, request hash, and receipt hash.
2. Added healthy and controlled admission-not-granted regression smoke constructors.
3. Bound batch-run-request evidence to batch-evaluation-admission and batch-execution-plan receipt hashes.
4. Added root validator compact modes for healthy and regression batch-run-request receipts.
5. Added validation harness contracts for batch-run-request semantics, source binding, compact output, and controlled failing evidence.
6. Updated external CLI mode fixture for the two batch-run-request public modes and raised mode count from 83 to 85.
7. Updated validation harness expected test count from 212 to 216.
8. Kept kernel authority unchanged and did not execute batches, promote policy, write retrieval storage, train a student model, or alter live runtime behavior.

Implemented deterministic semantics:

- `batch_request_ready = true` only when batch-evaluation-admission passed, batch-execution-plan passed, the request remains explicitly no-execute, no execution was performed, requested policy-reuse cases are positive, and `not_requestable_reason = "none"`.
- Healthy evidence reuses batch-evaluation-admission and batch-execution-plan receipt hashes, reports request status `request_ready`, and records `not_requestable_reason = "none"`.
- Regression evidence remains structurally valid while exposing `batch_evaluation_admitted = false`, `plan_ready = false`, request status `not_requestable`, and `not_requestable_reason = "admission_not_granted"`.
- Batch-run-request source evidence reuses existing evidence receipt hashes instead of adding policy authority.
- The receipt is evidence-only; it records a no-execute external batch request boundary without executing batches, changing kernel authority, promoting policy, writing retrieval storage, training models, or altering runtime behavior.
- The new receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence Recorded For This Baseline

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo check --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract policy_reuse_evidence_batch_run_request --no-run --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo run --quiet --bin root_validate -- --policy-reuse-evidence-batch-run-request-smoke
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo run --quiet --bin root_validate -- --policy-reuse-evidence-batch-run-request-regression-smoke

cargo fmt --check: pass
cargo check --quiet: pass
validation_harness_contract policy_reuse_evidence_batch_run_request --no-run: pass
root_validate batch-run-request smoke mode: attempted, but connector returned 502 before a Rust result was available
root_validate batch-run-request regression mode: attempted, but connector returned 502 before a Rust result was available
combined root-mode validation attempt: attempted, but connector returned 502 before a Rust result was available
```

Batch-run-request compile/check validation passed. Direct root-mode execution was attempted, but the connector returned 502 before reporting Rust results.

## Planned Next Implementation Slice

Planning decision for the next implementation turn: keep the slice focused on the weakest remaining axis, **Scalability**, now from batch-run-request evidence toward deterministic external-evaluator-result evidence.

Current gap:

```text
Batch-run-request evidence is now explicit, but the evidence stack still lacks one deterministic external-evaluator-result receipt that records an externally evaluated batch outcome boundary without allowing the LLM to approve itself.
```

Recommended next slice:

```text
Add a deterministic policy reuse evidence external-evaluator-result receipt that composes batch-run-request and batch-evaluation-admission evidence into pass/fail evaluator-result evidence.
```

Recommended concrete surfaces:

```text
PolicyReuseEvidenceExternalEvaluatorResultReceipt
policy_reuse_evidence_external_evaluator_result_smoke_receipt()
policy_reuse_evidence_external_evaluator_result_regression_smoke_receipt()
--policy-reuse-evidence-external-evaluator-result-smoke
--policy-reuse-evidence-external-evaluator-result-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse batch-run-request and batch-evaluation-admission receipt hashes.
4. Keep the receipt evidence-only: it may state an external evaluator result, but it must not execute batches or promote policy.
5. Include healthy and controlled evaluator-failed regression cases.
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
- Parallel orchestration execution should wait until larger-batch reuse, validation health, retained validation/runtime cost, catalog completeness, evaluator savings, scaling projection, distillation readiness, evidence-surface indexing, bundled evidence inspection, quickcheck validation, maturity staging, stable summary evidence, manifest coverage, validation-budget evidence, rollout-readiness evidence, learning-admission evidence, retrieval-readiness evidence, compact-validation evidence, batch-readiness evidence, batch-execution-plan evidence, batch-evaluation-admission evidence, and batch-run-request evidence are proven together.
- Full-suite validation should run after the external-evaluator-result slice if connector stability allows or if fixture/CLI mode churn is broader than expected.

## Implementation Step 2 Decision

```text
turn_type = implementation_step_2
mode = implementation
selected_axis = Scalability
selected_slice = deterministic policy reuse evidence batch-run-request receipt completed
implementation_files_changed_this_turn = src/validation_harness.rs, src/bin/root_validate.rs, tests/validation_harness_contract.rs, tests/fixtures/external_agent_cli_modes.txt
existing_uncommitted_source_changes_observed = none in final status before commit
commit_scope = batch-run-request implementation, tests, fixture, plan.md, score.md
```

This implementation turn completed batch-run-request evidence in the validation-harness/root-validator layer and did not change kernel authority or runtime execution behavior.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Keep planning commits scoped to `plan.md` and `score.md` unless implementation work is explicitly requested.
4. During implementation, work in validation-harness/root-validator evidence only unless a stricter dependency is discovered.
5. Add deterministic contract tests.
6. Run targeted validation and record results.
7. Update `plan.md` and `score.md`.
8. Commit the turn.
