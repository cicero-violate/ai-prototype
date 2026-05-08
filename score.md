# Canon Agent Score

This scorecard is the current planning baseline, not a claim of full-system completion.

## Validation Evidence From This Turn

```text
command = RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet
result  = pass

planning_contract: 2 passed, 0 failed
score_contract:    5 passed, 0 failed
```

Full-suite validation was not run during this planning/scoring turn.

## Axis Scores

Scores are on a `0.00` to `1.00` scale and reflect the current repository evidence inspected this turn.

```text
I  Intelligence      = 0.64  typed capability surfaces exist; adaptive reuse loop still thin
E  Efficiency        = 0.61  validation footprint exists; LLM-cost reduction is not yet measured end-to-end
C  Correctness       = 0.78  deterministic contracts and targeted tests pass
A  Alignment         = 0.82  architecture matches GOAL.md separation of kernel and capability authority
R  Robustness        = 0.72  recovery classes, bounded receipts, replay checks, and validation harness are present
P  Performance       = 0.58  performance hooks exist; no fresh full benchmark run this turn
S  Scalability       = 0.56  orchestration surfaces exist; scaling should wait for stronger single-run proof
D  Determinism       = 0.84  kernel/runtime contracts, replay, and wrapper-clearing validation support determinism
T  Transparency      = 0.76  hash-chain, receipts, NDJSON, observe report, and proof surfaces are broad
Co Collaboration     = 0.66  docs and contracts are readable; planning files were previously placeholders
Em Empowerment       = 0.63  CLI/API/graph contracts exist; external mutation agent handoff still needs stronger receipts
B  Benefit           = 0.70  project direction is useful and coherent; implementation remains prototype-stage
L  Learning          = 0.49  verified distillation/policy surfaces exist, but measured policy reuse is weakest
Si Simplicity        = 0.57  many surfaces increase complexity; next work should consolidate evidence, not add authority
F  Future-Proofing   = 0.73  layered architecture and graph contracts leave room for verified evolution
```

Approximate geometric mean:

```text
G ≈ 0.667
```

## Current Judgment

```text
weakest_axis = Learning
next_action  = add deterministic policy reuse ledger summary with validation-health regression detection
scope        = capability-layer receipt/tests only; do not change kernel authority
validation   = targeted planning/scoring tests passed; full suite not run this turn
```

## Why Learning Is Weakest

The goal requires cost reduction through accumulated intelligence: policy should handle common cases and reserve LLM calls for novel cases. The repository has many prerequisite surfaces, including policy, eval, judgment, distillation rows, proof receipts, and validation harness fixtures. What is still under-evidenced is a compact, replayable measurement that proves policy reuse is increasing while correctness/validation health does not regress.

## Next Score Improvement Target

Raise `L` from `0.49` toward `0.58` by implementing and testing a deterministic policy reuse summary receipt:

```text
policy_hits
policy_misses
llm_fallbacks
validation_passes
validation_failures
reuse_rate
regression_flag
source_receipt_hash
```

The improvement should only count if replay/tamper tests pass and validation-health regression is explicitly detected.
