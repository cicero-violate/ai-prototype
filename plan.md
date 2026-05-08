# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation plan after implementation step 1 of the current agent loop.

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, external CLI mode evidence, and evidence-surface index coverage.

## Current Completed Implementation Baseline

The current working tree contains deterministic **policy reuse evidence validation-budget** evidence in the validation-harness/root-validator layer, while retaining evaluator-savings, scaling-projection, distillation-readiness, evidence-surface index, evidence-bundle, evidence-quickcheck, evidence-maturity, evidence-summary, and evidence-manifest evidence already present in this loop.

This slice answers:

```text
Can an evaluator inspect a deterministic receipt that states the minimum targeted validation budget for proving the policy-reuse summary/manifest stack instead of defaulting to full validation-harness execution?
```

Implemented surfaces:

```text
PolicyReuseEvidenceValidationBudgetReceipt
policy_reuse_evidence_validation_budget_smoke_receipt()
policy_reuse_evidence_validation_budget_regression_smoke_receipt()
--policy-reuse-evidence-validation-budget-smoke
--policy-reuse-evidence-validation-budget-regression-smoke
```

Implemented fields:

```text
schema
record_type
budget_version
source_manifest_hash
source_summary_hash
targeted_command_count
targeted_test_count
max_targeted_test_count
full_harness_test_count
avoided_full_harness_tests
manifest_complete
budget_within_limit
budget_status
regression_reason
budget_hash
receipt_hash
```

Completed implementation tasks:

1. Added `PolicyReuseEvidenceValidationBudgetReceipt` with deterministic validation, JSON output, budget hash, and receipt hash.
2. Added healthy and controlled budget-exceeded regression smoke constructors.
3. Bound validation-budget evidence to evidence-manifest and evidence-summary receipt hashes.
4. Added root validator compact modes for healthy and regression validation-budget receipts.
5. Added validation harness contracts for validation-budget semantics, source binding, compact output, and controlled budget-exceeded evidence.
6. Updated external CLI mode fixture and root compact-mode counts for the two new public modes.
7. Updated validation harness expected test count from 180 to 184.
8. Updated retained guarded-test fixture values from 190 to 194 and refreshed dependent validation-duration estimates.
9. Kept kernel authority unchanged and did not train a student model or promote policy.

Implemented deterministic semantics:

- `budget_status = "pass"` only when the source manifest is complete, the targeted validation test count is within the deterministic limit, and `regression_reason = "none"`.
- Healthy evidence records two targeted commands, four targeted tests, a maximum targeted-test budget of four, the full harness count of 184, and 180 avoided full-harness tests.
- Regression evidence remains structurally valid while exposing `budget_status = "fail"`, `budget_within_limit = false`, and `regression_reason = "budget_exceeded"`.
- Validation-budget source evidence reuses manifest and summary receipt hashes instead of adding policy authority.
- The receipt is evidence-only; it summarizes validation cost without changing kernel authority, policy promotion, or runtime behavior.
- The new receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence Recorded For This Baseline

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract policy_reuse_evidence_validation_budget --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet

cargo fmt --check: pass
validation_harness_contract policy_reuse_evidence_validation_budget filter: 4 passed, 0 failed, 180 filtered out
validation_harness_contract: 184 passed, 0 failed
planning_contract: 2 passed, 0 failed
score_contract: 5 passed, 0 failed
```

Planning and score contract validation passed after this document update.

## Planned Next Implementation Slice

Planning decision for the next turn: keep the next implementation slice focused on the weakest remaining axis, **Scalability**.

Current gap:

```text
The evidence stack now exposes coverage, manifest completeness, maturity, quickcheck, summary, and validation budget, but it still lacks a compact deterministic rollout-readiness receipt that decides whether the stack is ready to scale beyond single targeted validation slices.
```

Recommended next slice:

```text
Add a deterministic policy reuse evidence rollout-readiness receipt that composes validation-budget, manifest, maturity, and summary evidence into one evaluator-facing readiness verdict.
```

Recommended concrete surfaces:

```text
PolicyReuseEvidenceRolloutReadinessReceipt
policy_reuse_evidence_rollout_readiness_smoke_receipt()
policy_reuse_evidence_rollout_readiness_regression_smoke_receipt()
--policy-reuse-evidence-rollout-readiness-smoke
--policy-reuse-evidence-rollout-readiness-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse validation-budget, manifest, maturity, and summary receipt hashes.
4. Keep the receipt evidence-only: it may state readiness, but it must not promote policy or grant authority.
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
- Parallel orchestration execution should wait until larger-batch reuse, validation health, retained validation/runtime cost, catalog completeness, evaluator savings, scaling projection, distillation readiness, evidence-surface indexing, bundled evidence inspection, quickcheck validation, maturity staging, stable summary evidence, manifest coverage, validation-budget evidence, and rollout-readiness evidence are proven together.
- Full-suite validation should run after the rollout-readiness slice if fixture or CLI mode churn is broader than expected.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. During implementation, work in validation-harness/root-validator evidence only unless a stricter dependency is discovered.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.
