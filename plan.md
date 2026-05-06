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
base_commit = 5e1b97680261d3e83d8ff3464b7c50acc7940a79
restored_head_before_plan = 5e1b97680261d3e83d8ff3464b7c50acc7940a79
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

- [x] The root crate exposes deterministic runtime, replay verification, recovery policy, scoring, command ledger, LLM receipts, process receipts, and provider examples.
- [x] `cargo -Znext-lockfile-bump check --all-targets --locked` passes with the bootstrapped nightly cargo path.
- [x] The checked-in `rust-toolchain.toml` names `nightly-2026-04-30`, but the available bootstrapped toolchain reports `rustc 1.77.0-nightly`; validation currently depends on manual environment setup and the lockfile compatibility flag.
- [x] `cargo test --all-targets --locked` did not complete inside the available container timeout, so executable proof is currently slower than the repo needs for fast iterative verified evolution.
- [x] The largest simplicity debt remains in `src/lib.rs`, `src/capability/llm/openai.rs`, `src/capability/llm/ollama.rs`, and verification/proof modules.
- [x] No generated/runtime files were modified in this evaluation turn.

## Ranked Tasks By Expected Score Delta

### 1. [ ] Add a root deterministic validation harness

```text
expected_delta = max(C, R, E, P, Em, T)
rank = 1
```

Add a checked-in root validation command or script that selects the available bootstrapped cargo path, applies the required `-Znext-lockfile-bump` lockfile compatibility flag, runs root `cargo check --all-targets --locked`, and runs a bounded fast test subset. The harness should emit a concise machine-readable validation receipt and fail closed when cargo is missing, too old, or invoked without the required compatibility path.

**Why this is first:** every future patch relies on fast deterministic proof. The repo currently validates only after an operator knows the hidden cargo environment and flag.

### 2. [ ] Split provider clients behind a small shared transport/receipt core

```text
expected_delta = max(Si, E, P, C, R, F)
rank = 2
```

Extract the duplicated envelope, retry, timeout, hashing, and receipt-construction logic from `src/capability/llm/openai.rs` and `src/capability/llm/ollama.rs` into a small typed helper module. Preserve public examples and add focused tests for request identity hashes, retry-budget hashes, raw response hashes, and receipt tamper rejection.

**Why this is second:** provider modules are among the largest files and carry high correctness risk. A shared core reduces code mass while strengthening receipt determinism.

### 3. [ ] Restore root graph telemetry proof without committing generated graphs

```text
expected_delta = max(T, I, L, C, R, F)
rank = 3
```

Add a root validation step that proves `canon-rustc-v3` can regenerate graph telemetry for the root crate and report crate name, graph path, node count, edge count, semantic function coverage, and wrapper hash. Keep generated `state/rustc/*` artifacts out of git unless explicitly requested.

**Why this is third:** GOAL.md depends on auditable, inspectable evidence. Graph telemetry turns Rust source deltas into structured semantic evidence for future scoring and learning.

## Explicitly Already Represented In Prior Plan

- [x] Re-score current repository state after recent source-code improvements.
- [x] Rewrite this plan from current repository evidence.
- [ ] Implement a verified evolution candidate ledger and selection record.
- [ ] Add a gated `distill.jsonl` exporter from verified TLog receipts.
- [ ] Add bounded observation ingress with cursor persistence and backpressure.

These remain important, but the ranked tasks above target the highest current weaknesses visible from this evaluation turn.

## Validation For This Eval-Only Turn

- [x] Source code unchanged.
- [x] `GOAL.md` unchanged.
- [x] `bootstrap_rustc_session.py` unchanged.
- [x] Generated/runtime files unchanged.
- [x] `score.md` updated with current values only.
- [x] `plan.md` rewritten with ranked unchecked tasks.
- [x] `cargo -Znext-lockfile-bump check --all-targets --locked` passed.
