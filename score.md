# Canon Agent Score

This scorecard is the current implementation baseline, not a claim of full-system completion.

## Validation Evidence From This Turn

```text
command = RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --test score_contract --test planning_contract --test validation_harness_contract --quiet
command = RUSTC_WRAPPER="" RUSTC_WORKSPACE_WRAPPER="" cargo test --lib capability::judgment::record --quiet
result  = pass

planning_contract: 2 passed, 0 failed
score_contract:    5 passed, 0 failed
validation_harness_contract: 128 passed, 0 failed
judgment::record unit tests: 12 passed, 0 failed
```

Full-suite validation was not run during this implementation turn.

## Axis Scores

Scores are on a `0.00` to `1.00` scale and reflect the current repository evidence inspected this turn.

```text
I  Intelligence      = 0.65  typed capability surfaces now include compact policy reuse/validation summary
E  Efficiency        = 0.63  LLM fallback counts are now explicit in a deterministic summary receipt
C  Correctness       = 0.80  deterministic contracts and targeted tests pass
A  Alignment         = 0.82  architecture matches GOAL.md separation of kernel and capability authority
R  Robustness        = 0.73  summary receipt rejects tampered fallback counts and source binding
P  Performance       = 0.58  performance hooks exist; no fresh full benchmark run this turn
S  Scalability       = 0.57  orchestration surfaces exist; summary gives future batch routing a compact signal
D  Determinism       = 0.84  kernel/runtime contracts, replay, and wrapper-clearing validation support determinism
T  Transparency      = 0.78  policy hits, misses, LLM fallbacks, validation counts, and source hash are explicit
Co Collaboration     = 0.67  plan and score identify the next harness integration handoff
Em Empowerment       = 0.64  public crate exports now include the policy reuse ledger summary receipt
B  Benefit           = 0.71  implementation directly supports cost-reduction measurement
L  Learning          = 0.56  verified policy reuse is now measurable with validation-regression flagging
St Structure         = 0.72  structure is now a first-class typed score axis and markdown score axis
Si Simplicity        = 0.56  one new receipt adds surface area; next work should consolidate through harness output
F  Future-Proofing   = 0.74  layered architecture and graph contracts leave room for verified evolution
```

Approximate geometric mean:

```text
G ≈ 0.681
```

## Current Judgment

```text
weakest_axis = Learning
next_action  = expose policy reuse ledger summary through validation harness/catalog smoke evidence
scope        = validation-harness integration only; do not change kernel authority
validation   = targeted score/planning/validation-harness/judgment tests passed; full suite not run this turn
```

## Why Learning Is Still Weakest

The goal requires cost reduction through accumulated intelligence: policy should handle common cases and reserve LLM calls for novel cases. This turn added the compact, replayable `PolicyReuseLedgerSummaryReceipt` with policy hits, misses, LLM fallbacks, validation pass/fail counts, reuse rate, regression flag, and source receipt hash. What remains under-evidenced is surfacing that receipt through the root validation/observe path as a single consumer-facing signal.

## Next Score Improvement Target

Raise `L` from `0.56` toward `0.62` by integrating the deterministic policy reuse summary receipt into validation-harness smoke evidence:

```text
policy_hits
policy_misses
llm_fallbacks
validation_passes
validation_failures
reuse_rate_bps
regression_flag
source_receipt_hash
```

The next improvement should only count if the validation harness exposes both pass and regression cases while retaining tamper/replay coverage in the judgment unit tests.