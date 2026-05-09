# Domain Scoring

Purpose: define common scoring logic for domain intelligence without wiring it
into the runtime.

Scores should be bounded, replayable, explainable, and conservative under
uncertainty.

## Score Range

```text
0 = absent / worst / blocked
1 = maximum / best / fully satisfied
```

Scores should saturate rather than overflow. Missing evidence should lower
confidence and raise uncertainty.

## Core Equation

```text
domain_value = opportunity * confidence * policy_fit * verification_readiness
             - risk * uncertainty * staleness_penalty
```

## Actionability

```text
actionability = domain_value * source_quality * context_quality
```

Draft thresholds:

```text
0.00 - 0.20 = Ignore
0.20 - 0.40 = Watch
0.40 - 0.60 = Research
0.60 - 0.80 = Act only if verified and low-risk
0.80 - 1.00 = Strong candidate, still requires receipts
```

TODO:

- [ ] Validate threshold values with fixtures.
- [ ] Define domain-specific overrides.
- [ ] Define minimum verification requirements for action thresholds.

## Source Quality

```text
source_quality = reliability * directness * recency * independence
```

Source quality factors:

- Reliability: historical correctness.
- Directness: primary source versus commentary.
- Recency: whether the source is stale for the decision horizon.
- Independence: whether multiple sources are genuinely separate.

TODO:

- [ ] Define source quality receipts.
- [ ] Penalize circular citations.
- [ ] Require primary sources for high-impact finance/business claims.

## Confidence and Uncertainty

Confidence is not the inverse of uncertainty. Both should be tracked.

```text
confidence = evidence_strength * source_quality * consistency
uncertainty = missing_evidence + contradiction + volatility + model_risk
```

TODO:

- [ ] Define contradiction score.
- [ ] Define missing evidence score.
- [ ] Define model-risk score.
- [ ] Preserve uncertainty in all action plans.

## Risk Score

Generic risk:

```text
risk = impact * likelihood * irreversibility
```

Finance/trading risk:

```text
market_risk = drawdown * volatility * liquidity_penalty * concentration * leverage
```

Business risk:

```text
business_risk = delivery_complexity * trust_risk * compliance_risk * support_burden
```

TODO:

- [ ] Define risk dimensions per domain.
- [ ] Require risk envelope before execution-oriented plans.
- [ ] Block high-risk actions without verification.

## Promotion Score

```text
promotion_score = reproducibility * correctness * usefulness * risk_adherence
                - regression_risk
```

Promotion should require multiple successful evals, replay verification, and no
open contradiction above threshold.

TODO:

- [ ] Define minimum eval count.
- [ ] Define regression test fixtures.
- [ ] Define rollback path for bad promotions.