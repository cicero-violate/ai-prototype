# Evaluation Work Plan

## Variables

```text
I  = Intelligence
E  = Efficiency
C  = Correctness
A  = Alignment
R  = Robustness
P  = Performance
S  = Scalability
D  = Determinism
T  = Transparency
Co = Collaboration
Em = Empowerment
B  = Benefit
L  = Learning
Si = Simplicity
F  = Future-Proofing
G  = Goodness
```

## Equation

```text
G = (I*E*C*A*R*P*S*D*T*Co*Em*B*L*Si*F)^(1/15)
max(G) = good
```

Goodness is the geometric mean of all 15 dimensions; the best next work raises weak axes without lowering strong ones.

## Evaluation Boundary

```text
repository = ai
base_commit = 2b9f4c7
restored_head_before_plan = 2b9f4c7
bundle_output = /mnt/data/repo-delta-001.bundle
manifest_output = /mnt/data/DELTA_MANIFEST.md
allowed_mutation = score.md and plan.md only
```

## Inputs Read

- [x] `GOAL.md`
- [x] `score.md`
- [x] `git log --oneline -20`
- [x] Current `plan.md`
- [x] Repository structure, source surface, and validation surface

## Current State

- [x] The root crate exposes deterministic runtime, replay verification, recovery policy, scoring, command ledger, LLM receipts, process receipts, provider examples, API transport replay/idempotence, bounded observation ingress, durable policy promotion, generic verification proof records, semantic verification seams, a root validation harness, and graph telemetry validation receipt generation.
- [x] `cargo -Znext-lockfile-bump run --bin root_validate --locked` passes and covers root cargo check, bounded score contract tests, and canon-rustc-v3 graph telemetry evidence.
- [x] Bounded observation ingress with cursor persistence and backpressure is now implemented and tested; it should no longer appear as an open plan weakness.
- [x] The root validation harness now runs root cargo check, score-contract tests, library unit tests, API transport integration tests, validation harness contract tests, graph telemetry, and a deterministic Python-contract skip receipt when required validation scripts are absent from this checkout.
- [x] Planning now materializes a deterministic bounded task graph with dependency ordering, ready-set derivation, completion accounting, revision fields, and replayable lineage hashes while preserving the kernel-visible `TaskReady` evidence seam.
- [x] Orchestration still routes a single deterministic capability order through local gate readiness; it does not yet model bounded parallel runs, prioritization queues, resource budgets, or deterministic merge/selection receipts.
- [x] Semantic verification has a typed request/receipt seam and generic proof spine, but the implementation explicitly does not inspect real files yet, leaving artifact validation short of GOAL.md's semantic checking requirement.
- [x] No source code, `GOAL.md`, `bootstrap_rustc_session.py`, generated files, or runtime files were modified in this evaluation turn.

## Ranked Tasks By Expected Score Delta

### 1. [x] Widen root validation into a complete deterministic contract suite

```text
expected_delta = max(C, R, T, D, F, B)
rank = 1
```

Extend `root_validate` so the validation receipt covers the full current correctness surface, not only `cargo check`, `score_contract`, and graph telemetry. Include the Rust integration suites for API transport, validation harness, replay/proof contracts, provider receipt contracts, and the Python contract checks that validate observation, panic-surface, policy-learning, bootstrap, and manifest behavior. Emit a stable receipt that records each suite, command, exit code, stdout/stderr byte counts, and any skipped capability with a deterministic reason.

**Why this is first:** the repository claims auditability and correctness by construction, but the primary validation command currently proves only a subset of the code that now exists. A broader deterministic receipt would raise confidence across correctness, robustness, transparency, and future-proofing without changing the kernel.

### 2. [x] Replace single-task planning with deterministic objective decomposition and plan lineage

```text
expected_delta = max(I, E, Co, Em, L, S)
rank = 2
```

Upgrade planning from `PlanRecord::from_packet` producing one synthetic task into a deterministic planner that records objective hash, ordered task graph, dependency edges, ready-set derivation, completion accounting, and plan revision lineage. Add replay tests for stable ordering, dependency blocking, task completion progression, plan tamper rejection, and policy/context-informed decomposition without letting the LLM approve its own plan.

**Why this is second:** GOAL.md requires objectives to be decomposed into ordered tasks before observation, context, memory, policy, evaluation, learning, and orchestration can compound effectively. Better planning improves intelligence and empowerment while reducing repeated LLM work.

### 3. [ ] Add real artifact-backed semantic verification profiles

```text
expected_delta = max(C, A, R, T, B, F)
rank = 3
```

Back the semantic verification seam with artifact-aware checkers for the most important produced artifacts: source patches, command receipts, TLog NDJSON, policy rows, verification proof rows, and distillation rows. Each checker should bind semantic input, observable output, content hash, proof hash, source event, and rejection reason into a replayable receipt. Add negative tests for missing files, mismatched hashes, malformed rows, stale lineage, invalid replay, and semantically empty outputs.

**Why this is third:** the current verifier records a strong typed boundary, but real trustworthiness requires inspecting artifacts beyond packet-level hashes. This closes the gap between hash validation and the semantic artifact checking required by GOAL.md.

## Explicitly Already Represented In Prior Plan

- [x] Re-score current repository state after recent source-code improvements.
- [x] Rewrite this plan from current repository evidence.
- [x] Add a root deterministic validation harness.
- [x] Restore root graph telemetry proof without committing generated graphs.
- [x] Add bounded observation ingress with cursor persistence and backpressure.
- [ ] Split provider clients behind a small shared transport/receipt core.
- [ ] Implement a verified evolution candidate ledger and selection record.
- [ ] Add a gated `distill.jsonl` exporter from verified TLog receipts.

The unchecked items above remain important, but they were already addressed by the previous plan. The ranked tasks in this plan are the highest-leverage weaknesses found in this evaluation turn that were not already covered there.

## Validation For This Eval-Only Turn

- [x] Source code unchanged.
- [x] `GOAL.md` unchanged.
- [x] `bootstrap_rustc_session.py` unchanged.
- [x] Generated/runtime files unchanged.
- [x] `score.md` updated with current values only.
- [x] `plan.md` rewritten with ranked unchecked tasks.
- [x] `cargo -Znext-lockfile-bump run --bin root_validate --locked` passed after widening the validation receipt.
