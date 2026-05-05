# Phase 2 Turn 3 Plan

## Variables

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
Si = Simplicity
F  = Future-Proofing
G  = Goodness
```

## Equation

```text
G = (I*E*C*A*R*P*S*D*T*Co*Em*B*L*Si*F)^(1/15)
max(G) = good
```

One-line explanation: Goodness is the geometric mean of all 15 dimensions; one weak dimension lowers the whole system.

## Source Inputs

- `GOAL.md` requires verified evolution, TLog-derived learning, policy promotion, and distillation rows that retain semantic input/action/output/score/proof lineage.
- `score.md` remains preserved from the prior Phase 1 scorecard update and was not edited during this turn.
- Runtime archive inspection found `RUNTIME_MANIFEST.json`, `.repo-agent-runtime/audit.ndjson`, `log/chatgpt_project_agent.ndjson`, download manifests, observe-validation reports, score snapshots, and prior delta/download history.
- Active implementation TODO/FIXME scan found no source-level TODO/FIXME markers outside plan/score text.

## Boundary

- Preserve `learning -> policy`; policy remains append-only storage and must not own learning promotion.
- Do not move learning behavior into `policy/store.rs`.
- Do not edit `score.md` during this turn.
- Final artifacts are created only after the committed `B..H` range is complete.

## Tasks

1. Preserve the prior scorecard and policy-learning contract commits from this conversation.
2. Add a learning-owned `DistillationRow` record for training-ready trace rows derived from verified policy promotion.
3. Bind distillation rows to nonzero instruction, input-state, action, output, score, proof hash, and source event fields.
4. Reject rows with zero semantic fields or proof hashes not equal to the promotion proof hash.
5. Extend the policy-learning validator so the learning contract includes distillation evidence while policy remains append-only only.
6. Extend Python and Rust tests for the new distillation contract.
7. Validate with explicit bounded Python tests, Rust tests, validators, syntax checks, py_compile, and diff checks.
8. Commit the turn result, then create `/mnt/data/repo-delta-004.bundle` and `/mnt/data/DELTA_MANIFEST.md` for `B..H`.

## Completed Change

- Added `DISTILLATION_ROW_SCHEMA_VERSION`.
- Added `DistillationRow` in `src/capability/learning/promote.rs`.
- Added `DistillationRow::from_policy_promotion`, `is_valid_for`, and `expected_row_hash`.
- Exported distillation types through `capability::learning` and crate root exports.
- Added Rust tests:
  - `verified_policy_promotion_distills_training_ready_row`
  - `distillation_row_rejects_unbound_or_zero_training_fields`
- Extended `scripts/validate_policy_learning_trace.py` with `distillation_row_contract`.
- Extended `tests/test_policy_learning_trace_contract.py` to require the distillation contract under `timeout=30s`.

## Validation Result

```text
bootstrap_rustc_session.py with writable cargo home: pass
runtime archive inspected: pass
TODO/FIXME source scan: pass, no active source markers outside plan/score text
timeout 30s python3 -X faulthandler -m unittest tests.test_policy_learning_trace_contract -v: pass, 3 tests
timeout 30s python3 -X faulthandler -m unittest tests.test_observe_validation_contract -v: pass, 12 tests
timeout 30s python3 -X faulthandler -m unittest tests.test_write_delta_manifest -v: pass, 10 tests
timeout 300s cargo test --all-targets --no-fail-fast: pass, 105 Rust tests
timeout 30s python3 scripts/validate_policy_learning_trace.py --root . --report target/observe/policy-learning-trace.json: pass, missing_count=0
timeout 30s python3 scripts/validate_rust_panic_surface.py --root . --fail-production-unwrap --report target/observe/panic-surface.json: pass, production_total=0
timeout 30s bash -n scripts/observe_validation.sh: pass
timeout 30s python3 -m py_compile scripts/write_delta_manifest.py scripts/validate_policy_learning_trace.py scripts/validate_rust_panic_surface.py: pass
git diff --check: pass
git diff --exit-code -- score.md: pass, untouched during this turn
```

## Remaining Risk

- Distillation rows are now typed and proof-bound, but this turn does not implement JSONL export, full dataset retention, a student-model training job, or live AlphaEvolve candidate selection.