# Implementation Plan

## Variables

```text
I,E,C,A,R,P,S,D,T,Co,Em,B,L,Si,F = score dimensions
G = (I·E·C·A·R·P·S·D·T·Co·Em·B·L·Si·F)^(1/15)
B = base commit
H = committed head
κ = implementation risk
ΔG = expected score lift
```

## Equation

```text
next_work = argmax(ΔG / κ)
          = portable current-head validation closure
```

One-line explanation: close reproducible validation gaps before claiming deeper autonomy gains.

## Source Evidence

- `GOAL.md` targets a frozen deterministic kernel, append-only TLog, replayable evidence, bounded recovery, policy learning, LLM promotion, and cheaper repeated reasoning.
- `score.md` shows the dominant weakness is not architecture; it is missing current-head validation for Rust, graph telemetry, Ollama judgment, semantic artifact verification, and policy-learning replay.
- Existing router offline validation is useful but insufficient because it does not prove the root Rust crate or live CDP/API behavior.

## Highest-Impact Target

Implement one portable observe command that records:

1. working-tree diff hygiene
2. base-to-head diff hygiene
3. router offline validation
4. cargo fmt/test/clippy pass/fail/unavailable state
5. configured rustc-wrapper presence and override use
6. runtime archive/download/cache/log evidence
7. graph telemetry presence/absence
8. Ollama judgment run or explicit skip reason
9. missing validation flags
10. manifest/receipt fields needed for safe delta output

## Mutation Scope

Primary files:

```text
scripts/observe_validation.sh
scripts/write_delta_manifest.py
README.md
score.md
IMPLEMENTATION_PLAN.md
```

No kernel, runtime, API, router, or capability semantic rewrite is justified unless validation exposes a concrete defect.

## Safety Rules

```text
no_fake_graph_json = true
no_fake_ollama_receipt = true
no_test_relaxation = true
no_generated_bundle_committed = true
no_token_or_signed_url_cache_committed = true
no_delta_without_manifest = true
```

## Validation Commands

```bash
B=fae4c60c6fae6177f92837119930e412c9d02e65
CANON_DELTA_BASE="$B" \
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz \
CANON_OBSERVE_REPORT=target/observe/validation-report-execute.ndjson \
bash scripts/observe_validation.sh

git status --short
git diff --check
git bundle create /mnt/data/repo-delta-004.bundle "$B..$(git rev-parse HEAD)"
git bundle verify /mnt/data/repo-delta-004.bundle
python3 scripts/write_delta_manifest.py \
  --base "$B" \
  --head "$(git rev-parse HEAD)" \
  --report target/observe/validation-report-execute.ndjson \
  --bundle /mnt/data/repo-delta-004.bundle \
  --bundle-verify pass \
  --out /mnt/data/DELTA_MANIFEST.md \
  --receipt-out target/observe/delta-validation-receipt.json
```

Receiver command:

```bash
git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD
```

## Expected Score Movement

```text
C,R,D,T,Em,F ↑ from reproducible validation closure
L,graph,Ollama,semantic-proof scores stay capped until those commands pass
```

`max(G)=good`; the immediate good is deterministic evidence, not inflated scores.