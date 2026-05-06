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
base_commit = 8a60ea0
restored_head_before_plan = 8a60ea0
bundle_output = /mnt/data/repo-delta-001.bundle
manifest_output = /mnt/data/DELTA_MANIFEST.md
allowed_mutation = score.md and plan.md only
```

## Inputs Read

- [x] `GOAL.md`
- [x] `score.md`
- [x] `git log --oneline -20`
- [x] Current `plan.md`
- [x] Repository structure and validation surface

## Current State

- [x] The root crate exposes deterministic runtime, replay verification, recovery policy, scoring, command ledger, LLM receipts, process receipts, provider examples, a root validation harness, and graph telemetry validation receipt generation.
- [x] `cargo -Znext-lockfile-bump run --bin root_validate --locked` passes and covers root cargo check, bounded score contract tests, and canon-rustc-v3 graph telemetry evidence.
- [x] The checked-in `rust-toolchain.toml` still names `nightly-2026-04-30`; current validation succeeds through available cargo with `-Znext-lockfile-bump` rather than a fully self-contained toolchain bootstrap.
- [x] OpenAI and Ollama provider clients remain large duplicated modules, creating simplicity, correctness, and future-proofing debt.
- [x] The GOAL.md verified evolution loop is not yet backed by a durable candidate database, lineage ledger, or deterministic winner selection record.
- [x] Verified TLog distillation into training-ready `distill.jsonl` rows is still represented as a goal and validation expectation, not a gated exporter.
- [x] No source code, `GOAL.md`, `bootstrap_rustc_session.py`, generated files, or runtime files were modified in this evaluation turn.

## Ranked Tasks By Expected Score Delta

### 1. [ ] Split provider clients behind a small shared transport/receipt core

```text
expected_delta = max(Si, E, P, C, R, F)
rank = 1
```

Extract duplicated request envelope, retry, timeout, hashing, response parsing, and receipt-construction logic from `src/capability/llm/openai.rs` and `src/capability/llm/ollama.rs` into a typed helper module. Preserve public examples and add focused tests for request identity hashes, retry-budget hashes, raw response hashes, replay verification, and receipt tamper rejection.

**Why this is first:** provider modules are among the largest files and are structurally similar. A shared core should reduce code mass while improving deterministic receipt coverage.

### 2. [ ] Implement a verified evolution candidate ledger and selection record

```text
expected_delta = max(I, L, T, C, D, F)
rank = 2
```

Add an append-only candidate ledger for patch identity, parent lineage, sandbox command receipts, evaluator scores, replay verdicts, and selection decisions. Ensure the LLM can propose candidates but cannot approve them; only external validation receipts and measured score deltas can mark a winner.

**Why this is second:** GOAL.md centers the AlphaEvolve-style loop, but current evidence mostly validates runtime behavior rather than durable candidate evolution and winner selection.

### 3. [ ] Add a gated `distill.jsonl` exporter from verified TLog receipts

```text
expected_delta = max(L, E, B, T, C, A)
rank = 3
```

Create a bounded exporter that emits training-ready rows only when the source event has pass verdict, valid replay, retained semantic inputs, inspectable outputs, measured score, proof hash, and source event lineage. Add rejection tests for failed evals, invalid replay, missing inputs, missing outputs, and hash/lineage mismatches.

**Why this is third:** the project goal depends on reducing LLM cost over time by converting verified wins into policy or student-model data. Without a gated exporter, learning remains mostly aspirational.

## Explicitly Already Represented In Prior Plan

- [x] Re-score current repository state after recent source-code improvements.
- [x] Rewrite this plan from current repository evidence.
- [x] Add a root deterministic validation harness.
- [x] Restore root graph telemetry proof without committing generated graphs.
- [ ] Add bounded observation ingress with cursor persistence and backpressure.

These remain important, but the ranked tasks above target the highest current weaknesses visible from this evaluation turn that are not already addressed.

## Validation For This Eval-Only Turn

- [x] Source code unchanged.
- [x] `GOAL.md` unchanged.
- [x] `bootstrap_rustc_session.py` unchanged.
- [x] Generated/runtime files unchanged.
- [x] `score.md` updated with current values only.
- [x] `plan.md` rewritten with ranked unchecked tasks.
- [x] `cargo -Znext-lockfile-bump run --bin root_validate --locked` passed.
