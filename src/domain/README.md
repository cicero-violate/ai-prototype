# Domain Architecture

Purpose: define the domain-specific intelligence layer for AI global intelligence,
finance, business workflows, and trading research while preserving the canonical
runtime boundary.

This directory is intentionally **not wired into `lib.rs`**. It is a specification
surface. Nothing here should affect the kernel, runtime, API, capability registry,
or validation harness until the contracts are stable.

## Core Boundary

```text
Domain
  = interpretation + strategy + scoring + policy intent + domain risk model

Capability
  = evidence production + rich records + receipts + tools + verification

Kernel
  = stable state, packet, phases, gates, evidence tokens

Runtime
  = deterministic reducer, writer, replay, recovery, persistence
```

The domain layer must not directly mutate `State`, `GateSet`, `Packet`, or `TLog`.
Future domain output should become capability input, then capability output should
be submitted as evidence through the existing command/API path.

## Target System Shape

```text
[External World]
  ├─ market data
  ├─ macro data
  ├─ news / filings / transcripts
  ├─ business/customer/workflow signals
  ├─ platform telemetry
  └─ user objectives
        ↓
[Domain Intake]
  ├─ normalize raw signal
  ├─ identify source and timestamp
  ├─ classify domain and subdomain
  ├─ assign source quality
  └─ attach risk/provenance metadata
        ↓
[Domain Context]
  ├─ retrieve memory
  ├─ compare against policy
  ├─ assemble market/business/global context
  ├─ preserve uncertainty
  └─ identify missing evidence
        ↓
[Domain Judgment]
  ├─ opportunity assessment
  ├─ risk assessment
  ├─ confidence / uncertainty score
  ├─ contradiction handling
  ├─ policy constraints
  └─ actionability verdict
        ↓
[Domain Plan]
  ├─ research plan
  ├─ business action plan
  ├─ allocation hypothesis
  ├─ automation workflow plan
  └─ trading simulation plan
        ↓
[Existing Capabilities]
  ├─ observation
  ├─ context
  ├─ memory
  ├─ llm
  ├─ judgment
  ├─ planning
  ├─ tooling
  ├─ verification
  ├─ eval
  └─ learning
        ↓
[Kernel / Runtime / Ledger]
  ├─ deterministic gates
  ├─ append-only events
  ├─ receipts
  ├─ replay
  ├─ recovery
  └─ policy promotion
```

## Strategic Direction

```text
Learning → Intelligence → Tools → Business Cashflow → Finance Intelligence → Capital Allocation
```

Primary path:

```text
AI Global Intelligence + Finance/Business Automation
```

Secondary path:

```text
Trading = sandbox for risk, prediction, verification, and eval discipline
```

Trading should not become the first production business path. The business path
should monetize reliable intelligence workflows first; trading can later validate
forecasting, market data, and risk controls under strict sandbox constraints.

## Domain File Map

```text
src/domain/
  README.md                 # architecture, boundaries, roadmap
  mod.rs                    # commented Rust-facing sketch; still unwired
  contracts.md              # shared domain record contracts
  scoring.md                # scoring model and decision equations
  integration.md            # future capability mappings
  roadmap.md                # staged build plan
  global_intelligence.md    # macro/geopolitical/technology signal model
  finance.md                # market, capital, risk, allocation model
  business.md               # SaaS/workflow/customer/value model
  trading.md                # sandbox-only market execution research model
```

## Domain Lifecycle

```text
1. Observe
   Raw signal enters the system.

2. Classify
   Signal is assigned a domain, source quality, horizon, and risk class.

3. Contextualize
   Signal is compared against memory, policy, prior results, and current objective.

4. Judge
   System estimates opportunity, risk, uncertainty, contradiction, and actionability.

5. Plan
   System proposes research, business action, allocation hypothesis, or simulation.

6. Execute Through Capabilities
   Tooling, LLM, planning, verification, and eval produce receipts.

7. Verify
   Claims, artifacts, actions, and outputs are checked for provenance and replayability.

8. Evaluate
   Result quality is scored against objective and risk constraints.

9. Learn
   Repeatable, verified wins can become policy candidates.
```

## Domain Contract Principles

- Every domain claim needs provenance.
- Every domain score needs an uncertainty component.
- Every domain action needs a policy boundary.
- Every tool effect needs a receipt.
- Every promoted rule needs replay-verifiable evidence.
- Business actions, research, simulation, allocation, and execution must stay distinct.
- Trading remains sandbox-only until explicit future contracts say otherwise.

## Future Record Families

These are conceptual records only. They are not Rust structs yet.

```text
DomainSignal
  raw observed domain item with source, timestamp, class, and payload hash

DomainContext
  assembled memory, policy, objective, and recent evidence around a signal

DomainJudgment
  opportunity/risk/confidence/uncertainty/actionability verdict

DomainPlan
  recommended research, business, finance, or sandbox trading path

DomainRiskEnvelope
  constraints that prevent unsafe, overconfident, or unverified action

DomainEval
  post-result scorecard for quality, usefulness, correctness, and ROI

DomainPromotionCandidate
  verified repeated pattern eligible for policy promotion
```

## TODO

### Structure

- [ ] Decide whether `domain` remains documentation-only or becomes a Rust module.
- [ ] Keep `domain` out of `lib.rs` until record contracts stabilize.
- [ ] Define domain records before behavior.
- [ ] Define domain IDs and schema versions before serialization.
- [ ] Keep domain semantics out of `kernel`.
- [ ] Avoid duplicating `capability` responsibilities.

### Global Intelligence

- [ ] Define signal classes: macro, geopolitical, technology, regulatory, liquidity, sentiment.
- [ ] Define source quality scoring.
- [ ] Define contradiction handling.
- [ ] Define time horizons: immediate, tactical, strategic, secular.
- [ ] Define actionability thresholds.
- [ ] Define stale-signal handling.

### Finance

- [ ] Define asset universe model.
- [ ] Define finance risk model: drawdown, volatility, liquidity, concentration, leverage.
- [ ] Define capital allocation hypothesis record.
- [ ] Define catalyst and event calendar model.
- [ ] Define finance-specific verification checks.
- [ ] Define explicit boundary between research and allocation.

### Business

- [ ] Define customer/workflow/problem records.
- [ ] Define value scoring: pain, urgency, willingness to pay, automation fit.
- [ ] Define monetization path from intelligence output to product/service.
- [ ] Define feedback loop into learning and policy promotion.
- [ ] Define workflow receipt requirements.

### Trading

- [ ] Keep trading sandbox-only by default.
- [ ] Define paper-trade receipt requirements.
- [ ] Define backtest/eval requirements before any live execution path is considered.
- [ ] Define hard risk limits.
- [ ] Define invalidation conditions.
- [ ] Define post-trade review.

### Capability Integration, Future Only

- [ ] Map `DomainSignal` to `ObservationRecord`.
- [ ] Map `DomainContext` to `ContextRecord` and `MemoryLookupRecord`.
- [ ] Map `DomainJudgment` to `JudgmentRecord` or `PolicyJudgmentRecord`.
- [ ] Map `DomainPlan` to `PlanRecord`.
- [ ] Map research/tool actions to `ToolExecutionRecord`.
- [ ] Map claim checks to `VerificationRecord`.
- [ ] Map result quality to `EvalRecord`.
- [ ] Map repeatable verified wins to `PolicyPromotion`.

### Safety / Correctness

- [ ] Require source provenance for every domain claim.
- [ ] Require confidence and uncertainty fields.
- [ ] Require receipts for tool effects.
- [ ] Require replay-verifiable evidence before promotion.
- [ ] Separate advice, research, simulation, allocation, and execution paths.
- [ ] Add explicit stale-data handling.
- [ ] Add explicit contradiction handling.