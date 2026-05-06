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
base_commit = 449ca9f823e7ed2e24e5d49a5bf1a467d04391ac
restored_head_before_plan = 449ca9f823e7ed2e24e5d49a5bf1a467d04391ac
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

- [x] The codebase now has stronger deterministic runtime, replay, recovery, policy, evidence, and scoring surfaces than the stale baseline score reflected.
- [x] `cargo check --all-targets --locked` passes only with nightly cargo plus `-Znext-lockfile-bump`; the default bootstrap Rust 1.75 path cannot parse the lockfile v4.
- [x] `README.md` still references root validation and delta-manifest scripts under `scripts/`, but no root `scripts/` directory is present in the restored tree.
- [x] The prior plan already names verified evolution, distillation, and manifest restoration, so the ranked tasks below focus on high-leverage gaps not yet represented as primary plan items.
- [x] The weakest current axes remain `P`, `Si`, `E`, `R`, and `C`; next work should raise those without weakening determinism.

## Ranked Tasks By Expected Score Delta

### 1. [ ] Canonicalize validation toolchain compatibility

```text
expected_delta = max(C, R, E, P, Em, T)
rank = 1
```

Make the repository validate from a single documented command path that works with the checked-in lockfile. Either align bootstrap/toolchain support with lockfile v4 or add an explicit root validation shim that selects the required nightly `cargo -Znext-lockfile-bump` path deterministically. Tests or script assertions should fail closed when the wrong cargo version is used.

**Why this is first:** validation is the gate for every future score-improvement turn. A repo that requires hidden cargo flags leaks operator time, lowers reproducibility, and weakens confidence in all subsequent patches.

### 2. [ ] Add root wrapper graph telemetry regeneration

```text
expected_delta = max(T, C, I, R, L, F)
rank = 2
```

Add a deterministic root validation step that proves `canon-rustc-v3` can emit or refresh `state/rustc/*/graph.json` for the root crate without committing generated graph files. The validation evidence should include crate name, graph path, node count, edge count, semantic function coverage, and wrapper version/hash.

**Why this is second:** the goal depends on replayable, inspectable evidence. Graph telemetry connects Rust source changes to semantic structure, making future evaluation and learning less dependent on free-form model judgment.

### 3. [ ] Reduce large-module simplicity debt with semantic-preserving splits

```text
expected_delta = max(Si, P, E, C, Co, F)
rank = 3
```

Split the largest runtime and provider modules into smaller typed units without changing behavior. Preserve public APIs, add regression tests around replay, recovery, receipt verification, and provider envelopes, and prefer pure helper functions over broad rewrites.

**Why this is third:** simplicity remains the lowest score axis. Smaller modules reduce review friction, improve compile/test targeting, and make verified evolution candidates easier to isolate and score.

## Explicitly Already Represented In Prior Plan

- [ ] Implement a verified evolution candidate ledger and selection record.
- [ ] Add a gated `distill.jsonl` exporter from verified TLog receipts.
- [ ] Restore the missing root validation and delta-manifest harness.

These remain important, but they were already primary items in the previous plan and therefore are not counted as the three newly identified weaknesses for this evaluation turn.

## Validation For This Eval-Only Turn

- [x] Source code unchanged.
- [x] `GOAL.md` unchanged.
- [x] `bootstrap_rustc_session.py` unchanged.
- [x] Generated/runtime files unchanged.
- [x] `score.md` updated with current values only.
- [x] `plan.md` rewritten with ranked unchecked tasks.
