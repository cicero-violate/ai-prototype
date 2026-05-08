# Canon Agent Implementation Plan

This plan tracks the current deterministic implementation plan for the planning/scoring turn after implementation step 4 of the current agent loop.

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
- retained fixtures for policy reuse, policy validation health, policy validation trend, orchestration capacity, policy capacity/cost, runtime performance trend, validation duration planning, validation command footprint, external CLI mode evidence, and evidence-surface index coverage.

## Current Completed Implementation Baseline

The current working tree contains deterministic **policy reuse evidence-quickcheck** evidence in the validation-harness/root-validator layer, while retaining evaluator-savings, scaling-projection, distillation-readiness, evidence-surface index, and evidence-bundle evidence already present in this loop.

This slice answers:

```text
Can an evaluator inspect one deterministic receipt that says whether the bundled policy-reuse learning evidence surface has the minimum validation command coverage needed for this turn?
```

Implemented surfaces:

```text
PolicyReuseEvidenceQuickcheckReceipt
policy_reuse_evidence_quickcheck_smoke_receipt()
policy_reuse_evidence_quickcheck_regression_smoke_receipt()
--policy-reuse-evidence-quickcheck-smoke
--policy-reuse-evidence-quickcheck-regression-smoke
```

Implemented fields:

```text
schema
record_type
quickcheck_version
source_bundle_hash
validation_harness_expected_tests
required_command_count
observed_command_count
minimum_command_set_hash
bundle_complete
commands_complete
quickcheck_passed
missing_command
quickcheck_hash
receipt_hash
```

Completed implementation tasks:

1. Added `PolicyReuseEvidenceQuickcheckReceipt` with deterministic validation, JSON output, quickcheck hash, and receipt hash.
2. Added healthy and controlled missing-validation-harness-command regression smoke constructors.
3. Bound quickcheck evidence to the policy reuse evidence bundle receipt and a deterministic minimum validation command-set hash.
4. Added root validator compact modes for healthy and regression quickcheck receipts.
5. Added validation harness contracts for quickcheck semantics, source binding, compact output, and controlled missing-command evidence.
6. Updated external CLI mode fixture and root compact-mode counts for the two new public modes.
7. Updated validation harness guarded-test count from 164 to 168.
8. Updated retained guarded-test fixture values from 174 to 178 and refreshed dependent validation-duration estimates.
9. Kept kernel authority unchanged and did not train a student model or promote policy.

Implemented deterministic semantics:

- `quickcheck_passed = true` only when the evidence bundle is complete, required command coverage is complete, and `missing_command = "none"`.
- Regression evidence remains structurally valid while exposing `quickcheck_passed = false` and `missing_command = "validation_harness_contract"`.
- Quickcheck source evidence reuses the bundle receipt hash and fixed validation command names instead of adding policy authority.
- The quickcheck receipt is evidence-only; it summarizes validation coverage without granting policy authority.
- The new receipt does not introduce live LLM, network, wall-clock, or environment-dependent measurement.

## Validation Evidence Recorded For This Baseline

```text
cargo fmt --check
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract policy_reuse_evidence_quickcheck --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet

cargo fmt --check: pass
validation_harness_contract policy_reuse_evidence_quickcheck filter: 4 passed, 0 failed, 164 filtered out
validation_harness_contract: 168 passed, 0 failed
planning_contract: 2 passed, 0 failed
score_contract: 5 passed, 0 failed
```

Full-suite validation was not run before this planning/scoring update.

## Planned Next Implementation Slice

The weakest remaining axis is **Simplicity**.

Current gap:

```text
Policy-reuse learning evidence is now indexed, bundled, and quickchecked, but each added receipt still expands the root-mode and fixture-count surface.
```

Recommended next slice:

```text
Add a deterministic policy reuse evidence maturity receipt that summarizes the reuse/readiness/index/bundle/quickcheck stack into one staged maturity level for evaluators.
```

Recommended concrete surfaces:

```text
PolicyReuseEvidenceMaturityReceipt
policy_reuse_evidence_maturity_smoke_receipt()
policy_reuse_evidence_maturity_regression_smoke_receipt()
--policy-reuse-evidence-maturity-smoke
--policy-reuse-evidence-maturity-regression-smoke
```

Recommended constraints:

1. Keep the kernel untouched.
2. Do not introduce live LLM, network, wall-clock, or environment-dependent measurement.
3. Reuse existing quickcheck and bundle hashes.
4. Prefer one maturity level over additional scattered receipt interpretation.
5. Include healthy and controlled quickcheck-failed or immature-stage regression cases.
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
- Parallel orchestration execution should wait until larger-batch reuse, validation health, retained validation/runtime cost, catalog completeness, evaluator savings, scaling projection, distillation readiness, evidence-surface indexing, bundled evidence inspection, quickcheck validation, and maturity staging are proven together.
- Full-suite validation should run after the maturity slice if fixture or CLI mode churn is broader than expected.

## Turn Protocol

1. Plan and score first.
2. Choose the smallest implementation slice improving the weakest axis.
3. Implement in capability/runtime-adjacent code without kernel authority drift.
4. Add deterministic contract tests.
5. Run targeted validation and record results.
6. Update `plan.md` and `score.md`.
7. Commit the turn.
