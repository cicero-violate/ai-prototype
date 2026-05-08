# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation plan for the planning/scoring turn after implementation step 2 of the current agent loop.

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, external CLI mode evidence, and evidence-surface index coverage.

## Current Completed Implementation Baseline

The current working tree contains deterministic **policy reuse evidence-surface index** evidence in the validation-harness/root-validator layer, while retaining evaluator-savings, scaling-projection, and distillation-readiness evidence already present in this loop.

This slice answers:

```text
Can an evaluator inspect one deterministic receipt that says whether the policy-reuse learning evidence surface has the required receipt families, healthy root modes, regression root modes, and dependency groups?
```

Implemented surfaces:

```text
PolicyReuseEvidenceSurfaceIndexReceipt
policy_reuse_evidence_surface_index_smoke_receipt()
policy_reuse_evidence_surface_index_regression_smoke_receipt()
--policy-reuse-evidence-surface-index-smoke
--policy-reuse-evidence-surface-index-regression-smoke
```

Implemented fields:

```text
schema
record_type
index_version
evidence_family_count
healthy_mode_count
regression_mode_count
dependency_group_count
indexed_root_mode_count
source_policy_reuse_hash
source_cost_catalog_hash
source_evaluator_savings_hash
source_scaling_projection_hash
source_distillation_readiness_hash
required_healthy_modes_present
required_regression_modes_present
required_dependency_groups_present
index_complete
missing_surface
surface_hash
receipt_hash
```

Completed implementation tasks:

1. Added `PolicyReuseEvidenceSurfaceIndexReceipt` with deterministic validation, JSON output, surface hash, and receipt hash.
2. Added healthy and controlled required-regression-modes-missing smoke constructors.
3. Bound index evidence to policy reuse, cost catalog, evaluator-savings, scaling-projection, and distillation-readiness source hashes.
4. Added root validator compact modes for healthy and regression index receipts.
5. Added validation harness contracts for index semantics, source binding, mode counts, compact output, and controlled regression evidence.
6. Updated external CLI mode fixture and root compact-mode counts for the two new public modes.
7. Updated validation harness guarded-test count from 156 to 160.
8. Updated retained guarded-test fixture values from 166 to 170 and refreshed dependent validation-duration estimates.
9. Kept kernel authority unchanged and did not train a student model or promote policy.

Implemented deterministic semantics:

- `index_complete = true` only when required healthy modes, required regression modes, required dependency groups, and `missing_surface = "none"` all hold.
- Regression evidence remains structurally valid while exposing `index_complete = false` and `missing_surface = "required_regression_modes"`.
- Index source hashes bind to policy reuse, cost catalog, evaluator-savings, scaling-projection, and distillation-readiness evidence.
- The index receipt is evidence-only; it summarizes evaluator-facing learning evidence without granting policy authority.
- The new receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence Recorded For This Baseline

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract policy_reuse_evidence_surface_index --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet

cargo fmt --check: pass
validation_harness_contract policy_reuse_evidence_surface_index filter: 4 passed, 0 failed, 156 filtered out
planning_contract: 2 passed, 0 failed
score_contract: 5 passed, 0 failed
```

Full-suite validation was not run before this planning/scoring update.

## Planned Next Implementation Slice

The weakest remaining axis is **Performance**.

Current gap:

```text
The evidence surface now indexes learning receipts and dependencies, but root validation still exposes many compact modes that must be invoked one at a time to inspect the learning evidence surface.
```

Recommended next slice:

```text
Add a deterministic policy reuse evidence bundle receipt that rolls up the indexed policy-reuse evidence surface into one compact root mode for evaluator inspection.
```

Recommended concrete surfaces:

```text
PolicyReuseEvidenceBundleReceipt
policy_reuse_evidence_bundle_smoke_receipt()
policy_reuse_evidence_bundle_regression_smoke_receipt()
--policy-reuse-evidence-bundle-smoke
--policy-reuse-evidence-bundle-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse existing receipt hashes instead of recomputing independent semantics.
4. Prefer one evaluator-facing bundle verdict over additional scattered root-mode interpretation.
5. Include healthy and controlled incomplete-index or dependency-mismatch regression cases.
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
- Parallel orchestration execution should wait until larger-batch reuse, validation health, retained validation/runtime cost, catalog completeness, evaluator savings, scaling projection, distillation readiness, evidence-surface indexing, and bundled evidence inspection are proven together.
- Full-suite validation should run after the evidence-bundle slice if fixture or CLI mode churn is broader than expected.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Implement in capability/runtime-adjacent code without kernel authority drift.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.
