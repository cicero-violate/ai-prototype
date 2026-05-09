# Trading Domain

Purpose: sandbox market reasoning, risk discipline, simulation, and evaluation.

Trading is not the primary business engine. It is a controlled testbed for data
quality, prediction discipline, risk limits, and verification.

## Default Policy

```text
sandbox_only = true
live_execution_allowed = false
broker_integration_allowed = false
human_review_required = true for any future live path
```

## Candidate Concepts

```text
PaperTradeHypothesis
  market, thesis, setup, entry condition, exit condition, invalidation

BacktestReceipt
  dataset hash, date range, rule hash, metric hash, limitations

RiskLimit
  max loss, max drawdown, max position, max leverage, stop condition

PostTradeReview
  rule adherence, outcome, error, lesson, promotion eligibility
```

## Draft Record: TradingSimulationPlan

```text
TradingSimulationPlan
  market_id
  thesis_hash
  rule_set_hash
  dataset_hash
  time_range_hash
  risk_limit_hash
  entry_rule_hash
  exit_rule_hash
  invalidation_hash
  expected_metric_hash
  sandbox_only = true
```

## Eval Metrics

```text
expectancy
drawdown
hit_rate
risk_adjusted_return
rule_adherence
slippage_assumption_quality
data_quality
overfit_risk
```

## Block Conditions

- Missing dataset provenance.
- Missing risk limit.
- Missing invalidation condition.
- Live execution requested.
- Overfit risk above threshold.
- No post-trade review path.

## TODO

- [ ] Keep all trading flows sandbox-only by default.
- [ ] Define paper-trade receipt structure.
- [ ] Define backtest requirements.
- [ ] Define hard risk constraints.
- [ ] Define no-live-execution policy until verification gates are mature.
- [ ] Define eval metrics: expectancy, drawdown, hit rate, risk-adjusted return, rule adherence.
- [ ] Define overfitting checks.
- [ ] Define post-trade learning restrictions.
