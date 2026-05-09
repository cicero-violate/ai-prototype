# Domain Roadmap

## Stage 0: Documentation and Contracts

Status: current.

Goals:

- [x] Create unwired `src/domain` directory.
- [x] Define architecture boundary.
- [x] Add subdomain sketches.
- [ ] Stabilize conceptual records.
- [ ] Stabilize scoring equations.
- [ ] Stabilize future capability mappings.

Exit criteria:

- [ ] Domain records are clearly named.
- [ ] Domain records do not duplicate capability records.
- [ ] Trading remains explicitly sandbox-only.
- [ ] README explains the full lifecycle.

## Stage 1: Fixtures Before Code

Goals:

- [ ] Create sample global-intelligence signal fixture.
- [ ] Create sample finance allocation-hypothesis fixture.
- [ ] Create sample business workflow-opportunity fixture.
- [ ] Create sample trading paper-simulation fixture.
- [ ] Define expected scores and verdicts for each fixture.

Exit criteria:

- [ ] Each fixture maps to an existing capability target.
- [ ] Each fixture includes provenance and uncertainty.
- [ ] Each fixture has expected verification/eval behavior.

## Stage 2: Rust Record Drafts, Still Unwired

Goals:

- [ ] Convert stable contracts into Rust structs/enums inside `src/domain`.
- [ ] Keep module unwired from `lib.rs` until tests are ready.
- [ ] Add local unit tests only if module becomes compiled later.
- [ ] Avoid I/O in domain records.

Exit criteria:

- [ ] Records are pure data.
- [ ] Hash identity is deterministic.
- [ ] No direct runtime mutation exists.

## Stage 3: Capability Bridge

Goals:

- [ ] Add conversion from domain records to capability records.
- [ ] Submit through existing `CommandEnvelope` path.
- [ ] Verify command ledger compatibility.
- [ ] Verify replay compatibility.

Exit criteria:

- [ ] Domain-originated evidence replays deterministically.
- [ ] Capability registry enforces allowed effects.
- [ ] No direct kernel mutation path exists.

## Stage 4: Business-First Productization

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

Goals:

- [ ] Add paper-trade-only simulation fixtures.
- [ ] Add backtest receipt requirements.
- [ ] Add post-trade review requirements.
- [ ] Evaluate rule adherence and drawdown.

Exit criteria:

- [ ] Trading remains sandbox-only.
- [ ] Every simulated action has a receipt.
- [ ] Eval includes expectancy, drawdown, hit rate, and rule adherence.