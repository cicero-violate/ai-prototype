# Finance Domain

Purpose: turn global/business signals into capital, risk, and allocation
intelligence.

Finance is a research and allocation-intelligence layer first. It should not
become an execution layer until verification, risk envelopes, and policy gates
are mature.

## Candidate Concepts

```text
AssetUniverse
  equities, ETFs, indexes, rates, FX, commodities, crypto, private assets

RiskBudget
  max drawdown, liquidity, concentration, leverage, volatility, correlation

AllocationHypothesis
  thesis, catalyst, horizon, expected value, invalidation condition

MarketRegime
  trend, volatility, liquidity, credit, macro, sentiment

CatalystCalendar
  earnings, central bank dates, economic releases, regulatory events
```

## Draft Record: FinanceHypothesis

```text
FinanceHypothesis
  asset_or_theme_id
  thesis_hash
  source_set_hash
  catalyst_hash
  horizon
  expected_value_score
  downside_risk_score
  liquidity_score
  confidence_score
  uncertainty_score
  invalidation_hash
  execution_allowed = false
```

## Risk Dimensions

```text
drawdown
volatility
liquidity
concentration
correlation
leverage
counterparty_or_platform_risk
regulatory_risk
model_risk
```

## Research vs Allocation vs Execution

```text
Research
  allowed: claims, summaries, hypotheses, risk analysis

Allocation Hypothesis
  allowed: proposed sizing logic, risk bounds, scenario analysis

Execution
  not allowed in this layer
```

## Eval Metrics

- Claim correctness.
- Source quality.
- Thesis clarity.
- Risk completeness.
- Invalidation quality.
- Forecast calibration.
- Decision usefulness.

## TODO

- [ ] Define asset universe scope.
- [ ] Define `FinanceHypothesis`.
- [ ] Define finance risk score dimensions.
- [ ] Define verification requirements for finance claims.
- [ ] Define eval metrics for financial research quality.
- [ ] Keep execution separate from analysis.
- [ ] Require primary-source support for high-impact claims.
- [ ] Require explicit stale-data policy.
