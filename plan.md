# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation plan for the planning/scoring turn after implementation step 6 of the current agent loop.

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, external CLI mode evidence, and evidence-surface index coverage.

## Current Completed Implementation Baseline

The current working tree contains deterministic **policy reuse evidence-summary** evidence in the validation-harness/root-validator layer, while retaining evaluator-savings, scaling-projection, distillation-readiness, evidence-surface index, evidence-bundle, evidence-quickcheck, and evidence-maturity evidence already present in this loop.

This slice answers:

```text
Can an evaluator inspect one stable deterministic summary receipt that exposes the current healthy maturity result and key source hashes for the policy-reuse learning evidence stack?
```

Implemented surfaces:

```text
PolicyReuseEvidenceSummaryReceipt
policy_reuse_evidence_summary_smoke_receipt()
policy_reuse_evidence_summary_regression_smoke_receipt()
--policy-reuse-evidence-summary-smoke
--policy-reuse-evidence-summary-regression-smoke
```

Implemented fields:

```text
schema
record_type
summary_version
source_maturity_hash
source_quickcheck_hash
source_bundle_hash
maturity_stage
promotion_eligible
summary_status
evaluator_action
regression_reason
summary_hash
receipt_hash
```

Completed implementation tasks:

1. Added `PolicyReuseEvidenceSummaryReceipt` with deterministic validation, JSON output, summary hash, and receipt hash.
2. Added healthy and controlled immature-maturity regression smoke constructors.
3. Bound summary evidence to the policy reuse evidence maturity, quickcheck, and bundle receipts.
4. Added root validator compact modes for healthy and regression summary receipts.
5. Added validation harness contracts for summary semantics, source binding, compact output, and controlled immature-maturity evidence.
6. Updated external CLI mode fixture and root compact-mode counts for the two new public modes.
7. Updated validation harness guarded-test count from 172 to 176.
8. Updated retained guarded-test fixture values from 182 to 186 and refreshed dependent validation-duration estimates.
9. Kept kernel authority unchanged and did not train a student model or promote policy.

Implemented deterministic semantics:

- `summary_status = "pass"` only when promotion eligibility is true, maturity stage is `candidate`, evaluator action is `accept_summary`, and `regression_reason = "none"`.
- Regression evidence remains structurally valid while exposing `summary_status = "fail"`, `evaluator_action = "inspect_maturity"`, and `regression_reason = "maturity_immature"`.
- Summary source evidence reuses maturity, quickcheck, and bundle receipt hashes instead of adding policy authority.
- The summary receipt is evidence-only; it provides a stable evaluator-facing landing surface without granting policy authority.
- The new receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence Recorded For This Baseline

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract policy_reuse_evidence_summary --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet

cargo fmt --check: pass
validation_harness_contract policy_reuse_evidence_summary filter: 4 passed, 0 failed, 172 filtered out
validation_harness_contract: 176 passed, 0 failed
planning_contract: 2 passed, 0 failed
score_contract: 5 passed, 0 failed
```

Full-suite validation was not run before this planning/scoring update.

## Planned Next Implementation Slice

The weakest remaining axis is **Simplicity**.

Current gap:

```text
Policy-reuse learning evidence now has a stable summary receipt, but historical fixture/count churn still requires multiple retained files to be interpreted together.
```

Recommended next slice:

```text
Add a deterministic policy reuse evidence manifest receipt that lists the stable evaluator-facing evidence modes and fixture dependencies as one manifest.
```

Recommended concrete surfaces:

```text
PolicyReuseEvidenceManifestReceipt
policy_reuse_evidence_manifest_smoke_receipt()
policy_reuse_evidence_manifest_regression_smoke_receipt()
--policy-reuse-evidence-manifest-smoke
--policy-reuse-evidence-manifest-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse existing summary and maturity hashes.
4. Prefer one manifest of evaluator-facing evidence surfaces over additional scattered fixture interpretation.
5. Include healthy and controlled missing-summary-mode regression cases.
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
- Parallel orchestration execution should wait until larger-batch reuse, validation health, retained validation/runtime cost, catalog completeness, evaluator savings, scaling projection, distillation readiness, evidence-surface indexing, bundled evidence inspection, quickcheck validation, maturity staging, and stable summary evidence, and manifest coverage are proven together.
- Full-suite validation should run after the manifest slice if fixture or CLI mode churn is broader than expected.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Implement in capability/runtime-adjacent code without kernel authority drift.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.
