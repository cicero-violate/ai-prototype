# Global Intelligence Domain

Purpose: model world signals that affect business, finance, technology,
regulation, liquidity, and risk.

Global intelligence is the broadest signal layer. It should explain the world
state without jumping directly to allocation or execution.

## Signal Classes

```text
Macro
  rates, inflation, employment, GDP, fiscal policy, liquidity

Geopolitical
  conflict, sanctions, elections, trade policy, sovereign risk

Technology
  AI, semiconductors, cloud, cybersecurity, infrastructure, robotics

Regulatory
  financial rules, data rules, AI rules, market structure, tax rules

Liquidity
  credit conditions, dollar funding, central-bank operations, market plumbing

Sentiment
  news tone, social trend, analyst consensus, positioning

SupplyChain
  energy, shipping, commodities, manufacturing constraints
```

## Horizon Model

```text
Immediate = hours to days
Tactical  = days to weeks
Strategic = months to quarters
Secular   = years
```

## Output Intent

Global intelligence should usually output:

- watchlist updates,
- research questions,
- risk warnings,
- context for finance/business judgment,
- contradiction flags,
- stale-data warnings.

It should not directly output live trade decisions.

## Draft Record: GlobalSignalProfile

```text
GlobalSignalProfile
  signal_class
  horizon
  region
  sector
  affected_domains
  source_quality_score
  uncertainty_score
  contradiction_score
  actionability_hint
```

## Verification Questions

- Is this a primary source or interpretation?
- Is the signal stale for the decision horizon?
- Are there independent confirming sources?
- Does the signal contradict prior memory?
- Is the signal relevant to business, finance, or both?

## TODO

- [ ] Define `SignalClass`.
- [ ] Define `GlobalSignalProfile`.
- [ ] Define source quality scoring.
- [ ] Define contradiction handling.
- [ ] Define horizon-specific staleness limits.
- [ ] Define when a signal is actionable versus informational.
- [ ] Define how global signals become `ObservationRecord` inputs.
- [ ] Define mapping from global signals to finance/business contexts.
