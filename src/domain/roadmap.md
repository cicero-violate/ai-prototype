# Domain Roadmap

## Current Turn Summary

Status: Stage 1 fixture baseline implemented.

This update keeps `src/domain` unwired from runtime and kernel behavior while making the domain plan executable enough to validate. The project now has deterministic JSON fixtures for global intelligence, business workflow opportunity, finance research hypothesis, sandbox trading simulation, and blocked live trading. A Python contract test validates provenance, uncertainty, expected verdicts, expected capability bridge targets, score equations, and the sandbox-only trading boundary.

Implemented artifacts:

- `tests/fixtures/domain/global_signal_macro.json`
- `tests/fixtures/domain/business_workflow_opportunity.json`
- `tests/fixtures/domain/finance_hypothesis_research.json`
- `tests/fixtures/domain/trading_simulation_sandbox.json`
- `tests/fixtures/domain/trading_live_blocked.json`
- `tests/test_domain_fixture_contract.py`

Observed validation:

- `python3 -m unittest tests/test_domain_fixture_contract.py` passed.

No domain Rust module wiring, runtime mutation, kernel change, TLog append path, capability execution path, live trading path, or external dependency was added.

## Stage 0: Documentation and Contracts

Status: substantially complete; keep refining as Rust records stabilize.

Goals:

- [x] Create unwired `src/domain` directory.
- [x] Define architecture boundary.
- [x] Add subdomain sketches.
- [x] Stabilize conceptual record families enough to create fixtures.
- [x] Stabilize initial integer scoring equations in fixture-test form.
- [x] Stabilize future capability mappings at descriptor level.

Exit criteria:

- [x] Domain records are clearly named.
- [x] Domain records do not duplicate capability records.
- [x] Trading remains explicitly sandbox-only.
- [x] README explains the full lifecycle.

Remaining Stage 0 work:

- Keep markdown specs synchronized with compiled records once Stage 2 begins.
- Replace fixture-only score contracts with Rust scoring functions when domain code is wired.

## Stage 1: Fixtures Before Code

Status: complete for baseline coverage; extend when adding new domain record families.

Goals:

- [x] Create sample global-intelligence signal fixture.
- [x] Create sample finance allocation-hypothesis fixture.
- [x] Create sample business workflow-opportunity fixture.
- [x] Create sample trading paper-simulation fixture.
- [x] Create sample blocked live-trading fixture.
- [x] Define expected scores and verdicts for each fixture.

Exit criteria:

- [x] Each fixture maps to an existing or future capability target descriptor.
- [x] Each fixture includes provenance and uncertainty.
- [x] Each fixture has expected verification/eval behavior.
- [x] Fixture score constants are checked by deterministic integer equations.
- [x] Trading fixtures prove sandbox-only simulation passes while live execution blocks.

Fixture contract rules:

- All fixture scores are bounded integers in `0..=1000`.
- `domain_value_score` is computed as:

```text
positive = opportunity * confidence * policy_fit * verification_readiness
penalty  = risk * uncertainty * staleness_penalty
domain_value_score = saturating_bounded(positive - penalty)
```

with each multiplication scaled by `1000` using deterministic integer truncation.

- `actionability_score` is computed as:

```text
actionability_score = domain_value_score * source_quality * context_quality
```

with the same bounded integer truncation.

- Fixtures are descriptor-only. They do not contain `CommandEnvelope`, `TLog`, or runtime mutation fields.

## Stage 2: Rust Record Drafts, Still Unwired

Status: next concrete implementation stage.

Goals:

- [ ] Convert stable contracts into Rust structs/enums inside `src/domain`.
- [ ] Keep module effects pure and free of I/O.
- [ ] Wire `pub mod domain;` only after record modules and tests compile.
- [ ] Add local unit tests or integration tests for deterministic identity, scoring, and risk envelope behavior.
- [ ] Avoid direct runtime, kernel, command-ledger, or TLog mutation.

Exit criteria:

- [ ] Records are pure data.
- [ ] Hash identity is deterministic.
- [ ] No direct runtime mutation exists.
- [ ] Fixtures deserialize into Rust records.
- [ ] Score and verdict tests match the Stage 1 JSON fixtures.

Next Stage 2 implementation order:

1. `src/domain/contracts.rs`: `DomainId`, schema version, source kind, horizon, signal class, verdict, plan kind, live-effect level, and shared record structs.
2. `src/domain/scoring.rs`: bounded integer score type and deterministic score functions matching fixture equations.
3. `src/domain/risk.rs`: sandbox/live-effect checks, especially finance execution block and trading sandbox-only block.
4. `src/domain/bridge.rs`: descriptor-only mapping to future capability record families.
5. `src/domain/identity.rs`: deterministic hash identity for domain records without adding external dependencies.
6. Subdomain modules for global intelligence, business, finance, and trading after shared contracts compile.

## Stage 3: Capability Bridge

Status: deferred until Stage 2 records compile and tests pass.

Goals:

- [ ] Add conversion from domain records to capability descriptors.
- [ ] Submit through existing `CommandEnvelope` path only after descriptor tests pass.
- [ ] Verify command ledger compatibility.
- [ ] Verify replay compatibility.

Exit criteria:

- [ ] Domain-originated evidence replays deterministically.
- [ ] Capability registry enforces allowed effects.
- [ ] No direct kernel mutation path exists.

## Stage 4: Business-First Productization

Status: deferred; business fixture exists.

Goals:

- [ ] Build business workflow intelligence loops first.
- [ ] Generate verified workflow receipts.
- [ ] Score cashflow/value impact.
- [ ] Promote repeatable business wins into policy.

Exit criteria:

- [ ] At least one workflow has repeated eval success.
- [ ] Promotion candidate has replay-verifiable support.
- [ ] Finance intelligence benefits from business cashflow data.

## Stage 5: Finance Intelligence

Status: deferred; finance research fixture exists.

Goals:

- [ ] Build finance research and allocation-hypothesis workflows.
- [ ] Add risk envelope fixtures.
- [ ] Separate research from execution.
- [ ] Require primary-source support for high-impact claims.

Exit criteria:

- [ ] Research plans are verified and evaluated.
- [ ] Allocation hypotheses include risk bounds.
- [ ] No execution path exists without future explicit policy.

## Stage 6: Trading Sandbox

Status: deferred; sandbox and live-block fixtures exist.

Goals:

- [ ] Add paper-trade-only simulation fixtures.
- [ ] Add backtest receipt requirements.
- [ ] Add post-trade review requirements.
- [ ] Evaluate rule adherence and drawdown.

Exit criteria:

- [ ] Trading remains sandbox-only.
- [ ] Every simulated action has a receipt.
- [ ] Eval includes expectancy, drawdown, hit rate, and rule adherence.

## Non-Goals Preserved

- No live trading execution.
- No direct domain mutation of kernel/runtime state.
- No direct domain append to TLog.
- No domain bypass around capability registry or verification.
- No external dependencies for the fixture baseline.
