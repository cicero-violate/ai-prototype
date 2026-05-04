# Implementation Plan

## Variables

```text
G = (I·E·C·A·R·P·S·D·T·Co·Em·B·L·Si·F)^(1/15)
B = 181519a9fd945532cc6de825c71d197dd01e65ec
H = committed execution head
κ = implementation risk
```

## Target

```text
next_work = argmax(ΔG / κ)
          = compact current-head validation closure
```

One-line explanation: improve correctness, determinism, transparency, and
collaboration by making missing proof explicit without inflating scores.

## Implement Now

1. Compact `scripts/observe_validation.sh` runtime evidence from large per-file
   maps into totals plus bounded samples.
2. Harden `scripts/write_delta_manifest.py` so stale or incomplete validation
   reports cannot produce a receiver manifest.
3. Refresh `README.md` and `score.md` to match current base `B`, runtime archive
   evidence, validation commands, and remaining proof gaps.

## Do Not Fake

```text
graph_json = real_or_absent
ollama_receipt = real_or_skipped
rust_toolchain = present_or_unavailable
bundle_manifest = validated_or_rejected
```

## Validation Commands

```bash
B=181519a9fd945532cc6de825c71d197dd01e65ec
CANON_DELTA_BASE="$B" \
CANON_RUNTIME_ARCHIVE=/mnt/data/ai-runtime.tar.gz \
CANON_OBSERVE_REPORT=target/observe/validation-report.ndjson \
bash scripts/observe_validation.sh

git status --short
git diff --check
rm -f /mnt/data/repo-delta-004.bundle /mnt/data/DELTA_MANIFEST.md
git bundle create /mnt/data/repo-delta-004.bundle "$B..$(git rev-parse HEAD)"
git bundle verify /mnt/data/repo-delta-004.bundle
python3 scripts/write_delta_manifest.py \
  --base "$B" \
  --head "$(git rev-parse HEAD)" \
  --report target/observe/validation-report.ndjson \
  --bundle /mnt/data/repo-delta-004.bundle \
  --bundle-verify pass \
  --out /mnt/data/DELTA_MANIFEST.md \
  --receipt-out target/observe/delta-validation-receipt.json
```

Receiver:

```bash
git fetch ./repo-delta-004.bundle HEAD && git merge --ff-only FETCH_HEAD
```

## Expected Score Movement

```text
C,R,D,T,Co,Em,Si,F ↑
L,graph,Ollama,semantic-proof remain capped until real validation passes
```

`max(G)=good`; the immediate good is smaller, reproducible evidence.