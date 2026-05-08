# Canon Agent Score

This scorecard is the current implementation baseline, not a claim of full-system completion.

## Validation Evidence

Targeted validation run this turn:

```text
command = RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test validation_harness_contract --quiet
command = RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
command = RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --quiet
result  = pass

planning_contract: 2 passed, 0 failed
score_contract:    5 passed, 0 failed
validation_harness_contract: 132 passed, 0 failed
judgment::record unit tests: 12 passed, 0 failed
```

Full-suite validation was not run in this implementation turn.

## Axis Scores

Scores are on a `0.00` to `1.00` scale and reflect the current repository evidence inspected this turn.

```text
I  Intelligence      = 0.66  compact policy reuse/validation summary is now exposed through validation harness smoke evidence
E  Efficiency        = 0.64  LLM fallback counts are now visible in root_validate compact output
C  Correctness       = 0.81  targeted harness, judgment, score, and planning contracts pass
A  Alignment         = 0.82  architecture remains aligned with kernel/capability authority separation
R  Robustness        = 0.74  summary receipt supports healthy and controlled-regression harness cases
P  Performance       = 0.58  performance hooks exist; no fresh full benchmark run this turn
S  Scalability       = 0.58  compact signal exists; larger retained-batch evidence is still needed
D  Determinism       = 0.84  deterministic contracts, replay receipts, and wrapper-cleared validation remain central
T  Transparency      = 0.80  policy hits, misses, LLM fallbacks, validation counts, regression flag, and source hash are in compact output
Co Collaboration     = 0.69  plan and score now hand off a scale-trace slice with passing evidence
Em Empowerment       = 0.65  consumers can call root_validate compact modes for direct learning-health evidence
B  Benefit           = 0.72  implementation directly supports cost-reduction measurement at the harness boundary
L  Learning          = 0.62  verified policy reuse is now a root validation/observe-facing signal
St Structure         = 0.73  structure remains explicit with smoke and controlled-regression paths
Si Simplicity        = 0.60  consumers no longer need to join reuse and validation receipts for the compact signal
F  Future-Proofing   = 0.75  layered architecture preserves kernel authority while extending capability evidence
```

Approximate geometric mean:

```text
G ≈ 0.698
```

## Current Judgment

```text
weakest_axis = Scalability
secondary_risk = Performance
next_action  = add deterministic policy reuse scale trace over larger retained batches
scope        = capability/validation-harness evidence only; do not change kernel authority
validation   = targeted validation-harness/judgment/score/planning tests passed; full suite not run this turn
```

## Why Learning Is Still Weakest

The project goal requires cost reduction through accumulated intelligence: policy should handle common cases while LLM calls are reserved for novel cases. This turn exposed `PolicyReuseLedgerSummaryReceipt` through validation harness/root_validate smoke modes, including healthy reuse and validation-regression cases. The remaining weak point is proving the same signal across larger retained batches and orchestration-scale traces.

## Next Score Improvement Target

Raise `S` from `0.58` toward `0.63` by adding a deterministic policy reuse scale trace over larger retained batches.

The next improvement should only count if the harness exposes these fields for a larger deterministic retained batch:

```text
batch_size
policy_hits
policy_misses
llm_fallbacks
validation_passes
validation_failures
reuse_rate_bps
regression_flag
avoided_llm_calls_per_batch
```

Required scoring evidence for the next turn:

- retained larger-batch healthy reuse case exposed by the harness;
- retained larger-batch regression or cost-growth case exposed by the harness;
- retained judgment-layer replay/tamper coverage;
- targeted validation commands and results recorded in this file.