# Business Domain

Purpose: convert intelligence into products, workflows, services, and cashflow.

This is the preferred first monetization layer because verified business
workflows can produce repeatable value without requiring live market execution.

## Candidate Concepts

```text
CustomerProblem
  who has the problem, why it hurts, how often it appears

WorkflowBottleneck
  manual step, delay, error source, cost center, missed revenue

AutomationFit
  data availability, repeatability, tool access, verification path

ValueSignal
  urgency, willingness to pay, budget owner, frequency, measurable ROI

DeliveryChannel
  SaaS, service, internal tool, report, agent workflow, API

FeedbackLoop
  usage, customer correction, outcome score, retention, expansion
```

## Draft Record: BusinessOpportunity

```text
BusinessOpportunity
  customer_segment_hash
  problem_hash
  workflow_hash
  urgency_score
  willingness_to_pay_score
  automation_fit_score
  delivery_complexity_score
  trust_risk_score
  expected_value_score
  verification_path_hash
```

## Monetization Score

```text
monetization_score = pain * urgency * willingness_to_pay * automation_fit
                   - delivery_complexity * trust_risk
```

## Preferred Outputs

- Workflow automation plan.
- Customer discovery plan.
- Repeatable service blueprint.
- SaaS feature hypothesis.
- Internal productivity tool.
- Verified report pipeline.

## Eval Metrics

- Time saved.
- Error reduction.
- Revenue impact.
- Customer willingness to pay.
- Repeatability.
- Support burden.
- Verification quality.

## TODO

- [ ] Define `BusinessOpportunity`.
- [ ] Define workflow automation record.
- [ ] Define monetization score fixtures.
- [ ] Define customer feedback receipt.
- [ ] Define productization path from repeated successful workflows.
- [ ] Define business feedback loop into `PolicyPromotion`.
- [ ] Define trust/compliance risk scoring.
