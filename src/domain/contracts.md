# Domain Contracts

This file sketches shared domain records. These are conceptual contracts only;
they are not compiled Rust structs yet.

## Contract Status

```text
status: draft
wiring: none
crate_surface: none
serialization: none
runtime_effect: none
```

## Domain IDs

```text
DomainId
  Unknown = 0
  GlobalIntelligence = 1
  Finance = 2
  Business = 3
  TradingSandbox = 4
```

## DomainSignal

Purpose: represent one normalized domain input.

```text
DomainSignal
  schema_version
  signal_id
  domain_id
  source_id
  source_kind
  observed_at
  received_at
  horizon
  signal_class
  payload_hash
  provenance_hash
  source_quality_score
  freshness_score
  contradiction_score
  initial_risk_class
```

TODO:

- [ ] Decide whether `source_id` is opaque hash, stable enum, or URI-derived hash.
- [ ] Define source quality calculation.
- [ ] Define freshness decay.
- [ ] Define contradiction scoring.
- [ ] Define signal hash identity rules.

## DomainContext

Purpose: assemble surrounding evidence for a signal or objective.

```text
DomainContext
  schema_version
  context_id
  objective_id
  domain_id
  input_signal_hash
  memory_set_hash
  policy_hash
  recent_evidence_hash
  missing_evidence_hash
  contradiction_set_hash
  context_quality_score
```

TODO:

- [ ] Define required context fields per domain.
- [ ] Define memory retrieval bounds.
- [ ] Define missing-evidence thresholds.
- [ ] Define contradiction set semantics.

## DomainJudgment

Purpose: convert context into a bounded verdict.

```text
DomainJudgment
  schema_version
  judgment_id
  domain_id
  context_hash
  opportunity_score
  risk_score
  confidence_score
  uncertainty_score
  actionability_score
  policy_fit_score
  expected_value_score
  verdict
  rationale_hash
```

Verdicts:

```text
Ignore
Watch
Research
ActBusiness
ActFinanceResearch
SimulateTrading
Block
```

TODO:

- [ ] Define score ranges and saturation behavior.
- [ ] Define actionability thresholds.
- [ ] Define blocked verdict conditions.
- [ ] Define rationale hashing policy.

## DomainPlan

Purpose: describe the next domain path without directly executing it.

```text
DomainPlan
  schema_version
  plan_id
  domain_id
  judgment_hash
  plan_kind
  required_capability_set_hash
  expected_receipt_set_hash
  risk_envelope_hash
  success_metric_hash
  rollback_or_invalidation_hash
```

Plan kinds:

```text
ResearchPlan
BusinessWorkflowPlan
FinanceAnalysisPlan
AllocationHypothesisPlan
TradingSimulationPlan
LearningPromotionPlan
```

TODO:

- [ ] Define capability requirements by plan kind.
- [ ] Define receipt requirements by plan kind.
- [ ] Define invalidation contracts.
- [ ] Define eval metrics per plan kind.

## DomainRiskEnvelope

Purpose: prevent unsafe, stale, overconfident, or unverified action.

```text
DomainRiskEnvelope
  schema_version
  envelope_id
  domain_id
  max_uncertainty
  min_confidence
  max_staleness
  max_concentration
  max_live_effect_level
  requires_human_review
  requires_verification
  sandbox_only
```

TODO:

- [ ] Define live effect levels.
- [ ] Define when human review is mandatory.
- [ ] Define finance/trading-specific risk fields.
- [ ] Define stale-data blocking policy.

## DomainEval

Purpose: score post-result quality and decide whether learning is allowed.

```text
DomainEval
  schema_version
  eval_id
  domain_id
  plan_hash
  result_hash
  correctness_score
  usefulness_score
  risk_adherence_score
  roi_or_value_score
  reproducibility_score
  promotion_allowed
```

TODO:

- [ ] Define eval metrics by domain.
- [ ] Define reproducibility requirements.
- [ ] Define promotion thresholds.
- [ ] Define regression detection.

## DomainPromotionCandidate

Purpose: identify repeatable verified wins suitable for policy promotion.

```text
DomainPromotionCandidate
  schema_version
  candidate_id
  domain_id
  source_eval_set_hash
  repeated_pattern_hash
  policy_delta_hash
  expected_gain_score
  regression_risk_score
  promotion_verdict
```

TODO:

- [ ] Require multiple independent eval receipts before promotion.
- [ ] Define regression-risk threshold.
- [ ] Define rollback receipt requirements.
- [ ] Map successful candidates to `PolicyPromotion` only after replay verification.