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
base_commit = f3d5cffa01710aa75e16166fc1c70d03b0c4deff
restored_head_before_plan = f3d5cffa01710aa75e16166fc1c70d03b0c4deff
bundle_output = /mnt/data/repo-delta-001.bundle
manifest_output = /mnt/data/DELTA_MANIFEST.md
allowed_mutation = plan.md only
```

## Inputs Read

- [x] `GOAL.md`
- [x] `score.md`
- [x] `git log --oneline -20`
- [x] Current `plan.md`
- [x] Repository structure and validation surface

## Current State

- [x] The goal is not only a deterministic runtime; it explicitly requires a verified evolution loop, program database, TLog distillation, gated reuse, and policy/student learning path.
- [x] The scorecard baseline still shows the weakest axes as `Si = 5.0`, `P = 5.2`, `E = 5.8`, `R = 5.8`, and `C = 6.0`.
- [x] The latest commits already emphasize recovery, replay, retry ledgers, deterministic timing, and goodness-kernel scoring.
- [x] The old plan's remaining work covers toolchain formatting/linting, graph telemetry, live endpoint exercise, and large-module simplification.
- [x] The three tasks below are therefore selected because they are high-leverage goal gaps not already represented in the old plan.

## Ranked Tasks By Expected Score Delta

### 1. [ ] Implement a verified evolution candidate ledger and selection record

```text
expected_delta = max(L, I, T, D, C, R)
rank = 1
```

Add a typed record path for `seed program → candidate patch → sandbox run → evaluator → fitness score → program database → winner selection`. The next patch should create source-backed records plus tests proving that a candidate cannot enter the winning set unless evaluator evidence, replay validity, score improvement, and lineage hashes all bind to the same candidate.

**Why this is first:** `GOAL.md` makes verified evolution the central improvement loop, but current code history is stronger on replay/recovery than on candidate lineage and winner selection. This raises learning, intelligence, transparency, determinism, correctness, and robustness at once.

### 2. [ ] Add a gated `distill.jsonl` exporter from verified TLog receipts

```text
expected_delta = max(L, F, T, C, A, E)
rank = 2
```

Create a deterministic extractor that emits only rows satisfying `eval.verdict = pass`, `replay.valid = true`, `score >= threshold`, retained inputs, and semantically inspectable outputs. Each row must include `(instruction, input_state, action, output, score, proof_hash, source_event)` with tests rejecting missing proof hashes, failed replay, low scores, and uninspectable outputs.

**Why this is second:** the repo has policy-promotion work, but not the full training-data boundary the goal requires. This converts proven work into reusable learning data without letting model self-review become authority.

### 3. [ ] Restore the missing root validation and delta-manifest harness

```text
expected_delta = max(T, Em, R, C, E, Co)
rank = 3
```

Add the root scripts referenced by `README.md`: an `observe_validation` harness and `write_delta_manifest` path that produce machine-readable evidence for git hygiene, base-to-head mutation scope, Rust checks, wrapper graph presence, runtime archive signals, live/local LLM status, and bundle verification. Tests should prove the manifest fails closed when required evidence is absent.

**Why this is third:** the documentation references validation scripts that are not present in the restored tree. Rebuilding this harness directly improves operator empowerment, transparency, robustness, correctness, efficiency, and collaboration while making future one-shot turns easier to verify.

## Explicitly Not Selected This Turn

- [ ] Add `rustfmt` and `clippy` toolchain support.
- [ ] Generate root wrapper graph telemetry under `state/rustc/*/graph.json`.
- [ ] Exercise live Ollama/OpenAI-compatible paths against an available endpoint.
- [ ] Reduce large-module simplicity debt in `src/lib.rs`, `openai.rs`, and `ollama.rs`.

These are useful but were already represented in the prior plan, so they are not counted as the three highest-leverage weaknesses newly identified in this evaluation turn.

## Validation For This Plan-Only Turn

- [x] Source code unchanged.
- [x] `GOAL.md` unchanged.
- [x] `score.md` unchanged.
- [x] Generated/runtime files unchanged.
- [x] `plan.md` is the only intended repository mutation.
